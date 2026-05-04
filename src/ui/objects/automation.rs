//! Automation presets view.
//!
//! Plan v7 Phase 1.95b2 cut-over (from `super::mod.rs`). Phase 4b 에서 즉시
//! 실행 + 예약(tokio::spawn + interval) 기능 추가 예정.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::state::AppState;
use crate::ui::theme;

use super::{code_line, quote_ident, selected_schema_or_public, ObjectAction};

pub(super) fn render_automation_tools(
    ui: &mut egui::Ui,
    state: &AppState,
) -> Option<ObjectAction> {
    let schema = selected_schema_or_public(state);
    ui.add_space(theme::SPACE_XL);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Automation Presets")
                .color(theme::text_primary())
                .size(14.0)
                .strong(),
        );
        ui.label(
            RichText::new("Create maintenance query tabs from the current schema.")
                .color(theme::text_muted())
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_LG);

    let presets = [
        (
            "Vacuum Analyze",
            format!("VACUUM (VERBOSE, ANALYZE) {};", quote_ident(&schema)),
        ),
        (
            "Reindex Schema",
            format!("REINDEX SCHEMA {};", quote_ident(&schema)),
        ),
        (
            "Refresh Mat Views",
            format!(
                "DO $$\nDECLARE r record;\nBEGIN\n  FOR r IN SELECT schemaname, matviewname FROM pg_matviews WHERE schemaname = '{}'\n  LOOP\n    EXECUTE format('REFRESH MATERIALIZED VIEW %I.%I', r.schemaname, r.matviewname);\n  END LOOP;\nEND $$;",
                schema.replace('\'', "''")
            ),
        ),
    ];

    let mut action = None;
    for (title, sql) in presets {
        egui::Frame::new()
            .fill(theme::bg_medium())
            .stroke(Stroke::new(1.0, theme::border_subtle()))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_LG as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(title)
                            .color(theme::text_primary())
                            .size(12.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(theme::secondary_button("Create Query")).clicked() {
                            action = Some(ObjectAction::AddAutomationQuery {
                                title: title.to_string(),
                                sql: sql.clone(),
                            });
                        }
                    });
                });
                ui.add_space(theme::SPACE_SM);
                code_line(ui, &sql);
            });
        ui.add_space(theme::SPACE_MD);
    }

    action
}
