//! DBA 세션 모니터 — pg_stat_activity 조회 + backend cancel/terminate.

use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub pid: i32,
    pub user: String,
    pub database: String,
    pub client_addr: String,
    pub application_name: String,
    pub state: String,
    pub wait_event: String,
    pub query_start: String,
    pub query: String,
}

/// 현재 백엔드를 제외한 모든 세션 (pg_stat_activity).
pub async fn list_sessions(
    client: &Client,
    conn_id: ConnectionId,
) -> Result<Vec<SessionRow>, DbError> {
    let sql = "SELECT pid, \
               COALESCE(usename, ''), \
               COALESCE(datname, ''), \
               COALESCE(host(client_addr), ''), \
               COALESCE(application_name, ''), \
               COALESCE(state, ''), \
               COALESCE(wait_event_type || ':' || wait_event, ''), \
               COALESCE(to_char(query_start, 'HH24:MI:SS'), ''), \
               COALESCE(query, '') \
               FROM pg_stat_activity \
               WHERE pid <> pg_backend_pid() \
               ORDER BY query_start ASC NULLS LAST";
    let rows = client
        .query(sql, &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    Ok(rows
        .iter()
        .map(|r| SessionRow {
            pid: r.get(0),
            user: r.get(1),
            database: r.get(2),
            client_addr: r.get(3),
            application_name: r.get(4),
            state: r.get(5),
            wait_event: r.get(6),
            query_start: r.get(7),
            query: r.get(8),
        })
        .collect())
}

/// `pg_cancel_backend` (terminate=false, 실행 중 문장만 취소) 또는
/// `pg_terminate_backend` (terminate=true, 연결 자체 종료). 성공 bool 반환.
pub async fn kill_backend(
    client: &Client,
    pid: i32,
    terminate: bool,
    conn_id: ConnectionId,
) -> Result<bool, DbError> {
    let sql = if terminate {
        "SELECT pg_terminate_backend($1)"
    } else {
        "SELECT pg_cancel_backend($1)"
    };
    let row = client
        .query_one(sql, &[&pid])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    Ok(row.get(0))
}
