//! Database object views (tables / views / functions / roles / backup /
//! automation / model / bi).
//!
//! Plan v7 Phase 1.95b1 — objects.rs (2029줄) 를 폴더 구조로 변환. sub-modules
//! 는 현재 빈 placeholder. 실제 함수 cut-over 는 US-P1.95b2 (작은 함수
//! model/bi/automation) 와 US-P1.95b3 (큰 함수 tables/functions/roles/backup/
//! views) 에서 진행. Phase 2 가 Create/Replace UI 추가 시 sub-module 들의
//! `pub mod` 가시성 전환 검토 필요.

mod automation;
mod backup;
mod bi;
mod functions;
mod model;
mod roles;
mod tables;
mod views;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, ScrollArea, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{
    build_data_select_sql_with_columns, AppState, ConnectionStatus, DataSource, MainView,
};
use crate::storage::settings::AppSettings;
use crate::types::{
    ConnectionId,
};
use crate::ui::{icons_svg, theme};

// TableRow 가 Phase 1.95b3c 에서 src/ui/objects/tables.rs 로 cut-over.

#[derive(Clone, Copy)]
pub(super) enum ColumnSpec {
    Fixed(f32),
    Flex(f32),
}

pub(super) const TABLE_COLUMNS: [ColumnSpec; 7] = [
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Flex(1.5),
    ColumnSpec::Fixed(120.0),
    ColumnSpec::Fixed(80.0),
    ColumnSpec::Fixed(64.0),
    ColumnSpec::Fixed(64.0),
    ColumnSpec::Fixed(190.0),
];
pub(super) const FUNCTION_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Fixed(150.0),
    ColumnSpec::Flex(1.6),
    ColumnSpec::Fixed(150.0),
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Fixed(150.0),
];
pub(super) const ROLE_COLUMNS: [ColumnSpec; 5] = [
    ColumnSpec::Flex(1.2),
    ColumnSpec::Fixed(92.0),
    ColumnSpec::Flex(1.4),
    ColumnSpec::Fixed(132.0),
    ColumnSpec::Fixed(88.0),
];
pub(super) const BI_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::Flex(1.5),
    ColumnSpec::Fixed(170.0),
    ColumnSpec::Fixed(96.0),
    ColumnSpec::Fixed(112.0),
    ColumnSpec::Fixed(112.0),
    ColumnSpec::Fixed(112.0),
];

#[derive(Clone)]
pub(super) enum ObjectAction {
    ViewData {
        conn_id: ConnectionId,
        schema: String,
        name: String,
    },
    DesignTable {
        schema: String,
        name: String,
    },
    DropTable {
        conn_id: ConnectionId,
        schema: String,
        name: String,
        kind: crate::state::DropTargetKind,
    },
    CopySql(String),
    NewTable,
    OpenModel,
    AddAutomationQuery {
        title: String,
        sql: String,
    },
    AutomationCreate {
        title: String,
        sql: String,
        interval_secs: u64,
    },
    AutomationRunNow {
        id: uuid::Uuid,
        sql: String,
    },
    AutomationCancel {
        id: uuid::Uuid,
    },
    SelectTable {
        schema: String,
        name: String,
    },
    SelectFunction {
        schema: String,
        name: String,
    },
    SelectRole {
        name: String,
    },
}

pub fn render_objects_view(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
) {
    render_tabs(ui, state);
    let action = render_sub_toolbar(ui, state, bridge);
    if let Some(action) = action {
        handle_action(ui, state, bridge, action);
    }
    render_objects_list(ui, state, bridge, settings);
}

