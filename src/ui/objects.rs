use eframe::egui::{
    self, Color32, ComboBox, CornerRadius, Margin, RichText, ScrollArea, Sense, Stroke,
};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, AppState, ConnectionStatus, DataSource, MainView,
};
use crate::storage::settings::AppSettings;
use crate::types::{
    BackupFormat, BackupRecord, BackupRequest, CellValue, ConnectionConfig, ConnectionId,
    FunctionInfo, RoleInfo, TableInfo,
};
use crate::ui::{icons_svg, theme};

#[derive(Clone)]
struct TableRow {
    schema: String,
    name: String,
    table_type: String,
    column_count: Option<usize>,
    index_count: Option<usize>,
}

#[derive(Clone, Copy)]
enum ColumnSpec {
    Fixed(f32),
    Flex(f32),
}

const TABLE_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Flex(1.5),
    ColumnSpec::Fixed(132.0),
    ColumnSpec::Fixed(74.0),
    ColumnSpec::Fixed(74.0),
    ColumnSpec::Fixed(190.0),
];
const FUNCTION_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Fixed(150.0),
    ColumnSpec::Flex(1.6),
    ColumnSpec::Fixed(150.0),
    ColumnSpec::Fixed(86.0),
    ColumnSpec::Fixed(150.0),
];
const ROLE_COLUMNS: [ColumnSpec; 5] = [
    ColumnSpec::Flex(1.2),
    ColumnSpec::Fixed(92.0),
    ColumnSpec::Flex(1.4),
    ColumnSpec::Fixed(132.0),
    ColumnSpec::Fixed(88.0),
];
const BI_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::Flex(1.5),
    ColumnSpec::Fixed(170.0),
    ColumnSpec::Fixed(96.0),
    ColumnSpec::Fixed(112.0),
    ColumnSpec::Fixed(112.0),
    ColumnSpec::Fixed(112.0),
];

#[derive(Clone)]
enum ObjectAction {
    ViewData {
        conn_id: ConnectionId,
        schema: String,
        name: String,
    },
    DesignTable {
        schema: String,
        name: String,
    },
    CopySql(String),
    NewTable,
    OpenModel,
    AddAutomationQuery {
        title: String,
        sql: String,
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
    let (title, subtitle, _color) = view_copy(state.active_main_view);
    let tab_frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(38.0);
        ui.horizontal(|ui| {
            let (icon, icon_name) = view_icon(state.active_main_view);
            crate::ui::icon_img(ui, icon, icon_name, 16.0);
            ui.add_space(6.0);
            ui.label(
                RichText::new(title)
                    .color(theme::text_primary())
                    .size(13.0)
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
        .fill(theme::bg_dark())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
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

    ComboBox::from_id_salt("objects_schema_filter")
        .width(150.0)
        .selected_text(if state.objects_schema_filter.is_empty() {
            t("objects_all_schemas")
        } else {
            state.objects_schema_filter.clone()
        })
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(
                    state.objects_schema_filter.is_empty(),
                    t("objects_all_schemas"),
                )
                .clicked()
            {
                state.objects_schema_filter.clear();
            }
            for schema in schemas {
                if ui
                    .selectable_label(state.objects_schema_filter == schema, &schema)
                    .clicked()
                {
                    state.objects_schema_filter = schema;
                }
            }
        });
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
                render_table_like_objects(ui, state, bridge)
            }
            MainView::Function => render_functions(ui, state, bridge),
            MainView::User => render_roles(ui, state, bridge),
            MainView::Backup => render_backup_tools(ui, state, bridge, settings),
            MainView::Automation => render_automation_tools(ui, state),
            MainView::Model => render_model_tools(ui, state, bridge),
            MainView::BI => render_bi_tools(ui, state),
            MainView::Connection | MainView::Query | MainView::Data => None,
        };

        if let Some(action) = action {
            handle_action(ui, state, bridge, action);
        }
    });
}

