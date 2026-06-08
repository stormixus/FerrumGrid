//! 객체 단위 권한(ACL) 조회 + GRANT/REVOKE DDL 생성.
//! 기존 롤 뷰는 전역 롤 속성만 보여주므로 per-object 권한을 보완.

use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::types::ConnectionId;

#[derive(Debug, Clone)]
pub struct GrantRow {
    pub object_type: String,
    pub schema: String,
    pub table: String,
    pub grantee: String,
    pub privilege: String,
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub fn list_grants_sql() -> &'static str {
    "SELECT object_type, object_schema, object_name, grantee, privilege_type \
     FROM ( \
       SELECT 'table' AS object_type, table_schema AS object_schema, table_name AS object_name, grantee, privilege_type \
       FROM information_schema.role_table_grants \
       WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
       UNION ALL \
       SELECT 'sequence' AS object_type, n.nspname AS object_schema, c.relname AS object_name, \
              COALESCE(r.rolname, 'PUBLIC') AS grantee, acl.privilege_type::text AS privilege_type \
       FROM pg_catalog.pg_class c \
       JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace \
       CROSS JOIN LATERAL pg_catalog.aclexplode(c.relacl) AS acl \
       LEFT JOIN pg_catalog.pg_roles r ON r.oid = acl.grantee \
       WHERE c.relkind = 'S' \
         AND n.nspname NOT IN ('pg_catalog', 'information_schema') \
       UNION ALL \
       SELECT 'function' AS object_type, n.nspname AS object_schema, \
              p.proname || '(' || pg_catalog.pg_get_function_identity_arguments(p.oid) || ')' AS object_name, \
              COALESCE(r.rolname, 'PUBLIC') AS grantee, acl.privilege_type::text AS privilege_type \
       FROM pg_catalog.pg_proc p \
       JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace \
       CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) AS acl \
       LEFT JOIN pg_catalog.pg_roles r ON r.oid = acl.grantee \
       WHERE p.prokind IN ('f', 'a', 'w') \
         AND n.nspname NOT IN ('pg_catalog', 'information_schema') \
     ) grants \
     ORDER BY object_schema, object_name, grantee, privilege_type, object_type"
}

/// user 스키마의 객체 권한 부여 현황.
pub async fn list_grants(client: &Client, conn_id: ConnectionId) -> Result<Vec<GrantRow>, DbError> {
    let rows = client
        .query(list_grants_sql(), &[])
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;
    Ok(rows
        .iter()
        .map(|r| GrantRow {
            object_type: r.get(0),
            schema: r.get(1),
            table: r.get(2),
            grantee: r.get(3),
            privilege: r.get(4),
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
            build_grant_sql(
                true,
                "SELECT",
                GrantObject::Table,
                "public",
                "users",
                "app_ro"
            ),
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
            build_grant_sql(
                true,
                "USAGE",
                GrantObject::Sequence,
                "public",
                "id_seq",
                "app"
            ),
            "GRANT USAGE ON SEQUENCE \"public\".\"id_seq\" TO \"app\";"
        );
        assert_eq!(
            build_grant_sql(true, "EXECUTE", GrantObject::AllFunctions, "api", "", "app"),
            "GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA \"api\" TO \"app\";"
        );
    }

    #[test]
    fn grants_list_query_covers_supported_object_types() {
        let sql = list_grants_sql();

        assert!(sql.contains("role_table_grants"));
        assert!(sql.contains("'table'"));
        assert!(sql.contains("'sequence'"));
        assert!(sql.contains("'function'"));
        assert!(sql.contains("pg_catalog.pg_class"));
        assert!(sql.contains("pg_catalog.pg_proc"));
    }
}
