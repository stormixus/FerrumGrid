use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::db::schema_diff;
use crate::i18n::t;
use crate::state::migration::MigrationStep;
use crate::state::AppState;
use crate::types::ConnectionId;
use crate::ui::theme;

pub fn render_migration_wizard(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.migration_wizard.show {
        return;
    }

    let mut close = false;

    egui::Window::new(t("migration_title"))
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(600.0)
        .show(ctx, |ui| {
            ui.set_min_width(500.0);
            ui.set_max_width(750.0);

            render_step_indicator(ui, state.migration_wizard.step);
            ui.add_space(theme::SPACE_MD);
            ui.separator();
            ui.add_space(theme::SPACE_SM);

            match state.migration_wizard.step {
                MigrationStep::SelectConnections => {
                    render_step_select(ui, state, bridge);
                }
                MigrationStep::DiffResult => {
                    render_step_diff(ui, state);
                }
                MigrationStep::SqlPreview => {
                    render_step_sql(ui, state, bridge);
                }
                MigrationStep::Applying => {
                    ui.label(
                        RichText::new(t("migration_applying"))
                            .color(theme::ACCENT_YELLOW)
                            .size(13.0),
                    );
                    ui.spinner();
                }
                MigrationStep::Complete => {
                    render_step_complete(ui, state);
                }
            }

            ui.add_space(theme::SPACE_MD);

            if let Some(err) = &state.migration_wizard.apply_error {
                ui.label(
                    RichText::new(err.as_str())
                        .color(theme::ACCENT_RED)
                        .size(11.0),
                );
                ui.add_space(theme::SPACE_SM);
            }

            ui.horizontal(|ui| {
                if state.migration_wizard.step != MigrationStep::SelectConnections
                    && state.migration_wizard.step != MigrationStep::Applying
                {
                    if ui.button(t("migration_back")).clicked() {
                        let prev = match state.migration_wizard.step {
                            MigrationStep::DiffResult => MigrationStep::SelectConnections,
                            MigrationStep::SqlPreview => MigrationStep::DiffResult,
                            MigrationStep::Complete => MigrationStep::SqlPreview,
                            other => other,
                        };
                        state.migration_wizard.apply_error = None;
                        state.migration_wizard.go_to(prev);
                    }
                }
                if ui.button(t("migration_close")).clicked() {
                    close = true;
                }
            });
        });

    if close {
        state.migration_wizard.reset();
    }
}

fn render_step_indicator(ui: &mut egui::Ui, current: MigrationStep) {
    let steps = [
        (MigrationStep::SelectConnections, t("migration_step_select")),
        (MigrationStep::DiffResult, t("migration_step_diff")),
        (MigrationStep::SqlPreview, t("migration_step_sql")),
    ];

    ui.horizontal(|ui| {
        for (i, (step, label)) in steps.iter().enumerate() {
            let is_current = *step == current;
            let is_past = step_index(*step) < step_index(current);

            let color = if is_current {
                theme::ACCENT_TEAL
            } else if is_past {
                theme::ACCENT_GREEN
            } else {
                theme::text_muted()
            };

            ui.label(
                RichText::new(format!("{}. {}", i + 1, label))
                    .color(color)
                    .size(11.0)
                    .strong(),
            );

            if i < steps.len() - 1 {
                ui.label(
                    RichText::new(" → ")
                        .color(theme::text_disabled())
                        .size(11.0),
                );
            }
        }
    });
}

fn step_index(step: MigrationStep) -> usize {
    match step {
        MigrationStep::SelectConnections => 0,
        MigrationStep::DiffResult => 1,
        MigrationStep::SqlPreview => 2,
        MigrationStep::Applying => 3,
        MigrationStep::Complete => 4,
    }
}

