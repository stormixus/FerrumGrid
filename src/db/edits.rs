//! Row-level edit operations.
//!
//! Plan v7 Phase 1.1 / ADR-5 — `RowEditOp` 가 in-memory 분리 표현
//! (`HashMap<updates>` + `Vec<inserts>` + `HashSet<deletes>`) 의 wire
//! 정규화 결과. Apply 시점에 insert → updates → deletes 순서로 정렬되어
//! 단일 트랜잭션 안에서 순차 실행된다.
//!
//! 본 단계 (Phase 1.1) 에서는 enum + SQL 생성 헬퍼 + 단위 테스트만 도입.
//! UI (New Row / Delete Row 버튼) 와 RowKey Tmp→Pk 재매핑은 Phase 1.2 에서
//! 추가한다. 본 phase 의 호출 site (`grid.rs::build_data_edits`) 는 현재로서는
//! `RowEditOp::Update` 만 생성한다.

use uuid::Uuid;

use crate::types::CellValue;

/// 단일 PK 컬럼의 식별 값.
#[derive(Debug, Clone)]
pub struct PkColumn {
    pub column: String,
    pub column_type: String,
    pub value: CellValue,
}

/// SET / VALUES 에 들어갈 컬럼 할당 값.
#[derive(Debug, Clone)]
pub enum EditValue {
    Null,
    Text(String),
}

/// 단일 컬럼 할당 (`column = value`).
#[derive(Debug, Clone)]
pub struct ColumnAssignment {
    pub column: String,
    pub column_type: String,
    pub value: EditValue,
}

/// 행 단위 편집 작업.
///
/// in-memory 표현 (`HashMap<(RowKey, ColIdx), SqlValue>` + `Vec<TmpRow>` +
/// `HashSet<RowKey>`) 의 wire 정규화 결과. Apply 시점에 insert → updates →
/// deletes 순서로 정렬되어 단일 트랜잭션 안에서 순차 실행된다.
///
/// Phase 1.1 단계에서는 `apply_data_edits` 가 모든 3 variant 를 처리하지만
/// UI 호출 site (`grid::build_data_edits`) 는 `Update` 만 생성한다. `Insert` /
/// `Delete` variant 는 Phase 1.2 의 New Row / Delete Row 액션이 활성화하면서
/// 자연스레 사용되므로 `#[allow(dead_code)]` 로 의도적 미사용을 명시.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RowEditOp {
    /// 신규 행 삽입. `RETURNING <pk>` 로 server-generated PK 회수 후
    /// `MutationOutcome::inserted_keys` 로 노출.
    Insert {
        tmp_id: Uuid,
        schema: String,
        table: String,
        columns: Vec<ColumnAssignment>,
        returning_pk: Vec<String>,
    },
    /// 기존 행의 단일 컬럼 갱신.
    Update {
        schema: String,
        table: String,
        column: ColumnAssignment,
        pk: Vec<PkColumn>,
    },
    /// 기존 행 삭제.
    Delete {
        schema: String,
        table: String,
        pk: Vec<PkColumn>,
    },
}

/// `apply_data_edits` 의 결과.
#[derive(Debug, Clone, Default)]
pub struct MutationOutcome {
    /// 트랜잭션 내에서 실제 적용된 op 수.
    pub applied: usize,
    /// INSERT 의 `RETURNING <pk>` 결과. `(tmp_id, [pk values])` 의 vec.
    /// 동일 트랜잭션 내 INSERT 후 그 PK 를 UPDATE 대상으로 재매핑할 때
    /// 사용된다 (Phase 1.2 RowKey::Tmp → ::Pk 승격).
    pub inserted_keys: Vec<(Uuid, Vec<CellValue>)>,
}

/// SQL identifier (table / column name) 안전 따옴표 처리.
fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

