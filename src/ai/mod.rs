//! AI text-to-SQL backend facade.
//!
//! UI code calls this module while backend dispatch, endpoint handling, and
//! schema collection live in focused submodules.

mod schema;
mod service;

pub use schema::{collect_schema_context, TableContext};

/// Background AI job slot shared with the UI.
#[derive(Default)]
pub struct AiJob {
    pub running: bool,
    /// Ok(sql/text) or Err(message). The UI drains this once per frame.
    pub result: Option<Result<String, String>>,
}

/// Natural language request -> one PostgreSQL statement using a rendered schema string.
pub fn generate_sql(
    prompt: &str,
    schema: &str,
    settings: &crate::storage::settings::AppSettings,
) -> Result<String, String> {
    service::generate_sql_from_text_schema(prompt, schema, settings)
}

/// Natural language request -> one PostgreSQL statement using structured table metadata.
pub fn generate_sql_with_tables(
    prompt: &str,
    schema: &[TableContext],
    settings: &crate::storage::settings::AppSettings,
) -> Result<String, String> {
    service::generate_sql_from_tables(prompt, schema, settings)
}

/// Failed SQL + Postgres error -> one corrected PostgreSQL statement.
pub fn fix_sql(
    sql: &str,
    error: &str,
    schema: &str,
    settings: &crate::storage::settings::AppSettings,
) -> Result<String, String> {
    service::fix_sql(sql, error, schema, settings)
}

/// EXPLAIN plan text -> short tuning advice.
pub fn interpret_plan(
    plan_text: &str,
    settings: &crate::storage::settings::AppSettings,
) -> Result<String, String> {
    service::interpret_plan(plan_text, settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_requires_api_key_for_remote_backend() {
        let settings = crate::storage::settings::AppSettings {
            ai_backend: "OpenAI".to_string(),
            ai_api_key: String::new(),
            ..Default::default()
        };

        let err = generate_sql("list users", "schema", &settings).expect_err("missing key");
        assert!(err.contains("OpenAI API key"));
    }

    #[test]
    fn local_backend_does_not_require_api_key() {
        let settings = crate::storage::settings::AppSettings {
            ai_backend: "Local".to_string(),
            ai_endpoint: "http://127.0.0.1:9/api/chat".to_string(),
            ai_api_key: String::new(),
            ..Default::default()
        };

        let err = generate_sql("list users", "schema", &settings)
            .expect_err("test endpoint should be unavailable");

        assert!(
            !err.contains("API key"),
            "local backend should attempt the configured local endpoint without an API key"
        );
    }
}
