//! Grid input dispatch types.
//!
//! Plan v7 Phase 1.95 / §5 / ADR-1:
//! - **`GridInput`**: 사용자 입력 이벤트 (Click / Drag(phase) / Key / Edit /
//!   Paste / InfoPanelClick).
//! - **`DragPhase`**: drag 의 진행 단계 (Start/Move/End) — 단일 `Drag{from,to}`
//!   대신 phase 분리로 visual feedback (selection box, hover highlight) 이
//!   dispatch 외부로 누출되지 않게 함.
//! - **`StateOp`**: state 에 적용 가능한 14 가지 작업 — UI mutation 의 단일
//!   진입점 (P2 Dispatch-only mutation 강제).
//! - **`dispatch`**: `GridInput` → `StateOp` 매핑 함수. 본체 구현은 Phase 1.95c
//!   에서 실제 grid.rs 분리와 함께 채워진다 (현 단계는 시그니처 + matrix 단위
//!   테스트만).

use crate::db::invalidate::InvalidationPhase;
use crate::state::AppState;

/// Grid 의 cell 좌표 (row, column 인덱스).
///
/// 결정성을 위해 결정적 i32 (음수는 헤더 등 sentinel 위치).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellKey {
    pub row: i32,
    pub col: i32,
}

/// Cursor 이동 방향 (arrow / tab keys).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Drag 의 진행 단계.
///
/// Plan v7 ADR-1 / Critic v5: 단일 `Drag{from,to}` enum 으로는 *진행 중* 의
/// visual feedback (hover hi-light, selection box 그리기) 이 dispatch 외부로
/// 누출된다. 3 phase 로 분리하여 모든 시각 효과가 `StateOp` 의 결과로 표현되게
/// 한다.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    /// mouse-down at anchor cell — selection 시작.
    Start { anchor: CellKey },
    /// mouse-move during drag — selection box 갱신.
    Move { cursor: CellKey },
    /// mouse-up — focus 확정.
    End { focus: CellKey },
}

/// Edit 진입/종료 이벤트.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EditEvent {
    /// Enter / double-click on cell.
    Begin,
    /// Enter (commit) / Tab — 새 값과 함께 commit.
    Commit { new_value: String },
    /// Esc — rollback.
    Cancel,
}

/// Sort direction (column 헤더 클릭).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// 사용자 입력 이벤트의 정규화된 표현.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum GridInput {
    /// 단일 셀 클릭.
    Click(CellKey),
    /// Drag 의 진행 단계 (Start/Move/End).
    Drag(DragPhase),
    /// Key event (cursor / shortcut).
    Key(Direction),
    /// Edit 진입/commit/cancel.
    Edit(EditEvent),
    /// 클립보드 paste — text 는 grid 가 row × column 으로 분해.
    Paste(String),
    /// InfoPanel 의 cell 표시 클릭 → grid focus 이동.
    InfoPanelClick(CellKey),
    /// 우클릭 컨텍스트 메뉴.
    RightClick(CellKey),
    /// Column 헤더 클릭 (sort).
    SortHeader { col: i32, direction: SortDirection },
    /// Focus loss (다른 위젯으로 이동).
    FocusLost,
}

/// State 에 적용 가능한 작업.
///
/// Plan v7 Phase 1.95 — UI mutation 의 단일 진입점. 14 variants:
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StateOp {
    /// 1. focus cell 변경 (InfoPanel 클릭, Drag End 등).
    SetFocus(CellKey),
    /// 2. selection 시작 (단일 셀).
    BeginSelection(CellKey),
    /// 3. selection range 확장 (drag move, shift+click).
    ExtendSelection(CellKey),
    /// 4. selection 종료 (mouse up, focus 확정).
    EndSelection,
    /// 5. cursor 이동 (arrow / tab).
    MoveCursor(Direction),
    /// 6. cell edit 진입.
    BeginEdit(CellKey),
    /// 7. edit commit (DB 반영 전 in-memory 값 갱신).
    CommitEdit { new_value: String },
    /// 8. edit cancel (rollback).
    CancelEdit,
    /// 9. clipboard paste 적용.
    Paste(String),
    /// 10. selection 삭제 (Delete key).
    Delete,
    /// 11. cache invalidation (LISTEN/NOTIFY 수신 → table 재조회).
    InvalidateTable {
        table_oid: u32,
        phase: InvalidationPhase,
    },
    /// 12. metadata refresh (post_drop NOTIFY → re-introspect).
    RefreshMetadata { table_oid: u32 },
    /// 13. 전체 dirty edits commit (Apply 버튼).
    ApplyEdits,
    /// 14. dirty edits 전체 revert (Discard 버튼).
    RevertEdits,
}

