use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, AppState, ConnectionState, ConnectionStatus, DataSource,
    MainView, SchemaContextMenuState,
};
use crate::types::{
    ColumnInfo, ConnectionConfig, ConnectionId, FunctionInfo, IndexInfo, RuleInfo, TableInfo,
    TriggerInfo,
};
use crate::ui::{icon_image, icon_img, icon_img_tinted, icons_svg, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_tree(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if state.connections.is_empty() && state.saved_connections.is_empty() {
        ui.add_space(theme::SPACE_XXL);
        ui.vertical_centered(|ui| {
            icon_img(ui, icons_svg::DATABASE, "database_empty", 32.0);
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new(t("tree_no_connections"))
                    .color(theme::text_muted())
                    .size(12.0),
            );
            ui.label(
                RichText::new(t("tree_create_connection"))
                    .color(theme::text_disabled())
                    .size(11.0),
            );
        });
        render_explorer_empty_space(ui, state, bridge);
        return;
    }

    let conn_ids: Vec<ConnectionId> = state.connections.keys().copied().collect();
    for conn_id in conn_ids {
        render_connection_node(ui, state, bridge, conn_id);
        ui.add_space(theme::SPACE_SM);
    }

    let saved_connections: Vec<(usize, ConnectionConfig)> = state
        .saved_connections
        .iter()
        .enumerate()
        .filter(|(_, config)| !state.connections.contains_key(&config.id))
        .map(|(index, config)| (index, config.clone()))
        .collect();

    if !saved_connections.is_empty() {
        if !state.connections.is_empty() {
            ui.add_space(theme::SPACE_SM);
            ui.separator();
            ui.add_space(theme::SPACE_SM);
        }

        let can_reorder = saved_connections.len() > 1;
        for (_, config) in saved_connections {
            render_saved_connection_row(ui, state, bridge, config, can_reorder);
            ui.add_space(theme::SPACE_XS);
        }
    }

    if ui.input(|input| input.pointer.any_released()) {
        state.dragging_saved_connection = None;
    }

    render_explorer_empty_space(ui, state, bridge);
    render_schema_context_menu_popup(ui, state, bridge);
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

    let config = conn.config.clone();
    let display_name = config.display_name.clone();
    let database = conn.config.database.clone();
    let databases = conn.databases.clone();
    let is_connected = matches!(conn.status, ConnectionStatus::Connected { .. });
    let is_connecting = matches!(conn.status, ConnectionStatus::Connecting);
    let schemas = conn.schemas.clone();
    let connection_error = conn.connection_error.clone();
    let node_id = egui::Id::new(format!("conn_{conn_id}"));

    let dot_color = if is_connecting {
        Some(theme::ACCENT_YELLOW)
    } else if is_connected {
        Some(theme::ACCENT_GREEN)
    } else {
        Some(theme::ACCENT_RED)
    };

    let header_text = RichText::new(display_name)
        .color(theme::text_primary())
        .size(13.0)
        .strong();

    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color,
            is_root: true,
            depth: 0,
            default_open: true,
            force_open: false,
            selected: false,
            icon_svg: icons_svg::DATABASE,
            icon_name: "conn",
            double_click_to_expand: false,
            icon_tint: None,
        },
        |ui| {
            if !is_connected && !is_connecting {
                indented(ui, |ui| {
                    if let Some(err) = &connection_error {
                        ui.label(
                            RichText::new(err)
                                .color(theme::accent_red_soft())
                                .size(11.0),
                        );
                    } else {
                        ui.label(
                            RichText::new("Not connected")
                                .color(theme::text_muted())
                                .size(12.0),
                        );
                    }
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

            if databases.is_empty() {
                let loading = state
                    .connections
                    .get(&conn_id)
                    .is_some_and(|c| c.loading_databases);
                if !loading {
                    if let Some(c) = state.connections.get_mut(&conn_id) {
                        c.loading_databases = true;
                    }
                    bridge.send(DbCommand::ListDatabases { conn_id });
                }
                indented(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(t("loading"))
                                .color(theme::text_muted())
                                .size(11.0),
                        );
                    });
                });
                return;
            }

            let mut database_names = databases.clone();
            if !database_names.iter().any(|name| name == &database) {
                database_names.push(database.clone());
                database_names.sort();
            }

            for database_name in database_names {
                if database_name == database {
                    render_database_node(ui, state, bridge, conn_id, &database_name, &schemas);
                } else {
                    render_database_leaf(ui, 1, &database_name);
                }
            }
        },
    );

    resp.header_response.context_menu(|ui| {
        render_connection_context_menu(
            ui,
            state,
            bridge,
            config.clone(),
            Some(conn_id),
            is_connected,
            is_connecting,
        );
    });
}

fn render_saved_connection_row(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    config: ConnectionConfig,
    can_reorder: bool,
) {
    let detail = format!("{}:{}/{}", config.host, config.port, config.database);
    let full_width = ui.available_width().max(1.0);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(full_width, 54.0),
        if can_reorder {
            Sense::click_and_drag()
        } else {
            Sense::click()
        },
    );
    let hover_hint = if can_reorder {
        "Click to connect. Drag to reorder."
    } else {
        "Click to connect."
    };

    if response.hovered() || response.dragged() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(2.0, 1.0)),
            CornerRadius::same(theme::RADIUS_LG),
            theme::with_alpha(theme::ACCENT_TEAL, 18),
        );
    }

    if response.drag_started() {
        state.dragging_saved_connection = Some(config.id);
    }

    let dragging_this = state.dragging_saved_connection == Some(config.id);
    if dragging_this {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(2.0, 1.0)),
            CornerRadius::same(theme::RADIUS_LG),
            Stroke::new(1.0, theme::ACCENT_EMERALD),
            egui::StrokeKind::Inside,
        );
    }

    let pointer_pos = ui.input(|input| input.pointer.interact_pos());
    if let (Some(dragged_id), Some(pointer_pos)) = (state.dragging_saved_connection, pointer_pos) {
        if can_reorder && dragged_id != config.id && rect.contains(pointer_pos) {
            let insert_after = pointer_pos.y > rect.center().y;
            let y = if insert_after {
                rect.bottom() + 1.0
            } else {
                rect.top() - 1.0
            };
            ui.painter()
                .hline(rect.x_range(), y, Stroke::new(2.0, theme::ACCENT_EMERALD));

            if ui.input(|input| input.pointer.any_released()) {
                reorder_saved_connection(state, dragged_id, config.id, insert_after);
            }
        }
    }

    let icon_center = rect.left_center() + egui::vec2(if can_reorder { 29.0 } else { 17.0 }, 0.0);
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            icon_center,
            egui::vec2(16.0, 16.0),
        )),
        |ui| {
            icon_img(ui, icons_svg::DATABASE, "saved_connection", 16.0);
        },
    );

    if can_reorder {
        let handle_x = rect.left() + 8.0;
        for y in [
            rect.center().y - 6.0,
            rect.center().y,
            rect.center().y + 6.0,
        ] {
            ui.painter().circle_filled(
                egui::pos2(handle_x, y),
                1.4,
                if dragging_this {
                    theme::ACCENT_EMERALD
                } else {
                    theme::text_disabled()
                },
            );
        }
    }

    let text_x = rect.left() + if can_reorder { 44.0 } else { 30.0 };
    let name_y = rect.top() + 18.0;
    let detail_y = rect.top() + 37.0;
    let text_right = (rect.right() - theme::SPACE_MD).max(text_x + 24.0);
    let name_rect = egui::Rect::from_min_max(
        egui::pos2(text_x, rect.top() + 6.0),
        egui::pos2(text_right, rect.top() + 29.0),
    );
    let detail_rect = egui::Rect::from_min_max(
        egui::pos2(text_x, rect.top() + 27.0),
        egui::pos2(text_right, rect.bottom() - 5.0),
    );

    ui.painter().with_clip_rect(name_rect).text(
        egui::pos2(text_x, name_y),
        egui::Align2::LEFT_CENTER,
        &config.display_name,
        egui::FontId::proportional(13.0),
        theme::text_primary(),
    );

    ui.painter().with_clip_rect(detail_rect).text(
        egui::pos2(text_x, detail_y),
        egui::Align2::LEFT_CENTER,
        &detail,
        egui::FontId::monospace(10.5),
        theme::text_muted(),
    );
    show_dark_hover_tooltip(
        ui,
        response.id.with("saved_connection_tooltip"),
        &response,
        &format!("{}\n{}\n{}", config.display_name, detail, hover_hint),
    );
    if response.hovered() {
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
    }

    if response.clicked() {
        connect_saved_connection(state, bridge, config.clone());
    }

    response.context_menu(|ui| {
        render_connection_context_menu(ui, state, bridge, config.clone(), None, false, false);
    });
}

