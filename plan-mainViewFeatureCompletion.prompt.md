## Plan v7: MainView 기능 완성 로드맵 (RALPLAN-DR Consensus)

> **Version**: v7 (Planner ↔ Architect ↔ Critic 4-round consensus, 2026-05-04)
> **Scope**: FerrumGrid (Rust + egui PostgreSQL GUI) MainView 의 11개 탭(Connection / Table / View / MaterializedView / Function / User / Query / Data / Backup / Automation / Model / BI) 을 "조회 중심" 에서 "실작업 가능" 상태로 완성.
> **Schedule**: floor 20d (산술합), actual 20~25d (context switch + PR review + 회귀 수정 buffer). 모든 phase 직렬.

---

## 1. Principles

- **P1 Single Source of Truth** — 모든 cell 상태 변경은 `MainViewState` 만 mutate. View / Input layer 는 직접 mutation 금지.
- **P2 Dispatch-only mutation** — UI 상태 변경은 모두 `StateOp` enum 경유. View layer (grid render) 는 state getter (`selection_range`, `cursor_cell`, `editing_cell`) 만 읽고 paint, 자체 mutation 0.
- **P3 Type-driven invariants** — NaN / encoding / ordering 으로 깨질 수 있는 invariant 는 *타입 레벨 게이트* 로 강제. grep CI rule 같은 soft gate 금지.
- **P3' Adapter sunset rule** — legacy API 는 항상 (a) `legacy_*` prefix, (b) `#[deprecated(note = "remove in <milestone>")]`, (c) 해당 milestone 진입 시 `grep 0` CI 게이트 통과. 어댑터 도입 = 제거 milestone 동시 약속.
- **P4 DDL 안전성 — explicit ordering** — invisibility 위험 작업 (DROP CASCADE 등) 은 *2-step NOTIFY (pre / post) + viewer ack window* 로 ordering 명시. advisory lock + NOTIFY 동일 트랜잭션 패턴 금지 (commit 시 lock 해제와 notify flush 가 동시 → ordering 보장 불가).
- **P5 Phase 단위 PR — revertibility** — 각 phase 는 독립 revertible PR. schema 도입과 사용처 마이그레이션은 동일 PR 금지 (R7 위험).

---

## 2. Decision Drivers

1. **Correctness > Performance** — NaN-safe 비교, ordering-safe DDL, atomic edits.
2. **Compile-time enforcement > Runtime/Process gate** — type system 게이트가 grep CI 보다 강함.
3. **Phase 격리성 (revertibility)** — 각 PR 가 독립 revert 가능, 단계별 회귀 차단.

---

## 3. Viable Options & Selection

### 비교 게이트 (P3 강제 메커니즘)

| Option | 메커니즘 | 강도 | 채택 |
|--------|---------|------|------|
| O1 | `grep -RnE 'cell.*==.*cell' src/` CI rule | soft, `// allow:` escape hatch 우회 가능 | ✗ |
| O2 | `clippy::disallowed_methods` for PartialEq | semi-hard, false positive 多 | ✗ |
| **O3** | **`CmpCell<'_>` newtype + `CellValue: PartialEq` `#[cfg(test)]` 격리** | **hard (compile error)** | **✓** |

### DDL Ordering (P4 강제 메커니즘)

| Option | 메커니즘 | Race | 채택 |
|--------|---------|------|------|
| O1 | `pg_advisory_xact_lock` + `NOTIFY` 동일 트랜잭션 | lock 해제와 notify flush 가 같은 commit 시점 → ordering 불가 | ✗ |
| O2 | LISTEN/NOTIFY only (single notify) | viewer 무효화 전에 DDL 완료 가능 | ✗ |
| **O3** | **2-step (pre_drop NOTIFY → 1s ack → DDL → post_drop NOTIFY)** | viewer 가 pre_drop 수신 후 stale marker → race-free | **✓** |

### Edit 모델

| Option | 설명 | 채택 |
|--------|------|------|
| A | 현행 `cells: HashMap<(row,col), EditableCell>` 에 `inserted_rows/deleted_rows` HashSet 추가 | ✗ — INSERT 후 동일 row UPDATE 시 row_idx 충돌, server-generated PK 회수 경로 부재 |
| **B** | **In-memory 분리 (`updates`/`inserts: Vec<TmpRow>`/`deletes: HashSet<RowKey>`) + Apply 시 `Vec<RowEditOp>` 정규화 (insert → updates → deletes 순서)** | **✓** — squash O(1) per cell, tmp row 명시, server-generated PK 회수 가능 |
| C | server-side cursor + portal stateful editing | ✗ — single-writer 모델과 불필요 |

### 벤치 baseline (Option B 검증)

- **Baseline A** — 현재 `&CellValue == &CellValue` (PartialEq direct, NaN-unsafe). **reference only**.
- **Baseline B** — NaN-safe naive (Float→`total_cmp`, 외 PartialEq). **ceiling 임계값 적용 대상**.
- 임계값:
  - **<3x slowdown vs B**: 무조건 채택 (현재 결정).
  - **3x ~ 10x**: 채택 + Follow-up 등록, Phase 1.0 종료 후 모니터링.
  - **≥10x**: Phase 1.0 entry 게이트에서 재논의 (Option B 모델 2중화 비용 vs 마이크로 perf 재평가).

