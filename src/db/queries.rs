use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::{CellValue, ColumnMeta, ConnectionId, DataCellEdit, DataEditValue, QueryResult};

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

pub async fn apply_data_edits(
    client: &Client,
    edits: &[DataCellEdit],
    conn_id: ConnectionId,
) -> Result<usize, DbError> {
    if edits.is_empty() {
        return Ok(0);
    }

    client
        .batch_execute("BEGIN")
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let mut applied = 0usize;
    for edit in edits {
        let sql = build_update_sql(edit, conn_id)?;
        match client.execute(sql.as_str(), &[]).await {
            Ok(affected) if affected == 1 => {
                applied += 1;
            }
            Ok(affected) => {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(DbError::internal(
                    conn_id,
                    format!(
                        "Expected to update exactly 1 row for {}.{}, but PostgreSQL reported {affected}.",
                        edit.schema, edit.table
                    ),
                ));
            }
            Err(err) => {
                let _ = client.batch_execute("ROLLBACK").await;
                return Err(DbError::from_pg(&err, conn_id));
            }
        }
    }

    client
        .batch_execute("COMMIT")
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(applied)
}

fn build_update_sql(edit: &DataCellEdit, conn_id: ConnectionId) -> Result<String, DbError> {
    if edit.pk.is_empty() {
        return Err(DbError::internal(
            conn_id,
            "Table data edits require a primary key.",
        ));
    }

    let set_value = match &edit.value {
        DataEditValue::Null => "NULL".to_string(),
        DataEditValue::Text(value) => sql_literal(value, &edit.column_type),
    };
    let where_clause = edit
        .pk
        .iter()
        .map(|pk| {
            format!(
                "{} = {}",
                quote_ident(&pk.column),
                cell_to_sql_literal(&pk.value, &pk.column_type)
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ");

    Ok(format!(
        "UPDATE {}.{} SET {} = {} WHERE {}",
        quote_ident(&edit.schema),
        quote_ident(&edit.table),
        quote_ident(&edit.column),
        set_value,
        where_clause
    ))
}

fn cell_to_sql_literal(value: &CellValue, type_name: &str) -> String {
    match value {
        CellValue::Null => "NULL".to_string(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Text(v) | CellValue::Timestamp(v) | CellValue::Unknown(v) => {
            sql_literal(v, type_name)
        }
        CellValue::Json(v) => sql_literal(&v.to_string(), type_name),
        CellValue::Uuid(v) => sql_literal(&v.to_string(), type_name),
        CellValue::Bytes(v) => sql_literal(&format!("\\x{}", hex_encode(v)), type_name),
    }
}

fn sql_literal(value: &str, type_name: &str) -> String {
    let lower = type_name.to_ascii_lowercase();
    if is_numeric_type(&lower) && value.trim().parse::<f64>().is_ok() {
        return value.trim().to_string();
    }
    if is_bool_type(&lower) {
        return normalize_bool_literal(value)
            .map(|v| v.to_string())
            .unwrap_or_else(|| quote_literal(value));
    }
    quote_literal(value)
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn is_numeric_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "smallint"
            | "integer"
            | "bigint"
            | "int2"
            | "int4"
            | "int8"
            | "real"
            | "double precision"
            | "float4"
            | "float8"
            | "numeric"
            | "decimal"
    )
}

fn is_bool_type(type_name: &str) -> bool {
    matches!(type_name, "boolean" | "bool")
}

fn normalize_bool_literal(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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
