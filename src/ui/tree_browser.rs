use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::{AppState, ConnectionStatus, ObjectFilter};
use crate::types::ConnectionId;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, Tokens};
use crate::ui::icons;

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn render_tree(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let t = Tokens::current(ui.ctx());

    if state.connections.is_empty() {
        render_empty_state(ui, t);
        return;
    }

    let conn_ids: Vec<ConnectionId> = state.connections.keys().copied().collect();
    for conn_id in conn_ids {
        render_connection_node(ui, t, state, bridge, conn_id);
        ui.add_space(theme::SPACE_SM);
    }
}

fn render_empty_state(ui: &mut egui::Ui, t: Tokens) {
    ui.add_space(theme::SPACE_XXL);
    ui.vertical_centered(|ui| {
        icons::icon(ui, Icon::Database, 28.0, t.text_disabled);
        ui.add_space(theme::SPACE_MD);
        ui.label(
            RichText::new("No connections")
                .color(t.text_secondary)
                .size(13.0)
                .strong(),
        );
        ui.label(
            RichText::new("Click \u{201C}New\u{201D} to add a database")
                .color(t.text_muted)
                .size(11.0),
        );
    });
}

// ---------------------------------------------------------------------------
// Connection node
// ---------------------------------------------------------------------------

fn render_connection_node(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
) {
    let conn = match state.connections.get(&conn_id) {
        Some(c) => c,
        None => return,
    };

    let display_name = conn.config.display_name.clone();
    let is_connected = matches!(conn.status, ConnectionStatus::Connected { .. });
    let is_connecting = matches!(conn.status, ConnectionStatus::Connecting);
    let schemas = conn.schemas.clone();
    let node_id = egui::Id::new(format!("conn_{conn_id}"));

    let dot_color = if is_connecting {
        t.warn
    } else if is_connected {
        t.success
    } else {
        t.danger
    };

    let resp = collapsing_node(
        ui,
        t,
        node_id,
        &display_name,
        Some(dot_color),
        true,
        |ui| {
            if !is_connected && !is_connecting {
                indented(ui, |ui| {
                    ui.label(
                        RichText::new("Not connected")
                            .color(t.text_muted)
                            .size(12.0),
                    );
                });
                return;
            }
            if is_connecting {
                indented(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Connecting\u{2026}")
                                .color(t.warn)
                                .size(12.0),
                        );
                    });
                });
                return;
            }

            if schemas.is_empty() {
                let loading = state
                    .connections
                    .get(&conn_id)
                    .map_or(false, |c| c.loading_schemas);
                if !loading {
                    if let Some(c) = state.connections.get_mut(&conn_id) {
                        c.loading_schemas = true;
                    }
                    bridge.send(DbCommand::ListSchemas { conn_id });
                }
                indented(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Loading\u{2026}")
                                .color(t.text_muted)
                                .size(11.0),
                        );
                    });
                });
                return;
            }

            for schema in &schemas {
                render_schema_node(ui, t, state, bridge, conn_id, schema);
            }
        },
    );

    if is_connected {
        resp.context_menu(|ui| {
            if ui.button("Disconnect").clicked() {
                bridge.send(DbCommand::Disconnect { conn_id });
                ui.close_menu();
            }
            if ui.button("Refresh Schemas").clicked() {
                bridge.send(DbCommand::ListSchemas { conn_id });
                ui.close_menu();
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Schema node
// ---------------------------------------------------------------------------

fn render_schema_node(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_{conn_id}_{schema}"));
    let schema_owned = schema.to_string();

    let (tables, is_loading) = match state.connections.get(&conn_id) {
        Some(conn) => (
            conn.tables.get(schema).cloned(),
            conn.loading_tables.contains(schema),
        ),
        None => return,
    };

    collapsing_node(ui, t, node_id, schema, None, false, |ui| {
        match &tables {
            None => {
                if !is_loading {
                    if let Some(c) = state.connections.get_mut(&conn_id) {
                        c.loading_tables.insert(schema_owned.clone());
                    }
                    bridge.send(DbCommand::ListTables {
                        conn_id,
                        schema: schema_owned.clone(),
                    });
                }
                indented(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Loading\u{2026}")
                                .color(t.text_muted)
                                .size(11.0),
                        );
                    });
                });
            }
            Some(tables) => {
                let needle = state.sidebar_search.to_ascii_lowercase();
                let filter = state.object_filter;

                let filtered: Vec<_> = tables
                    .iter()
                    .filter(|tbl| object_filter_matches(filter, &tbl.table_type))
                    .filter(|tbl| {
                        needle.is_empty()
                            || tbl.name.to_ascii_lowercase().contains(&needle)
                    })
                    .collect();

                if filtered.is_empty() {
                    indented(ui, |ui| {
                        ui.label(
                            RichText::new(if needle.is_empty() {
                                "(empty)"
                            } else {
                                "(no matches)"
                            })
                            .color(t.text_disabled)
                            .size(11.0),
                        );
                    });
                    return;
                }

                for tbl in filtered {
                    render_table_node(
                        ui,
                        t,
                        state,
                        bridge,
                        conn_id,
                        &schema_owned,
                        &tbl.name,
                        &tbl.table_type,
                    );
                }
            }
        }
    });
}

