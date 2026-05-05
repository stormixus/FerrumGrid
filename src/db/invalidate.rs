//! Cache invalidation infrastructure.
//!
//! Plan v7 Phase 1.3 / §9 / ADR-2:
//! - **`Invalidate`**: 무효화 대상 (Schemas / Tables / Columns / Indexes / Fks /
//!   Views / Functions / Roles).
//! - **`InvalidationPhase`**: 2-step NOTIFY 의 `Pre` (DDL 직전) / `Post` (DDL 직후) /
//!   `SchemaChange` (일반).
//! - **`parse_payload`**: `<table_oid>:<phase>` (LISTEN ferrumgrid_invalidate
//!   payload) 파싱.
//! - **`dedupe_and_sort`**: 동일 프레임 내 중복 도착 dedup + coarse→fine 발사
//!   순서 (Schemas → Tables → Columns/Indexes/Fks).
//! - **`with_echo_timeout`**: `tokio::time::timeout(Duration::from_secs(5), ...)`
//!   thin wrapper. timeout 시 dirty bit 유지 + DiagnosticsPanel 경고.
//!
//! 본 단계 (Phase 1.3) 에서는 enum + parse + dedup + 단위 테스트만 도입.
//! `connection_task` 의 LISTEN 등록과 viewer 측 dispatch wire-up 은 Phase 1.95
//! 의 grid state surface 통합과 함께 진행한다.

use std::collections::HashSet;
use std::time::Duration;

/// 2-step NOTIFY 의 phase (Plan §9).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidationPhase {
    /// DDL 실행 직전. viewer 가 stale marker 표시 + cache invalidate.
    Pre,
    /// DDL 실행 직후. viewer 가 metadata refresh.
    Post,
    /// 일반 schema 변경 (DDL 외 invalidation).
    SchemaChange,
}

impl InvalidationPhase {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "pre_drop" | "pre" => Some(Self::Pre),
            "post_drop" | "post" => Some(Self::Post),
            "schema_change" => Some(Self::SchemaChange),
            _ => None,
        }
    }
}

/// 무효화 대상.
///
/// `Invalidate` 의 ordering 은 *coarse → fine*: Schemas (가장 광범위) →
/// Tables → Columns/Indexes/Fks (가장 구체적). dedupe 후 이 순서로 발사하면
/// catalog cache 의 fetch 의존성이 자연스럽게 해결된다.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Invalidate {
    /// 전체 schema 목록 재조회.
    Schemas,
    /// 특정 schema 의 tables 재조회.
    Tables { schema: String },
    /// 특정 schema 의 views 재조회.
    Views { schema: String },
    /// 특정 schema 의 functions 재조회.
    Functions { schema: String },
    /// roles 재조회 (schema 무관).
    Roles,
    /// 특정 table 의 columns 재조회.
    Columns { schema: String, table: String },
    /// 특정 table 의 indexes 재조회.
    Indexes { schema: String, table: String },
    /// 특정 schema 의 foreign keys 재조회.
    Fks { schema: String },
}

impl Invalidate {
    /// coarse→fine 정렬용 우선순위 (작을수록 먼저 발사).
    fn order_key(&self) -> u8 {
        match self {
            Self::Schemas => 0,
            Self::Tables { .. } => 1,
            Self::Views { .. } => 1,
            Self::Functions { .. } => 1,
            Self::Roles => 1,
            Self::Columns { .. } => 2,
            Self::Indexes { .. } => 2,
            Self::Fks { .. } => 2,
        }
    }
}

/// LISTEN ferrumgrid_invalidate payload 파싱.
///
/// 형식: `<table_oid>:<phase>` (예: `16384:pre_drop`).
/// `table_oid` 는 PostgreSQL OID (`u32`), `<phase>` 는 [`InvalidationPhase::parse`].
///
/// 잘못된 형식이면 `None` — 호출자는 무시 + tracing::warn 권장.
#[allow(dead_code)]
pub fn parse_payload(payload: &str) -> Option<(u32, InvalidationPhase)> {
    let (oid_str, phase_str) = payload.split_once(':')?;
    let oid = oid_str.parse::<u32>().ok()?;
    let phase = InvalidationPhase::parse(phase_str)?;
    tracing::info!(
        target: "ferrumgrid::cache",
        table_oid = oid,
        phase = ?phase,
        "parsed invalidation payload"
    );
    Some((oid, phase))
}

