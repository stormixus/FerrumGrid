//! 스키마 diff 시각화 윈도우.

use eframe::egui::{self, Margin, RichText, Stroke};

use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_schema_diff_window(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_schema_diff_window {
        return;
    }
    let mut open = true;
    egui::Window::new(t("schema_diff_title"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(640.0)
        .default_height(420.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_LG as i8)),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("schema_diff_hint"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_SM);
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    if state.schema_diff_rows.is_empty() {
                        ui.label(
                            RichText::new(t("schema_diff_empty"))
                                .color(theme::text_muted())
                                .size(11.0),
                        );
                        return;
                    }
                    for row in &state.schema_diff_rows {
                        ui.label(
                            RichText::new(row)
                                .color(theme::text_secondary())
                                .size(11.0)
                                .monospace(),
                        );
                    }
                });
        });
    if !open {
        state.show_schema_diff_window = false;
    }
}