fn render_table_like_objects(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_missing_tables(state, bridge, conn_id);
    let rows = collect_table_rows(state, conn_id);
    render_count_strip(ui, rows.len(), "objects");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_table_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &TABLE_COLUMNS,
                &[
                    t("objects_schema"),
                    t("objects_name"),
                    t("objects_type"),
                    t("objects_columns"),
                    t("objects_indexes"),
                    t("objects_actions"),
                ],
            );
            for row in rows {
                let row_action = render_table_row(ui, conn_id, &row);
                if row_action.is_some() {
                    action = row_action;
                }
            }
        });

    action
}

fn render_functions(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_missing_functions(state, bridge, conn_id);
    let rows = collect_functions(state, conn_id);
    render_count_strip(ui, rows.len(), "functions");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_function_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &FUNCTION_COLUMNS,
                &[
                    t("objects_schema"),
                    t("objects_name"),
                    t("objects_signature"),
                    t("objects_returns"),
                    t("objects_lang"),
                    t("objects_actions"),
                ],
            );
            for func in rows {
                let copied = render_function_row(ui, &func);
                if let Some(sql) = copied {
                    action = Some(ObjectAction::CopySql(sql));
                }
            }
        });
    action
}

fn render_roles(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_roles(state, bridge, conn_id);
    let rows = collect_roles(state, conn_id);
    render_count_strip(ui, rows.len(), "roles");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_role_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &ROLE_COLUMNS,
                &[
                    t("objects_role"),
                    t("objects_login"),
                    t("objects_privileges"),
                    t("objects_valid_until"),
                    t("objects_actions"),
                ],
            );
            for role in rows {
                if let Some(sql) = render_role_row(ui, &role) {
                    action = Some(ObjectAction::CopySql(sql));
                }
            }
        });
    action
}

fn render_backup_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
) -> Option<ObjectAction> {
    let Some(conn_id) = active_conn(state) else {
        return render_no_connection(ui);
    };
    let cfg = state.connections.get(&conn_id)?.config.clone();
    let schema =
        (!state.objects_schema_filter.is_empty()).then(|| state.objects_schema_filter.clone());

    ui.add_space(theme::SPACE_XL);
    render_backup_scope_card(ui, &cfg, schema.as_deref());
    ui.add_space(theme::SPACE_LG);
    render_backup_repository_card(
        ui,
        state,
        bridge,
        settings,
        conn_id,
        &cfg,
        schema.as_deref(),
    );
    ui.add_space(theme::SPACE_LG);
    render_backup_history(ui, state);
    None
}

fn render_backup_scope_card(ui: &mut egui::Ui, cfg: &ConnectionConfig, schema: Option<&str>) {
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::BACKUP, "backup_scope", 24.0);
                ui.add_space(theme::SPACE_SM);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(match schema {
                            Some(_) => t("backup_schema"),
                            None => t("backup_full_database"),
                        })
                        .color(theme::text_primary())
                        .size(15.0)
                        .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}  {}:{} / {}",
                            cfg.display_name, cfg.host, cfg.port, cfg.database
                        ))
                        .color(theme::text_muted())
                        .size(11.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(schema) = schema {
                        type_chip(ui, schema, theme::ACCENT_TEAL);
                    } else {
                        type_chip(ui, "FULL", theme::ACCENT_COPPER);
                    }
                });
            });
        });
}

