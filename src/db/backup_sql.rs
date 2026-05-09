//! Built-in SQL backup engine for FerrumGrid.
//!
//! Streams DDL + table data over a single `tokio_postgres` connection without
//! shelling out to `pg_dump`. The output is a self-contained, replayable `.sql`
//! file in plain-text COPY format.
//!
//! # v1 scope
//! - CREATE SCHEMA, CREATE TABLE (cols + NOT NULL + DEFAULT)
//! - PRIMARY KEY, UNIQUE, CHECK constraints
//! - Non-constraint indexes (CREATE INDEX)
//! - FOREIGN KEY constraints (added after data load, in dependency order)
//! - Table data via `COPY ... TO STDOUT` (text format)
//!
//! # Out of scope (v1)
//! VIEW, MATERIALIZED VIEW, FUNCTION, PROCEDURE, TRIGGER, SEQUENCE, TYPE,
//! DOMAIN, EXTENSION, RULE, POLICY, ROLE, GRANT, custom collations,
//! partitioning specifics.
//!
//! # Known limitations
//! - When `request.schema` selects a single schema, FKs whose target table
//!   lives in a non-selected schema are silently dropped from the dependency
//!   graph and their `ALTER TABLE ADD CONSTRAINT FOREIGN KEY` clauses are not
//!   emitted (the target wouldn't exist in the dump anyway).
//! - The dump is a *trust-equal* artifact: CHECK / DEFAULT / index expressions
//!   are emitted verbatim from the source DB via `pg_get_constraintdef` /
//!   `pg_get_expr`. Replaying it elevates whatever was in those expressions
//!   to the privileges of the restoring user. Only restore dumps from DBs you
//!   trust.

use std::fs;
use std::io::{BufWriter, Write};
use std::time::Instant;

use futures_util::StreamExt;
use tokio_postgres::Client;

use crate::db::connection::connect_any;
use crate::db::transfer::dependency_order;
use crate::types::{BackupRecord, BackupRequest, ForeignKey};

