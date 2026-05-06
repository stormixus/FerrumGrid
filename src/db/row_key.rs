//! Row identity model.
//!
//! Plan v7 Phase 1.2 / §10:
//! - **`RowKey`**: 16-byte truncated `blake3(table_oid_le || pk_canonical_concat)`.
//!   동일 트랜잭션 내 INSERT 후 그 PK 를 다시 UPDATE 대상으로 쓸 수 있도록
//!   `Eq + Hash` derive.
//! - **`RowKeyKind`**: 식별 출처 (`Pk` / `Tmp` / `Ctid`). Tmp 는 server-generated
//!   PK 회수 전 임시 식별, Ctid 는 PK 부재 테이블의 opt-in 모드 (Phase 1.3).
//! - **`is_pk_type_allowed`**: PK 컬럼 타입 화이트리스트 (Pre-mortem S1 차단).
//!
//! 본 단계 (Phase 1.2) 에서는 자료구조 + 정책 함수 + 단위 테스트만 도입.
//! UI (New Row / Delete Row 버튼, Tmp row 시각 표식) 와 RowKey::Tmp → ::Pk
//! 재매핑은 Phase 1.2 의 grid.rs 분기 추가 시 wire 된다.

use uuid::Uuid;

use crate::types::CellValue;

/// 16-byte truncated blake3 row identifier.
///
/// Phase 1.2 단계에서는 prod 호출 site 가 아직 없어 (RowKey 자료구조 + PK 정책 +
/// 단위 테스트 12개만 도입) `#[allow(dead_code)]` 로 의도적 미사용을 명시한다.
/// Phase 1.95 의 grid state surface 통합 시 attribute 는 자연스레 제거된다.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowKey([u8; 16]);

#[allow(dead_code)]
impl RowKey {
    /// `blake3(table_oid_le || pk_canonical_concat).truncate(16)` 로 생성.
    ///
    /// PK canonical concat 은 각 PK 컬럼의 `CellValue::canonical_string()` 결과를
    /// `\x1f` (Unit Separator) 로 join — 정규화된 wire-form 비교 가능.
    ///
    /// `pk_col_types` 는 `pk_values` 와 동일 길이의 슬라이스로, 각 컬럼의
    /// PostgreSQL 타입명 (소문자 권장, 대소문자 무관) 을 담는다. 슬라이스가
    /// 짧거나 비어 있으면 해당 인덱스는 타입 미지정으로 처리된다.
    ///
    /// **citext 정규화**: `citext` 타입 컬럼의 `Text` 값은 lowercase 로 변환한
    /// 뒤 hash 에 투입한다. PostgreSQL citext 는 DB 레벨 case-insensitive 비교를
    /// 사용하므로 `'Hello'` 와 `'hello'` 는 같은 행 — RowKey 도 동일해야 한다.
    pub fn from_pk(table_oid: u32, pk_values: &[CellValue], pk_col_types: &[&str]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&table_oid.to_le_bytes());
        for (idx, value) in pk_values.iter().enumerate() {
            if idx > 0 {
                hasher.update(&[0x1f]);
            }
            let is_citext = pk_col_types
                .get(idx)
                .map(|t| t.eq_ignore_ascii_case("citext"))
                .unwrap_or(false);
            if is_citext {
                if let CellValue::Text(s) = value {
                    hasher.update(s.to_lowercase().as_bytes());
                    continue;
                }
            }
            hasher.update(value.canonical_string().as_bytes());
        }
        let full = hasher.finalize();
        let mut truncated = [0u8; 16];
        truncated.copy_from_slice(&full.as_bytes()[..16]);
        Self(truncated)
    }

}

/// Row 식별의 출처.
///
/// - `Pk`: 기존 행 (DB 에서 RETURNING 또는 SELECT 로 회수된 진짜 PK).
/// - `Tmp`: 신규 INSERT 행, 서버 생성 PK 도착 전 임시 식별. Phase 1.2 의 New
///   Row 액션이 emit. INSERT 응답 도착 시 `MutationOutcome::inserted_keys` 로
///   `RowKey::Tmp` → `RowKey::Pk` 승격.
/// - `Ctid`: PK 부재 테이블의 opt-in 모드 (Phase 1.3, `ferrumgrid.unsafe_ctid`).
///   `RETURNING ctid` 강제 + `affected != 1` 즉시 ROLLBACK.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RowKeyKind {
    Pk(RowKey),
    Tmp(Uuid),
    Ctid(String),
}

