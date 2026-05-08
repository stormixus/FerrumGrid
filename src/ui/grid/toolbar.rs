//! Result toolbar primitives — action buttons, popups, chips.
//!
//! Plan v7 US-F3 — extracted from `render.rs`. Hosts toolbar buttons,
//! dark popups, popup field rows, meta chips.

use std::hash::Hash;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::ui::theme;

use super::selection::set_pointing_cursor_on_hover;

pub fn result_toolbar_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let font = egui::FontId::proportional(12.0);
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
    let width = (text_width + 38.0).max(58.0);
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icon_svg,
                icon_name,
                12.0,
                if enabled {
                    theme::ACCENT_EMERALD
                } else {
                    theme::text_disabled()
                },
            ));
        },
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(28.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    response
}

pub(super) fn result_toolbar_menu_button<R>(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    label: &str,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    let popup_id = ui.make_persistent_id(("result_toolbar_menu", id_source));
    let response = result_toolbar_text_button(ui, label, width, true);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }
    show_dark_popup_below(
        ui,
        popup_id,
        &response,
        160.0,
        theme::SPACE_MD_I,
        add_contents,
    );
    response
}

pub fn show_dark_popup_below<R>(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    response: &egui::Response,
    min_width: f32,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    if !ui.memory(|memory| memory.is_popup_open(popup_id)) {
        return;
    }

    let mut pos = response.rect.left_bottom() + egui::vec2(0.0, 4.0);
    if let Some(to_global) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        pos = to_global * pos;
    }
    let popup = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(margin))
                .show(ui, |ui| {
                    ui.set_width(min_width);
                    add_contents(ui);
                });
        });

    let should_close = ui.input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && popup.response.clicked_elsewhere());
    if should_close {
        ui.memory_mut(|memory| memory.close_popup());
    }
}

pub(super) fn result_popup_field_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_field: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = COLUMN_GAP;
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_WIDTH, theme::INPUT_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    RichText::new(label)
                        .color(theme::text_secondary())
                        .size(11.5)
                        .strong(),
                );
            },
        );
        add_field(ui)
    })
    .inner
}

pub(super) fn result_popup_apply_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + COLUMN_GAP);
        let width = ui.available_width().max(68.0);
        result_popup_action_button(ui, label, enabled, width)
    })
    .inner
}

pub(super) fn result_popup_action_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    width: f32,
) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );
    response
}

pub(super) fn result_toolbar_text_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 26.0), enabled);
    ui.painter().text(
        rect.center() - egui::vec2(4.0, 0.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.5),
        if enabled {
            theme::text_secondary()
        } else {
            theme::text_disabled()
        },
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(10.0, 1.0),
        egui::Align2::CENTER_CENTER,
        "⌄",
        egui::FontId::proportional(10.0),
        theme::text_muted(),
    );
    response
}

pub(super) fn result_toolbar_button_frame(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    enabled: bool,
) -> (egui::Rect, egui::Response) {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let hovered = enabled && response.hovered();
    let fill = if !enabled {
        theme::bg_darkest()
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_EMERALD, 150))
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
    set_pointing_cursor_on_hover(ui, &response, enabled);
    (rect, response)
}

pub(super) fn result_meta_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 18.0, 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 2.7, color);
    painter.text(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

pub(super) fn result_meta_chip_svg(ui: &mut egui::Ui, text: &str, svg: &str, name: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2((galley.rect.width() + 34.0).max(74.0), 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(11.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(ui, svg, name, 12.0, color));
        },
    );
    painter.text(
        rect.left_center() + egui::vec2(22.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

pub fn metric_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 26.0, 20.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    ui.painter()
        .circle_filled(rect.left_center() + egui::vec2(11.0, 0.0), 2.5, color);
    ui.painter().text(
        rect.left_center() + egui::vec2(17.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

// ---------------------------------------------------------------------------
// Header / Sort
// ---------------------------------------------------------------------------

