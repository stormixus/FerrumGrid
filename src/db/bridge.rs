use std::collections::HashMap;

use crate::db::edits::{MutationOutcome, RowEditOp};
use crate::db::error::DbError;
use crate::db::schema_diff::SchemaDiff;
use crate::state::transfer::{TransferProgress, TransferRequest, TransferResult};
use crate::types::{
    BackupRecord, BackupRequest, ColumnInfo, ConnectionConfig, ConnectionId, FunctionInfo,
    IndexInfo, QueryResult, RoleInfo, RuleInfo, TableInfo, TriggerInfo,
};

#[derive(Debug)]
pub enum DbCommand {
    Connect {
        conn_id: ConnectionId,
        config: ConnectionConfig,
    },
    Disconnect {
        conn_id: ConnectionId,
    },
    ExecuteQuery {
        conn_id: ConnectionId,
        sql: String,
        row_limit: Option<usize>,
    },
    /// Plan v7 Phase 2 — DDL 을 2-step NOTIFY (pre_drop / DDL / post_drop)
    /// sequence 안에서 실행. 성공 시 `schema_to_refresh` 가 있으면 자동으로
    /// `ListTables` 도 발사 (UI auto-refresh).
    ApplyDdlWithInvalidation {
        conn_id: ConnectionId,
        sql: String,
        table_oid: Option<u32>,
        schema_to_refresh: Option<String>,
    },
    ApplyDataEdits {
        conn_id: ConnectionId,
        edits: Vec<RowEditOp>,
    },
    ListSchemas {
        conn_id: ConnectionId,
    },
    ListDatabases {
        conn_id: ConnectionId,
    },
    ListTables {
        conn_id: ConnectionId,
        schema: String,
    },
    ListColumns {
        conn_id: ConnectionId,
        schema: String,
        table: String,
    },
    ListIndexes {
        conn_id: ConnectionId,
        schema: String,
        table: String,
    },
    ListForeignKeys {
        conn_id: ConnectionId,
        schema: String,
    },
    ListRules {
        conn_id: ConnectionId,
        schema: String,
        table: String,
    },
    ListTriggers {
        conn_id: ConnectionId,
        schema: String,
        table: String,
    },
    ListFunctions {
        conn_id: ConnectionId,
        schema: String,
    },
    ListRoles {
        conn_id: ConnectionId,
    },
    CancelQuery {
        conn_id: ConnectionId,
    },
    RunBackup {
        request: BackupRequest,
    },
    RunRestore {
        #[allow(dead_code)]
        conn_id: ConnectionId,
        config: ConnectionConfig,
        file_path: std::path::PathBuf,
    },
    /// Plan v7 Phase 4b — Automation 의 즉시 실행 (Run Now 버튼) 또는 background
    /// scheduler (runner.rs) 가 due task 발견 시 발사.
    ExecuteAutomation {
        conn_id: ConnectionId,
        task_id: uuid::Uuid,
        sql: String,
    },
    /// US-K1 — Drop 다이얼로그가 dependents 미리보기 위해 발사.
    /// `refobjid` 는 pg_class.oid (drop 대상 table). 결과는 `DependentsList` 응답.
    FetchDependents {
        conn_id: ConnectionId,
        refobjid: u32,
    },
    TransferTables {
        request: TransferRequest,
    },
    CompareSchemas {
        source_config: ConnectionConfig,
        target_config: ConnectionConfig,
        source_schema: String,
        target_schema: String,
    },
    ApplyMigration {
        target_config: ConnectionConfig,
        sql: String,
    },
    /// CSV 파일을 대상 테이블로 import (COPY FROM STDIN).
    ImportCsv {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        path: std::path::PathBuf,
    },
    /// `EXPLAIN (FORMAT JSON) <sql>` 실행 (plan-only, 쿼리 미실행).
    RunExplain {
        conn_id: ConnectionId,
        sql: String,
    },
    /// pg_stat_activity 세션 목록 조회.
    ListSessions {
        conn_id: ConnectionId,
    },
    /// backend cancel(terminate=false) 또는 terminate(true).
    KillBackend {
        conn_id: ConnectionId,
        pid: i32,
        terminate: bool,
    },
}

