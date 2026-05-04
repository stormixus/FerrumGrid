use crate::db::bridge::{DbBridge, DbCommand};
use crate::prisma::parser::{PrismaField, PrismaModel, PrismaSchema, PrismaType};
use crate::state::AppState;
use crate::types::{ColumnInfo, ConnectionId};

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub sql_statements: Vec<String>,
}

/// Sync Prisma schema to database (apply to DB)
pub fn sync_schema_to_db(
    schema: &PrismaSchema,
    conn_id: ConnectionId,
    bridge: &DbBridge,
) -> SyncResult {
    let mut sql_statements = Vec::new();

    for model in &schema.models {
        let sql = model.to_sql();
        sql_statements.push(sql.clone());

        // Execute the SQL
        bridge.send(DbCommand::ExecuteQuery {
            conn_id,
            sql,
            row_limit: None,
        });
    }

    SyncResult {
        success: true,
        message: format!("Applied {} models to database", schema.models.len()),
        sql_statements,
    }
}

/// Sync database to Prisma schema (introspection)
pub fn sync_db_to_schema(
    state: &AppState,
    schema_name: &str,
    conn_id: ConnectionId,
) -> Result<PrismaSchema, String> {
    let conn = state
        .connections
        .get(&conn_id)
        .ok_or("Connection not found")?;

    let tables = conn
        .tables
        .get(schema_name)
        .cloned()
        .ok_or("No tables found")?;

    let mut schema = PrismaSchema {
        datasource: Some(crate::prisma::parser::DatasourceBlock {
            name: "db".to_string(),
            provider: "postgresql".to_string(),
            url: "env(\"DATABASE_URL\")".to_string(),
        }),
        generator: Some(crate::prisma::parser::GeneratorBlock {
            name: "client".to_string(),
            provider: "prisma-client-js".to_string(),
            output: None,
        }),
        ..Default::default()
    };

    // Convert tables to models
    for table in tables {
        let key = (schema_name.to_string(), table.name.clone());
        let columns = conn.columns.get(&key).cloned().unwrap_or_default();

        let model = db_table_to_prisma_model(
            &table.name,
            &columns,
            &conn.indexes.get(&key).cloned().unwrap_or_default(),
        );

        schema.models.push(model);
    }

    Ok(schema)
}

/// Generate a migration SQL from Prisma schema changes
pub fn generate_migration(
    old_schema: Option<&PrismaSchema>,
    new_schema: &PrismaSchema,
    migration_name: &str,
) -> String {
    let mut migration = String::new();

    migration.push_str(&format!("-- Migration: {}\n", migration_name));
    migration.push_str(&format!("-- Generated at: {}\n\n", chrono::Utc::now()));

    if let Some(old) = old_schema {
        // Compare and generate ALTER statements
        for new_model in &new_schema.models {
            if let Some(old_model) = old.models.iter().find(|m| m.name == new_model.name) {
                // Model exists - check for changes
                migration.push_str(&generate_alter_table(old_model, new_model));
            } else {
                // New model - CREATE TABLE
                migration.push_str(&new_model.to_sql());
                migration.push('\n');
            }
        }

        // Check for dropped tables
        for old_model in &old.models {
            if !new_schema.models.iter().any(|m| m.name == old_model.name) {
                migration.push_str(&format!(
                    "DROP TABLE IF EXISTS \"{}\" CASCADE;\n",
                    old_model.name
                ));
            }
        }
    } else {
        // No old schema - create all tables
        for model in &new_schema.models {
            migration.push_str(&model.to_sql());
            migration.push('\n');
        }
    }

    migration
}

fn db_table_to_prisma_model(
    table_name: &str,
    columns: &[ColumnInfo],
    indexes: &[crate::types::IndexInfo],
) -> PrismaModel {
    let mut model = PrismaModel {
        name: table_name.to_string(),
        fields: Vec::new(),
        attributes: Vec::new(),
        documentation: None,
    };

    for col in columns {
        let field_type = db_type_to_prisma_field_type(&col.data_type, col.is_nullable);

        let mut field = PrismaField {
            name: col.name.clone(),
            field_type,
            attributes: Vec::new(),
            documentation: None,
        };

        // Add @id attribute for primary keys
        if col.is_primary_key {
            field
                .attributes
                .push(crate::prisma::parser::PrismaAttribute {
                    name: "id".to_string(),
                    arguments: Vec::new(),
                });
        }

        // Add @default for auto-increment
        if col
            .default_value
            .as_ref()
            .is_some_and(|d| d.contains("nextval") || d.contains("serial"))
        {
            field
                .attributes
                .push(crate::prisma::parser::PrismaAttribute {
                    name: "default".to_string(),
                    arguments: vec!["autoincrement()".to_string()],
                });
        }

        model.fields.push(field);
    }

    for index in indexes {
        if index.is_primary {
            continue;
        }
        let attr_name = if index.is_unique { "unique" } else { "index" };
        model
            .attributes
            .push(crate::prisma::parser::PrismaAttribute {
                name: attr_name.to_string(),
                arguments: vec![format!("[{}]", index.columns.join(", "))],
            });
        model.documentation = Some(format!(
            "Includes {} index {} ({})",
            index.index_type,
            index.name,
            index.columns.join(", ")
        ));
    }

    model
}

