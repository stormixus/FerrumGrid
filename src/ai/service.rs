use crate::storage::settings::AppSettings;

use super::schema::TableContext;

pub(crate) fn build_prompt_from_tables(
    natural_language: &str,
    schema: &[TableContext],
    send_schema: bool,
) -> String {
    let mut prompt = String::from(
        "You are a PostgreSQL expert. Convert the user's request into a single SQL statement.\n\
         Return ONLY the SQL — no markdown fences, no explanation.\n\
         Use standard PostgreSQL syntax. Qualify tables with schema when known.\n\n",
    );

    if send_schema && !schema.is_empty() {
        prompt.push_str("Database schema:\n");
        for table in schema.iter().take(80) {
            if table.columns.is_empty() {
                prompt.push_str(&format!("- {}.{}\n", table.schema, table.table));
            } else {
                let cols = table
                    .columns
                    .iter()
                    .map(|(name, ty)| format!("{name} {ty}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                prompt.push_str(&format!("- {}.{} ({cols})\n", table.schema, table.table));
            }
        }
        if schema.len() > 80 {
            prompt.push_str(&format!(
                "... and {} more tables omitted\n",
                schema.len() - 80
            ));
        }
        prompt.push('\n');
    }

    prompt.push_str("User request:\n");
    prompt.push_str(natural_language.trim());
    prompt.push_str("\n\nSQL:");
    prompt
}

pub(crate) fn build_prompt_from_text_schema(
    natural_language: &str,
    schema: &str,
    send_schema: bool,
) -> String {
    let mut prompt = String::from(
        "You are a PostgreSQL expert. Convert the user's request into a single SQL statement.\n\
         Return ONLY the SQL — no markdown fences, no explanation.\n\
         Use standard PostgreSQL syntax. Qualify tables with schema when known.\n\n",
    );

    if send_schema && !schema.trim().is_empty() {
        prompt.push_str("Database schema:\n");
        prompt.push_str(schema.trim());
        prompt.push_str("\n\n");
    }

    prompt.push_str("User request:\n");
    prompt.push_str(natural_language.trim());
    prompt.push_str("\n\nSQL:");
    prompt
}

fn strip_fences(sql: &str) -> String {
    let trimmed = sql.trim();
    if trimmed.starts_with("```") {
        let inner = trimmed
            .trim_start_matches('`')
            .trim_start_matches("sql")
            .trim_start_matches("SQL")
            .trim();
        if let Some(end) = inner.rfind("```") {
            return inner[..end].trim().to_string();
        }
        return inner.to_string();
    }
    trimmed.to_string()
}

fn endpoint_for_backend(backend: &str, custom: &str) -> String {
    let custom = custom.trim();
    if !custom.is_empty() {
        return custom.to_string();
    }
    match backend {
        "OpenAI" => "https://api.openai.com/v1/chat/completions".to_string(),
        "Anthropic" => "https://api.anthropic.com/v1/messages".to_string(),
        _ => "http://localhost:11434/api/chat".to_string(),
    }
}

fn api_key_for_backend(backend: &str, configured: &str) -> Option<String> {
    let configured = configured.trim();
    if !configured.is_empty() {
        return Some(configured.to_string());
    }
    match backend {
        "OpenAI" => std::env::var("OPENAI_API_KEY").ok(),
        "Anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn backend_requires_api_key(backend: &str) -> bool {
    matches!(backend, "OpenAI" | "Anthropic")
}

fn call_openai(endpoint: &str, api_key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "You output PostgreSQL SQL only."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1
    });

    let resp = ureq::post(endpoint)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("OpenAI request failed: {e}"))?;

    let value: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("OpenAI response parse failed: {e}"))?;

    value["choices"][0]["message"]["content"]
        .as_str()
        .map(strip_fences)
        .ok_or_else(|| "OpenAI response missing content".to_string())
}

fn call_anthropic(
    endpoint: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 2048,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let resp = ureq::post(endpoint)
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("Anthropic request failed: {e}"))?;

    let value: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Anthropic response parse failed: {e}"))?;

    value["content"][0]["text"]
        .as_str()
        .map(strip_fences)
        .ok_or_else(|| "Anthropic response missing content".to_string())
}

fn call_ollama(endpoint: &str, model: &str, prompt: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "stream": false
    });

    let resp = ureq::post(endpoint)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("Local LLM request failed: {e}"))?;

    let value: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Local LLM response parse failed: {e}"))?;

    if let Some(content) = value["message"]["content"].as_str() {
        return Ok(strip_fences(content));
    }
    if let Some(response) = value["response"].as_str() {
        return Ok(strip_fences(response));
    }
    Err("Local LLM response missing content".to_string())
}