#[derive(Debug)]
pub enum DbResponse {
    Connected {
        conn_id: ConnectionId,
        server_version: String,
    },
    Disconnected {
        conn_id: ConnectionId,
    },
    QueryResult {
        conn_id: ConnectionId,
        result: QueryResult,
        truncated: bool,
    },
    DataEditsApplied {
        conn_id: ConnectionId,
        outcome: MutationOutcome,
    },
    SchemaList {
        conn_id: ConnectionId,
        schemas: Vec<String>,
    },
    DatabaseList {
        conn_id: ConnectionId,
        databases: Vec<String>,
    },
    TableList {
        conn_id: ConnectionId,
        schema: String,
        tables: Vec<TableInfo>,
    },
    ColumnList {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        columns: Vec<ColumnInfo>,
    },
    IndexList {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        indexes: Vec<IndexInfo>,
    },
    ForeignKeyList {
        conn_id: ConnectionId,
        schema: String,
        foreign_keys: Vec<crate::ui::er_diagram::ForeignKey>,
    },
    RuleList {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        rules: Vec<RuleInfo>,
    },
    TriggerList {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        triggers: Vec<TriggerInfo>,
    },
    FunctionList {
        conn_id: ConnectionId,
        schema: String,
        functions: Vec<FunctionInfo>,
    },
    RoleList {
        conn_id: ConnectionId,
        roles: Vec<RoleInfo>,
    },
    BackupCompleted {
        record: BackupRecord,
    },
    BackupFailed {
        conn_id: ConnectionId,
        error: String,
    },
    BackupProgress {
        #[allow(dead_code)]
        conn_id: ConnectionId,
        progress: f32,
        current_table: String,
    },
    RestoreCompleted {
        file_path: std::path::PathBuf,
    },
    RestoreFailed {
        #[allow(dead_code)]
        file_path: std::path::PathBuf,
        error: String,
    },
    /// Plan v7 Phase 4b — `ExecuteAutomation` 의 응답. `conn_id` 는 future
    /// per-connection routing 용 (현재 app.rs handler 가 사용 안 함 — `_conn_id`).
    #[allow(dead_code)]
    AutomationResult {
        conn_id: ConnectionId,
        task_id: uuid::Uuid,
        result: crate::automation::scheduler::ApplyResult,
    },
    /// Plan v7 Phase 3b — Query 탭 명시 BEGIN/COMMIT/ROLLBACK 감지 후 tx 상태 변경.
    /// `conn_id` 는 future per-connection routing 용 (현재 app.rs handler 가 `_` 로 무시).
    #[allow(dead_code)]
    ExplicitTxChanged {
        conn_id: ConnectionId,
        active: bool,
    },
    Error {
        conn_id: ConnectionId,
        error: DbError,
    },
    TransferProgress {
        progress: TransferProgress,
    },
    TransferComplete {
        result: TransferResult,
    },
    SchemaDiffResult {
        diff: SchemaDiff,
    },
    SchemaDiffError {
        error: String,
    },
    MigrationApplied,
    MigrationFailed {
        error: String,
    },
    /// CSV import 완료 — 적재된 행 수.
    CsvImported {
        conn_id: ConnectionId,
        schema: String,
        table: String,
        rows: u64,
    },
    /// EXPLAIN 플랜 JSON 문자열. `conn_id` 는 future per-connection routing 용.
    ExplainPlan {
        #[allow(dead_code)]
        conn_id: ConnectionId,
        json: String,
    },
    /// pg_stat_activity 세션 목록.
    SessionList {
        #[allow(dead_code)]
        conn_id: ConnectionId,
        sessions: Vec<crate::db::sessions::SessionRow>,
    },
    /// backend cancel/terminate 결과.
    BackendKilled {
        #[allow(dead_code)]
        conn_id: ConnectionId,
        pid: i32,
        terminated: bool,
        ok: bool,
    },
    /// US-K1 — `FetchDependents` 응답. `deps` 는 표시용 string list, `truncated`
    /// 는 51 개 이상이었음을 의미. `conn_id` / `refobjid` 는 future per-dialog
    /// routing 용 (현재 app.rs handler 가 단일 drop_dialog 만 추적하므로 무시).
    #[allow(dead_code)]
    DependentsList {
        conn_id: ConnectionId,
        refobjid: u32,
        deps: Vec<String>,
        truncated: bool,
    },
}