fn render_connection_context_menu(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    config: ConnectionConfig,
    active_conn_id: Option<ConnectionId>,
    is_connected: bool,
    is_connecting: bool,
) {
    if let Some(conn_id) = active_conn_id {
        let can_close = is_connected || is_connecting;
        if ui
            .add_enabled(can_close, egui::Button::new(t("ctx_close_connection")))
            .clicked()
        {
            bridge.send(DbCommand::Disconnect { conn_id });
            ui.close_menu();
        }
    } else if ui.button(t("ctx_open_connection")).clicked() {
        connect_saved_connection(state, bridge, config.clone());
        ui.close_menu();
    }

    ui.menu_button(t("ctx_switch_connection_profile"), |ui| {
        if state.saved_connections.is_empty() {
            ui.add_enabled(false, egui::Button::new(t("ctx_no_saved_profiles")));
        }

        let profiles = state.saved_connections.clone();
        for profile in profiles {
            if ui.button(&profile.display_name).clicked() {
                connect_saved_connection(state, bridge, profile);
                ui.close_menu();
            }
        }
    });

    ui.separator();

    if ui.button(t("ctx_edit_connection")).clicked() {
        edit_connection(state, &config);
        ui.close_menu();
    }
    if ui.button(t("ctx_new_connection")).clicked() {
        new_connection(state);
        ui.close_menu();
    }
    if ui.button(t("ctx_delete_connection")).clicked() {
        delete_connection(state, bridge, config.id);
        ui.close_menu();
    }
    if ui.button(t("ctx_duplicate_connection")).clicked() {
        duplicate_connection(state, &config);
        ui.close_menu();
    }

    ui.separator();

    ui.add_enabled(false, egui::Button::new(t("ctx_new_database")));
    if ui.button(t("ctx_new_query")).clicked() {
        new_query_for_connection(state, Some(config.id));
        ui.close_menu();
    }

    ui.separator();

    ui.add_enabled(false, egui::Button::new(t("ctx_console")));
    ui.add_enabled(false, egui::Button::new(t("ctx_execute_sql_file")));

    ui.separator();

    ui.add_enabled(false, egui::Button::new(t("ctx_add_star")));
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(t("ctx_color"))
                .color(theme::text_secondary())
                .size(12.0),
        );
        if color_clear_button(ui).clicked() {
            set_connection_color(state, config.id, None);
            ui.close_menu();
        }
        for (name, color) in connection_colors() {
            if color_button(ui, color).clicked() {
                set_connection_color(state, config.id, Some(name.to_string()));
                ui.close_menu();
            }
        }
    });

    ui.separator();

    ui.menu_button(t("ctx_manage_group"), |ui| {
        ui.add_enabled(false, egui::Button::new(t("ctx_create_group")));
        ui.add_enabled(false, egui::Button::new(t("ctx_move_to_group")));
    });
    ui.add_enabled(false, egui::Button::new(t("ctx_share")));

    ui.separator();

    if ui.button(t("ctx_refresh")).clicked() {
        if let Some(conn_id) = active_conn_id {
            bridge.send(DbCommand::ListDatabases { conn_id });
            bridge.send(DbCommand::ListSchemas { conn_id });
        } else {
            connect_saved_connection(state, bridge, config);
        }
        ui.close_menu();
    }
}

fn render_explorer_empty_space(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let width = ui.available_width().max(1.0);
    let available_height = ui.available_height();
    let height = if available_height.is_finite() {
        available_height.max(140.0)
    } else {
        180.0
    };

    let (_rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    response.context_menu(|ui| {
        render_explorer_context_menu(ui, state, bridge);
    });
}

fn render_explorer_context_menu(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if ui.button(t("ctx_new_connection")).clicked() {
        new_connection(state);
        ui.close_menu();
    }

    let can_close_all = state.connections.values().any(|conn| {
        matches!(
            conn.status,
            ConnectionStatus::Connected { .. } | ConnectionStatus::Connecting
        )
    });
    if ui
        .add_enabled(
            can_close_all,
            egui::Button::new(t("ctx_close_all_connections")),
        )
        .clicked()
    {
        close_all_connections(state, bridge);
        ui.close_menu();
    }

    if ui.button(t("ctx_manage_connections")).clicked() {
        manage_connections(state);
        ui.close_menu();
    }

    ui.separator();
    ui.add_enabled(false, egui::Button::new(t("ctx_new_group")));
    ui.separator();

    if ui.button(t("ctx_refresh")).clicked() {
        refresh_explorer(state, bridge);
        ui.close_menu();
    }
}

fn edit_connection(state: &mut AppState, config: &ConnectionConfig) {
    state.connection_dialog = crate::state::ConnectionDialogState::from_config(config);
    state.show_connection_dialog = true;
}

fn new_connection(state: &mut AppState) {
    state.connection_dialog = Default::default();
    state.show_connection_dialog = true;
}

fn manage_connections(state: &mut AppState) {
    state.connection_dialog = Default::default();
    state.connection_dialog.clipboard_import_enabled = false;
    state.show_connection_dialog = true;
}

fn close_all_connections(state: &mut AppState, bridge: &DbBridge) {
    let conn_ids: Vec<ConnectionId> = state.connections.keys().copied().collect();
    for conn_id in conn_ids {
        bridge.send(DbCommand::Disconnect { conn_id });
    }
    state.status_message = t("tree_closing_all_connections");
}

fn refresh_explorer(state: &mut AppState, bridge: &DbBridge) {
    let conn_ids: Vec<ConnectionId> = state.connections.keys().copied().collect();
    for conn_id in &conn_ids {
        bridge.send(DbCommand::ListDatabases { conn_id: *conn_id });
        bridge.send(DbCommand::ListSchemas { conn_id: *conn_id });
    }

    state.status_message = if conn_ids.is_empty() {
        t("tree_explorer_refreshed")
    } else {
        tf(
            "tree_refreshing_connections",
            &[&conn_ids.len().to_string()],
        )
    };
}

fn delete_connection(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    if state.connections.contains_key(&conn_id) {
        bridge.send(DbCommand::Disconnect { conn_id });
        state.connections.remove(&conn_id);
    }
    if state.active_connection == Some(conn_id) {
        state.active_connection = state.connections.keys().next().copied();
    }
    state.saved_connections.retain(|saved| saved.id != conn_id);
    save_vault_connections(state);
}

fn duplicate_connection(state: &mut AppState, config: &ConnectionConfig) {
    let mut duplicate = config.clone();
    duplicate.id = ConnectionId::new();
    duplicate.display_name = format!("{} Copy", config.display_name);
    state.saved_connections.push(duplicate.clone());
    save_vault_connections(state);
    state.connection_dialog = crate::state::ConnectionDialogState::from_config(&duplicate);
    state.show_connection_dialog = true;
}

fn reorder_saved_connection(
    state: &mut AppState,
    dragged_id: ConnectionId,
    target_id: ConnectionId,
    insert_after: bool,
) {
    if dragged_id == target_id {
        state.dragging_saved_connection = None;
        return;
    }

    let Some(from) = state
        .saved_connections
        .iter()
        .position(|config| config.id == dragged_id)
    else {
        state.dragging_saved_connection = None;
        return;
    };

    let moved = state.saved_connections.remove(from);
    let Some(mut to) = state
        .saved_connections
        .iter()
        .position(|config| config.id == target_id)
    else {
        state.saved_connections.insert(from, moved);
        state.dragging_saved_connection = None;
        return;
    };

    if insert_after {
        to += 1;
    }
    let to = to.min(state.saved_connections.len());
    state.saved_connections.insert(to, moved);
    state.dragging_saved_connection = None;
    save_vault_connections(state);
}

fn new_query_for_connection(state: &mut AppState, conn_id: Option<ConnectionId>) {
    let n = state.editor_tabs.len() + 1;
    let mut tab = crate::types::EditorTab::new(format!("Query {n}"));
    tab.connection_id = conn_id;
    state.editor_tabs.push(tab);
    state.active_tab = state.editor_tabs.len() - 1;
    state.active_connection = conn_id;
    state.open_workspace_main_view(crate::state::MainView::Query);
}

fn set_connection_color(state: &mut AppState, conn_id: ConnectionId, color: Option<String>) {
    if let Some(saved) = state
        .saved_connections
        .iter_mut()
        .find(|saved| saved.id == conn_id)
    {
        saved.color_tag = color.clone();
    }
    if let Some(active) = state.connections.get_mut(&conn_id) {
        active.config.color_tag = color;
    }
    save_vault_connections(state);
}

fn connection_colors() -> [(&'static str, Color32); 8] {
    [
        ("red", Color32::from_rgb(235, 85, 85)),
        ("orange", Color32::from_rgb(240, 150, 55)),
        ("yellow", Color32::from_rgb(238, 216, 84)),
        ("green", Color32::from_rgb(92, 202, 112)),
        ("teal", Color32::from_rgb(38, 205, 190)),
        ("blue", Color32::from_rgb(82, 171, 255)),
        ("purple", Color32::from_rgb(151, 127, 255)),
        ("gray", Color32::from_rgb(190, 198, 208)),
    ]
}

fn color_button(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), Sense::click());
    ui.painter()
        .rect_filled(rect.shrink(2.0), CornerRadius::same(2), color);
    response
}