/// PK 컬럼 타입 화이트리스트 (Pre-mortem S1 차단).
///
/// Plan v7 §10 — 비허용 타입은 silent 데이터 손상 가능성 (특히 array, range,
/// composite 의 hash 비결정성, json 의 형식 변동성) 때문에 mutation UI 를
/// hard-disable 한다.
///
/// 허용:
/// - `int2 / int4 / int8`, `numeric`, 그 별칭
/// - `real / float4 / float8` — Plan §10 비명시 확장. 부동소수점 PK 는 PG 권장
///   사항이 아니지만, `canonical_string()` 의 NaN/±0 정규화로 hash 결정성은 보장
///   (단위 테스트 `row_key_uses_canonical_string_for_nan_safety`,
///   `row_key_collapses_signed_zero` 검증)
/// - `uuid`
/// - `text / varchar / char / citext` — citext 는 DB 레벨 case-insensitive 비교.
///   `from_pk` 에 `pk_col_types` 를 전달하면 citext 컬럼 값을 lowercase 로 정규화한
///   뒤 hash 에 투입하므로 `'Alice'` 와 `'alice'` 는 동일 RowKey 를 생성한다.
/// - `date / timestamp / timestamptz / time / timetz`
/// - `bytea`
/// - `bool / boolean` — Plan §10 비명시 확장. PK 로 실용성 낮으나 hash 결정성은
///   보장되므로 안전
///
/// 비허용:
/// - `json / jsonb` (key 정렬 / whitespace 변동성)
/// - `*[]` array (고정 hash 어려움)
/// - composite, range, custom enum (type-specific 처리 필요)
pub fn is_pk_type_allowed(type_name: &str) -> bool {
    let lower = type_name.to_ascii_lowercase();
    let trimmed = lower.trim();

    // Array postfix `[]` 또는 array typname (`_int4` 등) 거부.
    if trimmed.ends_with("[]") || trimmed.starts_with('_') {
        return false;
    }

    matches!(
        trimmed,
        "smallint"
            | "integer"
            | "bigint"
            | "int"
            | "int2"
            | "int4"
            | "int8"
            | "numeric"
            | "decimal"
            | "real"
            | "double precision"
            | "float4"
            | "float8"
            | "uuid"
            | "text"
            | "varchar"
            | "character varying"
            | "char"
            | "character"
            | "bpchar"
            | "name"
            | "citext"
            | "date"
            | "timestamp"
            | "timestamp without time zone"
            | "timestamptz"
            | "timestamp with time zone"
            | "time"
            | "time without time zone"
            | "timetz"
            | "time with time zone"
            | "bytea"
            | "bool"
            | "boolean"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_key_is_deterministic_for_same_inputs() {
        let a = RowKey::from_pk(16384, &[CellValue::Int(7)], &[]);
        let b = RowKey::from_pk(16384, &[CellValue::Int(7)], &[]);
        assert_eq!(a, b);
    }

    #[test]
    fn row_key_differs_when_table_oid_differs() {
        let a = RowKey::from_pk(16384, &[CellValue::Int(7)], &[]);
        let b = RowKey::from_pk(16385, &[CellValue::Int(7)], &[]);
        assert_ne!(a, b);
    }

    #[test]
    fn row_key_differs_when_pk_value_differs() {
        let a = RowKey::from_pk(1, &[CellValue::Int(1)], &[]);
        let b = RowKey::from_pk(1, &[CellValue::Int(2)], &[]);
        assert_ne!(a, b);
    }

    #[test]
    fn row_key_composite_pk_order_matters() {
        let a = RowKey::from_pk(1, &[CellValue::Int(1), CellValue::Int(2)], &[]);
        let b = RowKey::from_pk(1, &[CellValue::Int(2), CellValue::Int(1)], &[]);
        assert_ne!(a, b);
    }

    #[test]
    fn row_key_uses_canonical_string_for_nan_safety() {
        let a = RowKey::from_pk(1, &[CellValue::Float(f64::NAN)], &[]);
        let b = RowKey::from_pk(1, &[CellValue::Float(f64::NAN)], &[]);
        assert_eq!(a, b, "NaN/NaN 은 canonical_string 경유로 동일 RowKey");
    }

    #[test]
    fn row_key_collapses_signed_zero() {
        let a = RowKey::from_pk(1, &[CellValue::Float(0.0)], &[]);
        let b = RowKey::from_pk(1, &[CellValue::Float(-0.0)], &[]);
        assert_eq!(a, b, "+0/-0 은 canonical_string 으로 동일 표현");
    }

    #[test]
    fn row_key_text_uses_separator_to_avoid_collision() {
        // ("ab", "c") 와 ("a", "bc") 가 separator 없으면 같은 hash 입력이 됨.
        let a = RowKey::from_pk(
            1,
            &[
                CellValue::Text("ab".to_string()),
                CellValue::Text("c".to_string()),
            ],
            &[],
        );
        let b = RowKey::from_pk(
            1,
            &[
                CellValue::Text("a".to_string()),
                CellValue::Text("bc".to_string()),
            ],
            &[],
        );
        assert_ne!(a, b, "0x1F separator 가 collision 방지");
    }

    #[test]
    fn row_key_citext_pk_is_case_insensitive() {
        // citext 타입에서 'Hello' 와 'hello' 는 DB 레벨에서 동일 행.
        // RowKey 도 lowercase 정규화로 동일해야 한다.
        let upper = RowKey::from_pk(
            1,
            &[CellValue::Text("Hello".to_string())],
            &["citext"],
        );
        let lower = RowKey::from_pk(
            1,
            &[CellValue::Text("hello".to_string())],
            &["citext"],
        );
        assert_eq!(upper, lower, "citext PK 는 case 무관하게 동일 RowKey");
    }

    #[test]
    fn row_key_citext_type_name_is_case_insensitive() {
        // 타입명 자체도 대소문자 무관하게 citext 로 인식.
        let a = RowKey::from_pk(
            1,
            &[CellValue::Text("Alice".to_string())],
            &["CITEXT"],
        );
        let b = RowKey::from_pk(
            1,
            &[CellValue::Text("alice".to_string())],
            &["citext"],
        );
        assert_eq!(a, b, "타입명 CITEXT/citext 모두 동일 정규화");
    }

    #[test]
    fn row_key_regular_text_pk_is_case_sensitive() {
        // 일반 text PK 는 case-sensitive — 'A' 와 'a' 는 다른 행.
        let upper = RowKey::from_pk(
            1,
            &[CellValue::Text("A".to_string())],
            &["text"],
        );
        let lower = RowKey::from_pk(
            1,
            &[CellValue::Text("a".to_string())],
            &["text"],
        );
        assert_ne!(upper, lower, "text PK 는 case-sensitive");
    }

    // ---------- is_pk_type_allowed ----------

    #[test]
    fn pk_whitelist_accepts_common_integer_types() {
        for ty in ["int2", "int4", "int8", "smallint", "integer", "bigint"] {
            assert!(is_pk_type_allowed(ty), "{ty} 는 허용 PK 타입");
        }
    }

    #[test]
    fn pk_whitelist_accepts_text_and_varchar_and_citext() {
        for ty in ["text", "varchar", "character varying", "citext", "char"] {
            assert!(is_pk_type_allowed(ty), "{ty} 는 허용 PK 타입");
        }
    }

    #[test]
    fn pk_whitelist_accepts_uuid_and_temporal() {
        for ty in [
            "uuid",
            "date",
            "timestamp",
            "timestamptz",
            "timestamp with time zone",
        ] {
            assert!(is_pk_type_allowed(ty), "{ty} 는 허용 PK 타입");
        }
    }

    #[test]
    fn pk_whitelist_rejects_json_array_range_composite() {
        for ty in [
            "json",
            "jsonb",
            "int4[]",
            "_int4",
            "tsrange",
            "int4range",
            "user_composite",
        ] {
            assert!(!is_pk_type_allowed(ty), "{ty} 는 비허용 PK 타입이어야 함");
        }
    }

    #[test]
    fn pk_whitelist_is_case_insensitive() {
        assert!(is_pk_type_allowed("UUID"));
        assert!(is_pk_type_allowed("VarChar"));
        assert!(!is_pk_type_allowed("JSON"));
    }
}
