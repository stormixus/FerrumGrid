//! 객체 단위 권한(ACL) 조회 + GRANT/REVOKE DDL 생성.
//! 기존 롤 뷰는 전역 롤 속성만 보여주므로 per-object 권한을 보완.

use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone)]
pub struct GrantRow {
    pub schema: String,
    pub table: String,
    pub grantee: String,
    pub privilege: String,
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// user 스키마의 테이블/뷰 권한 부여 현황 (information_schema.role_table_grants).
pub async fn list_grants(client: &Client, conn_id: ConnectionId) -> Result<Vec<GrantRow>, DbError> {
    let rows = client
        .query(
            "SELECT table_schema, table_name, grantee, privilege_type \
             FROM information_schema.role_table_grants \
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
             ORDER BY table_schema, table_name, grantee, privilege_type",
            &[],
        )
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    Ok(rows
        .iter()
        .map(|r| GrantRow {
            schema: r.get(0),
            table: r.get(1),
            grantee: r.get(2),
            privilege: r.get(3),
        })
        .collect())
}

/// GRANT/REVOKE 대상 객체 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantObject {
    Table,
    Sequence,
    /// 스키마 내 모든 함수 (`ON ALL FUNCTIONS IN SCHEMA`). `name` 무시.
    AllFunctions,
}

/// `GRANT/REVOKE <priv> ON <object> TO/FROM <grantee>` 생성.
/// - Table:        `ON "s"."n"`
/// - Sequence:     `ON SEQUENCE "s"."n"`
/// - AllFunctions: `ON ALL FUNCTIONS IN SCHEMA "s"` (name 무시)
pub fn build_grant_sql(
    grant: bool,
    privilege: &str,
    object: GrantObject,
    schema: &str,
    name: &str,
    grantee: &str,
) -> String {
    let (verb, dir) = if grant {
        ("GRANT", "TO")
    } else {
        ("REVOKE", "FROM")
    };
    // PUBLIC 은 식별자 인용 없이 키워드.
    let target = if grantee.eq_ignore_ascii_case("public") {
        "PUBLIC".to_string()
    } else {
        quote_ident(grantee)
    };
    let on = match object {
        GrantObject::Table => format!("{}.{}", quote_ident(schema), quote_ident(name)),
        GrantObject::Sequence => {
            format!("SEQUENCE {}.{}", quote_ident(schema), quote_ident(name))
        }
        GrantObject::AllFunctions => {
            format!("ALL FUNCTIONS IN SCHEMA {}", quote_ident(schema))
        }
    };
    format!("{verb} {privilege} ON {on} {dir} {target};")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_sql_quotes_identifiers() {
        assert_eq!(
            build_grant_sql(true, "SELECT", GrantObject::Table, "public", "users", "app_ro"),
            "GRANT SELECT ON \"public\".\"users\" TO \"app_ro\";"
        );
    }

    #[test]
    fn revoke_and_public_keyword() {
        assert_eq!(
            build_grant_sql(false, "ALL", GrantObject::Table, "s", "t", "PUBLIC"),
            "REVOKE ALL ON \"s\".\"t\" FROM PUBLIC;"
        );
    }

    #[test]
    fn sequence_and_function_objects() {
        assert_eq!(
            build_grant_sql(true, "USAGE", GrantObject::Sequence, "public", "id_seq", "app"),
            "GRANT USAGE ON SEQUENCE \"public\".\"id_seq\" TO \"app\";"
        );
        assert_eq!(
            build_grant_sql(true, "EXECUTE", GrantObject::AllFunctions, "api", "", "app"),
            "GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA \"api\" TO \"app\";"
        );
    }
}
