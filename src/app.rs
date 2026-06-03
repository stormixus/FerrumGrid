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
    quit_requested: bool,
    /// Plan v7 Phase 4b3 — automation scheduler runner. 첫 Connected event 에서
    /// 1회 spawn. (handle, shutdown_tx, done_rx). done_rx 는 thread 가 run_scheduler
    /// 종료 후 신호를 보내며, on_exit 에서 recv_timeout 으로 graceful join.
    automation_runner: Option<(
        std::thread::JoinHandle<()>,
        tokio::sync::oneshot::Sender<()>,
        std::sync::mpsc::Receiver<()>,
    )>,
    update_check: crate::updater::UpdateCheck,
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

        let mut settings = storage::settings::load_settings();
        ui::theme::configure_fonts(&cc.egui_ctx, &settings.language);

        // Initialize i18n system
        init_with_saved(Some(&settings.language));

        settings.dark_mode = ui::theme::apply_appearance(&cc.egui_ctx, &settings.appearance, &settings.accent_color);
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                crate::app_icon::icon_for_dark_mode(settings.dark_mode),
            ))));

        #[cfg(target_os = "macos")]
        crate::ui::titlebar::configure_macos_titlebar();

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

        let backup_history = crate::storage::backups::load_backups();

        let should_show_connection_dialog = saved_connections.is_empty() && vault.is_unlocked();
        let app_state = AppState {
            default_row_limit: settings.default_row_limit,
            data_timezone: settings.data_timezone.clone(),
            show_connection_dialog: should_show_connection_dialog,
            saved_connections,
            vault,
            query_history: history.clone(),
            backup_history,
            ..Default::default()
        };

        crate::dock_menu::install();

        let mut update_check = crate::updater::UpdateCheck::default();
        update_check.start();

        Self {
            state: app_state,
            bridge: Some(bridge),
            history,
            settings,
            native_menu: crate::native_menu::NativeMenu::install(),
            toasts: egui_notify::Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
            quit_requested: false,
            automation_runner: None,
            update_check,
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
                        conn.connection_error = None;
                        conn.loading_databases = true;
                        conn.loading_schemas = true;
                    }
                    self.state.status_message = format!("Connected (PostgreSQL {server_version})");

                    // Plan v7 Phase 4b3 — automation scheduler runner 1회 spawn (첫 connection).
                    if self.automation_runner.is_none() {
                        let store = self.state.automation.clone();
                        let cmd_tx = bridge.cmd_sender();
                        let runner_conn_id = conn_id;
                        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
                        let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
                        let handle = std::thread::Builder::new()
                            .name("ferrumgrid-automation".to_string())
                            .spawn(move || {
                                let rt = tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build()
                                    .expect("automation runtime");
                                rt.block_on(crate::automation::runner::run_scheduler(
                                    store,
                                    cmd_tx,
                                    runner_conn_id,
                                    shutdown_rx,
                                ));
                                let _ = done_tx.send(());
                            })
                            .expect("spawn automation runner");
                        self.automation_runner = Some((handle, shutdown_tx, done_rx));
                        tracing::info!("automation runner spawned for conn {conn_id:?}");
                    }

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
                        // Clear tx state when the active connection disconnects.
                        self.state.explicit_tx_active = false;
                        self.state.explicit_tx_started = None;
                        self.state.explicit_tx_warned = false;
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
                                self.state.query_history.clone_from(&self.history);
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
                DbResponse::DataEditsApplied { conn_id, outcome } => {
                    self.state.data_edit.applying = false;
                    let applied = outcome.applied;
                    // outcome.inserted_keys 는 Phase 1.2 (Tmp→Pk 재매핑) 에서 소비.
                    self.state.status_message = format!("{applied} data change(s) applied");
                    self.toasts
                        .info(format!("{applied} data change(s) applied"));
                    self.state.diagnostics_panel.push_mutation_diagnostic(
                        crate::ui::diagnostics_panel::DiagSeverity::Info,
                        format!("{applied} data change(s) applied"),
                    );

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
                    if let Some(ref wizard_lock) = self.state.backup_wizard_state {
                        let mut wizard = wizard_lock.lock().unwrap();
                        wizard.running = false;
                        wizard.completed = true;
                        wizard.error = None;
                        wizard.progress = 1.0;
                    }
                    self.state.backup_history.insert(0, record.clone());
                    self.state.backup_history.truncate(100);
                    crate::storage::backups::save_backups(&self.state.backup_history);
                    self.toasts
                        .info(format!("Backup saved to {}", record.file_path.display()));
                }
                DbResponse::BackupFailed { conn_id, error } => {
                    self.state.backup_running = false;
                    self.state.backup_last_error = Some(error.clone());
                    self.state.status_message = format!("Backup failed on {conn_id}");
                    if let Some(ref wizard_lock) = self.state.backup_wizard_state {
                        let mut wizard = wizard_lock.lock().unwrap();
                        wizard.running = false;
                        wizard.completed = true;
                        wizard.error = Some(error.clone());
                    }
                    self.toasts.error(format!("Backup failed: {error}"));
                    self.state
                        .diagnostics_panel
                        .push_backup_error(format!("Backup failed: {error}"));
                }
                DbResponse::BackupProgress {
                    conn_id: _,
                    progress,
                    current_table,
                } => {
                    if let Some(ref wizard_lock) = self.state.backup_wizard_state {
                        let mut wizard = wizard_lock.lock().unwrap();
                        wizard.progress = progress;
                        wizard.current_table = current_table;
                    }
                }
                DbResponse::RestoreCompleted { file_path } => {
                    if let Some(ref mut dialog) = self.state.restore_confirm_dialog {
                        dialog.running = false;
                        dialog.completed = true;
                        dialog.error = None;
                        dialog.progress = 1.0;
                    }
                    self.toasts
                        .success(format!("Restore completed from {}", file_path.display()));
                }
                DbResponse::RestoreFailed { file_path: _, error } => {
                    if let Some(ref mut dialog) = self.state.restore_confirm_dialog {
                        dialog.running = false;
                        dialog.completed = true;
                        dialog.error = Some(error.clone());
                    }
                    self.toasts.error(format!("Restore failed: {}", error));
                }
                DbResponse::AutomationResult {
                    conn_id: _,
                    task_id,
                    result,
                } => {
                    // Plan v7 Phase 4b3 — runner / Run-Now 응답을 AutomationStore 에 적용.
                    use crate::automation::scheduler::ApplyResult;
                    if let Ok(mut store) = self.state.automation.write() {
                        store.mark_run(task_id, chrono::Utc::now(), result.clone());
                    }
                    // 실행 후 last_run/next_run 갱신을 디스크에 영속화.
                    crate::ui::objects::persist_automation(&self.state);
                    match result {
                        ApplyResult::Success { rows_affected } => {
                            self.toasts.info(format!(
                                "Automation {} succeeded ({} rows)",
                                task_id, rows_affected
                            ));
                        }
                        ApplyResult::Failed { error } => {
                            self.toasts
                                .error(format!("Automation {} failed: {}", task_id, error));
                            self.state.diagnostics_panel.push_mutation_diagnostic(
                                crate::ui::diagnostics_panel::DiagSeverity::Error,
                                format!("Automation {task_id} failed: {error}"),
                            );
                        }
                    }
                }
                DbResponse::ExplicitTxChanged { conn_id: _, active } => {
                    if active {
                        self.state.explicit_tx_active = true;
                        self.state.explicit_tx_started = Some(std::time::Instant::now());
                        self.state.explicit_tx_warned = false;
                        self.state.status_message = "Transaction active (BEGIN)".to_string();
                    } else {
                        self.state.explicit_tx_active = false;
                        self.state.explicit_tx_started = None;
                        self.state.explicit_tx_warned = false;
                    }
                }
                DbResponse::DependentsList {
                    conn_id: _,
                    refobjid: _,
                    deps,
                    truncated,
                } => {
                    // US-K1 — Drop dialog 의 dependents 채우기 + loading 종료.
                    if let Some(dlg) = self.state.drop_dialog.as_mut() {
                        dlg.dependents = deps;
                        dlg.truncated = truncated;
                        dlg.loading = false;
                    }
                }
                DbResponse::SchemaDiffResult { diff } => {
                    self.state.migration_wizard.loading_diff = false;
                    let (added, modified, removed) = diff.summary_counts();
                    self.state.status_message = format!(
                        "Schema diff: {added} added, {modified} modified, {removed} removed"
                    );
                    self.state.migration_wizard.diff = Some(diff);
                    self.state.migration_wizard.go_to(
                        crate::state::migration::MigrationStep::DiffResult,
                    );
                }
                DbResponse::SchemaDiffError { error } => {
                    self.state.migration_wizard.loading_diff = false;
                    self.state.migration_wizard.apply_error = Some(error.clone());
                    self.state.status_message = format!("Schema diff failed: {error}");
                }
                DbResponse::MigrationApplied => {
                    self.state.migration_wizard.applying = false;
                    self.state.migration_wizard.apply_success = true;
                    self.state.migration_wizard.go_to(
                        crate::state::migration::MigrationStep::Complete,
                    );
                    self.state.status_message = "Migration applied successfully".to_string();
                    self.toasts
                        .info("Migration applied successfully")
                        .duration(Some(std::time::Duration::from_secs(5)));
                }
                DbResponse::MigrationFailed { error } => {
                    self.state.migration_wizard.applying = false;
                    self.state.migration_wizard.apply_error = Some(error.clone());
                    self.state.status_message = format!("Migration failed: {error}");
                }
                DbResponse::CsvImported {
                    conn_id,
                    schema,
                    table,
                    rows,
                } => {
                    let msg = format!("Imported {rows} rows into {schema}.{table}");
                    self.state.status_message = msg.clone();
                    self.toasts
                        .info(msg)
                        .duration(Some(std::time::Duration::from_secs(5)));
                    // 가져온 테이블을 보고 있으면 그리드 새로고침.
                    let viewing = self
                        .state
                        .data_edit
                        .source
                        .as_ref()
                        .map(|s| s.conn_id == conn_id && s.schema == schema && s.table == table)
                        .unwrap_or(false);
                    if viewing {
                        let source = crate::state::DataSource {
                            conn_id,
                            schema: schema.clone(),
                            table: table.clone(),
                            filter: None,
                        };
                        let limit = self.state.data_edit.page_limit;
                        let columns = self.state.data_columns_for_source(&source);
                        let sql = build_data_select_sql_with_columns(
                            &source,
                            &self.state.data_edit.sort,
                            limit,
                            0,
                            &columns,
                        );
                        bridge.send(crate::db::bridge::DbCommand::ExecuteQuery {
                            conn_id,
                            sql,
                            row_limit: Some(limit),
                        });
                        self.state.query_running = true;
                    }
                }
                DbResponse::TransferProgress { progress } => {
                    self.state.transfer.progress = Some(progress);
                }
                DbResponse::TransferComplete { result } => {
                    self.state.transfer.result = Some(result.clone());
                    self.state.transfer.progress = None;
                    let msg = format!(
                        "Transfer: {} ok, {} failed, {} skipped",
                        result.tables_success, result.tables_failed, result.tables_skipped
                    );
                    self.state.status_message = msg.clone();
                    self.toasts
                        .info(msg)
                        .duration(Some(std::time::Duration::from_secs(5)));
                }
                DbResponse::Error { conn_id, error } => {
                    let was_applying_edits = self.state.data_edit.applying;
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
                                conn.connection_error = Some(error.message.clone());
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
                    if was_applying_edits {
                        self.state.diagnostics_panel.push_mutation_diagnostic(
                            crate::ui::diagnostics_panel::DiagSeverity::Error,
                            format!("Data edit failed: {}", error.message),
                        );
                    }
                }
            }
        }
    }
}