fn color_clear_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(13.0, 13.0), Sense::click());
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_top()],
        Stroke::new(1.4, theme::ACCENT_RED),
    );
    response
}

fn connect_saved_connection(state: &mut AppState, bridge: &DbBridge, config: ConnectionConfig) {
    let conn_id = config.id;

    if let Some(conn) = state.connections.get(&conn_id) {
        state.active_connection = Some(conn_id);
        if matches!(
            conn.status,
            ConnectionStatus::Connected { .. } | ConnectionStatus::Connecting
        ) {
            return;
        }
    }

    state
        .connections
        .insert(conn_id, ConnectionState::new(config.clone()));
    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.status = ConnectionStatus::Connecting;
    }
    state.active_connection = Some(conn_id);
    state.status_message = format!("Connecting to {}\u{2026}", config.display_name);
    bridge.send(DbCommand::Connect { conn_id, config });
}

fn save_vault_connections(state: &mut AppState) {
    let Some(session) = state.vault.session.as_ref() else {
        state.last_error = Some("Unlock the Personal Vault first.".to_string());
        return;
    };

    if let Err(err) =
        crate::storage::connections::save_connections(&state.saved_connections, session)
    {
        state.last_error = Some(err.to_string());
        state.status_message = err.to_string();
    }
}

// ---------------------------------------------------------------------------
// Database / Schema nodes
// ---------------------------------------------------------------------------

fn render_database_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    database: &str,
    schemas: &[String],
) {
    let node_id = egui::Id::new(format!("database_{conn_id}_{database}"));
    let header_text = RichText::new(database)
        .color(theme::text_primary())
        .size(12.0)
        .strong();

    collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 1,
            default_open: true,
            force_open: false,
            selected: false,
            icon_svg: icons_svg::DATABASE,
            icon_name: "database",
            double_click_to_expand: false,
            icon_tint: None,
        },
        |ui| {
            if schemas.is_empty() {
                let loading = state
                    .connections
                    .get(&conn_id)
                    .is_some_and(|c| c.loading_schemas);
                if !loading {
                    if let Some(c) = state.connections.get_mut(&conn_id) {
                        c.loading_schemas = true;
                    }
                    bridge.send(DbCommand::ListSchemas { conn_id });
                }
                render_status_row(ui, 2, &t("loading"), true);
                return;
            }

            for schema in schemas {
                render_schema_node(ui, state, bridge, conn_id, schema);
            }
        },
    );
}

fn render_database_leaf(ui: &mut egui::Ui, depth: usize, database: &str) {
    render_leaf_row(
        ui,
        depth,
        icons_svg::DATABASE,
        "database_leaf",
        database,
        None,
    );
}

fn render_schema_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_{conn_id}_{schema}"));
    let schema_owned = schema.to_string();
    let schema_opened = state
        .connections
        .get(&conn_id)
        .is_some_and(|c| c.opened_schemas.contains(schema));
    egui::containers::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        node_id,
        false,
    )
    .set_open(schema_opened);
    let schema_selected = state.objects_schema_filter == schema
        && state.active_connection == Some(conn_id);

    let header_text = RichText::new(schema)
        .color(if schema_selected {
            theme::text_primary()
        } else if schema_opened {
            theme::text_secondary()
        } else {
            theme::text_muted()
        })
        .size(12.0);

    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 2,
            default_open: false,
            force_open: schema_opened,
            selected: false,
            icon_svg: icons_svg::SCHEMA,
            icon_name: "schema",
            double_click_to_expand: true,
            icon_tint: None,
        },
        |ui| {
            if !schema_opened {
                return;
            }
            render_table_group_node(
                ui,
                state,
                bridge,
                conn_id,
                &schema_owned,
                SchemaTableGroup::Tables,
            );
            render_table_group_node(
                ui,
                state,
                bridge,
                conn_id,
                &schema_owned,
                SchemaTableGroup::Views,
            );
            render_table_group_node(
                ui,
                state,
                bridge,
                conn_id,
                &schema_owned,
                SchemaTableGroup::MaterializedViews,
            );
            render_function_group_node(ui, state, bridge, conn_id, &schema_owned);
            render_query_group_node(ui, state, conn_id, &schema_owned);
            render_backups_group_node(ui, state, conn_id, &schema_owned);
        },
    );

    if resp.header_response.clicked() && !resp.header_response.double_clicked() {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            if conn.opened_schemas.contains(schema) {
                conn.opened_schemas.remove(schema);
            } else {
                conn.opened_schemas.insert(schema.to_string());
                if !conn.tables.contains_key(schema) {
                    bridge.send(DbCommand::ListTables {
                        conn_id,
                        schema: schema.to_string(),
                    });
                }
            }
        }
    }

    if resp.header_response.secondary_clicked() {
        let pos = resp
            .header_response
            .interact_pointer_pos()
            .unwrap_or(resp.header_response.rect.right_center());
        state.schema_context_menu = Some(SchemaContextMenuState {
            conn_id,
            schema: schema.to_string(),
            pos: [pos.x, pos.y],
        });
    }
}

// ---------------------------------------------------------------------------
// macOS-style schema context menu
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum SchemaMenuAction {
    Open,
    BackupSchema,
    Edit,
    NewSchema,
    Delete,
    Console,
    DumpSql,
    DataDictionary,
    ReverseModel,
    Find,
    Share,
    Refresh,
}

const SCHEMA_MENU_WIDTH: f32 = 292.0;
const SCHEMA_MENU_ITEM_HEIGHT: f32 = 30.0;
const SCHEMA_MENU_SEPARATOR_HEIGHT: f32 = 9.0;

