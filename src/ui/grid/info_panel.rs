//! Info panel (selected cell / row metadata + edit affordances).
//!
//! Plan v7 Phase 1.95c — cut-over from `super::mod.rs`. Hosts
//! `render_info_panel`, `restore_active_data_tab`, and all sub-renderers
//! (table overview, columns, indexes, relations, rules/triggers, JSON tree,
//! enum control, dark-select, toggle, action buttons).

use std::hash::Hash;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{cell_edit_text_for_type, AppState, DataFilter, MainView};
use crate::types::{CellValue, ColumnInfo, ColumnMeta, IndexInfo, RuleInfo, TriggerInfo};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

use super::{
    data_column_info, data_edit_summary, metric_chip, reload_data_source,
    request_foreign_keys_for_schema, request_table_columns_for_data, revert_data_edits,
    set_pointing_cursor_on_hover, show_dark_popup_below,
};
use super::info_row::{
    render_info_empty, render_info_header, render_info_row_fields, render_info_row_summary,
};
use super::table_info::{ensure_table_info_metadata, render_info_table_overview};
use super::show_dark_hover_tooltip;

pub fn render_info_panel(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    render_info_header(ui);

    egui::ScrollArea::vertical()
        .id_salt("data_info_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_MD_I))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if state.active_main_view != MainView::Data {
                        render_info_empty(ui, &t("data_info_no_selection"));
                        return;
                    }

                    let Some(context) = selected_data_row_context(state) else {
                        let Some(source) = state.active_data_source() else {
                            render_info_empty(ui, &t("data_info_select_cell"));
                            return;
                        };
                        ensure_table_info_metadata(state, bridge, &source);
                        render_info_table_overview(ui, state, &source);
                        return;
                    };

                    for (col_idx, cell) in context.fallback_row.iter().enumerate() {
                        let type_name = context
                            .columns
                            .get(col_idx)
                            .map(|column| column.type_name.as_str())
                            .unwrap_or_default();
                        ensure_data_edit_cell(state, context.row_idx, col_idx, cell, type_name);
                    }

                    render_info_row_summary(ui, &context);
                    ui.add_space(theme::SPACE_LG);
                    render_info_row_fields(ui, state, bridge, &context);
                    render_info_apply_controls(ui, state, bridge);
                    ui.add_space(theme::SPACE_LG);
                });
        });
}

pub fn restore_active_data_tab(state: &mut AppState, bridge: &DbBridge) {
    if state.active_main_view != MainView::Data {
        return;
    }
    let Some(source) = state.active_data_source() else {
        return;
    };

    if state.data_edit.source.as_ref() != Some(&source) {
        state.begin_data_edit_with_filter(
            source.conn_id,
            &source.schema,
            &source.table,
            source.filter.clone(),
        );
    }
    request_table_columns_for_data(state, bridge, source.conn_id, &source.schema, &source.table);
    request_foreign_keys_for_schema(state, bridge, source.conn_id, &source.schema);
    reload_data_source(state, bridge);
}

#[derive(Clone)]
pub(super) struct SelectedRowContext {
    pub(super) row_idx: usize,
    pub(super) selected_col_idx: usize,
    pub(super) source_label: String,
    pub(super) columns: Vec<ColumnMeta>,
    pub(super) column_infos: Vec<Option<ColumnInfo>>,
    pub(super) fallback_row: Vec<CellValue>,
}

#[derive(Clone)]
pub(super) struct TableInfoContext {
    pub(super) source_label: String,
    pub(super) table_name: String,
    pub(super) schema: String,
    pub(super) table_type: String,
    pub(super) filter: Option<DataFilter>,
    pub(super) columns: Vec<ColumnInfo>,
    pub(super) indexes: Vec<IndexInfo>,
    pub(super) relations: Vec<ForeignKey>,
    pub(super) rules: Vec<RuleInfo>,
    pub(super) triggers: Vec<TriggerInfo>,
    pub(super) loading_columns: bool,
}

