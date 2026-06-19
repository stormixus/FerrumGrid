//! `pg_stat_activity` + `pg_locks` 실시간 모니터 윈도우.

use eframe::egui::{self, Margin, RichText, Stroke};

use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_session_monitor(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_session_monitor {
        return;
    }
    let mut open = true;
    egui::Window::new(t("session_monitor_title"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(720.0)
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
                RichText::new(t("session_monitor_hint"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_SM);
            ui.label("pid | state | query | duration");
            ui.add_space(theme::SPACE_XS);
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for s in &state.session_monitor_rows {
                    ui.label(
                        RichText::new(format!(
                            "{} | {} | {} | {}s",
                            s.pid,
                            s.state,
                            s.query.chars().take(80).collect::<String>(),
                            s.duration_secs
                        ))
                        .color(theme::text_secondary())
                        .size(11.0)
                        .monospace(),
                    );
                }
            });
        });
    if !open {
        state.show_session_monitor = false;
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionRow {
    pub pid: i32,
    pub state: String,
    pub query: String,
    pub duration_secs: i64,
}