fn render_tabs(ui: &mut egui::Ui, state: &AppState) {
    let (title, subtitle, _color) = views::view_copy(state.active_main_view);
    let tab_frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(38.0);
        ui.horizontal(|ui| {
            let (icon, icon_name) = views::view_icon(state.active_main_view);
            crate::ui::icon_img(ui, icon, icon_name, 16.0);
            ui.add_space(6.0);
            ui.label(
                RichText::new(title)
                    .color(theme::text_primary())
                    .size(12.0)
                    .strong(),
            );
            ui.label(
                RichText::new(subtitle)
                    .color(theme::text_muted())
                    .size(11.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let status = match &conn.status {
                            ConnectionStatus::Connected { server_version } => {
                                format!("{}  PG {}", conn.config.display_name, server_version)
                            }
                            ConnectionStatus::Connecting => {
                                format!("{}  {}", conn.config.display_name, t("status_connecting"))
                            }
                            ConnectionStatus::Disconnected => {
                                format!(
                                    "{}  {}",
                                    conn.config.display_name,
                                    t("status_disconnected")
                                )
                            }
                        };
                        ui.label(RichText::new(status).color(theme::text_muted()).size(11.0));
                    }
                }
            });
        });
    });
}

fn render_sub_toolbar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let mut action = None;
    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            if icon_button(ui, icons_svg::REFRESH, "objects_refresh", t("ctx_refresh")).clicked() {
                refresh_current_view(state, bridge);
            }

            if matches!(state.active_main_view, MainView::Table)
                && icon_button(ui, icons_svg::PLUS, "objects_new", t("objects_new_table")).clicked()
            {
                action = Some(ObjectAction::NewTable);
            }

            if matches!(state.active_main_view, MainView::Model)
                && icon_button(
                    ui,
                    icons_svg::MODEL,
                    "objects_model",
                    t("objects_open_model"),
                )
                .clicked()
            {
                action = Some(ObjectAction::OpenModel);
            }

            ui.add_space(theme::SPACE_MD);
            render_schema_filter(ui, state);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    theme::text_input(&mut state.objects_search)
                        .desired_width(210.0)
                        .hint_text(t("objects_search")),
                );
            });
        });
    });

    action
}

fn render_schema_filter(ui: &mut egui::Ui, state: &mut AppState) {
    let schemas = active_conn(state)
        .and_then(|conn_id| state.connections.get(&conn_id))
        .map(|conn| conn.schemas.clone())
        .unwrap_or_default();

    if matches!(
        state.active_main_view,
        MainView::User | MainView::Backup | MainView::BI
    ) {
        return;
    }

    let selected_label = if state.objects_schema_filter.is_empty() {
        t("objects_all_schemas")
    } else {
        state.objects_schema_filter.clone()
    };
    let popup_id = ui.make_persistent_id("objects_schema_filter_popup");
    let response = schema_filter_button(ui, &selected_label, 180.0);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    let mut next_schema: Option<String> = None;
    let mut choose_all = false;
    show_dark_popup_below(ui, popup_id, &response, 220.0, theme::SPACE_SM_I, |ui| {
        if schema_filter_option(
            ui,
            &t("objects_all_schemas"),
            state.objects_schema_filter.is_empty(),
        )
        .clicked()
        {
            choose_all = true;
            ui.memory_mut(|memory| memory.close_popup());
        }

        ScrollArea::vertical()
            .max_height(220.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for schema in schemas {
                    if schema_filter_option(ui, &schema, state.objects_schema_filter == schema)
                        .clicked()
                    {
                        next_schema = Some(schema);
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                }
            });
    });

    if choose_all {
        state.objects_schema_filter.clear();
    }
    if let Some(schema) = next_schema {
        state.objects_schema_filter = schema;
    }
}

fn render_objects_list(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
) {
    egui::Frame::new().fill(theme::bg_dark()).show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        let action = match state.active_main_view {
            MainView::Table | MainView::View | MainView::MaterializedView => {
                tables::render_table_like_objects(ui, state, bridge)
            }
            MainView::Function => functions::render_functions(ui, state, bridge),
            MainView::User => roles::render_roles(ui, state, bridge),
            MainView::Backup => backup::render_backup_tools(ui, state, bridge, settings),
            MainView::Automation => automation::render_automation_tools(ui, state),
            // (signature 가 &mut state 로 변경됨 — Create form draft 입력 보존)
            MainView::Model => model::render_model_tools(ui, state, bridge),
            MainView::BI => bi::render_bi_tools(ui, state, bridge),
            MainView::Connection | MainView::Query | MainView::Data => None,
        };

        if let Some(action) = action {
            handle_action(ui, state, bridge, action);
        }
    });
}