pub(super) fn selected_data_row_context(state: &AppState) -> Option<SelectedRowContext> {
    let (row_idx, col_idx) = state.data_edit.selected_cell?;
    let result = state.current_result.as_ref()?;
    result.columns.get(col_idx)?;
    let fallback_row = result.rows.get(row_idx)?.clone();
    let source_label = state
        .active_data_source()
        .map(|source| format!("{}.{}", source.schema, source.table))
        .unwrap_or_else(|| t("objects_data_title"));
    let column_infos = result
        .columns
        .iter()
        .map(|column| data_column_info(state, &column.name).cloned())
        .collect();

    Some(SelectedRowContext {
        row_idx,
        selected_col_idx: col_idx,
        source_label,
        columns: result.columns.clone(),
        column_infos,
        fallback_row,
    })
}

pub(super) fn ensure_data_edit_cell(
    state: &mut AppState,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    let timezone = state.data_timezone.clone();
    state
        .data_edit
        .cells
        .entry((row_idx, col_idx))
        .or_insert_with(|| {
            crate::state::EditableCell::from_cell_for_type(fallback_cell, type_name, &timezone)
        });
}

pub(super) fn render_info_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    if let Some(value) = dark_select_control(
        ui,
        ("info_enum_cell", row_idx, col_idx),
        &selected,
        enum_values,
        ui.available_width(),
    ) {
        edit.value = value;
    }
}

pub(super) fn render_info_apply_controls(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let Some(summary) = data_edit_summary(state) else {
        return;
    };

    // Plan v7 Phase 3b — disable apply when explicit tx active.
    let tx_blocked = state.explicit_tx_active;

    ui.add_space(theme::SPACE_LG);
    ui.separator();
    ui.add_space(theme::SPACE_MD);

    if tx_blocked {
        ui.label(
            RichText::new("Apply disabled — explicit transaction active in Query tab")
                .color(theme::ACCENT_YELLOW)
                .size(11.0),
        );
        ui.add_space(theme::SPACE_SM);
    }

    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("grid_edits", &[&summary.dirty_count.to_string()]),
            summary.color,
        );
        if let Some(reason) = &summary.blocked_reason {
            ui.label(RichText::new(reason).color(theme::ACCENT_YELLOW).size(11.0));
        }
    });
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        let can_apply = summary.can_apply && !state.data_edit.applying && !tx_blocked;
        if info_text_action_button(ui, &t("button_apply"), can_apply).clicked() {
            crate::ui::grid_dispatch::apply_state_op_with_bridge(
                state,
                crate::ui::grid_dispatch::StateOp::ApplyEdits,
                bridge,
            );
        }

        if info_text_action_button(ui, &t("grid_revert"), !state.data_edit.applying).clicked() {
            revert_data_edits(state);
        }
    });
}

pub(super) fn revert_data_cell(state: &mut AppState, cell_key: (usize, usize), type_name: &str) {
    let timezone = state.data_timezone.clone();
    if let Some(cell) = state.data_edit.cells.get_mut(&cell_key) {
        let value = cell_edit_text_for_type(&cell.original, type_name, &timezone);
        cell.value = value.clone();
        cell.original_text = value;
        cell.is_null = matches!(cell.original, CellValue::Null);
    }
    state.data_edit.editing_cell = None;
}

pub(super) fn editable_cell_display_text(cell: &crate::state::EditableCell) -> String {
    if cell.is_null {
        "NULL".to_string()
    } else {
        cell.value.clone()
    }
}

pub(super) fn info_icon_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    enabled: bool,
    icon_color: Color32,
) -> egui::Response {
    let (rect, response) = info_action_button_frame(ui, 30.0, enabled);
    let icon_color = if enabled {
        icon_color
    } else {
        theme::text_disabled()
    };
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.set_clip_rect(icon_rect.intersect(rect));
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, icon_color,
            ));
        },
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, label);
    response
}

pub(super) fn info_text_action_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let target_width = (text_width + 28.0).ceil().max(96.0);
    let (rect, response) = info_action_button_frame(ui, target_width, enabled);
    ui.painter()
        .with_clip_rect(rect.shrink2(egui::vec2(8.0, 2.0)))
        .text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font,
            text_color,
        );
    response
}

