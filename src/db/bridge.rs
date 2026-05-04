use std::collections::HashMap;

use crate::db::error::DbError;
use crate::types::{
    BackupRecord, BackupRequest, ColumnInfo, ConnectionConfig, ConnectionId, DataCellEdit,
    FunctionInfo, IndexInfo, QueryResult, RoleInfo, RuleInfo, TableInfo, TriggerInfo,
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
    ApplyDataEdits {
        conn_id: ConnectionId,
        edits: Vec<DataCellEdit>,
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
        applied: usize,
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
    Error {
        conn_id: ConnectionId,
        error: DbError,
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
    ApplyDataEdits {
        edits: Vec<DataCellEdit>,
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
            DbCommand::ApplyDataEdits { conn_id, edits } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle
                        .task_tx
                        .send(ConnCommand::ApplyDataEdits { edits })
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
            DbCommand::RunBackup { request } => {
                let resp_tx = resp_tx.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let conn_id = request.conn_id;
                    let response = match crate::db::backup::run_backup(request).await {
                        Ok(record) => DbResponse::BackupCompleted { record },
                        Err(error) => DbResponse::BackupFailed { conn_id, error },
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
                ctx.request_repaint();
            }
            ConnCommand::ApplyDataEdits { edits } => {
                let response =
                    match crate::db::queries::apply_data_edits(&client, &edits, conn_id).await {
                        Ok(applied) => DbResponse::DataEditsApplied { conn_id, applied },
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
            ConnCommand::Shutdown => {
                break;
            }
        }
    }
    let _ = cancel_token_tx.send(None);
}