fn render_backup_repository_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
    conn_id: ConnectionId,
    cfg: &ConnectionConfig,
    schema: Option<&str>,
) {
    let folder_set = !settings.backup_directory.trim().is_empty();
    let folder_label = (if folder_set {
        settings.backup_directory.clone()
    } else {
        t("backup_no_folder_selected")
    })
    .to_string();

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t("backup_folder_title"))
                            .color(theme::text_primary())
                            .size(15.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(folder_label)
                            .color(if folder_set {
                                theme::text_secondary()
                            } else {
                                theme::text_muted()
                            })
                            .size(11.0),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(theme::secondary_button(&t("backup_choose_folder")))
                        .clicked()
                    {
                        let mut dialog = rfd::FileDialog::new();
                        if folder_set {
                            dialog = dialog.set_directory(&settings.backup_directory);
                        }
                        if let Some(path) = dialog.pick_folder() {
                            settings.backup_directory = path.display().to_string();
                            crate::storage::settings::save_settings(settings);
                            state.status_message = t("backup_folder_updated");
                        }
                    }

                    if ui
                        .add_enabled(folder_set, theme::ghost_button(&t("backup_open_folder")))
                        .clicked()
                    {
                        open_backup_folder(&settings.backup_directory);
                    }
                });
            });

            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(t("backup_format"))
                        .color(theme::text_muted())
                        .size(11.0)
                        .strong(),
                );
                backup_format_button(ui, &mut state.backup_format, BackupFormat::Custom);
                backup_format_button(ui, &mut state.backup_format, BackupFormat::Plain);
            });

            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                let can_run = folder_set && !state.backup_running;
                let run_label = if state.backup_running {
                    t("backup_running_label")
                } else {
                    t("backup_run")
                };
                if ui
                    .add_enabled(can_run, theme::primary_button(&run_label))
                    .clicked()
                {
                    let request = BackupRequest {
                        conn_id,
                        config: cfg.clone(),
                        output_dir: std::path::PathBuf::from(&settings.backup_directory),
                        schema: schema.map(ToOwned::to_owned),
                        format: state.backup_format,
                    };
                    state.backup_running = true;
                    state.backup_last_error = None;
                    state.status_message = tf("backup_running_status", &[&cfg.display_name]);
                    bridge.send(DbCommand::RunBackup { request });
                }

                if state.backup_running {
                    ui.spinner();
                    ui.label(
                        RichText::new(t("backup_pg_dump_running"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                }
            });

            if let Some(error) = &state.backup_last_error {
                ui.add_space(theme::SPACE_MD);
                ui.label(RichText::new(error).color(theme::ACCENT_RED).size(11.0));
            }
        });
}

fn backup_format_button(ui: &mut egui::Ui, value: &mut BackupFormat, format: BackupFormat) {
    let selected = *value == format;
    let label = match format {
        BackupFormat::Custom => t("backup_custom_archive"),
        BackupFormat::Plain => t("backup_plain_sql"),
    };
    let button = egui::Button::new(RichText::new(label).color(if selected {
        Color32::WHITE
    } else {
        theme::text_secondary()
    }))
    .fill(if selected {
        theme::ACCENT_COPPER
    } else {
        theme::bg_light()
    })
    .stroke(Stroke::new(
        1.0,
        if selected {
            theme::ACCENT_COPPER_LIGHT
        } else {
            theme::border_default()
        },
    ))
    .corner_radius(CornerRadius::same(theme::RADIUS_MD));

    if ui.add(button).clicked() {
        *value = format;
    }
}

fn render_backup_history(ui: &mut egui::Ui, state: &AppState) {
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(t("backup_recent"))
                    .color(theme::text_primary())
                    .size(15.0)
                    .strong(),
            );
            ui.add_space(theme::SPACE_MD);

            if state.backup_history.is_empty() {
                ui.label(
                    RichText::new(t("backup_no_session"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                return;
            }

            for record in &state.backup_history {
                render_backup_record(ui, record);
            }
        });
}

fn render_backup_record(ui: &mut egui::Ui, record: &BackupRecord) {
    let response = egui::Frame::new()
        .fill(theme::bg_dark())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::BACKUP, "backup_record", 15.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(
                            record
                                .file_path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("backup"),
                        )
                        .color(theme::text_primary())
                        .size(12.0)
                        .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} / {} / {} / {}",
                            record.connection_name,
                            record.database,
                            record.schema.as_deref().unwrap_or("full"),
                            record.format.label()
                        ))
                        .color(theme::text_muted())
                        .size(10.5),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} ms", record.duration_ms))
                            .color(theme::text_muted())
                            .size(10.5),
                    );
                    ui.label(
                        RichText::new(format_size(record.size_bytes))
                            .color(theme::text_secondary())
                            .size(10.5),
                    );
                    ui.label(
                        RichText::new(&record.completed_at)
                            .color(theme::text_muted())
                            .size(10.5),
                    );
                });
            });
        })
        .response;
    response.on_hover_text(format!("Connection ID: {}", record.conn_id));
    ui.add_space(theme::SPACE_SM);
}

