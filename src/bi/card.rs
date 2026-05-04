//! BI 카드 status + cache key 모델.
//!
//! Plan v7 Phase 4c2 — `.omc/plans/bi-cache-scenarios.md` 의 결정에 따라
//! BI 카드는 Phase 2 의 `Invalidate` 인프라 를 *cache key* 차원에서 90% 재사용.
//! 추가로 카드별 *valid 상태* 만 별도 enum 으로 추적.

use crate::bi::aggregate::ColumnStats;
use crate::db::invalidate::Invalidate;

/// BI 카드 의 유효성 상태.
///
/// `Valid` — 카드의 SQL 이 referenced 모든 column / table 이 존재하여 정상 표시 가능.
/// `ColumnsRemoved` — 카드 SQL 의 일부 column 이 schema 변경으로 사라짐 (DROP COLUMN 등).
/// `TableDropped` — 카드 SQL 의 base table 자체가 사라짐 (DROP TABLE).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiCardStatus {
    Valid,
    ColumnsRemoved,
    TableDropped,
}

/// BI 카드 1개의 cache key + 마지막 측정값.
///
/// `sql_hash` 는 카드의 normalized SQL 의 안정 해시 (동일 SQL 의 카드 중복 검출 +
/// cache lookup key). `table_oid` 는 PostgreSQL 의 `pg_class.oid` — schema 변경
/// 이벤트 매칭 키. `last_data` 는 마지막 성공 fetch 의 column 통계 — re-fetch 실패
/// 시 fallback 표시 용.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BiCard {
    pub id: String,
    pub title: String,
    pub sql_hash: u64,
    pub table_oid: u32,
    pub status: BiCardStatus,
    pub last_data: Vec<ColumnStats>,
}

#[allow(dead_code)]
impl BiCard {
    /// 신규 카드를 `Valid` 상태로 생성.
    pub fn new(id: impl Into<String>, title: impl Into<String>, sql_hash: u64, table_oid: u32) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            sql_hash,
            table_oid,
            status: BiCardStatus::Valid,
            last_data: Vec::new(),
        }
    }
}

/// schema 변경 이벤트가 카드의 status 에 미치는 영향 평가.
///
/// 결정 규칙 (`.omc/plans/bi-cache-scenarios.md` Scenario 3):
/// - `Invalidate::Schemas` / `Invalidate::Tables` 가 카드의 base table 을 cover
///   하면 → 카드는 *물리적으로* drop 가능 가정 → `TableDropped`.
/// - `Invalidate::Columns { schema, table }` 가 카드의 base table 을 정확히 매칭
///   하면 → 일부 column 이 사라졌을 가능성 → `ColumnsRemoved`.
/// - 그 외 invalidation 은 status 에 영향 없음 (현 status 유지).
///
/// `card_schema` 는 카드가 가리키는 schema (예: `"public"`), `card_table` 은
/// table 이름. (실제 production 에서는 table_oid → name 역참조가 필요하지만
/// 본 헬퍼는 caller 가 매핑을 책임진다.)
#[allow(dead_code)]
pub fn evaluate_card_status(
    card: &BiCard,
    invalidation: &Invalidate,
    card_schema: &str,
    card_table: &str,
) -> BiCardStatus {
    match invalidation {
        Invalidate::Schemas => BiCardStatus::TableDropped,
        Invalidate::Tables { schema } if schema == card_schema => BiCardStatus::TableDropped,
        Invalidate::Columns { schema, table }
            if schema == card_schema && table == card_table =>
        {
            BiCardStatus::ColumnsRemoved
        }
        _ => card.status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bi::aggregate::ColumnStats;

    fn make_card() -> BiCard {
        BiCard::new("card-1", "Orders Summary", 0xDEAD_BEEFu64, 16384)
    }

    #[test]
    fn new_card_starts_valid() {
        let card = make_card();
        assert_eq!(card.status, BiCardStatus::Valid);
        assert_eq!(card.id, "card-1");
        assert_eq!(card.sql_hash, 0xDEAD_BEEF);
        assert_eq!(card.table_oid, 16384);
        assert!(card.last_data.is_empty());
    }

    #[test]
    fn last_data_can_hold_column_stats() {
        let mut card = make_card();
        card.last_data.push(ColumnStats {
            name: "price".to_string(),
            type_name: "int4".to_string(),
            non_null: 10,
            min: Some(1.0),
            max: Some(100.0),
            avg: Some(50.0),
        });
        assert_eq!(card.last_data.len(), 1);
        assert_eq!(card.last_data[0].name, "price");
    }

    #[test]
    fn schemas_invalidation_marks_table_dropped() {
        let card = make_card();
        let status = evaluate_card_status(&card, &Invalidate::Schemas, "public", "orders");
        assert_eq!(status, BiCardStatus::TableDropped);
    }

    #[test]
    fn tables_invalidation_in_same_schema_marks_table_dropped() {
        let card = make_card();
        let inv = Invalidate::Tables {
            schema: "public".to_string(),
        };
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::TableDropped);
    }

    #[test]
    fn tables_invalidation_in_other_schema_keeps_status() {
        let card = make_card();
        let inv = Invalidate::Tables {
            schema: "warehouse".to_string(),
        };
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::Valid);
    }

    #[test]
    fn columns_invalidation_matching_table_marks_columns_removed() {
        let card = make_card();
        let inv = Invalidate::Columns {
            schema: "public".to_string(),
            table: "orders".to_string(),
        };
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::ColumnsRemoved);
    }

    #[test]
    fn columns_invalidation_other_table_keeps_status() {
        let card = make_card();
        let inv = Invalidate::Columns {
            schema: "public".to_string(),
            table: "customers".to_string(),
        };
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::Valid);
    }

    #[test]
    fn views_invalidation_does_not_affect_card_status() {
        let card = make_card();
        let inv = Invalidate::Views {
            schema: "public".to_string(),
        };
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::Valid);
    }

    #[test]
    fn already_dropped_card_remains_dropped_on_unrelated_event() {
        let mut card = make_card();
        card.status = BiCardStatus::TableDropped;
        let inv = Invalidate::Roles;
        let status = evaluate_card_status(&card, &inv, "public", "orders");
        assert_eq!(status, BiCardStatus::TableDropped);
    }
}
