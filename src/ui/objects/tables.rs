//! Table / View / Materialized View objects view.
//!
//! Plan v7 Phase 1.95b3c cut-over (from `super::mod.rs`). Phase 2 의 Table/
//! View Designer (Create/Replace) 와 Phase 4c BI 의 카드 진입점이 본 모듈
//! 위에서 진행된다.

use eframe::egui::{self, ScrollArea};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{AppState, MainView};
use crate::types::{ConnectionId, TableInfo};
use crate::ui::theme;

use super::{
    active_conn, cell_label, data_row_alt, empty_state, quote_ident, render_count_strip,
    render_no_connection, selected_schemas, table_header, type_chip, views, ObjectAction,
    TABLE_COLUMNS,
};

#[derive(Clone)]
pub(super) struct TableRow {
    pub schema: String,
    pub name: String,
    pub table_type: String,
    pub column_count: Option<usize>,
    pub index_count: Option<usize>,
    pub row_estimate: Option<u64>,
}

pub(super) fn render_table_like_objects(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_missing_tables(state, bridge, conn_id);
    request_missing_metadata(state, bridge, conn_id);
    let rows = collect_table_rows(state, conn_id);
    render_count_strip(ui, rows.len(), "objects");

    let mut action = None;
    ScrollArea::both()
        .id_salt("objects_table_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &TABLE_COLUMNS,
                &[
                    t("objects_schema"),
                    t("objects_name"),
                    t("objects_type"),
                    t("objects_rows"),
                    t("objects_columns"),
                    t("objects_indexes"),
                    t("objects_actions"),
                ],
            );
            if rows.is_empty() {
                empty_state(
                    ui,
                    &t("objects_no_tables"),
                    &t("objects_no_tables_help"),
                );
            }
            for (i, row) in rows.iter().enumerate() {
                let row_action = render_table_row(ui, conn_id, row, i);
                if row_action.is_some() {
                    action = row_action;
                }
            }
        });

    action
}

