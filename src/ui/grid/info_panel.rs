//! Info panel (selected cell / row metadata + edit affordances).
//!
//! Plan v7 Phase 1.95c — cut-over from `super::mod.rs`. Hosts
//! `render_info_panel`, `restore_active_data_tab`, and all sub-renderers
//! (table overview, columns, indexes, relations, rules/triggers, JSON tree,
//! enum control, dark-select, toggle, action buttons).

use std::hash::Hash;

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    cell_edit_text_for_type, data_filter_from_text, AppState, DataFilter, DataSource, MainView,
};
use crate::types::{CellValue, ColumnInfo, ColumnMeta, IndexInfo, RuleInfo, TriggerInfo};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

use super::{
    build_data_edits, data_column_info, data_edit_summary, edit_kind, has_table_column_metadata,
    metric_chip, open_related_data, parse_bool, relation_for_column, reload_data_source,
    render_date_editor, request_foreign_keys_for_schema, request_table_columns_for_data,
    revert_data_edits, set_pointing_cursor_on_hover, show_dark_popup_below,
    validate_edit_value, EditKind,
};
use super::show_dark_hover_tooltip;

pub fn render_info_panel(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    render_info_header(ui);

    egui::ScrollArea::vertical()
        .id_salt("data_info_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_MD_I))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if state.active_main_view != MainView::Data {
                        render_info_empty(ui, &t("data_info_no_selection"));
                        return;
                    }

                    let Some(context) = selected_data_row_context(state) else {
                        let Some(source) = state.active_data_source() else {
                            render_info_empty(ui, &t("data_info_select_cell"));
                            return;
                        };
                        ensure_table_info_metadata(state, bridge, &source);
                        render_info_table_overview(ui, state, &source);
                        return;
                    };

                    for (col_idx, cell) in context.fallback_row.iter().enumerate() {
                        let type_name = context
                            .columns
                            .get(col_idx)
                            .map(|column| column.type_name.as_str())
                            .unwrap_or_default();
                        ensure_data_edit_cell(state, context.row_idx, col_idx, cell, type_name);
                    }

                    render_info_row_summary(ui, &context);
                    ui.add_space(theme::SPACE_LG);
                    render_info_row_fields(ui, state, bridge, &context);
                    render_info_apply_controls(ui, state, bridge);
                    ui.add_space(theme::SPACE_LG);
                });
        });
}

pub fn restore_active_data_tab(state: &mut AppState, bridge: &DbBridge) {
    if state.active_main_view != MainView::Data {
        return;
    }
    let Some(source) = state.active_data_source() else {
        return;
    };

    if state.data_edit.source.as_ref() != Some(&source) {
        state.begin_data_edit_with_filter(
            source.conn_id,
            &source.schema,
            &source.table,
            source.filter.clone(),
        );
    }
    request_table_columns_for_data(state, bridge, source.conn_id, &source.schema, &source.table);
    request_foreign_keys_for_schema(state, bridge, source.conn_id, &source.schema);
    reload_data_source(state, bridge);
}

#[derive(Clone)]
struct SelectedRowContext {
    row_idx: usize,
    selected_col_idx: usize,
    source_label: String,
    columns: Vec<ColumnMeta>,
    column_infos: Vec<Option<ColumnInfo>>,
    fallback_row: Vec<CellValue>,
}

#[derive(Clone)]
struct TableInfoContext {
    source_label: String,
    table_name: String,
    schema: String,
    table_type: String,
    filter: Option<DataFilter>,
    columns: Vec<ColumnInfo>,
    indexes: Vec<IndexInfo>,
    relations: Vec<ForeignKey>,
    rules: Vec<RuleInfo>,
    triggers: Vec<TriggerInfo>,
    loading_columns: bool,
}

fn ensure_table_info_metadata(state: &mut AppState, bridge: &DbBridge, source: &DataSource) {
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

fn table_info_context(state: &AppState, source: &DataSource) -> Option<TableInfoContext> {
    let conn = state.connections.get(&source.conn_id)?;
    let key = (source.schema.clone(), source.table.clone());
    let table_type = conn
        .tables
        .get(&source.schema)
        .and_then(|tables| tables.iter().find(|table| table.name == source.table))
        .map(|table| table.table_type.clone())
        .unwrap_or_else(|| "TABLE".to_string());
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
        filter: source.filter.clone(),
        columns,
        indexes,
        relations,
        rules,
        triggers,
        loading_columns,
    })
}

