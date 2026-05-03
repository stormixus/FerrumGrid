use crate::db::bridge::{DbBridge, DbResponse};
use crate::i18n::init_with_saved;
use crate::state::{
    build_data_select_sql_with_columns, AppState, ConnectionStatus, MainView, VaultUiState,
};
use crate::storage;
use crate::types::CellValue;
use crate::ui;
use eframe::egui;
use std::sync::Arc;

pub struct FerrumGridApp {
    state: AppState,
    bridge: Option<DbBridge>,
    history: Vec<storage::history::HistoryEntry>,
    settings: storage::settings::AppSettings,
    native_menu: crate::native_menu::NativeMenu,
    toasts: egui_notify::Toasts,
}

fn reload_enum_text_projection_if_needed(state: &mut AppState, bridge: &DbBridge) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    let columns = state.data_columns_for_source(&source);
    if !columns.iter().any(|column| !column.enum_values.is_empty()) {
        return false;
    }
    let Some(result) = state.current_result.as_ref() else {
        return false;
    };

    let needs_reload = result
        .columns
        .iter()
        .enumerate()
        .any(|(col_idx, result_col)| {
            columns
                .iter()
                .find(|column| column.name == result_col.name && !column.enum_values.is_empty())
                .is_some_and(|_| {
                    result.rows.iter().any(|row| {
                        matches!(
                            row.get(col_idx),
                            Some(CellValue::Unknown(value)) if value == "<unsupported>"
                        )
                    })
                })
        });
    if !needs_reload {
        return false;
    }

    let limit = state.data_edit.page_limit;
    let offset = state.data_edit.page_index.saturating_mul(limit);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    bridge.send(crate::db::bridge::DbCommand::ExecuteQuery {
        conn_id: source.conn_id,
        sql: build_data_select_sql_with_columns(
            &source,
            &state.data_edit.sort,
            limit,
            offset,
            &columns,
        ),
        row_limit: Some(limit),
    });
    true
}