fn render_schema_context_menu_popup(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let Some(menu) = state.schema_context_menu.clone() else {
        return;
    };

    let mut action = None;
    let mut close = false;
    let pos = egui::pos2(menu.pos[0], menu.pos[1]);
    let area = egui::Area::new(egui::Id::new("schema_context_menu_popup"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_elevated())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(18))
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.set_min_width(SCHEMA_MENU_WIDTH);
                    ui.set_max_width(SCHEMA_MENU_WIDTH);

                    if mac_menu_item(ui, &t("ctx_open_schema"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Open);
                    }
                    if mac_menu_item(
                        ui,
                        &tf("ctx_backup_schema", &[&menu.schema]),
                        None,
                        None,
                        true,
                    )
                    .clicked()
                    {
                        action = Some(SchemaMenuAction::BackupSchema);
                    }
                    mac_menu_separator(ui);

                    if mac_menu_item(ui, &t("ctx_edit_schema"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Edit);
                    }
                    if mac_menu_item(ui, &t("ctx_new_schema"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::NewSchema);
                    }
                    if mac_menu_item(ui, &t("ctx_delete_schema"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Delete);
                    }
                    mac_menu_separator(ui);

                    mac_menu_item(ui, &t("ctx_new_query"), Some("⌘ Y"), None, false);
                    mac_menu_separator(ui);

                    if mac_menu_item(ui, &t("ctx_console"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Console);
                    }
                    mac_menu_item(ui, &t("ctx_execute_sql_file"), None, None, false);
                    if mac_menu_item(ui, &t("ctx_dump_sql_file"), None, Some("›"), true).clicked()
                    {
                        action = Some(SchemaMenuAction::DumpSql);
                    }
                    if mac_menu_item(ui, &t("ctx_data_dictionary"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::DataDictionary);
                    }
                    if mac_menu_item(ui, &t("ctx_reverse_database_to_model"), None, None, true)
                        .clicked()
                    {
                        action = Some(SchemaMenuAction::ReverseModel);
                    }
                    if mac_menu_item(ui, &t("ctx_find_in_database"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Find);
                    }
                    mac_menu_separator(ui);

                    if mac_menu_item(ui, &t("ctx_share"), None, None, true).clicked() {
                        action = Some(SchemaMenuAction::Share);
                    }
                    mac_menu_separator(ui);

                    if mac_menu_item(ui, &t("ctx_refresh"), Some("⌘ R"), None, true).clicked() {
                        action = Some(SchemaMenuAction::Refresh);
                    }
                });
        });

    let area_rect = area.response.rect;
    ui.ctx().input(|input| {
        if input.key_pressed(egui::Key::Escape) {
            close = true;
        }
        if input.pointer.any_pressed()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|pointer_pos| !area_rect.contains(pointer_pos))
        {
            close = true;
        }
    });

    if let Some(action) = action {
        handle_schema_menu_action(state, bridge, &menu, action);
        close = true;
    }

    if close {
        state.schema_context_menu = None;
    }
}

fn mac_menu_item(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: Option<&str>,
    trailing: Option<&str>,
    enabled: bool,
) -> egui::Response {
    let width = ui.available_width().max(SCHEMA_MENU_WIDTH);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(width, SCHEMA_MENU_ITEM_HEIGHT),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );

    let hovered = response.hovered() && enabled;
    if hovered {
        ui.painter().rect_filled(
            rect.expand2(egui::vec2(6.0, 0.0)),
            CornerRadius::same(theme::RADIUS_MD),
            theme::with_alpha(theme::ACCENT_TEAL, 46),
        );
    }

    let text_color = if enabled {
        theme::text_primary()
    } else {
        theme::text_disabled()
    };
    let aux_color = if hovered {
        theme::ACCENT_TEAL
    } else {
        theme::text_muted()
    };

    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        text_color,
    );

    if let Some(shortcut) = shortcut {
        ui.painter().text(
            rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(12.0),
            aux_color,
        );
    }

    if let Some(trailing) = trailing {
        ui.painter().text(
            rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            trailing,
            egui::FontId::proportional(17.0),
            if hovered {
                theme::ACCENT_TEAL
            } else {
                theme::text_secondary()
            },
        );
    }

    response
}

fn mac_menu_separator(ui: &mut egui::Ui) {
    let width = ui.available_width().max(SCHEMA_MENU_WIDTH);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, SCHEMA_MENU_SEPARATOR_HEIGHT),
        Sense::hover(),
    );
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, theme::border_default()),
    );
}

fn handle_schema_menu_action(
    state: &mut AppState,
    bridge: &DbBridge,
    menu: &SchemaContextMenuState,
    action: SchemaMenuAction,
) {
    match action {
        SchemaMenuAction::Open => {
            activate_table_group_view(state, menu.conn_id, &menu.schema, SchemaTableGroup::Tables);
        }
        SchemaMenuAction::BackupSchema => {
            activate_backup_view(state, menu.conn_id, Some(&menu.schema));
        }
        SchemaMenuAction::Console => {
            new_query_for_connection(state, Some(menu.conn_id));
        }
        SchemaMenuAction::DumpSql => {
            activate_backup_view(state, menu.conn_id, Some(&menu.schema));
        }
        SchemaMenuAction::Refresh => {
            refresh_schema_objects(state, bridge, menu.conn_id, &menu.schema)
        }
        SchemaMenuAction::ReverseModel => {
            activate_schema_model_view(state, bridge, menu.conn_id, &menu.schema);
        }
        SchemaMenuAction::Edit
        | SchemaMenuAction::NewSchema
        | SchemaMenuAction::Delete
        | SchemaMenuAction::DataDictionary
        | SchemaMenuAction::Find
        | SchemaMenuAction::Share => {
            state.status_message = t("connection_coming_soon");
        }
    }
}

fn activate_schema_model_view(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    state.active_connection = Some(conn_id);
    state.er_diagram.selected_schema = schema.to_string();
    state.er_diagram.search.clear();
    state.er_diagram.show_diagram = true;
    state.open_workspace_view(
        crate::state::MainView::Model,
        format!("Model: {schema}"),
        schema,
        "",
    );

    let mut request_tables = false;
    let mut request_foreign_keys = false;
    if let Some(conn) = state.connections.get(&conn_id) {
        request_tables = !conn.tables.contains_key(schema) && !conn.loading_tables.contains(schema);
        request_foreign_keys =
            !conn.foreign_keys.contains_key(schema) && !conn.loading_foreign_keys.contains(schema);
    }
    if let Some(conn) = state.connections.get_mut(&conn_id) {
        if request_tables {
            conn.loading_tables.insert(schema.to_string());
        }
        if request_foreign_keys {
            conn.loading_foreign_keys.insert(schema.to_string());
        }
    }
    if request_tables {
        bridge.send(DbCommand::ListTables {
            conn_id,
            schema: schema.to_string(),
        });
    }
    if request_foreign_keys {
        bridge.send(DbCommand::ListForeignKeys {
            conn_id,
            schema: schema.to_string(),
        });
    }

    state.status_message = format!("Opening model for schema {schema}");
}

fn refresh_schema_objects(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.tables.remove(schema);
        conn.functions.remove(schema);
        conn.foreign_keys.remove(schema);
        conn.loading_tables.insert(schema.to_string());
        conn.loading_functions.insert(schema.to_string());
        conn.loading_foreign_keys.insert(schema.to_string());
    }

    bridge.send(DbCommand::ListTables {
        conn_id,
        schema: schema.to_string(),
    });
    bridge.send(DbCommand::ListFunctions {
        conn_id,
        schema: schema.to_string(),
    });
    bridge.send(DbCommand::ListForeignKeys {
        conn_id,
        schema: schema.to_string(),
    });
    state.status_message = format!("Refreshing schema {schema}");
}

// ---------------------------------------------------------------------------
// Schema object groups
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum SchemaTableGroup {
    Tables,
    Views,
    MaterializedViews,
}

impl SchemaTableGroup {
    fn label_key(self) -> &'static str {
        match self {
            Self::Tables => "tree_tables",
            Self::Views => "tree_views",
            Self::MaterializedViews => "tree_materialized_views",
        }
    }

    fn label(self) -> String {
        t(self.label_key())
    }

    fn id_key(self) -> &'static str {
        match self {
            Self::Tables => "tables",
            Self::Views => "views",
            Self::MaterializedViews => "materialized_views",
        }
    }

    fn icon(self) -> (&'static str, &'static str) {
        match self {
            Self::Tables => (icons_svg::TABLE, "group_tables"),
            Self::Views => (icons_svg::VIEW, "group_views"),
            Self::MaterializedViews => (icons_svg::MATERIALIZED_VIEW, "group_mat_views"),
        }
    }

    fn matches(self, table_type: &str) -> bool {
        match self {
            Self::Tables => !matches!(table_type, "VIEW" | "MATERIALIZED VIEW"),
            Self::Views => table_type == "VIEW",
            Self::MaterializedViews => table_type == "MATERIALIZED VIEW",
        }
    }
}

