//! AWS RDS / Aurora IAM 인증 토큰 생성.
//!
//! AWS SDK(~수십 개 transitive crate)를 끌어오는 대신 OS `aws` CLI 의
//! `aws rds generate-db-auth-token` 를 subprocess 로 호출한다(프로젝트의
//! "lean binary" 철학 유지). 토큰은 ~15분 만료되므로 매 (재)연결 시 생성한다.
//! RDS IAM 은 TLS 를 요구하므로 use_tls 가 켜져 있어야 한다.

use tokio::process::Command;

use crate::db::error::DbError;
use crate::types::ConnectionId;

/// `aws rds generate-db-auth-token` 으로 IAM 인증 토큰(=임시 비밀번호) 생성.
pub async fn generate_rds_token(
    host: &str,
    port: u16,
    username: &str,
    region: Option<&str>,
    conn_id: ConnectionId,
) -> Result<String, DbError> {
    let mut cmd = Command::new("aws");
    cmd.arg("rds")
        .arg("generate-db-auth-token")
        .arg("--hostname")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .arg("--username")
        .arg(username);
    if let Some(region) = region.filter(|r| !r.is_empty()) {
        cmd.arg("--region").arg(region);
    }

    let output = cmd.output().await.map_err(|e| {
        DbError::connection(
            conn_id,
            format!("Failed to run aws CLI (is it installed?): {e}"),
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DbError::connection(
            conn_id,
            format!(
                "aws rds generate-db-auth-token failed: {}",
                stderr.trim().chars().take(300).collect::<String>()
            ),
        ));
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        return Err(DbError::connection(
            conn_id,
            "aws returned an empty IAM token".to_string(),
        ));
    }
    Ok(token)
}