fn db_type_to_prisma_field_type(db_type: &str, is_nullable: bool) -> PrismaType {
    let base_type = match db_type.to_lowercase().as_str() {
        "character varying" | "varchar" | "char" | "text" | "character" | "bpchar" | "name" => {
            PrismaType::String
        }
        "integer" | "int" | "int4" | "serial" => PrismaType::Int,
        "bigint" | "int8" | "bigserial" => PrismaType::BigInt,
        "smallint" | "int2" => PrismaType::Int,
        "numeric" | "decimal" => PrismaType::Decimal,
        "real" | "float4" => PrismaType::Float,
        "double precision" | "float8" => PrismaType::Float,
        "boolean" | "bool" => PrismaType::Boolean,
        "timestamp"
        | "timestamp without time zone"
        | "timestamptz"
        | "timestamp with time zone" => PrismaType::DateTime,
        "date" => PrismaType::DateTime,
        "time" | "time without time zone" => PrismaType::DateTime,
        "interval" => PrismaType::String, // Prisma doesn't have Interval
        "bytea" => PrismaType::Bytes,
        "json" | "jsonb" => PrismaType::Json,
        "uuid" => PrismaType::String,
        "inet" | "cidr" => PrismaType::String,
        "tsvector" | "tsquery" => PrismaType::String,
        "money" => PrismaType::Decimal,
        "oid" => PrismaType::Int,
        "array" => PrismaType::Array(Box::new(PrismaType::String)),
        _ => {
            // Check for array types
            if db_type.ends_with("[]") {
                let inner = db_type.trim_end_matches("[]");
                let inner_type = db_type_to_prisma_field_type(inner, true);
                return PrismaType::Array(Box::new(inner_type));
            }
            PrismaType::Unsupported(db_type.to_string())
        }
    };

    if is_nullable {
        PrismaType::Optional(Box::new(base_type))
    } else {
        base_type
    }
}

fn generate_alter_table(old_model: &PrismaModel, new_model: &PrismaModel) -> String {
    let mut sql = String::new();

    // Find added columns
    for new_field in &new_model.fields {
        if !old_model.fields.iter().any(|f| f.name == new_field.name) {
            sql.push_str(&format!(
                "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {};\n",
                new_model.name,
                new_field.name,
                field_to_sql_type(&new_field.field_type)
            ));
        }
    }

    // Find removed columns
    for old_field in &old_model.fields {
        if !new_model.fields.iter().any(|f| f.name == old_field.name) {
            sql.push_str(&format!(
                "ALTER TABLE \"{}\" DROP COLUMN \"{}\";\n",
                new_model.name, old_field.name
            ));
        }
    }

    // Find modified columns
    for new_field in &new_model.fields {
        if let Some(old_field) = old_model.fields.iter().find(|f| f.name == new_field.name) {
            if !fields_equal(old_field, new_field) {
                sql.push_str(&format!(
                    "ALTER TABLE \"{}\" ALTER COLUMN \"{}\" TYPE {};\n",
                    new_model.name,
                    new_field.name,
                    field_to_sql_type(&new_field.field_type)
                ));
            }
        }
    }

    sql
}

fn fields_equal(a: &PrismaField, b: &PrismaField) -> bool {
    a.name == b.name && type_equal(&a.field_type, &b.field_type)
}

fn type_equal(a: &PrismaType, b: &PrismaType) -> bool {
    match (a, b) {
        (PrismaType::String, PrismaType::String) => true,
        (PrismaType::Int, PrismaType::Int) => true,
        (PrismaType::BigInt, PrismaType::BigInt) => true,
        (PrismaType::Float, PrismaType::Float) => true,
        (PrismaType::Decimal, PrismaType::Decimal) => true,
        (PrismaType::Boolean, PrismaType::Boolean) => true,
        (PrismaType::DateTime, PrismaType::DateTime) => true,
        (PrismaType::Json, PrismaType::Json) => true,
        (PrismaType::Bytes, PrismaType::Bytes) => true,
        (PrismaType::Optional(a), PrismaType::Optional(b)) => type_equal(a, b),
        (PrismaType::Array(a), PrismaType::Array(b)) => type_equal(a, b),
        (PrismaType::Unsupported(a), PrismaType::Unsupported(b)) => a == b,
        (PrismaType::Model(a), PrismaType::Model(b)) => a == b,
        _ => false,
    }
}

fn field_to_sql_type(field_type: &PrismaType) -> String {
    use crate::prisma::parser::PrismaType;

    let base = match field_type {
        PrismaType::String => "TEXT",
        PrismaType::Int => "INTEGER",
        PrismaType::BigInt => "BIGINT",
        PrismaType::Float => "DOUBLE PRECISION",
        PrismaType::Decimal => "DECIMAL",
        PrismaType::Boolean => "BOOLEAN",
        PrismaType::DateTime => "TIMESTAMPTZ",
        PrismaType::Json => "JSONB",
        PrismaType::Bytes => "BYTEA",
        PrismaType::Optional(inner) => return field_to_sql_type(inner),
        PrismaType::Array(inner) => return format!("{}[]", field_to_sql_type(inner)),
        PrismaType::Unsupported(s) => return s.clone(),
        PrismaType::Model(s) => return s.clone(),
    };

    base.to_string()
}
