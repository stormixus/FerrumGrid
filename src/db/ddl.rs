//! 2-step NOTIFY DDL execution (Plan v7 Phase 2 / ADR-2).
//!
//! DDL 의 invisibility 위험 (다른 viewer 가 stale cache 로 잘못된 SQL 발사) 을
//! 차단하기 위해 다음 시퀀스를 강제:
//!
//! ```text
//! Session A (DDL 실행자)             Session B (viewer)
//! ─────────────────────             ──────────────────
//! BEGIN; NOTIFY 'pre_drop'; COMMIT; ──>  receive pre_drop
//!                                        StateOp::InvalidateTable(oid, Pre)
//!                                        cache invalidate, stale marker render
//! wait 1s (best-effort ack window)
//! BEGIN; <DDL>; COMMIT;
//! BEGIN; NOTIFY 'post_drop'; COMMIT; ──> receive post_drop
//!                                        StateOp::RefreshMetadata(oid)
//! ```
//!
//! `pg_advisory_xact_lock` 패턴 폐기 사유: lock 해제와 NOTIFY flush 가 같은
//! commit 시점이라 ordering 불가 (Plan v7 Critic 4차 review D4).

use std::time::Duration;

use tokio_postgres::Client;

use crate::db::error::DbError;
use crate::db::invalidate::InvalidationPhase;
use crate::types::ConnectionId;

/// LISTEN/NOTIFY 채널 이름.
pub(crate) const NOTIFY_CHANNEL: &str = "ferrumgrid_invalidate";

/// 2-step NOTIFY 의 viewer ack window (Plan v7 §6 Phase 2 — best-effort 1s).
pub(crate) const ACK_WINDOW: Duration = Duration::from_secs(1);

/// `<table_oid>:<phase>` 형태의 NOTIFY payload 생성.
///
/// `table_oid: None` 인 경우 (CREATE TABLE 처럼 oid 미정) `0:<phase>` 로 fallback —
/// viewer 측은 oid 0 을 "전체 schema 무효화" 로 해석.
#[allow(dead_code)]
pub(crate) fn format_notify_payload(table_oid: Option<u32>, phase: InvalidationPhase) -> String {
    let phase_str = match phase {
        InvalidationPhase::Pre => "pre_drop",
        InvalidationPhase::Post => "post_drop",
        InvalidationPhase::SchemaChange => "schema_change",
    };
    format!("{}:{}", table_oid.unwrap_or(0), phase_str)
}

/// `pg_notify()` SQL 호출 SQL 생성 (NOTIFY statement 가 prepared statement 에서
/// 동작하지 않으므로 `pg_notify` 함수 호출 사용).
fn build_pg_notify_sql(payload: &str) -> String {
    let escaped = payload.replace('\'', "''");
    format!("SELECT pg_notify('{}', '{}')", NOTIFY_CHANNEL, escaped)
}

/// DDL 을 2-step NOTIFY sequence 안에서 실행.
///
/// 호출 site (table_designer.rs::apply_ddl 등) 가 `table_oid` 를 알 때 전달
/// (DROP/ALTER existing). CREATE 는 `None`.
///
/// 본 함수는 3 개의 별도 트랜잭션을 순차 실행 (lock + NOTIFY ordering race 회피).
/// DDL 실행 자체가 실패하면 post_drop 을 *안* 보내고 에러 반환 — viewer 는 stale
/// marker 로 남되 자동 refresh 는 안 됨 (사용자가 수동 refresh 가능).
#[allow(dead_code)]
pub async fn execute_ddl_with_invalidation(
    client: &Client,
    sql: &str,
    table_oid: Option<u32>,
    conn_id: ConnectionId,
) -> Result<(), DbError> {
    // Step 1 — pre_drop NOTIFY (별도 transaction, 즉시 commit 으로 viewer 에 flush)
    let pre_payload = format_notify_payload(table_oid, InvalidationPhase::Pre);
    let pre_sql = build_pg_notify_sql(&pre_payload);
    tracing::info!(
        target: "ferrumgrid::ddl",
        table_oid = ?table_oid,
        phase = "pre_drop",
        %conn_id,
        "sending pre_drop notify"
    );
    client
        .batch_execute(&pre_sql)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    // Step 1.5 — viewer ack window (best-effort, viewer 가 invalidate 처리할 시간)
    tokio::time::sleep(ACK_WINDOW).await;

    // Step 2 — actual DDL
    tracing::info!(
        target: "ferrumgrid::ddl",
        table_oid = ?table_oid,
        sql_len = sql.len(),
        %conn_id,
        "executing DDL"
    );
    client
        .batch_execute(sql)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    // Step 3 — post_drop NOTIFY (DDL 성공 후에만, viewer 가 metadata refresh)
    let post_payload = format_notify_payload(table_oid, InvalidationPhase::Post);
    let post_sql = build_pg_notify_sql(&post_payload);
    tracing::info!(
        target: "ferrumgrid::ddl",
        table_oid = ?table_oid,
        phase = "post_drop",
        %conn_id,
        "sending post_drop notify"
    );
    client
        .batch_execute(&post_sql)
        .await
        .map_err(|e| DbError::from_pg(&e, conn_id))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_pre_drop_with_oid() {
        assert_eq!(
            format_notify_payload(Some(16384), InvalidationPhase::Pre),
            "16384:pre_drop"
        );
    }

    #[test]
    fn payload_post_drop_with_oid() {
        assert_eq!(
            format_notify_payload(Some(16385), InvalidationPhase::Post),
            "16385:post_drop"
        );
    }

    #[test]
    fn payload_schema_change_with_oid() {
        assert_eq!(
            format_notify_payload(Some(2200), InvalidationPhase::SchemaChange),
            "2200:schema_change"
        );
    }

    #[test]
    fn payload_no_oid_fallback_to_zero() {
        assert_eq!(
            format_notify_payload(None, InvalidationPhase::Pre),
            "0:pre_drop"
        );
        assert_eq!(
            format_notify_payload(None, InvalidationPhase::Post),
            "0:post_drop"
        );
    }

    #[test]
    fn payload_round_trip_via_invalidate_parser() {
        // Plan v7 Phase 1.3 의 parse_payload 가 본 함수의 출력을 받아들일 수 있는지
        // 검증 — 양방향 호환.
        use crate::db::invalidate::parse_payload;

        for (oid, phase, expected_phase) in [
            (Some(16384), InvalidationPhase::Pre, InvalidationPhase::Pre),
            (Some(1), InvalidationPhase::Post, InvalidationPhase::Post),
            (
                Some(99),
                InvalidationPhase::SchemaChange,
                InvalidationPhase::SchemaChange,
            ),
        ] {
            let payload = format_notify_payload(oid, phase);
            let parsed = parse_payload(&payload);
            assert_eq!(parsed, Some((oid.unwrap_or(0), expected_phase)));
        }
    }

    #[test]
    fn pg_notify_sql_escapes_single_quote_in_payload() {
        // payload 에 ' 가 들어가면 안 되지만 (oid 는 숫자, phase 는 enum) defensive.
        let sql = build_pg_notify_sql("16384:'evil");
        assert!(sql.contains("'16384:''evil'"), "must escape: {sql}");
    }

    #[test]
    fn pg_notify_sql_uses_canonical_channel_name() {
        let sql = build_pg_notify_sql("0:pre_drop");
        assert!(sql.contains("'ferrumgrid_invalidate'"));
        assert_eq!(NOTIFY_CHANNEL, "ferrumgrid_invalidate");
    }

    #[test]
    fn ack_window_is_one_second() {
        assert_eq!(ACK_WINDOW, Duration::from_secs(1));
    }
}
