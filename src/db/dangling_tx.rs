//! Dangling transaction detection infrastructure (Plan v7 Phase 3 / S2).
//!
//! Query 탭에서 사용자가 명시 `BEGIN;` 후 `COMMIT;` 또는 `ROLLBACK;` 없이
//! 탭을 닫거나 앱을 종료하면 PostgreSQL connection 이 idle-in-transaction
//! 상태로 남아 connection pool 고갈 또는 lock 점유 위험. 본 모듈은 다음을 제공:
//!
//! - **`DanglingTxMode`** — probe 결과에 따른 감지 전략:
//!   - `ServerSide`: `pg_stat_activity` SELECT 권한 있음 → 5s polling 으로 확정 감지
//!   - `ClientTimer`: 권한 없음 → client-side `Instant::elapsed()` 기반 추정
//!   - `ProbeFailed`: probe 자체 네트워크/에러 → ClientTimer + tracing WARN
//! - **임계값 상수**: `TX_WARN_AFTER` (30s 토스트) / `TX_ROLLBACK_AFTER` (60s 강제) / `POLL_INTERVAL` (5s)
//! - **`evaluate_status(elapsed)`** — elapsed Duration → `DanglingTxStatus { Ok, ShouldWarn, ShouldRollback }`
//! - **`format_probe_sql()`** — probe SQL 생성
//!
//! 본 단계 (Phase 3a) 에서는 *infra-only*. Query 탭 통합 (BEGIN tracking +
//! DiagnosticsPanel 배너 + UI hard-disable + 자동 ROLLBACK) 은 Phase 3b 에서.

use std::time::Duration;

/// Query 탭 dangling transaction 감지 전략.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanglingTxMode {
    /// `pg_stat_activity.state = 'idle in transaction'` polling — 권한 있음.
    ServerSide,
    /// Client-side `Instant::elapsed()` — 권한 없음.
    ClientTimer,
    /// Probe 자체 실패 (네트워크/접속 단절) — ClientTimer 폴백 + tracing WARN.
    ProbeFailed,
}

/// idle-in-tx 감지 후 사용자 토스트 경고 임계값. Plan v7 §6 Phase 3.
#[allow(dead_code)]
pub const TX_WARN_AFTER: Duration = Duration::from_secs(30);

/// idle-in-tx 감지 후 강제 ROLLBACK 임계값. Plan v7 §6 Phase 3.
#[allow(dead_code)]
pub const TX_ROLLBACK_AFTER: Duration = Duration::from_secs(60);

/// `pg_stat_activity` polling 주기 (ServerSide 모드).
#[allow(dead_code)]
pub const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Probe SQL — 현재 user 가 `pg_stat_activity` SELECT 권한 있는지 확인.
#[allow(dead_code)]
pub fn format_probe_sql() -> &'static str {
    "SELECT has_table_privilege(current_user, 'pg_catalog.pg_stat_activity', 'SELECT')"
}

/// idle-in-tx elapsed 시간을 임계값과 비교한 결과.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanglingTxStatus {
    /// `< TX_WARN_AFTER` — 정상.
    Ok,
    /// `[TX_WARN_AFTER, TX_ROLLBACK_AFTER)` — 토스트 경고 필요.
    ShouldWarn,
    /// `>= TX_ROLLBACK_AFTER` — 강제 ROLLBACK 필요.
    ShouldRollback,
}

/// Begin 후 elapsed 시간 → 권장 액션.
#[allow(dead_code)]
pub fn evaluate_status(elapsed: Duration) -> DanglingTxStatus {
    if elapsed >= TX_ROLLBACK_AFTER {
        DanglingTxStatus::ShouldRollback
    } else if elapsed >= TX_WARN_AFTER {
        DanglingTxStatus::ShouldWarn
    } else {
        DanglingTxStatus::Ok
    }
}

/// Probe 결과 (`bool` 권한 여부) → 적절한 `DanglingTxMode`.
#[allow(dead_code)]
pub fn mode_from_probe(probe_result: Result<bool, ()>) -> DanglingTxMode {
    match probe_result {
        Ok(true) => DanglingTxMode::ServerSide,
        Ok(false) => DanglingTxMode::ClientTimer,
        Err(()) => DanglingTxMode::ProbeFailed,
    }
}

/// `DanglingTxMode` 가 영구 배너 표시를 요구하는지 (b/c 분기).
#[allow(dead_code)]
pub fn requires_permanent_banner(mode: DanglingTxMode) -> bool {
    matches!(mode, DanglingTxMode::ClientTimer | DanglingTxMode::ProbeFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_match_plan_v7() {
        assert_eq!(TX_WARN_AFTER, Duration::from_secs(30));
        assert_eq!(TX_ROLLBACK_AFTER, Duration::from_secs(60));
        assert_eq!(POLL_INTERVAL, Duration::from_secs(5));
    }

    #[test]
    fn status_under_warn_is_ok() {
        assert_eq!(
            evaluate_status(Duration::from_secs(0)),
            DanglingTxStatus::Ok
        );
        assert_eq!(
            evaluate_status(Duration::from_secs(29)),
            DanglingTxStatus::Ok
        );
    }

    #[test]
    fn status_at_warn_threshold_is_should_warn() {
        assert_eq!(
            evaluate_status(Duration::from_secs(30)),
            DanglingTxStatus::ShouldWarn
        );
    }

    #[test]
    fn status_between_warn_and_rollback_is_should_warn() {
        assert_eq!(
            evaluate_status(Duration::from_secs(45)),
            DanglingTxStatus::ShouldWarn
        );
        assert_eq!(
            evaluate_status(Duration::from_secs(59)),
            DanglingTxStatus::ShouldWarn
        );
    }

    #[test]
    fn status_at_rollback_threshold_is_should_rollback() {
        assert_eq!(
            evaluate_status(Duration::from_secs(60)),
            DanglingTxStatus::ShouldRollback
        );
        assert_eq!(
            evaluate_status(Duration::from_secs(120)),
            DanglingTxStatus::ShouldRollback
        );
    }

    #[test]
    fn mode_from_probe_true_is_server_side() {
        assert_eq!(mode_from_probe(Ok(true)), DanglingTxMode::ServerSide);
    }

    #[test]
    fn mode_from_probe_false_is_client_timer() {
        assert_eq!(mode_from_probe(Ok(false)), DanglingTxMode::ClientTimer);
    }

    #[test]
    fn mode_from_probe_err_is_probe_failed() {
        assert_eq!(mode_from_probe(Err(())), DanglingTxMode::ProbeFailed);
    }

    #[test]
    fn server_side_does_not_need_banner() {
        assert!(!requires_permanent_banner(DanglingTxMode::ServerSide));
    }

    #[test]
    fn client_timer_and_probe_failed_need_banner() {
        assert!(requires_permanent_banner(DanglingTxMode::ClientTimer));
        assert!(requires_permanent_banner(DanglingTxMode::ProbeFailed));
    }

    #[test]
    fn probe_sql_uses_pg_stat_activity_in_pg_catalog() {
        let sql = format_probe_sql();
        assert!(sql.contains("has_table_privilege"));
        assert!(sql.contains("pg_catalog.pg_stat_activity"));
        assert!(sql.contains("'SELECT'"));
    }
}