impl eframe::App for FerrumGridApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update_check.poll();
        let previous_dark_mode = self.settings.dark_mode;
        self.process_responses();
        self.state.diagnostics_panel.unsafe_ctid_active = self.settings.unsafe_ctid;

        // Plan v7 Phase 3b — dangling tx monitoring (30s warn, 60s auto-ROLLBACK).
        if self.state.explicit_tx_active {
            if let Some(started) = self.state.explicit_tx_started {
                let elapsed = started.elapsed();
                let status = crate::db::dangling_tx::evaluate_status(elapsed);
                match status {
                    crate::db::dangling_tx::DanglingTxStatus::ShouldRollback => {
                        if let Some(conn_id) = self.state.active_connection {
                            if let Some(bridge) = &self.bridge {
                                bridge.send(crate::db::bridge::DbCommand::ExecuteQuery {
                                    conn_id,
                                    sql: "ROLLBACK".to_string(),
                                    row_limit: None,
                                });
                            }
                        }
                        self.state.diagnostics_panel.push_dangling_tx(
                            crate::ui::diagnostics_panel::DiagSeverity::Error,
                            format!("Forced ROLLBACK after {}s idle in transaction", elapsed.as_secs()),
                        );
                        self.toasts.error("Transaction auto-rolled back after 60s");
                        self.state.explicit_tx_active = false;
                        self.state.explicit_tx_started = None;
                        self.state.explicit_tx_warned = false;
                    }
                    crate::db::dangling_tx::DanglingTxStatus::ShouldWarn => {
                        if !self.state.explicit_tx_warned {
                            self.state.explicit_tx_warned = true;
                            self.state.diagnostics_panel.push_dangling_tx(
                                crate::ui::diagnostics_panel::DiagSeverity::Warn,
                                format!("Transaction idle for {}s — auto-ROLLBACK at 60s", elapsed.as_secs()),
                            );
                            self.toasts.warning(format!("Transaction idle for {}s", elapsed.as_secs()));
                        }
                    }
                    crate::db::dangling_tx::DanglingTxStatus::Ok => {}
                }
            }
        }

        // US-M2 — pending invalidation tick: 5s 초과 → EchoTimeout, 30s 초과 → CacheStale.
        let actions = crate::db::invalidate::compute_diag_actions(
            &self.state.pending_invalidations,
            &self.state.echo_warned,
            std::time::Instant::now(),
        );
        for action in actions {
            match action {
                crate::db::invalidate::DiagAction::EchoTimeout(oid) => {
                    self.state.diagnostics_panel.push_echo_timeout(format!(
                        "Invalidate echo timeout (>5s) for table_oid {oid} — cache may be stale"
                    ));
                    self.state.echo_warned.insert(oid);
                }
                crate::db::invalidate::DiagAction::CacheStale(oid) => {
                    self.state.diagnostics_panel.push_cache_stale(format!(
                        "Cache stale (>30s without echo) for table_oid {oid} — manual refresh recommended"
                    ));
                    self.state.pending_invalidations.remove(&oid);
                    self.state.echo_warned.remove(&oid);
                }
            }
        }

        match crate::dock_menu::poll_action() {
            1 => show_main_window(ctx),
            2 => {
                show_main_window(ctx);
                self.state.show_connection_dialog = true;
                self.state.connection_dialog = Default::default();
            }
            _ => {}
        }

        let menu_actions = self
            .native_menu
            .handle_events(ctx, &mut self.state, &mut self.settings);
        if menu_actions.show_main_window {
            show_main_window(ctx);
        }
        if menu_actions.hide_main_window {
            hide_main_window(ctx, false);
        }
        if menu_actions.quit_requested {
            self.quit_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.handle_close_request(ctx);

        if !self.state.vault.is_unlocked() {
            ui::vault::render_vault_window(ctx, &mut self.state);
            self.render_update_bubble(ctx);
            self.toasts.show(ctx);
            return;
        }

        let bridge = self.bridge.as_ref().unwrap();
        ui::panels::render_panels(ctx, &mut self.state, bridge, &mut self.settings);
        ui::dialogs::render_connection_dialog(ctx, &mut self.state, bridge);
        ui::drop_dialog::render_drop_dialog(ctx, &mut self.state, bridge);
        ui::transfer_dialog::render_transfer_dialog(ctx, &mut self.state, bridge);
        ui::migration_wizard::render_migration_wizard(ctx, &mut self.state, bridge);
        ui::backup_dialogs::render_backup_wizard(ctx, &mut self.state, bridge, &self.settings);
        ui::backup_dialogs::render_restore_confirm_dialog(ctx, &mut self.state, bridge);
        ui::about::render_about_window(ctx, &mut self.state);
        if ui::settings::render_settings_window(ctx, &mut self.state, &mut self.settings) {
            self.native_menu.refresh_locale();
            ui::theme::configure_fonts(ctx, &self.settings.language);
        }
        if self.settings.dark_mode != previous_dark_mode {
            ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(
                crate::app_icon::icon_for_dark_mode(self.settings.dark_mode),
            ))));
        }
        ui::table_designer::render_table_designer(ctx, &mut self.state, bridge);
        crate::prisma::ui::render_prisma_window(ctx, &mut self.state, bridge);

        self.render_update_bubble(ctx);
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

        // Plan v7 Phase 4b3 — automation runner graceful shutdown.
        // shutdown_tx 발사 → done_rx.recv_timeout(1s) 으로 thread 종료 신호 대기.
        if let Some((handle, shutdown_tx, done_rx)) = self.automation_runner.take() {
            let _ = shutdown_tx.send(());
            match done_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(()) => {
                    let _ = handle.join();
                }
                Err(_) => {
                    tracing::warn!("automation runner did not exit within 1s — leaving as daemon");
                }
            }
        }

        // Disconnect all and drop bridge
        if let Some(bridge) = &self.bridge {
            for conn_id in self.state.connections.keys() {
                bridge.send(crate::db::bridge::DbCommand::Disconnect { conn_id: *conn_id });
            }
        }
        self.bridge = None;
    }
}