---

## 4. Pre-mortem (3 시나리오) + 차단

### S1 — DROP CASCADE invisibility
- **Trigger**: 동시 viewer 가 DROP 직전 row fetch 진행 중, DROP 완료 후에도 stale 캐시로 계속 렌더 → 잘못된 SQL 재발사 위험.
- **Block**: 2-step NOTIFY sequence (위 O3) — pre_drop 수신 즉시 cache invalidate + stale marker, DDL 완료 후 post_drop 으로 metadata refresh.
- **Detection**: `tests/integ/drop_cascade.rs` — 2 viewer 동시 시뮬, pre/post 모두 수신 검증.
- **Dependents preview SQL**: `WITH RECURSIVE deps AS (SELECT * FROM pg_depend WHERE refobjid = $1::regclass UNION SELECT pd.* FROM pg_depend pd JOIN deps ON pd.refobjid = deps.objid) SELECT * FROM deps LIMIT 51` (51 = 50 + over-cutoff 감지).

### S2 — `apply_data_edits` 트랜잭션 누락 → 부분 적용
- **Block**: `&Client` → `&mut Transaction<'_>` 시그니처 강제 (Phase 1.1). 호출 site (`bridge.rs:528-535`) 컴파일 오류 유도.
- **Detection**: `tests/integ/edits_atomicity.rs` — 중간 실패 시 모든 row 롤백 검증.

### S3 — Float NaN 으로 selection invariant 붕괴
- **Block**: `CmpCell<'_>` newtype 강제 (Phase 1.0). Float → canonical string normalization (NaN → `"\0NaN"` 고정).
- **Detection**: proptest (`tests/prop/cmp_cell_nan_safe.rs`) — NaN 포함 random vec 에서 `CmpCell` reflexive 검증.

---

## 5. Phase 상세

> 골격: **0 → 1.0 → 1.1 → 1.2 → 1.3 → 1.95 → 2 → 3 → 4a → 4b → 4c**, 직렬, floor 20d / actual 20~25d.

### Phase 0 — Baseline 측정 (1d)
- bench A/B 기록, 현재 grid render p95 측정, queries.rs/grid.rs/state.rs LOC snapshot.
- tracing assertion 헬퍼 (`tests/support/tracing_assert.rs`) 도입 — `LayerCollector` 기반 (snapshot 무관, 필드별 predicate 검증).
- DiagnosticsPanel scaffold (read-only).
- **DoD**: `bench/baseline.json` commit, 헬퍼로 기존 테스트 1개 마이그레이션 성공.

### Phase 1.0 — `CmpCell<'_>` newtype 도입 (2d)
```rust
#[derive(PartialEq, Eq, Hash)]
pub struct CmpCell<'a> { canonical: Cow<'a, str> }

impl CellValue {
    pub fn cmp_view(&self) -> CmpCell<'_> { /* canonical_string 경유 */ }
}

#[cfg(test)]
impl PartialEq for CellValue { /* test util only */ }
```
- prod 빌드에서 `CellValue: PartialEq` 부재 → 직접 `==` 컴파일 에러 (P3 hard gate).
- `// allow:` escape hatch **삭제** (D3 재발 차단).
- bench A/B 둘 다 측정, 결과를 ceiling (<3x / 3-10x / ≥10x) 분기 evidence 로 기록.
- **DoD**: `cargo build --release` 통과, 모든 비교 site `cmp_view()` 경유 grep 검증, NaN proptest 녹색.

### Phase 1.1 — `apply_data_edits` 트랜잭션화 (2d)
- 시그니처: `pub async fn apply_data_edits(tx: &mut Transaction<'_>, edits: &[RowEditOp]) -> Result<MutationOutcome, DbError>` (`src/db/queries.rs:142` 정의).
- 호출 site (`src/db/bridge.rs:528-535`) 트랜잭션 begin/commit 추가:
  ```rust
  ConnCommand::ApplyDataEdits { edits, reply } => {
      let mut tx = client.transaction().await?;
      let outcome = queries::apply_data_edits(&mut tx, &edits).await;
      match outcome {
          Ok(o)  => { tx.commit().await?;   reply.send(Ok(o)).ok(); }
          Err(e) => { tx.rollback().await?; reply.send(Err(e)).ok(); }
      }
  }
  ```
- INSERT 는 `RETURNING <pk>` 강제 → `MutationOutcome::inserted_keys: Vec<(TmpId, RowKey)>` 회수.
- 동일 트랜잭션 내 INSERT 후 그 PK 를 UPDATE 대상으로: `RowKey::Tmp` → `RowKey::Pk` 재매핑.
- **DoD**: `tests/integ/edits_atomicity.rs` 녹색, INSERT/UPDATE/DELETE 모두 round-trip.