fn result_columns_as_table_columns(state: &AppState) -> Vec<ColumnInfo> {
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
                })
                .collect()
        })
        .unwrap_or_default()
}

fn render_info_table_overview(ui: &mut egui::Ui, state: &AppState, source: &DataSource) {
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
    ui.add_space(theme::SPACE_SM);

    ui.horizontal_wrapped(|ui| {
        metric_chip(ui, &context.table_type.to_lowercase(), theme::ACCENT_COPPER);
        metric_chip(
            ui,
            &tf("data_info_columns_n", &[&context.columns.len().to_string()]),
            theme::ACCENT_BLUE,
        );
        metric_chip(
            ui,
            &tf("data_info_indexes_n", &[&context.indexes.len().to_string()]),
            theme::ACCENT_TEAL,
        );
        metric_chip(
            ui,
            &tf(
                "data_info_relations_n",
                &[&context.relations.len().to_string()],
            ),
            theme::ACCENT_GREEN,
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

fn render_info_table_columns(ui: &mut egui::Ui, context: &TableInfoContext) {
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
                        .color(theme::ACCENT_TEAL)
                        .monospace()
                        .size(10.0),
                    );
                }
            });
        ui.add_space(theme::SPACE_SM);
    }
}

fn render_info_table_indexes(ui: &mut egui::Ui, indexes: &[IndexInfo]) {
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
                Some(("UNIQUE".to_string(), theme::ACCENT_TEAL))
            } else {
                None
            },
        );
    }
}

fn render_info_table_relations(ui: &mut egui::Ui, context: &TableInfoContext) {
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
                theme::ACCENT_GREEN,
            )),
        );
    }
}

fn render_info_table_rules_and_triggers(
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
                theme::ACCENT_COPPER_LIGHT,
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

fn render_info_inline_empty(ui: &mut egui::Ui, message: &str) {
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

fn compact_metadata_row(
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

fn selected_data_row_context(state: &AppState) -> Option<SelectedRowContext> {
    let (row_idx, col_idx) = state.data_edit.selected_cell?;
    let result = state.current_result.as_ref()?;
    result.columns.get(col_idx)?;
    let fallback_row = result.rows.get(row_idx)?.clone();
    let source_label = state
        .active_data_source()
        .map(|source| format!("{}.{}", source.schema, source.table))
        .unwrap_or_else(|| t("objects_data_title"));
    let column_infos = result
        .columns
        .iter()
        .map(|column| data_column_info(state, &column.name).cloned())
        .collect();

    Some(SelectedRowContext {
        row_idx,
        selected_col_idx: col_idx,
        source_label,
        columns: result.columns.clone(),
        column_infos,
        fallback_row,
    })
}

fn ensure_data_edit_cell(
    state: &mut AppState,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    let timezone = state.data_timezone.clone();
    state
        .data_edit
        .cells
        .entry((row_idx, col_idx))
        .or_insert_with(|| {
            crate::state::EditableCell::from_cell_for_type(fallback_cell, type_name, &timezone)
        });
}

fn render_info_header(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::same(theme::SPACE_LG_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::INFO, "info", 14.0);
                ui.add_space(4.0);
                ui.label(
                    RichText::new(t("info"))
                        .color(theme::text_primary())
                        .strong(),
                );
            });
        });
}

fn render_info_empty(ui: &mut egui::Ui, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(92.0);
        crate::ui::icon_img_tinted(
            ui,
            crate::ui::icons_svg::TABLE,
            "data_info_empty",
            28.0,
            theme::text_disabled(),
        );
        ui.add_space(theme::SPACE_SM);
        ui.label(RichText::new(message).color(theme::text_muted()).size(13.0));
    });
}

