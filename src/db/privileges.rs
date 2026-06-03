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

/// `GRANT/REVOKE <priv> ON <schema>.<table> TO/FROM <grantee>` 생성.
pub fn build_grant_sql(
    grant: bool,
    privilege: &str,
    schema: &str,
    table: &str,
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
    format!(
        "{verb} {privilege} ON {}.{} {dir} {target};",
        quote_ident(schema),
        quote_ident(table),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_sql_quotes_identifiers() {
        assert_eq!(
            build_grant_sql(true, "SELECT", "public", "users", "app_ro"),
            "GRANT SELECT ON \"public\".\"users\" TO \"app_ro\";"
        );
    }

    #[test]
    fn revoke_and_public_keyword() {
        assert_eq!(
            build_grant_sql(false, "ALL", "s", "t", "PUBLIC"),
            "REVOKE ALL ON \"s\".\"t\" FROM PUBLIC;"
        );
    }
}
