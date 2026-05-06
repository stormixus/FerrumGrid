use crate::db::introspect;
use crate::db::schema_diff::{diff_schemas, SchemaDiff};
use crate::types::ConnectionConfig;

pub async fn compare_schemas(
    source_config: &ConnectionConfig,
    target_config: &ConnectionConfig,
    source_schema: &str,
    target_schema: &str,
) -> Result<SchemaDiff, String> {
    let source_client = crate::db::connection::connect_any(source_config)
        .await
        .map_err(|e| format!("Source connection: {e}"))?;

    let target_client = crate::db::connection::connect_any(target_config)
        .await
        .map_err(|e| format!("Target connection: {e}"))?;

    let source_tables =
        introspect_all_tables(&source_client, source_schema, source_config.id).await?;
    let target_tables =
        introspect_all_tables(&target_client, target_schema, target_config.id).await?;

    Ok(diff_schemas(&source_tables, &target_tables))
}

async fn introspect_all_tables(
    client: &tokio_postgres::Client,
    schema: &str,
    conn_id: crate::types::ConnectionId,
) -> Result<Vec<introspect::TableDef>, String> {
    let tables = crate::db::metadata::list_tables(client, schema, conn_id)
        .await
        .map_err(|e| format!("list tables: {e}"))?;

    let mut defs = Vec::new();
    for table in &tables {
        if table.table_type != "BASE TABLE" {
            continue;
        }
        let def = introspect::introspect_table(client, schema, &table.name, conn_id)
            .await
            .map_err(|e| format!("introspect {}: {e}", table.name))?;
        defs.push(def);
    }
    Ok(defs)
}