fn render_info_row_summary(ui: &mut egui::Ui, context: &SelectedRowContext) {
    info_section_label(ui, &t("data_info_row"));
    ui.add_space(theme::SPACE_XS);
    ui.label(
        RichText::new(tf("data_info_row_n", &[&(context.row_idx + 1).to_string()]))
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
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("data_info_row_n", &[&(context.row_idx + 1).to_string()]),
            theme::ACCENT_TEAL,
        );
        metric_chip(
            ui,
            &tf("data_info_columns_n", &[&context.columns.len().to_string()]),
            theme::ACCENT_BLUE,
        );
    });
}

fn render_info_row_fields(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    context: &SelectedRowContext,
) {
    info_section_label(ui, &t("data_info_columns"));
    ui.add_space(theme::SPACE_SM);

    for (col_idx, column) in context.columns.iter().enumerate() {
        let fallback_cell = context
            .fallback_row
            .get(col_idx)
            .cloned()
            .unwrap_or(CellValue::Null);
        let column_info = context.column_infos.get(col_idx).cloned().flatten();
        let field = RowFieldContext {
            row_idx: context.row_idx,
            col_idx,
            selected: context.selected_col_idx == col_idx,
            column: column.clone(),
            column_info,
            fallback_cell,
        };
        render_info_row_field(ui, state, bridge, field);
        ui.add_space(theme::SPACE_SM);
    }
}

struct RowFieldContext {
    row_idx: usize,
    col_idx: usize,
    selected: bool,
    column: ColumnMeta,
    column_info: Option<ColumnInfo>,
    fallback_cell: CellValue,
}

fn render_info_row_field(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    field: RowFieldContext,
) {
    let cell_key = (field.row_idx, field.col_idx);
    let type_name = field
        .column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .unwrap_or_else(|| field.column.type_name.clone());
    let nullable = field
        .column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = field
        .column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();
    let is_primary_key = field
        .column_info
        .as_ref()
        .is_some_and(|info| info.is_primary_key);
    let can_edit = has_table_column_metadata(state) && !is_primary_key;
    let data_timezone = state.data_timezone.clone();

    let stroke_color = if field.selected {
        theme::ACCENT_TEAL
    } else {
        theme::border_subtle()
    };

    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_LG_I))
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&field.column.name)
                        .color(theme::text_primary())
                        .strong()
                        .size(12.5),
                );
                ui.label(
                    RichText::new(&type_name)
                        .color(theme::text_muted())
                        .monospace()
                        .size(10.5),
                );
                if is_primary_key {
                    tiny_badge(ui, "PK", theme::ACCENT_YELLOW);
                }
                if field.selected {
                    tiny_badge(ui, &t("data_info_selected"), theme::ACCENT_TEAL);
                }
            });

            if let Some(default_value) = field
                .column_info
                .as_ref()
                .and_then(|info| info.default_value.as_ref())
            {
                ui.add_space(theme::SPACE_XS);
                ui.label(
                    RichText::new(format!("{}: {}", t("column_default"), default_value))
                        .color(theme::text_muted())
                        .monospace()
                        .size(10.0),
                );
            }

            ui.add_space(theme::SPACE_SM);

            if can_edit {
                if let Some(edit) = state.data_edit.cells.get_mut(&cell_key) {
                    if nullable {
                        info_toggle_control(ui, &mut edit.is_null, &t("grid_toggle_null"), true);
                        ui.add_space(theme::SPACE_XS);
                    }

                    if edit.is_null {
                        value_box(ui, "NULL", theme::text_muted());
                    } else if !enum_values.is_empty() {
                        render_info_enum_editor(
                            ui,
                            edit,
                            field.row_idx,
                            field.col_idx,
                            &enum_values,
                        );
                    } else {
                        render_info_editor_control(
                            ui,
                            edit,
                            &type_name,
                            &field.fallback_cell,
                            &data_timezone,
                        );
                    }
                }
            } else {
                value_box(ui, &field.fallback_cell.to_string(), theme::text_primary());
                ui.add_space(theme::SPACE_XS);
                ui.label(
                    RichText::new(if is_primary_key {
                        t("data_info_read_only_pk")
                    } else if !has_table_column_metadata(state) {
                        t("data_info_no_metadata")
                    } else {
                        t("data_info_read_only")
                    })
                    .color(theme::text_muted())
                    .size(10.5),
                );
            }

            let snapshot = state.data_edit.cells.get(&cell_key).cloned();
            if let Some(snapshot) = snapshot {
                if let Some(error) =
                    validate_edit_value(&snapshot, &type_name, nullable, &enum_values)
                {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(RichText::new(error).color(theme::ACCENT_RED).size(11.0));
                }

                ui.add_space(theme::SPACE_SM);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(theme::SPACE_SM, theme::SPACE_SM);
                    if info_icon_action_button(
                        ui,
                        crate::ui::icons_svg::COPY,
                        "info_copy_field",
                        &t("grid_copy_value"),
                        true,
                        theme::ACCENT_BLUE,
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(editable_cell_display_text(&snapshot));
                    }

                    if can_edit
                        && info_icon_action_button(
                            ui,
                            crate::ui::icons_svg::REFRESH,
                            "info_revert_cell",
                            &t("data_info_revert_cell"),
                            snapshot.is_dirty(),
                            theme::text_muted(),
                        )
                        .clicked()
                    {
                        revert_data_cell(state, cell_key, &type_name);
                    }

                    if let Some((fk, filter)) =
                        relation_filter_for_snapshot(state, &field.column, &type_name, &snapshot)
                    {
                        let relation_resp = info_icon_action_button(
                            ui,
                            crate::ui::icons_svg::CHEVRON_RIGHT,
                            "info_relation_jump",
                            &t("data_relation_open"),
                            true,
                            theme::ACCENT_TEAL,
                        );
                        if relation_resp.clicked() {
                            open_related_data(state, bridge, &fk, filter);
                        }
                    }
                });

                if snapshot.is_dirty() {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(
                        RichText::new(t("data_info_dirty"))
                            .color(theme::ACCENT_COPPER_LIGHT)
                            .size(10.5),
                    );
                }
            }
        });
}