impl FerrumGridApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        ui::theme::configure_fonts(&cc.egui_ctx);

        let mut settings = storage::settings::load_settings();

        // Initialize i18n system
        init_with_saved(Some(&settings.language));

        settings.dark_mode = ui::theme::apply_appearance(&cc.egui_ctx, &settings.appearance);
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                crate::app_icon::icon_for_dark_mode(settings.dark_mode),
            ))));

        let (saved_connections, vault) = match storage::connections::load_storage_state() {
            storage::connections::ConnectionStorageState::Empty => {
                (Vec::new(), VaultUiState::setup_required(Vec::new()))
            }
            storage::connections::ConnectionStorageState::Legacy(connections) => {
                (Vec::new(), VaultUiState::setup_required(connections))
            }
            storage::connections::ConnectionStorageState::VaultUnlocked {
                connections,
                session,
            } => (connections, VaultUiState::unlocked(session)),
            storage::connections::ConnectionStorageState::VaultLocked { name } => {
                (Vec::new(), VaultUiState::locked(name))
            }
            storage::connections::ConnectionStorageState::Corrupt(error) => {
                let mut vault = VaultUiState::locked("Personal".to_string());
                vault.error = Some(error);
                (Vec::new(), vault)
            }
        };
        let history = storage::history::load_history();

        let bridge = DbBridge::new(cc.egui_ctx.clone());

        let should_show_connection_dialog = saved_connections.is_empty() && vault.is_unlocked();
        let app_state = AppState {
            default_row_limit: settings.default_row_limit,
            data_timezone: settings.data_timezone.clone(),
            show_connection_dialog: should_show_connection_dialog,
            saved_connections,
            vault,
            ..Default::default()
        };

        Self {
            state: app_state,
            bridge: Some(bridge),
            history,
            settings,
            native_menu: crate::native_menu::NativeMenu::install(),
            toasts: egui_notify::Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
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
                            self.state.connection_dialog.test_result =
                                Some(Ok(format!("Connected! PostgreSQL {server_version}")));
                            // Disconnect the test connection
                            bridge.send(crate::db::bridge::DbCommand::Disconnect { conn_id });
                            continue;
                        }
                    }

                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.status = ConnectionStatus::Connected {
                            server_version: server_version.clone(),
                        };
                        conn.loading_databases = true;
                        conn.loading_schemas = true;
                    }
                    self.state.status_message = format!("Connected (PostgreSQL {server_version})");

                    // Auto-load the top-level browser model.
                    bridge.send(crate::db::bridge::DbCommand::ListDatabases { conn_id });
                    bridge.send(crate::db::bridge::DbCommand::ListSchemas { conn_id });

                    self.toasts
                        .info(format!("Connected to PostgreSQL {server_version}"));
                }
                DbResponse::DatabaseList {
                    conn_id,
                    mut databases,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        if !databases.iter().any(|name| name == &conn.config.database) {
                            databases.push(conn.config.database.clone());
                            databases.sort();
                        }
                        conn.databases = databases;
                        conn.loading_databases = false;
                    }
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
                    if self.state.active_main_view == MainView::Query {
                        if let Some(conn) = self.state.connections.get(&conn_id) {
                            if let Some(tab) = self.state.editor_tabs.get(self.state.active_tab) {
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
                    }

                    self.state.current_result = Some(result);
                    self.state.current_result_truncated = truncated;
                    if self.state.active_main_view == MainView::Data {
                        self.state.reset_data_edits_for_current_result(conn_id);
                        if reload_enum_text_projection_if_needed(&mut self.state, bridge) {
                            continue;
                        }
                    }
                    self.state.last_error = None;
                }
                DbResponse::DataEditsApplied { conn_id, applied } => {
                    self.state.data_edit.applying = false;
                    self.state.status_message = format!("{applied} data change(s) applied");
                    self.toasts
                        .info(format!("{applied} data change(s) applied"));

                    if let Some(source) = self.state.data_edit.source.clone() {
                        self.state.current_result = None;
                        self.state.current_result_truncated = false;
                        self.state.query_running = true;
                        let limit = self.state.data_edit.page_limit;
                        let offset = self.state.data_edit.page_index.saturating_mul(limit);
                        let columns = self.state.data_columns_for_source(&source);
                        bridge.send(crate::db::bridge::DbCommand::ExecuteQuery {
                            conn_id,
                            sql: build_data_select_sql_with_columns(
                                &source,
                                &self.state.data_edit.sort,
                                limit,
                                offset,
                                &columns,
                            ),
                            row_limit: Some(limit),
                        });
                    }
                }
                DbResponse::SchemaList { conn_id, schemas } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.schemas = schemas;
                        conn.loading_schemas = false;
                    }
                }
                DbResponse::TableList {
                    conn_id,
                    schema,
                    tables,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.loading_tables.remove(&schema);
                        conn.tables.insert(schema, tables);
                    }
                }
                DbResponse::ColumnList {
                    conn_id,
                    schema,
                    table,
                    columns,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.loading_columns.remove(&key);
                        conn.columns.insert(key, columns);
                    }
                }
                DbResponse::IndexList {
                    conn_id,
                    schema,
                    table,
                    indexes,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.loading_indexes.remove(&key);
                        conn.indexes.insert(key, indexes);
                    }
                }
                DbResponse::ForeignKeyList {
                    conn_id,
                    schema,
                    foreign_keys,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.loading_foreign_keys.remove(&schema);
                        conn.foreign_keys
                            .insert(schema.clone(), foreign_keys.clone());
                    }
                    if self.state.active_connection == Some(conn_id) {
                        crate::ui::er_diagram::handle_fk_response(
                            &mut self.state,
                            &schema,
                            &foreign_keys,
                        );
                        crate::ui::table_designer::apply_fk_info(&mut self.state, &foreign_keys);
                    }
                }
                DbResponse::RuleList {
                    conn_id,
                    schema,
                    table,
                    rules,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.loading_rules.remove(&key);
                        conn.rules.insert(key, rules);
                    }
                }
                DbResponse::TriggerList {
                    conn_id,
                    schema,
                    table,
                    triggers,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        let key = (schema, table);
                        conn.loading_triggers.remove(&key);
                        conn.triggers.insert(key, triggers);
                    }
                }
                DbResponse::FunctionList {
                    conn_id,
                    schema,
                    functions,
                } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.loading_functions.remove(&schema);
                        conn.functions.insert(schema, functions);
                    }
                }
                DbResponse::RoleList { conn_id, roles } => {
                    if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                        conn.loading_roles = false;
                        conn.roles = roles;
                    }
                }
                DbResponse::QueryCancelled { conn_id } => {
                    self.state.query_running = false;
                    self.state.last_error = Some("Query cancelled".to_string());
                    self.state.status_message = format!("Query cancelled on {conn_id}");
                }
                DbResponse::BackupCompleted { record } => {
                    self.state.backup_running = false;
                    self.state.backup_last_error = None;
                    self.state.status_message = format!(
                        "Backup completed: {}",
                        record
                            .file_path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("backup")
                    );
                    self.state.backup_history.insert(0, record.clone());
                    self.state.backup_history.truncate(20);
                    self.toasts
                        .info(format!("Backup saved to {}", record.file_path.display()));
                }
                DbResponse::BackupFailed { conn_id, error } => {
                    self.state.backup_running = false;
                    self.state.backup_last_error = Some(error.clone());
                    self.state.status_message = format!("Backup failed on {conn_id}");
                    self.toasts.error(format!("Backup failed: {error}"));
                }
                DbResponse::Error { conn_id, error } => {
                    self.state.data_edit.applying = false;
                    tracing::warn!(
                        conn_id = %error.conn_id,
                        category = ?error.category,
                        "database error: {}",
                        error.message
                    );

                    // Check if this was a test connection error
                    if self.state.connection_dialog.testing {
                        self.state.connection_dialog.testing = false;
                        self.state.connection_dialog.test_result = Some(Err(error.to_string()));
                        continue;
                    }

                    self.state.query_running = false;

                    match error.category {
                        crate::db::error::ErrorCategory::Connection => {
                            if let Some(conn) = self.state.connections.get_mut(&conn_id) {
                                conn.status = ConnectionStatus::Disconnected;
                            }
                            self.state.status_message =
                                format!("Connection error: {}", error.message);
                            self.toasts
                                .error(format!("Connection lost: {}", error.message));
                        }
                        crate::db::error::ErrorCategory::Query => {
                            self.state.last_error = Some(error.to_string());
                        }
                        crate::db::error::ErrorCategory::Internal => {
                            self.toasts.error(error.message.clone());
                        }
                        crate::db::error::ErrorCategory::Cancelled => {
                            self.state.last_error = Some("Query cancelled".to_string());
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
        self.native_menu
            .handle_events(ctx, &mut self.state, &mut self.settings);

        if !self.state.vault.is_unlocked() {
            ui::vault::render_vault_window(ctx, &mut self.state);
            self.toasts.show(ctx);
            return;
        }

        let bridge = self.bridge.as_ref().unwrap();
        ui::panels::render_panels(ctx, &mut self.state, bridge, &mut self.settings);
        ui::dialogs::render_connection_dialog(ctx, &mut self.state, bridge);
        ui::about::render_about_window(ctx, &mut self.state);
        let previous_dark_mode = self.settings.dark_mode;
        if ui::settings::render_settings_window(ctx, &mut self.state, &mut self.settings) {
            self.native_menu.refresh_locale();
        }
        if self.settings.dark_mode != previous_dark_mode {
            ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                crate::app_icon::icon_for_dark_mode(self.settings.dark_mode),
            ))));
        }
        ui::er_diagram::render_er_diagram(ctx, &mut self.state, bridge);
        ui::table_designer::render_table_designer(ctx, &mut self.state, bridge);
        crate::prisma::ui::render_prisma_window(ctx, &mut self.state, bridge);

        self.toasts.show(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save state
        if let Some(session) = self.state.vault.session.as_ref() {
            if let Err(err) =
                storage::connections::save_connections(&self.state.saved_connections, session)
            {
                tracing::warn!("failed to save encrypted vault: {err}");
            }
        }
        storage::settings::save_settings(&self.settings);

        // Disconnect all and drop bridge
        if let Some(bridge) = &self.bridge {
            for conn_id in self.state.connections.keys() {
                bridge.send(crate::db::bridge::DbCommand::Disconnect { conn_id: *conn_id });
            }
        }
        self.bridge = None;
    }
}
