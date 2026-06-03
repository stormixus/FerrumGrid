//! 객체 권한(ACL) 브라우저 + GRANT/REVOKE 폼 (floating window).
//! 기존 롤 뷰는 전역 속성만 보여주므로 per-object 권한 관리를 보완.

use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::db::privileges::build_grant_sql;
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

const PRIVILEGES: &[&str] = &["SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE", "ALL"];

pub fn render_privileges_window(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_privileges_window {
        return;
    }
    if state.privileges_needs_fetch {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::ListGrants { conn_id });
        }
        state.privileges_needs_fetch = false;
    }

    let mut open = true;
    let mut apply: Option<bool> = None; // Some(true)=grant, Some(false)=revoke
    let mut refresh = false;

    egui::Window::new(t("privileges_window_title"))
        .open(&mut open)
        .default_size([680.0, 480.0])
        .resizable(true)
        .show(ctx, |ui| {
            // --- GRANT/REVOKE 폼 ---
            egui::Frame::new()
                .fill(theme::bg_medium())
                .inner_margin(egui::Margin::same(theme::SPACE_SM as i8))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_MD))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(t("privileges_priv")).size(11.0).color(theme::text_muted()));
                        egui::ComboBox::from_id_salt("grant_priv")
                            .selected_text(&state.grant_form_privilege)
                            .width(110.0)
                            .show_ui(ui, |ui| {
                                for p in PRIVILEGES {
                                    ui.selectable_value(
                                        &mut state.grant_form_privilege,
                                        p.to_string(),
                                        *p,
                                    );
                                }
                            });
                        ui.label(RichText::new(t("privileges_on")).size(11.0).color(theme::text_muted()));
                        ui.add(
                            theme::mono_text_input(&mut state.grant_form_target)
                                .hint_text("schema.table")
                                .desired_width(160.0),
                        );
                        ui.label(RichText::new(t("privileges_grantee")).size(11.0).color(theme::text_muted()));
                        ui.add(
                            theme::mono_text_input(&mut state.grant_form_grantee)
                                .hint_text("role / PUBLIC")
                                .desired_width(120.0),
                        );
                        let valid = state.grant_form_target.contains('.')
                            && !state.grant_form_grantee.trim().is_empty();
                        if ui
                            .add_enabled(valid, theme::secondary_button(&t("privileges_grant")))
                            .clicked()
                        {
                            apply = Some(true);
                        }
                        if ui
                            .add_enabled(valid, theme::secondary_button(&t("privileges_revoke")))
                            .clicked()
                        {
                            apply = Some(false);
                        }
                    });
                });

            ui.add_space(theme::SPACE_XS);
            ui.horizontal(|ui| {
                if ui.add(theme::secondary_button(&t("sessions_refresh"))).clicked() {
                    refresh = true;
                }
                ui.label(
                    RichText::new(format!("{} grants", state.grants.len()))
                        .color(theme::text_muted())
                        .size(11.0),
                );
            });
            ui.separator();

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("grants_grid")
                        .striped(true)
                        .spacing([theme::SPACE_MD, theme::SPACE_XS])
                        .show(ui, |ui| {
                            for h in ["Object", "Grantee", "Privilege"] {
                                ui.label(
                                    RichText::new(h)
                                        .color(theme::text_secondary())
                                        .strong()
                                        .size(11.0),
                                );
                            }
                            ui.end_row();
                            for g in &state.grants {
                                ui.label(
                                    RichText::new(format!("{}.{}", g.schema, g.table))
                                        .monospace()
                                        .size(11.0),
                                );
                                ui.label(RichText::new(&g.grantee).size(11.0));
                                ui.label(
                                    RichText::new(&g.privilege)
                                        .color(theme::accent_color())
                                        .size(11.0),
                                );
                                ui.end_row();
                            }
                        });
                });
        });

    if let Some(grant) = apply {
        if let Some(conn_id) = state.active_connection {
            let (schema, table) = state
                .grant_form_target
                .split_once('.')
                .unwrap_or(("public", state.grant_form_target.as_str()));
            let sql = build_grant_sql(
                grant,
                &state.grant_form_privilege,
                schema,
                table,
                state.grant_form_grantee.trim(),
            );
            // 순차 처리: GRANT/REVOKE 가 먼저 실행된 뒤 목록 재조회.
            bridge.send(DbCommand::ApplyDdlWithInvalidation {
                conn_id,
                sql,
                table_oid: None,
                schema_to_refresh: None,
            });
            state.privileges_needs_fetch = true;
        }
    }
    if refresh {
        state.privileges_needs_fetch = true;
    }
    if !open {
        state.show_privileges_window = false;
    }
}