fn active_data_source_matches(
    state: &AppState,
    conn_id: ConnectionId,
    schema: &str,
    table: Option<&str>,
) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    if source.conn_id != conn_id || source.schema != schema {
        return false;
    }
    table.is_none_or(|table| source.table == table)
}

fn active_table_group_matches(
    state: &AppState,
    conn_id: ConnectionId,
    schema: &str,
    group: SchemaTableGroup,
) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    if source.conn_id != conn_id || source.schema != schema {
        return false;
    }
    state
        .connections
        .get(&conn_id)
        .and_then(|conn| conn.tables.get(schema))
        .and_then(|tables| tables.iter().find(|table| table.name == source.table))
        .is_none_or(|table| group.matches(&table.table_type))
}

fn render_table_group_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    group: SchemaTableGroup,
) {
    let node_id = egui::Id::new(format!(
        "schema_group_{conn_id}_{schema}_{}",
        group.id_key()
    ));
    let (icon_svg, icon_name) = group.icon();
    let group_active = state.active_connection == Some(conn_id)
        && state.objects_schema_filter == schema
        && match group {
            SchemaTableGroup::Tables => matches!(state.active_main_view, MainView::Table),
            SchemaTableGroup::Views => matches!(state.active_main_view, MainView::View),
            SchemaTableGroup::MaterializedViews => {
                matches!(state.active_main_view, MainView::MaterializedView)
            }
        };
    let header_text = RichText::new(group.label())
        .color(if group_active {
            theme::text_primary()
        } else {
            theme::text_secondary()
        })
        .size(12.0);
    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 3,
            default_open: false,
            force_open: false,
            selected: group_active,
            icon_svg,
            icon_name,
            double_click_to_expand: true,
            icon_tint: None,
        },
        |ui| {
            let Some(tables) = ensure_schema_tables(ui, state, bridge, conn_id, schema, 4) else {
                return;
            };

            let filtered: Vec<TableInfo> = tables
                .into_iter()
                .filter(|table| group.matches(&table.table_type))
                .collect();

            if filtered.is_empty() {
                render_status_row(ui, 4, &t("tree_empty"), false);
                return;
            }

            for table in filtered {
                render_table_node(
                    ui,
                    state,
                    bridge,
                    conn_id,
                    schema,
                    &table.name,
                    &table.table_type,
                );
            }
        },
    );

    if resp.header_response.clicked() {
        activate_table_group_view(state, conn_id, schema, group);
    }

    resp.header_response.context_menu(|ui| {
        let group_label = group.label();
        if ui.button(tf("tree_show_group", &[&group_label])).clicked() {
            activate_table_group_view(state, conn_id, schema, group);
            ui.close_menu();
        }
    });
}

fn activate_table_group_view(
    state: &mut AppState,
    conn_id: ConnectionId,
    schema: &str,
    group: SchemaTableGroup,
) {
    state.active_connection = Some(conn_id);
    let view = match group {
        SchemaTableGroup::Tables => crate::state::MainView::Table,
        SchemaTableGroup::Views => crate::state::MainView::View,
        SchemaTableGroup::MaterializedViews => crate::state::MainView::MaterializedView,
    };
    let group_label = group.label();
    state.open_workspace_view(view, format!("{group_label}: {schema}"), schema, "");
    state.status_message = tf("tree_showing_group", &[&group_label, schema]);
}

fn activate_function_view(state: &mut AppState, conn_id: ConnectionId, schema: &str) {
    state.active_connection = Some(conn_id);
    state.open_workspace_view(
        crate::state::MainView::Function,
        format!("{}: {schema}", t("tree_functions")),
        schema,
        "",
    );
    state.status_message = tf("tree_showing_functions", &[schema]);
}

fn render_function_group_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_group_{conn_id}_{schema}_functions"));
    let fn_active = state.active_connection == Some(conn_id)
        && state.objects_schema_filter == schema
        && matches!(state.active_main_view, MainView::Function);
    let header_text = RichText::new(t("tree_functions"))
        .color(if fn_active { theme::text_primary() } else { theme::text_secondary() })
        .size(12.0);

    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 3,
            default_open: false,
            force_open: false,
            selected: fn_active,
            icon_svg: icons_svg::FUNCTION,
            icon_name: "group_functions",
            double_click_to_expand: true,
            icon_tint: None,
        },
        |ui| {
            let Some(functions) = ensure_schema_functions(ui, state, bridge, conn_id, schema, 4)
            else {
                return;
            };

            if functions.is_empty() {
                render_status_row(ui, 4, &t("tree_empty"), false);
                return;
            }

            for function in functions {
                render_function_row(ui, 4, schema, &function);
            }
        },
    );

    if resp.header_response.clicked() {
        activate_function_view(state, conn_id, schema);
    }

    resp.header_response.context_menu(|ui| {
        if ui.button(t("tree_show_functions")).clicked() {
            activate_function_view(state, conn_id, schema);
            ui.close_menu();
        }
    });
}

fn render_query_group_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_group_{conn_id}_{schema}_queries"));
    let q_active = state.active_connection == Some(conn_id)
        && state.objects_schema_filter == schema
        && matches!(state.active_main_view, MainView::Query);
    let header_text = RichText::new(t("tree_queries"))
        .color(if q_active { theme::text_primary() } else { theme::text_secondary() })
        .size(12.0);

    collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 3,
            default_open: false,
            force_open: false,
            selected: q_active,
            icon_svg: icons_svg::QUERY,
            icon_name: "group_queries",
            double_click_to_expand: true,
            icon_tint: None,
        },
        |ui| {
            let response = render_leaf_row(
                ui,
                4,
                icons_svg::QUERY,
                "new_query",
                &t("ctx_new_query"),
                None,
            );
            if response.clicked() {
                let n = state.editor_tabs.len() + 1;
                let mut tab = crate::types::EditorTab::new(format!("Query {n}"));
                tab.connection_id = Some(conn_id);
                state.editor_tabs.push(tab);
                state.active_tab = state.editor_tabs.len() - 1;
                state.open_workspace_main_view(crate::state::MainView::Query);
            }

            let tabs: Vec<(usize, String)> = state
                .editor_tabs
                .iter()
                .enumerate()
                .filter(|(_, tab)| tab.connection_id == Some(conn_id))
                .map(|(idx, tab)| (idx, tab.title.clone()))
                .collect();

            for (idx, title) in tabs {
                let response = render_leaf_row(ui, 4, icons_svg::QUERY, "query_tab", &title, None);
                if response.clicked() {
                    state.active_tab = idx;
                    state.open_workspace_main_view(crate::state::MainView::Query);
                }
            }
        },
    );
}

fn render_backups_group_node(
    ui: &mut egui::Ui,
    state: &mut AppState,
    conn_id: ConnectionId,
    schema: &str,
) {
    let node_id = egui::Id::new(format!("schema_group_{conn_id}_{schema}_backups"));
    let bk_active = state.active_connection == Some(conn_id)
        && state.objects_schema_filter == schema
        && matches!(state.active_main_view, MainView::Backup);
    let header_text = RichText::new(t("tree_backups"))
        .color(if bk_active { theme::text_primary() } else { theme::text_secondary() })
        .size(12.0);

    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 3,
            default_open: false,
            force_open: false,
            selected: bk_active,
            icon_svg: icons_svg::BACKUP,
            icon_name: "group_backups",
            double_click_to_expand: true,
            icon_tint: None,
        },
        |ui| {
            let schema_response = render_leaf_row(
                ui,
                4,
                icons_svg::BACKUP,
                "schema_backup",
                &t("tree_schema_backup"),
                Some(schema),
            );
            if schema_response.clicked() {
                activate_backup_view(state, conn_id, Some(schema));
            }

            let full_response = render_leaf_row(
                ui,
                4,
                icons_svg::DATABASE,
                "full_database_backup",
                &t("tree_full_database_backup"),
                None,
            );
            if full_response.clicked() {
                activate_backup_view(state, conn_id, None);
            }
        },
    );

    if resp.header_response.clicked() {
        activate_backup_view(state, conn_id, Some(schema));
    }

    resp.header_response.context_menu(|ui| {
        if ui.button(t("tree_schema_backup")).clicked() {
            activate_backup_view(state, conn_id, Some(schema));
            ui.close_menu();
        }
        if ui.button(t("tree_full_database_backup")).clicked() {
            activate_backup_view(state, conn_id, None);
            ui.close_menu();
        }
    });
}