fn relation_filter_for_snapshot(
    state: &AppState,
    column: &ColumnMeta,
    type_name: &str,
    snapshot: &crate::state::EditableCell,
) -> Option<(ForeignKey, DataFilter)> {
    if snapshot.is_null {
        return None;
    }
    relation_for_column(state, Some(&column.name)).and_then(|fk| {
        data_filter_from_text(
            fk.target_column.clone(),
            type_name.to_string(),
            &snapshot.value,
        )
        .map(|filter| (fk, filter))
    })
}

fn render_info_editor_control(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    type_name: &str,
    fallback_cell: &CellValue,
    data_timezone: &str,
) {
    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let mut checked = parse_bool(&edit.value).unwrap_or(false);
            if info_toggle_control(ui, &mut checked, "true", true).changed() {
                edit.value = checked.to_string();
            }
        }
        EditKind::Date => {
            render_date_editor(ui, edit, false, data_timezone, None);
        }
        EditKind::DateTime => {
            render_date_editor(ui, edit, true, data_timezone, None);
        }
        EditKind::Json => {
            render_info_json_editor(ui, edit);
        }
        EditKind::Text => {
            ui.add(
                theme::multiline_text_input(&mut edit.value)
                    .desired_width(ui.available_width())
                    .desired_rows(2),
            );
        }
        EditKind::Number | EditKind::Uuid | EditKind::Bytes => {
            ui.add(theme::mono_text_input(&mut edit.value).desired_width(ui.available_width()));
        }
    }
}

fn render_info_json_editor(ui: &mut egui::Ui, edit: &mut crate::state::EditableCell) {
    let source = edit.value.trim();
    let mut value = if source.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        match serde_json::from_str::<serde_json::Value>(source) {
            Ok(value) => value,
            Err(error) => {
                ui.label(
                    RichText::new(error.to_string())
                        .color(theme::ACCENT_RED)
                        .size(11.0),
                );
                ui.add_space(theme::SPACE_XS);
                ui.add(
                    theme::multiline_mono_text_input(&mut edit.value)
                        .desired_width(ui.available_width())
                        .desired_rows(4)
                        .code_editor(),
                );
                return;
            }
        }
    };

    let mut changed = false;
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            changed |= render_json_value_node(ui, "JSON", &mut value, 0, "$");
        });

    if changed {
        if let Ok(next) = serde_json::to_string(&value) {
            edit.value = next;
        }
    }
}

