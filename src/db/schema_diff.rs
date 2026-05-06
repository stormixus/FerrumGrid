use crate::db::introspect::TableDef;

#[derive(Debug, Clone)]
pub struct SchemaDiff {
    pub tables_added: Vec<TableDef>,
    pub tables_removed: Vec<String>,
    pub tables_modified: Vec<TableDiff>,
}

#[derive(Debug, Clone)]
pub struct TableDiff {
    pub name: String,
    pub columns_added: Vec<ColumnAdd>,
    pub columns_removed: Vec<String>,
    pub columns_modified: Vec<ColumnChange>,
    pub indexes_added: Vec<String>,
    pub indexes_removed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnAdd {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnChange {
    pub name: String,
    pub old_type: String,
    pub new_type: String,
    pub nullable_changed: Option<bool>,
    pub default_changed: Option<(Option<String>, Option<String>)>,
}

impl SchemaDiff {
    pub fn is_empty(&self) -> bool {
        self.tables_added.is_empty()
            && self.tables_removed.is_empty()
            && self.tables_modified.is_empty()
    }

    pub fn summary_counts(&self) -> (usize, usize, usize) {
        (
            self.tables_added.len(),
            self.tables_modified.len(),
            self.tables_removed.len(),
        )
    }
}

impl TableDiff {
    pub fn change_count(&self) -> usize {
        self.columns_added.len()
            + self.columns_removed.len()
            + self.columns_modified.len()
            + self.indexes_added.len()
            + self.indexes_removed.len()
    }
}

pub fn diff_schemas(source: &[TableDef], target: &[TableDef]) -> SchemaDiff {
    let source_map: std::collections::HashMap<&str, &TableDef> =
        source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: std::collections::HashMap<&str, &TableDef> =
        target.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut tables_added = Vec::new();
    let mut tables_removed = Vec::new();
    let mut tables_modified = Vec::new();

    for src_table in source {
        if let Some(tgt_table) = target_map.get(src_table.name.as_str()) {
            let diff = diff_table(src_table, tgt_table);
            if diff.change_count() > 0 {
                tables_modified.push(diff);
            }
        } else {
            tables_added.push(src_table.clone());
        }
    }

    for tgt_table in target {
        if !source_map.contains_key(tgt_table.name.as_str()) {
            tables_removed.push(tgt_table.name.clone());
        }
    }

    SchemaDiff {
        tables_added,
        tables_removed,
        tables_modified,
    }
}

fn diff_table(source: &TableDef, target: &TableDef) -> TableDiff {
    let src_cols: std::collections::HashMap<&str, &crate::db::introspect::ColumnDef> =
        source.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let tgt_cols: std::collections::HashMap<&str, &crate::db::introspect::ColumnDef> =
        target.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    let mut columns_added = Vec::new();
    let mut columns_removed = Vec::new();
    let mut columns_modified = Vec::new();

    for src_col in &source.columns {
        if let Some(tgt_col) = tgt_cols.get(src_col.name.as_str()) {
            let type_changed = src_col.data_type != tgt_col.data_type;
            let nullable_changed = if src_col.is_nullable != tgt_col.is_nullable {
                Some(src_col.is_nullable)
            } else {
                None
            };
            let default_changed = if src_col.default_value != tgt_col.default_value {
                Some((
                    src_col.default_value.clone(),
                    tgt_col.default_value.clone(),
                ))
            } else {
                None
            };

            if type_changed || nullable_changed.is_some() || default_changed.is_some() {
                columns_modified.push(ColumnChange {
                    name: src_col.name.clone(),
                    old_type: tgt_col.data_type.clone(),
                    new_type: src_col.data_type.clone(),
                    nullable_changed,
                    default_changed,
                });
            }
        } else {
            columns_added.push(ColumnAdd {
                name: src_col.name.clone(),
                data_type: src_col.data_type.clone(),
                is_nullable: src_col.is_nullable,
                default_value: src_col.default_value.clone(),
            });
        }
    }

    for tgt_col in &target.columns {
        if !src_cols.contains_key(tgt_col.name.as_str()) {
            columns_removed.push(tgt_col.name.clone());
        }
    }

    let src_idx: std::collections::HashSet<&str> =
        source.indexes.iter().map(|i| i.name.as_str()).collect();
    let tgt_idx: std::collections::HashSet<&str> =
        target.indexes.iter().map(|i| i.name.as_str()).collect();

    let indexes_added: Vec<String> = src_idx.difference(&tgt_idx).map(|s| s.to_string()).collect();
    let indexes_removed: Vec<String> = tgt_idx.difference(&src_idx).map(|s| s.to_string()).collect();

    TableDiff {
        name: source.name.clone(),
        columns_added,
        columns_removed,
        columns_modified,
        indexes_added,
        indexes_removed,
    }
}

pub fn generate_migration_sql(diff: &SchemaDiff, target_schema: &str) -> String {
    let mut sql = String::new();

    sql.push_str("BEGIN;\n\n");

    for table in &diff.tables_added {
        sql.push_str(&crate::db::introspect::generate_create_table_ddl(
            table,
            target_schema,
        ));
        sql.push('\n');
    }

    for table_diff in &diff.tables_modified {
        let fqn = format!(
            "{}.{}",
            quote_ident(target_schema),
            quote_ident(&table_diff.name)
        );

        for col in &table_diff.columns_added {
            let mut col_sql = format!(
                "ALTER TABLE {fqn} ADD COLUMN {} {}",
                quote_ident(&col.name),
                col.data_type
            );
            if !col.is_nullable {
                col_sql.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                col_sql.push_str(&format!(" DEFAULT {default}"));
            }
            col_sql.push_str(";\n");
            sql.push_str(&col_sql);
        }

        for col in &table_diff.columns_removed {
            sql.push_str(&format!(
                "ALTER TABLE {fqn} DROP COLUMN {};\n",
                quote_ident(col)
            ));
        }

        for col in &table_diff.columns_modified {
            if col.old_type != col.new_type {
                sql.push_str(&format!(
                    "ALTER TABLE {fqn} ALTER COLUMN {} TYPE {};\n",
                    quote_ident(&col.name),
                    col.new_type
                ));
            }
            if let Some(nullable) = col.nullable_changed {
                if nullable {
                    sql.push_str(&format!(
                        "ALTER TABLE {fqn} ALTER COLUMN {} DROP NOT NULL;\n",
                        quote_ident(&col.name)
                    ));
                } else {
                    sql.push_str(&format!(
                        "ALTER TABLE {fqn} ALTER COLUMN {} SET NOT NULL;\n",
                        quote_ident(&col.name)
                    ));
                }
            }
            if let Some((new_default, _old_default)) = &col.default_changed {
                if let Some(val) = new_default {
                    sql.push_str(&format!(
                        "ALTER TABLE {fqn} ALTER COLUMN {} SET DEFAULT {val};\n",
                        quote_ident(&col.name)
                    ));
                } else {
                    sql.push_str(&format!(
                        "ALTER TABLE {fqn} ALTER COLUMN {} DROP DEFAULT;\n",
                        quote_ident(&col.name)
                    ));
                }
            }
        }

        sql.push('\n');
    }

    for table_name in &diff.tables_removed {
        sql.push_str(&format!(
            "DROP TABLE {}.{} CASCADE;\n",
            quote_ident(target_schema),
            quote_ident(table_name)
        ));
    }

    sql.push_str("\nCOMMIT;\n");
    sql
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::introspect::{ColumnDef, TableDef};

    fn make_table(name: &str, columns: Vec<ColumnDef>) -> TableDef {
        TableDef {
            schema: "public".to_string(),
            name: name.to_string(),
            columns,
            primary_key: None,
            indexes: vec![],
            check_constraints: vec![],
        }
    }

    fn col(name: &str, data_type: &str, nullable: bool) -> ColumnDef {
        ColumnDef {
            name: name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: nullable,
            default_value: None,
        }
    }

    #[test]
    fn empty_diff_when_identical() {
        let tables = vec![make_table("users", vec![col("id", "int4", false)])];
        let diff = diff_schemas(&tables, &tables);
        assert!(diff.is_empty());
    }

    #[test]
    fn detects_added_table() {
        let source = vec![
            make_table("users", vec![col("id", "int4", false)]),
            make_table("posts", vec![col("id", "int4", false)]),
        ];
        let target = vec![make_table("users", vec![col("id", "int4", false)])];

        let diff = diff_schemas(&source, &target);
        assert_eq!(diff.tables_added.len(), 1);
        assert_eq!(diff.tables_added[0].name, "posts");
        assert!(diff.tables_removed.is_empty());
    }

    #[test]
    fn detects_removed_table() {
        let source = vec![make_table("users", vec![col("id", "int4", false)])];
        let target = vec![
            make_table("users", vec![col("id", "int4", false)]),
            make_table("legacy", vec![col("id", "int4", false)]),
        ];

        let diff = diff_schemas(&source, &target);
        assert!(diff.tables_added.is_empty());
        assert_eq!(diff.tables_removed, vec!["legacy"]);
    }

    #[test]
    fn detects_added_column() {
        let source = vec![make_table(
            "users",
            vec![col("id", "int4", false), col("email", "text", false)],
        )];
        let target = vec![make_table("users", vec![col("id", "int4", false)])];

        let diff = diff_schemas(&source, &target);
        assert_eq!(diff.tables_modified.len(), 1);
        assert_eq!(diff.tables_modified[0].columns_added.len(), 1);
        assert_eq!(diff.tables_modified[0].columns_added[0].name, "email");
    }

    #[test]
    fn detects_removed_column() {
        let source = vec![make_table("users", vec![col("id", "int4", false)])];
        let target = vec![make_table(
            "users",
            vec![col("id", "int4", false), col("old_col", "text", true)],
        )];

        let diff = diff_schemas(&source, &target);
        assert_eq!(diff.tables_modified.len(), 1);
        assert_eq!(diff.tables_modified[0].columns_removed, vec!["old_col"]);
    }

    #[test]
    fn detects_type_change() {
        let source = vec![make_table(
            "users",
            vec![col("email", "character varying(255)", false)],
        )];
        let target = vec![make_table(
            "users",
            vec![col("email", "character varying(100)", false)],
        )];

        let diff = diff_schemas(&source, &target);
        assert_eq!(diff.tables_modified.len(), 1);
        let change = &diff.tables_modified[0].columns_modified[0];
        assert_eq!(change.name, "email");
        assert_eq!(change.old_type, "character varying(100)");
        assert_eq!(change.new_type, "character varying(255)");
    }

    #[test]
    fn detects_nullable_change() {
        let source = vec![make_table("t", vec![col("x", "int4", false)])];
        let target = vec![make_table("t", vec![col("x", "int4", true)])];

        let diff = diff_schemas(&source, &target);
        assert_eq!(diff.tables_modified.len(), 1);
        let change = &diff.tables_modified[0].columns_modified[0];
        assert_eq!(change.nullable_changed, Some(false));
    }

    #[test]
    fn generates_alter_sql() {
        let source = vec![make_table(
            "users",
            vec![
                col("id", "int4", false),
                col("email", "character varying(255)", false),
                col("avatar", "text", true),
            ],
        )];
        let target = vec![make_table(
            "users",
            vec![
                col("id", "int4", false),
                col("email", "character varying(100)", false),
            ],
        )];

        let diff = diff_schemas(&source, &target);
        let sql = generate_migration_sql(&diff, "public");

        assert!(sql.contains("BEGIN;"));
        assert!(sql.contains("COMMIT;"));
        assert!(sql.contains("ADD COLUMN \"avatar\" text"));
        assert!(sql.contains("ALTER COLUMN \"email\" TYPE character varying(255)"));
    }

    #[test]
    fn generates_drop_table_sql() {
        let source = vec![];
        let target = vec![make_table("legacy", vec![col("id", "int4", false)])];

        let diff = diff_schemas(&source, &target);
        let sql = generate_migration_sql(&diff, "public");
        assert!(sql.contains("DROP TABLE \"public\".\"legacy\" CASCADE"));
    }

    #[test]
    fn summary_counts() {
        let source = vec![
            make_table("users", vec![col("id", "int4", false), col("new", "text", true)]),
            make_table("posts", vec![col("id", "int4", false)]),
        ];
        let target = vec![
            make_table("users", vec![col("id", "int4", false)]),
            make_table("old", vec![col("id", "int4", false)]),
        ];

        let diff = diff_schemas(&source, &target);
        let (added, modified, removed) = diff.summary_counts();
        assert_eq!(added, 1);    // posts
        assert_eq!(modified, 1); // users (new column)
        assert_eq!(removed, 1);  // old
    }
}
