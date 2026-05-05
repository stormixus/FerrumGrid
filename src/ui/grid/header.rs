//! Result header — top strip with row/col/exec metadata + toolbar buttons.
//!
//! Plan v7 US-G3 — extracted from `render.rs`.

use eframe::egui::{self, CornerRadius, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{AppState, MainView};
use crate::ui::theme;

use super::data_ops::{data_edit_summary, revert_data_edits};
use super::pager::render_data_pager;
use super::paste::{export_csv, result_to_tsv};
use super::toolbar::{
    metric_chip, result_meta_chip, result_meta_chip_svg, result_toolbar_action_button,
};

pub fn render_result_header(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;
    let data_edit_summary = data_edit_summary(state);

    let header_height = 56.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, theme::bg_shell());
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::border_subtle()),
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, 0.0));
    let content_rect = egui::Rect::from_center_size(
        inner.center(),
        egui::vec2(inner.width(), theme::BUTTON_HEIGHT),
    );
    let tsv_width = result_toolbar_action_width(ui, "Copy TSV");
    let csv_width = result_toolbar_action_width(ui, "CSV");
    let mut right_width = tsv_width + csv_width + theme::SPACE_SM;
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
                    .size(13.0),
            );
            ui.add_space(theme::SPACE_MD);
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    row_count,
                    if row_count == 1 { "row" } else { "rows" }
                ),
                theme::ACCENT_TEAL,
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

            let csv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::EXPORT,
                "export_csv",
                "CSV",
                true,
            );

            if csv_btn.clicked() {
                export_csv(state);
            }

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
        },
    );
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

