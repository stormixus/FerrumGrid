use crate::types::ConnectionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Connection,
    Query,
    Internal,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DbError {
    pub category: ErrorCategory,
    pub message: String,
    pub detail: Option<String>,
    pub hint: Option<String>,
    pub position: Option<usize>,
    pub sqlstate: Option<String>,
    pub conn_id: ConnectionId,
}

impl DbError {
    pub fn connection(conn_id: ConnectionId, message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Connection,
            message: message.into(),
            detail: None,
            hint: None,
            position: None,
            sqlstate: None,
            conn_id,
        }
    }

    pub fn internal(conn_id: ConnectionId, message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Internal,
            message: message.into(),
            detail: None,
            hint: None,
            position: None,
            sqlstate: None,
            conn_id,
        }
    }

    pub fn cancelled(conn_id: ConnectionId) -> Self {
        Self {
            category: ErrorCategory::Cancelled,
            message: "Query cancelled".to_string(),
            detail: None,
            hint: None,
            position: None,
            sqlstate: None,
            conn_id,
        }
    }

    pub fn from_pg(err: &tokio_postgres::Error, conn_id: ConnectionId) -> Self {
        if let Some(db_err) = err.as_db_error() {
            let category = match db_err.code().code() {
                c if c.starts_with("08") => ErrorCategory::Connection,
                c if c.starts_with("57014") => ErrorCategory::Cancelled,
                _ => ErrorCategory::Query,
            };

            Self {
                category,
                message: db_err.message().to_string(),
                detail: db_err.detail().map(|s| s.to_string()),
                hint: db_err.hint().map(|s| s.to_string()),
                position: db_err.position().map(|p| match p {
                    tokio_postgres::error::ErrorPosition::Original(pos) => {
                        *pos as usize
                    }
                    tokio_postgres::error::ErrorPosition::Internal { position, .. } => {
                        *position as usize
                    }
                }),
                sqlstate: Some(db_err.code().code().to_string()),
                conn_id,
            }
        } else if err.is_closed() {
            Self::connection(conn_id, "Connection closed")
        } else {
            Self {
                category: ErrorCategory::Connection,
                message: err.to_string(),
                detail: None,
                hint: None,
                position: None,
                sqlstate: None,
                conn_id,
            }
        }
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(detail) = &self.detail {
            write!(f, "\nDetail: {detail}")?;
        }
        if let Some(hint) = &self.hint {
            write!(f, "\nHint: {hint}")?;
        }
        if let Some(code) = &self.sqlstate {
            write!(f, " [SQLSTATE: {code}]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error() {
        let err = DbError::connection(ConnectionId::new(), "refused");
        assert_eq!(err.category, ErrorCategory::Connection);
        assert_eq!(err.message, "refused");
    }

    #[test]
    fn test_display() {
        let mut err = DbError::connection(ConnectionId::new(), "bad query");
        err.detail = Some("column missing".into());
        err.hint = Some("check schema".into());
        err.sqlstate = Some("42703".into());
        let s = err.to_string();
        assert!(s.contains("bad query"));
        assert!(s.contains("column missing"));
        assert!(s.contains("check schema"));
        assert!(s.contains("42703"));
    }
}