pub(super) fn info_action_button_frame(
    ui: &mut egui::Ui,
    width: f32,
    enabled: bool,
) -> (egui::Rect, egui::Response) {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), sense);
    let hovered = enabled && response.hovered();
    let fill = if !enabled {
        theme::bg_medium()
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke_color = if hovered {
        theme::with_alpha(theme::ACCENT_TEAL, 140)
    } else if enabled {
        theme::border_default()
    } else {
        theme::border_subtle()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    set_pointing_cursor_on_hover(ui, &response, enabled);
    (rect, response)
}

pub(super) fn info_toggle_control(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let font = egui::FontId::proportional(11.5);
    let label_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let width = (label_width + 30.0).min(ui.available_width()).max(48.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 22.0), sense);
    if response.clicked() && enabled {
        *checked = !*checked;
    }

    let hovered = enabled && response.hovered();
    let box_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(9.0, 0.0),
        egui::vec2(16.0, 16.0),
    );
    let box_fill = if *checked {
        theme::with_alpha(theme::ACCENT_TEAL, if hovered { 52 } else { 36 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let box_stroke = if *checked {
        theme::ACCENT_TEAL
    } else if hovered {
        theme::border_strong()
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(box_rect, CornerRadius::same(theme::RADIUS_SM), box_fill);
    ui.painter().rect_stroke(
        box_rect,
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(1.0, box_stroke),
        egui::StrokeKind::Inside,
    );
    if *checked {
        let a = box_rect.left_center() + egui::vec2(4.0, 0.5);
        let b = box_rect.center() + egui::vec2(-1.0, 4.0);
        let c = box_rect.right_center() + egui::vec2(-3.5, -4.5);
        ui.painter()
            .line_segment([a, b], Stroke::new(1.8, theme::ACCENT_TEAL));
        ui.painter()
            .line_segment([b, c], Stroke::new(1.8, theme::ACCENT_TEAL));
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(24.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    set_pointing_cursor_on_hover(ui, &response, enabled);
    response
}

pub(super) fn dark_select_control(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    selected_text: &str,
    options: &[String],
    width: f32,
) -> Option<String> {
    let popup_id = ui.make_persistent_id(("dark_select", id_source));
    let width = width.max(96.0);
    let response = dark_select_button(ui, selected_text, width);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    let mut selected_value = None;
    show_dark_popup_below(ui, popup_id, &response, width, theme::SPACE_SM_I, |ui| {
        for option in options {
            if dark_select_option(ui, option, option == selected_text, width).clicked() {
                selected_value = Some(option.clone());
                ui.memory_mut(|memory| memory.close_popup());
            }
        }
    });
    selected_value
}

fn dark_select_button(ui: &mut egui::Ui, selected_text: &str, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_medium()
    } else {
        theme::input_bg()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 150))
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let text_clip = rect.shrink2(egui::vec2(10.0, 0.0));
    ui.painter().with_clip_rect(text_clip).text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        selected_text,
        egui::FontId::proportional(12.0),
        theme::text_primary(),
    );
    let center = rect.right_center() - egui::vec2(14.0, 0.0);
    ui.painter().line_segment(
        [
            center + egui::vec2(-5.0, -2.5),
            center + egui::vec2(0.0, 3.5),
        ],
        Stroke::new(1.8, theme::text_secondary()),
    );
    ui.painter().line_segment(
        [
            center + egui::vec2(0.0, 3.5),
            center + egui::vec2(5.0, -2.5),
        ],
        Stroke::new(1.8, theme::text_secondary()),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn dark_select_option(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 34)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if selected {
            theme::ACCENT_TEAL
        } else {
            theme::text_secondary()
        },
    );
    if selected {
        ui.painter().circle_filled(
            rect.right_center() - egui::vec2(11.0, 0.0),
            3.0,
            theme::ACCENT_TEAL,
        );
    }
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

pub(super) fn info_section_label(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .color(theme::text_muted())
            .strong()
            .size(11.0),
    );
}

pub(super) fn tiny_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(9.5),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 12.0, 17.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 28),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(9.5),
        color,
    );
}

pub(super) fn value_box(ui: &mut egui::Ui, value: &str, color: Color32) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(value).color(color).monospace().size(11.5)).wrap(),
            );
        });
}
