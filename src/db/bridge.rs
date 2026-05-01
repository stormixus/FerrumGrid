use std::collections::HashMap;

use crate::db::error::DbError;
use crate::types::{
    ColumnInfo, ConnectionConfig, ConnectionId, IndexInfo, QueryResult, TableInfo,
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
    ListSchemas {
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
    CancelQuery {
        conn_id: ConnectionId,
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
    SchemaList {
        conn_id: ConnectionId,
        schemas: Vec<String>,
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
    QueryCancelled {
        conn_id: ConnectionId,
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
}

enum ConnCommand {
    ExecuteQuery {
        sql: String,
        row_limit: Option<usize>,
    },
    ListSchemas,
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
    CancelQuery,
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

                connections.insert(conn_id, ConnectionHandle { task_tx });

                tokio::spawn(async move {
                    connection_task(conn_id, config, task_rx, resp_tx, ctx).await;
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
            DbCommand::ListSchemas { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::ListSchemas).await;
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
            DbCommand::CancelQuery { conn_id } => {
                if let Some(handle) = connections.get(&conn_id) {
                    let _ = handle.task_tx.send(ConnCommand::CancelQuery).await;
                }
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
                let _ = resp_tx.send(DbResponse::Error {
                    conn_id,
                    error: e,
                });
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
                let _ = resp_tx.send(DbResponse::Error {
                    conn_id,
                    error: e,
                });
                ctx.request_repaint();
                return;
            }
        }
    };

    // Get server version
    let server_version = match client
        .query_one("SHOW server_version", &[])
        .await
    {
        Ok(row) => row.get::<_, String>(0),
        Err(_) => "unknown".to_string(),
    };

    let _ = resp_tx.send(DbResponse::Connected {
        conn_id,
        server_version,
    });
    ctx.request_repaint();

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
                    match crate::db::queries::execute_query(
                        &client, &sql, row_limit, conn_id,
                    )
                    .await
                    {
                        Ok((result, truncated)) => DbResponse::QueryResult {
                            conn_id,
                            result,
                            truncated,
                        },
                        Err(e) => DbResponse::Error {
                            conn_id,
                            error: e,
                        },
                    }
                } else {
                    match crate::db::queries::execute_statement(
                        &client, &sql, conn_id,
                    )
                    .await
                    {
                        Ok((result, truncated)) => DbResponse::QueryResult {
                            conn_id,
                            result,
                            truncated,
                        },
                        Err(e) => DbResponse::Error {
                            conn_id,
                            error: e,
                        },
                    }
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListSchemas => {
                let response =
                    match crate::db::metadata::list_schemas(&client, conn_id).await {
                        Ok(schemas) => DbResponse::SchemaList { conn_id, schemas },
                        Err(e) => DbResponse::Error {
                            conn_id,
                            error: e,
                        },
                    };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::ListTables { schema } => {
                let response =
                    match crate::db::metadata::list_tables(&client, &schema, conn_id)
                        .await
                    {
                        Ok(tables) => DbResponse::TableList { conn_id, schema: schema.clone(), tables },
                        Err(e) => DbResponse::Error {
                            conn_id,
                            error: e,
                        },
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
                    Ok(columns) => DbResponse::ColumnList { conn_id, schema: schema.clone(), table: table.clone(), columns },
                    Err(e) => DbResponse::Error {
                        conn_id,
                        error: e,
                    },
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
                    Ok(indexes) => DbResponse::IndexList { conn_id, schema: schema.clone(), table: table.clone(), indexes },
                    Err(e) => DbResponse::Error {
                        conn_id,
                        error: e,
                    },
                };
                let _ = resp_tx.send(response);
                ctx.request_repaint();
            }
            ConnCommand::CancelQuery => {
                let _ = resp_tx.send(DbResponse::QueryCancelled { conn_id });
                ctx.request_repaint();
            }
            ConnCommand::Shutdown => {
                break;
            }
        }
    }
}
