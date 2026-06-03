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
    egui::Window::new(t("catalog_window_title"))
        .open(&mut open)
        .default_size([640.0, 460.0])
        .resizable(true)
        .show(ctx, |ui| {
            if ui.add(theme::secondary_button(&t("sessions_refresh"))).clicked() {
                refresh = true;
            }
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
                        row2(ui, &e.name, &format!("v{}", e.version));
                    }

                    ui.add_space(theme::SPACE_MD);
                    section(ui, &t("catalog_enums"), catalog.enums.len());
                    for en in &catalog.enums {
                        row2(ui, &format!("{}.{}", en.schema, en.name), &en.labels);
                    }

                    ui.add_space(theme::SPACE_MD);
                    section(ui, &t("catalog_sequences"), catalog.sequences.len());
                    for s in &catalog.sequences {
                        row2(
                            ui,
                            &format!("{}.{}", s.schema, s.name),
                            &format!("{} start {} inc {}", s.data_type, s.start_value, s.increment),
                        );
                    }
                });
        });

    if refresh {
        state.catalog_needs_fetch = true;
    }
    if !open {
        state.show_catalog_window = false;
    }
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

fn row2(ui: &mut egui::Ui, name: &str, detail: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(name)
                .color(theme::text_primary())
                .monospace()
                .size(11.5),
        );
        ui.label(
            RichText::new(detail)
                .color(theme::text_muted())
                .size(11.0),
        );
    });
}
