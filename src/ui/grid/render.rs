//! Grid render entry — top-level render_grid + error/empty states.
//!
//! Plan v7 Phase 1.95c3c cut-over (from `super::mod.rs`). render_result_header
//! 는 ~1000줄 단일 함수 + 다수 헬퍼 의존성으로 별도 sub-iteration 에서 cut-over.
//! 본 모듈은 entry + 작은 helper 만 호스트.

use eframe::egui::{self, Color32, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::state::AppState;
use crate::ui::theme;

use super::{
    render_data_query_footer, render_grid_body_with_reserved_footer, render_result_header,
    render_table, should_show_data_query_footer,
};

pub fn render_grid(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if let Some(ref error) = state.last_error.clone() {
        render_error_bar(ui, error);
    }

    match &state.current_result {
        None => {
            if should_show_data_query_footer(state) {
                render_grid_body_with_reserved_footer(ui, |ui| {
                    render_empty_state(ui, state.query_running);
                });
                render_data_query_footer(ui, state);
            } else {
                render_empty_state(ui, state.query_running);
            }
        }
        Some(_) => {
            render_result_header(ui, state, bridge);
            if should_show_data_query_footer(state) {
                render_grid_body_with_reserved_footer(ui, |ui| {
                    render_table(ui, state, bridge);
                });
                render_data_query_footer(ui, state);
            } else {
                render_table(ui, state, bridge);
            }
        }
    }
}

fn render_error_bar(ui: &mut egui::Ui, error: &str) {
    let frame = egui::Frame::new()
        .fill(theme::with_alpha(theme::ACCENT_RED, 28))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_RED, 86)));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            crate::ui::icon_img(ui, crate::ui::icons_svg::ERROR, "grid_err", 12.0);
            ui.add_space(4.0);
            ui.label(
                RichText::new("Error")
                    .color(theme::ACCENT_RED)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new(error)
                    .color(Color32::from_rgb(220, 150, 150))
                    .size(12.0),
            );
        });
    });
}

fn render_empty_state(ui: &mut egui::Ui, running: bool) {
    ui.centered_and_justified(|ui| {
        if running {
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(theme::SPACE_MD);
                ui.label(
                    RichText::new("Executing query...")
                        .color(theme::text_muted())
                        .size(12.0),
                );
            });
        } else {
            ui.vertical_centered(|ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::TABLE, "grid_empty", 34.0);
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new("No result set")
                        .color(theme::text_muted())
                        .strong()
                        .size(13.0),
                );
                ui.label(
                    RichText::new("Run a query to populate the grid")
                        .color(theme::text_disabled())
                        .size(11.0),
                );
            });
        }
    });
}