fn open_backup_folder(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = bytes as f64;

    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

fn render_automation_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    let schema = selected_schema_or_public(state);
    ui.add_space(theme::SPACE_XL);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Automation Presets")
                .color(theme::text_primary())
                .size(14.0)
                .strong(),
        );
        ui.label(
            RichText::new("Create maintenance query tabs from the current schema.")
                .color(theme::text_muted())
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_LG);

    let presets = [
        (
            "Vacuum Analyze",
            format!("VACUUM (VERBOSE, ANALYZE) {};", quote_ident(&schema)),
        ),
        (
            "Reindex Schema",
            format!("REINDEX SCHEMA {};", quote_ident(&schema)),
        ),
        (
            "Refresh Mat Views",
            format!(
                "DO $$\nDECLARE r record;\nBEGIN\n  FOR r IN SELECT schemaname, matviewname FROM pg_matviews WHERE schemaname = '{}'\n  LOOP\n    EXECUTE format('REFRESH MATERIALIZED VIEW %I.%I', r.schemaname, r.matviewname);\n  END LOOP;\nEND $$;",
                schema.replace('\'', "''")
            ),
        ),
    ];

    let mut action = None;
    for (title, sql) in presets {
        egui::Frame::new()
            .fill(theme::bg_medium())
            .stroke(Stroke::new(1.0, theme::border_subtle()))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_LG as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(title)
                            .color(theme::text_primary())
                            .size(12.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(theme::secondary_button("Create Query")).clicked() {
                            action = Some(ObjectAction::AddAutomationQuery {
                                title: title.to_string(),
                                sql: sql.clone(),
                            });
                        }
                    });
                });
                ui.add_space(theme::SPACE_SM);
                code_line(ui, &sql);
            });
        ui.add_space(theme::SPACE_MD);
    }

    action
}

fn render_model_tools(
    ui: &mut egui::Ui,
    state: &AppState,
    _bridge: &DbBridge,
) -> Option<ObjectAction> {
    let _ = active_conn(state)?;
    ui.add_space(theme::SPACE_XL);
    let mut action = None;
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::MODEL, "objects_model_large", 24.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t("schema_visualizer_title"))
                            .color(theme::text_primary())
                            .size(15.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(t("schema_visualizer_desc"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(theme::primary_button(&t("schema_visualizer_open")))
                        .clicked()
                    {
                        action = Some(ObjectAction::OpenModel);
                    }
                });
            });
        });
    action
}

fn render_bi_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    ui.add_space(theme::SPACE_LG);
    let Some(result) = state.current_result.as_ref() else {
        empty_state(
            ui,
            "No result set",
            "Run a query first, then BI will summarize rows and numeric columns here.",
        );
        return None;
    };

    render_count_strip(
        ui,
        result.rows.len(),
        &format!("rows, {} columns", result.columns.len()),
    );
    ui.add_space(theme::SPACE_MD);

    ScrollArea::vertical().id_salt("bi_summary").show(ui, |ui| {
        table_header(
            ui,
            &BI_COLUMNS,
            &[
                t("objects_column"),
                t("objects_type"),
                t("objects_non_null"),
                t("objects_min"),
                t("objects_max"),
                t("objects_average"),
            ],
        );
        for (idx, column) in result.columns.iter().enumerate() {
            let mut count = 0usize;
            let mut values = Vec::new();
            for row in &result.rows {
                if let Some(cell) = row.get(idx) {
                    match cell {
                        CellValue::Int(v) => {
                            count += 1;
                            values.push(*v as f64);
                        }
                        CellValue::Float(v) => {
                            count += 1;
                            values.push(*v);
                        }
                        CellValue::Null => {}
                        _ => count += 1,
                    }
                }
            }

            let (min, max, avg) = if values.is_empty() {
                ("-".to_string(), "-".to_string(), "-".to_string())
            } else {
                let min = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                (format_number(min), format_number(max), format_number(avg))
            };

            data_row(ui, &BI_COLUMNS, |row| {
                row.col(|ui| cell_label(ui, &column.name, theme::text_primary(), 12.0, false));
                row.col(|ui| type_chip(ui, &column.type_name, theme::ACCENT_BLUE));
                row.col(|ui| {
                    cell_label(ui, &count.to_string(), theme::text_secondary(), 12.0, false)
                });
                row.col(|ui| cell_label(ui, &min, theme::text_muted(), 12.0, false));
                row.col(|ui| cell_label(ui, &max, theme::text_muted(), 12.0, false));
                row.col(|ui| cell_label(ui, &avg, theme::text_muted(), 12.0, false));
            });
        }
    });

    None
}

