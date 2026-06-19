//! Schema diff window backed by the existing migration diff engine.

use eframe::egui::{self, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::db::schema_diff::SchemaDiff;
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_schema_diff_window(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_schema_diff_window {
        return;
    }

    let mut open = true;
    let mut compare = false;
    egui::Window::new(t("schema_diff_title"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(720.0)
        .default_height(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_LG as i8)),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("schema_diff_hint"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_SM);

            render_selector_rows(ui, state, bridge);

            ui.add_space(theme::SPACE_MD);
            ui.horizontal(|ui| {
                if state.migration_wizard.loading_diff {
                    ui.spinner();
                    ui.label(
                        RichText::new(t("migration_comparing"))
                            .color(theme::ACCENT_YELLOW)
                            .size(11.0),
                    );
                } else if ui
                    .add_enabled(
                        state.migration_wizard.can_compare(),
                        theme::secondary_button(&t("migration_compare")),
                    )
                    .clicked()
                {
                    compare = true;
                }
            });

            ui.separator();
            render_diff_rows(ui, state);
        });

    if compare {
        start_compare(state, bridge);
    }
    if !open {
        state.show_schema_diff_window = false;
    }
}

fn render_selector_rows(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let connections: Vec<_> = state
        .connections
        .iter()
        .map(|(id, conn)| (*id, conn.config.display_name.clone()))
        .collect();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Source")
                .color(theme::text_secondary())
                .size(11.0),
        );
        let old = state.migration_wizard.source_conn;
        connection_combo(
            ui,
            "schema_diff_source_conn",
            &connections,
            &mut state.migration_wizard.source_conn,
        );
        if old != state.migration_wizard.source_conn {
            state.migration_wizard.source_schema.clear();
            request_schemas_for_selection(state.migration_wizard.source_conn, bridge);
        }

        schema_combo(
            ui,
            "schema_diff_source_schema",
            state
                .migration_wizard
                .source_conn
                .and_then(|id| state.connections.get(&id)),
            &mut state.migration_wizard.source_schema,
        );
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Target")
                .color(theme::text_secondary())
                .size(11.0),
        );
        let old = state.migration_wizard.target_conn;
        connection_combo(
            ui,
            "schema_diff_target_conn",
            &connections,
            &mut state.migration_wizard.target_conn,
        );
        if old != state.migration_wizard.target_conn {
            state.migration_wizard.target_schema.clear();
            request_schemas_for_selection(state.migration_wizard.target_conn, bridge);
        }

        schema_combo(
            ui,
            "schema_diff_target_schema",
            state
                .migration_wizard
                .target_conn
                .and_then(|id| state.connections.get(&id)),
            &mut state.migration_wizard.target_schema,
        );
    });
}

fn connection_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    connections: &[(crate::types::ConnectionId, String)],
    selected: &mut Option<crate::types::ConnectionId>,
) {
    let selected_text = selected
        .and_then(|selected_id| {
            connections
                .iter()
                .find(|(id, _)| *id == selected_id)
                .map(|(_, label)| label.as_str())
        })
        .unwrap_or("-");

    egui::ComboBox::from_id_salt(id)
        .width(180.0)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for (conn_id, label) in connections {
                ui.selectable_value(selected, Some(*conn_id), label);
            }
        });
}

fn schema_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    conn: Option<&crate::state::ConnectionState>,
    selected: &mut String,
) {
    let schemas = conn.map(|conn| conn.schemas.as_slice()).unwrap_or(&[]);
    egui::ComboBox::from_id_salt(id)
        .width(140.0)
        .selected_text(if selected.is_empty() {
            "-"
        } else {
            selected.as_str()
        })
        .show_ui(ui, |ui| {
            for schema in schemas {
                ui.selectable_value(selected, schema.clone(), schema);
            }
        });
}

fn request_schemas_for_selection(conn_id: Option<crate::types::ConnectionId>, bridge: &DbBridge) {
    if let Some(conn_id) = conn_id {
        bridge.send(DbCommand::ListSchemas { conn_id });
    }
}