fn object_filter_matches(filter: ObjectFilter, table_type: &str) -> bool {
    match filter {
        ObjectFilter::All => true,
        ObjectFilter::Tables => table_type == "BASE TABLE" || table_type == "TABLE",
        ObjectFilter::Views => {
            table_type == "VIEW" || table_type == "MATERIALIZED VIEW"
        }
        ObjectFilter::Functions => false, // Functions not yet enumerated
        ObjectFilter::Queries | ObjectFilter::History => false,
    }
}

// ---------------------------------------------------------------------------
// Table / View node
// ---------------------------------------------------------------------------

fn render_table_node(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table_name: &str,
    table_type: &str,
) {
    let node_id = egui::Id::new(format!("table_{conn_id}_{schema}_{table_name}"));
    let key = (schema.to_string(), table_name.to_string());

    let (columns, is_loading) = match state.connections.get(&conn_id) {
        Some(conn) => (
            conn.columns.get(&key).cloned(),
            conn.loading_columns.contains(&key),
        ),
        None => return,
    };

    let (mono_char, chip_color) = match table_type {
        "VIEW" => (icons::MONO_VIEW, t.chip_view),
        "MATERIALIZED VIEW" => (icons::MONO_MAT_VIEW, t.chip_mat_view),
        _ => (icons::MONO_TABLE, t.chip_table),
    };

    let resp = collapsing_node_with_chip(
        ui,
        t,
        node_id,
        table_name,
        mono_char,
        chip_color,
        |ui| match &columns {
            None => {
                if !is_loading {
                    if let Some(c) = state.connections.get_mut(&conn_id) {
                        c.loading_columns.insert(key.clone());
                    }
                    bridge.send(DbCommand::ListColumns {
                        conn_id,
                        schema: schema.to_string(),
                        table: table_name.to_string(),
                    });
                }
                indented(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Loading\u{2026}")
                                .color(t.text_muted)
                                .size(11.0),
                        );
                    });
                });
            }
            Some(cols) => {
                for col in cols {
                    render_column_row(ui, t, col);
                }
            }
        },
    );

    resp.context_menu(|ui| {
        if ui
            .button("View Data (Top 100)")
            .clicked()
        {
            let sql = format!(
                "SELECT * FROM {}.{} LIMIT 100",
                quote_ident(schema),
                quote_ident(table_name)
            );
            bridge.send(DbCommand::ExecuteQuery {
                conn_id,
                sql,
                row_limit: Some(100),
            });
            if let Some(conn) = state.connections.get(&conn_id) {
                if matches!(conn.status, ConnectionStatus::Connected { .. }) {
                    state.query_running = true;
                }
            }
            ui.close_menu();
        }
        if ui
            .button("Copy SELECT *")
            .clicked()
        {
            let sql = format!(
                "SELECT * FROM {}.{}",
                quote_ident(schema),
                quote_ident(table_name)
            );
            ui.ctx().copy_text(sql);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Refresh Columns").clicked() {
            bridge.send(DbCommand::ListColumns {
                conn_id,
                schema: schema.to_string(),
                table: table_name.to_string(),
            });
            bridge.send(DbCommand::ListIndexes {
                conn_id,
                schema: schema.to_string(),
                table: table_name.to_string(),
            });
            ui.close_menu();
        }
    });
}

// ---------------------------------------------------------------------------
// Column row (leaf)
// ---------------------------------------------------------------------------

fn render_column_row(ui: &mut egui::Ui, t: Tokens, col: &crate::types::ColumnInfo) {
    let indent = 36.0;
    let row_h = 20.0;
    let full_width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(full_width, row_h), Sense::hover());

    if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, t.bg_elev);
    }

    let left = rect.min + egui::vec2(indent, 0.0);

    if col.is_primary_key {
        let key_rect = egui::Rect::from_center_size(
            left + egui::vec2(5.0, row_h / 2.0),
            egui::vec2(10.0, 10.0),
        );
        icons::icon_at(ui.painter(), Icon::Key, key_rect, t.pk);
    }
    let col_offset = if col.is_primary_key { 14.0 } else { 0.0 };

    ui.painter().text(
        left + egui::vec2(col_offset, row_h / 2.0),
        egui::Align2::LEFT_CENTER,
        &col.name,
        egui::FontId::proportional(12.0),
        if col.is_primary_key {
            t.pk
        } else {
            t.text_secondary
        },
    );

    let nullable = if col.is_nullable { "?" } else { "" };
    let type_text = format!("{}{}", col.data_type, nullable);
    ui.painter().text(
        rect.right_center() - egui::vec2(theme::SPACE_MD, 0.0),
        egui::Align2::RIGHT_CENTER,
        &type_text,
        egui::FontId::monospace(10.0),
        t.text_muted,
    );
}