fn render_table_row(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    row: &TableRow,
) -> Option<ObjectAction> {
    let mut action = None;
    let response = data_row(ui, &TABLE_COLUMNS, |cells| {
        cells.col(|ui| cell_label(ui, &row.schema, theme::text_muted(), 12.0, false));
        cells.col(|ui| cell_label(ui, &row.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| type_chip(ui, &row.table_type, table_type_color(&row.table_type)));
        cells.col(|ui| {
            let count = row
                .column_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            cell_label(ui, &count, theme::text_secondary(), 12.0, false);
        });
        cells.col(|ui| {
            let count = row
                .index_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            cell_label(ui, &count, theme::text_secondary(), 12.0, false);
        });
        cells.col(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_MD;
                if ui.small_button("Data").clicked() {
                    action = Some(ObjectAction::ViewData {
                        conn_id,
                        schema: row.schema.clone(),
                        name: row.name.clone(),
                    });
                }
                if row.table_type != "VIEW" && ui.small_button("Design").clicked() {
                    action = Some(ObjectAction::DesignTable {
                        schema: row.schema.clone(),
                        name: row.name.clone(),
                    });
                }
                if ui.small_button("SQL").clicked() {
                    action = Some(ObjectAction::CopySql(format!(
                        "SELECT * FROM {}.{};",
                        quote_ident(&row.schema),
                        quote_ident(&row.name)
                    )));
                }
            });
        });
    });

    if response.double_clicked() {
        action = Some(ObjectAction::ViewData {
            conn_id,
            schema: row.schema.clone(),
            name: row.name.clone(),
        });
    }

    action
}

fn render_function_row(ui: &mut egui::Ui, func: &FunctionInfo) -> Option<String> {
    let mut copied = None;
    data_row(ui, &FUNCTION_COLUMNS, |cells| {
        cells.col(|ui| cell_label(ui, &func.schema, theme::text_muted(), 12.0, false));
        cells.col(|ui| cell_label(ui, &func.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| {
            cell_label(
                ui,
                &format!("({})", func.arguments),
                theme::text_secondary(),
                11.0,
                false,
            );
        });
        cells.col(|ui| type_chip(ui, &func.return_type, theme::ACCENT_TEAL));
        cells.col(|ui| cell_label(ui, &func.language, theme::text_muted(), 11.0, false));
        cells.col(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_MD;
                type_chip(ui, &func.kind, theme::ACCENT_COPPER);
                if ui.small_button("SQL").clicked() {
                    copied = Some(format!(
                        "SELECT * FROM {}.{}({});",
                        quote_ident(&func.schema),
                        quote_ident(&func.name),
                        argument_placeholders(&func.arguments)
                    ));
                }
            });
        });
    });
    copied
}