/// `GridInput` → `StateOp` 매핑.
///
/// **본 phase (1.95a) 에서는 시그니처만 고정**. 실제 매핑 본체는 1.95c 에서
/// grid.rs 분리 + state surface 통합과 함께 채워진다. 현재는 placeholder 로
/// `None` 을 반환하되, dispatch matrix 단위 테스트는 본 함수의 *대체 구현*
/// (`dispatch_matrix_lookup`) 으로 정합성을 검증한다.
///
/// **1.95c 구현 가이드**: 본 함수의 본체는 반드시 `dispatch_matrix_lookup`
/// 을 *내부 호출* 한 뒤, state-context 가 필요한 매핑 (예: `EditEvent::Begin`
/// → `BeginEdit(state.cursor_cell())`, `RightClick` → 컨텍스트 메뉴 좌표,
/// `SortHeader` → query 재발사) 을 *후처리* 로 추가해야 한다. 두 함수를
/// 독립 구현하면 정책 drift 발생 위험.
#[allow(dead_code)]
pub fn dispatch(input: GridInput, state: &AppState) -> Option<StateOp> {
    match &input {
        GridInput::Edit(EditEvent::Begin) => {
            let (row, col) = state.data_edit.selected_cell?;
            Some(StateOp::BeginEdit(CellKey {
                row: row as i32,
                col: col as i32,
            }))
        }
        _ => dispatch_matrix_lookup(&input),
    }
}

/// Plan v7 §6 Phase 1.95 의 dispatch matrix 의 *결정 함수*.
///
/// 이 함수는 `GridInput` → `StateOp` 의 *정책* 만 정의하고 state-free 다.
/// `dispatch` 본체는 1.95c 에서 grid.rs 분리 시 본 함수 + state mutation 으로
/// 합쳐진다.
#[allow(dead_code)]
pub fn dispatch_matrix_lookup(input: &GridInput) -> Option<StateOp> {
    match input {
        GridInput::Click(k) => Some(StateOp::BeginSelection(*k)),
        GridInput::Drag(DragPhase::Start { anchor }) => Some(StateOp::BeginSelection(*anchor)),
        GridInput::Drag(DragPhase::Move { cursor }) => Some(StateOp::ExtendSelection(*cursor)),
        GridInput::Drag(DragPhase::End { focus }) => Some(StateOp::SetFocus(*focus)),
        GridInput::Key(dir) => Some(StateOp::MoveCursor(*dir)),
        GridInput::Edit(EditEvent::Begin) => None, // 별도 BeginEdit(target_cell) 가 필요 — 1.95c 에서 wire
        GridInput::Edit(EditEvent::Commit { new_value }) => Some(StateOp::CommitEdit {
            new_value: new_value.clone(),
        }),
        GridInput::Edit(EditEvent::Cancel) => Some(StateOp::CancelEdit),
        GridInput::Paste(text) => Some(StateOp::Paste(text.clone())),
        GridInput::InfoPanelClick(k) => Some(StateOp::SetFocus(*k)),
        GridInput::RightClick(_) => None, // 컨텍스트 메뉴는 별도 표면 — 1.95c
        GridInput::SortHeader { .. } => None, // sort 는 query 재발사 — 1.95c
        GridInput::FocusLost => Some(StateOp::EndSelection),
    }
}

/// Phase 1.95c integration 시 RowKey 기반 외부 invalidation 을 dispatch 하는
/// 헬퍼 — LISTEN/NOTIFY 수신 site (Phase 1.95c 의 connection_task select! 분기)
/// 에서 호출.
#[allow(dead_code)]
pub fn invalidation_to_state_op(table_oid: u32, phase: InvalidationPhase) -> StateOp {
    StateOp::InvalidateTable { table_oid, phase }
}

