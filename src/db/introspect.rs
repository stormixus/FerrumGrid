use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone)]
pub struct TableDef {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub primary_key: Option<PrimaryKeyDef>,
    pub indexes: Vec<IndexDef>,
    pub check_constraints: Vec<CheckDef>,
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrimaryKeyDef {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexDef {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone)]
pub struct CheckDef {
    pub name: String,
    pub expression: String,
}

pub async fn introspect_table(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<TableDef, DbError> {
    let columns = fetch_columns(client, schema, table, conn_id).await?;
    let primary_key = fetch_primary_key(client, schema, table, conn_id).await?;
    let indexes = fetch_indexes(client, schema, table, conn_id).await?;
    let check_constraints = fetch_checks(client, schema, table, conn_id).await?;

    Ok(TableDef {
        schema: schema.to_string(),
        name: table.to_string(),
        columns,
        primary_key,
        indexes,
        check_constraints,
    })
}

async fn fetch_columns(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<ColumnDef>, DbError> {
    let rows = client
        .query(
            "SELECT column_name, \
                    CASE \
                        WHEN character_maximum_length IS NOT NULL \
                        THEN data_type || '(' || character_maximum_length || ')' \
                        WHEN numeric_precision IS NOT NULL AND data_type = 'numeric' \
                        THEN data_type || '(' || numeric_precision || ',' || COALESCE(numeric_scale, 0) || ')' \
                        ELSE udt_name::text \
                    END AS full_type, \
                    is_nullable, \
                    column_default \
             FROM information_schema.columns \
             WHERE table_schema = $1 AND table_name = $2 \
             ORDER BY ordinal_position",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| {
            let nullable_str: String = r.get(2);
            ColumnDef {
                name: r.get(0),
                data_type: r.get(1),
                is_nullable: nullable_str == "YES",
                default_value: r.get(3),
            }
        })
        .collect())
}

async fn fetch_primary_key(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Option<PrimaryKeyDef>, DbError> {
    let rows = client
        .query(
            "SELECT tc.constraint_name, kcu.column_name \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
               AND tc.table_schema = kcu.table_schema \
             WHERE tc.constraint_type = 'PRIMARY KEY' \
               AND tc.table_schema = $1 AND tc.table_name = $2 \
             ORDER BY kcu.ordinal_position",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    if rows.is_empty() {
        return Ok(None);
    }

    let name: String = rows[0].get(0);
    let columns: Vec<String> = rows.iter().map(|r| r.get(1)).collect();

    Ok(Some(PrimaryKeyDef { name, columns }))
}

async fn fetch_indexes(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<IndexDef>, DbError> {
    let rows = client
        .query(
            "SELECT indexname, indexdef \
             FROM pg_indexes \
             WHERE schemaname = $1 AND tablename = $2 \
               AND indexname NOT IN ( \
                   SELECT constraint_name FROM information_schema.table_constraints \
                   WHERE constraint_type = 'PRIMARY KEY' \
                     AND table_schema = $1 AND table_name = $2 \
               )",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| IndexDef {
            name: r.get(0),
            definition: r.get(1),
        })
        .collect())
}

async fn fetch_checks(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<Vec<CheckDef>, DbError> {
    let rows = client
        .query(
            "SELECT con.conname, pg_get_constraintdef(con.oid) \
             FROM pg_constraint con \
             JOIN pg_namespace nsp ON nsp.oid = con.connamespace \
             JOIN pg_class cls ON cls.oid = con.conrelid \
             WHERE nsp.nspname = $1 AND cls.relname = $2 AND con.contype = 'c'",
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(rows
        .iter()
        .map(|r| CheckDef {
            name: r.get(0),
            expression: r.get(1),
        })
        .collect())
}

pub fn generate_create_table_ddl(def: &TableDef, target_schema: &str) -> String {
    let mut sql = format!(
        "CREATE TABLE {}.{} (\n",
        quote_ident(target_schema),
        quote_ident(&def.name)
    );

    let mut parts: Vec<String> = Vec::new();

    for col in &def.columns {
        let mut col_sql = format!("    {} {}", quote_ident(&col.name), col.data_type);
        if !col.is_nullable {
            col_sql.push_str(" NOT NULL");
        }
        if let Some(default) = &col.default_value {
            if !is_serial_default(default) {
                col_sql.push_str(&format!(" DEFAULT {default}"));
            }
        }
        parts.push(col_sql);
    }

    if let Some(pk) = &def.primary_key {
        let cols = pk
            .columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("    CONSTRAINT {} PRIMARY KEY ({cols})", quote_ident(&pk.name)));
    }

    for check in &def.check_constraints {
        parts.push(format!(
            "    CONSTRAINT {} {}",
            quote_ident(&check.name),
            check.expression
        ));
    }

    sql.push_str(&parts.join(",\n"));
    sql.push_str("\n);\n");

    for idx in &def.indexes {
        let idx_sql = idx
            .definition
            .replace(
                &format!("{}.{}", quote_ident(&def.schema), quote_ident(&def.name)),
                &format!("{}.{}", quote_ident(target_schema), quote_ident(&def.name)),
            )
            .replace(
                &format!(" ON {}", quote_ident(&def.name)),
                &format!(
                    " ON {}.{}",
                    quote_ident(target_schema),
                    quote_ident(&def.name)
                ),
            );
        sql.push_str(&idx_sql);
        sql.push_str(";\n");
    }

    sql
}

pub async fn get_row_count(
    client: &Client,
    schema: &str,
    table: &str,
    conn_id: ConnectionId,
) -> Result<u64, DbError> {
    let row = client
        .query_one(
            &format!(
                "SELECT reltuples::bigint FROM pg_class \
                 JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace \
                 WHERE nspname = $1 AND relname = $2"
            ),
            &[&schema, &table],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    let count: i64 = row.get(0);
    Ok(count.max(0) as u64)
}

fn is_serial_default(default: &str) -> bool {
    default.starts_with("nextval(")
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_simple_table() {
        let def = TableDef {
            schema: "public".to_string(),
            name: "users".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: "int4".to_string(),
                    is_nullable: false,
                    default_value: Some("nextval('users_id_seq'::regclass)".to_string()),
                },
                ColumnDef {
                    name: "email".to_string(),
                    data_type: "character varying(255)".to_string(),
                    is_nullable: false,
                    default_value: None,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: "text".to_string(),
                    is_nullable: true,
                    default_value: None,
                },
            ],
            primary_key: Some(PrimaryKeyDef {
                name: "users_pkey".to_string(),
                columns: vec!["id".to_string()],
            }),
            indexes: vec![],
            check_constraints: vec![],
        };

        let ddl = generate_create_table_ddl(&def, "staging");
        assert!(ddl.contains("CREATE TABLE \"staging\".\"users\""));
        assert!(ddl.contains("\"id\" int4 NOT NULL"));
        assert!(!ddl.contains("nextval")); // serial defaults stripped
        assert!(ddl.contains("\"email\" character varying(255) NOT NULL"));
        assert!(ddl.contains("\"name\" text"));
        assert!(ddl.contains("PRIMARY KEY (\"id\")"));
    }

    #[test]
    fn generate_table_with_check() {
        let def = TableDef {
            schema: "public".to_string(),
            name: "products".to_string(),
            columns: vec![
                ColumnDef {
                    name: "price".to_string(),
                    data_type: "numeric(10,2)".to_string(),
                    is_nullable: false,
                    default_value: Some("0".to_string()),
                },
            ],
            primary_key: None,
            indexes: vec![],
            check_constraints: vec![CheckDef {
                name: "products_price_check".to_string(),
                expression: "CHECK ((price >= (0)::numeric))".to_string(),
            }],
        };

        let ddl = generate_create_table_ddl(&def, "public");
        assert!(ddl.contains("DEFAULT 0"));
        assert!(ddl.contains("CHECK ((price >= (0)::numeric))"));
    }

    #[test]
    fn serial_default_is_stripped() {
        assert!(is_serial_default("nextval('users_id_seq'::regclass)"));
        assert!(!is_serial_default("0"));
        assert!(!is_serial_default("now()"));
    }
}
