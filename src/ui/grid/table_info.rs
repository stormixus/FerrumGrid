//! Table info — overview / columns / indexes / relations / rules+triggers.
//!
//! Plan v7 US-G3 — extracted from `info_panel.rs`.

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{AppState, DataSource};
use crate::types::{ColumnInfo, IndexInfo, RuleInfo, TriggerInfo};
use crate::ui::theme;

use super::info_panel::{info_section_label, tiny_badge, value_box, TableInfoContext};
use super::info_row::render_info_empty;
use super::metric_chip;

pub(super) fn ensure_table_info_metadata(state: &mut AppState, bridge: &DbBridge, source: &DataSource) {
    let key = (source.schema.clone(), source.table.clone());
    let mut request_tables = false;
    let mut request_columns = false;
    let mut request_indexes = false;
    let mut request_foreign_keys = false;
    let mut request_rules = false;
    let mut request_triggers = false;

    if let Some(conn) = state.connections.get(&source.conn_id) {
        request_tables = !conn.tables.contains_key(&source.schema)
            && !conn.loading_tables.contains(&source.schema);
        request_columns = !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key);
        request_indexes = !conn.indexes.contains_key(&key) && !conn.loading_indexes.contains(&key);
        request_foreign_keys = !conn.foreign_keys.contains_key(&source.schema)
            && !conn.loading_foreign_keys.contains(&source.schema);
        request_rules = !conn.rules.contains_key(&key) && !conn.loading_rules.contains(&key);
        request_triggers =
            !conn.triggers.contains_key(&key) && !conn.loading_triggers.contains(&key);
    }

    if let Some(conn) = state.connections.get_mut(&source.conn_id) {
        if request_tables {
            conn.loading_tables.insert(source.schema.clone());
        }
        if request_columns {
            conn.loading_columns.insert(key.clone());
        }
        if request_indexes {
            conn.loading_indexes.insert(key.clone());
        }
        if request_foreign_keys {
            conn.loading_foreign_keys.insert(source.schema.clone());
        }
        if request_rules {
            conn.loading_rules.insert(key.clone());
        }
        if request_triggers {
            conn.loading_triggers.insert(key);
        }
    }

    if request_tables {
        bridge.send(DbCommand::ListTables {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
        });
    }
    if request_columns {
        bridge.send(DbCommand::ListColumns {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
            table: source.table.clone(),
        });
    }
    if request_indexes {
        bridge.send(DbCommand::ListIndexes {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
            table: source.table.clone(),
        });
    }
    if request_foreign_keys {
        bridge.send(DbCommand::ListForeignKeys {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
        });
    }
    if request_rules {
        bridge.send(DbCommand::ListRules {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
            table: source.table.clone(),
        });
    }
    if request_triggers {
        bridge.send(DbCommand::ListTriggers {
            conn_id: source.conn_id,
            schema: source.schema.clone(),
            table: source.table.clone(),
        });
    }
}

pub(super) fn table_info_context(state: &AppState, source: &DataSource) -> Option<TableInfoContext> {
    let conn = state.connections.get(&source.conn_id)?;
    let key = (source.schema.clone(), source.table.clone());
    let table_meta = conn
        .tables
        .get(&source.schema)
        .and_then(|tables| tables.iter().find(|table| table.name == source.table));
    let table_type = table_meta
        .map(|table| table.table_type.clone())
        .unwrap_or_else(|| "TABLE".to_string());
    let table_comment = table_meta.and_then(|table| table.comment.clone());
    let columns = conn
        .columns
        .get(&key)
        .cloned()
        .unwrap_or_else(|| result_columns_as_table_columns(state));
    let indexes = conn.indexes.get(&key).cloned().unwrap_or_default();
    let relations = conn
        .foreign_keys
        .get(&source.schema)
        .into_iter()
        .flat_map(|fks| fks.iter())
        .filter(|fk| {
            (fk.source_schema == source.schema && fk.source_table == source.table)
                || (fk.target_schema == source.schema && fk.target_table == source.table)
        })
        .cloned()
        .collect();
    let rules = conn.rules.get(&key).cloned().unwrap_or_default();
    let triggers = conn.triggers.get(&key).cloned().unwrap_or_default();
    let loading_columns = conn.loading_columns.contains(&key);

    Some(TableInfoContext {
        source_label: format!("{}.{}", source.schema, source.table),
        table_name: source.table.clone(),
        schema: source.schema.clone(),
        table_type,
        table_comment,
        filter: source.filter.clone(),
        columns,
        indexes,
        relations,
        rules,
        triggers,
        loading_columns,
    })
}