/// `StateOp` interpreter — production caller 가 dispatch() 결과를 state mutation
/// 으로 적용. 각 variant 별 단일 진입점.
///
/// 본 함수는 dispatch() 와 짝을 이루어 `dispatch → apply_state_op` chain 으로
/// grid input 처리 을 단일 진입점화한다. 미구현 variant 는 no-op (placeholder).
pub fn apply_state_op(state: &mut crate::state::AppState, op: StateOp) {
    match op {
        StateOp::SetFocus(k) => {
            state.data_edit.selected_cell = Some((k.row as usize, k.col as usize));
            state.show_info_panel = true;
        }
        StateOp::BeginSelection(k) => {
            state.data_edit.selected_cell = Some((k.row as usize, k.col as usize));
            state.data_edit.editing_cell = None;
            state.show_info_panel = true;
        }
        StateOp::ExtendSelection(_k) => {
            // range selection — Phase 후속 iteration. selected_cell anchor 유지.
        }
        StateOp::EndSelection => {
            // Drag mouse-up — selection 확정, 추가 mutation 없음.
        }
        StateOp::MoveCursor(dir) => {
            let Some(result) = state.current_result.as_ref() else {
                return;
            };
            let row_count = result.rows.len();
            let col_count = result.columns.len();
            if row_count == 0 || col_count == 0 {
                return;
            }
            let (mut row, mut col) = state.data_edit.selected_cell.unwrap_or((0, 0));
            match dir {
                Direction::Up => row = row.saturating_sub(1),
                Direction::Down => row = (row + 1).min(row_count - 1),
                Direction::Left => col = col.saturating_sub(1),
                Direction::Right => col = (col + 1).min(col_count - 1),
                Direction::PageUp => row = row.saturating_sub(20),
                Direction::PageDown => row = (row + 20).min(row_count - 1),
                Direction::Home => col = 0,
                Direction::End => col = col_count - 1,
            }
            state.data_edit.selected_cell = Some((row, col));
        }
        StateOp::BeginEdit(k) => {
            state.data_edit.editing_cell = Some((k.row as usize, k.col as usize));
        }
        StateOp::CommitEdit { new_value: _ } => {
            // 실제 edit commit 은 data_edit.cells HashMap entry 갱신. UI 가 직접 호출.
            state.data_edit.editing_cell = None;
        }
        StateOp::CancelEdit => {
            state.data_edit.editing_cell = None;
        }
        StateOp::Paste(text) => {
            // TSV 형식 — 행은 \n, 열은 \t. selected_cell 부터 우측+아래로 EditableCell.value 갱신.
            // US-I2 — 미등록 셀은 ensure_data_edit_cell_from_result 로 on-demand 등록.
            let (row_count, col_count) = match state.current_result.as_ref() {
                Some(result) => (result.rows.len(), result.columns.len()),
                None => return,
            };
            if row_count == 0 || col_count == 0 {
                return;
            }
            let (start_row, start_col) = state.data_edit.selected_cell.unwrap_or((0, 0));
            for (r_off, line) in text.split('\n').enumerate() {
                let row = start_row + r_off;
                if row >= row_count {
                    break;
                }
                for (c_off, value) in line.split('\t').enumerate() {
                    let col = start_col + c_off;
                    if col >= col_count {
                        break;
                    }
                    crate::ui::grid::data_ops::ensure_data_edit_cell_from_result(state, row, col);
                    if let Some(edit) = state.data_edit.cells.get_mut(&(row, col)) {
                        edit.value = value.to_string();
                        edit.is_null = false;
                    }
                }
            }
        }
        StateOp::Delete => {
            // 선택된 셀의 EditableCell.is_null = true 마킹.
            // US-I2 — 미등록 셀은 ensure_data_edit_cell_from_result 로 on-demand 등록.
            if let Some((row, col)) = state.data_edit.selected_cell {
                crate::ui::grid::data_ops::ensure_data_edit_cell_from_result(state, row, col);
                if let Some(edit) = state.data_edit.cells.get_mut(&(row, col)) {
                    edit.is_null = true;
                }
            }
        }
        StateOp::InvalidateTable { table_oid, phase } => {
            // US-I3 — Pre phase: stale marker (banner). Post/SchemaChange: bridge 필요 → with_bridge 만 처리.
            if matches!(phase, InvalidationPhase::Pre) {
                state.last_error =
                    Some("Schema 변경 진행 중 — 데이터 새로고침 대기".to_string());
                // US-M1 — pending tracker insert. table_oid==0 (legacy/SchemaChange-only)
                // 은 oid 매칭 불가 → 추적 제외.
                if table_oid != 0 {
                    state
                        .pending_invalidations
                        .entry(table_oid)
                        .or_insert_with(std::time::Instant::now);
                }
            }
        }
        StateOp::RefreshMetadata { .. } => {
            // bridge 필요 — apply_state_op_with_bridge 만 처리. 본 path 는 noop.
        }
        StateOp::ApplyEdits => {
            // bridge 가 필요 — apply_state_op_with_bridge() 사용. 본 placeholder
            // path 는 caller 가 bridge 없이 호출했음을 의미하므로 noop.
        }
        StateOp::RevertEdits => {
            crate::ui::grid::data_ops::revert_data_edits(state);
        }
    }
}

