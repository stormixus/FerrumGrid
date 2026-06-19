use std::fs;
use std::io::{BufWriter, Write, BufReader, Read};
use std::time::Instant;
use futures_util::{SinkExt, StreamExt};
use tokio_postgres::Client;
use serde::{Serialize, Deserialize};

use crate::db::connection::connect_any;
use crate::types::{BackupRecord, BackupRequest, ConnectionConfig, ForeignKey};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableMetadata {
    pub schema: String,
    pub name: String,
    pub oid: u32,
    pub ddl: String,
    pub constraints: Vec<ConstraintItem>,
    pub indexes: Vec<IndexItem>,
    pub cols: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConstraintItem {
    pub name: String,
    pub def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexItem {
    pub name: String,
    pub def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FgbFk {
    pub name: String,
    pub source_schema: String,
    pub source_table: String,
    pub target_schema: String,
    pub target_table: String,
    pub def: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FgbMetadata {
    pub database_name: String,
    pub schema_scope: Option<String>,
    pub schemas: Vec<String>,
    pub tables: Vec<TableMetadata>,
    pub foreign_keys: Vec<FgbFk>,
}

#[derive(Debug, Clone)]
struct TableRef {
    pub schema: String,
    pub name: String,
    pub oid: u32,
}

use eframe::egui;
use crate::db::bridge::DbResponse;

/// Entry point for FGB backup creation.
pub async fn run_fgb_backup(
    request: BackupRequest,
    resp_tx: std::sync::mpsc::Sender<DbResponse>,
    ctx: egui::Context,
) -> Result<BackupRecord, String> {
    fs::create_dir_all(&request.output_dir)
        .map_err(|err| format!("Backup folder is not writable: {err}"))?;

    let started = Instant::now();
    let completed_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file_name = super::backup::backup_file_name(
        &request.config.database,
        request.schema.as_deref(),
        request.format.extension(),
        &stamp,
    );
    let file_path = request.output_dir.join(file_name);

    match write_fgb_backup(&request, &file_path, &completed_at, resp_tx, ctx).await {
        Ok(()) => {}
        Err(err) => {
            let _ = fs::remove_file(&file_path);
            return Err(err);
        }
    }

    let size_bytes = fs::metadata(&file_path).map(|meta| meta.len()).unwrap_or(0);

    Ok(BackupRecord {
        conn_id: request.conn_id,
        connection_name: request.config.display_name.clone(),
        database: request.config.database.clone(),
        schema: request.schema.clone(),
        format: request.format,
        file_path,
        size_bytes,
        duration_ms: started.elapsed().as_millis(),
        completed_at,
    })
}

async fn write_fgb_backup(
    request: &BackupRequest,
    file_path: &std::path::Path,
    _completed_at: &str,
    resp_tx: std::sync::mpsc::Sender<DbResponse>,
    ctx: egui::Context,
) -> Result<(), String> {
    let _ = resp_tx.send(DbResponse::BackupProgress {
        conn_id: request.conn_id,
        progress: 0.01,
        current_table: "Connecting & Resolving Schemas...".to_string(),
    });
    ctx.request_repaint();

    let client = connect_any(&request.config)
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    let schemas = resolve_schemas(&client, request.schema.as_deref()).await?;

    let mut all_tables: Vec<TableRef> = Vec::new();
    for schema in &schemas {
        let tables = list_tables(&client, schema).await?;
        all_tables.extend(tables);
    }

    // 1. Gathers all metadata (DDL, primary keys, indices)
    let mut tables_meta = Vec::new();
    let total_all_tables = all_tables.len();
    for (idx, table) in all_tables.iter().enumerate() {
        let progress = 0.05 + (idx as f32 / total_all_tables.max(1) as f32) * 0.15; // 5% to 20%
        let _ = resp_tx.send(DbResponse::BackupProgress {
            conn_id: request.conn_id,
            progress,
            current_table: format!("Metadata: {}.{}", table.schema, table.name),
        });
        ctx.request_repaint();

        let ddl = build_create_table(&client, table).await?;
        let constraints_rows = fetch_table_constraints(&client, &table.schema, &table.name).await?;
        let constraints = constraints_rows
            .into_iter()
            .map(|c| ConstraintItem { name: c.name, def: c.def })
            .collect();

        let indexes_rows = fetch_table_indexes(&client, &table.schema, &table.name).await?;
        let indexes = indexes_rows
            .into_iter()
            .map(|(name, def)| IndexItem { name, def })
            .collect();

        let col_rows = client
            .query(
                "SELECT a.attname \
                 FROM pg_attribute a \
                 WHERE a.attrelid = $1 AND a.attnum > 0 AND NOT a.attisdropped \
                 ORDER BY a.attnum",
                &[&table.oid],
            )
            .await
            .map_err(|e| format!("Failed to fetch columns: {e}"))?;

        let cols = col_rows
            .into_iter()
            .map(|r| quote_ident(&r.get::<_, String>(0)))
            .collect();

        tables_meta.push(TableMetadata {
            schema: table.schema.clone(),
            name: table.name.clone(),
            oid: table.oid,
            ddl,
            constraints,
            indexes,
            cols,
        });
    }

    let fks = fetch_foreign_keys(&client, &schemas).await?;
    let mut fgb_fks = Vec::new();
    for fk in fks {
        let def = fetch_fk_def(&client, &fk.source_schema, &fk.source_table, &fk.name).await?;
        fgb_fks.push(FgbFk {
            name: fk.name,
            source_schema: fk.source_schema,
            source_table: fk.source_table,
            target_schema: fk.target_schema,
            target_table: fk.target_table,
            def,
        });
    }

    let meta = FgbMetadata {
        database_name: request.config.database.clone(),
        schema_scope: request.schema.clone(),
        schemas,
        tables: tables_meta,
        foreign_keys: fgb_fks,
    };

    let meta_json = serde_json::to_string(&meta).map_err(|e| format!("Failed to serialize metadata: {e}"))?;
    let meta_bytes = meta_json.as_bytes();
    let meta_len = meta_bytes.len() as u32;

    let raw_file = fs::File::create(file_path)
        .map_err(|err| format!("Cannot create backup file: {err}"))?;
    let mut file = BufWriter::with_capacity(128 * 1024, raw_file);

    // Write file header: Magic (FGB\x01) + Metadata Len (u32) + Metadata (JSON)
    file.write_all(b"FGB\x01").map_err(io_err)?;
    file.write_all(&meta_len.to_be_bytes()).map_err(io_err)?;
    file.write_all(meta_bytes).map_err(io_err)?;

    // 2. Streams table data in block frames
    for (idx, table) in all_tables.iter().enumerate() {
        let progress = 0.20 + (idx as f32 / total_all_tables.max(1) as f32) * 0.80; // 20% to 100%
        let _ = resp_tx.send(DbResponse::BackupProgress {
            conn_id: request.conn_id,
            progress,
            current_table: format!("Data: {}.{}", table.schema, table.name),
        });
        ctx.request_repaint();

        let copy_sql = format!(
            "COPY {}.{} TO STDOUT",
            quote_ident(&table.schema),
            quote_ident(&table.name)
        );
        let reader = client
            .copy_out(&copy_sql)
            .await
            .map_err(|e| format!("COPY OUT failed for {}.{}: {e}", table.schema, table.name))?;

        let mut reader = std::pin::pin!(reader);
        while let Some(chunk) = reader.next().await {
            let bytes = chunk.map_err(|e| format!("COPY OUT chunk read error for {}.{}: {e}", table.schema, table.name))?;
            if bytes.is_empty() {
                continue;
            }
            
            // Frame: "DATA" (4B) + Table Index (4B) + Chunk Len (4B) + Payload
            file.write_all(b"DATA").map_err(io_err)?;
            file.write_all(&(idx as u32).to_be_bytes()).map_err(io_err)?;
            file.write_all(&(bytes.len() as u32).to_be_bytes()).map_err(io_err)?;
            file.write_all(&bytes).map_err(io_err)?;
        }

        // Frame: "DONE" (4B) + Table Index (4B)
        file.write_all(b"DONE").map_err(io_err)?;
        file.write_all(&(idx as u32).to_be_bytes()).map_err(io_err)?;
    }

    file.flush().map_err(io_err)?;
    Ok(())
}

/// Entry point for FGB backup restoration.
pub async fn run_fgb_restore(config: &ConnectionConfig, file_path: &std::path::Path) -> Result<(), String> {
    let raw_file = fs::File::open(file_path)
        .map_err(|err| format!("Cannot open FGB backup file: {err}"))?;
    let mut reader = BufReader::new(raw_file);

    // 1. Read magic & verify
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic).map_err(|e| format!("Failed to read magic header: {e}"))?;
    if &magic != b"FGB\x01" {
        return Err("Invalid backup file: header magic mismatch. Expected '.fgb' format.".to_string());
    }

    // 2. Read metadata length & payload
    let mut meta_len_bytes = [0u8; 4];
    reader.read_exact(&mut meta_len_bytes).map_err(|e| format!("Failed to read metadata length: {e}"))?;
    let meta_len = u32::from_be_bytes(meta_len_bytes);

    let mut meta_bytes = vec![0u8; meta_len as usize];
    reader.read_exact(&mut meta_bytes).map_err(|e| format!("Failed to read metadata payload: {e}"))?;
    let meta: FgbMetadata = serde_json::from_slice(&meta_bytes)
        .map_err(|e| format!("Failed to parse metadata JSON: {e}"))?;

    // 3. Connect to DB
    let client = connect_any(config)
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    // 4. Create schemas
    for schema in &meta.schemas {
        client.execute(&format!("CREATE SCHEMA IF NOT EXISTS {};", quote_ident(schema)), &[])
            .await
            .map_err(|e| format!("Failed to create schema {schema}: {e}"))?;
    }

    // 5. Drop target tables if they already exist, to ensure clean replay
    for table in &meta.tables {
        let drop_sql = format!("DROP TABLE IF EXISTS {}.{} CASCADE;", quote_ident(&table.schema), quote_ident(&table.name));
        client.execute(&drop_sql, &[]).await.ok();
    }

    // 6. Create tables
    for table in &meta.tables {
        client.execute(&table.ddl, &[])
            .await
            .map_err(|e| format!("Failed to create table {}.{}: {e}", table.schema, table.name))?;

        // Primary key, unique, check constraints
        for c in &table.constraints {
            let constraint_sql = format!(
                "ALTER TABLE {}.{} ADD CONSTRAINT {} {};",
                quote_ident(&table.schema),
                quote_ident(&table.name),
                quote_ident(&c.name),
                c.def
            );
            client.execute(&constraint_sql, &[])
                .await
                .map_err(|e| format!("Failed to apply constraint {} on {}.{}: {e}", c.name, table.schema, table.name))?;
        }

        // Indices
        for idx in &table.indexes {
            client.execute(&idx.def, &[])
                .await
                .map_err(|e| format!("Failed to create index {} on {}.{}: {e}", idx.name, table.schema, table.name))?;
        }
    }

    // 7. Stream data back to tables via COPY FROM STDIN
    // Maintain a list of active COPY writers, mapped by table index.
    let mut copy_writers: Vec<Option<std::pin::Pin<Box<tokio_postgres::CopyInSink<bytes::Bytes>>>>> = Vec::new();
    for table in &meta.tables {
        let col_list = table.cols.join(",");
        let copy_sql = format!(
            "COPY {}.{} ({}) FROM stdin",
            quote_ident(&table.schema),
            quote_ident(&table.name),
            col_list
        );
        let writer = client.copy_in(&copy_sql).await
            .map_err(|e| format!("COPY IN failed for {}.{}: {e}", table.schema, table.name))?;
        copy_writers.push(Some(Box::pin(writer)));
    }

    // Read frame loop
    let mut marker = [0u8; 4];
    loop {
        match reader.read_exact(&mut marker) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break; // Clean EOF
            }
            Err(e) => return Err(format!("Error reading frame marker: {e}")),
        }

        if &marker == b"DATA" {
            let mut table_idx_bytes = [0u8; 4];
            reader.read_exact(&mut table_idx_bytes).map_err(|e| format!("Failed to read frame table index: {e}"))?;
            let table_idx = u32::from_be_bytes(table_idx_bytes) as usize;

            let mut chunk_len_bytes = [0u8; 4];
            reader.read_exact(&mut chunk_len_bytes).map_err(|e| format!("Failed to read frame chunk size: {e}"))?;
            let chunk_len = u32::from_be_bytes(chunk_len_bytes) as usize;

            let mut payload = vec![0u8; chunk_len];
            reader.read_exact(&mut payload).map_err(|e| format!("Failed to read frame payload: {e}"))?;

            if let Some(Some(writer)) = copy_writers.get_mut(table_idx) {
                writer.as_mut().send(bytes::Bytes::from(payload)).await
                    .map_err(|e| format!("Failed streaming copy data chunk: {e}"))?;
            }
        } else if &marker == b"DONE" {
            let mut table_idx_bytes = [0u8; 4];
            reader.read_exact(&mut table_idx_bytes).map_err(|e| format!("Failed to read done table index: {e}"))?;
            let table_idx = u32::from_be_bytes(table_idx_bytes) as usize;

            if let Some(opt_writer) = copy_writers.get_mut(table_idx) {
                if let Some(mut writer) = opt_writer.take() {
                    writer.as_mut().finish().await
                        .map_err(|e| format!("Failed to finalize COPY block for table index {table_idx}: {e}"))?;
                }
            }
        } else {
            return Err(format!("Corrupt backup file: unknown frame marker {:?}", marker));
        }
    }

    // 8. Re-apply foreign keys
    for fk in &meta.foreign_keys {
        let fk_sql = format!(
            "ALTER TABLE {}.{} ADD CONSTRAINT {} {};",
            quote_ident(&fk.source_schema),
            quote_ident(&fk.source_table),
            quote_ident(&fk.name),
            fk.def
        );
        client.execute(&fk_sql, &[])
            .await
            .map_err(|e| format!("Failed to restore Foreign Key constraint {}: {e}", fk.name))?;
    }

    Ok(())
}

// Introspection helper adaptations from backup_sql.rs
async fn resolve_schemas(client: &Client, scope: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(s) = scope.filter(|s| !s.trim().is_empty()) {
        let row = client
            .query_one(
                "SELECT 1 FROM pg_namespace WHERE nspname = $1",
                &[&s],
            )
            .await;
        if row.is_err() {
            return Err(format!("Schema '{s}' does not exist in target database"));
        }
        return Ok(vec![s.to_string()]);
    }

    let rows = client
        .query(
            "SELECT nspname FROM pg_namespace \
             WHERE nspname NOT LIKE 'pg_%' AND nspname != 'information_schema' \
             ORDER BY nspname",
            &[],
        )
        .await
        .map_err(|e| format!("Failed to list schemas: {e}"))?;

    Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
}

async fn list_tables(client: &Client, schema: &str) -> Result<Vec<TableRef>, String> {
    let rows = client
        .query(
            "SELECT c.oid, c.relname AS table_name \
             FROM pg_class c \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = $1 AND c.relkind IN ('r','p') \
             ORDER BY c.relname",
            &[&schema],
        )
        .await
        .map_err(|e| format!("Failed to list tables in {schema}: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| TableRef {
            schema: schema.to_string(),
            name: r.get::<_, String>(1),
            oid: r.get::<_, u32>(0),
        })
        .collect())
}

