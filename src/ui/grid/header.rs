//! Result header — top strip with row/col/exec metadata + toolbar buttons.
//!
//! Plan v7 US-G3 — extracted from `render.rs`.

use eframe::egui::{self, CornerRadius, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{AppState, MainView};
use crate::ui::theme;

use eframe::egui::Color32;
use super::data_ops::{data_edit_summary, revert_data_edits};

fn render_result_tab(ui: &mut egui::Ui, label: &str, count: Option<usize>, active: bool) {
    let text_color = if active {
        theme::text_primary()
    } else {
        theme::text_muted()
    };
    let bg = if active {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };

    let text = label.to_string();

    let galley = ui.painter().layout_no_wrap(
        text.clone(),
        egui::FontId::proportional(11.5),
        text_color,
    );
    let count_width = count.map_or(0.0, |n| {
        let cg = ui.painter().layout_no_wrap(
            n.to_string(),
            egui::FontId::monospace(10.0),
            if active { theme::ACCENT_EMERALD } else { theme::text_disabled() },
        );
        cg.rect.width() + 8.0
    });
    let btn_width = galley.rect.width() + count_width + 20.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(btn_width, 24.0), egui::Sense::click());

    let fill = if response.hovered() && !active {
        theme::bg_light()
    } else {
        bg
    };
    ui.painter().rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);

    // Label
    ui.painter().galley(
        egui::pos2(rect.left() + 10.0, rect.center().y - galley.rect.height() / 2.0),
        galley,
        text_color,
    );

    // Count
    if let Some(n) = count {
        let count_color = if active { theme::ACCENT_EMERALD } else { theme::text_disabled() };
        let cg = ui.painter().layout_no_wrap(
            n.to_string(),
            egui::FontId::monospace(10.0),
            count_color,
        );
        ui.painter().galley(
            egui::pos2(
                rect.right() - 10.0 - cg.rect.width(),
                rect.center().y - cg.rect.height() / 2.0,
            ),
            cg,
            count_color,
        );
    }
}
use crate::types::CellValue;
use super::pager::render_data_pager;
use super::paste::{export_csv, export_json, export_sql_insert, result_to_tsv};
use super::toolbar::{
    metric_chip, result_meta_chip, result_meta_chip_svg, result_toolbar_action_button,
};