fn render_role_row(ui: &mut egui::Ui, role: &RoleInfo) -> Option<String> {
    let mut copied = None;
    data_row(ui, &ROLE_COLUMNS, |cells| {
        cells.col(|ui| cell_label(ui, &role.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| {
            type_chip(
                ui,
                if role.can_login { "LOGIN" } else { "NOLOGIN" },
                if role.can_login {
                    theme::ACCENT_GREEN
                } else {
                    theme::text_muted()
                },
            );
        });
        let mut flags = Vec::new();
        if role.is_superuser {
            flags.push("SUPERUSER");
        }
        if role.can_create_db {
            flags.push("CREATEDB");
        }
        if role.can_create_role {
            flags.push("CREATEROLE");
        }
        if role.can_replicate {
            flags.push("REPLICATION");
        }
        let privileges = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.join(", ")
        };
        cells.col(|ui| cell_label(ui, &privileges, theme::text_secondary(), 11.0, false));
        cells.col(|ui| {
            cell_label(
                ui,
                role.valid_until.as_deref().unwrap_or("-"),
                theme::text_muted(),
                11.0,
                false,
            );
        });
        cells.col(|ui| {
            if ui.small_button("SQL").clicked() {
                copied = Some(format!("ALTER ROLE {};", quote_ident(&role.name)));
            }
        });
    });
    copied
}

fn collect_table_rows(state: &AppState, conn_id: ConnectionId) -> Vec<TableRow> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let schemas = selected_schemas(state);
    let search = state.objects_search.to_lowercase();
    let mut rows = Vec::new();

    for schema in schemas {
        if let Some(tables) = conn.tables.get(&schema) {
            for table in tables {
                if !matches_table_kind(state.active_main_view, table) {
                    continue;
                }
                if !search.is_empty()
                    && !table.name.to_lowercase().contains(&search)
                    && !schema.to_lowercase().contains(&search)
                    && !table.table_type.to_lowercase().contains(&search)
                {
                    continue;
                }

                let key = (schema.clone(), table.name.clone());
                rows.push(TableRow {
                    schema: schema.clone(),
                    name: table.name.clone(),
                    table_type: table.table_type.clone(),
                    column_count: conn.columns.get(&key).map(Vec::len),
                    index_count: conn.indexes.get(&key).map(Vec::len),
                });
            }
        }
    }

    rows.sort_by(|a, b| (&a.schema, &a.name).cmp(&(&b.schema, &b.name)));
    rows
}

fn collect_functions(state: &AppState, conn_id: ConnectionId) -> Vec<FunctionInfo> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let schemas = selected_schemas(state);
    let search = state.objects_search.to_lowercase();
    let mut rows = Vec::new();

    for schema in schemas {
        if let Some(functions) = conn.functions.get(&schema) {
            for func in functions {
                if !search.is_empty()
                    && !func.name.to_lowercase().contains(&search)
                    && !func.arguments.to_lowercase().contains(&search)
                    && !func.return_type.to_lowercase().contains(&search)
                {
                    continue;
                }
                rows.push(func.clone());
            }
        }
    }

    rows.sort_by(|a, b| {
        (&a.schema, &a.name, &a.arguments).cmp(&(&b.schema, &b.name, &b.arguments))
    });
    rows
}

fn collect_roles(state: &AppState, conn_id: ConnectionId) -> Vec<RoleInfo> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let search = state.objects_search.to_lowercase();
    conn.roles
        .iter()
        .filter(|role| search.is_empty() || role.name.to_lowercase().contains(&search))
        .cloned()
        .collect()
}

fn request_missing_tables(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schemas = selected_schemas(state);
    let mut to_load = Vec::new();
    if let Some(conn) = state.connections.get(&conn_id) {
        for schema in &schemas {
            if !conn.tables.contains_key(schema) && !conn.loading_tables.contains(schema) {
                to_load.push(schema.clone());
            }
        }
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for schema in &to_load {
            conn.loading_tables.insert(schema.clone());
        }
    }

    for schema in to_load {
        bridge.send(DbCommand::ListTables { conn_id, schema });
    }
}

fn request_missing_functions(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schemas = selected_schemas(state);
    let mut to_load = Vec::new();
    if let Some(conn) = state.connections.get(&conn_id) {
        for schema in &schemas {
            if !conn.functions.contains_key(schema) && !conn.loading_functions.contains(schema) {
                to_load.push(schema.clone());
            }
        }
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for schema in &to_load {
            conn.loading_functions.insert(schema.clone());
        }
    }

    for schema in to_load {
        bridge.send(DbCommand::ListFunctions { conn_id, schema });
    }
}