// render_table_like_objects 가 Phase 1.95b3c 에서 src/ui/objects/tables.rs 로 cut-over.
// render_functions 가 Phase 1.95b3b 에서 src/ui/objects/functions.rs 로 cut-over.
// render_roles 가 Phase 1.95b3b 에서 src/ui/objects/roles.rs 로 cut-over.

// render_backup_tools 가 Phase 1.95b3d 에서 src/ui/objects/backup.rs 로 cut-over.

// render_backup_scope_card 가 backup.rs 로 cut-over.

// render_backup_repository_card / backup_format_button / render_backup_history /
// render_backup_record / open_backup_folder / format_size 모두 backup.rs 로 cut-over.

// render_automation_tools, render_model_tools, render_bi_tools 가 Phase 1.95b2 에서
// src/ui/objects/{automation, model, bi}.rs 로 cut-over 됨.

// render_table_row 가 tables.rs 로 cut-over.
// render_function_row 가 functions.rs 로 cut-over.
// render_role_row 가 roles.rs 로 cut-over.

// collect_table_rows 가 tables.rs 로 cut-over.
// collect_functions 가 functions.rs 로 cut-over.
// collect_roles 가 roles.rs 로 cut-over.

// request_missing_tables 가 tables.rs 로 cut-over.
// request_missing_functions 가 functions.rs 로 cut-over.
// request_roles 가 roles.rs 로 cut-over.

fn refresh_current_view(state: &mut AppState, bridge: &DbBridge) {
    let Some(conn_id) = active_conn(state) else {
        return;
    };
    let schemas = selected_schemas(state);

    match state.active_main_view {
        MainView::Table
        | MainView::View
        | MainView::MaterializedView
        | MainView::Model
        | MainView::BI => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                for schema in &schemas {
                    conn.tables.remove(schema);
                    conn.loading_tables.insert(schema.clone());
                }
            }
            for schema in schemas {
                bridge.send(DbCommand::ListTables { conn_id, schema });
            }
        }
        MainView::Function => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                for schema in &schemas {
                    conn.functions.remove(schema);
                    conn.loading_functions.insert(schema.clone());
                }
            }
            for schema in schemas {
                bridge.send(DbCommand::ListFunctions { conn_id, schema });
            }
        }
        MainView::User => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                conn.roles.clear();
                conn.loading_roles = true;
            }
            bridge.send(DbCommand::ListRoles { conn_id });
        }
        _ => {}
    }
}

/// 자동화 작업 목록을 디스크에 스냅샷 (add/remove/mark_run 직후 호출).
pub fn persist_automation(state: &AppState) {
    if let Ok(store) = state.automation.read() {
        crate::storage::automation::save_tasks(&store.all_tasks());
    }
}

