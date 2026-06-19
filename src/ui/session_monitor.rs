//! Read-only pg_stat_activity monitor window.

use eframe::egui::{self, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_session_monitor(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_session_monitor {
        return;
    }

    if state.sessions_needs_fetch {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::ListSessions { conn_id });
        }
        state.sessions_needs_fetch = false;
    }

    let mut open = true;
    let mut refresh = false;
    egui::Window::new(t("session_monitor_title"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(820.0)
        .default_height(460.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_LG as i8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(theme::secondary_button(&t("sessions_refresh")))
                    .clicked()
                {
                    refresh = true;
                }
                ui.label(
                    RichText::new(format!("{} sessions", state.sessions.len()))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                if state.active_connection.is_none() {
                    ui.label(
                        RichText::new(t("sessions_no_connection"))
                            .color(theme::ACCENT_YELLOW)
                            .size(11.0),
                    );
                }
            });
            ui.add_space(theme::SPACE_SM);
            ui.label(
                RichText::new(t("session_monitor_hint"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.separator();

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("session_monitor_grid")
                        .striped(true)
                        .spacing([theme::SPACE_MD, theme::SPACE_XS])
                        .show(ui, |ui| {
                            for h in ["PID", "User", "DB", "State", "Wait", "Start", "Query"] {
                                ui.label(
                                    RichText::new(h)
                                        .color(theme::text_secondary())
                                        .strong()
                                        .size(11.0),
                                );
                            }
                            ui.end_row();

                            for session in &state.sessions {
                                ui.label(
                                    RichText::new(session.pid.to_string())
                                        .monospace()
                                        .size(11.0),
                                );
                                ui.label(RichText::new(&session.user).size(11.0));
                                ui.label(RichText::new(&session.database).size(11.0));
                                let state_color = match session.state.as_str() {
                                    "active" => theme::ACCENT_GREEN,
                                    "idle in transaction" => theme::ACCENT_RED,
                                    _ => theme::text_muted(),
                                };
                                ui.label(
                                    RichText::new(&session.state).color(state_color).size(11.0),
                                );
                                ui.label(
                                    RichText::new(&session.wait_event)
                                        .color(theme::text_muted())
                                        .size(10.5),
                                );
                                ui.label(
                                    RichText::new(&session.query_start)
                                        .monospace()
                                        .color(theme::text_muted())
                                        .size(10.5),
                                );
                                let preview = session
                                    .query
                                    .chars()
                                    .take(90)
                                    .collect::<String>()
                                    .replace('\n', " ");
                                ui.label(RichText::new(preview).monospace().size(10.5))
                                    .on_hover_text(format!(
                                        "client: {}    app: {}\n\n{}",
                                        if session.client_addr.is_empty() {
                                            "local"
                                        } else {
                                            &session.client_addr
                                        },
                                        if session.application_name.is_empty() {
                                            "-"
                                        } else {
                                            &session.application_name
                                        },
                                        session.query
                                    ));
                                ui.end_row();
                            }
                        });
                });
        });

    if refresh {
        state.sessions_needs_fetch = true;
    }
    if !open {
        state.show_session_monitor = false;
    }
}