### Phase 1.2 — PK 화이트리스트 & RowKey Hash (2d)
- 허용 타입: `int2/4/8`, `numeric`, `uuid`, `text/varchar/citext`, `date`, `timestamp(tz)` (UTC 정규화), `bytea`.
- 비허용 타입: `json/jsonb`, `array`, `range`, `composite`, custom enum → mutation UI **hard-disable** + 상단 배너 "PK whitelist 외 타입 — 편집 비활성".
- Tmp row 는 INSERT 응답 도착까지 read-only + 시각 표식 (회색 + spinner). Optimistic 편집 금지.
- `RowKey = blake3(table_oid_le || pk_canonical_concat).truncate(16 bytes)` — `CellValue::canonical_string()` 정규화 후 join.
- **DoD**: composite PK 테이블에서 RowKey 충돌 0 검증, integration test 4 PK 변종 (single int / composite / identity / citext) + PK 없음 (거부) 통과.

### Phase 1.3 — Cache Invalidation 인프라 (2d)
- LISTEN channel `ferrumgrid_invalidate`, payload `<table_oid>:<phase>` (`pre_drop` / `post_drop` / `schema_change`).
- viewer 측 dispatch: `StateOp::InvalidateTable(oid, InvalidationPhase)`.
- Invalidate dedupe: 동일 프레임 내 중복 도착 시 `HashSet<(Oid, Phase)>` dedupe → 일괄 fetch.
- 발사 순서: coarse → fine (Schemas → Tables → Columns/Indexes/Fks).
- echo timeout: `tokio::time::timeout(Duration::from_secs(5), echo_rx.recv())`. 미수신 시 dirty bit 유지 + DiagnosticsPanel 경고 + 수동 Refresh.
- **ctid opt-in 모드** (PK 화이트리스트 보완): settings 토글 `ferrumgrid.unsafe_ctid` (기본 OFF). 진입 조건 = Phase 1.2 stable. 가드: 모든 mutation 에 `RETURNING ctid` 강제 + affected ≠ 1 즉시 ROLLBACK + DiagnosticsPanel 영구 배너.
- **DoD**: 단위 테스트 (mock notify) 녹색, ctid opt-in 회귀 2 케이스.

### Phase 1.95 — StateOp dispatch matrix 확장 + grid.rs 분리 (2d)

**`StateOp` enum (14 variant)**:
| # | Variant | Trigger |
|---|---------|---------|
| 1 | `SetFocus(CellKey)` | InfoPanel 클릭 / `Drag(End{focus})` (P2 일관, "no-op state" 였던 Drag End 명시화) |
| 2 | `BeginEdit(CellKey)` | KeyEnter on cell |
| 3 | `CommitEdit { key, new: CellValue }` | normalize → bridge |
| 4 | `CancelEdit` | Esc |
| 5 | `BeginSelection(CellKey)` | `Drag(Start{anchor})` |
| 6 | `ExtendSelection(CellKey)` | Shift+Click / `Drag(Move{cursor})` |
| 7 | `EndSelection` | mouse up |
| 8 | `MoveCursor(Direction)` | arrow / tab |
| 9 | `Paste(Vec<Vec<CellValue>>)` | clipboard |
| 10 | `Delete(SelectionRange)` | DEL key |
| 11 | `InvalidateTable(Oid, InvalidationPhase)` | NOTIFY 수신 |
| 12 | `RefreshMetadata(Oid)` | NOTIFY post_drop |
| 13 | `ApplyEdits(Vec<RowEditOp>)` | Save |
| 14 | `RevertEdits` | discard |

**GridInput 분해** (visual feedback 캡슐화):
```rust
enum GridInput {
    Click(CellKey),
    Drag(DragPhase),
    Key(KeyEvent),
    Edit(EditEvent),
    Paste(String),
    InfoPanelClick(CellKey),
}
enum DragPhase {
    Start { anchor: CellKey },
    Move  { cursor: CellKey },
    End   { focus:  CellKey },
}
fn dispatch(state: &State, input: GridInput) -> Option<StateOp>;
```

**View layer 1문장 명시**: View layer (grid render) 는 state getter (`selection_range`, `cursor_cell`, `editing_cell`) 만 읽고 paint, 자체 mutation 0. Selection box / hover hi-light 는 모두 state getter 의 시각화 (별도 `RenderHint` 도입 안 함).

**모듈 분리** (`src/ui/grid.rs` 5240줄 → 8 모듈):
| 모듈 | 목표 ≤ | 흡수/삭제 |
|------|-------|----------|
| `grid/mod.rs` (dispatcher) | 300 | — |
| `grid/render.rs` (paint 만) | 800 | -200 (paint 헬퍼 통합) |
| `grid/selection.rs` | 500 | -150 (중복 hit-rect 헬퍼) |
| `grid/hit_test.rs` | 400 | -200 (좌표 변환 통합) |
| `grid/info_panel.rs` | 500 | -300 (JSON/enum 컨트롤 분리) |
| `grid/paste.rs` | 600 | -100 (clipboard 어댑터 단일화) |
| `grid/footer.rs` | 400 | -50 |
| `grid/tooltips.rs` | 300 | — |
| dead code drop | — | -200 |
| 기타 cleanup | — | -240 |
| **합** | **3800** | **-1440** |