async fn build_create_table(client: &Client, table: &TableRef) -> Result<String, String> {
    let rows = client
        .query(
            "SELECT a.attname AS col_name, \
                    format_type(a.atttypid, a.atttypmod) AS col_type, \
                    a.attnotnull AS not_null, \
                    pg_get_expr(d.adbin, d.adrelid) AS default_expr, \
                    a.attidentity AS identity_kind \
             FROM pg_attribute a \
             LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum \
             WHERE a.attrelid = $1 AND a.attnum > 0 AND NOT a.attisdropped \
             ORDER BY a.attnum",
            &[&table.oid],
        )
        .await
        .map_err(|e| format!("Failed to fetch columns for {}.{}: {e}", table.schema, table.name))?;

    let mut out = String::new();
    out.push_str(&format!(
        "CREATE TABLE {}.{} (\n",
        quote_ident(&table.schema),
        quote_ident(&table.name)
    ));

    let mut first = true;
    for row in &rows {
        let col_name: String = row.get("col_name");
        let col_type: String = row.get("col_type");
        let not_null: bool = row.get("not_null");
        let default_expr: Option<String> = row.get("default_expr");
        let identity: i8 = row.get("identity_kind");

        if !first {
            out.push_str(",\n");
        }
        first = false;

        out.push_str("    ");
        out.push_str(&quote_ident(&col_name));
        out.push(' ');
        out.push_str(&col_type);
        match identity as u8 as char {
            'a' => out.push_str(" GENERATED ALWAYS AS IDENTITY"),
            'd' => out.push_str(" GENERATED BY DEFAULT AS IDENTITY"),
            _ => {
                if let Some(def) = default_expr {
                    out.push_str(" DEFAULT ");
                    out.push_str(&def);
                }
            }
        }
        if not_null {
            out.push_str(" NOT NULL");
        }
    }
    out.push_str("\n);");
    Ok(out)
}

