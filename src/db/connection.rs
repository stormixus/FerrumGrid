use tokio_postgres::{CancelToken, Client, Config, NoTls, Socket};

use crate::db::error::DbError;
use crate::types::ConnectionConfig;

/// sslmode + (선택) CA root / client cert+key 를 반영한 native-tls connector.
///
/// sslmode 매핑 (native-tls 는 require/verify-ca/verify-full 만 의미 있게 구분):
/// - verify-full: 체인 + 호스트명 모두 검증 (기본)
/// - verify-ca:   체인만 검증, 호스트명 무시
/// - require/prefer/allow: 암호화만, 검증 안 함
#[allow(clippy::result_large_err)]
fn build_tls_connector(cfg: &ConnectionConfig) -> Result<native_tls::TlsConnector, DbError> {
    let mut builder = native_tls::TlsConnector::builder();
    let mode = if cfg.sslmode.is_empty() {
        "require"
    } else {
        cfg.sslmode.as_str()
    };
    match mode {
        "verify-full" => {}
        "verify-ca" => {
            builder.danger_accept_invalid_hostnames(true);
        }
        _ => {
            builder.danger_accept_invalid_certs(true);
            builder.danger_accept_invalid_hostnames(true);
        }
    }

    if let Some(path) = cfg.ssl_root_cert.as_deref().filter(|p| !p.is_empty()) {
        let pem = std::fs::read(path)
            .map_err(|e| DbError::connection(cfg.id, format!("Failed to read CA cert: {e}")))?;
        let cert = native_tls::Certificate::from_pem(&pem)
            .map_err(|e| DbError::connection(cfg.id, format!("Invalid CA cert: {e}")))?;
        builder.add_root_certificate(cert);
    }

    if let (Some(cert_path), Some(key_path)) = (
        cfg.ssl_client_cert.as_deref().filter(|p| !p.is_empty()),
        cfg.ssl_client_key.as_deref().filter(|p| !p.is_empty()),
    ) {
        let cert_pem = std::fs::read(cert_path)
            .map_err(|e| DbError::connection(cfg.id, format!("Failed to read client cert: {e}")))?;
        let key_pem = std::fs::read(key_path)
            .map_err(|e| DbError::connection(cfg.id, format!("Failed to read client key: {e}")))?;
        let identity = native_tls::Identity::from_pkcs8(&cert_pem, &key_pem)
            .map_err(|e| DbError::connection(cfg.id, format!("Invalid client cert/key: {e}")))?;
        builder.identity(identity);
    }

    builder
        .build()
        .map_err(|e| DbError::connection(cfg.id, format!("TLS setup failed: {e}")))
}

pub async fn connect(
    config: &ConnectionConfig,
) -> Result<
    (
        Client,
        tokio_postgres::Connection<Socket, postgres_native_tls::TlsStream<Socket>>,
    ),
    DbError,
> {
    connect_impl(config).await
}

async fn connect_impl(
    cfg: &ConnectionConfig,
) -> Result<
    (
        Client,
        tokio_postgres::Connection<Socket, postgres_native_tls::TlsStream<Socket>>,
    ),
    DbError,
> {
    let mut pg_config = Config::new();
    pg_config
        .host(&cfg.host)
        .port(cfg.port)
        .dbname(&cfg.database)
        .user(&cfg.username)
        .password(&cfg.password)
        .connect_timeout(std::time::Duration::from_secs(10));

    if cfg.use_tls {
        let tls_connector = build_tls_connector(cfg)?;
        let connector = postgres_native_tls::MakeTlsConnector::new(tls_connector);
        pg_config
            .connect(connector)
            .await
            .map_err(|e| DbError::from_pg(&e, cfg.id))
    } else {
        // For non-TLS, we need a different return type, so we use a workaround
        Err(DbError::internal(
            cfg.id,
            "use connect_no_tls for non-TLS connections",
        ))
    }
}

pub async fn connect_no_tls(
    cfg: &ConnectionConfig,
) -> Result<
    (
        Client,
        tokio_postgres::Connection<Socket, tokio_postgres::tls::NoTlsStream>,
    ),
    DbError,
> {
    let mut pg_config = Config::new();
    pg_config
        .host(&cfg.host)
        .port(cfg.port)
        .dbname(&cfg.database)
        .user(&cfg.username)
        .password(&cfg.password)
        .connect_timeout(std::time::Duration::from_secs(10));

    pg_config
        .connect(NoTls)
        .await
        .map_err(|e| DbError::from_pg(&e, cfg.id))
}

pub async fn connect_any(cfg: &ConnectionConfig) -> Result<Client, DbError> {
    if cfg.use_tls {
        let (client, connection) = connect(cfg).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("transfer connection error: {e}");
            }
        });
        Ok(client)
    } else {
        let (client, connection) = connect_no_tls(cfg).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("transfer connection error: {e}");
            }
        });
        Ok(client)
    }
}

pub async fn cancel_query(
    token: CancelToken,
    use_tls: bool,
    conn_id: crate::types::ConnectionId,
) -> Result<(), DbError> {
    if use_tls {
        let tls_connector = native_tls::TlsConnector::builder()
            .build()
            .map_err(|e| DbError::connection(conn_id, format!("TLS setup failed: {e}")))?;
        let connector = postgres_native_tls::MakeTlsConnector::new(tls_connector);
        token
            .cancel_query(connector)
            .await
            .map_err(|e| DbError::from_pg(&e, conn_id))
    } else {
        token
            .cancel_query(NoTls)
            .await
            .map_err(|e| DbError::from_pg(&e, conn_id))
    }
}