/// SQL string literal 안전 따옴표 처리.
fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// PG numeric 타입 판정 (literal 직접 삽입 가능 여부).
fn is_numeric_type(type_name: &str) -> bool {
    matches!(
        type_name.to_ascii_lowercase().as_str(),
        "smallint"
            | "integer"
            | "bigint"
            | "int2"
            | "int4"
            | "int8"
            | "real"
            | "double precision"
            | "float4"
            | "float8"
            | "numeric"
            | "decimal"
    )
}

/// PG bool 타입 판정.
fn is_bool_type(type_name: &str) -> bool {
    matches!(type_name.to_ascii_lowercase().as_str(), "boolean" | "bool")
}

/// 텍스트 입력값을 PG SQL literal 로 정규화.
fn sql_literal(value: &str, type_name: &str) -> String {
    let lower = type_name.to_ascii_lowercase();
    if is_numeric_type(&lower) && value.trim().parse::<f64>().is_ok() {
        return value.trim().to_string();
    }
    if is_bool_type(&lower) {
        return normalize_bool_literal(value)
            .map(|v| v.to_string())
            .unwrap_or_else(|| quote_literal(value));
    }
    quote_literal(value)
}

fn normalize_bool_literal(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `CellValue` (PK 값 등) 를 SQL literal 로 변환.
fn cell_to_sql_literal(value: &CellValue, type_name: &str) -> String {
    match value {
        CellValue::Null => "NULL".to_string(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Text(v) | CellValue::Timestamp(v) | CellValue::Unknown(v) => {
            sql_literal(v, type_name)
        }
        CellValue::Json(v) => sql_literal(&v.to_string(), type_name),
        CellValue::Uuid(v) => sql_literal(&v.to_string(), type_name),
        CellValue::Bytes(v) => sql_literal(&format!("\\x{}", hex_encode(v)), type_name),
    }
}

/// `EditValue` 를 SQL literal 로 변환.
fn edit_to_sql_literal(value: &EditValue, type_name: &str) -> String {
    match value {
        EditValue::Null => "NULL".to_string(),
        EditValue::Text(v) => sql_literal(v, type_name),
    }
}

/// PK 컬럼 vec 를 `WHERE pk1 = v1 AND pk2 = v2` 형태로 직렬화.
fn build_where_pk(pk: &[PkColumn]) -> String {
    pk.iter()
        .map(|key| {
            format!(
                "{} = {}",
                quote_ident(&key.column),
                cell_to_sql_literal(&key.value, &key.column_type)
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// `INSERT INTO <schema>.<table> (...) VALUES (...) RETURNING <pk_cols>` 생성.
///
/// 빈 `columns` 입력은 빌더 단계에서 명시적 sentinel SQL 을 반환한다 — 호출처
/// (현 phase 의 `apply_single_op`) 가 미리 거른다고 가정하지 않고, 직접 호출
/// 경로가 추가되더라도 invalid SQL 이 DB 로 흘러가지 않도록 방어.
pub(crate) fn build_insert_sql(
    schema: &str,
    table: &str,
    columns: &[ColumnAssignment],
    returning_pk: &[String],
) -> String {
    if columns.is_empty() {
        return format!(
            "-- INSERT INTO {}.{} requires at least one column",
            quote_ident(schema),
            quote_ident(table)
        );
    }
    let column_list = columns
        .iter()
        .map(|c| quote_ident(&c.column))
        .collect::<Vec<_>>()
        .join(", ");
    let value_list = columns
        .iter()
        .map(|c| edit_to_sql_literal(&c.value, &c.column_type))
        .collect::<Vec<_>>()
        .join(", ");
    let returning = if returning_pk.is_empty() {
        String::new()
    } else {
        format!(
            " RETURNING {}",
            returning_pk
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "INSERT INTO {}.{} ({}) VALUES ({}){}",
        quote_ident(schema),
        quote_ident(table),
        column_list,
        value_list,
        returning
    )
}

/// `UPDATE <schema>.<table> SET col = val WHERE pk = ...` 생성.
pub(crate) fn build_update_sql(
    schema: &str,
    table: &str,
    column: &ColumnAssignment,
    pk: &[PkColumn],
) -> String {
    format!(
        "UPDATE {}.{} SET {} = {} WHERE {}",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(&column.column),
        edit_to_sql_literal(&column.value, &column.column_type),
        build_where_pk(pk)
    )
}

/// `DELETE FROM <schema>.<table> WHERE pk = ...` 생성.
pub(crate) fn build_delete_sql(schema: &str, table: &str, pk: &[PkColumn]) -> String {
    format!(
        "DELETE FROM {}.{} WHERE {}",
        quote_ident(schema),
        quote_ident(table),
        build_where_pk(pk)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(col: &str, ty: &str, value: CellValue) -> PkColumn {
        PkColumn {
            column: col.to_string(),
            column_type: ty.to_string(),
            value,
        }
    }

    fn assign_text(col: &str, ty: &str, value: &str) -> ColumnAssignment {
        ColumnAssignment {
            column: col.to_string(),
            column_type: ty.to_string(),
            value: EditValue::Text(value.to_string()),
        }
    }

    fn assign_null(col: &str, ty: &str) -> ColumnAssignment {
        ColumnAssignment {
            column: col.to_string(),
            column_type: ty.to_string(),
            value: EditValue::Null,
        }
    }

    // ---------- build_update_sql ----------

    #[test]
    fn build_update_sql_text_value_quotes_literal() {
        let sql = build_update_sql(
            "public",
            "users",
            &assign_text("email", "text", "a@b.co"),
            &[pk("id", "int8", CellValue::Int(7))],
        );
        assert_eq!(
            sql,
            "UPDATE \"public\".\"users\" SET \"email\" = 'a@b.co' WHERE \"id\" = 7"
        );
    }

    #[test]
    fn build_update_sql_null_value_uses_keyword() {
        let sql = build_update_sql(
            "public",
            "users",
            &assign_null("nickname", "text"),
            &[pk("id", "int8", CellValue::Int(1))],
        );
        assert_eq!(
            sql,
            "UPDATE \"public\".\"users\" SET \"nickname\" = NULL WHERE \"id\" = 1"
        );
    }

    #[test]
    fn build_update_sql_numeric_value_unquoted() {
        let sql = build_update_sql(
            "public",
            "items",
            &assign_text("price", "numeric", "42.50"),
            &[pk("id", "int4", CellValue::Int(3))],
        );
        assert_eq!(
            sql,
            "UPDATE \"public\".\"items\" SET \"price\" = 42.50 WHERE \"id\" = 3"
        );
    }

    #[test]
    fn build_update_sql_composite_pk_joins_with_and() {
        let sql = build_update_sql(
            "shop",
            "orders",
            &assign_text("status", "text", "shipped"),
            &[
                pk("tenant_id", "int4", CellValue::Int(1)),
                pk("order_id", "uuid", CellValue::Uuid(uuid::Uuid::nil())),
            ],
        );
        assert_eq!(
            sql,
            "UPDATE \"shop\".\"orders\" SET \"status\" = 'shipped' \
             WHERE \"tenant_id\" = 1 AND \"order_id\" = '00000000-0000-0000-0000-000000000000'"
        );
    }

    #[test]
    fn build_update_sql_escapes_single_quote_in_text() {
        let sql = build_update_sql(
            "public",
            "users",
            &assign_text("note", "text", "it's fine"),
            &[pk("id", "int8", CellValue::Int(1))],
        );
        assert!(
            sql.contains("'it''s fine'"),
            "single quote must be escaped: {sql}"
        );
    }

    // ---------- build_insert_sql ----------

    #[test]
    fn build_insert_sql_basic_with_returning() {
        let sql = build_insert_sql(
            "public",
            "users",
            &[
                assign_text("email", "text", "a@b.co"),
                assign_text("age", "int4", "30"),
            ],
            &["id".to_string()],
        );
        assert_eq!(
            sql,
            "INSERT INTO \"public\".\"users\" (\"email\", \"age\") \
             VALUES ('a@b.co', 30) RETURNING \"id\""
        );
    }

    #[test]
    fn build_insert_sql_null_value() {
        let sql = build_insert_sql(
            "public",
            "users",
            &[
                assign_text("email", "text", "x@y.co"),
                assign_null("nickname", "text"),
            ],
            &["id".to_string()],
        );
        assert_eq!(
            sql,
            "INSERT INTO \"public\".\"users\" (\"email\", \"nickname\") \
             VALUES ('x@y.co', NULL) RETURNING \"id\""
        );
    }

    #[test]
    fn build_insert_sql_no_returning_when_pk_empty() {
        let sql = build_insert_sql(
            "audit",
            "events",
            &[assign_text("payload", "text", "x")],
            &[],
        );
        assert_eq!(
            sql,
            "INSERT INTO \"audit\".\"events\" (\"payload\") VALUES ('x')"
        );
    }

    #[test]
    fn build_insert_sql_composite_returning() {
        let sql = build_insert_sql(
            "shop",
            "orders",
            &[assign_text("status", "text", "new")],
            &["tenant_id".to_string(), "order_id".to_string()],
        );
        assert_eq!(
            sql,
            "INSERT INTO \"shop\".\"orders\" (\"status\") VALUES ('new') \
             RETURNING \"tenant_id\", \"order_id\""
        );
    }

    #[test]
    fn build_insert_sql_empty_columns_returns_sentinel_comment() {
        let sql = build_insert_sql("public", "users", &[], &["id".to_string()]);
        assert!(
            sql.starts_with("-- INSERT INTO"),
            "empty columns must return sentinel comment, got: {sql}"
        );
        assert!(
            !sql.contains("VALUES ()"),
            "empty columns must not produce invalid SQL, got: {sql}"
        );
    }

    #[test]
    fn build_insert_sql_bool_value_normalized() {
        let sql = build_insert_sql(
            "public",
            "flags",
            &[assign_text("enabled", "boolean", "yes")],
            &["id".to_string()],
        );
        assert_eq!(
            sql,
            "INSERT INTO \"public\".\"flags\" (\"enabled\") VALUES (true) \
             RETURNING \"id\""
        );
    }

    // ---------- build_delete_sql ----------

    #[test]
    fn build_delete_sql_single_pk() {
        let sql = build_delete_sql(
            "public",
            "users",
            &[pk("id", "int8", CellValue::Int(42))],
        );
        assert_eq!(
            sql,
            "DELETE FROM \"public\".\"users\" WHERE \"id\" = 42"
        );
    }

    #[test]
    fn build_delete_sql_composite_pk() {
        let sql = build_delete_sql(
            "shop",
            "orders",
            &[
                pk("tenant_id", "int4", CellValue::Int(1)),
                pk("order_id", "uuid", CellValue::Uuid(uuid::Uuid::nil())),
            ],
        );
        assert_eq!(
            sql,
            "DELETE FROM \"shop\".\"orders\" \
             WHERE \"tenant_id\" = 1 AND \"order_id\" = '00000000-0000-0000-0000-000000000000'"
        );
    }

    #[test]
    fn build_delete_sql_text_pk_quotes_literal() {
        let sql = build_delete_sql(
            "public",
            "users",
            &[pk("email", "text", CellValue::Text("a@b.co".to_string()))],
        );
        assert_eq!(
            sql,
            "DELETE FROM \"public\".\"users\" WHERE \"email\" = 'a@b.co'"
        );
    }

    #[test]
    fn build_delete_sql_escapes_single_quote_in_pk() {
        let sql = build_delete_sql(
            "public",
            "users",
            &[pk("name", "text", CellValue::Text("o'brien".to_string()))],
        );
        assert!(sql.contains("'o''brien'"), "must escape: {sql}");
    }

    // ---------- quote_ident edge cases ----------

    #[test]
    fn quote_ident_escapes_embedded_quote() {
        assert_eq!(quote_ident("a\"b"), "\"a\"\"b\"");
    }
}
