use crate::state::ConnectionState;

#[derive(Debug, Clone)]
pub struct TableContext {
    pub schema: String,
    pub table: String,
    pub columns: Vec<(String, String)>,
}

/// Build schema context from cached connection metadata for AI prompts.
pub fn collect_schema_context(conn: &ConnectionState) -> Vec<TableContext> {
    let mut tables = Vec::new();

    for (schema, table_list) in &conn.tables {
        for table in table_list {
            let key = (schema.clone(), table.name.clone());
            let columns = conn
                .columns
                .get(&key)
                .map(|cols| {
                    cols.iter()
                        .map(|c| (c.name.clone(), c.data_type.clone()))
                        .collect()
                })
                .unwrap_or_default();
            tables.push(TableContext {
                schema: schema.clone(),
                table: table.name.clone(),
                columns,
            });
        }
    }

    tables.sort_by(|a, b| {
        (&a.schema, &a.table)
            .cmp(&(&b.schema, &b.table))
    });
    tables
}
