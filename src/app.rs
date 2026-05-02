use crate::db::bridge::{DbBridge, DbResponse};
use crate::i18n::init_with_saved;
use crate::state::{AppState, ConnectionStatus};
use crate::storage;
use crate::ui;

pub struct FerrumGridApp {
    state: AppState,
    bridge: Option<DbBridge>,
    history: Vec<storage::history::HistoryEntry>,
    settings: storage::settings::AppSettings,
    toasts: egui_notify::Toasts,
}

impl FerrumGridApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::configure_fonts(&cc.egui_ctx);

        let settings = storage::settings::load_settings();

        // Initialize i18n system
        init_with_saved(Some(&settings.language));

        if settings.dark_mode {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
        } else {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::light());
        }

        let saved_connections = storage::connections::load_connections();
        let history = storage::history::load_history();

        let bridge = DbBridge::new(cc.egui_ctx.clone());

        let app_state = AppState {
            default_row_limit: settings.default_row_limit,
            saved_connections,
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
                    // Check if this was a test connection
                    if self.state.connection_dialog.testing {
                        let config = self.state.connection_dialog.to_config();
                        if config.id == conn_id {
                            self.state.connection_dialog.testing = false;
                            self.state.connection_dialog.test_result = Some(Ok(
                                format!("Connected! PostgreSQL {server_version}"),
                            ));
                            // Disconnect the test connection
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

                    // Auto-load schemas
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

                    // Add to history
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
                DbResponse::ForeignKeyList { conn_id: _, schema, foreign_keys } => {
                    crate::ui::er_diagram::handle_fk_response(&mut self.state, &schema, &foreign_keys);
                    crate::ui::table_designer::apply_fk_info(&mut self.state, &foreign_keys);
                }
                DbResponse::QueryCancelled { conn_id: _ } => {
                    self.state.query_running = false;
                    self.state.last_error = Some("Query cancelled".to_string());
                }
                DbResponse::Error { conn_id, error } => {
                    // Check if this was a test connection error
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
}

impl eframe::App for FerrumGridApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.process_responses();

        let bridge = self.bridge.as_ref().unwrap();
        ui::panels::render_panels(ctx, &mut self.state, bridge);
        ui::dialogs::render_connection_dialog(ctx, &mut self.state, bridge);
        ui::er_diagram::render_er_diagram(ctx, &mut self.state, bridge);
        ui::table_designer::render_table_designer(ctx, &mut self.state, bridge);
        crate::prisma::ui::render_prisma_window(ctx, &mut self.state, bridge);

        self.toasts.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save state
        storage::connections::save_connections(&self.state.saved_connections);
        storage::settings::save_settings(&self.settings);

        // Store passwords in keyring
        for config in &self.state.saved_connections {
            if !config.password.is_empty() {
                storage::connections::store_password(&config.id, &config.password);
            }
        }

        // Disconnect all and drop bridge
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
