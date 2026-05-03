use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, AppState, ConnectionState, ConnectionStatus, DataSource,
};
use crate::types::{
    ColumnInfo, ConnectionConfig, ConnectionId, FunctionInfo, IndexInfo, RuleInfo, TableInfo,
    TriggerInfo,
};
use crate::ui::{icon_img, icon_img_tinted, icons_svg, theme};

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

    let saved_connections: Vec<ConnectionConfig> = state
        .saved_connections
        .iter()
        .filter(|config| !state.connections.contains_key(&config.id))
        .cloned()
        .collect();

    if !saved_connections.is_empty() {
        if !state.connections.is_empty() {
            ui.add_space(theme::SPACE_SM);
            ui.separator();
            ui.add_space(theme::SPACE_SM);
        }

        for config in saved_connections {
            render_saved_connection_row(ui, state, bridge, config);
            ui.add_space(theme::SPACE_XS);
        }
    }

    render_explorer_empty_space(ui, state, bridge);
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
        node_id,
        header_text,
        dot_color,
        true,
        0,
        true,
        icons_svg::DATABASE,
        "conn",
        |ui| {
            if !is_connected && !is_connecting {
                indented(ui, |ui| {
                    ui.label(
                        RichText::new("Not connected")
                            .color(theme::text_muted())
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

            if databases.is_empty() {
                let loading = state
                    .connections
                    .get(&conn_id)
                    .map_or(false, |c| c.loading_databases);
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
) {
    let detail = format!("{}:{}/{}", config.host, config.port, config.database);
    let response = render_leaf_row(
        ui,
        0,
        icons_svg::DATABASE,
        "saved_connection",
        &config.display_name,
        Some(&detail),
    )
    .on_hover_text("Connect");

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
    state.connection_dialog.clipboard_import_checked = true;
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
        node_id,
        header_text,
        None,
        false,
        1,
        true,
        icons_svg::DATABASE,
        "database",
        |ui| {
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

    let header_text = RichText::new(schema)
        .color(theme::text_secondary())
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        2,
        false,
        icons_svg::SCHEMA,
        "schema",
        |ui| {
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

    resp.header_response.context_menu(|ui| {
        if ui.button(t("ctx_new_table")).clicked() {
            ui.close_menu();
            crate::ui::table_designer::open_for_new_table_with_schema(state, schema);
        }
    });
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
    let header_text = RichText::new(group.label())
        .color(theme::text_secondary())
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        3,
        false,
        icon_svg,
        icon_name,
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
    let header_text = RichText::new(t("tree_functions"))
        .color(theme::text_secondary())
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        3,
        false,
        icons_svg::FUNCTION,
        "group_functions",
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
    let header_text = RichText::new(t("tree_queries"))
        .color(theme::text_secondary())
        .size(12.0);

    collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        3,
        false,
        icons_svg::QUERY,
        "group_queries",
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
    let header_text = RichText::new(t("tree_backups"))
        .color(theme::text_secondary())
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        3,
        false,
        icons_svg::BACKUP,
        "group_backups",
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

    let header_text = RichText::new(table_name)
        .color(theme::text_primary())
        .size(12.0);

    let resp = collapsing_node(
        ui,
        node_id,
        header_text,
        None,
        false,
        4,
        false,
        icon_svg,
        icon_name,
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

    if resp.header_response.double_clicked() {
        open_table_data(state, bridge, conn_id, schema, table_name);
    }

    resp.header_response.context_menu(|ui| {
        if !matches!(table_type, "VIEW" | "MATERIALIZED VIEW") {
            let edit_resp = ui.button(format!("      {}", t("tree_edit_table")));
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
                crate::ui::table_designer::open_for_existing_table(
                    state, schema, table_name, bridge,
                );
                ui.close_menu();
            }

            ui.separator();
        }

        let view_resp = ui.button(format!("      {}", t("tree_view_data_top_100")));
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
            open_table_data(state, bridge, conn_id, schema, table_name);
            ui.close_menu();
        }

        let copy_resp = ui.button(format!("      {}", t("tree_copy_select")));
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

        let refresh_resp = ui.button(format!("      {}", t("tree_refresh_metadata")));
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
            Some(columns) if columns.is_empty() => {
                render_status_row(ui, 6, &t("tree_empty"), false)
            }
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
            Some(indexes) if indexes.is_empty() => {
                render_status_row(ui, 6, &t("tree_empty"), false)
            }
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
            Some(keys) if keys.is_empty() => render_status_row(ui, 6, &t("tree_empty"), false),
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
            Some(indexes) if indexes.is_empty() => {
                render_status_row(ui, 6, &t("tree_empty"), false)
            }
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
            Some(rules) if rules.is_empty() => render_status_row(ui, 6, &t("tree_empty"), false),
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
            Some(triggers) if triggers.is_empty() => {
                render_status_row(ui, 6, &t("tree_empty"), false)
            }
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
        egui::Id::new(format!("table_meta_{conn_id}_{schema}_{table}_{group_key}")),
        header_text,
        None,
        false,
        5,
        default_open,
        icon_svg,
        icon_name,
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
    depth: usize,
    default_open: bool,
    icon_svg: &str,
    icon_name: &str,
    body: impl FnOnce(&mut egui::Ui),
) -> CollapsingResult {
    let mut collapse_state =
        egui::containers::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id,
            default_open,
        );

    let full_width = ui.available_width();
    let row_height = if is_root { 26.0 } else { 22.0 };

    let (header_rect, header_resp) =
        ui.allocate_exact_size(egui::vec2(full_width, row_height), Sense::click());

    let bg = if header_resp.is_pointer_button_down_on() {
        Some(theme::bg_elevated())
    } else if header_resp.hovered() {
        Some(theme::with_alpha(theme::ACCENT_TEAL, 20))
    } else if is_root {
        Some(theme::bg_dark())
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
            theme::accent_copper_dim(),
        );
    }

    if header_resp.clicked() {
        collapse_state.toggle(ui);
    }

    let openness = collapse_state.openness(ui.ctx());

    let indent_x: f32 = depth_indent(depth);

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
        label.text(),
        egui::FontId::proportional(if is_root { 13.0 } else { 12.0 }),
        if is_root || depth <= 1 {
            theme::text_primary()
        } else if depth >= 4 {
            theme::text_secondary()
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