fn request_roles(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let should_load = state
        .connections
        .get(&conn_id)
        .is_some_and(|conn| conn.roles.is_empty() && !conn.loading_roles);
    if should_load {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_roles = true;
        }
        bridge.send(DbCommand::ListRoles { conn_id });
    }
}

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
            request_table_columns_for_editing(state, bridge, conn_id, &schema, &name);
            let source = DataSource {
                conn_id,
                schema: schema.clone(),
                table: name.clone(),
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
            if state.er_diagram.selected_schema.is_empty() {
                state.er_diagram.selected_schema = selected_schema_or_public(state);
            }
            state.open_workspace_view(
                MainView::Model,
                format!("Model: {}", state.er_diagram.selected_schema),
                state.er_diagram.selected_schema.clone(),
                "",
            );
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
    }
}

fn request_table_columns_for_editing(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
) {
    let key = (schema.to_string(), table.to_string());
    let should_request = state.connections.get(&conn_id).is_some_and(|conn| {
        !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key)
    });
    if should_request {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_columns.insert(key);
        }
        bridge.send(DbCommand::ListColumns {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        });
    }
}

fn render_no_connection(ui: &mut egui::Ui) -> Option<ObjectAction> {
    empty_state(
        ui,
        &t("objects_no_active_connection"),
        &t("objects_no_active_connection_help"),
    );
    None
}

fn empty_state(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        crate::ui::icon_img(ui, icons_svg::DATABASE, "objects_empty", 32.0);
        ui.add_space(theme::SPACE_MD);
        ui.label(RichText::new(title).color(theme::text_muted()).size(16.0));
        ui.label(
            RichText::new(subtitle)
                .color(theme::text_disabled())
                .size(11.0),
        );
    });
}

