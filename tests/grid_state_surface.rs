//! Plan v7 Phase 1.95 — Grid state surface 회귀 테스트.
//!
//! 본 scaffold 는 1.95a 의 dispatch types + matrix 위에 얹히는 *state surface*
//! 회귀 케이스의 시그니처를 고정한다. 실제 state mutation 을 호출하는 본체는
//! 1.95c 의 grid.rs 분리 + dispatch wire-up 후 채워진다.
//!
//! 5 회귀 카테고리 (Plan v7 §11):
//! 1. selection (단일/range)
//! 2. hit_test (좌표 → cell 변환)
//! 3. edit start/commit
//! 4. paste (clipboard text → cell 분해)
//! 5. info_panel (편집 정보 표시 / dirty marker / PK 표시 / null 표시 / input focus)
//!
//! 단위 테스트 (src/ui/grid_dispatch.rs:tests, 12 cases) 가 dispatch matrix 의
//! 정책 정합성을 보장하므로, 본 파일은 *integration 시그니처 freeze* 역할.

#[ignore = "wired in Phase 1.95c after grid.rs split + dispatch implementation"]
#[test]
fn selection_single_click_sets_begin_selection() {
    // Single click on cell → state.selection_anchor == clicked cell
    // Drag move → state.selection_range expanded
    // FIXME(Phase 1.95c): state surface 노출 후 채움
}

#[ignore = "wired in Phase 1.95c after grid.rs split + dispatch implementation"]
#[test]
fn hit_test_pixel_coords_resolve_to_cell_key() {
    // ui hit-test → CellKey { row, col } 정합성 검증
    // FIXME(Phase 1.95c)
}

#[ignore = "wired in Phase 1.95c after grid.rs split + dispatch implementation"]
#[test]
fn edit_begin_then_commit_produces_dirty_cell() {
    // BeginEdit → editing_cell 설정
    // CommitEdit { new_value } → state.data_edit.cells 의 dirty marker
    // FIXME(Phase 1.95c)
}

#[ignore = "wired in Phase 1.95c after grid.rs split + dispatch implementation"]
#[test]
fn paste_text_decomposes_into_grid_cells() {
    // Paste("a\tb\nc\td") → 2x2 셀에 적용
    // FIXME(Phase 1.95c)
}

#[ignore = "wired in Phase 1.95c after grid.rs split + dispatch implementation"]
#[test]
fn info_panel_displays_focus_dirty_pk_and_null_markers() {
    // 편집 정보 / dirty marker / PK 표시 / null 표시 / input focus 5-in-1 검증
    // FIXME(Phase 1.95c)
}
