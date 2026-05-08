use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use tokio::process::Command;

use crate::types::{BackupFormat, BackupRecord, BackupRequest};

pub async fn run_backup(request: BackupRequest) -> Result<BackupRecord, String> {
    // Built-in SQL engine path — never invoke pg_dump for SqlOnly.
    if request.format == BackupFormat::SqlOnly {
        return crate::db::backup_sql::run_sql_backup(request).await;
    }

    fs::create_dir_all(&request.output_dir)
        .map_err(|err| format!("Backup folder is not writable: {err}"))?;

    let started = Instant::now();
    let completed_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let file_name = backup_file_name(
        &request.config.database,
        request.schema.as_deref(),
        request.format.extension(),
        &stamp,
    );
    let file_path = request.output_dir.join(file_name);

    let mut cmd = Command::new(pg_dump_binary());
    cmd.arg("--host")
        .arg(&request.config.host)
        .arg("--port")
        .arg(request.config.port.to_string())
        .arg("--username")
        .arg(&request.config.username)
        .arg("--dbname")
        .arg(&request.config.database)
        .arg("--format")
        .arg(request.format.pg_dump_format())
        .arg("--file")
        .arg(&file_path)
        .arg("--no-owner")
        .env("PGCONNECT_TIMEOUT", "15");

    if !request.config.password.is_empty() {
        cmd.env("PGPASSWORD", &request.config.password);
    }
    if request.config.use_tls {
        cmd.env("PGSSLMODE", "require");
    }
    if let Some(schema) = request
        .schema
        .as_deref()
        .filter(|schema| !schema.is_empty())
    {
        cmd.arg("--schema").arg(schema);
    }

    let output = cmd.output().await.map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            "pg_dump was not found. Install PostgreSQL client tools and make sure pg_dump is on PATH."
                .to_string()
        } else {
            format!("Failed to start pg_dump: {err}")
        }
    })?;

    if !output.status.success() {
        let _ = fs::remove_file(&file_path);
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(if detail.is_empty() {
            format!("pg_dump failed with status {}", output.status)
        } else {
            detail
        });
    }

    let size_bytes = fs::metadata(&file_path).map(|meta| meta.len()).unwrap_or(0);

    Ok(BackupRecord {
        conn_id: request.conn_id,
        connection_name: request.config.display_name,
        database: request.config.database,
        schema: request.schema,
        format: request.format,
        file_path,
        size_bytes,
        duration_ms: started.elapsed().as_millis(),
        completed_at,
    })
}

pub(super) fn backup_file_name(
    database: &str,
    schema: Option<&str>,
    extension: &str,
    stamp: &str,
) -> String {
    let scope = schema.unwrap_or("full");
    format!(
        "{}_{}_{}.{}",
        sanitize_filename(database),
        sanitize_filename(scope),
        sanitize_filename(stamp),
        extension
    )
}

fn sanitize_filename(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    if sanitized.is_empty() {
        "backup".to_string()
    } else {
        sanitized
    }
}

fn pg_dump_binary() -> PathBuf {
    if let Some(path) = find_on_path("pg_dump") {
        return path;
    }

    for candidate in [
        "/opt/homebrew/bin/pg_dump",
        "/usr/local/bin/pg_dump",
        "/Applications/Postgres.app/Contents/Versions/latest/bin/pg_dump",
    ] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }

    PathBuf::from("pg_dump")
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(binary))
        .find(|path| is_executable_file(path))
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_schema_backup_file_name() {
        let name = backup_file_name("hotel-pos", Some("public"), "dump", "20260504-010203");
        assert_eq!(name, "hotel-pos_public_20260504-010203.dump");
    }

    #[test]
    fn sanitizes_unsafe_file_name_parts() {
        let name = backup_file_name("db/name", Some("public schema"), "sql", "now");
        assert_eq!(name, "db_name_public_schema_now.sql");
    }
}
