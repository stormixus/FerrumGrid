use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::{CellValue, ColumnMeta, ConnectionId, QueryResult};

pub async fn execute_query(
    client: &Client,
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

    let pg_rows = client
        .query(&stmt, &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let truncated = row_limit.is_some_and(|limit| pg_rows.len() > limit);
    let take_count = if truncated {
        row_limit.unwrap()
    } else {
        pg_rows.len()
    };

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
