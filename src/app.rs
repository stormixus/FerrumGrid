use crate::db::bridge::{DbBridge, DbCommand, DbResponse};
use crate::menu::{self, MenuAction};
use crate::state::{AppState, ConnectionStatus};
use crate::storage;
use crate::types::EditorTab;
use crate::ui;
use crate::ui::theme::{FerrumTheme, ThemeMode};

pub struct FerrumGridApp {
    state: AppState,
    bridge: Option<DbBridge>,
    history: Vec<storage::history::HistoryEntry>,
    settings: storage::settings::AppSettings,
    toasts: egui_notify::Toasts,
}

impl FerrumGridApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        FerrumTheme::init(&cc.egui_ctx);

        // Install native macOS menu bar (no-op on other platforms).
        menu::install();

        let settings = storage::settings::load_settings();
        // Apply persisted theme preference up-front (Auto/Light/Dark)
        FerrumTheme::apply_mode(&cc.egui_ctx, settings.theme);

        let saved_connections = storage::connections::load_connections();
        let history = storage::history::load_history();

        let bridge = DbBridge::new(cc.egui_ctx.clone());

        let app_state = AppState {
            default_row_limit: settings.default_row_limit,
            saved_connections,
            theme_mode: settings.theme,
            sidebar_visible: settings.sidebar_visible,
            result_panel_visible: settings.result_panel_visible,
            ..Default::default()
        };

        Self {
            state: app_state,
            bridge: Some(bridge),
            history,
            settings,
            toasts: egui_notify::Toasts::default()
                .with_anchor(egui_notify::Anchor::BottomRight),
        }
    }

    fn process_responses(&mut self) {
        let bridge = match &self.bridge {
            Some(b) => b,
            None => return,
        };

        while let Some(response) = bridge.try_recv() {
            match response {
                DbResponse::Connected {
                    conn_id,
                    server_version,
                } => {
                    if self.state.connection_dialog.testing {
                        let config = self.state.connection_dialog.to_config();
                        if config.id == conn_id {
                            self.state.connection_dialog.testing = false;
                            self.state.connection_dialog.test_result = Some(Ok(
                                format!("Connected! PostgreSQL {server_version}"),
                            ));
                            bridge.send(crate::db::bridge::DbCommand::Disconnect {
                                conn_id,
                            });
                            continue;
                        }
                    }

                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.status = ConnectionStatus::Connected {
                            server_version: server_version.clone(),
                        };
                    }
                    self.state.status_message =
                        format!("Connected (PostgreSQL {server_version})");

                    bridge.send(crate::db::bridge::DbCommand::ListSchemas {
                        conn_id,
                    });

                    self.toasts
                        .info(format!("Connected to PostgreSQL {server_version}"));
                }
                DbResponse::Disconnected { conn_id } => {
                    self.state.connections.remove(&conn_id);
                    if self.state.active_connection == Some(conn_id) {
                        self.state.active_connection =
                            self.state.connections.keys().next().copied();
                    }
                    if self.state.connections.is_empty() {
                        self.state.status_message = "Disconnected".to_string();
                    }
                }
                DbResponse::QueryResult {
                    conn_id,
                    result,
                    truncated,
                } => {
                    self.state.query_running = false;

                    if let Some(conn) = self.state.connections.get(&conn_id) {
                        if let Some(tab) =
                            self.state.editor_tabs.get(self.state.active_tab)
                        {
                            storage::history::add_entry(
                                &mut self.history,
                                storage::history::HistoryEntry {
                                    query: tab.content.clone(),
                                    timestamp: chrono::Utc::now(),
                                    duration_ms: result.execution_time_ms,
                                    row_count: result.rows.len(),
                                    conn_id,
                                    conn_name: conn.config.display_name.clone(),
                                    truncated,
                                },
                            );
                        }
                    }

                    self.state.current_result = Some(result);
                    self.state.current_result_truncated = truncated;
                    self.state.last_error = None;
                }
                DbResponse::SchemaList { conn_id, schemas } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.schemas = schemas;
                        conn.loading_schemas = false;
                    }
                }
                DbResponse::TableList { conn_id, schema, tables } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.loading_tables.remove(&schema);
                        conn.tables.insert(schema, tables);
                    }
                }
                DbResponse::ColumnList { conn_id, schema, table, columns } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.loading_columns.remove(&key);
                        conn.columns.insert(key, columns);
                    }
                }
                DbResponse::IndexList { conn_id, schema, table, indexes } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.indexes.insert(key, indexes);
                    }
                }
                DbResponse::QueryCancelled { conn_id: _ } => {
                    self.state.query_running = false;
                    self.state.last_error = Some("Query cancelled".to_string());
                }
                DbResponse::Error { conn_id, error } => {
                    if self.state.connection_dialog.testing {
                        self.state.connection_dialog.testing = false;
                        self.state.connection_dialog.test_result =
                            Some(Err(error.to_string()));
                        continue;
                    }

                    self.state.query_running = false;

                    match error.category {
                        crate::db::error::ErrorCategory::Connection => {
                            if let Some(conn) =
                                self.state.connections.get_mut(&conn_id)
                            {
                                conn.status = ConnectionStatus::Disconnected;
                            }
                            self.state.status_message =
                                format!("Connection error: {}", error.message);
                            self.toasts.error(format!(
                                "Connection lost: {}",
                                error.message
                            ));
                        }
                        crate::db::error::ErrorCategory::Query => {
                            self.state.last_error = Some(error.to_string());
                        }
                        crate::db::error::ErrorCategory::Internal => {
                            self.toasts.error(error.message.clone());
                        }
                        crate::db::error::ErrorCategory::Cancelled => {
                            self.state.last_error =
                                Some("Query cancelled".to_string());
                        }
                    }
                }
            }
        }
    }

    fn dispatch_menu_actions(&mut self) {
        // Drain queues from both sources: native macOS NSMenu (muda) and the
        // in-app menu bar (panels.rs pushes into state.pending_menu_actions).
        let in_app: Vec<MenuAction> =
            std::mem::take(&mut self.state.pending_menu_actions);
        let native = menu::drain();

        for action in in_app.into_iter().chain(native) {
            match action {
                MenuAction::NewConnection => {
                    self.state.show_connection_dialog = true;
                    self.state.connection_dialog = Default::default();
                }
                MenuAction::NewQueryTab => {
                    let n = self.state.editor_tabs.len() + 1;
                    self.state
                        .editor_tabs
                        .push(EditorTab::new(format!("Query {n}")));
                    self.state.active_tab = self.state.editor_tabs.len() - 1;
                }
                MenuAction::CloseTab => {
                    if self.state.editor_tabs.len() > 1 {
                        let idx = self.state.active_tab;
                        self.state.editor_tabs.remove(idx);
                        if self.state.active_tab >= self.state.editor_tabs.len() {
                            self.state.active_tab = self.state.editor_tabs.len() - 1;
                        }
                    }
                }
                MenuAction::RunQuery => {
                    if let (Some(conn_id), Some(bridge)) =
                        (self.state.active_connection, &self.bridge)
                    {
                        if let Some(tab) =
                            self.state.editor_tabs.get(self.state.active_tab)
                        {
                            let sql = tab.content.trim().to_string();
                            if !sql.is_empty() && !self.state.query_running {
                                self.state.query_running = true;
                                self.state.last_error = None;
                                bridge.send(DbCommand::ExecuteQuery {
                                    conn_id,
                                    sql,
                                    row_limit: Some(self.state.default_row_limit),
                                });
                            }
                        }
                    }
                }
                MenuAction::StopQuery => {
                    if let (Some(conn_id), Some(bridge)) =
                        (self.state.active_connection, &self.bridge)
                    {
                        if self.state.query_running {
                            bridge.send(DbCommand::CancelQuery { conn_id });
                        }
                    }
                }
                MenuAction::OpenCommandPalette => {
                    self.state.command_palette.open();
                }
                MenuAction::ToggleSidebar => {
                    self.state.sidebar_visible = !self.state.sidebar_visible;
                }
                MenuAction::ToggleResultPanel => {
                    self.state.result_panel_visible = !self.state.result_panel_visible;
                }
                MenuAction::ThemeAuto => self.state.theme_mode = ThemeMode::Auto,
                MenuAction::ThemeLight => self.state.theme_mode = ThemeMode::Light,
                MenuAction::ThemeDark => self.state.theme_mode = ThemeMode::Dark,
                MenuAction::About => {
                    self.toasts.info(format!(
                        "FerrumGrid {}",
                        env!("CARGO_PKG_VERSION")
                    ));
                }
            }
        }
    }
}

impl eframe::App for FerrumGridApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.dispatch_menu_actions();
        self.process_responses();

        let bridge = self.bridge.as_ref().unwrap();
        ui::panels::render_app(ctx, &mut self.state, bridge, &self.history);
        ui::dialogs::render_connection_dialog(ctx, &mut self.state, bridge);

        self.toasts.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Sync UI state back to settings
        self.settings.theme = self.state.theme_mode;
        self.settings.sidebar_visible = self.state.sidebar_visible;
        self.settings.result_panel_visible = self.state.result_panel_visible;

        storage::connections::save_connections(&self.state.saved_connections);
        storage::settings::save_settings(&self.settings);

        for config in &self.state.saved_connections {
            if !config.password.is_empty() {
                storage::connections::store_password(&config.id, &config.password);
            }
        }

        if let Some(bridge) = &self.bridge {
            for conn_id in self.state.connections.keys() {
                bridge.send(crate::db::bridge::DbCommand::Disconnect {
                    conn_id: *conn_id,
                });
            }
        }
        self.bridge = None;
    }
}
