use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub enum PrismaCommand {
    Introspect {
        schema_path: String,
        output_path: Option<String>,
    },
    Migrate {
        schema_path: String,
        name: String,
        create_only: bool,
    },
    MigrateDeploy {
        schema_path: String,
    },
    MigrateStatus {
        schema_path: String,
    },
    Generate {
        schema_path: String,
    },
    Validate {
        schema_path: String,
    },
    Format {
        schema_path: String,
    },
    DBPull {
        schema_path: String,
    },
    DBPush {
        schema_path: String,
        force: bool,
    },
}

#[derive(Debug, Clone)]
pub struct PrismaResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub async fn run_prisma_cli(command: PrismaCommand) -> Result<PrismaResult, String> {
    let mut cmd = Command::new("prisma");

    match &command {
        PrismaCommand::Introspect {
            schema_path,
            output_path,
        } => {
            cmd.arg("introspect");
            cmd.arg("--schema").arg(schema_path);
            if let Some(output) = output_path {
                cmd.arg("--output").arg(output);
            }
        }
        PrismaCommand::Migrate {
            schema_path,
            name,
            create_only,
        } => {
            cmd.arg("migrate").arg("dev");
            cmd.arg("--schema").arg(schema_path);
            cmd.arg("--name").arg(name);
            if *create_only {
                cmd.arg("--create-only");
            }
        }
        PrismaCommand::MigrateDeploy { schema_path } => {
            cmd.arg("migrate").arg("deploy");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::MigrateStatus { schema_path } => {
            cmd.arg("migrate").arg("status");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::Generate { schema_path } => {
            cmd.arg("generate");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::Validate { schema_path } => {
            cmd.arg("validate");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::Format { schema_path } => {
            cmd.arg("format");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::DBPull { schema_path } => {
            cmd.arg("db").arg("pull");
            cmd.arg("--schema").arg(schema_path);
        }
        PrismaCommand::DBPush { schema_path, force } => {
            cmd.arg("db").arg("push");
            cmd.arg("--schema").arg(schema_path);
            if *force {
                cmd.arg("--force");
            }
            cmd.arg("--accept-data-loss");
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute Prisma CLI: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(PrismaResult {
        success: output.status.success(),
        stdout,
        stderr,
        exit_code: output.status.code(),
    })
}

pub async fn check_prisma_installed() -> bool {
    match Command::new("prisma").arg("--version").output().await {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn get_prisma_version() -> Option<String> {
    match Command::new("prisma").arg("--version").output().await {
        Ok(output) => {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Create a basic schema.prisma file from connection config
pub fn generate_schema_file(
    provider: &str,
    connection_string: &str,
    output_path: &str,
) -> Result<String, String> {
    let url_line = if connection_string.trim().is_empty() {
        "env(\"DATABASE_URL\")".to_string()
    } else {
        format!("\"{}\"", connection_string.replace('"', "\\\""))
    };

    let schema = format!(
        r#"// This is your Prisma schema file,
// learn more about it in the docs: https://pris.ly/d/prisma-schema

generator client {{
  provider = "prisma-client-js"
}}

datasource db {{
  provider = "{}"
  url      = {}
}}
"#,
        provider, url_line
    );

    if let Some(parent) = std::path::Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create schema directory: {}", e))?;
    }
    std::fs::write(output_path, &schema)
        .map_err(|e| format!("Failed to write schema file: {}", e))?;

    Ok(schema)
}

/// Append model to schema file
pub fn append_model_to_schema(schema_path: &str, model_sql: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file: {}", e))?;

    // Convert SQL table definition to Prisma model
    let model = sql_to_prisma_model(model_sql)?;

    let new_content = format!("{}\n{}", content, model);

    std::fs::write(schema_path, new_content)
        .map_err(|e| format!("Failed to write schema file: {}", e))?;

    Ok(())
}

fn sql_to_prisma_model(sql: &str) -> Result<String, String> {
    // Basic SQL to Prisma model conversion
    // This is a simplified version - a full implementation would require
    // a proper SQL parser

    let sql = sql.trim();

    // Extract table name
    let table_name = sql
        .split_whitespace()
        .skip(2)
        .take(1)
        .next()
        .ok_or("Could not extract table name")?
        .trim_matches('"')
        .trim_matches('`')
        .trim_matches('[');

    let mut model = format!("model {} {{\n", table_name);

    // Extract columns
    if let Some(start) = sql.find('(') {
        if let Some(end) = sql.rfind(')') {
            let columns_str = &sql[start + 1..end];
            let columns: Vec<&str> = columns_str
                .split(',')
                .filter(|s| !s.trim().starts_with("CONSTRAINT"))
                .filter(|s| !s.trim().starts_with("PRIMARY KEY"))
                .filter(|s| !s.trim().starts_with("FOREIGN KEY"))
                .filter(|s| !s.trim().starts_with("UNIQUE"))
                .filter(|s| !s.trim().is_empty())
                .collect();

            for col in columns {
                let parts: Vec<&str> = col.split_whitespace().collect();
                if parts.len() >= 2 {
                    let col_name = parts[0].trim_matches('"').trim_matches('`');
                    let col_type = &parts[1];

                    let prisma_type = sql_type_to_prisma(col_type);
                    let nullable = !col.to_uppercase().contains("NOT NULL");
                    let is_pk = col.to_uppercase().contains("PRIMARY KEY")
                        || col.to_uppercase().contains("SERIAL");

                    let type_str = if nullable {
                        format!("{}?", prisma_type)
                    } else {
                        prisma_type
                    };

                    model.push_str(&format!("  {} {}", col_name, type_str));

                    if is_pk {
                        model.push_str(" @id");
                        if col.to_uppercase().contains("SERIAL") {
                            model.push_str(" @default(autoincrement())");
                        }
                    }

                    model.push('\n');
                }
            }
        }
    }

    model.push_str("}\n");
    Ok(model)
}

fn sql_type_to_prisma(sql_type: &str) -> String {
    let upper = sql_type.to_uppercase();

    if upper.starts_with("VARCHAR") || upper.starts_with("CHAR") || upper == "TEXT" {
        "String".to_string()
    } else if upper.starts_with("INT") || upper == "SERIAL" || upper == "SMALLINT" {
        "Int".to_string()
    } else if upper == "BIGINT" || upper == "BIGSERIAL" {
        "BigInt".to_string()
    } else if upper.starts_with("NUMERIC") || upper.starts_with("DECIMAL") {
        "Decimal".to_string()
    } else if matches!(upper.as_str(), "REAL" | "FLOAT" | "DOUBLE")
        || upper.starts_with("FLOAT")
        || upper.starts_with("DOUBLE PRECISION")
    {
        "Float".to_string()
    } else if upper == "BOOLEAN" || upper == "BOOL" {
        "Boolean".to_string()
    } else if upper.starts_with("TIMESTAMP") || upper == "DATE" {
        "DateTime".to_string()
    } else if upper == "JSON" || upper == "JSONB" {
        "Json".to_string()
    } else if upper == "BYTEA" || upper == "BLOB" {
        "Bytes".to_string()
    } else {
        "String".to_string() // Default fallback
    }
}