/// Entry point. Mirrors the pg_dump-based `run_backup` contract.
pub async fn run_sql_backup(request: BackupRequest) -> Result<BackupRecord, String> {
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

    // Wrap the actual work so we can clean up the partial file on any error.
    match write_backup(&request, &file_path, &completed_at).await {
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

async fn write_backup(
    request: &BackupRequest,
    file_path: &std::path::Path,
    completed_at: &str,
) -> Result<(), String> {
    let client = connect_any(&request.config)
        .await
        .map_err(|e| format!("Failed to connect: {e}"))?;

    let schemas = resolve_schemas(&client, request.schema.as_deref()).await?;

    let scope_label = match request.schema.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => "all".to_string(),
    };

    let raw_file = fs::File::create(file_path)
        .map_err(|err| format!("Cannot create backup file: {err}"))?;
    let mut file = BufWriter::with_capacity(64 * 1024, raw_file);

    // Header.
    writeln!(file, "-- FerrumGrid built-in SQL backup")
        .map_err(io_err)?;
    writeln!(file, "-- Generated: {completed_at}").map_err(io_err)?;
    writeln!(file, "-- Database: {}", request.config.database).map_err(io_err)?;
    writeln!(file, "-- Schema scope: {scope_label}").map_err(io_err)?;
    writeln!(file).map_err(io_err)?;

    // 1. CREATE SCHEMA for each schema.
    for schema in &schemas {
        writeln!(
            file,
            "CREATE SCHEMA IF NOT EXISTS {};",
            quote_ident(schema)
        )
        .map_err(io_err)?;
    }
    writeln!(file).map_err(io_err)?;

    // Collect all tables across all schemas (with their oids).
    let mut all_tables: Vec<TableRef> = Vec::new();
    for schema in &schemas {
        let tables = list_tables(&client, schema).await?;
        if tables.is_empty() {
            writeln!(file, "-- No tables in schema {schema}").map_err(io_err)?;
            writeln!(file).map_err(io_err)?;
        }
        all_tables.extend(tables);
    }

    // 2. CREATE TABLE for each table.
    for table in &all_tables {
        let ddl = build_create_table(&client, table).await?;
        writeln!(file, "{ddl}").map_err(io_err)?;
        writeln!(file).map_err(io_err)?;
    }

    // 3. PK / UNIQUE / CHECK constraints (after CREATE TABLE).
    for table in &all_tables {
        let constraints = fetch_table_constraints(&client, &table.schema, &table.name).await?;
        for c in constraints {
            writeln!(
                file,
                "ALTER TABLE {}.{} ADD CONSTRAINT {} {};",
                quote_ident(&table.schema),
                quote_ident(&table.name),
                quote_ident(&c.name),
                c.def
            )
            .map_err(io_err)?;
        }
    }
    writeln!(file).map_err(io_err)?;

    // 4. Indexes (excluding constraint-backing).
    for table in &all_tables {
        let indexes = fetch_table_indexes(&client, &table.schema, &table.name).await?;
        for (_name, def) in indexes {
            writeln!(file, "{def};").map_err(io_err)?;
        }
    }
    writeln!(file).map_err(io_err)?;

    // 5. Foreign keys — collect now so we can topologically sort table data.
    let foreign_keys = fetch_foreign_keys(&client, &schemas).await?;

    // 6. Data dump in dependency order.
    let table_pairs: Vec<(String, String)> = all_tables
        .iter()
        .map(|t| (t.schema.clone(), t.name.clone()))
        .collect();
    let ordered = dependency_order(&table_pairs, &foreign_keys)
        .map_err(|e| format!("Dependency cycle in tables: {e}"))?;

    for (schema, table) in &ordered {
        // Resolve column list for this table by looking up its TableRef.
        let oid = all_tables
            .iter()
            .find(|t| &t.schema == schema && &t.name == table)
            .map(|t| t.oid)
            .ok_or_else(|| format!("Lost table reference: {schema}.{table}"))?;

        dump_table_data(&client, &mut file, schema, table, oid).await?;
    }

    // 7. FK constraints (after data so loads succeed even with mid-graph cycles).
    for fk in &foreign_keys {
        let def = fetch_fk_def(&client, &fk.source_schema, &fk.source_table, &fk.name).await?;
        writeln!(
            file,
            "ALTER TABLE {}.{} ADD CONSTRAINT {} {};",
            quote_ident(&fk.source_schema),
            quote_ident(&fk.source_table),
            quote_ident(&fk.name),
            def
        )
        .map_err(io_err)?;
    }

    file.flush().map_err(io_err)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct TableRef {
    schema: String,
    name: String,
    oid: u32,
}

#[derive(Debug, Clone)]
struct ConstraintRow {
    name: String,
    def: String,
}

async fn resolve_schemas(
    client: &Client,
    requested: Option<&str>,
) -> Result<Vec<String>, String> {
    if let Some(s) = requested.filter(|s| !s.is_empty()) {
        return Ok(vec![s.to_string()]);
    }

    let rows = client
        .query(
            "SELECT nspname FROM pg_namespace \
             WHERE nspname NOT LIKE 'pg_%' \
               AND nspname NOT IN ('information_schema') \
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
        .map_err(|e| {
            format!(
                "Failed to fetch columns for {}.{}: {e}",
                table.schema, table.name
            )
        })?;

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
        // 'a' = ALWAYS, 'd' = BY DEFAULT, '' = not an identity column.
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
            // dependency_order only uses schema+table; column fields can be empty.
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

async fn dump_table_data(
    client: &Client,
    file: &mut BufWriter<fs::File>,
    schema: &str,
    table: &str,
    oid: u32,
) -> Result<(), String> {
    // Column list for the COPY header — must match the order COPY emits.
    let col_rows = client
        .query(
            "SELECT a.attname \
             FROM pg_attribute a \
             WHERE a.attrelid = $1 AND a.attnum > 0 AND NOT a.attisdropped \
             ORDER BY a.attnum",
            &[&oid],
        )
        .await
        .map_err(|e| format!("Failed to fetch columns for COPY header: {e}"))?;

    let cols: Vec<String> = col_rows
        .into_iter()
        .map(|r| quote_ident(&r.get::<_, String>(0)))
        .collect();
    let col_list = cols.join(",");

    writeln!(
        file,
        "COPY {}.{} ({}) FROM stdin;",
        quote_ident(schema),
        quote_ident(table),
        col_list
    )
    .map_err(io_err)?;

    let copy_sql = format!(
        "COPY {}.{} TO STDOUT",
        quote_ident(schema),
        quote_ident(table)
    );
    let reader = client
        .copy_out(&copy_sql)
        .await
        .map_err(|e| format!("COPY OUT failed for {schema}.{table}: {e}"))?;

    let mut reader = std::pin::pin!(reader);
    while let Some(chunk) = reader.next().await {
        let bytes =
            chunk.map_err(|e| format!("COPY OUT read error for {schema}.{table}: {e}"))?;
        file.write_all(&bytes).map_err(io_err)?;
    }

    // Terminate the COPY block (matches pg_dump plain-SQL output style).
    writeln!(file, "\\.").map_err(io_err)?;
    writeln!(file).map_err(io_err)?;

    Ok(())
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn io_err(err: std::io::Error) -> String {
    format!("Backup file write error: {err}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_ident_basic() {
        assert_eq!(quote_ident("public"), "\"public\"");
        assert_eq!(quote_ident("table"), "\"table\"");
    }

    #[test]
    fn quote_ident_escapes_embedded_quotes() {
        assert_eq!(quote_ident("a\"b"), "\"a\"\"b\"");
        assert_eq!(quote_ident("x\"y\"z"), "\"x\"\"y\"\"z\"");
    }

    #[test]
    fn io_err_prefixes_message() {
        let err = std::io::Error::other("disk full");
        let msg = io_err(err);
        assert!(msg.contains("disk full"));
        assert!(msg.starts_with("Backup file write error"));
    }
}