**`src/state.rs` 1025줄 → `src/state/{mod, data_edit, designer, query}.rs`**.
**`src/ui/objects.rs` 2029줄 → `src/ui/objects/{mod, tables, views, functions, roles, backup, automation, model, bi}.rs`** (Phase 2 직전 선결).
**기존 `DataCellEdit`/`DataEditValue` (`src/types.rs:121-156`) 는 *제거*** (deprecate 가 아님) — Phase 1.1 commit 안에서 호출처 일괄 치환.

**Input focus owner**: `state::DataEditState::editing_cell` (Q7 답). `info_panel.rs` 는 read-only consumer, 클릭으로 focus 변경 시 `StateOp::SetFocus(CellKey)` 경유.

**PR reject 기준**: 합 > 4000 OR 어떤 모듈 > 한도 + 10%. 라인 회계 estimate 는 PR review 시 site 측정으로 갱신 의무.

**DoD**: grid.rs/objects.rs 에서 `&mut state` 직접 호출 grep 0, 8 모듈 각 ≤800 + 평균 ≤300, 순환 의존 0.

### Phase 2 — DDL 2-step NOTIFY 통합 + Designer 즉시 실행 (2d)

**Critic Q5 closure**: Phase 2 entry 시 BI 캐시 시나리오 3개 (single-row update / batch / DDL 변경) 문서화 (`.omc/plans/bi-cache-scenarios.md`) — Phase 2 cache key 가 BI 와 동일 모델인지 결정. Phase 4c 는 결정 결과 사용. Authorship = Phase 2 owner (Phase 2 시작 전 작성).

**2-step NOTIFY 시퀀스**:
```
Session A (DDL 실행자)             Session B (viewer)
─────────────────────             ──────────────────
BEGIN; NOTIFY 'pre_drop'; COMMIT; ──>  receive pre_drop
                                       StateOp::InvalidateTable(oid, Pre)
                                       cache invalidate, stale marker
wait 1s (best-effort ack window)
BEGIN; DROP TABLE x CASCADE;
COMMIT;
BEGIN; NOTIFY 'post_drop'; COMMIT; ──> receive post_drop
                                       StateOp::RefreshMetadata(oid)
```
- helper: `Conn::execute_ddl_with_invalidation(sql, table_oid)` 가 위 시퀀스 자동 적용.
- Table Designer (`src/ui/table_designer.rs:801-837`) 의 `apply_ddl` 을 이 helper 경유로 변경. 실패 시 designer 창 유지 + 인라인 진단 (현재처럼 닫고 `last_error` 띄우는 동작 제거).
- View / Function / User(Role) Create/Replace UI 도 같은 helper 재사용 (`src/ui/objects/{views,functions,roles}.rs`).
- Drop 전 `pg_depend` 미리보기 (S1 SQL, dependents > 50 시 cutoff + "더보기").

**DoD**: `tests/integ/drop_cascade.rs` 2 viewer 시나리오 녹색, BI 시나리오 문서 산출, View/Function/Role 생성·수정 후 각 목록 즉시 반영.

### Phase 3 — pg_stat_activity 폴백 + Query 트랜잭션 모드 (2d)

**dangling tx 정책** (Query 탭 명시 BEGIN 보호):
- 진입 probe: `SELECT has_table_privilege('pg_catalog.pg_stat_activity', 'SELECT')`.
- **(a) 권한 있음**: 5s polling 활성, 30s 토스트 경고, 60s 강제 ROLLBACK + DiagnosticsPanel 알림.
- **(b) 권한 없음**: client-side timer (transaction 시작 시각 기록), 30s/60s 동일 적용 + DiagnosticsPanel 영구 배너 "server-side dangling tx detection unavailable — using client-side timer".
- **(c) probe 자체 실패**: client-side timer + 배너 + tracing WARN.
- 탭 close / 앱 quit → 즉시 ROLLBACK.
- Query 명시 BEGIN 동안 Data 탭 mutation UI **hard-disable** + 상단 배너 (P4 single writer 일관, connection_task 단일 task 유지 — N:1 분리는 ROI 음수).

**DoD**: `tests/integ/dangling_tx.rs` a/b/c × 30s/60s = 6 케이스 녹색.

### Phase 4a — Backup + BackupInfoV1 schema 도입 + legacy deprecation (2d)
```rust
pub enum BackupFormat { Custom, Plain, Tar }
pub enum BackupStatus {
    Idle,
    InProgress { pct: u8 },
    Done,
    Failed { error: String },
}
pub struct BackupInfoV1 {
    pub format: BackupFormat,
    pub encoding: String,
    pub status: BackupStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub bytes_total: Option<u64>,
    pub bytes_done: u64,
    pub eta_seconds: Option<u64>,
}
pub fn current_backup_info() -> BackupInfoV1 { /* ... */ }
```
- `legacy_backup_format_*` 즉시 `#[deprecated(note = "use BackupInfoV1 / current_backup_info() — remove in Phase 4c entry; Phase 4b MUST NOT introduce new call sites; CI grep gate active in 4b PRs")]`.
- DiagnosticsPanel 에 `MutationSource::Backup` 태그 합류.