/// 동일 프레임 내 중복 도착 dedupe + coarse→fine 발사 순서로 정렬.
#[allow(dead_code)]
pub fn dedupe_and_sort(events: Vec<Invalidate>) -> Vec<Invalidate> {
    let input_count = events.len();
    let unique: HashSet<Invalidate> = events.into_iter().collect();
    let mut sorted: Vec<Invalidate> = unique.into_iter().collect();
    // 1차: coarse→fine, 2차: variant 결정성을 위해 Debug 문자열 비교 (HashSet 순서
    // 무의미 → 테스트 안정성 위해 secondary 정렬).
    sorted.sort_by_key(|e| (e.order_key(), format!("{e:?}")));
    tracing::info!(
        target: "ferrumgrid::cache",
        input = input_count,
        output = sorted.len(),
        "deduped invalidation events"
    );
    sorted
}

/// `tokio::time::timeout(Duration::from_secs(5), fut)` thin wrapper.
///
/// timeout 도달 시 호출자는 dirty bit 유지 + DiagnosticsPanel 경고 (Plan §9).
#[allow(dead_code)]
pub async fn with_echo_timeout<F, T>(fut: F) -> Result<T, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(Duration::from_secs(5), fut).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- parse_payload ----------

    #[test]
    fn parse_payload_pre_drop_with_oid() {
        assert_eq!(
            parse_payload("16384:pre_drop"),
            Some((16384, InvalidationPhase::Pre))
        );
    }

    #[test]
    fn parse_payload_post_drop_with_oid() {
        assert_eq!(
            parse_payload("16385:post_drop"),
            Some((16385, InvalidationPhase::Post))
        );
    }

    #[test]
    fn parse_payload_schema_change() {
        assert_eq!(
            parse_payload("0:schema_change"),
            Some((0, InvalidationPhase::SchemaChange))
        );
    }

    #[test]
    fn parse_payload_rejects_missing_colon() {
        assert_eq!(parse_payload("16384"), None);
    }

    #[test]
    fn parse_payload_rejects_non_numeric_oid() {
        assert_eq!(parse_payload("xyz:pre_drop"), None);
    }

    #[test]
    fn parse_payload_rejects_unknown_phase() {
        assert_eq!(parse_payload("16384:exploded"), None);
    }

    // ---------- dedupe_and_sort ----------

    #[test]
    fn dedupe_collapses_repeated_events() {
        let events = vec![
            Invalidate::Schemas,
            Invalidate::Schemas,
            Invalidate::Schemas,
        ];
        let result = dedupe_and_sort(events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Invalidate::Schemas);
    }

    #[test]
    fn sort_is_coarse_to_fine_schemas_first() {
        let events = vec![
            Invalidate::Columns {
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            Invalidate::Schemas,
            Invalidate::Tables {
                schema: "public".to_string(),
            },
        ];
        let result = dedupe_and_sort(events);
        assert_eq!(result[0], Invalidate::Schemas);
        assert!(matches!(result[1], Invalidate::Tables { .. }));
        assert!(matches!(result[2], Invalidate::Columns { .. }));
    }

    #[test]
    fn dedupe_distinguishes_different_schemas() {
        let events = vec![
            Invalidate::Tables {
                schema: "public".to_string(),
            },
            Invalidate::Tables {
                schema: "audit".to_string(),
            },
            Invalidate::Tables {
                schema: "public".to_string(),
            },
        ];
        let result = dedupe_and_sort(events);
        assert_eq!(result.len(), 2, "두 schema 의 Tables 는 별개 invalidation");
    }

    #[test]
    fn empty_input_returns_empty_output() {
        let result = dedupe_and_sort(Vec::new());
        assert!(result.is_empty());
    }

    // ---------- with_echo_timeout ----------

    #[tokio::test]
    async fn echo_timeout_completes_when_future_resolves_quickly() {
        let result = with_echo_timeout(async { 42 }).await;
        assert_eq!(result, Ok(42));
    }

    // 5s timeout 의 *경계값* 동작 (Err 반환) 은 `tokio::time::timeout` 자체가
    // 보장한다 (tokio test 스위트 검증). 본 모듈은 Ok 경로 + 시그니처 호환만
    // 검증하고, hang 시뮬레이션 테스트는 tokio test-util feature 의존을 피하기
    // 위해 추가하지 않는다.
}