fn activate_backup_view(state: &mut AppState, conn_id: ConnectionId, schema: Option<&str>) {
    state.active_connection = Some(conn_id);
    let schema_filter = schema.unwrap_or_default();
    state.open_workspace_view(
        crate::state::MainView::Backup,
        match schema {
            Some(schema) => tf("tree_backup_schema_title", &[schema]),
            None => t("tree_backup_full_title"),
        },
        schema_filter,
        "",
    );
    state.status_message = match schema {
        Some(schema) => tf("tree_backup_scope_schema", &[schema]),
        None => t("tree_backup_scope_full"),
    };
}

fn ensure_schema_tables(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    depth: usize,
) -> Option<Vec<TableInfo>> {
    let conn = state.connections.get(&conn_id)?;
    let tables = conn.tables.get(schema).cloned();
    let is_loading = conn.loading_tables.contains(schema);

    match tables {
        Some(tables) => Some(tables),
        None => {
            if !is_loading {
                if let Some(c) = state.connections.get_mut(&conn_id) {
                    c.loading_tables.insert(schema.to_string());
                }
                bridge.send(DbCommand::ListTables {
                    conn_id,
                    schema: schema.to_string(),
                });
            }
            render_status_row(ui, depth, &t("loading"), true);
            None
        }
    }
}

fn ensure_schema_functions(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    depth: usize,
) -> Option<Vec<FunctionInfo>> {
    let conn = state.connections.get(&conn_id)?;
    let functions = conn.functions.get(schema).cloned();
    let is_loading = conn.loading_functions.contains(schema);

    match functions {
        Some(functions) => Some(functions),
        None => {
            if !is_loading {
                if let Some(c) = state.connections.get_mut(&conn_id) {
                    c.loading_functions.insert(schema.to_string());
                }
                bridge.send(DbCommand::ListFunctions {
                    conn_id,
                    schema: schema.to_string(),
                });
            }
            render_status_row(ui, depth, &t("loading"), true);
            None
        }
    }
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

    let Some(conn) = state.connections.get(&conn_id) else {
        return;
    };
    let columns = conn.columns.get(&key).cloned();
    let indexes = conn.indexes.get(&key).cloned();
    let foreign_keys = conn.foreign_keys.get(schema).map(|keys| {
        keys.iter()
            .filter(|fk| fk.source_table == table_name)
            .cloned()
            .collect::<Vec<_>>()
    });
    let rules = conn.rules.get(&key).cloned();
    let triggers = conn.triggers.get(&key).cloned();
    let loading_columns = conn.loading_columns.contains(&key);
    let loading_indexes = conn.loading_indexes.contains(&key);
    let loading_foreign_keys = conn.loading_foreign_keys.contains(schema);
    let loading_rules = conn.loading_rules.contains(&key);
    let loading_triggers = conn.loading_triggers.contains(&key);

    let (icon_svg, icon_name) = match table_type {
        "VIEW" => (icons_svg::VIEW, "view"),
        "MATERIALIZED VIEW" => (icons_svg::MATERIALIZED_VIEW, "mat_view"),
        _ => (icons_svg::TABLE, "table"),
    };
    let active_table = active_data_source_matches(state, conn_id, schema, Some(table_name));
    let workspace_match = state.active_connection == Some(conn_id)
        && state.objects_schema_filter == schema
        && state.objects_search == table_name;
    let is_selected = active_table || workspace_match;

    let header_text = RichText::new(table_name)
        .color(if is_selected {
            theme::ACCENT_TEAL
        } else {
            theme::text_primary()
        })
        .size(12.0);

    let resp = collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: node_id,
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 4,
            default_open: false,
            force_open: is_selected,
            selected: is_selected,
            icon_svg,
            icon_name,
            double_click_to_expand: false,
            icon_tint: None,
        },
        |ui| {
            request_table_metadata(state, bridge, conn_id, schema, table_name);
            render_fields_group(
                ui,
                conn_id,
                schema,
                table_name,
                columns.as_deref(),
                loading_columns,
            );
            render_indexes_group(
                ui,
                conn_id,
                schema,
                table_name,
                indexes.as_deref(),
                loading_indexes,
            );
            render_foreign_keys_group(
                ui,
                conn_id,
                schema,
                table_name,
                foreign_keys.as_deref(),
                loading_foreign_keys,
            );
            render_unique_group(
                ui,
                conn_id,
                schema,
                table_name,
                indexes.as_deref(),
                loading_indexes,
            );
            render_rules_group(
                ui,
                conn_id,
                schema,
                table_name,
                rules.as_deref(),
                loading_rules,
            );
            render_triggers_group(
                ui,
                conn_id,
                schema,
                table_name,
                triggers.as_deref(),
                loading_triggers,
            );
        },
    );

    if resp.header_response.clicked() {
        state.active_connection = Some(conn_id);
        state.objects_schema_filter = schema.to_string();
        state.objects_search = table_name.to_string();
    }
    if resp.header_response.double_clicked() {
        open_table_data(state, bridge, conn_id, schema, table_name);
    }

    resp.header_response.context_menu(|ui| {
        if !matches!(table_type, "VIEW" | "MATERIALIZED VIEW") {
            let edit_resp = ui.add(theme::ghost_icon_button(
                icon_image(ui, icons_svg::TABLE, "edit_table_icon", 12.0),
                t("tree_edit_table"),
            ));
            if edit_resp.clicked() {
                crate::ui::table_designer::open_for_existing_table(
                    state, schema, table_name, bridge,
                );
                ui.close_menu();
            }

            ui.separator();
        }

        let view_resp = ui.add(theme::ghost_icon_button(
            icon_image(ui, icons_svg::EXECUTE, "view_data_icon", 12.0),
            t("tree_view_data_top_100"),
        ));
        if view_resp.clicked() {
            open_table_data(state, bridge, conn_id, schema, table_name);
            ui.close_menu();
        }

        let copy_resp = ui.add(theme::ghost_icon_button(
            crate::ui::icon_image_tinted(
                ui,
                icons_svg::COPY,
                "copy_sql_icon",
                12.0,
                theme::ACCENT_BLUE,
            ),
            t("tree_copy_select"),
        ));
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

        let refresh_resp = ui.add(theme::ghost_icon_button(
            icon_image(ui, icons_svg::REFRESH, "refresh_cols_icon", 12.0),
            t("tree_refresh_metadata"),
        ));
        if refresh_resp.clicked() {
            let key = (schema.to_string(), table_name.to_string());
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                conn.columns.remove(&key);
                conn.indexes.remove(&key);
                conn.rules.remove(&key);
                conn.triggers.remove(&key);
                conn.foreign_keys.remove(schema);
                conn.loading_columns.insert(key.clone());
                conn.loading_indexes.insert(key.clone());
                conn.loading_rules.insert(key.clone());
                conn.loading_triggers.insert(key);
                conn.loading_foreign_keys.insert(schema.to_string());
            }
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
            bridge.send(DbCommand::ListForeignKeys {
                conn_id,
                schema: schema.to_string(),
            });
            bridge.send(DbCommand::ListRules {
                conn_id,
                schema: schema.to_string(),
                table: table_name.to_string(),
            });
            bridge.send(DbCommand::ListTriggers {
                conn_id,
                schema: schema.to_string(),
                table: table_name.to_string(),
            });
            ui.close_menu();
        }
    });
}