pub struct DbBridge {
    cmd_tx: tokio::sync::mpsc::Sender<DbCommand>,
    resp_rx: std::sync::mpsc::Receiver<DbResponse>,
    _thread: Option<std::thread::JoinHandle<()>>,
}

impl DbBridge {
    pub fn new(ctx: eframe::egui::Context) -> Self {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<DbCommand>(256);
        let (resp_tx, resp_rx) = std::sync::mpsc::channel::<DbResponse>();

        let thread = std::thread::Builder::new()
            .name("ferrumgrid-db".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                    .expect("failed to create tokio runtime");

                rt.block_on(dispatch_loop(cmd_rx, resp_tx, ctx));

                rt.shutdown_timeout(std::time::Duration::from_secs(5));
            })
            .expect("failed to spawn db thread");

        Self {
            cmd_tx,
            resp_rx,
            _thread: Some(thread),
        }
    }

    /// Plan v7 Phase 4b3 runner — 외부 (e.g., automation scheduler) 가 cmd
    /// channel sender 를 clone 해서 spawn 한 task 에서 직접 발사할 수 있도록
    /// 노출. Sender 는 mpsc 의 Clone trait 으로 self-multiplex.
    pub fn cmd_sender(&self) -> tokio::sync::mpsc::Sender<DbCommand> {
        self.cmd_tx.clone()
    }

    pub fn send(&self, cmd: DbCommand) {
        if let Err(e) = self.cmd_tx.try_send(cmd) {
            tracing::error!("failed to send command to db bridge: {e}");
        }
    }

    pub fn try_recv(&self) -> Option<DbResponse> {
        self.resp_rx.try_recv().ok()
    }
}

struct ConnectionHandle {
    task_tx: tokio::sync::mpsc::Sender<ConnCommand>,
    cancel_token_rx: tokio::sync::watch::Receiver<Option<tokio_postgres::CancelToken>>,
    use_tls: bool,
}

enum ConnCommand {
    ExecuteQuery {
        sql: String,
        row_limit: Option<usize>,
    },
    ApplyDdlWithInvalidation {
        sql: String,
        table_oid: Option<u32>,
        schema_to_refresh: Option<String>,
    },
    ExecuteAutomation {
        task_id: uuid::Uuid,
        sql: String,
    },
    ApplyDataEdits {
        edits: Vec<RowEditOp>,
    },
    ListSchemas,
    ListDatabases,
    ListTables {
        schema: String,
    },
    ListColumns {
        schema: String,
        table: String,
    },
    ListIndexes {
        schema: String,
        table: String,
    },
    ListForeignKeys {
        schema: String,
    },
    ListRules {
        schema: String,
        table: String,
    },
    ListTriggers {
        schema: String,
        table: String,
    },
    ListFunctions {
        schema: String,
    },
    ListRoles,
    /// US-K1 — Drop dialog dependents 미리보기.
    FetchDependents {
        refobjid: u32,
    },
    ImportCsv {
        schema: String,
        table: String,
        path: std::path::PathBuf,
    },
    RunExplain {
        sql: String,
    },
    ListSessions,
    KillBackend {
        pid: i32,
        terminate: bool,
    },
    Shutdown,
}