**DoD**: schema 컴파일 + `#[deprecated]` warning 가시 + 4b 사용처 시뮬레이션 PR-time review.

### Phase 4b — Automation + DiagnosticsPanel 통합 (2d)
- 즉시 실행 + 예약 (tokio::spawn + interval) 기능. cancel/pause 가능, 다음 실행 시각 panel 노출.
- DiagnosticsPanel 의 backup 정보 표시는 **`current_backup_info()` 만 사용**, `legacy_backup_format_*` 신규 호출 0.
- CI grep 게이트 active (Phase 4b PR 부터): `! grep -Rn 'legacy_backup_format_' src/ui/diagnostics_panel.rs && exit 0 || exit 1`.
- 채널 5종 표면화: echo_timeout / dangling_tx / cache_stale / backup_error / mutation_diagnostic.

**DoD**: legacy 호출 grep 0, 각 채널 표면화 e2e 1개씩.

### Phase 4c — BI + legacy 물리 삭제 (1d)
- 그룹/피벗/차트 카드 (`src/ui/objects/bi.rs`). `QueryResult` 캐시 모델 변경 (Phase 2 entry 결정 결과 적용).
- `legacy_backup_format_*` 코드 제거.

**DoD**: `cargo build` 통과, dead_code lint 0, BI 시나리오 3개 모두 동작.

---

## 6. Bridge / State 변경 명세

```rust
// src/state/data_edit.rs (신규)
pub struct RowKey([u8; 16]);  // blake3(table_oid_le || pk_canonical_concat)[..16]
pub enum RowKeyKind { Pk(RowKey), Tmp(Uuid), Ctid(String) }
pub enum RowEditOp {
    Insert { tmp_id: Uuid, values: Vec<(ColName, SqlValue)> },
    Update { key: RowKeyKind, set: Vec<(ColName, SqlValue)> },
    Delete { key: RowKeyKind },
}

// In-memory (mutation-friendly)
pub struct DataEditState {
    pub updates: HashMap<(RowKeyKind, ColIdx), SqlValue>,
    pub inserts: Vec<TmpRow>,
    pub deletes: HashSet<RowKeyKind>,
    // ...
}
// Apply 시점: insert → updates → deletes 순서로 Vec<RowEditOp> 정규화.

// src/db/bridge.rs
pub enum Invalidate {
    Schemas, Tables(SchemaName), Columns(QualName),
    Indexes(QualName), Fks(QualName),
    Views(SchemaName), Functions(SchemaName), Roles,
}
pub enum InvalidationPhase { Pre, Post, SchemaChange }
pub enum MutationSource { Ddl, Data, Query, Automation, Backup }
pub struct MutationDiagnostic {
    pub source: MutationSource,
    pub op_idx: usize,
    pub sql: String,
    pub sqlstate: Option<String>,
    pub error: Option<DbError>,
    pub jump: Option<SourceLoc>,
}
pub struct MutationOutcome {
    pub affected: Vec<RowEcho>,
    pub inserted_keys: Vec<(Uuid, RowKey)>,
    pub diagnostics: Vec<MutationDiagnostic>,
    pub invalidates: Vec<Invalidate>,
}

pub enum DbCommand {
    ApplyDataEdits { schema: String, table: String, ops: Vec<RowEditOp> },
    ApplyDdl { sql: String, table_oid: Option<u32>, invalidations: Vec<Invalidate> },
    // ...기존
}
pub enum DbResponse {
    DataEditsApplied(MutationOutcome),
    DdlApplied { invalidated: Vec<Invalidate>, diagnostic: MutationDiagnostic },
    MutationFailed { diagnostic: MutationDiagnostic },
    // ...
}
```

**호출 site 정정**:
- 정의: `src/db/queries.rs:142` — `apply_data_edits` 시그니처 변경.
- 호출: `src/db/bridge.rs:528-535` — `ConnCommand::ApplyDataEdits` 처리 (begin/commit 추가).
- 영향 범위: **2 site** (정의 1 + 호출 1).

---

## 7. In-memory ↔ Wire 정규화 규칙

| In-memory `CellValue` | Wire (canonical_string) |
|----------------------|------------------------|
| `Null` | `"\0NULL"` (sentinel, 빈 문자열 ≠ Null) |
| `Bool(b)` | `"t"` / `"f"` |
| `Int(i)` | `i.to_string()` (trim, leading zero 제거) |
| `Float(f)` if `f.is_nan()` | `"\0NaN"` (고정) |
| `Float(±0.0)` | `"0"` (sign drop) |
| `Float(f)` else | `format!("{:?}", f)` (Rust round-trip, shortest repr) |
| `Text(s)` | UTF-8 NFC normalize |
| `Json(v)` | minified, 키 sort |
| `Bytes(b)` | `format!("\\x{}", hex(b))` (lowercase) |
| `Timestamp(t)` | ISO 8601, UTC 변환 (timezone offset 정규화) |

→ `CmpCell { canonical: Cow<'_, str> }` 가 이 표 기반.

---

## 8. PK 화이트리스트 & RowKey Hash 전략

