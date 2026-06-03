//! AI text-to-SQL 백엔드 (BYOK). OpenAI / Anthropic Messages API 를
//! 기존 `ureq` 의존성으로 호출한다 (추가 의존성 없음).
//!
//! 네트워크 호출은 blocking 이므로 호출자는 별도 스레드에서 실행하고
//! 결과를 `AiJob` 슬롯으로 전달한다.

use serde_json::Value;

/// 백그라운드 AI 작업 슬롯 (Arc<Mutex<…>> 로 UI 와 공유).
#[derive(Default)]
pub struct AiJob {
    pub running: bool,
    /// Ok(sql) 또는 Err(message). UI 가 매 프레임 take.
    pub result: Option<Result<String, String>>,
}

/// 자연어 요청 → 단일 PostgreSQL 문. 백엔드별 API 호출.
pub fn generate_sql(
    backend: &str,
    model: &str,
    api_key: &str,
    schema: &str,
    prompt: &str,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("No API key set (Settings → AI Assist)".to_string());
    }
    if prompt.trim().is_empty() {
        return Err("Empty prompt".to_string());
    }
    let system = build_system_prompt(schema);
    match backend {
        "OpenAI" => openai(model, api_key, &system, prompt),
        "Anthropic" => anthropic(model, api_key, &system, prompt),
        other => Err(format!("AI backend '{other}' is not supported yet")),
    }
}

fn build_system_prompt(schema: &str) -> String {
    let mut s = String::from(
        "You are a PostgreSQL expert. Output ONLY a single valid PostgreSQL SQL \
         statement answering the user's request. No prose, no explanation, no \
         markdown code fences.",
    );
    if !schema.trim().is_empty() {
        s.push_str("\n\nDatabase schema (table(column type, ...)):\n");
        s.push_str(schema);
    }
    s
}

fn openai(model: &str, key: &str, system: &str, user: &str) -> Result<String, String> {
    let resp = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {key}"))
        .send_json(serde_json::json!({
            "model": model,
            "temperature": 0,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        }))
        .map_err(stringify_ureq)?;
    let v: Value = resp.into_json().map_err(|e| e.to_string())?;
    let sql = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "OpenAI: no content in response".to_string())?;
    Ok(strip_fences(sql))
}

fn anthropic(model: &str, key: &str, system: &str, user: &str) -> Result<String, String> {
    let resp = ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", key)
        .set("anthropic-version", "2023-06-01")
        .send_json(serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "system": system,
            "messages": [{"role": "user", "content": user}],
        }))
        .map_err(stringify_ureq)?;
    let v: Value = resp.into_json().map_err(|e| e.to_string())?;
    let sql = v["content"][0]["text"]
        .as_str()
        .ok_or_else(|| "Anthropic: no content in response".to_string())?;
    Ok(strip_fences(sql))
}

/// ureq 의 4xx/5xx body 에 API 에러 메시지가 있으면 그것을 노출.
fn stringify_ureq(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            format!("HTTP {code}: {}", body.chars().take(300).collect::<String>())
        }
        ureq::Error::Transport(t) => format!("Network error: {t}"),
    }
}

/// ```sql … ``` 펜스 제거 + 트림.
fn strip_fences(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix("```sql").or_else(|| t.strip_prefix("```")).unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t);
    t.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_fences_removes_markdown() {
        assert_eq!(strip_fences("```sql\nSELECT 1;\n```"), "SELECT 1;");
        assert_eq!(strip_fences("```\nSELECT 2;\n```"), "SELECT 2;");
        assert_eq!(strip_fences("  SELECT 3;  "), "SELECT 3;");
    }

    #[test]
    fn generate_requires_api_key() {
        assert!(generate_sql("OpenAI", "gpt-4o", "", "schema", "list users").is_err());
    }

    #[test]
    fn system_prompt_includes_schema_when_present() {
        let p = build_system_prompt("public.users(id int)");
        assert!(p.contains("public.users(id int)"));
        assert!(!build_system_prompt("").contains("schema:\n\n"));
    }
}
