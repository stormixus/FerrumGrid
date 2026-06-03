//! 객체 권한(ACL) 브라우저 + GRANT/REVOKE 폼 (floating window).
//! 기존 롤 뷰는 전역 속성만 보여주므로 per-object 권한 관리를 보완.

use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::db::privileges::{build_grant_sql, GrantObject};
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

/// 객체 종류별 (코드, 라벨키, 허용 권한 목록).
const OBJECT_TYPES: &[(&str, &str, &[&str])] = &[
    ("table", "privileges_obj_table", &["SELECT", "INSERT", "UPDATE", "DELETE", "TRUNCATE", "ALL"]),
    ("sequence", "privileges_obj_sequence", &["USAGE", "SELECT", "UPDATE", "ALL"]),
    ("functions", "privileges_obj_functions", &["EXECUTE", "ALL"]),
];

fn privileges_for(object: &str) -> &'static [&'static str] {
    OBJECT_TYPES
        .iter()
        .find(|(code, _, _)| *code == object)
        .map(|(_, _, privs)| *privs)
        .unwrap_or(OBJECT_TYPES[0].2)
}

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
                    let is_functions = state.grant_form_object == "functions";
                    ui.horizontal_wrapped(|ui| {
                        // 객체 종류 선택 — 종류가 바뀌면 권한 목록도 바뀐다.
                        egui::ComboBox::from_id_salt("grant_object")
                            .selected_text(t(OBJECT_TYPES
                                .iter()
                                .find(|(c, _, _)| *c == state.grant_form_object)
                                .map(|(_, label, _)| *label)
                                .unwrap_or("privileges_obj_table")))
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for (code, label, _) in OBJECT_TYPES {
                                    ui.selectable_value(
                                        &mut state.grant_form_object,
                                        code.to_string(),
                                        t(label),
                                    );
                                }
                            });
                        // 선택 권한이 현 객체에 유효하지 않으면 첫 권한으로 보정.
                        let privs = privileges_for(&state.grant_form_object);
                        if !privs.contains(&state.grant_form_privilege.as_str()) {
                            state.grant_form_privilege = privs[0].to_string();
                        }
                        egui::ComboBox::from_id_salt("grant_priv")
                            .selected_text(&state.grant_form_privilege)
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                for p in privs {
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
                                .hint_text(if is_functions { "schema" } else { "schema.table" })
                                .desired_width(160.0),
                        );
                        ui.label(RichText::new(t("privileges_grantee")).size(11.0).color(theme::text_muted()));
                        ui.add(
                            theme::mono_text_input(&mut state.grant_form_grantee)
                                .hint_text("role / PUBLIC")
                                .desired_width(120.0),
                        );
                        // functions = 스키마만 필요, 그 외 = schema.name 필요.
                        let valid = !state.grant_form_grantee.trim().is_empty()
                            && !state.grant_form_target.trim().is_empty()
                            && (is_functions || state.grant_form_target.contains('.'));
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
            let (object, schema, name) = match state.grant_form_object.as_str() {
                "sequence" => {
                    let (s, n) = state
                        .grant_form_target
                        .split_once('.')
                        .unwrap_or(("public", state.grant_form_target.as_str()));
                    (GrantObject::Sequence, s.to_string(), n.to_string())
                }
                "functions" => (
                    GrantObject::AllFunctions,
                    state.grant_form_target.trim().to_string(),
                    String::new(),
                ),
                _ => {
                    let (s, n) = state
                        .grant_form_target
                        .split_once('.')
                        .unwrap_or(("public", state.grant_form_target.as_str()));
                    (GrantObject::Table, s.to_string(), n.to_string())
                }
            };
            let sql = build_grant_sql(
                grant,
                &state.grant_form_privilege,
                object,
                &schema,
                &name,
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