fn render_json_value_node(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut serde_json::Value,
    depth: usize,
    path: &str,
) -> bool {
    match value {
        serde_json::Value::Object(object) => {
            render_json_branch_header(
                ui,
                label,
                "object",
                &object.len().to_string(),
                depth,
                theme::ACCENT_BLUE,
            );
            if object.is_empty() {
                render_json_empty_line(ui, "{}", depth + 1);
                return false;
            }

            let mut changed = false;
            let keys = object.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                if let Some(child) = object.get_mut(&key) {
                    let child_path = json_child_path(path, &key);
                    changed |= render_json_value_node(ui, &key, child, depth + 1, &child_path);
                }
            }
            changed
        }
        serde_json::Value::Array(items) => {
            render_json_branch_header(
                ui,
                label,
                "array",
                &items.len().to_string(),
                depth,
                theme::ACCENT_TEAL,
            );
            if items.is_empty() {
                render_json_empty_line(ui, "[]", depth + 1);
                return false;
            }

            let mut changed = false;
            for (idx, item) in items.iter_mut().enumerate() {
                let child_path = format!("{path}[{idx}]");
                changed |=
                    render_json_value_node(ui, &format!("[{idx}]"), item, depth + 1, &child_path);
            }
            changed
        }
        _ => render_json_scalar_node(ui, label, value, depth, path),
    }
}

fn render_json_branch_header(
    ui: &mut egui::Ui,
    label: &str,
    kind: &str,
    count: &str,
    depth: usize,
    color: Color32,
) {
    ui.add_space(theme::SPACE_XS);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(label)
                .color(theme::text_primary())
                .strong()
                .size(11.5),
        );
        tiny_badge(ui, kind, color);
        tiny_badge(ui, count, theme::text_muted());
    });
}

fn render_json_scalar_node(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut serde_json::Value,
    depth: usize,
    path: &str,
) -> bool {
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(label)
                .color(theme::text_primary())
                .strong()
                .size(11.5),
        );
        tiny_badge(ui, json_value_kind(value), json_value_color(value));
    });

    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_space(json_depth_indent(depth + 1));
        changed |= render_json_scalar_control(ui, value, path);
    });
    changed
}

fn render_json_scalar_control(
    ui: &mut egui::Ui,
    value: &mut serde_json::Value,
    path: &str,
) -> bool {
    match value {
        serde_json::Value::String(text) => ui
            .add(
                theme::text_input(text)
                    .id_salt(("json_string", path.to_owned()))
                    .desired_width(ui.available_width()),
            )
            .changed(),
        serde_json::Value::Number(number) => {
            let buffer_id = ui.make_persistent_id(("json_number_buffer", path.to_owned()));
            let canonical = number.to_string();
            let mut text = ui
                .data_mut(|data| data.get_temp::<String>(buffer_id))
                .unwrap_or(canonical);
            let response = ui.add(
                theme::mono_text_input(&mut text)
                    .id_salt(("json_number", path.to_owned()))
                    .desired_width(ui.available_width()),
            );
            if response.changed() {
                ui.data_mut(|data| data.insert_temp(buffer_id, text.clone()));
            } else if !response.has_focus() {
                ui.data_mut(|data| {
                    data.remove_temp::<String>(buffer_id);
                });
            }
            if response.changed() {
                if let Ok(serde_json::Value::Number(parsed)) =
                    serde_json::from_str::<serde_json::Value>(&text)
                {
                    *value = serde_json::Value::Number(parsed);
                    return true;
                }
            }
            false
        }
        serde_json::Value::Bool(flag) => {
            let mut checked = *flag;
            let response = info_toggle_control(ui, &mut checked, &flag.to_string(), true);
            if response.changed() {
                *flag = checked;
                true
            } else {
                false
            }
        }
        serde_json::Value::Null => {
            let buffer_id = ui.make_persistent_id(("json_null_buffer", path.to_owned()));
            let mut text = ui
                .data_mut(|data| data.get_temp::<String>(buffer_id))
                .unwrap_or_else(|| "null".to_string());
            let response = ui.add(
                theme::mono_text_input(&mut text)
                    .id_salt(("json_null", path.to_owned()))
                    .desired_width(ui.available_width()),
            );
            if response.changed() {
                ui.data_mut(|data| data.insert_temp(buffer_id, text.clone()));
            } else if !response.has_focus() {
                ui.data_mut(|data| {
                    data.remove_temp::<String>(buffer_id);
                });
            }
            if response.changed() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    *value = parsed;
                    return true;
                }
            }
            false
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => false,
    }
}