impl FerrumGridApp {
    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return;
        }

        if self.quit_requested {
            return;
        }

        hide_main_window(ctx, true);
    }

    fn render_update_bubble(&mut self, ctx: &egui::Context) {
        let status = &self.update_check.status;
        
        let show = matches!(
            status,
            crate::updater::UpdateStatus::UpdateAvailable { .. }
                | crate::updater::UpdateStatus::Downloading { .. }
                | crate::updater::UpdateStatus::Installing { .. }
                | crate::updater::UpdateStatus::Error(_)
        );
        
        if !show {
            return;
        }

        let offset = if self.state.vault.is_unlocked() {
            egui::vec2(16.0, -38.0)
        } else {
            egui::vec2(16.0, -16.0)
        };

        egui::Area::new(egui::Id::new("auto_updater_bubble"))
            .anchor(egui::Align2::LEFT_BOTTOM, offset)
            .show(ctx, |ui| {
                let frame = egui::Frame::new()
                    .fill(crate::ui::theme::with_alpha(crate::ui::theme::BG_SHELL, 245))
                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER_STRONG))
                    .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_LG))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .shadow(egui::Shadow {
                        offset: [0, 6],
                        blur: 16,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(100),
                    });

                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;

                        // 1. Icon (emerald circle, yellow error, or spinner)
                        match &self.update_check.status {
                            crate::updater::UpdateStatus::UpdateAvailable { .. } => {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                                ui.painter().circle_filled(rect.center(), 9.0, crate::ui::theme::ACCENT_EMERALD_DIM);
                                ui.painter().circle_filled(rect.center(), 4.0, crate::ui::theme::ACCENT_EMERALD);
                            }
                            crate::updater::UpdateStatus::Downloading { .. }
                            | crate::updater::UpdateStatus::Installing { .. } => {
                                ui.add(egui::Spinner::new().size(16.0));
                            }
                            crate::updater::UpdateStatus::Error(_) => {
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                                ui.painter().circle_filled(rect.center(), 9.0, crate::ui::theme::ACCENT_RED_DIM);
                                ui.painter().circle_filled(rect.center(), 4.0, crate::ui::theme::ACCENT_RED);
                            }
                            _ => {}
                        }

                        // 2. Info text
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            match &self.update_check.status {
                                crate::updater::UpdateStatus::UpdateAvailable { latest, dmg_url, .. } => {
                                    ui.label(
                                        egui::RichText::new("Update Available")
                                            .color(crate::ui::theme::TEXT_PRIMARY)
                                            .strong()
                                            .size(13.0),
                                    );
                                    let is_dmg = dmg_url.is_some();
                                    let subtext = if is_dmg {
                                        format!("v{} is ready to install", latest)
                                    } else {
                                        format!("v{} is available (manual download)", latest)
                                    };
                                    ui.label(
                                        egui::RichText::new(subtext)
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0),
                                    );
                                }
                                crate::updater::UpdateStatus::Downloading { latest } => {
                                    ui.label(
                                        egui::RichText::new("Downloading Update")
                                            .color(crate::ui::theme::TEXT_PRIMARY)
                                            .strong()
                                            .size(13.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("Downloading v{}...", latest))
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0),
                                    );
                                }
                                crate::updater::UpdateStatus::Installing { latest } => {
                                    ui.label(
                                        egui::RichText::new("Installing Update")
                                            .color(crate::ui::theme::TEXT_PRIMARY)
                                            .strong()
                                            .size(13.0),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("Extracting & preparing v{}...", latest))
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0),
                                    );
                                }
                                crate::updater::UpdateStatus::Error(err) => {
                                    ui.label(
                                        egui::RichText::new("Update Failed")
                                            .color(crate::ui::theme::ACCENT_RED)
                                            .strong()
                                            .size(13.0),
                                    );
                                    let short_err = if err.len() > 32 {
                                        format!("{}...", &err[..30])
                                    } else {
                                        err.clone()
                                    };
                                    ui.label(
                                        egui::RichText::new(short_err)
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0),
                                    );
                                }
                                _ => {}
                            }
                        });

                        ui.add_space(8.0);

                        // 3. Actions
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            match &self.update_check.status {
                                crate::updater::UpdateStatus::UpdateAvailable { latest, dmg_url, url, .. } => {
                                    let latest = latest.clone();
                                    let dmg_url = dmg_url.clone();
                                    let url = url.clone();

                                    if let Some(dmg_url) = dmg_url {
                                        let btn = egui::Button::new(
                                            egui::RichText::new("Update Now")
                                                .color(egui::Color32::from_rgb(15, 15, 15))
                                                .strong()
                                                .size(11.0)
                                        )
                                        .fill(crate::ui::theme::ACCENT_EMERALD)
                                        .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_MD));
                                        
                                        if ui.add(btn).clicked() {
                                            self.update_check.start_update(dmg_url, latest, ctx.clone());
                                        }
                                    } else {
                                        let btn = egui::Button::new(
                                            egui::RichText::new("Download")
                                                .color(egui::Color32::from_rgb(15, 15, 15))
                                                .strong()
                                                .size(11.0)
                                        )
                                        .fill(crate::ui::theme::ACCENT_EMERALD)
                                        .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_MD));
                                        
                                        if ui.add(btn).clicked() {
                                            let _ = std::process::Command::new("open")
                                                .arg(&url)
                                                .spawn();
                                            self.update_check.status = crate::updater::UpdateStatus::Idle;
                                        }
                                    }

                                    let dismiss_btn = egui::Button::new(
                                        egui::RichText::new("Dismiss")
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0)
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER_STRONG))
                                    .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_MD));

                                    if ui.add(dismiss_btn).clicked() {
                                        self.update_check.status = crate::updater::UpdateStatus::Idle;
                                    }
                                }
                                crate::updater::UpdateStatus::Downloading { .. }
                                | crate::updater::UpdateStatus::Installing { .. } => {
                                    let btn = egui::Button::new(
                                        egui::RichText::new("Processing...")
                                            .color(crate::ui::theme::TEXT_DISABLED)
                                            .size(11.0)
                                    )
                                    .fill(crate::ui::theme::BG_LIGHT)
                                    .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_MD));
                                    
                                    ui.add_enabled(false, btn);
                                }
                                crate::updater::UpdateStatus::Error(_) => {
                                    let dismiss_btn = egui::Button::new(
                                        egui::RichText::new("Close")
                                            .color(crate::ui::theme::TEXT_MUTED)
                                            .size(11.0)
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER_STRONG))
                                    .corner_radius(egui::CornerRadius::same(crate::ui::theme::RADIUS_MD));

                                    if ui.add(dismiss_btn).clicked() {
                                        self.update_check.status = crate::updater::UpdateStatus::Idle;
                                    }
                                }
                                _ => {}
                            }
                        });
                    });
                });
            });
    }
}

fn hide_main_window(ctx: &egui::Context, cancel_close: bool) {
    if cancel_close {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
    }
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
}

fn show_main_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}