fn handle_action(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge, action: ObjectAction) {
    match action {
        ObjectAction::ViewData {
            conn_id,
            schema,
            name,
        } => {
            state.active_connection = Some(conn_id);
            state.current_result = None;
            state.current_result_truncated = false;
            state.begin_data_edit(conn_id, &schema, &name);
            tables::request_table_columns_for_editing(state, bridge, conn_id, &schema, &name);
            let source = DataSource {
                conn_id,
                schema: schema.clone(),
                table: name.clone(),
                filter: None,
            };
            let limit = state.data_edit.page_limit;
            let columns = state.data_columns_for_source(&source);
            let sql = build_data_select_sql_with_columns(
                &source,
                &state.data_edit.sort,
                limit,
                0,
                &columns,
            );
            bridge.send(DbCommand::ExecuteQuery {
                conn_id,
                sql,
                row_limit: Some(limit),
            });
            state.query_running = true;
            state.open_workspace_view(MainView::Data, format!("{schema}.{name}"), schema, name);
        }
        ObjectAction::DesignTable { schema, name } => {
            crate::ui::table_designer::open_for_existing_table(state, &schema, &name, bridge);
        }
        ObjectAction::DropTable {
            conn_id,
            schema,
            name,
            kind,
        } => {
            // US-K1/L1 — Drop 다이얼로그 표시 + dependents fetch trigger.
            // table 의 oid 를 conn.tables 에서 조회 → FetchDependents 발사.
            let oid = state
                .connections
                .get(&conn_id)
                .and_then(|c| c.tables.get(&schema))
                .and_then(|tables| tables.iter().find(|t| t.name == name))
                .and_then(|t| t.oid);
            state.drop_dialog = Some(crate::state::DropDialogState::new(
                conn_id, &schema, &name, kind,
            ));
            if let Some(refobjid) = oid {
                bridge.send(crate::db::bridge::DbCommand::FetchDependents {
                    conn_id,
                    refobjid,
                });
            } else if let Some(dlg) = state.drop_dialog.as_mut() {
                dlg.loading = false; // oid 없음 — fetch 불가
            }
        }
        ObjectAction::CopySql(sql) => {
            ui.ctx().copy_text(sql);
            state.status_message = "Copied to clipboard".to_string();
        }
        ObjectAction::NewTable => {
            if state.objects_schema_filter.is_empty() {
                crate::ui::table_designer::open_for_new_table(state);
            } else {
                let schema = state.objects_schema_filter.clone();
                crate::ui::table_designer::open_for_new_table_with_schema(state, &schema);
            }
        }
        ObjectAction::OpenModel => {
            let schema = selected_schema_or_public(state);
            state.er_diagram.selected_schema = schema.clone();
            state.open_workspace_view(MainView::Model, format!("Model: {schema}"), schema, "");
            state.er_diagram.show_diagram = true;
        }
        ObjectAction::AddAutomationQuery { title, sql } => {
            state.editor_tabs.push(crate::types::EditorTab::new(title));
            state.active_tab = state.editor_tabs.len() - 1;
            if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
                tab.content = sql;
                tab.connection_id = state.active_connection;
            }
            state.open_workspace_main_view(MainView::Query);
        }
        ObjectAction::AutomationCreate {
            title,
            sql,
            interval_secs,
        } => {
            use crate::automation::scheduler::{Schedule, ScheduledTask};
            let schedule = if interval_secs == 0 {
                Schedule::Once {
                    at: chrono::Utc::now(),
                }
            } else {
                Schedule::Interval {
                    period: std::time::Duration::from_secs(interval_secs),
                }
            };
            state
                .automation
                .write()
                .expect("automation lock poisoned")
                .add(ScheduledTask::new(title, sql, schedule));
            state.automation_draft.reset();
            persist_automation(state);
        }
        ObjectAction::AutomationRunNow { id, sql } => {
            if let Some(conn_id) = state.active_connection {
                bridge.send(crate::db::bridge::DbCommand::ExecuteAutomation {
                    conn_id,
                    task_id: id,
                    sql,
                });
            }
        }
        ObjectAction::AutomationCancel { id } => {
            state
                .automation
                .write()
                .expect("automation lock poisoned")
                .remove(id);
            persist_automation(state);
        }
        ObjectAction::SelectTable { schema, name } => {
            state.objects_selected_table = Some((schema.clone(), name.clone()));
            if let Some(conn_id) = state.active_connection {
                let key = (schema.clone(), name.clone());
                if let Some(conn) = state.connections.get_mut(&conn_id) {
                    if !conn.columns.contains_key(&key)
                        && !conn.loading_columns.contains(&key)
                    {
                        conn.loading_columns.insert(key.clone());
                        bridge.send(crate::db::bridge::DbCommand::ListColumns {
                            conn_id,
                            schema: schema.clone(),
                            table: name.clone(),
                        });
                    }
                    if !conn.indexes.contains_key(&key)
                        && !conn.loading_indexes.contains(&key)
                    {
                        conn.loading_indexes.insert(key.clone());
                        bridge.send(crate::db::bridge::DbCommand::ListIndexes {
                            conn_id,
                            schema: schema.clone(),
                            table: name.clone(),
                        });
                    }
                    if !conn.foreign_keys.contains_key(&schema)
                        && !conn.loading_foreign_keys.contains(&schema)
                    {
                        conn.loading_foreign_keys.insert(schema.clone());
                        bridge.send(crate::db::bridge::DbCommand::ListForeignKeys {
                            conn_id,
                            schema: schema.clone(),
                        });
                    }
                }
            }
        }
        ObjectAction::SelectFunction { schema, name } => {
            state.objects_selected_function = Some((schema, name));
        }
        ObjectAction::SelectRole { name } => {
            state.objects_selected_role = Some(name);
        }
    }
}