pub fn render_result_header(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let exec_ms = result.execution_time_ms;

    // Mockup-style result tabs header
    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(egui::Margin::symmetric(8, 4))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;

            // Result tab (active)
            render_result_tab(ui, "Result", Some(row_count), true);
            render_result_tab(ui, "Messages", Some(0), false);
            render_result_tab(ui, "Plan", None, false);

            // Right side: meta info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_LG;

                // Download button
                ui.add(theme::ghost_icon_button(
                    crate::ui::icon_image_tinted(ui, crate::ui::icons_svg::DOWNLOAD, "res_dl", 12.0, theme::text_muted()),
                    "",
                ));
                // Filter button
                ui.add(theme::ghost_icon_button(
                    crate::ui::icon_image_tinted(ui, crate::ui::icons_svg::FILTER, "res_filter", 12.0, theme::text_muted()),
                    "",
                ));

                // Meta segments
                ui.label(
                    egui::RichText::new(format!("rows {}", row_count))
                        .color(theme::text_muted())
                        .monospace()
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new("elapsed")
                        .color(theme::text_disabled())
                        .size(11.0),
                );
                ui.label(
                    egui::RichText::new(format!("{} ms", exec_ms))
                        .color(theme::ACCENT_EMERALD)
                        .monospace()
                        .size(11.0),
                );
            });
        });
    });

    // Keep the old layout code below for data edit actions
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };
    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;
    let data_edit_summary = data_edit_summary(state);

    // Skip old header painting — already rendered above
    let header_height = 0.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover(),
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, 0.0));
    let content_rect = egui::Rect::from_center_size(
        inner.center(),
        egui::vec2(inner.width(), theme::BUTTON_HEIGHT),
    );
    let tsv_width = result_toolbar_action_width(ui, "Copy TSV");
    let export_width = result_toolbar_action_width(ui, "Export");
    let mut right_width = tsv_width + export_width + theme::SPACE_SM * 2.0;
    if data_edit_summary.is_some() {
        right_width += 330.0;
    }
    right_width = right_width.min(content_rect.width() * 0.46);

    let meta_width = result_meta_group_width(ui, row_count, col_count, exec_ms, truncated)
        .min((content_rect.width() - right_width - theme::SPACE_LG).max(120.0));
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.right() - right_width, content_rect.top()),
        content_rect.right_bottom(),
    );
    let meta_rect = egui::Rect::from_min_size(
        content_rect.left_top(),
        egui::vec2(meta_width, content_rect.height()),
    );
    let middle_rect = egui::Rect::from_min_max(
        egui::pos2(meta_rect.right() + theme::SPACE_LG, content_rect.top()),
        egui::pos2(right_rect.left() - theme::SPACE_LG, content_rect.bottom()),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(meta_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(meta_rect);
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
            ui.label(
                RichText::new("Result")
                    .color(theme::text_primary())
                    .strong()
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_MD);
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    row_count,
                    if row_count == 1 { "row" } else { "rows" }
                ),
                theme::ACCENT_EMERALD,
            );
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    col_count,
                    if col_count == 1 { "col" } else { "cols" }
                ),
                theme::ACCENT_BLUE,
            );
            result_meta_chip(ui, &format!("{exec_ms}ms"), theme::ACCENT_COPPER);

            if truncated {
                result_meta_chip_svg(
                    ui,
                    "trunc",
                    crate::ui::icons_svg::TRUNCATED,
                    "truncated_icon",
                    theme::ACCENT_YELLOW,
                );
            }
        },
    );

    if middle_rect.width() > 120.0 && state.active_main_view == MainView::Data {
        let pager_width = 488.0_f32.min(middle_rect.width());
        let pager_rect = egui::Rect::from_center_size(
            middle_rect.center(),
            egui::vec2(pager_width, content_rect.height()),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(pager_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(pager_rect);
                render_data_pager(ui, state, bridge, truncated, row_count);
            },
        );
    }

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(right_rect);
            if let Some(summary) = &data_edit_summary {
                let can_apply = summary.can_apply && !state.data_edit.applying && !state.explicit_tx_active;
                let apply_label = t("button_apply");
                let apply_button = if can_apply {
                    theme::primary_button(&apply_label)
                } else {
                    theme::secondary_button(&apply_label)
                };
                if ui.add_enabled(can_apply, apply_button).clicked() {
                    crate::ui::grid_dispatch::apply_state_op_with_bridge(
                        state,
                        crate::ui::grid_dispatch::StateOp::ApplyEdits,
                        bridge,
                    );
                }

                ui.add_space(theme::SPACE_SM);

                if ui
                    .add_enabled(
                        !state.data_edit.applying,
                        theme::ghost_button(&t("grid_revert")),
                    )
                    .clicked()
                {
                    revert_data_edits(state);
                }

                ui.add_space(theme::SPACE_MD);
                metric_chip(
                    ui,
                    &tf("grid_edits", &[&summary.dirty_count.to_string()]),
                    summary.color,
                );

                if let Some(reason) = &summary.blocked_reason {
                    ui.label(RichText::new(reason).color(theme::ACCENT_YELLOW).size(11.0));
                }

                ui.add_space(theme::SPACE_LG);
            }

            if state.active_main_view == MainView::Data && state.data_edit.source.is_some() {
                let has_selection = state.data_edit.selected_cell.is_some();
                let del_btn = result_toolbar_action_button(
                    ui,
                    crate::ui::icons_svg::CLOSE,
                    "delete_row",
                    &t("grid_delete_row"),
                    has_selection,
                );
                if del_btn.clicked() && has_selection {
                    if let Some((row, _)) = state.data_edit.selected_cell {
                        state.data_edit.pending_deletes.insert(row);
                        state.data_edit.selected_cell = None;
                        state.data_edit.editing_cell = None;
                    }
                }

                ui.add_space(theme::SPACE_SM);

                let add_btn = result_toolbar_action_button(
                    ui,
                    crate::ui::icons_svg::PLUS,
                    "add_row",
                    &t("grid_add_row"),
                    true,
                );
                if add_btn.clicked() {
                    add_empty_row(state);
                }

                ui.add_space(theme::SPACE_MD);
            }

            let export_popup_id = ui.make_persistent_id("export_popup");
            let export_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::EXPORT,
                "export_menu_btn",
                "Export",
                true,
            );
            if export_btn.clicked() {
                ui.memory_mut(|m| m.toggle_popup(export_popup_id));
            }
            super::toolbar::show_dark_popup_below(
                ui,
                export_popup_id,
                &export_btn,
                120.0,
                theme::SPACE_MD_I,
                |ui| {
                    if ui.button("CSV").clicked() {
                        export_csv(state);
                        ui.close_menu();
                    }
                    if ui.button("JSON").clicked() {
                        export_json(state);
                        ui.close_menu();
                    }
                    if ui.button("SQL INSERT").clicked() {
                        export_sql_insert(state);
                        ui.close_menu();
                    }
                },
            );

            ui.add_space(theme::SPACE_SM);

            let tsv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_tsv",
                "Copy TSV",
                true,
            );

            if tsv_btn.clicked() {
                if let Some(ref result) = state.current_result {
                    let tsv = result_to_tsv(result);
                    ui.ctx().copy_text(tsv);
                }
            }

            ui.add_space(theme::SPACE_SM);

            let close_btn = result_toolbar_close_button(ui);
            if close_btn.clicked() {
                state.show_result_panel = false;
            }
        },
    );
}