async fn dispatch_loop(
    mut cmd_rx: tokio::sync::mpsc::Receiver<DbCommand>,
    resp_tx: std::sync::mpsc::Sender<DbResponse>,
    ctx: eframe::egui::Context,
) {
    let mut connections: HashMap<ConnectionId, ConnectionHandle> = HashMap::new();

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            DbCommand::Connect { conn_id, config } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                let (task_tx, task_rx) = tokio::sync::mpsc::channel::<ConnCommand>(64);
                let (cancel_token_tx, cancel_token_rx) = tokio::sync::watch::channel(None);
                let use_tls = config.use_tls;

                connections.insert(
                    conn_id,
                    ConnectionHandle {
                        task_tx,
                        cancel_token_rx,
                        use_tls,
                    },
                );

                tokio::spawn(async move {
                    connection_task(conn_id, config, task_rx, resp_tx, ctx, cancel_token_tx).await;
                });
            }
            DbCommand::Disconnect { conn_id } => {
                if let Some(handle) = connections.remove(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::Shutdown).await;
                }
                let _ = resp_tx.send(DbResponse::Disconnected { conn_id });
                ctx.request_repaint();
            }
            DbCommand::ExecuteQuery {
                conn_id,
                sql,
                row_limit,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ExecuteQuery { sql, row_limit })
                        .await;
                }
            }
            DbCommand::ApplyDdlWithInvalidation {
                conn_id,
                sql,
                table_oid,
                schema_to_refresh,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ApplyDdlWithInvalidation {
                            sql,
                            table_oid,
                            schema_to_refresh,
                        })
                        .await;
                }
            }
            DbCommand::ApplyDataEdits { conn_id, edits } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ApplyDataEdits { edits })
                        .await;
                }
            }
            DbCommand::ExecuteAutomation {
                conn_id,
                task_id,
                sql,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ExecuteAutomation { task_id, sql })
                        .await;
                }
            }
            DbCommand::ImportCsv {
                conn_id,
                schema,
                table,
                path,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ImportCsv {
                            schema,
                            table,
                            path,
                        })
                        .await;
                }
            }
            DbCommand::RunExplain { conn_id, sql } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::RunExplain { sql }).await;
                }
            }
            DbCommand::ListSessions { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::ListSessions).await;
                }
            }
            DbCommand::KillBackend {
                conn_id,
                pid,
                terminate,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::KillBackend { pid, terminate })
                        .await;
                }
            }
            DbCommand::ListSchemas { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::ListSchemas).await;
                }
            }
            DbCommand::ListDatabases { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::ListDatabases).await;
                }
            }
            DbCommand::ListTables { conn_id, schema } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListTables { schema })
                        .await;
                }
            }
            DbCommand::ListColumns {
                conn_id,
                schema,
                table,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListColumns { schema, table })
                        .await;
                }
            }
            DbCommand::ListIndexes {
                conn_id,
                schema,
                table,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListIndexes { schema, table })
                        .await;
                }
            }
            DbCommand::ListForeignKeys { conn_id, schema } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListForeignKeys { schema })
                        .await;
                }
            }
            DbCommand::ListRules {
                conn_id,
                schema,
                table,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListRules { schema, table })
                        .await;
                }
            }
            DbCommand::ListTriggers {
                conn_id,
                schema,
                table,
            } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListTriggers { schema, table })
                        .await;
                }
            }
            DbCommand::ListFunctions { conn_id, schema } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ListFunctions { schema })
                        .await;
                }
            }
            DbCommand::ListRoles { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::ListRoles).await;
                }
            }
            DbCommand::FetchDependents { conn_id, refobjid } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::FetchDependents { refobjid })
                        .await;
                }
            }
            DbCommand::CancelQuery { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    if let Some(token) = handle.cancel_token_rx.borrow().clone() {
                        let resp_tx = resp_tx.clone();
                        let ctx = ctx.clone();
                        let use_tls = handle.use_tls;
                        tokio::spawn(async move {
                            if let Err(error) =
                                crate::db::connection::cancel_query(token, use_tls, conn_id).await
                            {
                                let _ = resp_tx.send(DbResponse::Error { conn_id, error });
                                ctx.request_repaint();
                            }
                        });
                    } else {
                        let error = DbError::internal(
                            conn_id,
                            "Query cannot be cancelled before the connection is ready.",
                        );
                        let _ = resp_tx.send(DbResponse::Error { conn_id, error });
                        ctx.request_repaint();
                    }
                }
            }
            DbCommand::CompareSchemas {
                source_config,
                target_config,
                source_schema,
                target_schema,
            } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let response =
                        match crate::db::schema_diff_exec::compare_schemas(
                            &source_config,
                            &target_config,
                            &source_schema,
                            &target_schema,
                        )
                        .await
                    {
                        Ok(diff) => DbResponse::SchemaDiffResult { diff },
                        Err(e) => DbResponse::SchemaDiffError {
                            error: e.to_string(),
                        },
                    };
                    let _ = resp_tx.send(response);
                    ctx.request_repaint();
                });
            }
            DbCommand::ApplyMigration {
                target_config,
                sql,
            } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let response = match crate::db::connection::connect_any(&target_config).await {
                        Ok(client) => match client.batch_execute(&sql).await {
                            Ok(()) => DbResponse::MigrationApplied,
                            Err(e) => DbResponse::MigrationFailed {
                                error: e.to_string(),
                            },
                        },
                        Err(e) => DbResponse::MigrationFailed {
                            error: e.to_string(),
                        },
                    };
                    let _ = resp_tx.send(response);
                    ctx.request_repaint();
                });
            }
            DbCommand::TransferTables { request } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result =
                        crate::db::transfer_exec::execute_transfer(request, resp_tx.clone(), ctx.clone())
                            .await;
                    let _ = resp_tx.send(DbResponse::TransferComplete { result });
                    ctx.request_repaint();
                });
            }
            DbCommand::RunBackup { request } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let conn_id = request.conn_id;
                    let response = match crate::db::backup::run_backup(request, resp_tx.clone(), ctx.clone()).await {
                        Ok(record) => DbResponse::BackupCompleted { record },
                        Err(error) => DbResponse::BackupFailed { conn_id, error },
                    };
                    let _ = resp_tx.send(response);
                    ctx.request_repaint();
                });
            }
            DbCommand::RunRestore { conn_id: _, config, file_path } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let response = match crate::db::backup_fgb::run_fgb_restore(&config, &file_path).await {
                        Ok(()) => DbResponse::RestoreCompleted { file_path },
                        Err(error) => DbResponse::RestoreFailed { file_path, error },
                    };
                    let _ = resp_tx.send(response);
                    ctx.request_repaint();
                });
            }
        }
    }

    // Shutdown all connections
    for (_, handle) in connections.drain() {
        let _ = handle.task_tx.send(ConnCommand::Shutdown).await;
    }
}

