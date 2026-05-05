//! Data pager — page navigation toolbar (prev/next/limit input).
//!
//! Plan v7 US-G3 — extracted from `render.rs`.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{AppState, MainView};
use crate::ui::theme;

use super::data_ops::{
    apply_data_limit_input, apply_data_page_input, data_page_offset, normalized_data_limit,
    set_data_page_index,
};
use super::selection::enter_pressed;
use super::toolbar::{
    result_popup_apply_button, result_popup_field_row, result_toolbar_button_frame,
    result_toolbar_menu_button,
};
use super::tooltips::show_dark_hover_tooltip;

pub(super) fn render_data_pager(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    has_next_page: bool,
    visible_rows: usize,
) {
    if state.active_main_view != MainView::Data {
        return;
    }

    let page_index = state.data_edit.page_index;
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    let page_start = if visible_rows == 0 { 0 } else { offset + 1 };
    let page_end = if visible_rows == 0 {
        0
    } else {
        offset + visible_rows
    };
    let page_label = tf("grid_page_n", &[&(page_index + 1).to_string()]);
    let limit_label = tf("grid_limit_n", &[&limit.to_string()]);
    let range_label = tf(
        "grid_visible_range",
        &[&page_start.to_string(), &page_end.to_string()],
    );

    egui::Frame::new()
        .fill(theme::bg_darkest())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(theme::SPACE_SM_I, theme::SPACE_XS_I))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
                let first = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_DOUBLE_LEFT,
                    "data_first_page",
                    &t("grid_first_page"),
                    page_index > 0 && !state.query_running,
                );
                if first.clicked() {
                    set_data_page_index(state, bridge, 0);
                }

                let prev = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_LEFT,
                    "data_prev_page",
                    &t("grid_prev_page"),
                    page_index > 0 && !state.query_running,
                );
                if prev.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_sub(1));
                }

                result_toolbar_menu_button(ui, "page", &page_label, 64.0, |ui| {
                    result_popup_field_row(ui, &t("grid_page"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_index_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_page_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_page_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                let next = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_RIGHT,
                    "data_next_page",
                    &t("grid_next_page"),
                    has_next_page && !state.query_running,
                );
                if next.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_add(1));
                }

                ui.add_space(theme::SPACE_SM);
                result_toolbar_menu_button(ui, "limit", &limit_label, 78.0, |ui| {
                    result_popup_field_row(ui, &t("grid_limit"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_limit_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_limit_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_limit_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(range_label)
                        .color(theme::text_muted())
                        .size(11.0),
                );
            });
        });
}

fn pager_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
    enabled: bool,
) -> egui::Response {
    let color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(26.0, 26.0), enabled);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, color,
            ));
        },
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    response
}

