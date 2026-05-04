//! Grid hover tooltip helpers — dark themed, screen-bounded smart positioning.
//!
//! Plan v7 Phase 1.95c3 cut-over (from `super::mod.rs`).

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::ui::theme;

pub(super) fn show_dark_hover_tooltip(
    ui: &egui::Ui,
    tooltip_id: egui::Id,
    response: &egui::Response,
    text: &str,
) {
    if !response.hovered() {
        return;
    }

    let pointer = ui
        .ctx()
        .pointer_hover_pos()
        .unwrap_or_else(|| response.rect.left_top());
    let max_width = 720.0;
    let pos = smart_tooltip_pos(ui.ctx(), pointer, estimate_tooltip_size(text, max_width));
    egui::Area::new(tooltip_id)
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_MD_I))
                .show(ui, |ui| {
                    ui.set_max_width(max_width);
                    ui.add(
                        egui::Label::new(
                            RichText::new(text)
                                .color(theme::text_secondary())
                                .monospace()
                                .size(11.0),
                        )
                        .wrap(),
                    );
                });
        });
}

fn smart_tooltip_pos(
    ctx: &egui::Context,
    anchor: egui::Pos2,
    estimated_size: egui::Vec2,
) -> egui::Pos2 {
    let bounds = ctx.screen_rect().shrink(8.0);
    let gap = 12.0;
    let right_x = anchor.x + gap;
    let left_x = anchor.x - gap - estimated_size.x;
    let bottom_y = anchor.y + gap;
    let top_y = anchor.y - gap - estimated_size.y;

    let x = if right_x + estimated_size.x <= bounds.right() {
        right_x
    } else if left_x >= bounds.left() {
        left_x
    } else {
        clamp_axis(right_x, bounds.left(), bounds.right() - estimated_size.x)
    };

    let y = if bottom_y + estimated_size.y <= bounds.bottom() {
        bottom_y
    } else if top_y >= bounds.top() {
        top_y
    } else {
        clamp_axis(bottom_y, bounds.top(), bounds.bottom() - estimated_size.y)
    };

    egui::pos2(x, y)
}

fn estimate_tooltip_size(text: &str, max_width: f32) -> egui::Vec2 {
    let char_width = 7.2;
    let content_max = (max_width - theme::SPACE_MD * 2.0).max(80.0);
    let mut visual_lines = 0.0_f32;
    let mut widest = 0.0_f32;

    for line in text.lines().chain((text.is_empty()).then_some("")) {
        let line_width = line.chars().count() as f32 * char_width;
        widest = widest.max(line_width);
        visual_lines += (line_width / content_max).ceil().max(1.0);
    }

    let width = (widest + theme::SPACE_MD * 2.0).clamp(48.0, max_width);
    let height = visual_lines * 15.0 + theme::SPACE_MD * 2.0;
    egui::vec2(width, height)
}

fn clamp_axis(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        min
    } else {
        value.clamp(min, max)
    }
}