- **허용**: `int2/4/8`, `numeric`, `uuid`, `text/varchar/citext`, `date`, `timestamp(tz)`, `bytea`.
- **비허용** (편집 비활성): `json/jsonb`, `array`, `range`, `composite`, custom enum.
- **PK 후보 우선순위**:
  1. `pg_index.indisprimary`
  2. fallback: `indisunique AND NOT indisexpression AND NOT indisdeferrable`
  3. ctid opt-in (Phase 1.3, `ferrumgrid.unsafe_ctid` flag, `RETURNING ctid` 강제)
- `RowKey = blake3(table_oid_le || pk_canonical_concat).truncate(16 bytes)`. 충돌 시 full 32B fallback + tracing warn.

---

## 9. 테스트 매트릭스

| 레이어 | 위치 | 케이스 |
|--------|------|--------|
| Compile-fail (P3 게이트) | `tests/compile_fail/cmp_cell.rs` | `CellValue == CellValue` 직접 호출 시 컴파일 실패 |
| Property test | `tests/prop/cmp_cell_nan_safe.rs` | NaN 포함 random vec 에서 `CmpCell` reflexive |
| Bench | `benches/cmp_cell.rs` | A (NaN-unsafe ref) + B (NaN-safe naive, ceiling 대상) |
| Unit | `src/state/data_edit.rs` inline | `RowEditOp` squash 정합, in-mem ↔ wire 정규화 |
| Unit | `src/types.rs` inline (proptest) | `canonical_string()` round-trip |
| Integration (postgres_seed) | `tests/integ/edits_atomicity.rs` | 중간 실패 시 전체 롤백 |
| Integration | `tests/integ/pk_strategy.rs` | PK 4종 (single int / composite / identity / citext / uuid) + 없음 거부 |
| Integration | `tests/integ/ctid_opt_in.rs` | flag ON/OFF 2 케이스 |
| Integration | `tests/integ/drop_cascade.rs` | 2 viewer pre/post 모두 수신, dependents > 50 cutoff |
| Integration | `tests/integ/dangling_tx.rs` | a/b/c × 30s/60s = 6 케이스 |
| Integration | `tests/integ/cache_invalidation.rs` | dedupe + coarse→fine + 5s timeout |
| Integration | `tests/integ/backup_info_v1_roundtrip.rs` | schema 직렬화 round-trip |
| Build | `tests/build/legacy_deprecation.rs` | 4a 빌드 시 `#[deprecated]` warning 가시, 4b PR 에서 grep 0 |
| State surface | `tests/grid_state_surface.rs` | dispatch matrix — selection / hit-test / edit start·commit / paste / info_panel (편집 정보, dirty marker, PK 표시, null 표시, input focus 표시 5 케이스) |
| Tracing assertion 헬퍼 | `tests/support/tracing_assert.rs` | `LayerCollector` + `EventAssertion` API (`field_non_empty` / `field_eq` / `field_present`) |
| (Phase 5+) E2E | `egui_kittest` info_panel render 회귀 | 별도 결정 |

---

## 10. 관측성 & DiagnosticsPanel 정책

- **tracing target**: `ferrumgrid::mutation`, `ferrumgrid::ddl`, `ferrumgrid::cache`.
- **필드 스키마**: `conn_id`, `source` (MutationSource), `op_idx`, `sql_hash`, `sqlstate`, `dur_ms`, `outcome`.
- **DiagnosticsPanel 단일 진입점** + `source` 태그 필터 + jump-to-source 액션 (sql line → designer/grid 위치).
- **표면화 채널 5종**: echo_timeout / dangling_tx / cache_stale / backup_error / mutation_diagnostic.
- **재시도 액션**: 마지막 `MutationDiagnostic` 의 `sql` 을 query editor 로 로드.
- 채널 capacity 256 (`src/db/bridge.rs:166-167`) ≫ per-batch op 수 → per-op `MutationDiagnostic` 안전.

---

## 11. Risk 표

| ID | Risk | 확률 | 영향 | Mitigation |
|----|------|-----|------|-----------|
| R1 | `CmpCell` borrow lifetime 폭증 | 중 | 중 | `Cow<'a, str>` 유지, owned 변환은 hash key 한정 |
| R2 | 2-step NOTIFY 중간에 DDL 실패 → post_drop 미발송 | 저 | 중 | **별도 cleanup task** (Drop async 한계 회피) — Connection 레벨 background task 가 fail_drop NOTIFY 보장. Drop trait 의존 안 함 |
| R3 | viewer ack 1s 부족 | 저 | 저 | best-effort, stale marker 가 fallback. adaptive ack window (slow viewer 감지 시 연장) — Follow-up |
| R4 | RowKey blake3 truncate 16B 충돌 | 극저 | 중 | full 32B fallback, 모니터링 |
| R5 | `pg_stat_activity` 권한 부족 비율 과다 | 중 | 저 | a/b/c 폴백 매트릭스 + 영구 배너 + docs |
| R6a | legacy_* grep 0 위반 PR 머지 | 저 | 저 | CI grep + `#[deprecated]` warning |
| R6b | adapter rename 누락 | 저 | 중 | 4a PR template 체크리스트 |
| R6c | 4c 전 legacy 사용 발견 | 저 | 저 | 4b DoD 가 grep 0 강제 |
| R7 | `BackupInfoV1` 8 필드 부족 발견 → 4a 롤백 | 저 | 중 | 4a PR review 시 4b 사용처 시뮬레이션 의무화 |
| R8a | Option B (`CmpCell`) bench A 대비 <3x slowdown | 고 | 저 | 무조건 채택 |
| R8b | `CmpCell` bench A 대비 3-10x slowdown | 중 | 중 | 채택 + Follow-up 등록 |
| R8c | `CmpCell` bench A 대비 ≥10x slowdown | 저 | 고 | Phase 1.0 entry 게이트 재논의, 모델 2중화 비용 vs 마이크로 perf 재평가 |
| R9 | 단일 메인테이너 컨텍스트 스위치로 floor 20d 미달 | 중 | 저 | actual 20-25d range 명시, phase 별 actual 측정 회고 |