fn open_table_data(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table_name: &str,
) {
    state.active_connection = Some(conn_id);
    state.current_result = None;
    state.current_result_truncated = false;
    state.begin_data_edit(conn_id, schema, table_name);
    request_table_columns_for_editing(state, bridge, conn_id, schema, table_name);
    let source = DataSource {
        conn_id,
        schema: schema.to_string(),
        table: table_name.to_string(),
        filter: None,
    };
    let limit = state.data_edit.page_limit;
    let columns = state.data_columns_for_source(&source);
    let sql =
        build_data_select_sql_with_columns(&source, &state.data_edit.sort, limit, 0, &columns);
    bridge.send(DbCommand::ExecuteQuery {
        conn_id,
        sql,
        row_limit: Some(limit),
    });
    if let Some(conn) = state.connections.get(&conn_id) {
        if matches!(conn.status, ConnectionStatus::Connected { .. }) {
            state.query_running = true;
        }
    }
    state.open_workspace_view(
        crate::state::MainView::Data,
        format!("{schema}.{table_name}"),
        schema,
        table_name,
    );
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

fn request_table_metadata(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
) {
    let key = (schema.to_string(), table.to_string());

    let Some(conn) = state.connections.get_mut(&conn_id) else {
        return;
    };

    if !conn.columns.contains_key(&key) && conn.loading_columns.insert(key.clone()) {
        bridge.send(DbCommand::ListColumns {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        });
    }

    if !conn.indexes.contains_key(&key) && conn.loading_indexes.insert(key.clone()) {
        bridge.send(DbCommand::ListIndexes {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        });
    }

    if !conn.foreign_keys.contains_key(schema)
        && conn.loading_foreign_keys.insert(schema.to_string())
    {
        bridge.send(DbCommand::ListForeignKeys {
            conn_id,
            schema: schema.to_string(),
        });
    }

    if !conn.rules.contains_key(&key) && conn.loading_rules.insert(key.clone()) {
        bridge.send(DbCommand::ListRules {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        });
    }

    if !conn.triggers.contains_key(&key) && conn.loading_triggers.insert(key) {
        bridge.send(DbCommand::ListTriggers {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        });
    }
}

fn render_fields_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    columns: Option<&[ColumnInfo]>,
    loading: bool,
) {
    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "fields",
        &t("tree_fields"),
        icons_svg::COLUMN,
        "table_fields",
        columns.map(|items| items.len()),
        false,
        |ui| match columns {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(columns) => {
                for col in columns {
                    render_column_row(ui, col, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || columns.is_none()),
        },
    );
}

fn render_indexes_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    indexes: Option<&[IndexInfo]>,
    loading: bool,
) {
    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "indexes",
        &t("tree_indexes"),
        icons_svg::INDEX,
        "table_indexes",
        indexes.map(|items| items.len()),
        false,
        |ui| match indexes {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(indexes) => {
                for index in indexes {
                    render_index_row(ui, index, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || indexes.is_none()),
        },
    );
}

fn render_foreign_keys_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    foreign_keys: Option<&[crate::ui::er_diagram::ForeignKey]>,
    loading: bool,
) {
    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "foreign_keys",
        &t("tree_foreign_keys"),
        icons_svg::KEY,
        "table_foreign_keys",
        foreign_keys.map(|items| items.len()),
        false,
        |ui| match foreign_keys {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(keys) => {
                for fk in keys {
                    render_foreign_key_row(ui, fk, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || foreign_keys.is_none()),
        },
    );
}

fn render_unique_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    indexes: Option<&[IndexInfo]>,
    loading: bool,
) {
    let unique_indexes = indexes.map(|items| {
        items
            .iter()
            .filter(|index| index.is_unique && !index.is_primary)
            .cloned()
            .collect::<Vec<_>>()
    });

    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "unique",
        &t("tree_unique"),
        icons_svg::UNIQUE,
        "table_unique",
        unique_indexes.as_ref().map(|items| items.len()),
        false,
        |ui| match unique_indexes.as_deref() {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(indexes) => {
                for index in indexes {
                    render_index_row(ui, index, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || indexes.is_none()),
        },
    );
}

fn render_rules_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    rules: Option<&[RuleInfo]>,
    loading: bool,
) {
    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "rules",
        &t("tree_rules"),
        icons_svg::RULE,
        "table_rules",
        rules.map(|items| items.len()),
        false,
        |ui| match rules {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(rules) => {
                for rule in rules {
                    render_rule_row(ui, rule, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || rules.is_none()),
        },
    );
}

fn render_triggers_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    triggers: Option<&[TriggerInfo]>,
    loading: bool,
) {
    render_metadata_group(
        ui,
        conn_id,
        schema,
        table,
        "triggers",
        &t("tree_triggers"),
        icons_svg::TRIGGER,
        "table_triggers",
        triggers.map(|items| items.len()),
        false,
        |ui| match triggers {
            Some([]) => render_status_row(ui, 6, &t("tree_empty"), false),
            Some(triggers) => {
                for trigger in triggers {
                    render_trigger_row(ui, trigger, 6);
                }
            }
            None => render_status_row(ui, 6, &t("loading"), loading || triggers.is_none()),
        },
    );
}

#[allow(clippy::too_many_arguments)]
fn render_metadata_group(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    schema: &str,
    table: &str,
    group_key: &str,
    label: &str,
    icon_svg: &str,
    icon_name: &str,
    count: Option<usize>,
    default_open: bool,
    body: impl FnOnce(&mut egui::Ui),
) {
    let count_text = count
        .map(|count| format!(" ({count})"))
        .unwrap_or_else(|| " (...)".to_string());
    let header_text = RichText::new(format!("{label}{count_text}"))
        .color(theme::text_secondary())
        .size(12.0);

    collapsing_node(
        ui,
        CollapsingNodeSpec {
            id: egui::Id::new(format!("table_meta_{conn_id}_{schema}_{table}_{group_key}")),
            label: header_text,
            dot_color: None,
            is_root: false,
            depth: 5,
            default_open,
            force_open: false,
            selected: false,
            icon_svg,
            icon_name,
            double_click_to_expand: false,
            icon_tint: None,
        },
        body,
    );
}

// ---------------------------------------------------------------------------
// Leaf rows
// ---------------------------------------------------------------------------

fn render_column_row(ui: &mut egui::Ui, col: &crate::types::ColumnInfo, depth: usize) {
    let full_width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(full_width, 18.0), Sense::hover());

    if resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(4.0, 1.0)),
            CornerRadius::same(theme::RADIUS_MD),
            theme::with_alpha(theme::ACCENT_TEAL, 16),
        );
    }

    let indent_x = depth_indent(depth);
    let icon_svg = if col.is_primary_key {
        icons_svg::KEY
    } else {
        icons_svg::COLUMN
    };
    let icon_name = if col.is_primary_key { "pk" } else { "col" };
    paint_inline_icon(ui, rect, indent_x, icon_svg, icon_name, 12.0);

    let text_color = if col.is_primary_key {
        theme::ACCENT_YELLOW
    } else {
        theme::text_secondary()
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(indent_x + 18.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &col.name,
        egui::FontId::proportional(12.0),
        text_color,
    );

    let nullable_marker = if col.is_nullable { "?" } else { "" };
    let type_text = format!("{}{}", col.data_type, nullable_marker);
    ui.painter().text(
        rect.right_center() - egui::vec2(theme::SPACE_MD, 0.0),
        egui::Align2::RIGHT_CENTER,
        &type_text,
        egui::FontId::monospace(10.0),
        theme::text_muted(),
    );
}

fn render_index_row(ui: &mut egui::Ui, index: &IndexInfo, depth: usize) {
    let kind = if index.is_primary {
        "primary"
    } else if index.is_unique {
        "unique"
    } else {
        index.index_type.as_str()
    };
    let detail = format!("{} · {}", kind, index.columns.join(", "));
    render_leaf_row(
        ui,
        depth,
        if index.is_unique {
            icons_svg::UNIQUE
        } else {
            icons_svg::INDEX
        },
        "index_leaf",
        &index.name,
        Some(&detail),
    );
}

fn render_foreign_key_row(ui: &mut egui::Ui, fk: &crate::ui::er_diagram::ForeignKey, depth: usize) {
    let detail = format!(
        "{} -> {}.{}.{}",
        fk.source_column, fk.target_schema, fk.target_table, fk.target_column
    );
    render_leaf_row(
        ui,
        depth,
        icons_svg::KEY,
        "foreign_key_leaf",
        &fk.name,
        Some(&detail),
    );
}

