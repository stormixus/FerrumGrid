//! Cell editing, selection, date/time pickers, and relation-jump UI.
//!
//! Extracted from `super::mod.rs` — hosts render_editable_cell,
//! render_readonly_data_cell, date/time editors, cell overlay widgets,
//! relation jump buttons, and supporting helpers.

use eframe::egui::{self, CornerRadius, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{data_filter_from_cell, data_filter_from_text, AppState};
use crate::types::{CellValue, ColumnMeta};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

use super::cell_overlay::{
    cell_overlay_editor_rect, cell_overlay_null_toggle, render_cell_bool_overlay,
    render_cell_overlay_value, render_cell_text_overlay,
};
use super::info_panel::{dark_select_control, editable_cell_display_text};
use super::tooltips::show_dark_hover_tooltip;
use super::{
    data_column_info, edit_kind, has_table_column_metadata, open_related_data, parse_bool,
    relation_for_column, render_cell, render_passive_cell, render_passive_copyable_cell,
    passive_value_pill, request_foreign_keys_for_schema, show_cell_copy_context_menu,
    validate_edit_value, EditKind,
};

pub(super) fn render_editable_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
) {
    if !has_table_column_metadata(state) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            None,
        );
        return;
    }

    if !state.data_edit.cells.contains_key(&(row_idx, col_idx)) {
        state.data_edit.cells.insert(
            (row_idx, col_idx),
            crate::state::EditableCell::from_cell_for_type(
                fallback_cell,
                column.map(|col| col.type_name.as_str()).unwrap_or_default(),
                &state.data_timezone,
            ),
        );
    }

    let column_info = column.and_then(|col| data_column_info(state, &col.name).cloned());
    if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            Some(&t("data_info_read_only_pk")),
        );
        return;
    }

    let cell_key = (row_idx, col_idx);
    let type_name = column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .or_else(|| column.map(|col| col.type_name.clone()))
        .unwrap_or_default();
    let nullable = column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();

    let Some(snapshot) = state.data_edit.cells.get(&cell_key).cloned() else {
        render_cell(ui, fallback_cell);
        return;
    };

    let dirty = snapshot.is_dirty();
    let error = validate_edit_value(&snapshot, &type_name, nullable, &enum_values);
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();
    if selected {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            theme::with_alpha(theme::accent_color(), 16),
        );
    }
    if dirty {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            theme::with_alpha(theme::accent_color(), 30),
        );
    } else if error.is_some() {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            theme::with_alpha(theme::ACCENT_RED, 28),
        );
    }

    let is_editing = state.data_edit.editing_cell == Some(cell_key);
    if !is_editing {
        let relation_target = if !snapshot.is_null && error.is_none() {
            relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
                data_filter_from_text(fk.target_column.clone(), type_name.clone(), &snapshot.value)
                    .map(|filter| (fk, filter))
            })
        } else {
            None
        };
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("data_cell", row_idx, col_idx)),
            egui::Sense::click(),
        );
        let copy_text = editable_cell_display_text(&snapshot);
        show_cell_copy_context_menu(&response, &copy_text);
        let content_width = relation_content_width(rect, relation_target.is_some());
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, rect.height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                render_editable_display_cell(ui, &snapshot, fallback_cell, &type_name);
                if dirty {
                    ui.add_space(2.0);
                    ui.painter().circle_filled(
                        ui.cursor().left_center() + egui::vec2(4.0, 0.0),
                        2.0,
                        theme::accent_color(),
                    );
                    ui.add_space(8.0);
                }
            },
        );
        let relation_clicked = if let Some((fk, filter)) = relation_target {
            let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
            if clicked {
                open_related_data(state, bridge, &fk, filter);
            }
            clicked
        } else {
            false
        };

        if response.double_clicked() && !relation_clicked {
            select_data_cell(state, row_idx, col_idx, true);
        } else if response.clicked() && !relation_clicked {
            select_data_cell(state, row_idx, col_idx, false);
        }
        if let Some(error) = error {
            show_dark_hover_tooltip(ui, response.id.with("error"), &response, &error);
        }
        return;
    }

    let mut close_editor = false;
    let Some(edit) = state.data_edit.cells.get_mut(&cell_key) else {
        render_cell(ui, fallback_cell);
        return;
    };

    let editor_rect = cell_overlay_editor_rect(rect, nullable);
    if nullable && cell_overlay_null_toggle(ui, rect, cell_key, edit.is_null).clicked() {
        edit.is_null = !edit.is_null;
    }

    if edit.is_null {
        render_cell_overlay_value(ui, editor_rect, "NULL", theme::text_muted());
    } else if !enum_values.is_empty() {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(editor_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(editor_rect);
                close_editor |= render_enum_editor(ui, edit, row_idx, col_idx, &enum_values);
            },
        );
    } else {
        match edit_kind(&type_name, fallback_cell) {
            EditKind::Bool => {
                close_editor |= render_cell_bool_overlay(ui, editor_rect, edit);
            }
            EditKind::Date => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::DateTime => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Number | EditKind::Json | EditKind::Uuid | EditKind::Bytes => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Text => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, false, error.as_deref());
            }
        }
    }

    if close_editor || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.data_edit.editing_cell = None;
    }
}

