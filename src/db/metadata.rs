use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::{
    ColumnInfo, ConnectionId, FunctionInfo, IndexInfo, RoleInfo, RuleInfo, TableInfo, TriggerInfo,
};

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

pub async fn list_databases(
    client: &Client,
    conn_id: ConnectionId,
) -> Result<Vec<String>, DbError> {
    let rows = client
        .query(
            "SELECT datname \
             FROM pg_catalog.pg_database \
             WHERE datallowconn = true \
             AND datistemplate = false \
             ORDER BY datname",
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
    // US-K2 — pg_class JOIN 으로 table_name + table_type + oid 동시 fetch.
    // information_schema.tables 는 oid 를 직접 노출하지 않으므로 pg_class JOIN.
    let rows = client
        .query(
            "SELECT t.table_name, t.table_type, c.oid::int8, \
                    GREATEST(c.reltuples, 0)::int8 \
             FROM information_schema.tables t \
             LEFT JOIN pg_catalog.pg_namespace n ON n.nspname = t.table_schema \
             LEFT JOIN pg_catalog.pg_class c ON c.relname = t.table_name AND c.relnamespace = n.oid \
             WHERE t.table_schema = $1 \
             ORDER BY t.table_type, t.table_name",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let mut tables: Vec<TableInfo> = rows
        .iter()
        .map(|r| TableInfo {
            name: r.get(0),
            table_type: r.get(1),
            oid: r.get::<_, Option<i64>>(2).map(|v| v as u32),
            row_estimate: r.get::<_, Option<i64>>(3).map(|v| v.max(0) as u64),
        })
        .collect();

    // Also fetch materialized views from pg_catalog (oid included via pg_class.oid 직접 SELECT)
    let mat_rows = client
        .query(
            "SELECT m.matviewname, c.oid::int8, \
                    GREATEST(c.reltuples, 0)::int8 \
             FROM pg_catalog.pg_matviews m \
             LEFT JOIN pg_catalog.pg_namespace n ON n.nspname = m.schemaname \
             LEFT JOIN pg_catalog.pg_class c ON c.relname = m.matviewname AND c.relnamespace = n.oid \
             WHERE m.schemaname = $1 \
             ORDER BY m.matviewname",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    for r in mat_rows {
        tables.push(TableInfo {
            name: r.get(0),
            table_type: "MATERIALIZED VIEW".to_string(),
            oid: r.get::<_, Option<i64>>(1).map(|v| v as u32),
            row_estimate: r.get::<_, Option<i64>>(2).map(|v| v.max(0) as u64),
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
                CASE \
                    WHEN c.data_type = 'USER-DEFINED' THEN c.udt_name \
                    ELSE c.data_type \
                END AS data_type, \
                COALESCE(enum_labels.labels, ARRAY[]::text[]) AS enum_values, \
                c.is_nullable = 'YES' AS is_nullable, \
                c.column_default, \
                COALESCE(tc.constraint_type = 'PRIMARY KEY', false) AS is_pk \
             FROM information_schema.columns c \
             LEFT JOIN pg_catalog.pg_namespace tn \
                ON tn.nspname = c.udt_schema \
             LEFT JOIN pg_catalog.pg_type typ \
                ON typ.typnamespace = tn.oid \
                AND typ.typname = c.udt_name \
             LEFT JOIN LATERAL ( \
                SELECT array_agg(e.enumlabel ORDER BY e.enumsortorder) AS labels \
                FROM pg_catalog.pg_enum e \
                WHERE e.enumtypid = typ.oid \
             ) enum_labels ON true \
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
            enum_values: r.get(2),
            is_nullable: r.get(3),
            default_value: r.get(4),
            is_primary_key: r.get(5),
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

pub async fn list_rules(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<RuleInfo>, DbError> {
    let rows = client
        .query(
            "SELECT
                r.rulename,
                pg_get_ruledef(r.oid, true) AS definition,
                r.ev_enabled <> 'D' AS enabled
             FROM pg_catalog.pg_rewrite r
             JOIN pg_catalog.pg_class c ON c.oid = r.ev_class
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1
               AND c.relname = $2
               AND r.rulename <> '_RETURN'
             ORDER BY r.rulename",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| RuleInfo {
            name: r.get(0),
            definition: r.get(1),
            enabled: r.get(2),
        })
        .collect())
}

pub async fn list_triggers(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<TriggerInfo>, DbError> {
    let rows = client
        .query(
            "SELECT
                tg.tgname,
                pg_get_triggerdef(tg.oid, true) AS definition,
                tg.tgenabled <> 'D' AS enabled
             FROM pg_catalog.pg_trigger tg
             JOIN pg_catalog.pg_class c ON c.oid = tg.tgrelid
             JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
             WHERE n.nspname = $1
               AND c.relname = $2
               AND NOT tg.tgisinternal
             ORDER BY tg.tgname",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| TriggerInfo {
            name: r.get(0),
            definition: r.get(1),
            enabled: r.get(2),
        })
        .collect())
}

pub async fn list_functions(
    client: &Client,
    schema: &str,
    conn_id: ConnectionId,
) -> Result<Vec<FunctionInfo>, DbError> {
    let rows = client
        .query(
            "SELECT
                n.nspname AS schema_name,
                p.proname AS function_name,
                pg_get_function_identity_arguments(p.oid) AS arguments,
                pg_get_function_result(p.oid) AS return_type,
                CASE p.prokind
                    WHEN 'p' THEN 'PROCEDURE'
                    WHEN 'a' THEN 'AGGREGATE'
                    WHEN 'w' THEN 'WINDOW'
                    ELSE 'FUNCTION'
                END AS function_kind,
                l.lanname AS language
             FROM pg_catalog.pg_proc p
             JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
             JOIN pg_catalog.pg_language l ON l.oid = p.prolang
             WHERE n.nspname = $1
             ORDER BY p.proname, arguments",
            &[&schema],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| FunctionInfo {
            schema: r.get(0),
            name: r.get(1),
            arguments: r.get(2),
            return_type: r.get(3),
            kind: r.get(4),
            language: r.get(5),
        })
        .collect())
}

pub async fn list_roles(client: &Client, conn_id: ConnectionId) -> Result<Vec<RoleInfo>, DbError> {
    let rows = client
        .query(
            "SELECT
                rolname,
                rolcanlogin,
                rolsuper,
                rolcreatedb,
                rolcreaterole,
                rolreplication,
                rolvaliduntil::text
             FROM pg_catalog.pg_roles
             ORDER BY rolcanlogin DESC, rolname",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| RoleInfo {
            name: r.get(0),
            can_login: r.get(1),
            is_superuser: r.get(2),
            can_create_db: r.get(3),
            can_create_role: r.get(4),
            can_replicate: r.get(5),
            valid_until: r.get(6),
        })
        .collect())
}