fn render_rule_row(ui: &mut egui::Ui, rule: &RuleInfo, depth: usize) {
    let status = if rule.enabled { "enabled" } else { "disabled" };
    let response = render_leaf_row(
        ui,
        depth,
        icons_svg::RULE,
        "rule_leaf",
        &rule.name,
        Some(status),
    );
    response.context_menu(|ui| {
        if ui.button(t("tree_copy_rule_ddl")).clicked() {
            ui.ctx().copy_text(rule.definition.clone());
            ui.close_menu();
        }
    });
}

fn render_trigger_row(ui: &mut egui::Ui, trigger: &TriggerInfo, depth: usize) {
    let status = if trigger.enabled {
        "enabled"
    } else {
        "disabled"
    };
    let response = render_leaf_row(
        ui,
        depth,
        icons_svg::TRIGGER,
        "trigger_leaf",
        &trigger.name,
        Some(status),
    );
    response.context_menu(|ui| {
        if ui.button(t("tree_copy_trigger_ddl")).clicked() {
            ui.ctx().copy_text(trigger.definition.clone());
            ui.close_menu();
        }
    });
}

fn render_function_row(ui: &mut egui::Ui, depth: usize, schema: &str, function: &FunctionInfo) {
    let signature = format!("{}({})", function.name, function.arguments);
    let detail = format!("{} · {}", function.kind, function.return_type);
    let response = render_leaf_row(
        ui,
        depth,
        icons_svg::FUNCTION,
        "function_leaf",
        &signature,
        Some(&detail),
    );

    response.context_menu(|ui| {
        if ui.button(t("tree_copy_signature")).clicked() {
            ui.ctx()
                .copy_text(format!("{}.{}", quote_ident(schema), signature));
            ui.close_menu();
        }
    });
}

fn render_status_row(ui: &mut egui::Ui, depth: usize, text: &str, spinner: bool) {
    let full_width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(full_width, 20.0), Sense::hover());
    let indent_x = depth_indent(depth);

    if spinner {
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
                rect.left_center() + egui::vec2(indent_x + 6.0, 0.0),
                egui::vec2(12.0, 12.0),
            )),
            |ui| {
                ui.spinner();
            },
        );
    }

    ui.painter().text(
        rect.left_center() + egui::vec2(indent_x + if spinner { 20.0 } else { 0.0 }, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_disabled(),
    );
}

fn render_leaf_row(
    ui: &mut egui::Ui,
    depth: usize,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    detail: Option<&str>,
) -> egui::Response {
    let full_width = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(full_width, 22.0), Sense::click());

    if resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(2.0, 1.0)),
            CornerRadius::same(theme::RADIUS_MD),
            theme::with_alpha(theme::ACCENT_TEAL, 16),
        );
    }

    let indent_x = depth_indent(depth);
    paint_inline_icon(ui, rect, indent_x, icon_svg, icon_name, 13.0);

    ui.painter().text(
        rect.left_center() + egui::vec2(indent_x + 18.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        theme::text_secondary(),
    );

    if let Some(detail) = detail {
        ui.painter().text(
            rect.right_center() - egui::vec2(theme::SPACE_MD, 0.0),
            egui::Align2::RIGHT_CENTER,
            detail,
            egui::FontId::proportional(10.0),
            theme::text_muted(),
        );
    }

    resp
}

fn paint_inline_icon(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    indent_x: f32,
    icon_svg: &str,
    icon_name: &str,
    size: f32,
) {
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            rect.left_center() + egui::vec2(indent_x + size / 2.0, 0.0),
            egui::vec2(size, size),
        )),
        |ui| {
            icon_img(ui, icon_svg, icon_name, size);
        },
    );
}

fn show_dark_hover_tooltip(
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

// ---------------------------------------------------------------------------
// Custom collapsing header widget
// ---------------------------------------------------------------------------

struct CollapsingResult {
    header_response: egui::Response,
}

struct CollapsingNodeSpec<'a> {
    id: egui::Id,
    label: RichText,
    dot_color: Option<Color32>,
    is_root: bool,
    depth: usize,
    default_open: bool,
    force_open: bool,
    selected: bool,
    icon_svg: &'a str,
    icon_name: &'a str,
    double_click_to_expand: bool,
    icon_tint: Option<Color32>,
}

fn collapsing_node(
    ui: &mut egui::Ui,
    spec: CollapsingNodeSpec<'_>,
    body: impl FnOnce(&mut egui::Ui),
) -> CollapsingResult {
    let mut collapse_state =
        egui::containers::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            spec.id,
            spec.default_open,
        );
    if spec.force_open {
        collapse_state.set_open(true);
    }

    let full_width = ui.available_width();
    let row_height = if spec.is_root { 26.0 } else { 22.0 };

    let (header_rect, header_resp) =
        ui.allocate_exact_size(egui::vec2(full_width, row_height), Sense::click());

    let bg = if spec.selected {
        Some(theme::with_alpha(theme::ACCENT_TEAL, 42))
    } else if header_resp.is_pointer_button_down_on() {
        Some(theme::bg_elevated())
    } else if header_resp.hovered() {
        Some(theme::with_alpha(theme::ACCENT_TEAL, 20))
    } else if spec.is_root {
        Some(theme::bg_dark())
    } else {
        None
    };

    let paint_rect = header_rect.shrink2(egui::vec2(if spec.is_root { 5.0 } else { 2.0 }, 1.0));
    if let Some(color) = bg {
        ui.painter().rect_filled(
            paint_rect,
            CornerRadius::same(if spec.is_root {
                theme::RADIUS_LG
            } else {
                theme::RADIUS_MD
            }),
            color,
        );
    }
    if spec.selected {
        ui.painter().rect_stroke(
            paint_rect,
            CornerRadius::same(theme::RADIUS_MD),
            Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 120)),
            egui::StrokeKind::Inside,
        );
        let stripe =
            egui::Rect::from_min_size(paint_rect.left_top(), egui::vec2(2.0, paint_rect.height()));
        ui.painter().rect_filled(
            stripe,
            CornerRadius::same(theme::RADIUS_SM),
            theme::ACCENT_TEAL,
        );
    }

    // Left copper accent stripe for root nodes
    if spec.is_root && !spec.selected {
        let stripe =
            egui::Rect::from_min_size(paint_rect.min, egui::vec2(2.0, paint_rect.height()));
        ui.painter().rect_filled(
            stripe,
            CornerRadius::same(theme::RADIUS_SM),
            theme::accent_copper_dim(),
        );
    }

    if spec.double_click_to_expand {
        if header_resp.double_clicked() {
            collapse_state.toggle(ui);
        }
    } else if header_resp.clicked() {
        collapse_state.toggle(ui);
    }

    let openness = collapse_state.openness(ui.ctx());

    let indent_x: f32 = depth_indent(spec.depth);

    // Chevron ▾ / ▸
    let (chevron_svg, chevron_name) = if openness > 0.5 {
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
            let chevron_color = if openness > 0.5 || header_resp.hovered() {
                theme::text_secondary()
            } else {
                theme::text_muted()
            };
            icon_img_tinted(ui, chevron_svg, chevron_name, 10.0, chevron_color);
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
            if let Some(tint) = spec.icon_tint {
                let img = icon_image(ui, spec.icon_svg, spec.icon_name, 14.0).tint(tint);
                ui.add(img);
            } else {
                icon_img(ui, spec.icon_svg, spec.icon_name, 14.0);
            }
        },
    );

    // Optional status dot
    let text_start = if let Some(color) = spec.dot_color {
        let dot_x = icon_start + 24.0;
        let dot_center = header_rect.min + egui::vec2(dot_x, row_height / 2.0);
        ui.painter().circle_filled(dot_center, 4.0, color);
        ui.painter().circle_stroke(
            dot_center,
            5.5,
            Stroke::new(0.5, theme::with_alpha(color, 60)),
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
        spec.label.text(),
        egui::FontId::proportional(if spec.is_root { 13.0 } else { 12.0 }),
        if spec.selected || spec.is_root || spec.depth <= 1 {
            theme::text_primary()
        } else {
            theme::text_secondary()
        },
    );

    collapse_state.show_body_unindented(ui, body);

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

fn depth_indent(depth: usize) -> f32 {
    8.0 + depth as f32 * 16.0
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
