//! SSH 터널 — OS `ssh` 클라이언트를 subprocess 로 띄워 로컬 포트포워딩.
//!
//! russh 같은 무거운 의존성 대신 시스템 `ssh -N -L` 를 사용한다(프로젝트의
//! "lean binary" 철학 유지). 키/agent 인증은 ssh 가 ~/.ssh/config + agent 로
//! 자동 처리하며, `-i` 로 개인키 경로를 지정할 수 있다. (비밀번호 인증은
//! 비대화형으로 지원하지 않음 — 키/agent 사용 권장.)

use std::time::Duration;

use tokio::net::TcpStream;
use tokio::process::{Child, Command};

use crate::db::error::DbError;
use crate::types::{ConnectionId, SshTunnelConfig};

/// 살아있는 터널. Drop 시 ssh 자식 프로세스를 종료한다.
pub struct Tunnel {
    child: Child,
    pub local_port: u16,
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        // 자식 ssh 종료 (best-effort).
        let _ = self.child.start_kill();
    }
}

/// 사용 가능한 로컬 포트를 OS 로부터 할당받는다 (bind→포트확인→해제).
#[allow(clippy::result_large_err)]
fn pick_local_port(conn_id: ConnectionId) -> Result<u16, DbError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| DbError::connection(conn_id, format!("Failed to allocate local port: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| DbError::connection(conn_id, format!("Failed to read local port: {e}")))?
        .port();
    drop(listener);
    Ok(port)
}

/// SSH 터널을 열고, 로컬 포트가 listen 할 때까지 대기 후 `Tunnel` 반환.
/// 호출자는 이후 127.0.0.1:`local_port` 로 DB 에 접속한다.
pub async fn open_tunnel(
    cfg: &SshTunnelConfig,
    db_host: &str,
    db_port: u16,
    conn_id: ConnectionId,
) -> Result<Tunnel, DbError> {
    let local_port = pick_local_port(conn_id)?;
    let ssh_port = if cfg.port == 0 { 22 } else { cfg.port };
    let target = if cfg.username.is_empty() {
        cfg.host.clone()
    } else {
        format!("{}@{}", cfg.username, cfg.host)
    };

    let mut cmd = Command::new("ssh");
    cmd.arg("-N") // 원격 명령 실행 없이 포워딩만
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-o")
        .arg("BatchMode=yes") // 비대화형 (암호 프롬프트 방지 → 키/agent 필수)
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-p")
        .arg(ssh_port.to_string())
        .arg("-L")
        .arg(format!("{local_port}:{db_host}:{db_port}"));
    if let Some(key) = cfg.key_path.as_deref().filter(|p| !p.is_empty()) {
        cmd.arg("-i").arg(key);
    }
    cmd.arg(target);
    cmd.kill_on_drop(true);
    cmd.stdin(std::process::Stdio::null());

    let child = cmd
        .spawn()
        .map_err(|e| DbError::connection(conn_id, format!("Failed to spawn ssh: {e}")))?;
    let mut tunnel = Tunnel { child, local_port };

    // 로컬 포트가 listen 시작할 때까지 폴링 (최대 ~10s).
    for _ in 0..100 {
        // ssh 가 일찍 죽었는지 확인.
        if let Ok(Some(status)) = tunnel.child.try_wait() {
            return Err(DbError::connection(
                conn_id,
                format!("SSH tunnel exited early ({status}); check key/agent and host"),
            ));
        }
        if TcpStream::connect(("127.0.0.1", local_port)).await.is_ok() {
            return Ok(tunnel);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(DbError::connection(
        conn_id,
        "SSH tunnel did not become ready within 10s".to_string(),
    ))
}