pub(super) fn render_readonly_data_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    cell_key: (usize, usize),
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
    tooltip: Option<&str>,
) {
    let (row_idx, col_idx) = cell_key;
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();

    if selected {
        ui.painter().rect_filled(
            rect,
            CornerRadius::ZERO,
            theme::with_alpha(theme::accent_color(), 16),
        );
    }

    let response = ui.interact(
        rect,
        ui.make_persistent_id(("data_cell_readonly", row_idx, col_idx)),
        egui::Sense::click(),
    );
    let copy_text = fallback_cell.to_string();
    show_cell_copy_context_menu(&response, &copy_text);
    let relation_target =
        relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
            let type_name = column
                .map(|col| col.type_name.clone())
                .unwrap_or_else(|| fk.source_column.clone());
            data_filter_from_cell(fk.target_column.clone(), type_name, fallback_cell)
                .map(|filter| (fk, filter))
        });
    let content_width = relation_content_width(rect, relation_target.is_some());
    ui.allocate_ui_with_layout(
        egui::vec2(content_width, rect.height()),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            render_passive_cell(ui, fallback_cell);
        },
    );

    let relation_clicked = if let Some((fk, filter)) = relation_target {
        let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
        if clicked {
            open_related_data(state, bridge, &fk, filter);
        }
        clicked
    } else {
        false
    };

    if response.clicked() && !relation_clicked {
        select_data_cell(state, row_idx, col_idx, false);
    }
    if let Some(tooltip) = tooltip {
        show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    }
}

fn select_data_cell(state: &mut AppState, row_idx: usize, col_idx: usize, editable: bool) {
    // Plan v7 Phase 1.95 / US-F2 — dispatch → apply_state_op chain.
    // editable=true (Click): BeginSelection 후 BeginEdit 로 즉시 진입.
    // editable=false (InfoPanelClick): SetFocus 만, editing_cell 유지하지 않음.
    use crate::ui::grid_dispatch::{apply_state_op, dispatch, CellKey, GridInput};
    let cell_key = CellKey {
        row: row_idx as i32,
        col: col_idx as i32,
    };
    let dispatch_input = if editable {
        GridInput::Click(cell_key)
    } else {
        GridInput::InfoPanelClick(cell_key)
    };
    if let Some(op) = dispatch(dispatch_input, state) {
        apply_state_op(state, op);
    }
    if editable {
        // editable Click 은 BeginSelection 후 즉시 BeginEdit (legacy 동작 유지)
        apply_state_op(state, crate::ui::grid_dispatch::StateOp::BeginEdit(cell_key));
    }
}

pub(super) fn ensure_foreign_keys_for_active_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    request_foreign_keys_for_schema(state, bridge, source.conn_id, &source.schema);
}

fn relation_content_width(cell_rect: egui::Rect, has_relation: bool) -> f32 {
    if has_relation {
        (cell_rect.width() - 34.0).max(0.0)
    } else {
        cell_rect.width()
    }
}

fn render_relation_jump_button(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    selected: bool,
) -> bool {
    let button_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.right() - 30.0, cell_rect.center().y - 12.0),
        egui::pos2(cell_rect.right() - 6.0, cell_rect.center().y + 12.0),
    );
    let response = ui.interact(
        button_rect,
        ui.make_persistent_id(("relation_jump", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let emphasized = hovered || selected;
    let fill = if emphasized {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::accent_color())
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(button_rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        button_rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let icon_color = if emphasized {
        theme::accent_color()
    } else {
        theme::with_alpha(theme::accent_color(), 190)
    };
    let center = button_rect.center();
    let tip = egui::pos2(center.x + 3.0, center.y);
    ui.painter().line_segment(
        [egui::pos2(center.x - 3.5, center.y - 5.5), tip],
        Stroke::new(2.2, icon_color),
    );
    ui.painter().line_segment(
        [tip, egui::pos2(center.x - 3.5, center.y + 5.5)],
        Stroke::new(2.2, icon_color),
    );

    if hovered {
        set_pointing_cursor_on_hover(ui, &response, true);
    }
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("data_relation_open"),
    );

    response.clicked()
}

pub(super) fn relation_tab_title(fk: &ForeignKey, value: &str) -> String {
    format!(
        "{}.{} · {}",
        fk.target_schema,
        fk.target_table,
        compact_relation_value(value)
    )
}

fn compact_relation_value(value: &str) -> String {
    const MAX_CHARS: usize = 22;
    if value.chars().count() <= MAX_CHARS {
        value.to_string()
    } else {
        let mut compact = value.chars().take(MAX_CHARS).collect::<String>();
        compact.push_str("...");
        compact
    }
}

fn render_editable_display_cell(
    ui: &mut egui::Ui,
    edit: &crate::state::EditableCell,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    if edit.is_null {
        passive_value_pill(ui, "NULL", theme::text_muted());
        return;
    }

    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let value = parse_bool(&edit.value).unwrap_or(false);
            let (text, color) = if value {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            passive_value_pill(ui, text, color);
        }
        EditKind::Number => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_YELLOW)
            });
        }
        EditKind::Json => render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_PURPLE),
        EditKind::Date | EditKind::DateTime => {
            render_passive_copyable_cell(ui, &edit.value, theme::text_secondary())
        }
        EditKind::Uuid => render_passive_copyable_cell(ui, &edit.value, theme::text_muted()),
        EditKind::Bytes => render_passive_copyable_cell(ui, &edit.value, theme::text_muted()),
        EditKind::Text => render_passive_copyable_cell(ui, &edit.value, theme::text_primary()),
    }
}

fn render_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) -> bool {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    if let Some(value) = dark_select_control(
        ui,
        ("enum_cell", row_idx, col_idx),
        &selected,
        enum_values,
        ui.available_width().max(96.0),
    ) {
        edit.value = value;
        true
    } else {
        false
    }
}

pub(super) fn set_pointing_cursor_on_hover(ui: &mut egui::Ui, response: &egui::Response, enabled: bool) {
    if enabled && response.hovered() {
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

pub(super) fn enter_pressed(ui: &egui::Ui) -> bool {
    ui.input(|i| i.key_pressed(egui::Key::Enter))
}