fn render_step_select(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let conn_ids: Vec<(ConnectionId, String)> = state
        .connections
        .iter()
        .map(|(id, cs)| {
            let name = if cs.config.display_name.is_empty() {
                format!("{}@{}", cs.config.username, cs.config.host)
            } else {
                cs.config.display_name.clone()
            };
            (*id, name)
        })
        .collect();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(t("migration_source_conn"))
                .color(theme::text_muted())
                .size(11.0),
        );
        egui::ComboBox::from_id_salt("mig_src_conn")
            .selected_text(
                state
                    .migration_wizard
                    .source_conn
                    .and_then(|id| conn_ids.iter().find(|(cid, _)| *cid == id))
                    .map(|(_, n)| n.as_str())
                    .unwrap_or("—"),
            )
            .show_ui(ui, |ui| {
                for (id, name) in &conn_ids {
                    ui.selectable_value(&mut state.migration_wizard.source_conn, Some(*id), name);
                }
            });
    });

    if let Some(src_id) = state.migration_wizard.source_conn {
        let schemas: Vec<String> = state
            .connections
            .get(&src_id)
            .map(|c| c.schemas.clone())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t("migration_source_schema"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            egui::ComboBox::from_id_salt("mig_src_schema")
                .selected_text(if state.migration_wizard.source_schema.is_empty() {
                    "—"
                } else {
                    &state.migration_wizard.source_schema
                })
                .show_ui(ui, |ui| {
                    for s in &schemas {
                        ui.selectable_value(
                            &mut state.migration_wizard.source_schema,
                            s.clone(),
                            s,
                        );
                    }
                });
        });
    }

    ui.add_space(theme::SPACE_SM);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(t("migration_target_conn"))
                .color(theme::text_muted())
                .size(11.0),
        );
        egui::ComboBox::from_id_salt("mig_tgt_conn")
            .selected_text(
                state
                    .migration_wizard
                    .target_conn
                    .and_then(|id| conn_ids.iter().find(|(cid, _)| *cid == id))
                    .map(|(_, n)| n.as_str())
                    .unwrap_or("—"),
            )
            .show_ui(ui, |ui| {
                for (id, name) in &conn_ids {
                    ui.selectable_value(&mut state.migration_wizard.target_conn, Some(*id), name);
                }
            });
    });

    if let Some(tgt_id) = state.migration_wizard.target_conn {
        let schemas: Vec<String> = state
            .connections
            .get(&tgt_id)
            .map(|c| c.schemas.clone())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(t("migration_target_schema"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            egui::ComboBox::from_id_salt("mig_tgt_schema")
                .selected_text(if state.migration_wizard.target_schema.is_empty() {
                    "—"
                } else {
                    &state.migration_wizard.target_schema
                })
                .show_ui(ui, |ui| {
                    for s in &schemas {
                        ui.selectable_value(
                            &mut state.migration_wizard.target_schema,
                            s.clone(),
                            s,
                        );
                    }
                });
        });
    }

    ui.add_space(theme::SPACE_MD);

    let can_compare = state.migration_wizard.can_compare();

    ui.horizontal(|ui| {
        if state.migration_wizard.loading_diff {
            ui.spinner();
            ui.label(
                RichText::new(t("migration_comparing"))
                    .color(theme::ACCENT_YELLOW)
                    .size(11.0),
            );
        } else if ui
            .add_enabled(can_compare, egui::Button::new(t("migration_compare")))
            .clicked()
        {
            let src_id = state.migration_wizard.source_conn.unwrap();
            let tgt_id = state.migration_wizard.target_conn.unwrap();
            let src_config = state.connections.get(&src_id).map(|c| c.config.clone());
            let tgt_config = state.connections.get(&tgt_id).map(|c| c.config.clone());

            if let (Some(src_config), Some(tgt_config)) = (src_config, tgt_config) {
                state.migration_wizard.loading_diff = true;
                state.migration_wizard.apply_error = None;
                bridge.send(DbCommand::CompareSchemas {
                    source_config: src_config,
                    target_config: tgt_config,
                    source_schema: state.migration_wizard.source_schema.clone(),
                    target_schema: state.migration_wizard.target_schema.clone(),
                });
            }
        }
    });
}