---

## 12. Open Questions

**잔존 0.** (모든 Q1~Q9 본 plan 에서 closure)

---

## 13. ADR

### ADR-1: `CmpCell<'_>` newtype + `CellValue: PartialEq` `#[cfg(test)]` 격리
- **Decision**: prod 빌드에서 `CellValue` 직접 비교 차단, `CmpCell` 경유만 허용.
- **Drivers**: P3 (type-driven invariants), grep 게이트 escape hatch 재발 방지.
- **Alternatives**: grep CI rule (소프트), `clippy::disallowed_methods` (false positive).
- **Why**: compile-time hard gate 가 가장 강함. escape hatch 0.
- **Consequences**: borrow lifetime 약간 증가 (R1), 테스트 utility 격리 필요.
- **Follow-ups**: `CmpCell::Hash` stability proptest, 3-10x slowdown 시 monitoring.

### ADR-2: 2-step NOTIFY ordering (pre_drop → ack → DDL → post_drop)
- **Decision**: advisory_xact_lock 폐기, explicit 2-transaction sequence 채택.
- **Drivers**: P4 (DDL 안전성), S1 차단.
- **Alternatives**: advisory_xact_lock + 동시 NOTIFY (ordering 불가능 — commit 시 lock 해제와 notify flush 동시), single NOTIFY (race).
- **Why**: lock 해제와 notify flush 가 같은 commit 시점이라 ordering 불가능. 2-step 만 race-free.
- **Consequences**: DDL latency +1s (ack window), viewer 측 stale marker UX 추가.
- **Follow-ups**: ack window adaptive 화 (slow viewer 감지 시 연장), R2 cleanup task 구현 (Drop async 한계 회피).

### ADR-3: Phase 4a PR scope = `BackupInfoV1` schema + legacy `#[deprecated]` 동시
- **Decision**: scaffold-only 4a 거부, 4a 에서 schema 전체 + deprecation 마킹.
- **Drivers**: P5 (revertibility), 4b 사용처 마이그레이션이 schema 의존.
- **Alternatives**: 4a scaffold → 4b schema → 4c migrate (3-step) — schema/사용처 분리 위반 위험.
- **Why**: schema 가 작고 (8 필드), 4b 시뮬레이션이 PR review 시점에 가능.
- **Consequences**: 4a PR LOC +200, R7 (필드 부족) 위험.
- **Follow-ups**: 4a PR template 에 "4b 사용처 시뮬레이션" 체크박스.

### ADR-4: 일정 표기 통일 (floor 20d / actual 20~25d)
- **Decision**: "floor 13.5d / 17d" 잔재 완전 제거.
- **Drivers**: 산술합 정확성, buffer 명시화.
- **Why**: 산술합 = 1+2+2+2+2+2+2+2+2+2+1 = 20d. Phase 1.3 → 1.95 직렬 의존성 (1.3 의 LISTEN/NOTIFY infra 가 1.95 의 dispatch 변형 정의) 으로 부분 병렬 가정 폐기. context switch + PR review + 회귀 수정 buffer = +0~5d.
- **Consequences**: stakeholder 기대 정렬, 단일 메인테이너 환경에서 actual upper 25d 가능.
- **Follow-ups**: phase 별 actual 측정, 회고 시 buffer 비율 기록.

### ADR-5: In-memory mutation-friendly state ⊕ wire 정규화 op 시퀀스
- **Decision**: in-mem (`HashMap` + `Vec` + `HashSet`) ↔ wire (`Vec<RowEditOp>` 정규화) 분리.
- **Drivers**: 데이터 안전성, 확장성.
- **Alternatives**: Option A (incremental cells + flag), Option C (server-side cursor).
- **Why**: squash O(1) per cell, tmp row 명시 추적, server-generated PK 회수 가능.
- **Consequences**: 정규화 비용 O(N) per apply (수용), tmp row UX 보수적 (read-only until echo).
- **Follow-ups**: criterion 마이크로벤치 (Phase 0 종료 시) 로 정성 추정 정량화.

---

## 14. Relevant files