struct ConstraintRow {
    pub name: String,
    pub def: String,
}

async fn fetch_table_constraints(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ConstraintRow>, String> {
    let rows = client
        .query(
            "SELECT con.conname AS name, con.contype AS ctype, \
                    pg_get_constraintdef(con.oid, true) AS def \
             FROM pg_constraint con \
             JOIN pg_class c ON c.oid = con.conrelid \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = $1 AND c.relname = $2 \
               AND con.contype IN ('p','u','c') \
             ORDER BY con.contype, con.conname",
            &[&schema, &table],
        )
        .await
        .map_err(|e| format!("Failed to fetch constraints for {schema}.{table}: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| ConstraintRow {
            name: r.get("name"),
            def: r.get("def"),
        })
        .collect())
}

async fn fetch_table_indexes(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<(String, String)>, String> {
    let rows = client
        .query(
            "SELECT i.indexname, i.indexdef \
             FROM pg_indexes i \
             WHERE i.schemaname = $1 AND i.tablename = $2 \
               AND NOT EXISTS ( \
                 SELECT 1 FROM pg_constraint con \
                 JOIN pg_class c ON c.oid = con.conrelid \
                 JOIN pg_namespace n ON n.oid = c.relnamespace \
                 WHERE n.nspname = i.schemaname AND c.relname = i.tablename \
                   AND con.conname = i.indexname AND con.contype IN ('p','u') \
               )",
            &[&schema, &table],
        )
        .await
        .map_err(|e| format!("Failed to fetch indexes for {schema}.{table}: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| (r.get::<_, String>(0), r.get::<_, String>(1)))
        .collect())
}

async fn fetch_foreign_keys(
    client: &Client,
    schemas: &[String],
) -> Result<Vec<ForeignKey>, String> {
    let rows = client
        .query(
            "SELECT con.conname AS name, \
                    sn.nspname AS source_schema, sc.relname AS source_table, \
                    tn.nspname AS target_schema, tc.relname AS target_table \
             FROM pg_constraint con \
             JOIN pg_class sc ON sc.oid = con.conrelid \
             JOIN pg_namespace sn ON sn.oid = sc.relnamespace \
             JOIN pg_class tc ON tc.oid = con.confrelid \
             JOIN pg_namespace tn ON tn.oid = tc.relnamespace \
             WHERE con.contype = 'f' \
               AND sn.nspname = ANY($1::text[]) \
               AND tn.nspname = ANY($1::text[])",
            &[&schemas],
        )
        .await
        .map_err(|e| format!("Failed to fetch foreign keys: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| ForeignKey {
            name: r.get("name"),
            source_schema: r.get("source_schema"),
            source_table: r.get("source_table"),
            source_column: String::new(),
            target_schema: r.get("target_schema"),
            target_table: r.get("target_table"),
            target_column: String::new(),
        })
        .collect())
}

async fn fetch_fk_def(
    client: &Client,
    schema: &str,
    table: &str,
    name: &str,
) -> Result<String, String> {
    let row = client
        .query_one(
            "SELECT pg_get_constraintdef(con.oid, true) AS def \
             FROM pg_constraint con \
             JOIN pg_class c ON c.oid = con.conrelid \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = $1 AND c.relname = $2 AND con.conname = $3",
            &[&schema, &table, &name],
        )
        .await
        .map_err(|e| format!("Failed to fetch FK def for {schema}.{table}.{name}: {e}"))?;

    Ok(row.get("def"))
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn io_err(err: std::io::Error) -> String {
    format!("Backup file write/read error: {err}")
}