fn render_step_diff(ui: &mut egui::Ui, state: &mut AppState) {
    let Some(diff) = &state.migration_wizard.diff else {
        ui.label(t("migration_no_diff"));
        return;
    };

    if diff.is_empty() {
        ui.label(
            RichText::new(t("migration_no_changes"))
                .color(theme::ACCENT_GREEN)
                .size(13.0),
        );
        return;
    }

    let (added, modified, removed) = diff.summary_counts();

    ui.horizontal(|ui| {
        if added > 0 {
            ui.label(
                RichText::new(format!("+ {added} {}", t("migration_tables_added")))
                    .color(theme::ACCENT_GREEN)
                    .size(12.0),
            );
        }
        if modified > 0 {
            ui.label(
                RichText::new(format!("~ {modified} {}", t("migration_tables_modified")))
                    .color(theme::ACCENT_YELLOW)
                    .size(12.0),
            );
        }
        if removed > 0 {
            ui.label(
                RichText::new(format!("- {removed} {}", t("migration_tables_removed")))
                    .color(theme::ACCENT_RED)
                    .size(12.0),
            );
        }
    });

    ui.add_space(theme::SPACE_SM);

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for table in &diff.tables_added {
                ui.label(
                    RichText::new(format!("+ {} (new table)", table.name))
                        .color(theme::ACCENT_GREEN)
                        .size(12.0),
                );
            }

            for table_diff in &diff.tables_modified {
                ui.label(
                    RichText::new(format!(
                        "~ {} ({} changes)",
                        table_diff.name,
                        table_diff.change_count()
                    ))
                    .color(theme::ACCENT_YELLOW)
                    .size(12.0),
                );

                for col in &table_diff.columns_added {
                    ui.label(
                        RichText::new(format!("    + {} {}", col.name, col.data_type))
                            .color(theme::ACCENT_GREEN)
                            .size(11.0),
                    );
                }
                for col in &table_diff.columns_removed {
                    ui.label(
                        RichText::new(format!("    - {col}"))
                            .color(theme::ACCENT_RED)
                            .size(11.0),
                    );
                }
                for col in &table_diff.columns_modified {
                    let mut desc = format!("    ~ {} {} → {}", col.name, col.old_type, col.new_type);
                    if let Some(nullable) = col.nullable_changed {
                        desc.push_str(if nullable {
                            " (now nullable)"
                        } else {
                            " (now NOT NULL)"
                        });
                    }
                    ui.label(
                        RichText::new(desc)
                            .color(theme::ACCENT_YELLOW)
                            .size(11.0),
                    );
                }
                for idx in &table_diff.indexes_added {
                    ui.label(
                        RichText::new(format!("    + index: {idx}"))
                            .color(theme::ACCENT_GREEN)
                            .size(11.0),
                    );
                }
                for idx in &table_diff.indexes_removed {
                    ui.label(
                        RichText::new(format!("    - index: {idx}"))
                            .color(theme::ACCENT_RED)
                            .size(11.0),
                    );
                }
            }

            for name in &diff.tables_removed {
                ui.label(
                    RichText::new(format!("- {name} (removed)"))
                        .color(theme::ACCENT_RED)
                        .size(12.0),
                );
            }
        });

    ui.add_space(theme::SPACE_MD);

    if ui.button(t("migration_preview_sql")).clicked() {
        let sql = schema_diff::generate_migration_sql(
            diff,
            &state.migration_wizard.target_schema,
        );
        state.migration_wizard.generated_sql = Some(sql);
        state
            .migration_wizard
            .go_to(MigrationStep::SqlPreview);
    }
}

fn render_step_sql(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let Some(sql) = state.migration_wizard.generated_sql.clone() else {
        ui.label("No SQL generated");
        return;
    };

    egui::ScrollArea::vertical()
        .max_height(350.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut sql.as_str())
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(15),
            );
        });

    ui.add_space(theme::SPACE_MD);

    ui.horizontal(|ui| {
        if ui.button(t("migration_copy_sql")).clicked() {
            ui.ctx().copy_text(sql.clone());
            state.status_message = "SQL copied to clipboard".to_string();
        }

        let can_apply = !state.migration_wizard.applying;
        if ui
            .add_enabled(can_apply, egui::Button::new(t("migration_apply")))
            .clicked()
        {
            if let Some(tgt_id) = state.migration_wizard.target_conn {
                if let Some(tgt_config) =
                    state.connections.get(&tgt_id).map(|c| c.config.clone())
                {
                    state.migration_wizard.applying = true;
                    state.migration_wizard.apply_error = None;
                    state
                        .migration_wizard
                        .go_to(MigrationStep::Applying);
                    bridge.send(DbCommand::ApplyMigration {
                        target_config: tgt_config,
                        sql,
                    });
                }
            }
        }
    });
}

fn render_step_complete(ui: &mut egui::Ui, state: &AppState) {
    if state.migration_wizard.apply_success {
        ui.label(
            RichText::new(t("migration_success"))
                .color(theme::ACCENT_GREEN)
                .size(14.0)
                .strong(),
        );
    }
}
