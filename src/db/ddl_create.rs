//! View / Function / Role 의 CREATE/REPLACE/ALTER SQL builder.
//!
//! Plan v7 Phase 2c — Designer 확장. UI 의 form 입력 → 안정한 DDL SQL 텍스트로
//! 변환. 본 helper 는 *문자열 생성만* — 실제 실행은 `db::ddl::execute_ddl_with_invalidation`
//! 경유.

/// View 정의.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewSpec {
    pub schema: String,
    pub name: String,
    /// `SELECT ...` body — caller 책임 (validation 은 PostgreSQL 에 위임).
    pub select_body: String,
    /// `true` 이면 CREATE OR REPLACE.
    pub or_replace: bool,
}

/// Function 정의.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    pub schema: String,
    pub name: String,
    /// 인자 시그니처 (예: `"a int, b text"`). 빈 문자열 허용.
    pub args: String,
    /// 반환 타입 (예: `"int"`, `"TABLE(id int, name text)"`).
    pub returns: String,
    /// 언어 (예: `"plpgsql"`, `"sql"`).
    pub language: String,
    /// 함수 본문. `$$ ... $$` quoting 은 builder 가 적용.
    pub body: String,
    /// `true` 이면 CREATE OR REPLACE.
    pub or_replace: bool,
}

/// Role 정의 (CREATE).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoleSpec {
    pub name: String,
    pub login: bool,
    pub superuser: bool,
    pub create_db: bool,
    pub create_role: bool,
    /// `Some(password)` → WITH PASSWORD '...'. None → 비밀번호 없이.
    pub password: Option<String>,
}

/// View 의 CREATE [OR REPLACE] SQL 생성.
///
/// 예시: `CREATE OR REPLACE VIEW "public"."v1" AS SELECT 1`
#[allow(dead_code)]
pub fn build_create_view_sql(spec: &ViewSpec) -> String {
    let head = if spec.or_replace {
        "CREATE OR REPLACE VIEW"
    } else {
        "CREATE VIEW"
    };
    format!(
        "{head} {schema}.{name} AS {body}",
        schema = quote_ident(&spec.schema),
        name = quote_ident(&spec.name),
        body = spec.select_body.trim()
    )
}

/// Function 의 CREATE [OR REPLACE] SQL 생성. body 는 dollar-quoted ($$ ... $$).
#[allow(dead_code)]
pub fn build_create_function_sql(spec: &FunctionSpec) -> String {
    let head = if spec.or_replace {
        "CREATE OR REPLACE FUNCTION"
    } else {
        "CREATE FUNCTION"
    };
    let dollar_tag = pick_dollar_tag(&spec.body);
    format!(
        "{head} {schema}.{name}({args}) RETURNS {ret} LANGUAGE {lang} AS {tag}{body}{tag}",
        schema = quote_ident(&spec.schema),
        name = quote_ident(&spec.name),
        args = spec.args.trim(),
        ret = spec.returns.trim(),
        lang = spec.language.trim(),
        tag = dollar_tag,
        body = spec.body.trim_end_matches('\n'),
    )
}

/// Role 의 CREATE ROLE SQL 생성.
///
/// PASSWORD 는 single-quote literal escaping (`'` → `''`) 만 적용.
/// PostgreSQL 의 `CREATE ROLE` 는 PASSWORD 를 평문으로 받음 (이후 PG 가 hash).
#[allow(dead_code)]
pub fn build_create_role_sql(spec: &RoleSpec) -> String {
    let mut parts = Vec::new();
    parts.push(if spec.login { "LOGIN" } else { "NOLOGIN" });
    if spec.superuser {
        parts.push("SUPERUSER");
    }
    if spec.create_db {
        parts.push("CREATEDB");
    }
    if spec.create_role {
        parts.push("CREATEROLE");
    }
    let opts = parts.join(" ");
    let pwd_clause = match &spec.password {
        Some(pw) => format!(" PASSWORD '{}'", escape_sql_literal(pw)),
        None => String::new(),
    };
    format!(
        "CREATE ROLE {name} WITH {opts}{pwd}",
        name = quote_ident(&spec.name),
        opts = opts,
        pwd = pwd_clause,
    )
}

/// `ALTER ROLE name WITH ...` SQL 생성. password 가 Some 이면 PASSWORD 갱신 포함.
#[allow(dead_code)]
pub fn build_alter_role_sql(spec: &RoleSpec) -> String {
    let mut parts = Vec::new();
    parts.push(if spec.login { "LOGIN" } else { "NOLOGIN" });
    parts.push(if spec.superuser { "SUPERUSER" } else { "NOSUPERUSER" });
    parts.push(if spec.create_db { "CREATEDB" } else { "NOCREATEDB" });
    parts.push(if spec.create_role { "CREATEROLE" } else { "NOCREATEROLE" });
    let opts = parts.join(" ");
    let pwd_clause = match &spec.password {
        Some(pw) => format!(" PASSWORD '{}'", escape_sql_literal(pw)),
        None => String::new(),
    };
    format!(
        "ALTER ROLE {name} WITH {opts}{pwd}",
        name = quote_ident(&spec.name),
        opts = opts,
        pwd = pwd_clause,
    )
}

/// PostgreSQL identifier double-quote escaping (`"` → `""`).
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// SQL string literal escaping — single quotes only.
fn escape_sql_literal(s: &str) -> String {
    s.replace('\'', "''")
}

