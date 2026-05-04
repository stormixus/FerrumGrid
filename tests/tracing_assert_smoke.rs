//! tracing_assert 헬퍼의 self-test.
//!
//! Plan v7 Phase 0 acceptance #2: "기존 테스트 1개를 새 헬퍼로 마이그레이션".
//! ferrumgrid 는 단일 binary crate (src/main.rs) 라 통합 테스트에서 내부 함수
//! 호출이 어렵고, 기존 tracing emit site (src/db/bridge.rs:193 등) 는 모두
//! error path 라 정상 시나리오에서 발사되지 않는다. 따라서 헬퍼 자체에 대한
//! smoke test 를 마이그레이션 evidence 로 사용한다.

#[path = "support/tracing_assert.rs"]
mod tracing_assert;

use tracing_assert::{assert_event, with_capture};

#[test]
fn captures_info_event_with_string_field() {
    let events = with_capture(|| {
        tracing::info!(target: "ferrumgrid::mutation", op = "insert", "applied edits");
    });

    assert_event(&events, "ferrumgrid::mutation")
        .level(tracing::Level::INFO)
        .field_eq("op", "insert")
        .field_present("message");
}

#[test]
fn captures_multiple_fields_with_numeric_values() {
    let events = with_capture(|| {
        tracing::warn!(
            target: "ferrumgrid::ddl",
            applied_count = 42_u64,
            sqlstate = "23505",
            "duplicate key"
        );
    });

    assert_event(&events, "ferrumgrid::ddl")
        .level(tracing::Level::WARN)
        .field_eq("applied_count", "42")
        .field_non_empty("sqlstate");
}

#[test]
fn returns_most_recent_event_when_target_repeats() {
    let events = with_capture(|| {
        tracing::info!(target: "ferrumgrid::cache", op = "first");
        tracing::info!(target: "ferrumgrid::cache", op = "second");
    });

    assert_event(&events, "ferrumgrid::cache").field_eq("op", "second");
}

#[test]
fn ignores_events_from_other_targets() {
    let events = with_capture(|| {
        tracing::info!(target: "noisy::other", op = "ignored");
        tracing::info!(target: "ferrumgrid::mutation", op = "kept");
    });

    assert_event(&events, "ferrumgrid::mutation").field_eq("op", "kept");
}