// request_table_columns_for_editing 가 tables.rs 로 cut-over.

pub(super) fn render_no_connection(ui: &mut egui::Ui) -> Option<ObjectAction> {
    empty_state(
        ui,
        &t("objects_no_active_connection"),
        &t("objects_no_active_connection_help"),
    );
    None
}

pub(super) fn empty_state(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        crate::ui::icon_img(ui, icons_svg::DATABASE, "objects_empty", 32.0);
        ui.add_space(theme::SPACE_MD);
        ui.label(RichText::new(title).color(theme::text_muted()).size(12.0));
        ui.label(
            RichText::new(subtitle)
                .color(theme::text_disabled())
                .size(11.0),
        );
    });
}

pub(super) fn table_header(ui: &mut egui::Ui, specs: &[ColumnSpec], headers: &[impl AsRef<str>]) {
    debug_assert_eq!(specs.len(), headers.len());

    object_row_frame(
        ui,
        specs,
        28.0,
        theme::bg_shell(),
        Sense::hover(),
        |cells| {
            for header in headers {
                cells.col(|ui| {
                    cell_label(ui, header.as_ref(), theme::text_muted(), 10.5, true);
                });
            }
        },
    );
}

pub(super) fn data_row(
    ui: &mut egui::Ui,
    specs: &[ColumnSpec],
    content: impl FnOnce(&mut RowCells<'_>),
) -> egui::Response {
    object_row_frame(ui, specs, 33.0, theme::bg_dark(), Sense::click(), |cells| {
        content(cells);
    })
}

pub(super) fn data_row_alt(
    ui: &mut egui::Ui,
    specs: &[ColumnSpec],
    row_index: usize,
    content: impl FnOnce(&mut RowCells<'_>),
) -> egui::Response {
    let fill = if row_index.is_multiple_of(2) {
        theme::bg_dark()
    } else {
        theme::bg_shell()
    };
    object_row_frame(ui, specs, 33.0, fill, Sense::click(), |cells| {
        content(cells);
    })
}

fn object_row_frame(
    ui: &mut egui::Ui,
    specs: &[ColumnSpec],
    height: f32,
    fill: Color32,
    sense: Sense,
    content: impl FnOnce(&mut RowCells<'_>),
) -> egui::Response {
    let natural_inner_width = natural_total_width(specs);
    let side_padding = theme::SPACE_LG * 2.0;
    let natural_width = natural_inner_width + side_padding;
    let available = ui.available_width().max(320.0);
    let full_width = available.max(natural_width);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(full_width, height), sense);
    let clip_rect = ui.clip_rect();
    let paint_rect = rect.intersect(clip_rect);
    if paint_rect.is_negative() {
        return response;
    }

    let fill = if response.hovered() {
        theme::with_alpha(theme::accent_color(), 14)
    } else {
        fill
    };

    let painter = ui.painter().with_clip_rect(paint_rect);
    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_SM), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(1.0, theme::border_subtle()),
        egui::StrokeKind::Inside,
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, 0.0));
    let widths = resolve_column_widths(specs, inner.width());
    let mut cells = RowCells {
        ui,
        rect: inner,
        widths,
        index: 0,
        cursor_x: inner.left(),
        clip_rect,
    };
    content(&mut cells);

    response
}

