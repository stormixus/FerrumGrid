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
}
