//! Plan v7 Phase 2 — DROP CASCADE 2 viewer 시나리오 통합 테스트.
//!
//! testcontainer/docker-compose 가 필요하므로 `#[ignore]`. 사용자가 다음 명령으로
//! 실행:
//!
//! ```sh
//! docker compose -f docker-compose.test.yml up -d --wait
//! cargo test --test drop_cascade -- --ignored
//! ```
//!
//! Plan v7 §5 S1 차단 시나리오:
//! 1. Session A 가 `db::ddl::execute_ddl_with_invalidation(client, "DROP TABLE x", oid)` 실행
//! 2. Session B (viewer) 가 `LISTEN ferrumgrid_invalidate` 활성
//! 3. 검증:
//!    - Session B 가 pre_drop NOTIFY 수신 (DDL 실행 *전*)
//!    - Session A 의 1s ack window 후 DDL 실행
//!    - Session B 가 post_drop NOTIFY 수신 (DDL 실행 *후*)
//!    - 두 NOTIFY 가 ordering 보장 (pre 가 항상 post 보다 먼저)
//!
//! 본 scaffold 는 시나리오 *서면 명세* 이며, 실제 wire-up 은 향후 LISTEN 인프라
//! (Phase 1.95+ 의 connection_task 통합) 가 완료된 후 채워진다. 단위 테스트
//! (`src/db/ddl.rs::tests`, 8 cases + `src/db/invalidate.rs::tests`, 11 cases)
//! 가 NOTIFY payload 정합성 + parse 호환성을 보장한다.

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn drop_cascade_two_viewer_pre_then_post_notify_ordering() {
    // FIXME(Phase 1.95+ LISTEN integration): 시나리오 본체.
    //
    // pseudo:
    //   let conn_a = setup_connection().await;
    //   let conn_b = setup_listener().await;
    //   conn_b.listen("ferrumgrid_invalidate").await;
    //
    //   let table_oid = create_test_table(&conn_a).await;
    //   let drop_handle = tokio::spawn(execute_ddl_with_invalidation(
    //       &conn_a, "DROP TABLE ferrumgrid_test.disposable", Some(table_oid), conn_id_a,
    //   ));
    //
    //   let pre = conn_b.recv_notify().await;
    //   assert_eq!(pre.payload, format!("{table_oid}:pre_drop"));
    //
    //   drop_handle.await.expect("DDL succeeds");
    //
    //   let post = conn_b.recv_notify().await;
    //   assert_eq!(post.payload, format!("{table_oid}:post_drop"));
}

#[tokio::test]
#[ignore = "requires docker compose -f docker-compose.test.yml up -d --wait"]
async fn drop_cascade_dependents_query_returns_at_most_51_rows() {
    // Plan v7 §5 S1 차단의 dependents preview SQL 검증 (US-P2d 후속):
    //
    //   WITH RECURSIVE deps AS (
    //     SELECT * FROM pg_depend WHERE refobjid = $1::regclass
    //     UNION
    //     SELECT pd.* FROM pg_depend pd JOIN deps ON pd.refobjid = deps.objid
    //   )
    //   SELECT * FROM deps LIMIT 51
    //
    // dependents > 50 인 테이블에서 정확히 51 rows 반환되어 cutoff label 표시 가능.
    //
    // FIXME(US-P2d): pg_depend 헬퍼 도입 후 채움.
}