fn table_header(ui: &mut egui::Ui, specs: &[ColumnSpec], headers: &[impl AsRef<str>]) {
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

fn data_row(
    ui: &mut egui::Ui,
    specs: &[ColumnSpec],
    content: impl FnOnce(&mut RowCells<'_>),
) -> egui::Response {
    object_row_frame(ui, specs, 33.0, theme::bg_dark(), Sense::click(), |cells| {
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
    let full_width = ui.available_width().max(320.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(full_width, height), sense);
    let clip_rect = ui.clip_rect();
    let paint_rect = rect.intersect(clip_rect);
    if paint_rect.is_negative() {
        return response;
    }

    let fill = if response.hovered() {
        theme::with_alpha(theme::ACCENT_TEAL, 14)
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

struct RowCells<'a> {
    ui: &'a mut egui::Ui,
    rect: egui::Rect,
    widths: Vec<f32>,
    index: usize,
    cursor_x: f32,
    clip_rect: egui::Rect,
}

impl RowCells<'_> {
    fn col(&mut self, content: impl FnOnce(&mut egui::Ui)) {
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

fn cell_label(ui: &mut egui::Ui, text: &str, color: Color32, size: f32, strong: bool) {
    let mut text = RichText::new(text).color(color).size(size);
    if strong {
        text = text.strong();
    }
    ui.add(egui::Label::new(text).truncate());
}

fn render_count_strip(ui: &mut egui::Ui, count: usize, label: &str) {
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

fn code_line(ui: &mut egui::Ui, text: &str) {
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

fn type_chip(ui: &mut egui::Ui, label: &str, color: Color32) {
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

    let response = ui
        .add_sized(
            BUTTON_SIZE,
            egui::Button::new("")
                .fill(theme::bg_light())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(theme::RADIUS_MD)),
        )
        .on_hover_text(tooltip);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(response.rect).layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ),
        |ui| {
            crate::ui::icon_img(ui, svg, name, ICON_SIZE);
        },
    );

    response
}

fn active_conn(state: &AppState) -> Option<ConnectionId> {
    let conn_id = state.active_connection?;
    let conn = state.connections.get(&conn_id)?;
    matches!(conn.status, ConnectionStatus::Connected { .. }).then_some(conn_id)
}

fn selected_schemas(state: &AppState) -> Vec<String> {
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

fn selected_schema_or_public(state: &AppState) -> String {
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

fn matches_table_kind(view: MainView, table: &TableInfo) -> bool {
    match view {
        MainView::View => table.table_type == "VIEW",
        MainView::MaterializedView => table.table_type == "MATERIALIZED VIEW",
        MainView::Table => table.table_type != "VIEW" && table.table_type != "MATERIALIZED VIEW",
        _ => true,
    }
}

fn view_copy(view: MainView) -> (String, String, Color32) {
    match view {
        MainView::Table => (
            t("objects_tables_title"),
            t("objects_tables_subtitle"),
            theme::ACCENT_COPPER,
        ),
        MainView::View => (
            t("objects_views_title"),
            t("objects_views_subtitle"),
            theme::ACCENT_BLUE,
        ),
        MainView::MaterializedView => (
            t("objects_materialized_title"),
            t("objects_materialized_subtitle"),
            theme::ACCENT_TEAL,
        ),
        MainView::Function => (
            t("objects_functions_title"),
            t("objects_functions_subtitle"),
            theme::ACCENT_YELLOW,
        ),
        MainView::User => (
            t("objects_users_title"),
            t("objects_users_subtitle"),
            theme::ACCENT_COPPER_LIGHT,
        ),
        MainView::Backup => (
            t("objects_backup_title"),
            t("objects_backup_subtitle"),
            theme::text_muted(),
        ),
        MainView::Automation => (
            t("objects_automation_title"),
            t("objects_automation_subtitle"),
            theme::ACCENT_TEAL,
        ),
        MainView::Model => (
            t("objects_model_title"),
            t("objects_model_subtitle"),
            theme::ACCENT_GREEN,
        ),
        MainView::BI => (
            t("objects_bi_title"),
            t("objects_bi_subtitle"),
            theme::ACCENT_RED,
        ),
        MainView::Connection => (
            t("objects_connections_title"),
            t("objects_connections_subtitle"),
            theme::ACCENT_GREEN,
        ),
        MainView::Query => (
            t("objects_query_title"),
            t("objects_query_subtitle"),
            theme::ACCENT_BLUE,
        ),
        MainView::Data => (
            t("objects_data_title"),
            t("objects_data_subtitle"),
            theme::ACCENT_TEAL,
        ),
    }
}

fn view_icon(view: MainView) -> (&'static str, &'static str) {
    match view {
        MainView::Table => (icons_svg::TABLE, "objects_title_table"),
        MainView::View => (icons_svg::VIEW, "objects_title_view"),
        MainView::MaterializedView => (icons_svg::MATERIALIZED_VIEW, "objects_title_materialized"),
        MainView::Function => (icons_svg::FUNCTION, "objects_title_function"),
        MainView::User => (icons_svg::USER, "objects_title_user"),
        MainView::Backup => (icons_svg::BACKUP, "objects_title_backup"),
        MainView::Automation => (icons_svg::AUTOMATION, "objects_title_automation"),
        MainView::Model => (icons_svg::MODEL, "objects_title_model"),
        MainView::BI => (icons_svg::BI, "objects_title_bi"),
        MainView::Connection => (icons_svg::CONNECTION, "objects_title_connection"),
        MainView::Query => (icons_svg::QUERY, "objects_title_query"),
        MainView::Data => (icons_svg::TABLE, "objects_title_data"),
    }
}

fn table_type_color(table_type: &str) -> Color32 {
    match table_type {
        "VIEW" => theme::ACCENT_BLUE,
        "MATERIALIZED VIEW" => theme::ACCENT_TEAL,
        _ => theme::ACCENT_COPPER,
    }
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn argument_placeholders(args: &str) -> String {
    if args.trim().is_empty() {
        return String::new();
    }
    args.split(',')
        .enumerate()
        .map(|(idx, arg)| {
            let name = arg.split_whitespace().next().unwrap_or("arg");
            format!("/* {}: ${} */", name, idx + 1)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_number(value: f64) -> String {
    if value.abs() >= 1000.0 || value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.3}")
    }
}
