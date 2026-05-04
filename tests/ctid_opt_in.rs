//! Plan v7 Phase 1.3 — `ferrumgrid.unsafe_ctid` ON/OFF 통합 테스트.
//!
//! testcontainer/docker-compose 가 필요하므로 `#[ignore]`. 사용자가 다음 명령으로
//! 실행:
//!
//! ```sh
//! docker compose -f docker-compose.test.yml up -d --wait
//! cargo test --test ctid_opt_in -- --ignored
//! ```
//!
//! Plan v7 §6 Phase 1.3 의 2 케이스:
//!  1. **flag OFF (default)**: PK 부재 테이블에서 mutation UI hard-disable + 배너
//!     ("PK whitelist 외 타입 — 편집 비활성")
//!  2. **flag ON**: `RowKeyKind::Ctid` 활성 + `RETURNING ctid` 강제 +
//!     `affected != 1` 즉시 ROLLBACK + DiagnosticsPanel 영구 배너 표시
//!
//! 본 scaffold 는 정책 *서면 명세* 이며, 단위 테스트 (src/db/invalidate.rs:tests
//! 12 cases + src/db/row_key.rs:tests 12 cases + Settings::unsafe_ctid 직렬화)
//! 가 정책 정합성을 보장한다. integration 시나리오는 Phase 1.95 의 grid state
//! surface 통합 후 채워진다.

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn ctid_opt_in_flag_off_blocks_mutation_on_pkless_table() {
    // PK 부재 테이블 (예: PG `pg_stat_activity` view, 또는 seed 의 pk-less table)
    // 에서 grid 가 read-only 표시 검증.
    //
    // FIXME(Phase 1.95): grid state surface 통합 후 클릭 시뮬레이션으로 채움.
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn ctid_opt_in_flag_on_uses_ctid_with_returning_guard() {
    // unsafe_ctid = true 설정 후:
    //  1. PK 없는 테이블 INSERT → RowKeyKind::Ctid 회수 (RETURNING ctid)
    //  2. UPDATE WHERE ctid = ... → affected = 1 commit
    //  3. UPDATE WHERE ctid = (이미 변경된) → affected = 0 → ROLLBACK
    //
    // FIXME(Phase 1.95): apply_data_edits ctid 분기 wire-up 후 채움.
}