fn result_toolbar_close_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    let hovered = response.hovered();
    let bg = if hovered {
        theme::with_alpha(theme::ACCENT_RED, 38)
    } else {
        theme::bg_medium()
    };
    let stroke_color = if hovered {
        theme::with_alpha(theme::ACCENT_RED, 160)
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    let icon_color = if hovered {
        theme::ACCENT_RED
    } else {
        theme::text_secondary()
    };
    let cx = rect.center();
    let arm = 5.0;
    ui.painter().line_segment(
        [
            cx + egui::vec2(-arm, -arm),
            cx + egui::vec2(arm, arm),
        ],
        Stroke::new(1.6, icon_color),
    );
    ui.painter().line_segment(
        [
            cx + egui::vec2(arm, -arm),
            cx + egui::vec2(-arm, arm),
        ],
        Stroke::new(1.6, icon_color),
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.on_hover_text("Close result panel")
}

fn result_meta_group_width(
    ui: &egui::Ui,
    row_count: usize,
    col_count: usize,
    exec_ms: u128,
    truncated: bool,
) -> f32 {
    let title_width = ui
        .painter()
        .layout_no_wrap(
            "Result".to_string(),
            egui::FontId::proportional(13.0),
            theme::text_primary(),
        )
        .rect
        .width();
    let row_text = format!(
        "{} {}",
        row_count,
        if row_count == 1 { "row" } else { "rows" }
    );
    let col_text = format!(
        "{} {}",
        col_count,
        if col_count == 1 { "col" } else { "cols" }
    );
    let mut width = title_width
        + theme::SPACE_MD
        + result_meta_chip_width(ui, &row_text)
        + result_meta_chip_width(ui, &col_text)
        + result_meta_chip_width(ui, &format!("{exec_ms}ms"))
        + theme::SPACE_SM * 4.0;
    if truncated {
        width += result_meta_chip_svg_width(ui, "trunc") + theme::SPACE_SM;
    }
    width
}

pub fn result_toolbar_action_width(ui: &egui::Ui, label: &str) -> f32 {
    let width = ui
        .painter()
        .layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(12.0),
            theme::text_secondary(),
        )
        .rect
        .width();
    (width + 38.0).max(58.0)
}

fn result_meta_chip_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width()
        + 18.0
}

fn result_meta_chip_svg_width(ui: &egui::Ui, text: &str) -> f32 {
    let text_width = ui
        .painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width();
    (text_width + 34.0).max(74.0)
}

pub fn add_empty_row(state: &mut AppState) {
    let col_count = state
        .current_result
        .as_ref()
        .map(|r| r.columns.len())
        .unwrap_or(0);
    if col_count == 0 {
        return;
    }

    let new_row: Vec<CellValue> = vec![CellValue::Null; col_count];
    let row_idx = state
        .current_result
        .as_ref()
        .map(|r| r.rows.len())
        .unwrap_or(0);

    if let Some(result) = state.current_result.as_mut() {
        result.rows.push(new_row);
    }

    state.data_edit.inserted_rows.insert(row_idx);

    for col_idx in 0..col_count {
        state.data_edit.cells.insert(
            (row_idx, col_idx),
            crate::state::EditableCell {
                original: CellValue::Null,
                original_text: String::new(),
                value: String::new(),
                is_null: true,
            },
        );
    }

    state.data_edit.selected_cell = Some((row_idx, 0));
}