fn json_child_path(parent: &str, key: &str) -> String {
    format!("{parent}.{}", key.replace('\\', "\\\\").replace('.', "\\."))
}

fn render_json_empty_line(ui: &mut egui::Ui, text: &str, depth: usize) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(text)
                .color(theme::text_muted())
                .monospace()
                .size(11.0),
        );
    });
}

fn json_value_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn json_value_color(value: &serde_json::Value) -> Color32 {
    match value {
        serde_json::Value::Null => theme::text_muted(),
        serde_json::Value::Bool(_) => theme::ACCENT_YELLOW,
        serde_json::Value::Number(_) => theme::ACCENT_COPPER_LIGHT,
        serde_json::Value::String(_) => theme::ACCENT_GREEN,
        serde_json::Value::Array(_) => theme::ACCENT_TEAL,
        serde_json::Value::Object(_) => theme::ACCENT_BLUE,
    }
}

fn json_depth_indent(depth: usize) -> f32 {
    (depth as f32 * 14.0).min(84.0)
}

fn render_info_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    if let Some(value) = dark_select_control(
        ui,
        ("info_enum_cell", row_idx, col_idx),
        &selected,
        enum_values,
        ui.available_width(),
    ) {
        edit.value = value;
    }
}

fn render_info_apply_controls(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let Some(summary) = data_edit_summary(state) else {
        return;
    };

    ui.add_space(theme::SPACE_LG);
    ui.separator();
    ui.add_space(theme::SPACE_MD);
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("grid_edits", &[&summary.dirty_count.to_string()]),
            summary.color,
        );
        if let Some(reason) = &summary.blocked_reason {
            ui.label(RichText::new(reason).color(theme::ACCENT_YELLOW).size(11.0));
        }
    });
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        let can_apply = summary.can_apply && !state.data_edit.applying;
        if info_text_action_button(ui, &t("button_apply"), can_apply).clicked() {
            match build_data_edits(state) {
                Ok(edits) => {
                    state.data_edit.applying = true;
                    state.last_error = None;
                    bridge.send(DbCommand::ApplyDataEdits {
                        conn_id: summary.conn_id,
                        edits,
                    });
                }
                Err(err) => {
                    state.last_error = Some(err);
                }
            }
        }

        if info_text_action_button(ui, &t("grid_revert"), !state.data_edit.applying).clicked() {
            revert_data_edits(state);
        }
    });
}

fn revert_data_cell(state: &mut AppState, cell_key: (usize, usize), type_name: &str) {
    let timezone = state.data_timezone.clone();
    if let Some(cell) = state.data_edit.cells.get_mut(&cell_key) {
        let value = cell_edit_text_for_type(&cell.original, type_name, &timezone);
        cell.value = value.clone();
        cell.original_text = value;
        cell.is_null = matches!(cell.original, CellValue::Null);
    }
    state.data_edit.editing_cell = None;
}

pub(super) fn editable_cell_display_text(cell: &crate::state::EditableCell) -> String {
    if cell.is_null {
        "NULL".to_string()
    } else {
        cell.value.clone()
    }
}

fn info_icon_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    enabled: bool,
    icon_color: Color32,
) -> egui::Response {
    let (rect, response) = info_action_button_frame(ui, 30.0, enabled);
    let icon_color = if enabled {
        icon_color
    } else {
        theme::text_disabled()
    };
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.set_clip_rect(icon_rect.intersect(rect));
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, icon_color,
            ));
        },
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, label);
    response
}

fn info_text_action_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let target_width = (text_width + 28.0).ceil().max(96.0);
    let (rect, response) = info_action_button_frame(ui, target_width, enabled);
    ui.painter()
        .with_clip_rect(rect.shrink2(egui::vec2(8.0, 2.0)))
        .text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font,
            text_color,
        );
    response
}