pub(super) fn result_columns_as_table_columns(state: &AppState) -> Vec<ColumnInfo> {
    state
        .current_result
        .as_ref()
        .map(|result| {
            result
                .columns
                .iter()
                .map(|column| ColumnInfo {
                    name: column.name.clone(),
                    data_type: column.type_name.clone(),
                    enum_values: Vec::new(),
                    is_nullable: true,
                    default_value: None,
                    is_primary_key: false,
                    comment: None,
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn render_info_table_overview(ui: &mut egui::Ui, state: &AppState, source: &DataSource) {
    let Some(context) = table_info_context(state, source) else {
        render_info_empty(ui, &t("data_info_select_cell"));
        return;
    };

    info_section_label(ui, &t("data_info_table"));
    ui.add_space(theme::SPACE_XS);
    ui.label(
        RichText::new(&context.table_name)
            .color(theme::text_primary())
            .strong()
            .size(16.0),
    );
    ui.label(
        RichText::new(&context.source_label)
            .color(theme::text_muted())
            .monospace()
            .size(11.0),
    );
    if let Some(comment) = &context.table_comment {
        ui.add_space(theme::SPACE_XS);
        ui.label(
            RichText::new(comment)
                .color(theme::text_secondary())
                .italics()
                .size(11.5),
        );
    }
    ui.add_space(theme::SPACE_SM);

    ui.horizontal_wrapped(|ui| {
        metric_chip(ui, &context.table_type.to_lowercase(), theme::accent_color());
        metric_chip(
            ui,
            &tf("data_info_columns_n", &[&context.columns.len().to_string()]),
            theme::ACCENT_BLUE,
        );
        metric_chip(
            ui,
            &tf("data_info_indexes_n", &[&context.indexes.len().to_string()]),
            theme::accent_color(),
        );
        metric_chip(
            ui,
            &tf(
                "data_info_relations_n",
                &[&context.relations.len().to_string()],
            ),
            theme::accent_color(),
        );
    });

    if let Some(filter) = &context.filter {
        ui.add_space(theme::SPACE_SM);
        info_section_label(ui, &t("data_info_active_filter"));
        ui.add_space(theme::SPACE_XS);
        value_box(
            ui,
            &format!("{} = {}", filter.column, filter.display_value),
            theme::text_primary(),
        );
    }

    ui.add_space(theme::SPACE_LG);
    render_info_table_columns(ui, &context);
    render_info_table_indexes(ui, &context.indexes);
    render_info_table_relations(ui, &context);
    render_info_table_rules_and_triggers(ui, &context.rules, &context.triggers);
}

pub(super) fn render_info_table_columns(ui: &mut egui::Ui, context: &TableInfoContext) {
    info_section_label(ui, &t("data_info_columns"));
    ui.add_space(theme::SPACE_SM);

    if context.columns.is_empty() {
        let message = if context.loading_columns {
            t("visualizer_loading_columns")
        } else {
            t("data_info_no_metadata")
        };
        render_info_inline_empty(ui, &message);
        return;
    }

    for column in &context.columns {
        let relation = context.relations.iter().find(|fk| {
            fk.source_schema == context.schema
                && fk.source_table == context.table_name
                && fk.source_column == column.name
        });
        egui::Frame::new()
            .fill(theme::bg_darkest())
            .inner_margin(Margin::same(theme::SPACE_MD_I))
            .stroke(Stroke::new(1.0, theme::border_subtle()))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(&column.name)
                            .color(theme::text_primary())
                            .strong()
                            .size(12.5),
                    );
                    ui.label(
                        RichText::new(&column.data_type)
                            .color(theme::text_muted())
                            .monospace()
                            .size(10.5),
                    );
                    if column.is_primary_key {
                        tiny_badge(ui, "PK", theme::ACCENT_YELLOW);
                    }
                    if relation.is_some() {
                        tiny_badge(ui, "FK", theme::ACCENT_BLUE);
                    }
                    if column.is_nullable {
                        tiny_badge(ui, "NULL", theme::text_muted());
                    }
                });
                if let Some(default_value) = &column.default_value {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(
                        RichText::new(format!("{}: {}", t("column_default"), default_value))
                            .color(theme::text_muted())
                            .monospace()
                            .size(10.0),
                    );
                }
                if let Some(fk) = relation {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(
                        RichText::new(format!(
                            "{}: {}.{}({})",
                            t("column_foreign_key"),
                            fk.target_schema,
                            fk.target_table,
                            fk.target_column
                        ))
                        .color(theme::accent_color())
                        .monospace()
                        .size(10.0),
                    );
                }
                if let Some(comment) = &column.comment {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(
                        RichText::new(comment)
                            .color(theme::text_secondary())
                            .italics()
                            .size(10.0),
                    );
                }
            });
        ui.add_space(theme::SPACE_SM);
    }
}