fn default_model(settings: &AppSettings) -> String {
    if settings.ai_model.trim().is_empty() {
        "llama3.2".to_string()
    } else {
        settings.ai_model.clone()
    }
}

fn call_backend(settings: &AppSettings, prompt: &str) -> Result<String, String> {
    let endpoint = endpoint_for_backend(&settings.ai_backend, &settings.ai_endpoint);
    let model = default_model(settings);

    match settings.ai_backend.as_str() {
        "OpenAI" => {
            let key = api_key_for_backend("OpenAI", &settings.ai_api_key)
                .ok_or_else(|| "Set an OpenAI API key in Settings -> AI Assist.".to_string())?;
            call_openai(&endpoint, &key, &model, prompt)
        }
        "Anthropic" => {
            let key = api_key_for_backend("Anthropic", &settings.ai_api_key)
                .ok_or_else(|| "Set an Anthropic API key in Settings -> AI Assist.".to_string())?;
            call_anthropic(&endpoint, &key, &model, prompt)
        }
        _ => call_ollama(&endpoint, &model, prompt),
    }
}

/// Generate SQL from natural language using configured AI backend and table metadata.
pub(crate) fn generate_sql_from_tables(
    natural_language: &str,
    schema: &[TableContext],
    settings: &AppSettings,
) -> Result<String, String> {
    if natural_language.trim().is_empty() {
        return Err("Enter a description of the query you want.".to_string());
    }

    let prompt = build_prompt_from_tables(natural_language, schema, settings.ai_send_schema);
    call_backend(settings, &prompt)
}

/// Generate SQL from natural language using a pre-rendered schema string.
pub(crate) fn generate_sql_from_text_schema(
    natural_language: &str,
    schema: &str,
    settings: &AppSettings,
) -> Result<String, String> {
    if natural_language.trim().is_empty() {
        return Err("Enter a description of the query you want.".to_string());
    }

    let prompt = build_prompt_from_text_schema(natural_language, schema, settings.ai_send_schema);
    call_backend(settings, &prompt)
}

pub(crate) fn fix_sql(
    sql: &str,
    error: &str,
    schema: &str,
    settings: &AppSettings,
) -> Result<String, String> {
    if sql.trim().is_empty() {
        return Err("No SQL to fix.".to_string());
    }
    let mut prompt = String::from(
        "You are a PostgreSQL expert. The user's SQL failed. Return ONLY one corrected PostgreSQL statement.\n\
         No markdown fences, no prose.\n\n",
    );
    if settings.ai_send_schema && !schema.trim().is_empty() {
        prompt.push_str("Database schema:\n");
        prompt.push_str(schema.trim());
        prompt.push_str("\n\n");
    }
    prompt.push_str("Failed SQL:\n");
    prompt.push_str(sql.trim());
    prompt.push_str("\n\nPostgres error:\n");
    prompt.push_str(error.trim());
    prompt.push_str("\n\nSQL:");
    call_backend(settings, &prompt)
}

pub(crate) fn interpret_plan(plan_text: &str, settings: &AppSettings) -> Result<String, String> {
    if plan_text.trim().is_empty() {
        return Err("No EXPLAIN plan to interpret.".to_string());
    }
    let prompt = format!(
        "You are a PostgreSQL performance expert. Given this EXPLAIN plan, \
         give a short, concrete diagnosis with bottleneck nodes and specific index or rewrite suggestions. \
         Plain text, no markdown.\n\n{plan_text}"
    );
    call_backend(settings, &prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_removes_markdown() {
        assert_eq!(strip_fences("```sql\nSELECT 1;\n```"), "SELECT 1;");
    }

    #[test]
    fn build_prompt_includes_schema_when_enabled() {
        let schema = vec![TableContext {
            schema: "public".to_string(),
            table: "users".to_string(),
            columns: vec![("id".to_string(), "integer".to_string())],
        }];
        let prompt = build_prompt_from_tables("show users", &schema, true);
        assert!(prompt.contains("public.users"));
        assert!(prompt.contains("show users"));
    }

    #[test]
    fn local_backend_does_not_require_api_key() {
        assert!(!backend_requires_api_key("Local"));
        assert!(!backend_requires_api_key("Ollama"));
        assert!(backend_requires_api_key("OpenAI"));
        assert!(backend_requires_api_key("Anthropic"));
    }
}
