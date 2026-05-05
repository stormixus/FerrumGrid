//! Automation presets + scheduled tasks view.
//!
//! Plan v7 Phase 1.95b2 cut-over (from `super::mod.rs`). Phase 4b3 에서
//! `AutomationStore` 가 backing — 본 view 가 list/create/run-now/cancel UI 제공.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::automation::scheduler::{ApplyResult, Schedule};
use crate::state::AppState;
use crate::ui::theme;

use super::{code_line, quote_ident, selected_schema_or_public, ObjectAction};

pub(super) fn render_automation_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
) -> Option<ObjectAction> {
    ui.add_space(theme::SPACE_XL);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Automation")
                .color(theme::text_primary())
                .size(14.0)
                .strong(),
        );
        ui.label(
            RichText::new("Schedule maintenance queries or run them ad-hoc.")
                .color(theme::text_muted())
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_LG);

    let mut action: Option<ObjectAction> = None;

    // Section 1: Scheduled Tasks list
    if let Some(a) = render_scheduled_tasks(ui, state) {
        action = Some(a);
    }

    ui.add_space(theme::SPACE_LG);

    // Section 2: Create form
    if let Some(a) = render_create_form(ui, state) {
        action = Some(a);
    }

    ui.add_space(theme::SPACE_XL);
    ui.separator();
    ui.add_space(theme::SPACE_MD);

    // Section 3: Existing presets (unchanged behavior — open as Query tab)
    if let Some(a) = render_presets(ui, state) {
        action = Some(a);
    }

    action
}

fn render_scheduled_tasks(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    ui.label(
        RichText::new("Scheduled Tasks")
            .color(theme::text_primary())
            .size(12.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_SM);

    let store = state.automation.read().expect("automation lock poisoned");
    if store.is_empty() {
        ui.label(
            RichText::new("No scheduled tasks yet. Use the Create form below.")
                .color(theme::text_disabled())
                .size(11.0),
        );
        return None;
    }

    let mut action = None;
    for task in store.iter() {
        egui::Frame::new()
            .fill(theme::bg_medium())
            .stroke(Stroke::new(1.0, theme::border_subtle()))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_LG as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&task.title)
                            .color(theme::text_primary())
                            .size(12.0)
                            .strong(),
                    );
                    let schedule_label = match &task.schedule {
                        Schedule::Once { at } => {
                            format!("Once at {}", at.format("%Y-%m-%d %H:%M:%S UTC"))
                        }
                        Schedule::Interval { period } => {
                            format!("Every {}s", period.as_secs())
                        }
                    };
                    ui.label(
                        RichText::new(schedule_label)
                            .color(theme::text_muted())
                            .size(11.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(theme::secondary_button("Cancel")).clicked() {
                            action = Some(ObjectAction::AutomationCancel { id: task.id });
                        }
                        if ui.add(theme::secondary_button("Run Now")).clicked() {
                            action = Some(ObjectAction::AutomationRunNow {
                                id: task.id,
                                sql: task.sql.clone(),
                            });
                        }
                    });
                });

                ui.add_space(theme::SPACE_SM);

                // last_run + last_result + next_run summary
                let last_label = match &task.last_run {
                    Some(t) => format!("Last run: {}", t.format("%H:%M:%S UTC")),
                    None => "Last run: —".to_string(),
                };
                let next_label = match &task.next_run {
                    Some(t) => format!("Next: {}", t.format("%Y-%m-%d %H:%M:%S UTC")),
                    None => "Next: (done)".to_string(),
                };
                let result_label = match &task.last_result {
                    Some(ApplyResult::Success { rows_affected }) => {
                        format!("Last result: {rows_affected} rows")
                    }
                    Some(ApplyResult::Failed { error }) => format!("Last result: error {error}"),
                    None => "Last result: —".to_string(),
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(last_label)
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                    ui.label(RichText::new("|").color(theme::text_disabled()).size(11.0));
                    ui.label(
                        RichText::new(next_label)
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                    ui.label(RichText::new("|").color(theme::text_disabled()).size(11.0));
                    ui.label(
                        RichText::new(result_label)
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                });

                ui.add_space(theme::SPACE_XS);
                code_line(ui, &task.sql);
            });
        ui.add_space(theme::SPACE_MD);
    }

    action
}

fn render_create_form(ui: &mut egui::Ui, state: &mut AppState) -> Option<ObjectAction> {
    ui.label(
        RichText::new("Create Scheduled Task")
            .color(theme::text_primary())
            .size(12.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_SM);

    let mut action = None;

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_LG as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Title")
                        .color(theme::text_secondary())
                        .size(11.0),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut state.automation_draft.title)
                        .hint_text("e.g., Daily Vacuum")
                        .desired_width(240.0),
                );
            });
            ui.add_space(theme::SPACE_SM);

            ui.label(
                RichText::new("SQL")
                    .color(theme::text_secondary())
                    .size(11.0),
            );
            ui.add(
                egui::TextEdit::multiline(&mut state.automation_draft.sql)
                    .hint_text("VACUUM ANALYZE my_table;")
                    .desired_rows(3)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
            ui.add_space(theme::SPACE_SM);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Interval (seconds)")
                        .color(theme::text_secondary())
                        .size(11.0),
                );
                ui.add(
                    egui::DragValue::new(&mut state.automation_draft.interval_secs)
                        .range(0..=86_400 * 7)
                        .speed(60),
                );
                ui.label(
                    RichText::new("(0 = Once, e.g. 3600 = hourly)")
                        .color(theme::text_disabled())
                        .size(10.0),
                );
            });
            ui.add_space(theme::SPACE_MD);

            let can_create = !state.automation_draft.title.trim().is_empty()
                && !state.automation_draft.sql.trim().is_empty();

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = ui.add_enabled(can_create, theme::primary_button("Create"));
                    if btn.clicked() {
                        action = Some(ObjectAction::AutomationCreate {
                            title: state.automation_draft.title.trim().to_string(),
                            sql: state.automation_draft.sql.trim().to_string(),
                            interval_secs: state.automation_draft.interval_secs,
                        });
                    }
                });
            });
        });

    action
}

fn render_presets(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    let schema = selected_schema_or_public(state);

    ui.label(
        RichText::new("Quick Presets")
            .color(theme::text_primary())
            .size(12.0)
            .strong(),
    );
    ui.label(
        RichText::new("One-shot maintenance queries — opens as new Query tab.")
            .color(theme::text_muted())
            .size(11.0),
    );
    ui.add_space(theme::SPACE_SM);

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