pub(super) struct RowCells<'a> {
    ui: &'a mut egui::Ui,
    rect: egui::Rect,
    widths: Vec<f32>,
    index: usize,
    cursor_x: f32,
    clip_rect: egui::Rect,
}

impl RowCells<'_> {
    pub(super) fn col(&mut self, content: impl FnOnce(&mut egui::Ui)) {
        let Some(width) = self.widths.get(self.index).copied() else {
            return;
        };
        let col_rect = egui::Rect::from_min_size(
            egui::pos2(self.cursor_x, self.rect.top()),
            egui::vec2(width, self.rect.height()),
        )
        .shrink2(egui::vec2(3.0, 0.0));
        let col_clip = col_rect.intersect(self.clip_rect);

        self.ui.allocate_new_ui(
            egui::UiBuilder::new()
                .max_rect(col_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(col_clip);
                ui.set_min_height(col_rect.height());
                content(ui);
            },
        );

        self.cursor_x += width + theme::SPACE_MD;
        self.index += 1;
    }
}

fn natural_total_width(specs: &[ColumnSpec]) -> f32 {
    let gap = theme::SPACE_MD * specs.len().saturating_sub(1) as f32;
    let fixed_sum: f32 = specs
        .iter()
        .map(|spec| match spec {
            ColumnSpec::Fixed(width) => *width,
            ColumnSpec::Flex(_) => 0.0,
        })
        .sum();
    let flex_min = 120.0;
    let flex_count = specs
        .iter()
        .filter(|spec| matches!(spec, ColumnSpec::Flex(_)))
        .count() as f32;
    fixed_sum + flex_count * flex_min + gap
}

fn resolve_column_widths(specs: &[ColumnSpec], available_width: f32) -> Vec<f32> {
    let gap = theme::SPACE_MD * specs.len().saturating_sub(1) as f32;
    let usable = (available_width - gap).max(1.0);
    let fixed_sum = specs
        .iter()
        .map(|spec| match spec {
            ColumnSpec::Fixed(width) => *width,
            ColumnSpec::Flex(_) => 0.0,
        })
        .sum::<f32>();
    let flex_weight = specs
        .iter()
        .map(|spec| match spec {
            ColumnSpec::Fixed(_) => 0.0,
            ColumnSpec::Flex(weight) => *weight,
        })
        .sum::<f32>();
    let flex_count = specs
        .iter()
        .filter(|spec| matches!(spec, ColumnSpec::Flex(_)))
        .count() as f32;
    let min_flex_width = 72.0;
    let required = fixed_sum + flex_count * min_flex_width;

    if required > usable {
        let scale = (usable / required).clamp(0.45, 1.0);
        return specs
            .iter()
            .map(|spec| match spec {
                ColumnSpec::Fixed(width) => width * scale,
                ColumnSpec::Flex(_) => min_flex_width * scale,
            })
            .collect();
    }

    let remaining = (usable - fixed_sum).max(0.0);
    specs
        .iter()
        .map(|spec| match spec {
            ColumnSpec::Fixed(width) => *width,
            ColumnSpec::Flex(weight) if flex_weight > 0.0 => remaining * (*weight / flex_weight),
            ColumnSpec::Flex(_) => min_flex_width,
        })
        .collect()
}

