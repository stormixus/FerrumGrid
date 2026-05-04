use tokio_postgres::{Client, Row};

use crate::db::edits::{
    build_delete_sql, build_insert_sql, build_update_sql, MutationOutcome, RowEditOp,
};
use crate::db::error::DbError;
use crate::types::{CellValue, ColumnMeta, ConnectionId, QueryResult};

pub async fn execute_query(
    client: &mut Client,
    sql: &str,
    row_limit: Option<usize>,
    conn_id: ConnectionId,
) -> Result<(QueryResult, bool), DbError> {
    let start = std::time::Instant::now();

    let stmt = client
        .prepare(sql)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let columns: Vec<ColumnMeta> = stmt
        .columns()
        .iter()
        .map(|c| ColumnMeta {
            name: c.name().to_string(),
            type_name: c.type_().name().to_string(),
        })
        .collect();

    let pg_rows = fetch_query_rows(client, &stmt, row_limit, conn_id).await?;

    let truncated = row_limit.is_some_and(|limit| pg_rows.len() > limit);
    let take_count = row_limit
        .map(|limit| limit.min(pg_rows.len()))
        .unwrap_or(pg_rows.len());

    let rows: Vec<Vec<CellValue>> = pg_rows
        .iter()
        .take(take_count)
        .map(|row| {
            (0..columns.len())
                .map(|i| extract_cell_value(row, i))
                .collect()
        })
        .collect();

    let execution_time_ms = start.elapsed().as_millis();

    Ok((
        QueryResult {
            columns,
            rows,
            execution_time_ms,
        },
        truncated,
    ))
}

