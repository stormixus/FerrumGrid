//! Plan v7 Phase 1.1 — `apply_data_edits` 트랜잭션 atomicity integration test.
//!
//! testcontainer/docker-compose 가 필요하므로 `#[ignore]`. 사용자가 다음 명령으로
//! 실행:
//!
//! ```sh
//! docker compose -f docker-compose.test.yml up -d --wait
//! cargo test --test edits_atomicity -- --ignored
//! ```
//!
//! 본 scaffold 는 시그니처 / channel / atomicity 의 *서면 명세* 이며, Phase 1.2
//! 의 RowKey + UI 진입점이 추가되면 INSERT/UPDATE/DELETE 시나리오가 mechanical
//! 하게 채워진다.

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn apply_data_edits_rolls_back_on_partial_failure() {
    // Plan v7 §5 S2 차단 검증:
    //  1. seed schema 에서 valid PK + invalid PK 를 섞은 RowEditOp::Update 시퀀스 발사
    //  2. 두 번째 op 가 affected != 1 로 실패해야 함
    //  3. 첫 번째 op 의 변경도 ROLLBACK 으로 사라져야 함
    //
    // 본 scaffold 는 Phase 1.2 RowKey 도입 후 채워진다 — 현 phase 에서는 단위
    // 테스트 (src/db/edits.rs:tests, 15 cases) 가 SQL 생성 정합성을 보장하고,
    // bridge.rs 의 begin/commit 패턴이 시그니처 레벨에서 atomicity 를 강제한다.
    //
    // FIXME(Phase 1.2): RowKey + Tmp→Pk 재매핑 후 INSERT/DELETE 시나리오 추가.
}