async fn connection_task(
    conn_id: ConnectionId,
    config: ConnectionConfig,
    mut task_rx: tokio::sync::mpsc::Receiver<ConnCommand>,
    resp_tx: std::sync::mpsc::Sender<DbResponse>,
    ctx: eframe::egui::Context,
    cancel_token_tx: tokio::sync::watch::Sender<Option<tokio_postgres::CancelToken>>,
) {
    // Connect
    let client = if config.use_tls {
        match crate::db::connection::connect(&config).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        tracing::error!("connection error: {e}");
                    }
                });
                client
            }
            Err(e) => {
                let _ = resp_tx.send(DbResponse::Error { conn_id, error: e });
                ctx.request_repaint();
                return;
            }
        }
    } else {
        match crate::db::connection::connect_no_tls(&config).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        tracing::error!("connection error: {e}");
                    }
                });
                client
            }
            Err(e) => {
                let _ = resp_tx.send(DbResponse::Error { conn_id, error: e });
                ctx.request_repaint();
                return;
            }
        }
    };

    // Get server version
    let server_version = match client.query_one("SHOW server_version", &[]).await {
        Ok(row) => row.get::<_, String>(0),
        Err(_) => "unknown".to_string(),
    };

    let _ = resp_tx.send(DbResponse::Connected {
        conn_id,
        server_version,
    });
    let _ = cancel_token_tx.send(Some(client.cancel_token()));
    ctx.request_repaint();

    let mut client = client;

    // Process commands
    while let Some(cmd) = task_rx.recv().await {
        match cmd {
            ConnCommand::ExecuteQuery { sql, row_limit } => {
                let trimmed = sql.trim().to_uppercase();
                let is_select = trimmed.starts_with("SELECT")
                    || trimmed.starts_with("WITH")
                    || trimmed.starts_with("SHOW")
                    || trimmed.starts_with("EXPLAIN");

                let response = if is_select {
                    match crate::db::queries::execute_query(&mut client, &sql, row_limit, conn_id)
                        .await
                    {
                        Ok((result, truncated)) => DbResponse::QueryResult {
                            conn_id,
                            result,
                            truncated,
                        },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    }
                } else {
                    match crate::db::queries::execute_statement(&client, &sql, conn_id).await {
                        Ok((result, truncated)) => DbResponse::QueryResult {
                            conn_id,
                            result,
                            truncated,
                        },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    }
                };
                let _ = resp_tx.send(response);

                // Plan v7 Phase 3b — classify explicit tx boundary and notify UI.
                use crate::db::begin_detect::{classify_explicit_tx, ExplicitTxClass};
                let tx_class = classify_explicit_tx(&sql);
                match tx_class {
                    ExplicitTxClass::Begin => {
                        let _ = resp_tx.send(DbResponse::ExplicitTxChanged { conn_id, active: true });
                    }
                    ExplicitTxClass::Commit | ExplicitTxClass::Rollback => {
                        let _ = resp_tx.send(DbResponse::ExplicitTxChanged { conn_id, active: false });
                    }
                    _ => {}
                }

                ctx.request_repaint();
            }
            ConnCommand::ApplyDdlWithInvalidation {
                sql,
                table_oid,
                schema_to_refresh,
            } => {
                // Plan v7 Phase 2 — 2-step NOTIFY DDL.
                let ddl_result =
                    crate::db::ddl::execute_ddl_with_invalidation(&client, &sql, table_oid, conn_id)
                        .await;
                match ddl_result {
                    Ok(()) => {
                        // 성공 시 한 줄 status row + 자동 schema/tables refresh.
                        let response = DbResponse::QueryResult {
                            conn_id,
                            result: crate::types::QueryResult {
                                columns: vec![crate::types::ColumnMeta {
                                    name: "status".to_string(),
                                    type_name: "text".to_string(),
                                }],
                                rows: vec![vec![crate::types::CellValue::Text(
                                    "DDL applied with invalidation".to_string(),
                                )]],
                                execution_time_ms: 0,
                            },
                            truncated: false,
                        };
                        let _ = resp_tx.send(response);
                        if let Some(schema) = schema_to_refresh {
                            let refresh = match crate::db::metadata::list_tables(
                                &client, &schema, conn_id,
                            )
                            .await
                            {
                                Ok(tables) => DbResponse::TableList {
                                    conn_id,
                                    schema: schema.clone(),
                                    tables,
                                },
                                Err(e) => DbResponse::Error { conn_id, error: e },
                            };
                            let _ = resp_tx.send(refresh);
                        }
                    }
                    Err(e) => {
                        let _ = resp_tx.send(DbResponse::Error { conn_id, error: e });
                    }
                }
                ctx.request_repaint();
            }
            ConnCommand::ExecuteAutomation { task_id, sql } => {
                // Plan v7 Phase 4b — Automation 즉시 실행 / scheduler 호출.
                // SELECT/EXPLAIN 도 단순 execute (rows_affected) 로 처리 — Automation
                // 의 결과는 progress reporting 만 필요, full result set 은 Query 탭으로.
                let result = match client.execute(sql.as_str(), &[]).await {
                    Ok(rows_affected) => {
                        crate::automation::scheduler::ApplyResult::Success { rows_affected }
                    }
                    Err(err) => crate::automation::scheduler::ApplyResult::Failed {
                        error: err.to_string(),
                    },
                };
                let _ = resp_tx.send(DbResponse::AutomationResult {
                    conn_id,
                    task_id,
                    result,
                });
                ctx.request_repaint();
            }
            ConnCommand::ApplyDataEdits { edits } => {
                let response =
                    match crate::db::queries::apply_data_edits(&mut client, &edits, conn_id).await
                    {
                        Ok(outcome) => DbResponse::DataEditsApplied { conn_id, outcome },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListSchemas => {
                let response = match crate::db::metadata::list_schemas(&client, conn_id).await {
                    Ok(schemas) => DbResponse::SchemaList { conn_id, schemas },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListDatabases => {
                let response = match crate::db::metadata::list_databases(&client, conn_id).await {
                    Ok(databases) => DbResponse::DatabaseList { conn_id, databases },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListTables { schema } => {
                let response =
                    match crate::db::metadata::list_tables(&client, &schema, conn_id).await {
                        Ok(tables) => DbResponse::TableList {
                            conn_id,
                            schema: schema.clone(),
                            tables,
                        },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListColumns { schema, table } => {
                let response = match crate::db::metadata::list_columns(
                    &client, &schema, &table, conn_id,
                )
                .await
                {
                    Ok(columns) => DbResponse::ColumnList {
                        conn_id,
                        schema: schema.clone(),
                        table: table.clone(),
                        columns,
                    },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListIndexes { schema, table } => {
                let response = match crate::db::metadata::list_indexes(
                    &client, &schema, &table, conn_id,
                )
                .await
                {
                    Ok(indexes) => DbResponse::IndexList {
                        conn_id,
                        schema: schema.clone(),
                        table: table.clone(),
                        indexes,
                    },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListForeignKeys { schema } => {
                let response = match crate::db::metadata_fk::list_foreign_keys(
                    &client, &schema, conn_id,
                )
                .await
                {
                    Ok(foreign_keys) => DbResponse::ForeignKeyList {
                        conn_id,
                        schema: schema.clone(),
                        foreign_keys,
                    },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListRules { schema, table } => {
                let response = match crate::db::metadata::list_rules(
                    &client, &schema, &table, conn_id,
                )
                .await
                {
                    Ok(rules) => DbResponse::RuleList {
                        conn_id,
                        schema: schema.clone(),
                        table: table.clone(),
                        rules,
                    },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListTriggers { schema, table } => {
                let response =
                    match crate::db::metadata::list_triggers(&client, &schema, &table, conn_id)
                        .await
                    {
                        Ok(triggers) => DbResponse::TriggerList {
                            conn_id,
                            schema: schema.clone(),
                            table: table.clone(),
                            triggers,
                        },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListFunctions { schema } => {
                let response =
                    match crate::db::metadata::list_functions(&client, &schema, conn_id).await {
                        Ok(functions) => DbResponse::FunctionList {
                            conn_id,
                            schema: schema.clone(),
                            functions,
                        },
                        Err(e) => DbResponse::Error { conn_id, error: e },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListRoles => {
                let response = match crate::db::metadata::list_roles(&client, conn_id).await {
                    Ok(roles) => DbResponse::RoleList { conn_id, roles },
                    Err(e) => DbResponse::Error { conn_id, error: e },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::FetchDependents { refobjid } => {
                // US-K1 — pg_depend_recursive_sql 실행 후 결과를 표시용 string list 로 변환.
                let sql = crate::db::dependencies::pg_depend_recursive_sql();
                let response = match client
                    .query(sql, &[&(refobjid as i64)])
                    .await
                {
                    Ok(rows) => {
                        let truncated =
                            rows.len() >= crate::db::dependencies::PREVIEW_FETCH_LIMIT;
                        let deps: Vec<String> = rows
                            .iter()
                            .take(crate::db::dependencies::MAX_DISPLAY)
                            .map(|r| {
                                let objid: i64 = r.get(0);
                                let classid: i64 = r.get(1);
                                let kind = crate::db::dependencies::classify_object(classid as u32);
                                format!("oid {} ({})", objid, kind)
                            })
                            .collect();
                        DbResponse::DependentsList {
                            conn_id,
                            refobjid,
                            deps,
                            truncated,
                        }
                    }
                    Err(e) => DbResponse::Error {
                        conn_id,
                        error: crate::db::error::DbError::from_pg(&e, conn_id),
                    },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ImportCsv {
                schema,
                table,
                path,
            } => {
                let response = match crate::db::import::import_csv_file(
                    &client, &schema, &table, &path, conn_id,
                )
                .await
                {
                    Ok(rows) => DbResponse::CsvImported {
                        conn_id,
                        schema,
                        table,
                        rows,
                    },
                    Err(error) => DbResponse::Error { conn_id, error },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::RunExplain { sql } => {
                let response = match crate::db::explain::run_explain(&client, &sql, conn_id).await {
                    Ok(json) => DbResponse::ExplainPlan { conn_id, json },
                    Err(error) => DbResponse::Error { conn_id, error },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListSessions => {
                let response = match crate::db::sessions::list_sessions(&client, conn_id).await {
                    Ok(sessions) => DbResponse::SessionList { conn_id, sessions },
                    Err(error) => DbResponse::Error { conn_id, error },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::KillBackend { pid, terminate } => {
                let response =
                    match crate::db::sessions::kill_backend(&client, pid, terminate, conn_id).await {
                        Ok(ok) => DbResponse::BackendKilled {
                            conn_id,
                            pid,
                            terminated: terminate,
                            ok,
                        },
                        Err(error) => DbResponse::Error { conn_id, error },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::Shutdown => {
                break;
            }
        }
    }
    let _ = cancel_token_tx.send(None);
}
