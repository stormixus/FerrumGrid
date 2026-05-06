use futures_util::{SinkExt, StreamExt};
use std::time::Instant;

use crate::db::bridge::DbResponse;
use crate::db::introspect;
use crate::state::transfer::{
    IfExists, TransferProgress, TransferRequest, TransferResult,
};

pub async fn execute_transfer(
    request: TransferRequest,
    resp_tx: std::sync::mpsc::Sender<DbResponse>,
    ctx: eframe::egui::Context,
) -> TransferResult {
    let start = Instant::now();

    let source = match crate::db::connection::connect_any(&request.source_config).await {
        Ok(c) => c,
        Err(e) => {
            return TransferResult {
                tables_success: 0,
                tables_failed: request.tables.len(),
                tables_skipped: 0,
                total_rows: 0,
                duration_ms: start.elapsed().as_millis(),
                errors: vec![("connection".to_string(), format!("Source: {e}"))],
            };
        }
    };

    let target = match crate::db::connection::connect_any(&request.target_config).await {
        Ok(c) => c,
        Err(e) => {
            return TransferResult {
                tables_success: 0,
                tables_failed: request.tables.len(),
                tables_skipped: 0,
                total_rows: 0,
                duration_ms: start.elapsed().as_millis(),
                errors: vec![("connection".to_string(), format!("Target: {e}"))],
            };
        }
    };

    let total_tables = request.tables.len();
    let mut success = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut total_rows = 0u64;
    let mut errors: Vec<(String, String)> = Vec::new();

    for (i, table_name) in request.tables.iter().enumerate() {
        let _ = resp_tx.send(DbResponse::TransferProgress {
            progress: TransferProgress {
                current_table: table_name.clone(),
                current_table_index: i,
                total_tables,
                rows_transferred: 0,
                rows_total: None,
                bytes_transferred: 0,
            },
        });
        ctx.request_repaint();

        match transfer_one_table(
            &source,
            &target,
            &request.source_schema,
            &request.target_schema,
            table_name,
            &request.options,
            &request.source_config,
            i,
            total_tables,
            &resp_tx,
            &ctx,
        )
        .await
        {
            Ok(rows) => {
                total_rows += rows;
                success += 1;
            }
            Err(TransferTableError::Skipped) => {
                skipped += 1;
            }
            Err(TransferTableError::Failed(msg)) => {
                errors.push((table_name.clone(), msg));
                failed += 1;
            }
        }
    }

    TransferResult {
        tables_success: success,
        tables_failed: failed,
        tables_skipped: skipped,
        total_rows,
        duration_ms: start.elapsed().as_millis(),
        errors,
    }
}

enum TransferTableError {
    Skipped,
    Failed(String),
}

#[allow(clippy::too_many_arguments)]
async fn transfer_one_table(
    source: &tokio_postgres::Client,
    target: &tokio_postgres::Client,
    source_schema: &str,
    target_schema: &str,
    table_name: &str,
    options: &crate::state::transfer::TransferOptions,
    source_config: &crate::types::ConnectionConfig,
    table_index: usize,
    total_tables: usize,
    resp_tx: &std::sync::mpsc::Sender<DbResponse>,
    ctx: &eframe::egui::Context,
) -> Result<u64, TransferTableError> {
    let table_exists = check_table_exists(target, target_schema, table_name)
        .await
        .map_err(|e| TransferTableError::Failed(e.to_string()))?;

    if table_exists {
        match options.if_exists {
            IfExists::Skip => return Err(TransferTableError::Skipped),
            IfExists::Drop => {
                let drop_sql = format!(
                    "DROP TABLE IF EXISTS {}.{} CASCADE",
                    quote_ident(target_schema),
                    quote_ident(table_name)
                );
                target
                    .execute(&drop_sql, &[])
                    .await
                    .map_err(|e| TransferTableError::Failed(format!("DROP: {e}")))?;
            }
            IfExists::Truncate => {
                let truncate_sql = format!(
                    "TRUNCATE TABLE {}.{} CASCADE",
                    quote_ident(target_schema),
                    quote_ident(table_name)
                );
                target
                    .execute(&truncate_sql, &[])
                    .await
                    .map_err(|e| TransferTableError::Failed(format!("TRUNCATE: {e}")))?;
            }
        }
    }

    if !table_exists || options.if_exists == IfExists::Drop {
        let def = introspect::introspect_table(source, source_schema, table_name, source_config.id)
            .await
            .map_err(|e| TransferTableError::Failed(format!("introspect: {e}")))?;

        let ddl = introspect::generate_create_table_ddl(&def, target_schema);
        target
            .batch_execute(&ddl)
            .await
            .map_err(|e| TransferTableError::Failed(format!("DDL: {e}")))?;
    }

    if !options.include_data {
        return Ok(0);
    }

    let row_count = introspect::get_row_count(source, source_schema, table_name, source_config.id)
        .await
        .unwrap_or(0);

    let copy_out_sql = format!(
        "COPY {}.{} TO STDOUT",
        quote_ident(source_schema),
        quote_ident(table_name)
    );
    let copy_in_sql = format!(
        "COPY {}.{} FROM STDIN",
        quote_ident(target_schema),
        quote_ident(table_name)
    );

    let reader = source
        .copy_out(&copy_out_sql)
        .await
        .map_err(|e| TransferTableError::Failed(format!("COPY OUT: {e}")))?;

    let writer = target
        .copy_in(&copy_in_sql)
        .await
        .map_err(|e| TransferTableError::Failed(format!("COPY IN: {e}")))?;

    let mut reader = std::pin::pin!(reader);
    let mut writer = Box::pin(writer);
    let mut bytes_transferred = 0u64;
    let mut rows_transferred = 0u64;

    while let Some(chunk) = reader.next().await {
        let data =
            chunk.map_err(|e| TransferTableError::Failed(format!("COPY OUT read: {e}")))?;

        let newline_count = data.iter().filter(|&&b| b == b'\n').count() as u64;
        rows_transferred += newline_count;
        bytes_transferred += data.len() as u64;

        writer
            .send(data)
            .await
            .map_err(|e| TransferTableError::Failed(format!("COPY IN write: {e}")))?;

        if bytes_transferred % (8 * 1024 * 1024) < 65536 {
            let _ = resp_tx.send(DbResponse::TransferProgress {
                progress: TransferProgress {
                    current_table: table_name.to_string(),
                    current_table_index: table_index,
                    total_tables,
                    rows_transferred,
                    rows_total: if row_count > 0 { Some(row_count) } else { None },
                    bytes_transferred,
                },
            });
            ctx.request_repaint();
        }
    }

    writer
        .close()
        .await
        .map_err(|e| TransferTableError::Failed(format!("COPY IN close: {e}")))?;

    Ok(rows_transferred)
}

async fn check_table_exists(
    client: &tokio_postgres::Client,
    schema: &str,
    table: &str,
) -> Result<bool, tokio_postgres::Error> {
    let row = client
        .query_one(
            "SELECT EXISTS( \
                SELECT 1 FROM information_schema.tables \
                WHERE table_schema = $1 AND table_name = $2 \
            )",
            &[&schema, &table],
        )
        .await?;
    Ok(row.get(0))
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
