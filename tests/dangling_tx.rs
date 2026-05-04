//! Plan v7 Phase 3 — pg_stat_activity 폴백 a/b/c × 30s/60s 통합 테스트.
//!
//! testcontainer/docker-compose 가 필요하므로 `#[ignore]`. 사용자가 다음 명령으로
//! 실행:
//!
//! ```sh
//! docker compose -f docker-compose.test.yml up -d --wait
//! cargo test --test dangling_tx -- --ignored
//! ```
//!
//! Plan v7 §6 Phase 3 의 6 케이스 매트릭스 (probe 권한 분기 × 임계값 경계):
//!
//! |          | 30s WARN | 60s ROLLBACK |
//! |----------|----------|--------------|
//! | (a) ServerSide   | 5s polling 으로 idle-in-tx 확정 → 토스트 | 60s 도달 → 강제 ROLLBACK + DiagnosticsPanel 알림 |
//! | (b) ClientTimer  | client `Instant::elapsed >= 30s` → 토스트 | client `Instant::elapsed >= 60s` → 강제 ROLLBACK + 영구 배너 |
//! | (c) ProbeFailed  | client timer + tracing WARN | client timer + 강제 ROLLBACK + 영구 배너 |
//!
//! 단위 테스트 (`src/db/dangling_tx.rs::tests`, 11 cases) 가 임계값 + 정책 분기를
//! 보장. 본 scaffold 는 통합 시나리오 명세 + Phase 3b 의 Query BEGIN tracking 통합
//! 후 mechanical 채우기.

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn server_side_probe_warn_at_30s() {
    // (a) ServerSide × 30s WARN:
    //   1. CONNECT to PG with role having pg_stat_activity SELECT 권한
    //   2. probe → ServerSide
    //   3. BEGIN; (no further statements)
    //   4. 30s 후 polling 이 idle-in-tx 발견 → state.dangling_tx.status == ShouldWarn
    //   5. 토스트 메시지 발사 검증
    //
    // FIXME(Phase 3b): bridge.rs 에 dangling_tx polling 통합 후 채움.
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn server_side_rollback_at_60s() {
    // (a) ServerSide × 60s ROLLBACK:
    //   1. ServerSide mode 에서 BEGIN
    //   2. 60s 도달 → state.dangling_tx.status == ShouldRollback
    //   3. 자동 ROLLBACK 발사 검증 + DiagnosticsPanel 알림
    //
    // FIXME(Phase 3b)
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn client_timer_probe_warn_at_30s() {
    // (b) ClientTimer × 30s WARN:
    //   1. CONNECT to PG with limited role (no pg_stat_activity 권한)
    //   2. probe → ClientTimer + DiagnosticsPanel 영구 배너
    //   3. BEGIN
    //   4. client `Instant::elapsed >= 30s` → ShouldWarn
    //
    // FIXME(Phase 3b)
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn client_timer_rollback_at_60s() {
    // (b) ClientTimer × 60s ROLLBACK:
    //   client `Instant::elapsed >= 60s` → ShouldRollback + 자동 ROLLBACK
    //
    // FIXME(Phase 3b)
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn probe_failed_warn_at_30s() {
    // (c) ProbeFailed × 30s WARN:
    //   1. probe 자체 에러 (네트워크 단절 시뮬) → ProbeFailed + tracing WARN + 영구 배너
    //   2. BEGIN
    //   3. client `Instant::elapsed >= 30s` → ShouldWarn
    //
    // FIXME(Phase 3b): probe 실패 시뮬레이션 (firewall block 등)
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn probe_failed_rollback_at_60s() {
    // (c) ProbeFailed × 60s ROLLBACK:
    //   client `Instant::elapsed >= 60s` → ShouldRollback + 자동 ROLLBACK
    //
    // FIXME(Phase 3b)
}
