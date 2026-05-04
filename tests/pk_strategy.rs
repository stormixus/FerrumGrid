//! Plan v7 Phase 1.2 — PK whitelist + RowKey 5 변종 통합 테스트.
//!
//! testcontainer/docker-compose 가 필요하므로 `#[ignore]`. 사용자가 다음 명령으로
//! 실행:
//!
//! ```sh
//! docker compose -f docker-compose.test.yml up -d --wait
//! cargo test --test pk_strategy -- --ignored
//! ```
//!
//! Plan v7 §10 의 5 케이스 매트릭스:
//!  1. **single int PK**: 가장 흔한 SERIAL/identity 컬럼
//!  2. **composite PK**: (tenant_id, order_id) 형태
//!  3. **identity PK**: GENERATED ALWAYS AS IDENTITY
//!  4. **citext PK**: case-insensitive text 비교
//!  5. **uuid PK**: gen_random_uuid() default
//!  6. **PK 없음 (거부)**: data_edit_summary 가 grid_pk_required blocked_reason
//!
//! 본 scaffold 는 시그니처 / channel / 정책 의 *서면 명세* 이며, 단위 테스트
//! (src/db/row_key.rs:tests, 12 cases) 가 RowKey 결정성 + whitelist 정책을
//! 보장한다. integration 시나리오는 다음 phase 에서 채워진다.

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn pk_strategy_matrix_integration() {
    // Plan v7 §11 테스트 매트릭스 (PK 4종 + 없음 거부) 검증:
    //  - 각 PK 변종에서 INSERT → RowKey 생성 → UPDATE → DELETE round-trip
    //  - PK 없는 테이블에서는 apply_data_edits 가 거부 (Update 인 경우 에러)
    //
    // FIXME(Phase 1.95): grid state surface 통합 후 실제 사용자 시나리오로 채움.
}