fn info_action_button_frame(
    ui: &mut egui::Ui,
    width: f32,
    enabled: bool,
) -> (egui::Rect, egui::Response) {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), sense);
    let hovered = enabled && response.hovered();
    let fill = if !enabled {
        theme::bg_medium()
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke_color = if hovered {
        theme::with_alpha(theme::ACCENT_TEAL, 140)
    } else if enabled {
        theme::border_default()
    } else {
        theme::border_subtle()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    set_pointing_cursor_on_hover(ui, &response, enabled);
    (rect, response)
}

fn info_toggle_control(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let font = egui::FontId::proportional(11.5);
    let label_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let width = (label_width + 30.0).min(ui.available_width()).max(48.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 22.0), sense);
    if response.clicked() && enabled {
        *checked = !*checked;
    }

    let hovered = enabled && response.hovered();
    let box_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(9.0, 0.0),
        egui::vec2(16.0, 16.0),
    );
    let box_fill = if *checked {
        theme::with_alpha(theme::ACCENT_TEAL, if hovered { 52 } else { 36 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let box_stroke = if *checked {
        theme::ACCENT_TEAL
    } else if hovered {
        theme::border_strong()
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(box_rect, CornerRadius::same(theme::RADIUS_SM), box_fill);
    ui.painter().rect_stroke(
        box_rect,
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(1.0, box_stroke),
        egui::StrokeKind::Inside,
    );
    if *checked {
        let a = box_rect.left_center() + egui::vec2(4.0, 0.5);
        let b = box_rect.center() + egui::vec2(-1.0, 4.0);
        let c = box_rect.right_center() + egui::vec2(-3.5, -4.5);
        ui.painter()
            .line_segment([a, b], Stroke::new(1.8, theme::ACCENT_TEAL));
        ui.painter()
            .line_segment([b, c], Stroke::new(1.8, theme::ACCENT_TEAL));
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(24.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    set_pointing_cursor_on_hover(ui, &response, enabled);
    response
}

pub(super) fn dark_select_control(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    selected_text: &str,
    options: &[String],
    width: f32,
) -> Option<String> {
    let popup_id = ui.make_persistent_id(("dark_select", id_source));
    let width = width.max(96.0);
    let response = dark_select_button(ui, selected_text, width);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    let mut selected_value = None;
    show_dark_popup_below(ui, popup_id, &response, width, theme::SPACE_SM_I, |ui| {
        for option in options {
            if dark_select_option(ui, option, option == selected_text, width).clicked() {
                selected_value = Some(option.clone());
                ui.memory_mut(|memory| memory.close_popup());
            }
        }
    });
    selected_value
}

fn dark_select_button(ui: &mut egui::Ui, selected_text: &str, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_medium()
    } else {
        theme::input_bg()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 150))
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let text_clip = rect.shrink2(egui::vec2(10.0, 0.0));
    ui.painter().with_clip_rect(text_clip).text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        selected_text,
        egui::FontId::proportional(12.0),
        theme::text_primary(),
    );
    let center = rect.right_center() - egui::vec2(14.0, 0.0);
    ui.painter().line_segment(
        [
            center + egui::vec2(-5.0, -2.5),
            center + egui::vec2(0.0, 3.5),
        ],
        Stroke::new(1.8, theme::text_secondary()),
    );
    ui.painter().line_segment(
        [
            center + egui::vec2(0.0, 3.5),
            center + egui::vec2(5.0, -2.5),
        ],
        Stroke::new(1.8, theme::text_secondary()),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn dark_select_option(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    width: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 34)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if selected {
            theme::ACCENT_TEAL
        } else {
            theme::text_secondary()
        },
    );
    if selected {
        ui.painter().circle_filled(
            rect.right_center() - egui::vec2(11.0, 0.0),
            3.0,
            theme::ACCENT_TEAL,
        );
    }
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn info_section_label(ui: &mut egui::Ui, label: &str) {
    ui.label(
        RichText::new(label)
            .color(theme::text_muted())
            .strong()
            .size(11.0),
    );
}

fn tiny_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(9.5),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 12.0, 17.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 28),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(9.5),
        color,
    );
}

fn value_box(ui: &mut egui::Ui, value: &str, color: Color32) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.add(
                egui::Label::new(RichText::new(value).color(color).monospace().size(11.5)).wrap(),
            );
        });
}