- `src/state.rs` (1025줄) — `DataEditState` 신규/삭제 행 추적 구조 확장, Phase 1.95 에서 `src/state/{mod, data_edit, designer, query}.rs` 분리
- `src/types.rs:121-156` — `CellValue` (PartialEq `#[cfg(test)]` 격리), `DataCellEdit`/`DataEditValue` 제거, `CmpCell<'_>` 신규
- `src/db/queries.rs:142` — `apply_data_edits` 시그니처 `&Client` → `&mut Transaction<'_>`, INSERT/UPDATE/DELETE SQL 생성
- `src/db/bridge.rs:166-208, 528-535` — `DbCommand` / `DbResponse` 확장 (`ApplyDdl`, `Invalidate`, `MutationOutcome`), 호출 site begin/commit, channel capacity 256 유지
- `src/db/metadata.rs` — 변경 후 목록 재조회, `pg_depend` 미리보기 SQL
- `src/ui/grid.rs` (5240줄) — Phase 1.95 에서 `src/ui/grid/{mod, render, selection, hit_test, info_panel, paste, footer, tooltips}.rs` 분리, dispatch matrix + `StateOp` 14 variant
- `src/ui/objects.rs` (2029줄) — Phase 1.95 에서 `src/ui/objects/{mod, tables, views, functions, roles, backup, automation, model, bi}.rs` 분리, View/Function/Role Create/Replace UI
- `src/ui/table_designer.rs:801-837` — `apply_ddl` 을 `Conn::execute_ddl_with_invalidation` 경유로 변경, 실패 시 designer 창 유지
- `tests/postgres_seed.rs` — 통합 테스트 확장 기반
- `tests/support/tracing_assert.rs` (신규) — `LayerCollector` + `EventAssertion` 헬퍼
- `tests/compile_fail/cmp_cell.rs` (신규) — P3 type-system 게이트 검증
- `tests/prop/cmp_cell_nan_safe.rs` (신규) — NaN proptest
- `tests/integ/{edits_atomicity, pk_strategy, ctid_opt_in, drop_cascade, dangling_tx, cache_invalidation, backup_info_v1_roundtrip}.rs` (신규) — Phase 별 통합 테스트
- `tests/grid_state_surface.rs` (신규) — dispatch matrix 회귀 (info_panel 5 케이스 포함)
- `benches/cmp_cell.rs` (신규) — A/B baseline 측정
- `.omc/plans/bi-cache-scenarios.md` (신규, Phase 2 entry 산출) — BI 캐시 invalidation 시나리오 3개

---

## 15. Verification

1. **Phase 1.0**: `cargo build --release` 통과 (`CellValue: PartialEq` `#[cfg(test)]` 격리 검증), 모든 비교 site `cmp_view()` 경유, NaN proptest 녹색, bench A/B evidence.
2. **Phase 1.1**: Data 탭에서 INSERT/UPDATE/DELETE 각각 저장 후 DB 반영 확인, 중간 실패 시 전체 롤백, INSERT RETURNING PK 회수.
3. **Phase 1.2**: PK 4종 + 없음 거부 통합 테스트, 비허용 타입 시 mutation UI hard-disable + 배너.
4. **Phase 1.3**: ctid opt-in 토글, cache invalidation echo timeout 5s, dirty bit 폴백.
5. **Phase 1.95**: grid.rs 8 모듈 각 ≤800 + 평균 ≤300, 직접 `&mut state` grep 0, dispatch matrix 회귀 테스트.
6. **Phase 2**: Table Designer DDL 즉시 실행, 2 viewer 에서 DROP CASCADE pre/post 모두 수신, View/Function/Role 생성·수정 후 각 목록 즉시 반영.
7. **Phase 3**: pg_stat_activity a/b/c 분기, idle 60s 강제 ROLLBACK, Query 명시 BEGIN 동안 Data UI hard-disable.
8. **Phase 4a**: `BackupInfoV1` schema 컴파일, `#[deprecated]` warning 가시.
9. **Phase 4b**: `legacy_backup_format_*` grep 0, DiagnosticsPanel 5 채널 표면화.
10. **Phase 4c**: legacy 코드 물리 삭제, `cargo build` clean, dead_code 0, BI 시나리오 3개 동작.
11. **권한 부족 / 제약조건 위반 / enum/uuid/null / 1000+ 행 페이징** 시 메시지와 상태 복구 일관성 확인.

---

## 16. Decisions (요약)

- **포함**: MainView 실동작 완성 (조회 + 풀 mutation 루프 with type-system 안전 가드 + ordering-safe DDL).
- **제외 (초기)**: 고급 BI 대시보드 (Phase 4c 는 minimal 카드만), 외부 스케줄 오케스트레이션, 복구 마법사, 진짜 E2E (egui_kittest) — 모두 Phase 5+.
- **우선순위**: Data CRUD (Phase 1.0~1.2) → Designer 즉시 실행 (Phase 2) → Object 관리 (Phase 2 helper 재사용) → 실사용 강화 (Phase 3, 4).
- **단일 메인테이너 가정**: floor 20d, actual 20~25d. Phase 별 PR 단위 revert 가능.
