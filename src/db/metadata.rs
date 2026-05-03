use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::{ColumnInfo, ConnectionId, IndexInfo, TableInfo};

pub async fn list_schemas(client: &Client, conn_id: ConnectionId) -> Result<Vec<String>, DbError> {
    let rows = client
        .query(
            "SELECT schema_name FROM information_schema.schemata \
             WHERE schema_name NOT LIKE 'pg_toast%' \
             AND schema_name NOT LIKE 'pg_temp%' \
             ORDER BY schema_name",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows.iter().map(|r| r.get(0)).collect())
}

pub async fn list_tables(
    client: &Client,
    schema: &str,
    conn_id: ConnectionId,
) -> Result<Vec<TableInfo>, DbError> {
    let rows = client
        .query(
            "SELECT table_name, table_type FROM information_schema.tables \
             WHERE table_schema = $1 \
             ORDER BY table_type, table_name",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let mut tables: Vec<TableInfo> = rows
        .iter()
        .map(|r| TableInfo {
            name: r.get(0),
            table_type: r.get(1),
        })
        .collect();

    // Also fetch materialized views from pg_catalog
    let mat_rows = client
        .query(
            "SELECT matviewname FROM pg_catalog.pg_matviews \
             WHERE schemaname = $1 \
             ORDER BY matviewname",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    for r in mat_rows {
        tables.push(TableInfo {
            name: r.get(0),
            table_type: "MATERIALIZED VIEW".to_string(),
        });
    }

    Ok(tables)
}

pub async fn list_columns(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<ColumnInfo>, DbError> {
    let rows = client
        .query(
            "SELECT \
                c.column_name, \
                c.data_type, \
                c.is_nullable = 'YES' AS is_nullable, \
                c.column_default, \
                COALESCE(tc.constraint_type = 'PRIMARY KEY', false) AS is_pk \
             FROM information_schema.columns c \
             LEFT JOIN information_schema.key_column_usage kcu \
                ON c.table_schema = kcu.table_schema \
                AND c.table_name = kcu.table_name \
                AND c.column_name = kcu.column_name \
             LEFT JOIN information_schema.table_constraints tc \
                ON kcu.constraint_schema = tc.constraint_schema \
                AND kcu.constraint_name = tc.constraint_name \
                AND tc.constraint_type = 'PRIMARY KEY' \
             WHERE c.table_schema = $1 AND c.table_name = $2 \
             ORDER BY c.ordinal_position",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| ColumnInfo {
            name: r.get(0),
            data_type: r.get(1),
            is_nullable: r.get(2),
            default_value: r.get(3),
            is_primary_key: r.get(4),
        })
        .collect())
}

pub async fn list_indexes(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<IndexInfo>, DbError> {
    let rows = client
        .query(
            "SELECT \
                i.relname AS index_name, \
                array_agg(a.attname ORDER BY x.ordinality) AS columns, \
                ix.indisunique AS is_unique, \
                ix.indisprimary AS is_primary, \
                am.amname AS index_type \
             FROM pg_catalog.pg_index ix \
             JOIN pg_catalog.pg_class t ON t.oid = ix.indrelid \
             JOIN pg_catalog.pg_class i ON i.oid = ix.indexrelid \
             JOIN pg_catalog.pg_namespace n ON n.oid = t.relnamespace \
             JOIN pg_catalog.pg_am am ON am.oid = i.relam \
             CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS x(attnum, ordinality) \
             JOIN pg_catalog.pg_attribute a ON a.attrelid = t.oid AND a.attnum = x.attnum \
             WHERE n.nspname = $1 AND t.relname = $2 \
             GROUP BY i.relname, ix.indisunique, ix.indisprimary, am.amname \
             ORDER BY i.relname",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| {
            let columns: Vec<String> = r.get(1);
            IndexInfo {
                name: r.get(0),
                columns,
                is_unique: r.get(2),
                is_primary: r.get(3),
                index_type: r.get(4),
            }
        })
        .collect())
}
