//! 카탈로그 객체 브라우저 (floating window) — 시퀀스 / enum 타입 / 익스텐션.
//! 기존 객체 브라우저(테이블/뷰/함수/롤)가 노출하지 않는 1급 객체들을 표시.

use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_catalog_window(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_catalog_window {
        return;
    }
    if state.catalog_needs_fetch {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::ListCatalog { conn_id });
        }
        state.catalog_needs_fetch = false;
    }

    let mut open = true;
    let mut refresh = false;
    let mut ddl: Option<String> = None;
    let mut confirm_key: Option<String> = None;

    egui::Window::new(t("catalog_window_title"))
        .open(&mut open)
        .default_size([660.0, 480.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add(theme::secondary_button(&t("sessions_refresh"))).clicked() {
                    refresh = true;
                }
                ui.separator();
                ui.add(
                    theme::mono_text_input(&mut state.catalog_new_extension)
                        .hint_text("pg_trgm")
                        .desired_width(140.0),
                );
                let name = state.catalog_new_extension.trim().to_string();
                if ui
                    .add_enabled(!name.is_empty(), theme::secondary_button(&t("catalog_install_ext")))
                    .clicked()
                {
                    ddl = Some(format!("CREATE EXTENSION IF NOT EXISTS \"{}\";", name.replace('"', "")));
                    state.catalog_new_extension.clear();
                }
            });
            ui.add_space(theme::SPACE_XS);
            ui.horizontal(|ui| {
                ui.add(
                    theme::mono_text_input(&mut state.catalog_new_sequence)
                        .hint_text("schema.seq_name (or seq_name)")
                        .desired_width(200.0),
                );
                let raw = state.catalog_new_sequence.trim().to_string();
                if ui
                    .add_enabled(!raw.is_empty(), theme::secondary_button(&t("catalog_new_seq")))
                    .clicked()
                {
                    let (schema, name) = raw
                        .split_once('.')
                        .map(|(s, n)| (s.to_string(), n.to_string()))
                        .unwrap_or_else(|| ("public".to_string(), raw.clone()));
                    ddl = Some(format!(
                        "CREATE SEQUENCE \"{}\".\"{}\";",
                        schema.replace('"', ""),
                        name.replace('"', "")
                    ));
                    state.catalog_new_sequence.clear();
                }
            });
            ui.add_space(theme::SPACE_XS);

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let Some(catalog) = &state.catalog else {
                        ui.label(
                            RichText::new(t("catalog_loading"))
                                .color(theme::text_muted())
                                .size(11.0),
                        );
                        return;
                    };

                    section(ui, &t("catalog_extensions"), catalog.extensions.len());
                    for e in &catalog.extensions {
                        let key = format!("ext:{}", e.name);
                        if let Some(d) = drop_row(
                            ui,
                            &e.name,
                            &format!("v{}", e.version),
                            &key,
                            state.catalog_confirm_drop.as_deref(),
                            &format!("DROP EXTENSION \"{}\";", e.name.replace('"', "")),
                            &mut confirm_key,
                        ) {
                            ddl = Some(d);
                        }
                    }

                    ui.add_space(theme::SPACE_MD);
                    section(ui, &t("catalog_enums"), catalog.enums.len());
                    for en in &catalog.enums {
                        let key = format!("type:{}.{}", en.schema, en.name);
                        if let Some(d) = drop_row(
                            ui,
                            &format!("{}.{}", en.schema, en.name),
                            &en.labels,
                            &key,
                            state.catalog_confirm_drop.as_deref(),
                            &format!("DROP TYPE \"{}\".\"{}\";", en.schema, en.name),
                            &mut confirm_key,
                        ) {
                            ddl = Some(d);
                        }
                    }

                    ui.add_space(theme::SPACE_MD);
                    section(ui, &t("catalog_sequences"), catalog.sequences.len());
                    for s in &catalog.sequences {
                        let key = format!("seq:{}.{}", s.schema, s.name);
                        if let Some(d) = drop_row(
                            ui,
                            &format!("{}.{}", s.schema, s.name),
                            &format!("{} start {} inc {}", s.data_type, s.start_value, s.increment),
                            &key,
                            state.catalog_confirm_drop.as_deref(),
                            &format!("DROP SEQUENCE \"{}\".\"{}\";", s.schema, s.name),
                            &mut confirm_key,
                        ) {
                            ddl = Some(d);
                        }
                    }
                });
        });

    if let Some(key) = confirm_key {
        state.catalog_confirm_drop = Some(key);
    }
    if let Some(sql) = ddl {
        if let Some(conn_id) = state.active_connection {
            bridge.send(DbCommand::ApplyDdlWithInvalidation {
                conn_id,
                sql,
                table_oid: None,
                schema_to_refresh: None,
            });
            state.catalog_needs_fetch = true;
        }
        state.catalog_confirm_drop = None;
    }
    if refresh {
        state.catalog_needs_fetch = true;
    }
    if !open {
        state.show_catalog_window = false;
        state.catalog_confirm_drop = None;
    }
}

/// 객체 행 + DROP 버튼 (2-step 확인). 실제 실행할 DDL 을 반환하거나 confirm 키를 설정.
fn drop_row(
    ui: &mut egui::Ui,
    name: &str,
    detail: &str,
    key: &str,
    pending: Option<&str>,
    drop_sql: &str,
    confirm_key: &mut Option<String>,
) -> Option<String> {
    let mut result = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(name)
                .color(theme::text_primary())
                .monospace()
                .size(11.5),
        );
        ui.label(RichText::new(detail).color(theme::text_muted()).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if pending == Some(key) {
                let confirm = egui::Button::new(
                    RichText::new(t("catalog_confirm_drop"))
                        .color(egui::Color32::WHITE)
                        .size(10.5),
                )
                .fill(theme::ACCENT_RED);
                if ui.add(confirm).clicked() {
                    result = Some(drop_sql.to_string());
                }
            } else if ui.small_button("\u{00d7}").clicked() {
                *confirm_key = Some(key.to_string());
            }
        });
    });
    result
}

fn section(ui: &mut egui::Ui, title: &str, count: usize) {
    ui.label(
        RichText::new(format!("{title} ({count})"))
            .color(theme::accent_color())
            .strong()
            .size(12.0),
    );
    ui.separator();
}