pub(super) fn cell_label(ui: &mut egui::Ui, text: &str, color: Color32, size: f32, strong: bool) {
    let mut text = RichText::new(text).color(color).size(size);
    if strong {
        text = text.strong();
    }
    ui.add(egui::Label::new(text).truncate());
}

pub(super) fn render_count_strip(ui: &mut egui::Ui, count: usize, label: &str) {
    ui.horizontal(|ui| {
        ui.add_space(theme::SPACE_LG);
        ui.label(
            RichText::new(format!("{count} {label}"))
                .color(theme::text_muted())
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_SM);
}

pub(super) fn code_line(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::same(theme::SPACE_MD as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .font(egui::FontId::monospace(11.0))
                    .color(theme::text_secondary()),
            );
        });
}

pub(super) fn type_chip(ui: &mut egui::Ui, label: &str, color: Color32) {
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), egui::FontId::monospace(10.5), color);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(galley.rect.width() + 12.0, 19.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        theme::with_alpha(color, 26),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(10.5),
        color,
    );
}

fn icon_button(ui: &mut egui::Ui, svg: &str, name: &str, tooltip: String) -> egui::Response {
    const BUTTON_SIZE: egui::Vec2 = egui::vec2(28.0, 28.0);
    const ICON_SIZE: f32 = 13.0;

    let (rect, response) = ui.allocate_exact_size(BUTTON_SIZE, Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::accent_color(), 150))
    } else {
        Stroke::new(1.0, theme::border_default())
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(ICON_SIZE, ICON_SIZE));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui,
                svg,
                name,
                ICON_SIZE,
                theme::accent_color(),
            ));
        },
    );

    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, &tooltip);
    response
}

fn schema_filter_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, theme::INPUT_HEIGHT), Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::accent_color(), 140))
    } else {
        Stroke::new(1.0, theme::border_default())
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );

    let text_rect = rect
        .shrink2(egui::vec2(theme::SPACE_MD, 0.0))
        .with_max_x(rect.right() - 30.0);
    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.5),
        theme::text_primary(),
    );

    let icon_rect = egui::Rect::from_center_size(
        rect.right_center() - egui::vec2(16.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icons_svg::CHEVRON_DOWN,
                "objects_schema_filter_chevron",
                12.0,
                theme::text_muted(),
            ));
        },
    );

    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn schema_filter_option(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let width = ui.available_width().max(180.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::accent_color(), 28)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }

    let text_color = if selected {
        theme::text_primary()
    } else {
        theme::text_secondary()
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(theme::SPACE_MD, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    if selected {
        ui.painter().circle_filled(
            rect.right_center() - egui::vec2(theme::SPACE_LG, 0.0),
            3.0,
            theme::accent_color(),
        );
    }

    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn show_dark_popup_below<R>(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    response: &egui::Response,
    min_width: f32,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    if !ui.memory(|memory| memory.is_popup_open(popup_id)) {
        return;
    }

    let mut pos = response.rect.left_bottom() + egui::vec2(0.0, 4.0);
    if let Some(to_global) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        pos = to_global * pos;
    }
    let popup = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(margin))
                .show(ui, |ui| {
                    ui.set_width(min_width);
                    add_contents(ui);
                });
        });

    let should_close = ui.input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && popup.response.clicked_elsewhere());
    if should_close {
        ui.memory_mut(|memory| memory.close_popup());
    }
}

pub(super) fn show_dark_hover_tooltip(
    ui: &egui::Ui,
    tooltip_id: egui::Id,
    response: &egui::Response,
    text: &str,
) {
    if !response.hovered() {
        return;
    }

    let pointer = ui
        .ctx()
        .pointer_hover_pos()
        .unwrap_or_else(|| response.rect.left_bottom());
    let max_width = 420.0;
    let pos = smart_tooltip_pos(ui.ctx(), pointer, estimate_tooltip_size(text, max_width));
    egui::Area::new(tooltip_id)
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_MD_I))
                .show(ui, |ui| {
                    ui.set_max_width(max_width);
                    ui.add(
                        egui::Label::new(
                            RichText::new(text)
                                .color(theme::text_secondary())
                                .monospace()
                                .size(11.0),
                        )
                        .wrap(),
                    );
                });
        });
}