pub(super) fn render_info_table_indexes(ui: &mut egui::Ui, indexes: &[IndexInfo]) {
    if indexes.is_empty() {
        return;
    }

    ui.add_space(theme::SPACE_MD);
    info_section_label(ui, &t("objects_indexes"));
    ui.add_space(theme::SPACE_SM);
    for index in indexes {
        compact_metadata_row(
            ui,
            &index.name,
            &format!("{} · {}", index.index_type, index.columns.join(", ")),
            if index.is_primary {
                Some(("PK".to_string(), theme::ACCENT_YELLOW))
            } else if index.is_unique {
                Some(("UNIQUE".to_string(), theme::accent_color()))
            } else {
                None
            },
        );
    }
}

pub(super) fn render_info_table_relations(ui: &mut egui::Ui, context: &TableInfoContext) {
    if context.relations.is_empty() {
        return;
    }

    ui.add_space(theme::SPACE_MD);
    info_section_label(ui, &t("tree_foreign_keys"));
    ui.add_space(theme::SPACE_SM);
    for fk in &context.relations {
        let outgoing = fk.source_schema == context.schema && fk.source_table == context.table_name;
        let title = if outgoing {
            format!(
                "{} -> {}.{}",
                fk.source_column, fk.target_schema, fk.target_table
            )
        } else {
            format!(
                "{}.{} -> {}",
                fk.source_schema, fk.source_table, fk.target_column
            )
        };
        let subtitle = if outgoing {
            format!("{}({})", fk.target_table, fk.target_column)
        } else {
            format!("{}({})", fk.source_table, fk.source_column)
        };
        compact_metadata_row(
            ui,
            &title,
            &subtitle,
            Some((
                if outgoing {
                    t("data_info_relation_out")
                } else {
                    t("data_info_relation_in")
                },
                theme::accent_color(),
            )),
        );
    }
}

pub(super) fn render_info_table_rules_and_triggers(
    ui: &mut egui::Ui,
    rules: &[RuleInfo],
    triggers: &[TriggerInfo],
) {
    if rules.is_empty() && triggers.is_empty() {
        return;
    }

    ui.add_space(theme::SPACE_MD);
    ui.horizontal_wrapped(|ui| {
        if !rules.is_empty() {
            metric_chip(
                ui,
                &tf("data_info_rules_n", &[&rules.len().to_string()]),
                theme::accent_color_light(),
            );
        }
        if !triggers.is_empty() {
            metric_chip(
                ui,
                &tf("data_info_triggers_n", &[&triggers.len().to_string()]),
                theme::ACCENT_RED,
            );
        }
    });
}

pub(super) fn render_info_inline_empty(ui: &mut egui::Ui, message: &str) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(message).color(theme::text_muted()).size(11.5));
        });
}

pub(super) fn compact_metadata_row(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    badge: Option<(String, Color32)>,
) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(title)
                        .color(theme::text_primary())
                        .strong()
                        .size(12.0),
                );
                if let Some((text, color)) = badge.as_ref() {
                    tiny_badge(ui, text, *color);
                }
            });
            ui.add_space(theme::SPACE_XS);
            ui.label(
                RichText::new(subtitle)
                    .color(theme::text_muted())
                    .monospace()
                    .size(10.5),
            );
        });
    ui.add_space(theme::SPACE_SM);
}