fn render_table_row(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    row: &TableRow,
    row_index: usize,
) -> Option<ObjectAction> {
    let mut action = None;
    let response = data_row_alt(ui, &TABLE_COLUMNS, row_index, |cells| {
        cells.col(|ui| cell_label(ui, &row.schema, theme::text_muted(), 12.0, false));
        cells.col(|ui| cell_label(ui, &row.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| type_chip(ui, &row.table_type, views::table_type_color(&row.table_type)));
        cells.col(|ui| {
            let text = row
                .row_estimate
                .map(format_row_count)
                .unwrap_or_else(|| "~".to_string());
            cell_label(ui, &text, theme::text_secondary(), 12.0, false);
        });
        cells.col(|ui| {
            let count = row
                .column_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "~".to_string());
            cell_label(ui, &count, theme::text_secondary(), 12.0, false);
        });
        cells.col(|ui| {
            let count = row
                .index_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "~".to_string());
            cell_label(ui, &count, theme::text_secondary(), 12.0, false);
        });
        cells.col(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if icon_action_chip(
                    ui,
                    crate::ui::icons_svg::PLAY,
                    "tbl_data",
                    &t("button_data"),
                    theme::accent_color(),
                )
                .clicked()
                {
                    action = Some(ObjectAction::ViewData {
                        conn_id,
                        schema: row.schema.clone(),
                        name: row.name.clone(),
                    });
                }
                if row.table_type != "VIEW"
                    && icon_action_chip(
                        ui,
                        crate::ui::icons_svg::EDIT,
                        "tbl_design",
                        &t("button_design"),
                        theme::accent_color_light(),
                    )
                    .clicked()
                {
                    action = Some(ObjectAction::DesignTable {
                        schema: row.schema.clone(),
                        name: row.name.clone(),
                    });
                }
                if icon_action_chip(
                    ui,
                    crate::ui::icons_svg::CODE,
                    "tbl_sql",
                    &t("button_sql"),
                    theme::ACCENT_BLUE,
                )
                .clicked()
                {
                    action = Some(ObjectAction::CopySql(format!(
                        "SELECT * FROM {}.{};",
                        quote_ident(&row.schema),
                        quote_ident(&row.name)
                    )));
                }
                if icon_action_chip(
                    ui,
                    crate::ui::icons_svg::TRASH,
                    "tbl_drop",
                    &t("button_drop"),
                    theme::ACCENT_RED,
                )
                .clicked()
                {
                    action = Some(ObjectAction::DropTable {
                        conn_id,
                        schema: row.schema.clone(),
                        name: row.name.clone(),
                        kind: crate::state::DropTargetKind::from_table_type(&row.table_type),
                    });
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
    } else if response.clicked() {
        action = Some(ObjectAction::SelectTable {
            schema: row.schema.clone(),
            name: row.name.clone(),
        });
    }

    action
}

fn icon_action_chip(
    ui: &mut egui::Ui,
    svg: &str,
    name: &str,
    tooltip: &str,
    color: egui::Color32,
) -> egui::Response {
    let size = egui::vec2(24.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::with_alpha(color, 32)
    } else {
        theme::with_alpha(color, 14)
    };
    let stroke = egui::Stroke::new(
        1.0,
        theme::with_alpha(color, if hovered { 160 } else { 70 }),
    );
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(theme::RADIUS_SM), fill);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(theme::RADIUS_SM),
        stroke,
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    let icon_img = crate::ui::icon_image_tinted(ui, svg, name, 13.0, color);
    icon_img.paint_at(ui, icon_rect);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text(tooltip)
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
                    row_estimate: table.row_estimate,
                });
            }
        }
    }

    rows.sort_by(|a, b| (&a.schema, &a.name).cmp(&(&b.schema, &b.name)));

    // 검색어와 정확히 같은 이름이 존재하면 그것만 보여준다 — "TaxBill" 검색 시
    // "TaxBillItem" 까지 같이 나오는 substring 매칭을 좁힌다.
    if !search.is_empty() {
        let exact_matches: Vec<TableRow> = rows
            .iter()
            .filter(|row| row.name.to_lowercase() == search)
            .cloned()
            .collect();
        if !exact_matches.is_empty() {
            return exact_matches;
        }
    }

    rows
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

fn request_missing_metadata(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let Some(conn) = state.connections.get(&conn_id) else {
        return;
    };
    let schemas = selected_schemas(state);
    let mut col_requests = Vec::new();
    let mut idx_requests = Vec::new();
    for schema in &schemas {
        let Some(tables) = conn.tables.get(schema) else {
            continue;
        };
        for table in tables {
            if !matches_table_kind(state.active_main_view, table) {
                continue;
            }
            let key = (schema.clone(), table.name.clone());
            if !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key) {
                col_requests.push(key.clone());
            }
            if !conn.indexes.contains_key(&key) && !conn.loading_indexes.contains(&key) {
                idx_requests.push(key);
            }
        }
    }
    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for key in &col_requests {
            conn.loading_columns.insert(key.clone());
        }
        for key in &idx_requests {
            conn.loading_indexes.insert(key.clone());
        }
    }
    for (schema, table) in col_requests {
        bridge.send(DbCommand::ListColumns {
            conn_id,
            schema,
            table,
        });
    }
    for (schema, table) in idx_requests {
        bridge.send(DbCommand::ListIndexes {
            conn_id,
            schema,
            table,
        });
    }
}

pub(super) fn request_table_columns_for_editing(
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

fn matches_table_kind(view: MainView, table: &TableInfo) -> bool {
    match view {
        MainView::View => table.table_type == "VIEW",
        MainView::MaterializedView => table.table_type == "MATERIALIZED VIEW",
        MainView::Table => table.table_type != "VIEW" && table.table_type != "MATERIALIZED VIEW",
        _ => true,
    }
}

fn format_row_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
