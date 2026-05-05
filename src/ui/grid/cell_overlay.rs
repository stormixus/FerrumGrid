//! Cell overlay widgets — null toggle, text input, bool checkbox, value
//! placeholder. Plan v7 US-F3 — extracted from `selection.rs`.

use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke};

use crate::i18n::t;
use crate::ui::theme;

use super::selection::{enter_pressed, set_pointing_cursor_on_hover};
use super::tooltips::show_dark_hover_tooltip;
use super::{parse_bool, GRID_CELL_RIGHT_PAD};

pub(super) fn cell_overlay_editor_rect(cell_rect: egui::Rect, nullable: bool) -> egui::Rect {
    let left_pad = if nullable { 38.0 } else { 0.0 };
    let left = cell_rect.left() + left_pad;
    let right = (cell_rect.right() - GRID_CELL_RIGHT_PAD).max(left + 28.0);
    egui::Rect::from_min_max(
        egui::pos2(left, cell_rect.center().y - 12.0),
        egui::pos2(right, cell_rect.center().y + 12.0),
    )
}

pub(super) fn cell_overlay_null_toggle(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    checked: bool,
) -> egui::Response {
    let rect = egui::Rect::from_center_size(
        egui::pos2(cell_rect.left() + 17.0, cell_rect.center().y),
        egui::vec2(32.0, 18.0),
    );
    let response = ui.interact(
        rect,
        ui.make_persistent_id(("cell_null_toggle", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let fill = if checked {
        theme::with_alpha(theme::ACCENT_TEAL, if hovered { 52 } else { 34 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if checked {
        theme::ACCENT_TEAL
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "NULL",
        egui::FontId::proportional(9.5),
        if checked {
            theme::ACCENT_TEAL
        } else {
            theme::text_muted()
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("grid_toggle_null"),
    );
    response
}

pub(super) fn render_cell_text_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
    monospace: bool,
    error: Option<&str>,
) -> bool {
    let response = ui.put(
        rect,
        cell_overlay_text_input(&mut edit.value, monospace).desired_width(rect.width()),
    );
    response.request_focus();
    if let Some(error) = error {
        show_dark_hover_tooltip(ui, response.id.with("error"), &response, error);
    }
    enter_pressed(ui)
}

fn cell_overlay_text_input(text: &mut String, monospace: bool) -> egui::TextEdit<'_> {
    let input = egui::TextEdit::singleline(text)
        .background_color(theme::bg_darkest())
        .text_color(theme::text_primary())
        .margin(Margin::symmetric(7, 2))
        .min_size(egui::vec2(0.0, 24.0))
        .vertical_align(egui::Align::Center);
    if monospace {
        input.font(egui::TextStyle::Monospace)
    } else {
        input.font(egui::TextStyle::Body)
    }
}

pub(super) fn render_cell_bool_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
) -> bool {
    let mut checked = parse_bool(&edit.value).unwrap_or(false);
    let response = ui.put(rect, egui::Checkbox::new(&mut checked, ""));
    if response.changed() {
        edit.value = checked.to_string();
    }
    response.lost_focus() && enter_pressed(ui)
}

pub(super) fn render_cell_overlay_value(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    text: &str,
    color: Color32,
) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::bg_darkest(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::monospace(12.0),
        color,
    );
}