fn start_compare(state: &mut AppState, bridge: &DbBridge) {
    let Some(source_conn) = state.migration_wizard.source_conn else {
        return;
    };
    let Some(target_conn) = state.migration_wizard.target_conn else {
        return;
    };
    let Some(source_config) = state
        .connections
        .get(&source_conn)
        .map(|conn| conn.config.clone())
    else {
        return;
    };
    let Some(target_config) = state
        .connections
        .get(&target_conn)
        .map(|conn| conn.config.clone())
    else {
        return;
    };

    state.schema_diff_rows.clear();
    state.migration_wizard.diff = None;
    state.migration_wizard.apply_error = None;
    state.migration_wizard.loading_diff = true;
    bridge.send(DbCommand::CompareSchemas {
        source_config,
        target_config,
        source_schema: state.migration_wizard.source_schema.clone(),
        target_schema: state.migration_wizard.target_schema.clone(),
    });
}

fn render_diff_rows(ui: &mut egui::Ui, state: &AppState) {
    if let Some(error) = state.migration_wizard.apply_error.as_ref() {
        ui.label(RichText::new(error).color(theme::ACCENT_RED).size(11.0));
        return;
    }

    if state.schema_diff_rows.is_empty() {
        ui.label(
            RichText::new(t("schema_diff_empty"))
                .color(theme::text_muted())
                .size(11.0),
        );
        return;
    }

    egui::ScrollArea::vertical()
        .max_height(330.0)
        .show(ui, |ui| {
            for row in &state.schema_diff_rows {
                ui.label(
                    RichText::new(row)
                        .color(theme::text_secondary())
                        .size(11.0)
                        .monospace(),
                );
            }
        });
}

pub fn diff_to_rows(diff: &SchemaDiff) -> Vec<String> {
    let mut rows = Vec::new();
    let (added, modified, removed) = diff.summary_counts();
    rows.push(format!(
        "Summary: {added} added, {modified} modified, {removed} removed"
    ));

    for table in &diff.tables_added {
        rows.push(format!("+ table {}", table.name));
    }
    for table in &diff.tables_modified {
        rows.push(format!(
            "~ table {} ({} changes)",
            table.name,
            table.change_count()
        ));
        for column in &table.columns_added {
            rows.push(format!("  + column {} {}", column.name, column.data_type));
        }
        for column in &table.columns_removed {
            rows.push(format!("  - column {column}"));
        }
        for column in &table.columns_modified {
            rows.push(format!(
                "  ~ column {}: {} -> {}",
                column.name, column.old_type, column.new_type
            ));
        }
        for index in &table.indexes_added {
            rows.push(format!("  + index {index}"));
        }
        for index in &table.indexes_removed {
            rows.push(format!("  - index {index}"));
        }
    }
    for table in &diff.tables_removed {
        rows.push(format!("- table {table}"));
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::introspect::TableDef;
    use crate::db::schema_diff::{ColumnAdd, SchemaDiff, TableDiff};

    #[test]
    fn diff_rows_include_summary_and_table_changes() {
        let diff = SchemaDiff {
            tables_added: vec![TableDef {
                schema: "public".to_string(),
                name: "users".to_string(),
                columns: Vec::new(),
                primary_key: None,
                indexes: Vec::new(),
                check_constraints: Vec::new(),
            }],
            tables_removed: vec!["legacy".to_string()],
            tables_modified: vec![TableDiff {
                name: "orders".to_string(),
                columns_added: vec![ColumnAdd {
                    name: "status".to_string(),
                    data_type: "text".to_string(),
                    is_nullable: true,
                    default_value: None,
                }],
                columns_removed: Vec::new(),
                columns_modified: Vec::new(),
                indexes_added: Vec::new(),
                indexes_removed: Vec::new(),
            }],
        };

        let rows = diff_to_rows(&diff);

        assert!(rows
            .iter()
            .any(|row| row == "Summary: 1 added, 1 modified, 1 removed"));
        assert!(rows.iter().any(|row| row == "+ table users"));
        assert!(rows.iter().any(|row| row == "~ table orders (1 changes)"));
        assert!(rows.iter().any(|row| row == "- table legacy"));
    }
}
