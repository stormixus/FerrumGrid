use tokio_postgres::{CancelToken, Client, Config, NoTls, Socket};

use crate::db::error::DbError;
use crate::types::ConnectionConfig;

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
        let tls_connector = native_tls::TlsConnector::builder()
            .build()
            .map_err(|e| DbError::connection(cfg.id, format!("TLS setup failed: {e}")))?;
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
