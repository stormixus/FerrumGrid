//! DBA 세션 모니터 (floating window) — pg_stat_activity 표시 + cancel/terminate.
//!
//! 창이 열려 있으면 `sessions_needs_fetch` 플래그로 `ListSessions` 발사.
//! Cancel = pg_cancel_backend (실행 문장만 취소), Terminate = pg_terminate_backend
//! (연결 종료, 파괴적이므로 2-step 확인).

use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_sessions_window(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_sessions_window {
        return;
    }

    // 창이 열려 있고 fetch 요청이 있으면 조회.
    if state.sessions_needs_fetch {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::ListSessions { conn_id });
        }
        state.sessions_needs_fetch = false;
    }

    let mut open = true;
    let mut refresh = false;
    let mut kill: Option<(i32, bool)> = None;

    egui::Window::new(t("sessions_window_title"))
        .open(&mut open)
        .default_size([760.0, 420.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(theme::secondary_button(&t("sessions_refresh"))).clicked() {
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
            ui.separator();

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("sessions_grid")
                        .striped(true)
                        .spacing([theme::SPACE_MD, theme::SPACE_XS])
                        .show(ui, |ui| {
                            for h in [
                                "PID", "User", "DB", "State", "Wait", "Start", "Query", "",
                            ] {
                                ui.label(
                                    RichText::new(h)
                                        .color(theme::text_secondary())
                                        .strong()
                                        .size(11.0),
                                );
                            }
                            ui.end_row();

                            for s in &state.sessions {
                                ui.label(RichText::new(s.pid.to_string()).monospace().size(11.0));
                                ui.label(RichText::new(&s.user).size(11.0));
                                ui.label(RichText::new(&s.database).size(11.0));
                                let state_color = match s.state.as_str() {
                                    "active" => theme::ACCENT_GREEN,
                                    "idle in transaction" => theme::ACCENT_RED,
                                    _ => theme::text_muted(),
                                };
                                ui.label(RichText::new(&s.state).color(state_color).size(11.0));
                                ui.label(
                                    RichText::new(&s.wait_event)
                                        .color(theme::text_muted())
                                        .size(10.5),
                                );
                                ui.label(
                                    RichText::new(&s.query_start)
                                        .monospace()
                                        .color(theme::text_muted())
                                        .size(10.5),
                                );
                                let q: String =
                                    s.query.chars().take(60).collect::<String>().replace('\n', " ");
                                ui.label(RichText::new(q).monospace().size(10.5)).on_hover_text(
                                    format!(
                                        "client: {}    app: {}\n\n{}",
                                        if s.client_addr.is_empty() { "local" } else { &s.client_addr },
                                        if s.application_name.is_empty() { "-" } else { &s.application_name },
                                        s.query
                                    ),
                                );

                                ui.horizontal(|ui| {
                                    if ui
                                        .add(theme::secondary_button(&t("sessions_cancel")))
                                        .on_hover_text(t("sessions_cancel_hint"))
                                        .clicked()
                                    {
                                        kill = Some((s.pid, false));
                                    }
                                    if state.sessions_confirm_terminate == Some(s.pid) {
                                        let confirm = egui::Button::new(
                                            RichText::new(t("sessions_confirm"))
                                                .color(egui::Color32::WHITE)
                                                .size(11.0),
                                        )
                                        .fill(theme::ACCENT_RED);
                                        if ui.add(confirm).clicked() {
                                            kill = Some((s.pid, true));
                                        }
                                    } else if ui
                                        .add(theme::secondary_button(&t("sessions_terminate")))
                                        .on_hover_text(t("sessions_terminate_hint"))
                                        .clicked()
                                    {
                                        state.sessions_confirm_terminate = Some(s.pid);
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });
        });

    if refresh {
        state.sessions_needs_fetch = true;
    }
    if let Some((pid, terminate)) = kill {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::KillBackend {
                conn_id,
                pid,
                terminate,
            });
        }
        state.sessions_confirm_terminate = None;
    }
    if !open {
        state.show_sessions_window = false;
        state.sessions_confirm_terminate = None;
    }
}