/// Function body 가 `$$` 를 포함하면 `$_$`, 그것도 포함하면 `$_a$` ... 로 escalate.
fn pick_dollar_tag(body: &str) -> &'static str {
    const CANDIDATES: &[&str] = &["$$", "$_$", "$_a$", "$_b$", "$_c$"];
    for tag in CANDIDATES {
        if !body.contains(tag) {
            return tag;
        }
    }
    "$_z$"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_view_basic() {
        let spec = ViewSpec {
            schema: "public".to_string(),
            name: "v1".to_string(),
            select_body: "SELECT 1 AS x".to_string(),
            or_replace: false,
        };
        assert_eq!(
            build_create_view_sql(&spec),
            "CREATE VIEW \"public\".\"v1\" AS SELECT 1 AS x"
        );
    }

    #[test]
    fn create_view_or_replace() {
        let spec = ViewSpec {
            schema: "analytics".to_string(),
            name: "orders_summary".to_string(),
            select_body: "  SELECT id, sum(price) FROM orders GROUP BY id  ".to_string(),
            or_replace: true,
        };
        let sql = build_create_view_sql(&spec);
        assert!(sql.starts_with("CREATE OR REPLACE VIEW"));
        assert!(sql.contains("\"analytics\".\"orders_summary\""));
        assert!(sql.contains("SELECT id, sum(price)"));
    }

    #[test]
    fn quote_ident_escapes_double_quotes() {
        let spec = ViewSpec {
            schema: "we\"ird".to_string(),
            name: "v".to_string(),
            select_body: "SELECT 1".to_string(),
            or_replace: false,
        };
        let sql = build_create_view_sql(&spec);
        assert!(sql.contains("\"we\"\"ird\""));
    }

    #[test]
    fn create_function_basic_plpgsql() {
        let spec = FunctionSpec {
            schema: "public".to_string(),
            name: "add".to_string(),
            args: "a int, b int".to_string(),
            returns: "int".to_string(),
            language: "plpgsql".to_string(),
            body: "BEGIN RETURN a + b; END;".to_string(),
            or_replace: false,
        };
        let sql = build_create_function_sql(&spec);
        assert!(sql.starts_with("CREATE FUNCTION"));
        assert!(sql.contains("\"public\".\"add\"(a int, b int)"));
        assert!(sql.contains("RETURNS int"));
        assert!(sql.contains("LANGUAGE plpgsql"));
        assert!(sql.contains("$$BEGIN RETURN a + b; END;$$"));
    }

    #[test]
    fn create_function_or_replace_dollar_tag_escalates() {
        let spec = FunctionSpec {
            schema: "public".to_string(),
            name: "f".to_string(),
            args: "".to_string(),
            returns: "void".to_string(),
            language: "sql".to_string(),
            body: "SELECT $$inner$$".to_string(),
            or_replace: true,
        };
        let sql = build_create_function_sql(&spec);
        // body contains $$, so tag must escalate to $_$
        assert!(sql.contains("$_$SELECT $$inner$$$_$"));
        assert!(sql.starts_with("CREATE OR REPLACE FUNCTION"));
    }

    #[test]
    fn create_role_with_password_and_login() {
        let spec = RoleSpec {
            name: "alice".to_string(),
            login: true,
            superuser: false,
            create_db: false,
            create_role: false,
            password: Some("p@ss".to_string()),
        };
        let sql = build_create_role_sql(&spec);
        assert_eq!(sql, "CREATE ROLE \"alice\" WITH LOGIN PASSWORD 'p@ss'");
    }

    #[test]
    fn create_role_password_escapes_single_quote() {
        let spec = RoleSpec {
            name: "bob".to_string(),
            login: true,
            superuser: false,
            create_db: false,
            create_role: false,
            password: Some("o'brien".to_string()),
        };
        let sql = build_create_role_sql(&spec);
        assert!(sql.contains("PASSWORD 'o''brien'"));
    }

    #[test]
    fn create_role_no_login_no_password() {
        let spec = RoleSpec {
            name: "svc".to_string(),
            login: false,
            superuser: false,
            create_db: true,
            create_role: false,
            password: None,
        };
        let sql = build_create_role_sql(&spec);
        assert_eq!(sql, "CREATE ROLE \"svc\" WITH NOLOGIN CREATEDB");
        assert!(!sql.contains("PASSWORD"));
    }

    #[test]
    fn create_role_superuser_and_create_role_flags() {
        let spec = RoleSpec {
            name: "admin".to_string(),
            login: true,
            superuser: true,
            create_db: true,
            create_role: true,
            password: None,
        };
        let sql = build_create_role_sql(&spec);
        assert!(sql.contains("LOGIN"));
        assert!(sql.contains("SUPERUSER"));
        assert!(sql.contains("CREATEDB"));
        assert!(sql.contains("CREATEROLE"));
    }

    #[test]
    fn alter_role_includes_negative_flags() {
        let spec = RoleSpec {
            name: "alice".to_string(),
            login: false,
            superuser: false,
            create_db: false,
            create_role: false,
            password: None,
        };
        let sql = build_alter_role_sql(&spec);
        assert!(sql.contains("NOLOGIN"));
        assert!(sql.contains("NOSUPERUSER"));
        assert!(sql.contains("NOCREATEDB"));
        assert!(sql.contains("NOCREATEROLE"));
        assert!(sql.starts_with("ALTER ROLE \"alice\" WITH"));
    }

    #[test]
    fn alter_role_with_password_change() {
        let spec = RoleSpec {
            name: "alice".to_string(),
            login: true,
            superuser: false,
            create_db: false,
            create_role: false,
            password: Some("new-pw".to_string()),
        };
        let sql = build_alter_role_sql(&spec);
        assert!(sql.contains("PASSWORD 'new-pw'"));
    }

    #[test]
    fn dollar_tag_picks_double_dollar_when_safe() {
        assert_eq!(pick_dollar_tag("plain body"), "$$");
        assert_eq!(pick_dollar_tag("body with $$ inside"), "$_$");
        assert_eq!(pick_dollar_tag("body with $$ and $_$"), "$_a$");
    }
}