/// `apply_state_op` + `DbBridge` 의존성 — `ApplyEdits` 등 bridge 가 필요한
/// variant 도 처리한다.
///
/// **단일 mutation entry point 정책 (US-G2 / Plan v7 §10)** — 모든 production
/// caller (Apply 버튼, Cancel 버튼, paste 핸들러 등) 는 inline state mutation
/// 대신 본 함수 (또는 bridge 불필요 시 `apply_state_op`) 를 경유해야 한다.
/// 단일 진입점을 통해 dispatch matrix 가 모든 grid mutation 을 관찰 가능 +
/// 향후 logging / undo / replay 가 일관되게 적용된다.
pub fn apply_state_op_with_bridge(
    state: &mut crate::state::AppState,
    op: StateOp,
    bridge: &crate::db::bridge::DbBridge,
) {
    match op {
        StateOp::ApplyEdits => {
            let Some(summary) = crate::ui::grid::data_ops::data_edit_summary(state) else {
                return;
            };
            match crate::ui::grid::data_ops::build_data_edits(state) {
                Ok(edits) => {
                    state.data_edit.applying = true;
                    state.last_error = None;
                    bridge.send(crate::db::bridge::DbCommand::ApplyDataEdits {
                        conn_id: summary.conn_id,
                        edits,
                    });
                }
                Err(err) => {
                    state.last_error = Some(err);
                }
            }
        }
        StateOp::InvalidateTable { table_oid, phase } => {
            // US-I3 — Pre 는 apply_state_op 가 last_error 배너 처리, Post/SchemaChange 는 reload.
            match phase {
                InvalidationPhase::Pre => {
                    apply_state_op(state, StateOp::InvalidateTable { table_oid, phase });
                }
                InvalidationPhase::Post | InvalidationPhase::SchemaChange => {
                    // US-M1 — Post 도착 → pending tracker 정리 (echo_warned 동시 제거).
                    if table_oid != 0 {
                        state.pending_invalidations.remove(&table_oid);
                        state.echo_warned.remove(&table_oid);
                    }
                    // US-K2 — table_oid 매칭: active_data_source 의 oid 와 일치할 때만 reload.
                    // table_oid == 0 은 매칭 무시 (legacy fallback — 무조건 reload).
                    let should_reload = if table_oid == 0 {
                        true
                    } else if let Some(source) = state.active_data_source() {
                        state
                            .connections
                            .get(&source.conn_id)
                            .and_then(|conn| conn.tables.get(&source.schema))
                            .and_then(|tables| tables.iter().find(|t| t.name == source.table))
                            .and_then(|t| t.oid)
                            .map(|active_oid| active_oid == table_oid)
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if should_reload {
                        state.last_error = None;
                        crate::ui::grid::data_ops::reload_data_source(state, bridge);
                    }
                }
            }
        }
        StateOp::RefreshMetadata { table_oid: _ } => {
            // US-I3 — active_data_source 의 columns + foreign_keys 강제 재조회.
            if let Some(source) = state.active_data_source() {
                let conn_id = source.conn_id;
                let schema = source.schema.clone();
                let table = source.table.clone();
                // Invalidate caches 먼저 (request_* 은 cache miss 일 때만 fire)
                if let Some(conn) = state.connections.get_mut(&conn_id) {
                    conn.columns.remove(&(schema.clone(), table.clone()));
                    conn.foreign_keys.remove(&schema);
                }
                crate::ui::grid::request_table_columns_for_data(
                    state, bridge, conn_id, &schema, &table,
                );
                crate::ui::grid::request_foreign_keys_for_schema(state, bridge, conn_id, &schema);
            }
        }
        other => apply_state_op(state, other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(row: i32, col: i32) -> CellKey {
        CellKey { row, col }
    }

    #[test]
    fn click_maps_to_begin_selection() {
        let op = dispatch_matrix_lookup(&GridInput::Click(k(1, 2)));
        assert!(matches!(op, Some(StateOp::BeginSelection(c)) if c == k(1, 2)));
    }

    #[test]
    fn drag_start_maps_to_begin_selection() {
        let op = dispatch_matrix_lookup(&GridInput::Drag(DragPhase::Start { anchor: k(0, 0) }));
        assert!(matches!(op, Some(StateOp::BeginSelection(c)) if c == k(0, 0)));
    }

    #[test]
    fn drag_move_maps_to_extend_selection() {
        let op = dispatch_matrix_lookup(&GridInput::Drag(DragPhase::Move { cursor: k(5, 3) }));
        assert!(matches!(op, Some(StateOp::ExtendSelection(c)) if c == k(5, 3)));
    }

    #[test]
    fn drag_end_maps_to_set_focus() {
        let op = dispatch_matrix_lookup(&GridInput::Drag(DragPhase::End { focus: k(7, 1) }));
        assert!(matches!(op, Some(StateOp::SetFocus(c)) if c == k(7, 1)));
    }

    #[test]
    fn info_panel_click_maps_to_set_focus() {
        let op = dispatch_matrix_lookup(&GridInput::InfoPanelClick(k(2, 4)));
        assert!(matches!(op, Some(StateOp::SetFocus(c)) if c == k(2, 4)));
    }

    #[test]
    fn key_arrow_maps_to_move_cursor() {
        let op = dispatch_matrix_lookup(&GridInput::Key(Direction::Down));
        assert!(matches!(op, Some(StateOp::MoveCursor(Direction::Down))));
    }

    #[test]
    fn edit_commit_carries_value() {
        let op = dispatch_matrix_lookup(&GridInput::Edit(EditEvent::Commit {
            new_value: "hello".to_string(),
        }));
        if let Some(StateOp::CommitEdit { new_value }) = op {
            assert_eq!(new_value, "hello");
        } else {
            panic!("expected CommitEdit, got {op:?}");
        }
    }

    #[test]
    fn edit_cancel_maps_to_cancel_edit() {
        let op = dispatch_matrix_lookup(&GridInput::Edit(EditEvent::Cancel));
        assert!(matches!(op, Some(StateOp::CancelEdit)));
    }

    #[test]
    fn focus_lost_maps_to_end_selection() {
        let op = dispatch_matrix_lookup(&GridInput::FocusLost);
        assert!(matches!(op, Some(StateOp::EndSelection)));
    }

    #[test]
    fn paste_carries_text() {
        let op = dispatch_matrix_lookup(&GridInput::Paste("a\tb\nc\td".to_string()));
        if let Some(StateOp::Paste(text)) = op {
            assert_eq!(text, "a\tb\nc\td");
        } else {
            panic!("expected Paste, got {op:?}");
        }
    }

    #[test]
    fn invalidation_helper_builds_state_op() {
        let op = invalidation_to_state_op(16384, InvalidationPhase::Pre);
        assert!(matches!(
            op,
            StateOp::InvalidateTable { table_oid: 16384, phase: InvalidationPhase::Pre }
        ));
    }

    #[test]
    fn unimplemented_inputs_return_none_for_now() {
        // 1.95c 에서 wire 될 path — 현 단계에서는 None 으로 정책 명시.
        assert!(dispatch_matrix_lookup(&GridInput::Edit(EditEvent::Begin)).is_none());
        assert!(dispatch_matrix_lookup(&GridInput::RightClick(k(0, 0))).is_none());
        assert!(dispatch_matrix_lookup(&GridInput::SortHeader {
            col: 0,
            direction: SortDirection::Asc,
        })
        .is_none());
    }

    #[test]
    fn state_aware_dispatch_edit_begin_uses_selected_cell() {
        // Plan v7 Phase 1.95c — dispatch() 가 state-context 로 EditEvent::Begin 을
        // BeginEdit(selected_cell) 로 변환. selected_cell 이 None 이면 None 반환.
        let mut state = AppState::default();
        // 선택 없음 → None
        let op = dispatch(GridInput::Edit(EditEvent::Begin), &state);
        assert!(op.is_none(), "no selection → None");
        // 선택 있음 → BeginEdit(matching cell)
        state.data_edit.selected_cell = Some((3, 7));
        let op = dispatch(GridInput::Edit(EditEvent::Begin), &state);
        assert!(matches!(op, Some(StateOp::BeginEdit(c)) if c == k(3, 7)));
    }

    #[test]
    fn state_aware_dispatch_falls_back_to_matrix_for_pure_inputs() {
        // 본 함수는 state-free 한 input 에 대해 dispatch_matrix_lookup 와 동일.
        let state = AppState::default();
        let op = dispatch(GridInput::Click(k(2, 5)), &state);
        assert!(matches!(op, Some(StateOp::BeginSelection(c)) if c == k(2, 5)));
        let op = dispatch(GridInput::FocusLost, &state);
        assert!(matches!(op, Some(StateOp::EndSelection)));
    }

    #[test]
    fn apply_set_focus_updates_selected_cell_and_shows_info_panel() {
        let mut state = AppState {
            show_info_panel: false,
            ..AppState::default()
        };
        apply_state_op(&mut state, StateOp::SetFocus(k(4, 9)));
        assert_eq!(state.data_edit.selected_cell, Some((4, 9)));
        assert!(state.show_info_panel);
    }

    #[test]
    fn apply_begin_selection_sets_selected_clears_editing_shows_info() {
        let mut state = AppState::default();
        state.data_edit.editing_cell = Some((0, 0));
        state.show_info_panel = false;
        apply_state_op(&mut state, StateOp::BeginSelection(k(2, 3)));
        assert_eq!(state.data_edit.selected_cell, Some((2, 3)));
        assert_eq!(state.data_edit.editing_cell, None);
        assert!(state.show_info_panel);
    }

    #[test]
    fn apply_begin_edit_sets_editing_cell() {
        let mut state = AppState::default();
        apply_state_op(&mut state, StateOp::BeginEdit(k(7, 1)));
        assert_eq!(state.data_edit.editing_cell, Some((7, 1)));
    }

    #[test]
    fn apply_cancel_edit_clears_editing_cell() {
        let mut state = AppState::default();
        state.data_edit.editing_cell = Some((1, 2));
        apply_state_op(&mut state, StateOp::CancelEdit);
        assert_eq!(state.data_edit.editing_cell, None);
    }

    #[test]
    fn apply_commit_edit_clears_editing_cell() {
        let mut state = AppState::default();
        state.data_edit.editing_cell = Some((3, 4));
        apply_state_op(
            &mut state,
            StateOp::CommitEdit {
                new_value: "v".to_string(),
            },
        );
        assert_eq!(state.data_edit.editing_cell, None);
    }

    #[test]
    fn apply_end_selection_keeps_state_unchanged() {
        let mut state = AppState::default();
        state.data_edit.selected_cell = Some((5, 6));
        let before = state.data_edit.selected_cell;
        apply_state_op(&mut state, StateOp::EndSelection);
        assert_eq!(state.data_edit.selected_cell, before);
    }

    #[test]
    fn apply_revert_edits_clears_dirty_cells() {
        // RevertEdits → data_edit.cells 가 정리되어야 함 (revert_data_edits 위임)
        let mut state = AppState::default();
        // 빈 상태에서 호출해도 panic 없음
        apply_state_op(&mut state, StateOp::RevertEdits);
        assert!(state.data_edit.cells.is_empty());
    }

    #[test]
    fn table_info_carries_oid_field() {
        // US-K2 — TableInfo struct 가 oid: Option<u32> 필드 보유
        let t = crate::types::TableInfo {
            name: "users".to_string(),
            table_type: "BASE TABLE".to_string(),
            oid: Some(16384),
        };
        assert_eq!(t.oid, Some(16384));
        let t2 = crate::types::TableInfo {
            name: "v".to_string(),
            table_type: "VIEW".to_string(),
            oid: None,
        };
        assert!(t2.oid.is_none());
    }

    #[test]
    fn apply_invalidate_table_pre_phase_sets_stale_banner() {
        // US-I3 — InvalidateTable::Pre 는 last_error 배너로 stale 알림.
        let mut state = AppState::default();
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 16384,
                phase: InvalidationPhase::Pre,
            },
        );
        assert!(state.last_error.is_some());
        assert!(state.last_error.as_ref().unwrap().contains("Schema"));
    }

    #[test]
    fn apply_invalidate_table_post_phase_in_state_only_path_is_noop() {
        // apply_state_op (state-only) path 는 Post phase 에 대해 noop — bridge 필요한 reload 는 with_bridge 에서.
        let mut state = AppState {
            last_error: Some("preserved".to_string()),
            ..AppState::default()
        };
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 16384,
                phase: InvalidationPhase::Post,
            },
        );
        assert_eq!(state.last_error.as_deref(), Some("preserved"));
    }

    #[test]
    fn apply_delete_marks_already_registered_cell_null() {
        // Delete 가 이미 등록된 셀을 그대로 is_null=true 마킹 (재등록 noop).
        let mut state = make_state_with_result(2, 2);
        state.data_edit.selected_cell = Some((0, 0));
        state.data_edit.cells.insert(
            (0, 0),
            crate::state::EditableCell {
                original: crate::types::CellValue::Text("orig".to_string()),
                original_text: "orig".to_string(),
                value: "edited".to_string(),
                is_null: false,
            },
        );
        apply_state_op(&mut state, StateOp::Delete);
        let cell = state.data_edit.cells.get(&(0, 0)).unwrap();
        assert!(cell.is_null);
        // original 은 보존 (재등록 안 함)
        assert!(matches!(cell.original, crate::types::CellValue::Text(ref s) if s == "orig"));
    }

    fn make_state_with_result(rows: usize, cols: usize) -> AppState {
        use crate::types::{ColumnMeta, QueryResult};
        let mut state = AppState::default();
        let columns: Vec<ColumnMeta> = (0..cols)
            .map(|i| ColumnMeta {
                name: format!("c{i}"),
                type_name: "text".to_string(),
            })
            .collect();
        let result_rows: Vec<Vec<crate::types::CellValue>> = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| crate::types::CellValue::Text(format!("v{r}_{c}")))
                    .collect()
            })
            .collect();
        state.current_result = Some(QueryResult {
            columns,
            rows: result_rows,
            execution_time_ms: 0,
        });
        state
    }

    #[test]
    fn apply_move_cursor_down_increments_row_within_bounds() {
        let mut state = make_state_with_result(5, 3);
        state.data_edit.selected_cell = Some((1, 1));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::Down));
        assert_eq!(state.data_edit.selected_cell, Some((2, 1)));
    }

    #[test]
    fn apply_move_cursor_up_clamps_at_zero() {
        let mut state = make_state_with_result(5, 3);
        state.data_edit.selected_cell = Some((0, 1));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::Up));
        assert_eq!(state.data_edit.selected_cell, Some((0, 1)));
    }

    #[test]
    fn apply_move_cursor_right_clamps_at_last_col() {
        let mut state = make_state_with_result(5, 3);
        state.data_edit.selected_cell = Some((1, 2));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::Right));
        assert_eq!(state.data_edit.selected_cell, Some((1, 2)));
    }

    #[test]
    fn apply_move_cursor_home_end_jump_to_col_extremes() {
        let mut state = make_state_with_result(5, 3);
        state.data_edit.selected_cell = Some((2, 1));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::Home));
        assert_eq!(state.data_edit.selected_cell, Some((2, 0)));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::End));
        assert_eq!(state.data_edit.selected_cell, Some((2, 2)));
    }

    #[test]
    fn apply_move_cursor_page_up_down_jumps_20_rows_clamped() {
        let mut state = make_state_with_result(50, 3);
        state.data_edit.selected_cell = Some((25, 1));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::PageUp));
        assert_eq!(state.data_edit.selected_cell, Some((5, 1)));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::PageDown));
        assert_eq!(state.data_edit.selected_cell, Some((25, 1)));
        // PageUp on small grid clamps to 0
        let mut small = make_state_with_result(10, 3);
        small.data_edit.selected_cell = Some((5, 0));
        apply_state_op(&mut small, StateOp::MoveCursor(Direction::PageUp));
        assert_eq!(small.data_edit.selected_cell, Some((0, 0)));
        // PageDown beyond last clamps to row_count - 1
        small.data_edit.selected_cell = Some((5, 0));
        apply_state_op(&mut small, StateOp::MoveCursor(Direction::PageDown));
        assert_eq!(small.data_edit.selected_cell, Some((9, 0)));
    }

    #[test]
    fn apply_move_cursor_no_result_is_noop() {
        let mut state = AppState::default();
        state.data_edit.selected_cell = Some((1, 1));
        apply_state_op(&mut state, StateOp::MoveCursor(Direction::Down));
        // Selection should remain unchanged when no result is loaded
        assert_eq!(state.data_edit.selected_cell, Some((1, 1)));
    }

    #[test]
    fn apply_paste_single_value_overwrites_selected_cell() {
        let mut state = make_state_with_result(3, 3);
        state.data_edit.selected_cell = Some((1, 1));
        // Pre-populate cell so Paste finds it
        state.data_edit.cells.insert(
            (1, 1),
            crate::state::EditableCell {
                original: crate::types::CellValue::Text("orig".to_string()),
                original_text: "orig".to_string(),
                value: "orig".to_string(),
                is_null: false,
            },
        );
        apply_state_op(&mut state, StateOp::Paste("hello".to_string()));
        assert_eq!(state.data_edit.cells.get(&(1, 1)).unwrap().value, "hello");
    }

    #[test]
    fn apply_paste_tsv_writes_multi_row_multi_col() {
        let mut state = make_state_with_result(3, 3);
        state.data_edit.selected_cell = Some((0, 0));
        // Pre-populate 4 cells (0,0) (0,1) (1,0) (1,1)
        for (r, c) in &[(0usize, 0usize), (0, 1), (1, 0), (1, 1)] {
            state.data_edit.cells.insert(
                (*r, *c),
                crate::state::EditableCell {
                    original: crate::types::CellValue::Text("o".to_string()),
                    original_text: "o".to_string(),
                    value: "o".to_string(),
                    is_null: false,
                },
            );
        }
        apply_state_op(&mut state, StateOp::Paste("a\tb\nc\td".to_string()));
        assert_eq!(state.data_edit.cells.get(&(0, 0)).unwrap().value, "a");
        assert_eq!(state.data_edit.cells.get(&(0, 1)).unwrap().value, "b");
        assert_eq!(state.data_edit.cells.get(&(1, 0)).unwrap().value, "c");
        assert_eq!(state.data_edit.cells.get(&(1, 1)).unwrap().value, "d");
    }

    #[test]
    fn apply_delete_registers_unregistered_cell_and_marks_null() {
        // US-I2 — Delete 가 미등록 셀에 대해 ensure_data_edit_cell_from_result 호출 후 is_null=true.
        let mut state = make_state_with_result(2, 2);
        state.data_edit.selected_cell = Some((1, 1));
        assert!(state.data_edit.cells.is_empty());
        apply_state_op(&mut state, StateOp::Delete);
        let cell = state.data_edit.cells.get(&(1, 1)).expect("registered");
        assert!(cell.is_null);
        // original 은 result 의 v1_1 으로 시드됨 (Apply/Revert 안전)
        assert!(matches!(cell.original, crate::types::CellValue::Text(ref s) if s == "v1_1"));
    }

    #[test]
    fn apply_paste_registers_unregistered_cell_on_demand() {
        // US-I2 — 미등록 셀이라도 ensure_data_edit_cell_from_result 가 등록 후 값 적용
        let mut state = make_state_with_result(2, 2);
        state.data_edit.selected_cell = Some((0, 0));
        // 사전 등록 없음
        assert!(state.data_edit.cells.is_empty());
        apply_state_op(&mut state, StateOp::Paste("hello".to_string()));
        let cell = state.data_edit.cells.get(&(0, 0)).expect("registered");
        assert_eq!(cell.value, "hello");
        // original 은 result 의 v0_0 으로 시드됨
        assert!(matches!(cell.original, crate::types::CellValue::Text(ref s) if s == "v0_0"));
    }

    #[test]
    fn apply_paste_tsv_registers_multiple_unregistered_cells() {
        let mut state = make_state_with_result(2, 2);
        state.data_edit.selected_cell = Some((0, 0));
        apply_state_op(&mut state, StateOp::Paste("a\tb\nc\td".to_string()));
        assert_eq!(state.data_edit.cells.len(), 4);
        assert_eq!(state.data_edit.cells.get(&(0, 0)).unwrap().value, "a");
        assert_eq!(state.data_edit.cells.get(&(1, 1)).unwrap().value, "d");
    }

    #[test]
    fn apply_paste_overflow_clamps_within_grid() {
        let mut state = make_state_with_result(2, 2);
        state.data_edit.selected_cell = Some((1, 1));
        state.data_edit.cells.insert(
            (1, 1),
            crate::state::EditableCell {
                original: crate::types::CellValue::Text("o".to_string()),
                original_text: "o".to_string(),
                value: "o".to_string(),
                is_null: false,
            },
        );
        // 3 rows × 3 cols paste from (1,1) — only (1,1) writes (overflow rows 2,3 / cols 2,3 dropped)
        apply_state_op(
            &mut state,
            StateOp::Paste("x\ty\tz\n1\t2\t3\n4\t5\t6".to_string()),
        );
        assert_eq!(state.data_edit.cells.get(&(1, 1)).unwrap().value, "x");
        // (2, *) and (*, 2) shouldn't exist or shouldn't be written
        assert!(!state.data_edit.cells.contains_key(&(2, 1)));
    }

    #[test]
    fn apply_extend_selection_keeps_anchor() {
        let mut state = AppState::default();
        state.data_edit.selected_cell = Some((5, 6));
        apply_state_op(&mut state, StateOp::ExtendSelection(k(8, 9)));
        // anchor 유지 — Phase 후속에서 actual range 추적
        assert_eq!(state.data_edit.selected_cell, Some((5, 6)));
    }

    // ---------- US-M1 — pending_invalidations tracker ----------

    #[test]
    fn invalidate_pre_inserts_into_pending_invalidations() {
        let mut state = AppState::default();
        assert!(state.pending_invalidations.is_empty());
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 16384,
                phase: InvalidationPhase::Pre,
            },
        );
        assert!(state.pending_invalidations.contains_key(&16384));
        assert_eq!(state.pending_invalidations.len(), 1);
    }

    #[test]
    fn invalidate_pre_with_zero_oid_skips_tracker() {
        let mut state = AppState::default();
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 0,
                phase: InvalidationPhase::Pre,
            },
        );
        assert!(state.pending_invalidations.is_empty());
    }

    #[test]
    fn invalidate_pre_repeated_does_not_reset_started_instant() {
        let mut state = AppState::default();
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 16384,
                phase: InvalidationPhase::Pre,
            },
        );
        let first_instant = *state.pending_invalidations.get(&16384).unwrap();
        // 2번째 Pre 도착 — entry().or_insert_with() 패턴이므로 기존 instant 유지.
        std::thread::sleep(std::time::Duration::from_millis(2));
        apply_state_op(
            &mut state,
            StateOp::InvalidateTable {
                table_oid: 16384,
                phase: InvalidationPhase::Pre,
            },
        );
        let second_instant = *state.pending_invalidations.get(&16384).unwrap();
        assert_eq!(first_instant, second_instant, "재진입 Pre 가 instant 를 리셋하면 안 됨");
    }

    #[test]
    fn invalidate_pre_tracks_multiple_distinct_oids() {
        let mut state = AppState::default();
        for oid in [16384u32, 16385, 16386] {
            apply_state_op(
                &mut state,
                StateOp::InvalidateTable {
                    table_oid: oid,
                    phase: InvalidationPhase::Pre,
                },
            );
        }
        assert_eq!(state.pending_invalidations.len(), 3);
        for oid in [16384u32, 16385, 16386] {
            assert!(state.pending_invalidations.contains_key(&oid));
        }
    }
}
