use eframe::egui::{self, Color32, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::{AppState, ConnectionStatus};
use crate::types::ConnectionId;
use crate::ui::{icons, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_tree(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if state.connections.is_empty() {
        ui.add_space(theme::SPACE_XXL);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(icons::DATABASE)
                    .color(theme::TEXT_DISABLED)
                    .size(28.0),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new("No connections")
                    .color(theme::TEXT_MUTED)
                    .size(12.0),
            );
            ui.label(
                RichText::new("Click \"New\" to get started")
                    .color(theme::TEXT_DISABLED)
                    .size(11.0),
            );
        });
        return;
    }

    let conn_ids: Vec<ConnectionId> = state.connections.keys().copied().collect();
    for conn_id in conn_ids {
        render_connection_node(ui, state, bridge, conn_id);
        ui.add_space(theme::SPACE_SM);
    }
}

// ---------------------------------------------------------------------------
// Connection node
// ---------------------------------------------------------------------------

fn render_connection_node(
    ui: &mut egui::Ui,
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
        Some(theme::ACCENT_YELLOW)
    } else if is_connected {
        Some(theme::ACCENT_GREEN)
    } else {
        Some(theme::ACCENT_RED)
    };

    let header_text = RichText::new(format!("  {display_name}"))
        .color(theme::TEXT_PRIMARY)
        .size(13.0)
        .strong();

    let resp = collapsing_node(ui, node_id, header_text, dot_color, true, |ui| {
        if !is_connected && !is_connecting {
            indented(ui, |ui| {
                ui.label(
                    RichText::new("Not connected")
                        .color(theme::TEXT_MUTED)
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
                            .color(theme::ACCENT_YELLOW)
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
                            .color(theme::TEXT_MUTED)
                            .size(11.0),
                    );
                });
            });
            return;
        }

        for schema in &schemas {
            render_schema_node(ui, state, bridge, conn_id, schema);
        }
    });

    if is_connected {
        resp.header_response.context_menu(|ui| {
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
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_{conn_id}_{schema}"));
    let schema_owned = schema.to_string();

    let conn = match state.connections.get(&conn_id) {
        Some(c) => c,
        None => return,
    };
    let tables = conn.tables.get(schema).cloned();
    let is_loading = conn.loading_tables.contains(schema);

    let header_text = RichText::new(format!("  {schema}"))
        .color(theme::TEXT_SECONDARY)
        .size(12.0);

    collapsing_node(ui, node_id, header_text, None, false, |ui| {
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
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );
                    });
                });
            }
            Some(tables) => {
                if tables.is_empty() {
                    indented(ui, |ui| {
                        ui.label(
                            RichText::new("(empty)")
                                .color(theme::TEXT_DISABLED)
                                .size(11.0),
                        );
                    });
                    return;
                }
                for table in tables {
                    render_table_node(
                        ui,
                        state,
                        bridge,
                        conn_id,
                        &schema_owned,
                        &table.name.clone(),
                        &table.table_type.clone(),
                    );
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Table / View node
// ---------------------------------------------------------------------------

fn render_table_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table_name: &str,
    table_type: &str,
) {
    let node_id = egui::Id::new(format!("table_{conn_id}_{schema}_{table_name}"));
    let key = (schema.to_string(), table_name.to_string());

    let conn = match state.connections.get(&conn_id) {
        Some(c) => c,
        None => return,
    };
    let columns = conn.columns.get(&key).cloned();
    let is_loading = conn.loading_columns.contains(&key);

    let (icon, icon_color) = match table_type {
        "VIEW" => (icons::VIEW, theme::ACCENT_BLUE),
        "MATERIALIZED VIEW" => (icons::MATERIALIZED_VIEW, theme::ACCENT_COPPER),
        _ => (icons::TABLE, theme::TEXT_SECONDARY),
    };

    let header_text = RichText::new(format!("{icon} {table_name}"))
        .color(theme::TEXT_PRIMARY)
        .size(12.0);
    let _ = icon_color; // used in icon selection above

    let resp = collapsing_node(ui, node_id, header_text, None, false, |ui| {
        match &columns {
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
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );
                    });
                });
            }
            Some(cols) => {
                for col in cols {
                    render_column_row(ui, col);
                }
            }
        }
    });

    resp.header_response.context_menu(|ui| {
        if ui
            .button(format!("{} View Data (Top 100)", icons::EXECUTE))
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
            .button(format!("{} Copy SELECT *", icons::COPY))
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
// Column row (leaf node)
// ---------------------------------------------------------------------------

fn render_column_row(ui: &mut egui::Ui, col: &crate::types::ColumnInfo) {
    let indent = 28.0;
    let full_width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(full_width, 18.0), Sense::hover());

    if resp.hovered() {
        ui.painter().rect_filled(rect, 0.0, theme::BG_LIGHT);
    }

    let left = rect.min + egui::vec2(indent, 0.0);

    if col.is_primary_key {
        ui.painter().text(
            left + egui::vec2(0.0, rect.height() / 2.0),
            egui::Align2::LEFT_CENTER,
            icons::KEY,
            egui::FontId::proportional(10.0),
            theme::ACCENT_YELLOW,
        );
    }

    let col_offset = if col.is_primary_key { 14.0 } else { 0.0 };

    ui.painter().text(
        left + egui::vec2(col_offset, rect.height() / 2.0),
        egui::Align2::LEFT_CENTER,
        &col.name,
        egui::FontId::proportional(12.0),
        if col.is_primary_key {
            theme::ACCENT_YELLOW
        } else {
            theme::TEXT_SECONDARY
        },
    );

    let nullable_marker = if col.is_nullable { "?" } else { "" };
    let type_text = format!("{}{}", col.data_type, nullable_marker);
    ui.painter().text(
        rect.right_center() - egui::vec2(theme::SPACE_MD, 0.0),
        egui::Align2::RIGHT_CENTER,
        &type_text,
        egui::FontId::monospace(10.0),
        theme::TEXT_MUTED,
    );
}

// ---------------------------------------------------------------------------
// Custom collapsing header widget
// ---------------------------------------------------------------------------

struct CollapsingResult {
    header_response: egui::Response,
}

fn collapsing_node(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: RichText,
    dot_color: Option<Color32>,
    is_root: bool,
    body: impl FnOnce(&mut egui::Ui),
) -> CollapsingResult {
    let state_id = id.with("__open");
    let mut open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(state_id))
        .unwrap_or(false);

    let full_width = ui.available_width();
    let row_height = if is_root { 26.0 } else { 22.0 };

    let (header_rect, header_resp) =
        ui.allocate_exact_size(egui::vec2(full_width, row_height), Sense::click());

    let bg = if header_resp.is_pointer_button_down_on() {
        Some(theme::BG_ELEVATED)
    } else if header_resp.hovered() {
        Some(theme::BG_LIGHT)
    } else if is_root {
        Some(Color32::from_rgba_premultiplied(30, 32, 38, 200))
    } else {
        None
    };

    if let Some(color) = bg {
        ui.painter().rect_filled(header_rect, 0.0, color);
    }

    // Left copper accent stripe for root nodes
    if is_root {
        let stripe = egui::Rect::from_min_size(
            header_rect.min,
            egui::vec2(2.0, header_rect.height()),
        );
        ui.painter().rect_filled(stripe, 0.0, theme::ACCENT_COPPER_DIM);
    }

    if header_resp.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(state_id, open));
    }

    let indent_x: f32 = if is_root { 8.0 } else { 20.0 };

    // Chevron ▾ / ▸
    let chevron = if open { "\u{25BE}" } else { "\u{25B8}" };
    let chevron_color = if open { theme::ACCENT_COPPER } else { theme::TEXT_MUTED };
    ui.painter().text(
        header_rect.min + egui::vec2(indent_x, row_height / 2.0),
        egui::Align2::LEFT_CENTER,
        chevron,
        egui::FontId::proportional(10.0),
        chevron_color,
    );

    // Optional status dot
    let text_start = if let Some(color) = dot_color {
        let dot_x = indent_x + 14.0;
        let dot_center = header_rect.min + egui::vec2(dot_x, row_height / 2.0);
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
        indent_x + 14.0
    };

    // Label
    let text_pos = header_rect.min + egui::vec2(text_start, row_height / 2.0);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        label.text(),
        egui::FontId::proportional(if is_root { 13.0 } else { 12.0 }),
        if is_root { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY },
    );

    if open {
        body(ui);
    }

    CollapsingResult { header_response: header_resp }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn indented(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.add_space(20.0);
        ui.vertical(f);
    });
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