// ---------------------------------------------------------------------------
// Custom collapsing nodes
// ---------------------------------------------------------------------------

fn collapsing_node(
    ui: &mut egui::Ui,
    t: Tokens,
    id: egui::Id,
    label: &str,
    dot_color: Option<Color32>,
    is_root: bool,
    body: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let state_id = id.with("__open");
    let mut open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(state_id))
        .unwrap_or(is_root); // root nodes default-open

    let full_width = ui.available_width();
    let row_h = if is_root { 28.0 } else { 22.0 };

    let (header_rect, resp) =
        ui.allocate_exact_size(egui::vec2(full_width, row_h), Sense::click());

    if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(header_rect, 0.0, t.bg_elev);
    } else if resp.hovered() {
        ui.painter().rect_filled(header_rect, 0.0, t.bg_elev);
    }

    if is_root {
        // 2px copper accent stripe on left
        let stripe = egui::Rect::from_min_size(
            header_rect.min,
            egui::vec2(2.0, header_rect.height()),
        );
        ui.painter().rect_filled(stripe, 0.0, t.accent);
    }

    if resp.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(state_id, open));
    }

    let indent_x: f32 = if is_root { 8.0 } else { 20.0 };

    let chev_icon = if open { Icon::ChevronDown } else { Icon::ChevronRight };
    let chev_rect = egui::Rect::from_center_size(
        header_rect.min + egui::vec2(indent_x + 5.0, row_h / 2.0),
        egui::vec2(10.0, 10.0),
    );
    icons::icon_at(
        ui.painter(),
        chev_icon,
        chev_rect,
        if open { t.accent } else { t.text_muted },
    );

    let text_start = if let Some(color) = dot_color {
        let dot_x = indent_x + 16.0;
        let dot_center = header_rect.min + egui::vec2(dot_x, row_h / 2.0);
        ui.painter().circle_filled(dot_center, 4.0, color);
        ui.painter().circle_stroke(
            dot_center,
            5.5,
            Stroke::new(
                0.5,
                Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 60),
            ),
        );
        dot_x + 12.0
    } else {
        indent_x + 16.0
    };

    ui.painter().text(
        header_rect.min + egui::vec2(text_start, row_h / 2.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(if is_root { 13.0 } else { 12.0 }),
        if is_root {
            t.text_primary
        } else {
            t.text_secondary
        },
    );

    if open {
        body(ui);
    }

    resp
}

fn collapsing_node_with_chip(
    ui: &mut egui::Ui,
    t: Tokens,
    id: egui::Id,
    label: &str,
    chip_char: &str,
    chip_color: Color32,
    body: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let state_id = id.with("__open");
    let mut open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(state_id))
        .unwrap_or(false);

    let row_h = 22.0;
    let full_width = ui.available_width();
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(full_width, row_h), Sense::click());

    if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, t.bg_elev);
    }

    if resp.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(state_id, open));
    }

    let indent_x = 20.0;
    let chev_icon2 = if open { Icon::ChevronDown } else { Icon::ChevronRight };
    let chev_rect2 = egui::Rect::from_center_size(
        rect.min + egui::vec2(indent_x + 5.0, row_h / 2.0),
        egui::vec2(10.0, 10.0),
    );
    icons::icon_at(ui.painter(), chev_icon2, chev_rect2, t.text_muted);

    // Chip
    let chip_size = 14.0;
    let chip_rect = egui::Rect::from_min_size(
        rect.min + egui::vec2(indent_x + 14.0, (row_h - chip_size) / 2.0),
        egui::vec2(chip_size, chip_size),
    );
    ui.painter().rect_filled(
        chip_rect,
        CornerRadius::same(theme::RADIUS_SM),
        crate::ui::theme::monogram_bg(chip_color),
    );
    ui.painter().text(
        chip_rect.center(),
        egui::Align2::CENTER_CENTER,
        chip_char,
        egui::FontId::proportional(9.0),
        chip_color,
    );

    // Label
    ui.painter().text(
        rect.min + egui::vec2(indent_x + 14.0 + chip_size + 6.0, row_h / 2.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        t.text_primary,
    );

    if open {
        body(ui);
    }

    resp
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn indented(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(28.0);
        ui.vertical(f);
    });
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