pub async fn execute_statement(
    client: &Client,
    sql: &str,
    conn_id: ConnectionId,
) -> Result<(QueryResult, bool), DbError> {
    let start = std::time::Instant::now();
    let statement_count = sql
        .split(';')
        .filter(|part| !part.trim().is_empty())
        .count();

    if statement_count > 1 {
        client
            .batch_execute(sql)
            .await
            .map_err(|e| DbError::from_pg(&e, conn_id))?;

        let execution_time_ms = start.elapsed().as_millis();
        return Ok((
            QueryResult {
                columns: vec![ColumnMeta {
                    name: "status".to_string(),
                    type_name: "text".to_string(),
                }],
                rows: vec![vec![CellValue::Text(format!(
                    "{statement_count} statements executed"
                ))]],
                execution_time_ms,
            },
            false,
        ));
    }

    let rows_affected = client
        .execute(sql, &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let execution_time_ms = start.elapsed().as_millis();
    Ok((
        QueryResult {
            columns: vec![ColumnMeta {
                name: "rows_affected".to_string(),
                type_name: "int8".to_string(),
            }],
            rows: vec![vec![CellValue::Int(rows_affected as i64)]],
            execution_time_ms,
        },
        false,
    ))
}

async fn fetch_query_rows(
    client: &mut Client,
    stmt: &tokio_postgres::Statement,
    row_limit: Option<usize>,
    conn_id: ConnectionId,
) -> Result<Vec<Row>, DbError> {
    let Some(limit) = row_limit else {
        return client
            .query(stmt, &[])
            .await
            .map_err(|e| DbError::from_pg(&e, conn_id));
    };

    let max_rows = limit.saturating_add(1).min(i32::MAX as usize) as i32;
    let tx = client
        .transaction()
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let portal = tx
        .bind(stmt, &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    let rows = tx
        .query_portal(&portal, max_rows)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    tx.rollback()
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows)
}

/// 행 단위 편집 op 시퀀스를 단일 트랜잭션 안에서 순차 실행한다.
///
/// Plan v7 Phase 1.1 / ADR-2 / ADR-5:
/// - 시그니처: `&mut Client` (트랜잭션 객체 생성을 위해 mut 필요).
/// - 모든 op 가 성공해야 commit, 어느 하나라도 실패하면 rollback (atomicity).
/// - INSERT 의 `RETURNING <pk>` 결과는 `MutationOutcome::inserted_keys` 로 회수.
/// - PK 부재 시 (Update / Delete 의 `pk.is_empty()`) 즉시 에러.
/// - Update / Delete 는 `affected ≠ 1` 시 즉시 rollback (silent multi-row 손상 방지).
pub async fn apply_data_edits(
    client: &mut Client,
    edits: &[RowEditOp],
    conn_id: ConnectionId,
) -> Result<MutationOutcome, DbError> {
    if edits.is_empty() {
        return Ok(MutationOutcome::default());
    }

    let tx = client
        .transaction()
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let mut outcome = MutationOutcome::default();

    for edit in edits {
        match apply_single_op(&tx, edit, conn_id).await {
            Ok(maybe_inserted) => {
                outcome.applied += 1;
                if let Some((tmp_id, pk_values)) = maybe_inserted {
                    outcome.inserted_keys.push((tmp_id, pk_values));
                }
            }
            Err(err) => {
                if let Err(rollback_err) = tx.rollback().await {
                    tracing::warn!(
                        target: "ferrumgrid::mutation",
                        op = "rollback",
                        original_error = %err,
                        rollback_error = %rollback_err,
                        "transaction rollback failed after op error",
                    );
                }
                return Err(err);
            }
        }
    }

    tx.commit()
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(outcome)
}

/// 단일 op 실행 — 성공 시 INSERT RETURNING 결과를 `Option` 으로 반환.
async fn apply_single_op(
    tx: &tokio_postgres::Transaction<'_>,
    edit: &RowEditOp,
    conn_id: ConnectionId,
) -> Result<Option<(uuid::Uuid, Vec<CellValue>)>, DbError> {
    match edit {
        RowEditOp::Insert {
            tmp_id,
            schema,
            table,
            columns,
            returning_pk,
        } => {
            if columns.is_empty() {
                return Err(DbError::internal(
                    conn_id,
                    format!("INSERT into {schema}.{table} requires at least one column."),
                ));
            }
            let sql = build_insert_sql(schema, table, columns, returning_pk);
            if returning_pk.is_empty() {
                tx.execute(sql.as_str(), &[])
                    .await
                    .map_err(|e| DbError::from_pg(&e, conn_id))?;
                Ok(None)
            } else {
                let rows = tx
                    .query(sql.as_str(), &[])
                    .await
                    .map_err(|e| DbError::from_pg(&e, conn_id))?;
                let row = rows.into_iter().next().ok_or_else(|| {
                    DbError::internal(
                        conn_id,
                        format!("INSERT RETURNING for {schema}.{table} returned no rows."),
                    )
                })?;
                let pk_values: Vec<CellValue> = (0..returning_pk.len())
                    .map(|i| extract_cell_value(&row, i))
                    .collect();
                Ok(Some((*tmp_id, pk_values)))
            }
        }
        RowEditOp::Update {
            schema,
            table,
            column,
            pk,
        } => {
            if pk.is_empty() {
                return Err(DbError::internal(
                    conn_id,
                    "UPDATE requires a primary key.",
                ));
            }
            let sql = build_update_sql(schema, table, column, pk);
            let affected = tx
                .execute(sql.as_str(), &[])
                .await
                .map_err(|e| DbError::from_pg(&e, conn_id))?;
            if affected != 1 {
                return Err(DbError::internal(
                    conn_id,
                    format!(
                        "UPDATE {schema}.{table} expected to affect 1 row, got {affected}."
                    ),
                ));
            }
            Ok(None)
        }
        RowEditOp::Delete { schema, table, pk } => {
            if pk.is_empty() {
                return Err(DbError::internal(
                    conn_id,
                    "DELETE requires a primary key.",
                ));
            }
            let sql = build_delete_sql(schema, table, pk);
            let affected = tx
                .execute(sql.as_str(), &[])
                .await
                .map_err(|e| DbError::from_pg(&e, conn_id))?;
            if affected != 1 {
                return Err(DbError::internal(
                    conn_id,
                    format!(
                        "DELETE FROM {schema}.{table} expected to affect 1 row, got {affected}."
                    ),
                ));
            }
            Ok(None)
        }
    }
}

fn extract_cell_value(row: &tokio_postgres::Row, idx: usize) -> CellValue {
    let col_type = row.columns()[idx].type_();

    macro_rules! try_get {
        ($t:ty, $variant:ident) => {
            if let Ok(v) = row.try_get::<_, Option<$t>>(idx) {
                return match v {
                    Some(val) => CellValue::$variant(val),
                    None => CellValue::Null,
                };
            }
        };
        ($t:ty, $variant:ident, $convert:expr) => {
            if let Ok(v) = row.try_get::<_, Option<$t>>(idx) {
                return match v {
                    Some(val) => CellValue::$variant($convert(val)),
                    None => CellValue::Null,
                };
            }
        };
    }

    use tokio_postgres::types::Type;
    match *col_type {
        Type::BOOL => {
            try_get!(bool, Bool);
        }
        Type::INT2 => {
            try_get!(i16, Int, |v: i16| v as i64);
        }
        Type::INT4 => {
            try_get!(i32, Int, |v: i32| v as i64);
        }
        Type::INT8 => {
            try_get!(i64, Int);
        }
        Type::FLOAT4 => {
            try_get!(f32, Float, |v: f32| v as f64);
        }
        Type::FLOAT8 => {
            try_get!(f64, Float);
        }
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
            try_get!(String, Text);
        }
        Type::JSON | Type::JSONB => {
            try_get!(serde_json::Value, Json);
        }
        Type::UUID => {
            try_get!(uuid::Uuid, Uuid);
        }
        Type::BYTEA => {
            try_get!(Vec<u8>, Bytes);
        }
        Type::TIMESTAMP => {
            if let Ok(v) = row.try_get::<_, Option<chrono::NaiveDateTime>>(idx) {
                return match v {
                    Some(val) => CellValue::Timestamp(val.to_string()),
                    None => CellValue::Null,
                };
            }
        }
        Type::TIMESTAMPTZ => {
            if let Ok(v) = row.try_get::<_, Option<chrono::DateTime<chrono::Utc>>>(idx) {
                return match v {
                    Some(val) => CellValue::Timestamp(val.to_string()),
                    None => CellValue::Null,
                };
            }
        }
        Type::DATE => {
            if let Ok(v) = row.try_get::<_, Option<chrono::NaiveDate>>(idx) {
                return match v {
                    Some(val) => CellValue::Timestamp(val.to_string()),
                    None => CellValue::Null,
                };
            }
        }
        Type::NUMERIC => {
            // NUMERIC doesn't have a direct Rust mapping in tokio-postgres without extra features
            // Fall through to string fallback
        }
        _ => {}
    }

    // Fallback: try to get as string
    if let Ok(v) = row.try_get::<_, Option<String>>(idx) {
        return match v {
            Some(val) => CellValue::Unknown(val),
            None => CellValue::Null,
        };
    }

    CellValue::Unknown("<unsupported>".to_string())
}