fn smart_tooltip_pos(
    ctx: &egui::Context,
    anchor: egui::Pos2,
    estimated_size: egui::Vec2,
) -> egui::Pos2 {
    let bounds = ctx.screen_rect().shrink(8.0);
    let gap = 12.0;
    let right_x = anchor.x + gap;
    let left_x = anchor.x - gap - estimated_size.x;
    let bottom_y = anchor.y + gap;
    let top_y = anchor.y - gap - estimated_size.y;

    let x = if right_x + estimated_size.x <= bounds.right() {
        right_x
    } else if left_x >= bounds.left() {
        left_x
    } else {
        clamp_axis(right_x, bounds.left(), bounds.right() - estimated_size.x)
    };

    let y = if bottom_y + estimated_size.y <= bounds.bottom() {
        bottom_y
    } else if top_y >= bounds.top() {
        top_y
    } else {
        clamp_axis(bottom_y, bounds.top(), bounds.bottom() - estimated_size.y)
    };

    egui::pos2(x, y)
}

fn estimate_tooltip_size(text: &str, max_width: f32) -> egui::Vec2 {
    let char_width = 7.2;
    let content_max = (max_width - theme::SPACE_MD * 2.0).max(80.0);
    let mut visual_lines = 0.0_f32;
    let mut widest = 0.0_f32;

    for line in text.lines().chain((text.is_empty()).then_some("")) {
        let line_width = line.chars().count() as f32 * char_width;
        widest = widest.max(line_width);
        visual_lines += (line_width / content_max).ceil().max(1.0);
    }

    let width = (widest + theme::SPACE_MD * 2.0).clamp(48.0, max_width);
    let height = visual_lines * 15.0 + theme::SPACE_MD * 2.0;
    egui::vec2(width, height)
}

fn clamp_axis(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        min
    } else {
        value.clamp(min, max)
    }
}

fn set_pointing_cursor_on_hover(ui: &mut egui::Ui, response: &egui::Response, enabled: bool) {
    if enabled && response.hovered() {
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

pub(super) fn active_conn(state: &AppState) -> Option<ConnectionId> {
    let conn_id = state.active_connection?;
    let conn = state.connections.get(&conn_id)?;
    matches!(conn.status, ConnectionStatus::Connected { .. }).then_some(conn_id)
}

pub(super) fn selected_schemas(state: &AppState) -> Vec<String> {
    let Some(conn_id) = state.active_connection else {
        return Vec::new();
    };
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };

    if state.objects_schema_filter.is_empty() {
        conn.schemas.clone()
    } else {
        vec![state.objects_schema_filter.clone()]
    }
}

pub(super) fn selected_schema_or_public(state: &AppState) -> String {
    if !state.objects_schema_filter.is_empty() {
        return state.objects_schema_filter.clone();
    }
    state
        .active_connection
        .and_then(|id| state.connections.get(&id))
        .and_then(|conn| {
            conn.schemas
                .iter()
                .find(|schema| *schema == "public")
                .cloned()
        })
        .or_else(|| {
            state
                .active_connection
                .and_then(|id| state.connections.get(&id))
                .and_then(|conn| conn.schemas.first().cloned())
        })
        .unwrap_or_else(|| "public".to_string())
}

// matches_table_kind 가 tables.rs 로 cut-over.

// view_copy / view_icon / table_type_color 가 Phase 1.95b3 에서
// src/ui/objects/views.rs 로 cut-over 됨.

pub(super) fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

// argument_placeholders 가 Phase 1.95b3b 에서 functions.rs 로 cut-over.

pub(super) fn format_number(value: f64) -> String {
    if value.abs() >= 1000.0 || value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.3}")
    }
}
