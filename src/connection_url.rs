use percent_encoding::percent_decode_str;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConnectionUrl {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl PostgresConnectionUrl {
    pub fn suggested_display_name(&self) -> String {
        format!(
            "{}@{}:{}/{}",
            self.username, self.host, self.port, self.database
        )
    }

    pub fn has_password(&self) -> bool {
        !self.password.is_empty()
    }

    /// 폼 필드 → `postgres://user:pass@host:port/db[?sslmode=require]` URL.
    /// `parse_postgres_connection_url` 의 역방향 (라운드트립).
    pub fn to_url(&self) -> String {
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
        let user = utf8_percent_encode(&self.username, NON_ALPHANUMERIC).to_string();
        let auth = if self.password.is_empty() {
            user
        } else {
            let pass = utf8_percent_encode(&self.password, NON_ALPHANUMERIC).to_string();
            format!("{user}:{pass}")
        };
        let db = utf8_percent_encode(&self.database, NON_ALPHANUMERIC).to_string();
        let mut url = format!("postgres://{auth}@{}:{}/{}", self.host, self.port, db);
        if self.use_tls {
            url.push_str("?sslmode=require");
        }
        url
    }
}

pub fn parse_postgres_connection_url(raw: &str) -> Option<PostgresConnectionUrl> {
    let raw = normalize_clipboard_url(raw)?;
    let url = Url::parse(raw).ok()?;
    let scheme = url.scheme();
    if scheme != "postgres" && scheme != "postgresql" {
        return None;
    }

    let host = url.host_str()?.to_string();
    let port = url.port().unwrap_or(5432);
    let database = url
        .path_segments()
        .and_then(|mut segments| segments.next())
        .filter(|value| !value.is_empty())
        .map(decode_component)
        .unwrap_or_else(|| "postgres".to_string());
    let username = if url.username().is_empty() {
        "postgres".to_string()
    } else {
        decode_component(url.username())
    };
    let password = url.password().map(decode_component).unwrap_or_default();
    let use_tls = infer_tls(&url);

    Some(PostgresConnectionUrl {
        host,
        port,
        database,
        username,
        password,
        use_tls,
    })
}

fn normalize_clipboard_url(raw: &str) -> Option<&str> {
    let mut trimmed = raw.trim().trim_matches(|ch| matches!(ch, '"' | '\'' | '`'));
    if let Some((name, value)) = trimmed.split_once('=') {
        let name = name.trim();
        if name == "DATABASE_URL" || name == "export DATABASE_URL" {
            trimmed = value
                .trim()
                .trim_matches(|ch| matches!(ch, '"' | '\'' | '`'));
        }
    }

    let scheme_probe = trimmed.to_ascii_lowercase();
    if scheme_probe.starts_with("postgres://") || scheme_probe.starts_with("postgresql://") {
        Some(trimmed)
    } else {
        None
    }
}

fn decode_component(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

fn infer_tls(url: &Url) -> bool {
    for (key, value) in url.query_pairs() {
        if key.eq_ignore_ascii_case("sslmode") {
            return matches!(
                value.to_ascii_lowercase().as_str(),
                "allow" | "prefer" | "require" | "verify-ca" | "verify-full"
            );
        }

        if key.eq_ignore_ascii_case("ssl") || key.eq_ignore_ascii_case("tls") {
            return matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "require" | "required"
            );
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_postgres_url() {
        let parsed =
            parse_postgres_connection_url("postgres://alice:secret@example.com:6543/app_db")
                .unwrap();

        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 6543);
        assert_eq!(parsed.database, "app_db");
        assert_eq!(parsed.username, "alice");
        assert_eq!(parsed.password, "secret");
        assert!(!parsed.use_tls);
    }

    #[test]
    fn parses_postgresql_url_with_decoding_and_tls() {
        let parsed = parse_postgres_connection_url(
            "postgresql://user%40mail:p%40ss%2Fword@db.example.com/my%20db?sslmode=require",
        )
        .unwrap();

        assert_eq!(parsed.database, "my db");
        assert_eq!(parsed.username, "user@mail");
        assert_eq!(parsed.password, "p@ss/word");
        assert!(parsed.use_tls);
    }

    #[test]
    fn accepts_env_assignment_wrapping() {
        let parsed =
            parse_postgres_connection_url("'DATABASE_URL=postgres://postgres@localhost/postgres'")
                .unwrap();

        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, 5432);
        assert_eq!(parsed.database, "postgres");
        assert_eq!(parsed.username, "postgres");
    }

    #[test]
    fn accepts_env_assignment_with_spaces() {
        let parsed = parse_postgres_connection_url(
            "export DATABASE_URL = \"postgres://root@localhost/app\"",
        )
        .unwrap();

        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.database, "app");
        assert_eq!(parsed.username, "root");
    }

    #[test]
    fn rejects_non_postgres_urls() {
        assert!(parse_postgres_connection_url("https://example.com").is_none());
        assert!(parse_postgres_connection_url("not a url").is_none());
    }

    #[test]
    fn to_url_round_trips_through_parser() {
        let original = PostgresConnectionUrl {
            host: "db.example.com".to_string(),
            port: 6543,
            database: "my db".to_string(),
            username: "user@mail".to_string(),
            password: "p@ss/word".to_string(),
            use_tls: true,
        };
        let url = original.to_url();
        let parsed = parse_postgres_connection_url(&url).expect("round-trips");
        assert_eq!(parsed, original);
    }

    #[test]
    fn to_url_omits_empty_password_and_sslmode() {
        let url = PostgresConnectionUrl {
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            use_tls: false,
        }
        .to_url();
        assert_eq!(url, "postgres://postgres@localhost:5432/postgres");
    }
}
