use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::{AppState, ConnectionStatus};
use crate::types::ConnectionId;
use crate::ui::{icon_img, icons, icons_svg, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_tree(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if state.connections.is_empty() {
        ui.add_space(theme::SPACE_XXL);
        ui.vertical_centered(|ui| {
            icon_img(ui, icons_svg::DATABASE, "database_empty", 32.0);
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new("No connections")
                    .color(theme::TEXT_MUTED)
                    .size(12.0),
            );
            ui.label(
                RichText::new("Create a connection to browse schemas")
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

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        dot_color,
        true,
        icons_svg::DATABASE,
        "conn",
        |ui| {
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
                            RichText::new("Connecting...")
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
                            RichText::new("Loading...")
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
        },
    );

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

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        icons_svg::SCHEMA,
        "schema",
        |ui| match &tables {
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
                            RichText::new("Loading...")
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
        },
    );

    resp.header_response.context_menu(|ui| {
        if ui.button("New Table").clicked() {
            ui.close_menu();
            crate::ui::table_designer::open_for_new_table_with_schema(state, schema);
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

    let (icon_svg, icon_name) = match table_type {
        "VIEW" => (icons_svg::VIEW, "view"),
        "MATERIALIZED VIEW" => (icons_svg::MATERIALIZED_VIEW, "mat_view"),
        _ => (icons_svg::TABLE, "table"),
    };

    let header_text = RichText::new(table_name)
        .color(theme::TEXT_PRIMARY)
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        icon_svg,
        icon_name,
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
                            RichText::new("Loading...")
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
        },
    );

    resp.header_response.context_menu(|ui| {
        let edit_resp = ui.button("      Edit Table");
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                edit_resp
                    .rect
                    .shrink2(egui::vec2(edit_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                icon_img(ui, icons_svg::TABLE, "edit_table_icon", 12.0);
            },
        );
        if edit_resp.clicked() {
            crate::ui::table_designer::open_for_existing_table(state, schema, table_name, bridge);
            ui.close_menu();
        }

        ui.separator();

        let view_resp = ui.button("      View Data (Top 100)");
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                view_resp
                    .rect
                    .shrink2(egui::vec2(view_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                icon_img(ui, icons_svg::EXECUTE, "view_data_icon", 12.0);
            },
        );
        if view_resp.clicked() {
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

        let copy_resp = ui.button("      Copy SELECT *");
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                copy_resp
                    .rect
                    .shrink2(egui::vec2(copy_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                icon_img(ui, icons_svg::COPY, "copy_sql_icon", 12.0);
            },
        );
        if copy_resp.clicked() {
            let sql = format!(
                "SELECT * FROM {}.{}",
                quote_ident(schema),
                quote_ident(table_name)
            );
            ui.ctx().copy_text(sql);
            ui.close_menu();
        }

        ui.separator();

        let refresh_resp = ui.button("      Refresh Columns");
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                refresh_resp
                    .rect
                    .shrink2(egui::vec2(refresh_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                icon_img(ui, icons_svg::REFRESH, "refresh_cols_icon", 12.0);
            },
        );
        if refresh_resp.clicked() {
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
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(4.0, 1.0)),
            CornerRadius::same(theme::RADIUS_MD),
            theme::with_alpha(theme::ACCENT_TEAL, 16),
        );
    }

    let left = rect.min + egui::vec2(indent, 0.0);

    if col.is_primary_key {
        icon_img(ui, icons_svg::KEY, "pk", 12.0);
        ui.add_space(theme::SPACE_SM);
    } else {
        icon_img(ui, icons_svg::COLUMN, "col", 12.0);
        ui.add_space(theme::SPACE_SM);
    }

    ui.label(
        RichText::new(&col.name)
            .color(if col.is_primary_key {
                theme::ACCENT_YELLOW
            } else {
                theme::TEXT_SECONDARY
            })
            .size(12.0),
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
    icon_svg: &str,
    icon_name: &str,
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
        Some(theme::with_alpha(theme::ACCENT_TEAL, 20))
    } else if is_root {
        Some(theme::BG_DARK)
    } else {
        None
    };

    let paint_rect = header_rect.shrink2(egui::vec2(if is_root { 5.0 } else { 2.0 }, 1.0));
    if let Some(color) = bg {
        ui.painter().rect_filled(
            paint_rect,
            CornerRadius::same(if is_root {
                theme::RADIUS_LG
            } else {
                theme::RADIUS_MD
            }),
            color,
        );
    }

    // Left copper accent stripe for root nodes
    if is_root {
        let stripe =
            egui::Rect::from_min_size(paint_rect.min, egui::vec2(2.0, paint_rect.height()));
        ui.painter().rect_filled(
            stripe,
            CornerRadius::same(theme::RADIUS_SM),
            theme::ACCENT_COPPER_DIM,
        );
    }

    if header_resp.clicked() {
        open = !open;
        ui.ctx().data_mut(|d| d.insert_temp(state_id, open));
    }

    let indent_x: f32 = if is_root { 8.0 } else { 20.0 };

    // Chevron ▾ / ▸
    let (chevron_svg, chevron_name) = if open {
        (icons_svg::CHEVRON_DOWN, "chevron_down")
    } else {
        (icons_svg::CHEVRON_RIGHT, "chevron_right")
    };

    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            header_rect.min + egui::vec2(indent_x + 4.0, row_height / 2.0),
            egui::vec2(12.0, 12.0),
        )),
        |ui| {
            icon_img(ui, chevron_svg, chevron_name, 10.0);
        },
    );

    // Icon
    let icon_start = indent_x + 12.0;
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            header_rect.min + egui::vec2(icon_start + 8.0, row_height / 2.0),
            egui::vec2(16.0, 16.0),
        )),
        |ui| {
            icon_img(ui, icon_svg, icon_name, 14.0);
        },
    );

    // Optional status dot
    let text_start = if let Some(color) = dot_color {
        let dot_x = icon_start + 24.0;
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
        icon_start + 24.0
    };

    // Label
    let text_pos = header_rect.min + egui::vec2(text_start, row_height / 2.0);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        label.text(),
        egui::FontId::proportional(if is_root { 13.0 } else { 12.0 }),
        if is_root {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        },
    );

    if open {
        body(ui);
    }

    CollapsingResult {
        header_response: header_resp,
    }
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
