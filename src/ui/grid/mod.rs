//! Data grid view (cell rendering, selection, hit-test, edit, paste, info
//! panel, footer, tooltips).
//!
//! Plan v7 Phase 1.95c1 — grid.rs (5240줄) 를 폴더 구조로 변환. sub-modules
//! 는 현재 빈 placeholder. 실제 함수 cut-over 는 후속 1.95c sub-stories 에서
//! 진행. Phase 1.95a 의 dispatch() wire-up 도 cut-over 후 진행.

mod footer;
mod hit_test;
mod info_panel;
mod paste;
mod render;
mod selection;
mod tooltips;

use footer::{
    render_data_query_footer, render_grid_body_with_reserved_footer, should_show_data_query_footer,
};
pub use render::render_grid;
use tooltips::show_dark_hover_tooltip;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::{Datelike, Timelike};
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, cell_edit_text_for_type, data_filter_from_cell,
    data_filter_from_text, data_timezone_offset_seconds, is_timestamp_without_timezone_type,
    is_timestamptz_type, timestamp_display_to_utc, timestamp_display_to_utc_naive, AppState,
    DataFilter, DataSortClause, DataSortDirection, DataSource, MainView, MAX_DATA_PAGE_LIMIT,
};
use crate::types::{
    CellValue, ColumnInfo, ColumnMeta, IndexInfo,
    RuleInfo, TriggerInfo,
};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

const GRID_CELL_LEFT_PAD: f32 = 12.0;
const GRID_CELL_RIGHT_PAD: f32 = 8.0;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

// render_grid 가 Phase 1.95c3c 에서 src/ui/grid/render.rs 로 cut-over (pub use 재출).

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

fn editable_cell_display_text(cell: &crate::state::EditableCell) -> String {
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

fn dark_select_control(
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

// should_show_data_query_footer / render_grid_body_with_reserved_footer /
// render_data_query_footer / render_data_query_preview / active_data_query_sql /
// data_query_footer_height 가 Phase 1.95c3b 에서 src/ui/grid/footer.rs 로 cut-over.

// show_dark_hover_tooltip / smart_tooltip_pos / estimate_tooltip_size /
// clamp_axis 가 Phase 1.95c3a 에서 src/ui/grid/tooltips.rs 로 cut-over.

// ---------------------------------------------------------------------------
// Error bar
// ---------------------------------------------------------------------------

// render_error_bar / render_empty_state 가 Phase 1.95c3c 에서 render.rs 로 cut-over.

// ---------------------------------------------------------------------------
// Result info header strip
// ---------------------------------------------------------------------------

pub(super) fn render_result_header(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;
    let data_edit_summary = data_edit_summary(state);

    let header_height = 56.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, theme::bg_shell());
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::border_subtle()),
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, 0.0));
    let content_rect = egui::Rect::from_center_size(
        inner.center(),
        egui::vec2(inner.width(), theme::BUTTON_HEIGHT),
    );
    let tsv_width = result_toolbar_action_width(ui, "Copy TSV");
    let csv_width = result_toolbar_action_width(ui, "CSV");
    let mut right_width = tsv_width + csv_width + theme::SPACE_SM;
    if data_edit_summary.is_some() {
        right_width += 330.0;
    }
    right_width = right_width.min(content_rect.width() * 0.46);

    let meta_width = result_meta_group_width(ui, row_count, col_count, exec_ms, truncated)
        .min((content_rect.width() - right_width - theme::SPACE_LG).max(120.0));
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.right() - right_width, content_rect.top()),
        content_rect.right_bottom(),
    );
    let meta_rect = egui::Rect::from_min_size(
        content_rect.left_top(),
        egui::vec2(meta_width, content_rect.height()),
    );
    let middle_rect = egui::Rect::from_min_max(
        egui::pos2(meta_rect.right() + theme::SPACE_LG, content_rect.top()),
        egui::pos2(right_rect.left() - theme::SPACE_LG, content_rect.bottom()),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(meta_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(meta_rect);
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
            ui.label(
                RichText::new("Result")
                    .color(theme::text_primary())
                    .strong()
                    .size(13.0),
            );
            ui.add_space(theme::SPACE_MD);
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    row_count,
                    if row_count == 1 { "row" } else { "rows" }
                ),
                theme::ACCENT_TEAL,
            );
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    col_count,
                    if col_count == 1 { "col" } else { "cols" }
                ),
                theme::ACCENT_BLUE,
            );
            result_meta_chip(ui, &format!("{exec_ms}ms"), theme::ACCENT_COPPER);

            if truncated {
                result_meta_chip_svg(
                    ui,
                    "trunc",
                    crate::ui::icons_svg::TRUNCATED,
                    "truncated_icon",
                    theme::ACCENT_YELLOW,
                );
            }
        },
    );

    if middle_rect.width() > 120.0 && state.active_main_view == MainView::Data {
        let pager_width = 488.0_f32.min(middle_rect.width());
        let pager_rect = egui::Rect::from_center_size(
            middle_rect.center(),
            egui::vec2(pager_width, content_rect.height()),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(pager_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(pager_rect);
                render_data_pager(ui, state, bridge, truncated, row_count);
            },
        );
    }

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(right_rect);
            if let Some(summary) = &data_edit_summary {
                let can_apply = summary.can_apply && !state.data_edit.applying;
                let apply_label = t("button_apply");
                let apply_button = if can_apply {
                    theme::primary_button(&apply_label)
                } else {
                    theme::secondary_button(&apply_label)
                };
                if ui.add_enabled(can_apply, apply_button).clicked() {
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

                ui.add_space(theme::SPACE_SM);

                if ui
                    .add_enabled(
                        !state.data_edit.applying,
                        theme::ghost_button(&t("grid_revert")),
                    )
                    .clicked()
                {
                    revert_data_edits(state);
                }

                ui.add_space(theme::SPACE_MD);
                metric_chip(
                    ui,
                    &tf("grid_edits", &[&summary.dirty_count.to_string()]),
                    summary.color,
                );

                if let Some(reason) = &summary.blocked_reason {
                    ui.label(RichText::new(reason).color(theme::ACCENT_YELLOW).size(11.0));
                }

                ui.add_space(theme::SPACE_LG);
            }

            let csv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::EXPORT,
                "export_csv",
                "CSV",
                true,
            );

            if csv_btn.clicked() {
                export_csv(state);
            }

            ui.add_space(theme::SPACE_SM);

            let tsv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_tsv",
                "Copy TSV",
                true,
            );

            if tsv_btn.clicked() {
                if let Some(ref result) = state.current_result {
                    let tsv = result_to_tsv(result);
                    ui.ctx().copy_text(tsv);
                }
            }
        },
    );
}

fn result_meta_group_width(
    ui: &egui::Ui,
    row_count: usize,
    col_count: usize,
    exec_ms: u128,
    truncated: bool,
) -> f32 {
    let title_width = ui
        .painter()
        .layout_no_wrap(
            "Result".to_string(),
            egui::FontId::proportional(13.0),
            theme::text_primary(),
        )
        .rect
        .width();
    let row_text = format!(
        "{} {}",
        row_count,
        if row_count == 1 { "row" } else { "rows" }
    );
    let col_text = format!(
        "{} {}",
        col_count,
        if col_count == 1 { "col" } else { "cols" }
    );
    let mut width = title_width
        + theme::SPACE_MD
        + result_meta_chip_width(ui, &row_text)
        + result_meta_chip_width(ui, &col_text)
        + result_meta_chip_width(ui, &format!("{exec_ms}ms"))
        + theme::SPACE_SM * 4.0;
    if truncated {
        width += result_meta_chip_svg_width(ui, "trunc") + theme::SPACE_SM;
    }
    width
}

pub(super) fn result_toolbar_action_width(ui: &egui::Ui, label: &str) -> f32 {
    let width = ui
        .painter()
        .layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(12.0),
            theme::text_secondary(),
        )
        .rect
        .width();
    (width + 38.0).max(58.0)
}

fn result_meta_chip_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width()
        + 18.0
}

fn result_meta_chip_svg_width(ui: &egui::Ui, text: &str) -> f32 {
    let text_width = ui
        .painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width();
    (text_width + 34.0).max(74.0)
}

fn render_data_pager(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    has_next_page: bool,
    visible_rows: usize,
) {
    if state.active_main_view != MainView::Data {
        return;
    }

    let page_index = state.data_edit.page_index;
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    let page_start = if visible_rows == 0 { 0 } else { offset + 1 };
    let page_end = if visible_rows == 0 {
        0
    } else {
        offset + visible_rows
    };
    let page_label = tf("grid_page_n", &[&(page_index + 1).to_string()]);
    let limit_label = tf("grid_limit_n", &[&limit.to_string()]);
    let range_label = tf(
        "grid_visible_range",
        &[&page_start.to_string(), &page_end.to_string()],
    );

    egui::Frame::new()
        .fill(theme::bg_darkest())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(theme::SPACE_SM_I, theme::SPACE_XS_I))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
                let first = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_DOUBLE_LEFT,
                    "data_first_page",
                    &t("grid_first_page"),
                    page_index > 0 && !state.query_running,
                );
                if first.clicked() {
                    set_data_page_index(state, bridge, 0);
                }

                let prev = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_LEFT,
                    "data_prev_page",
                    &t("grid_prev_page"),
                    page_index > 0 && !state.query_running,
                );
                if prev.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_sub(1));
                }

                result_toolbar_menu_button(ui, "page", &page_label, 64.0, |ui| {
                    result_popup_field_row(ui, &t("grid_page"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_index_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_page_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_page_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                let next = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_RIGHT,
                    "data_next_page",
                    &t("grid_next_page"),
                    has_next_page && !state.query_running,
                );
                if next.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_add(1));
                }

                ui.add_space(theme::SPACE_SM);
                result_toolbar_menu_button(ui, "limit", &limit_label, 78.0, |ui| {
                    result_popup_field_row(ui, &t("grid_limit"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_limit_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_limit_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_limit_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(range_label)
                        .color(theme::text_muted())
                        .size(11.0),
                );
            });
        });
}

fn pager_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
    enabled: bool,
) -> egui::Response {
    let color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(26.0, 26.0), enabled);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, color,
            ));
        },
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    response
}

pub(super) fn result_toolbar_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let font = egui::FontId::proportional(12.0);
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
    let width = (text_width + 38.0).max(58.0);
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icon_svg,
                icon_name,
                12.0,
                if enabled {
                    theme::ACCENT_TEAL
                } else {
                    theme::text_disabled()
                },
            ));
        },
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(28.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    response
}

fn result_toolbar_menu_button<R>(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    label: &str,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    let popup_id = ui.make_persistent_id(("result_toolbar_menu", id_source));
    let response = result_toolbar_text_button(ui, label, width, true);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }
    show_dark_popup_below(
        ui,
        popup_id,
        &response,
        160.0,
        theme::SPACE_MD_I,
        add_contents,
    );
    response
}

fn show_dark_popup_below<R>(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    response: &egui::Response,
    min_width: f32,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    if !ui.memory(|memory| memory.is_popup_open(popup_id)) {
        return;
    }

    let mut pos = response.rect.left_bottom() + egui::vec2(0.0, 4.0);
    if let Some(to_global) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        pos = to_global * pos;
    }
    let popup = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(margin))
                .show(ui, |ui| {
                    ui.set_width(min_width);
                    add_contents(ui);
                });
        });

    let should_close = ui.input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && popup.response.clicked_elsewhere());
    if should_close {
        ui.memory_mut(|memory| memory.close_popup());
    }
}

fn result_popup_field_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_field: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = COLUMN_GAP;
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_WIDTH, theme::INPUT_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    RichText::new(label)
                        .color(theme::text_secondary())
                        .size(11.5)
                        .strong(),
                );
            },
        );
        add_field(ui)
    })
    .inner
}

fn result_popup_apply_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + COLUMN_GAP);
        let width = ui.available_width().max(68.0);
        result_popup_action_button(ui, label, enabled, width)
    })
    .inner
}

fn result_popup_action_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    width: f32,
) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );
    response
}

fn result_toolbar_text_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 26.0), enabled);
    ui.painter().text(
        rect.center() - egui::vec2(4.0, 0.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.5),
        if enabled {
            theme::text_secondary()
        } else {
            theme::text_disabled()
        },
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(10.0, 1.0),
        egui::Align2::CENTER_CENTER,
        "⌄",
        egui::FontId::proportional(10.0),
        theme::text_muted(),
    );
    response
}

fn result_toolbar_button_frame(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    enabled: bool,
) -> (egui::Rect, egui::Response) {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let hovered = enabled && response.hovered();
    let fill = if !enabled {
        theme::bg_darkest()
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
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
    set_pointing_cursor_on_hover(ui, &response, enabled);
    (rect, response)
}

fn result_meta_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 18.0, 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 2.7, color);
    painter.text(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

fn result_meta_chip_svg(ui: &mut egui::Ui, text: &str, svg: &str, name: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2((galley.rect.width() + 34.0).max(74.0), 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(11.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(ui, svg, name, 12.0, color));
        },
    );
    painter.text(
        rect.left_center() + egui::vec2(22.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

fn metric_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 18.0, 20.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    ui.painter()
        .circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 2.5, color);
    ui.painter().text(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

// ---------------------------------------------------------------------------
// Result table
// ---------------------------------------------------------------------------

fn render_header_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    column_name: &str,
    type_name: &str,
) {
    let cell_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(cell_width, 26.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(GRID_CELL_LEFT_PAD);
            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.label(
                    RichText::new(column_name)
                        .color(theme::text_primary())
                        .strong()
                        .size(12.0),
                );
                ui.label(
                    RichText::new(type_name)
                        .color(theme::text_muted())
                        .size(9.5)
                        .monospace(),
                );
            });

            if state.active_main_view != MainView::Data {
                return;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_sort_menu(ui, state, bridge, column_name);
            });
        },
    );
}

fn render_sort_menu(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge, column_name: &str) {
    let sort_index = state
        .data_edit
        .sort
        .iter()
        .position(|clause| clause.column == column_name);
    let direction = sort_index.map(|idx| state.data_edit.sort[idx].direction);
    let (icon, icon_name, icon_color) = match direction {
        Some(DataSortDirection::Asc) => (
            crate::ui::icons_svg::SORT_ASC,
            "header_sort_asc",
            theme::ACCENT_TEAL,
        ),
        Some(DataSortDirection::Desc) => (
            crate::ui::icons_svg::SORT_DESC,
            "header_sort_desc",
            theme::ACCENT_COPPER_LIGHT,
        ),
        None => (
            crate::ui::icons_svg::SORT,
            "header_sort",
            theme::text_muted(),
        ),
    };
    let popup_id = ui.make_persistent_id(("header_sort_menu", column_name));
    let response = render_header_sort_button(ui, icon, icon_name, icon_color, sort_index);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    show_dark_popup_below(ui, popup_id, &response, 184.0, theme::SPACE_SM_I, |ui| {
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT_ASC,
            "sort_menu_asc",
            &t("grid_sort_asc"),
            theme::ACCENT_TEAL,
            true,
            direction == Some(DataSortDirection::Asc),
        )
        .clicked()
        {
            set_sort_clause(state, bridge, column_name, DataSortDirection::Asc);
            ui.memory_mut(|memory| memory.close_popup());
        }
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT_DESC,
            "sort_menu_desc",
            &t("grid_sort_desc"),
            theme::ACCENT_TEAL,
            true,
            direction == Some(DataSortDirection::Desc),
        )
        .clicked()
        {
            set_sort_clause(state, bridge, column_name, DataSortDirection::Desc);
            ui.memory_mut(|memory| memory.close_popup());
        }
        sort_menu_separator(ui);
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT,
            "sort_menu_remove",
            &t("grid_sort_remove"),
            theme::text_muted(),
            sort_index.is_some(),
            false,
        )
        .clicked()
        {
            remove_sort_clause(state, bridge, column_name);
            ui.memory_mut(|memory| memory.close_popup());
        }
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::CLOSE,
            "sort_menu_clear",
            &t("grid_sort_clear_all"),
            theme::ACCENT_RED,
            !state.data_edit.sort.is_empty(),
            false,
        )
        .clicked()
        {
            clear_sort_clauses(state, bridge);
            ui.memory_mut(|memory| memory.close_popup());
        }
    });
}

fn render_header_sort_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    color: Color32,
    sort_index: Option<usize>,
) -> egui::Response {
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(24.0, 24.0), true);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, color,
            ));
        },
    );

    if let Some(idx) = sort_index {
        let badge_rect = egui::Rect::from_center_size(
            rect.right_top() + egui::vec2(-4.0, 4.0),
            egui::vec2(11.0, 11.0),
        );
        ui.painter().circle_filled(badge_rect.center(), 5.5, color);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            (idx + 1).to_string(),
            egui::FontId::proportional(8.0),
            theme::bg_darkest(),
        );
    }

    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("grid_sort_asc"),
    );
    response
}

fn sort_menu_item(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    color: Color32,
    enabled: bool,
    selected: bool,
) -> egui::Response {
    let full_width = ui.available_width().max(184.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(full_width, 30.0), sense);
    let hovered = enabled && response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 26)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }

    let icon_color = if enabled {
        color
    } else {
        theme::text_disabled()
    };
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(13.0, 13.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, icon_color,
            ));
        },
    );

    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(32.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    if selected {
        ui.painter().circle_filled(
            rect.right_center() - egui::vec2(13.0, 0.0),
            3.0,
            theme::ACCENT_TEAL,
        );
    }

    set_pointing_cursor_on_hover(ui, &response, enabled);
    response
}

fn sort_menu_separator(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 9.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, theme::border_default()),
    );
}

pub(super) fn render_table(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match state.current_result.clone() {
        Some(r) => r,
        None => return,
    };

    if result.columns.is_empty() {
        return;
    }

    let available_width = ui.available_width();
    let column_widths = compute_column_widths(ui, &result);
    let content_width = column_widths.iter().sum::<f32>().max(available_width);
    let row_height = 28.0;
    let header_height = 30.0;
    let header_bg = theme::bg_medium();

    ensure_foreign_keys_for_active_data_source(state, bridge);

    let table_id = grid_table_id(state, &result, &column_widths);
    egui::ScrollArea::horizontal()
        .id_salt(format!("grid_hscroll_{table_id}"))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.scope(|ui| {
                apply_grid_table_visuals(ui);
                let mut table = TableBuilder::new(ui)
                    .id_salt(table_id)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

                for width in &column_widths {
                    table = table.column(
                        Column::initial(*width)
                            .clip(true)
                            .at_least(64.0)
                            .at_most(560.0),
                    );
                }

                table
                    .header(header_height, |mut header| {
                        for col in &result.columns {
                            header.col(|ui| {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(rect, 0.0, header_bg);
                                render_header_cell(ui, state, bridge, &col.name, &col.type_name);
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(row_height, result.rows.len(), |mut row| {
                            let row_idx = row.index();
                            let row_data = &result.rows[row_idx];
                            for (col_idx, cell) in row_data.iter().enumerate() {
                                row.col(|ui| {
                                    ui.add_space(GRID_CELL_LEFT_PAD);
                                    if state.active_main_view == MainView::Data {
                                        let column = result.columns.get(col_idx);
                                        render_editable_cell(
                                            ui, state, bridge, row_idx, col_idx, cell, column,
                                        );
                                    } else {
                                        render_cell(ui, cell);
                                    }
                                });
                            }
                        });
                    });
            });
        });
}

fn apply_grid_table_visuals(ui: &mut egui::Ui) {
    let mut style = (**ui.style()).clone();
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, grid_separator_color());
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, grid_separator_hover_color());
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, grid_separator_active_color());
    ui.set_style(style);
}

fn grid_separator_color() -> Color32 {
    if theme::is_dark() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 24)
    } else {
        theme::border_default()
    }
}

fn grid_separator_hover_color() -> Color32 {
    if theme::is_dark() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 54)
    } else {
        theme::border_strong()
    }
}

fn grid_separator_active_color() -> Color32 {
    if theme::is_dark() {
        theme::with_alpha(theme::ACCENT_TEAL, 150)
    } else {
        theme::ACCENT_TEAL
    }
}

fn compute_column_widths(ui: &egui::Ui, result: &crate::types::QueryResult) -> Vec<f32> {
    result
        .columns
        .iter()
        .enumerate()
        .map(|(col_idx, column)| {
            let header_width = measure_text_width(
                ui,
                &format!("{}  {}", column.name, column.type_name),
                egui::FontId::proportional(12.0),
            ) + 58.0;

            let max_sample_width = result
                .rows
                .iter()
                .take(80)
                .filter_map(|row| row.get(col_idx))
                .map(|cell| {
                    let sample = cell_auto_width_text(cell);
                    let font = if matches!(cell, CellValue::Text(_)) {
                        egui::FontId::proportional(12.0)
                    } else {
                        egui::FontId::monospace(12.0)
                    };
                    measure_text_width(ui, &sample, font) + cell_width_padding(cell)
                })
                .fold(0.0_f32, f32::max);

            let base = header_width.max(max_sample_width);
            let max_width = column_width_cap(&column.type_name);
            base.clamp(72.0, max_width)
        })
        .collect()
}

fn measure_text_width(ui: &egui::Ui, text: &str, font_id: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_string(), font_id, theme::text_primary())
        .rect
        .width()
}

fn cell_auto_width_text(cell: &CellValue) -> String {
    let text = cell.to_string();
    const MAX_SAMPLE_CHARS: usize = 96;
    if text.chars().count() <= MAX_SAMPLE_CHARS {
        text
    } else {
        let mut truncated = text.chars().take(MAX_SAMPLE_CHARS).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

fn cell_width_padding(cell: &CellValue) -> f32 {
    match cell {
        CellValue::Bool(_) | CellValue::Null => 42.0,
        CellValue::Int(_) | CellValue::Float(_) => 32.0,
        CellValue::Uuid(_) => 26.0,
        CellValue::Timestamp(_) => 34.0,
        CellValue::Json(_) | CellValue::Bytes(_) => 46.0,
        CellValue::Text(_) | CellValue::Unknown(_) => 34.0,
    }
}

fn column_width_cap(type_name: &str) -> f32 {
    match type_name.to_ascii_lowercase().as_str() {
        "uuid" => 310.0,
        "bool" | "boolean" => 110.0,
        "int2" | "int4" | "int8" | "smallint" | "integer" | "bigint" | "numeric" | "decimal"
        | "float4" | "float8" | "real" | "double precision" => 150.0,
        "date"
        | "timestamp"
        | "timestamptz"
        | "timestamp without time zone"
        | "timestamp with time zone" => 230.0,
        "json" | "jsonb" => 520.0,
        "bytea" => 360.0,
        _ => 420.0,
    }
}

fn grid_table_id(
    state: &AppState,
    result: &crate::types::QueryResult,
    column_widths: &[f32],
) -> String {
    let source = state
        .active_data_source()
        .map(|source| {
            let filter = source
                .filter
                .as_ref()
                .map(|filter| format!("_{}_{}", filter.column, filter.sql_value))
                .unwrap_or_default();
            format!(
                "{}_{}_{}{}",
                source.conn_id, source.schema, source.table, filter
            )
        })
        .unwrap_or_else(|| "query_result".to_string());
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    for column in &result.columns {
        column.name.hash(&mut hasher);
        column.type_name.hash(&mut hasher);
    }
    for width in column_widths {
        (*width as u32).hash(&mut hasher);
    }
    format!("grid_{:x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// Cell rendering
// ---------------------------------------------------------------------------

fn render_editable_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
) {
    if !has_table_column_metadata(state) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            None,
        );
        return;
    }

    if !state.data_edit.cells.contains_key(&(row_idx, col_idx)) {
        state.data_edit.cells.insert(
            (row_idx, col_idx),
            crate::state::EditableCell::from_cell_for_type(
                fallback_cell,
                column.map(|col| col.type_name.as_str()).unwrap_or_default(),
                &state.data_timezone,
            ),
        );
    }

    let column_info = column.and_then(|col| data_column_info(state, &col.name).cloned());
    if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            Some(&t("data_info_read_only_pk")),
        );
        return;
    }

    let cell_key = (row_idx, col_idx);
    let type_name = column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .or_else(|| column.map(|col| col.type_name.clone()))
        .unwrap_or_default();
    let nullable = column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();

    let Some(snapshot) = state.data_edit.cells.get(&cell_key).cloned() else {
        render_cell(ui, fallback_cell);
        return;
    };

    let dirty = snapshot.is_dirty();
    let error = validate_edit_value(&snapshot, &type_name, nullable, &enum_values);
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();
    if selected {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_TEAL, 34),
        );
    }
    if dirty {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_COPPER, 30),
        );
    } else if error.is_some() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_RED, 28),
        );
    }
    if selected {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(1.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::ACCENT_TEAL),
            egui::StrokeKind::Inside,
        );
    }

    let is_editing = state.data_edit.editing_cell == Some(cell_key);
    if !is_editing {
        let relation_target = if !snapshot.is_null && error.is_none() {
            relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
                data_filter_from_text(fk.target_column.clone(), type_name.clone(), &snapshot.value)
                    .map(|filter| (fk, filter))
            })
        } else {
            None
        };
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("data_cell", row_idx, col_idx)),
            egui::Sense::click(),
        );
        let copy_text = editable_cell_display_text(&snapshot);
        show_cell_copy_context_menu(&response, &copy_text);
        let content_width = relation_content_width(rect, relation_target.is_some());
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, rect.height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                render_editable_display_cell(ui, &snapshot, fallback_cell, &type_name);
                if dirty {
                    ui.add_space(2.0);
                    ui.painter().circle_filled(
                        ui.cursor().left_center() + egui::vec2(4.0, 0.0),
                        2.0,
                        theme::ACCENT_COPPER,
                    );
                    ui.add_space(8.0);
                }
            },
        );
        let relation_clicked = if let Some((fk, filter)) = relation_target {
            let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
            if clicked {
                open_related_data(state, bridge, &fk, filter);
            }
            clicked
        } else {
            false
        };

        if response.clicked() && !relation_clicked {
            select_data_cell(state, row_idx, col_idx, true);
        }
        if let Some(error) = error {
            show_dark_hover_tooltip(ui, response.id.with("error"), &response, &error);
        }
        return;
    }

    let mut close_editor = false;
    let Some(edit) = state.data_edit.cells.get_mut(&cell_key) else {
        render_cell(ui, fallback_cell);
        return;
    };

    let editor_rect = cell_overlay_editor_rect(rect, nullable);
    if nullable && cell_overlay_null_toggle(ui, rect, cell_key, edit.is_null).clicked() {
        edit.is_null = !edit.is_null;
    }

    if edit.is_null {
        render_cell_overlay_value(ui, editor_rect, "NULL", theme::text_muted());
    } else if !enum_values.is_empty() {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(editor_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(editor_rect);
                close_editor |= render_enum_editor(ui, edit, row_idx, col_idx, &enum_values);
            },
        );
    } else {
        match edit_kind(&type_name, fallback_cell) {
            EditKind::Bool => {
                close_editor |= render_cell_bool_overlay(ui, editor_rect, edit);
            }
            EditKind::Date => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::DateTime => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Number | EditKind::Json | EditKind::Uuid | EditKind::Bytes => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Text => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, false, error.as_deref());
            }
        }
    }

    if close_editor || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.data_edit.editing_cell = None;
    }
}

fn cell_overlay_editor_rect(cell_rect: egui::Rect, nullable: bool) -> egui::Rect {
    let left_pad = if nullable { 38.0 } else { 0.0 };
    let left = cell_rect.left() + left_pad;
    let right = (cell_rect.right() - GRID_CELL_RIGHT_PAD).max(left + 28.0);
    egui::Rect::from_min_max(
        egui::pos2(left, cell_rect.center().y - 12.0),
        egui::pos2(right, cell_rect.center().y + 12.0),
    )
}

fn cell_overlay_null_toggle(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    checked: bool,
) -> egui::Response {
    let rect = egui::Rect::from_center_size(
        egui::pos2(cell_rect.left() + 17.0, cell_rect.center().y),
        egui::vec2(32.0, 18.0),
    );
    let response = ui.interact(
        rect,
        ui.make_persistent_id(("cell_null_toggle", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let fill = if checked {
        theme::with_alpha(theme::ACCENT_TEAL, if hovered { 52 } else { 34 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if checked {
        theme::ACCENT_TEAL
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "NULL",
        egui::FontId::proportional(9.5),
        if checked {
            theme::ACCENT_TEAL
        } else {
            theme::text_muted()
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("grid_toggle_null"),
    );
    response
}

fn render_cell_text_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
    monospace: bool,
    error: Option<&str>,
) -> bool {
    let response = ui.put(
        rect,
        cell_overlay_text_input(&mut edit.value, monospace).desired_width(rect.width()),
    );
    response.request_focus();
    if let Some(error) = error {
        show_dark_hover_tooltip(ui, response.id.with("error"), &response, error);
    }
    enter_pressed(ui)
}

fn cell_overlay_text_input(text: &mut String, monospace: bool) -> egui::TextEdit<'_> {
    let input = egui::TextEdit::singleline(text)
        .background_color(theme::bg_darkest())
        .text_color(theme::text_primary())
        .margin(Margin::symmetric(7, 2))
        .min_size(egui::vec2(0.0, 24.0))
        .vertical_align(egui::Align::Center);
    if monospace {
        input.font(egui::TextStyle::Monospace)
    } else {
        input.font(egui::TextStyle::Body)
    }
}

fn render_cell_bool_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
) -> bool {
    let mut checked = parse_bool(&edit.value).unwrap_or(false);
    let response = ui.put(rect, egui::Checkbox::new(&mut checked, ""));
    if response.changed() {
        edit.value = checked.to_string();
    }
    response.lost_focus() && enter_pressed(ui)
}

fn render_cell_overlay_value(ui: &mut egui::Ui, rect: egui::Rect, text: &str, color: Color32) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::bg_darkest(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::monospace(12.0),
        color,
    );
}

fn render_readonly_data_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    cell_key: (usize, usize),
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
    tooltip: Option<&str>,
) {
    let (row_idx, col_idx) = cell_key;
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();

    if selected {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_TEAL, 30),
        );
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(1.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::ACCENT_TEAL),
            egui::StrokeKind::Inside,
        );
    }

    let response = ui.interact(
        rect,
        ui.make_persistent_id(("data_cell_readonly", row_idx, col_idx)),
        egui::Sense::click(),
    );
    let copy_text = fallback_cell.to_string();
    show_cell_copy_context_menu(&response, &copy_text);
    let relation_target =
        relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
            let type_name = column
                .map(|col| col.type_name.clone())
                .unwrap_or_else(|| fk.source_column.clone());
            data_filter_from_cell(fk.target_column.clone(), type_name, fallback_cell)
                .map(|filter| (fk, filter))
        });
    let content_width = relation_content_width(rect, relation_target.is_some());
    ui.allocate_ui_with_layout(
        egui::vec2(content_width, rect.height()),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            render_passive_cell(ui, fallback_cell);
        },
    );

    let relation_clicked = if let Some((fk, filter)) = relation_target {
        let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
        if clicked {
            open_related_data(state, bridge, &fk, filter);
        }
        clicked
    } else {
        false
    };

    if response.clicked() && !relation_clicked {
        select_data_cell(state, row_idx, col_idx, false);
    }
    if let Some(tooltip) = tooltip {
        show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    }
}

fn select_data_cell(state: &mut AppState, row_idx: usize, col_idx: usize, editable: bool) {
    let cell_key = (row_idx, col_idx);
    state.data_edit.selected_cell = Some(cell_key);
    state.data_edit.editing_cell = editable.then_some(cell_key);
    state.show_info_panel = true;
}

fn ensure_foreign_keys_for_active_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    request_foreign_keys_for_schema(state, bridge, source.conn_id, &source.schema);
}

fn request_foreign_keys_for_schema(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: crate::types::ConnectionId,
    schema: &str,
) {
    let should_request = state.connections.get(&conn_id).is_some_and(|conn| {
        !conn.foreign_keys.contains_key(schema) && !conn.loading_foreign_keys.contains(schema)
    });
    if !should_request {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.loading_foreign_keys.insert(schema.to_string());
    }
    bridge.send(DbCommand::ListForeignKeys {
        conn_id,
        schema: schema.to_string(),
    });
}

fn request_table_columns_for_data(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: crate::types::ConnectionId,
    schema: &str,
    table: &str,
) {
    let key = (schema.to_string(), table.to_string());
    let should_request = state.connections.get(&conn_id).is_some_and(|conn| {
        !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key)
    });
    if !should_request {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.loading_columns.insert(key);
    }
    bridge.send(DbCommand::ListColumns {
        conn_id,
        schema: schema.to_string(),
        table: table.to_string(),
    });
}

fn relation_for_column(state: &AppState, column_name: Option<&str>) -> Option<ForeignKey> {
    let column_name = column_name?;
    let source = state.active_data_source()?;
    state
        .connections
        .get(&source.conn_id)?
        .foreign_keys
        .get(&source.schema)?
        .iter()
        .find(|fk| {
            fk.source_schema == source.schema
                && fk.source_table == source.table
                && fk.source_column == column_name
        })
        .cloned()
}

fn relation_content_width(cell_rect: egui::Rect, has_relation: bool) -> f32 {
    if has_relation {
        (cell_rect.width() - 34.0).max(0.0)
    } else {
        cell_rect.width()
    }
}

fn render_relation_jump_button(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    selected: bool,
) -> bool {
    let button_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.right() - 30.0, cell_rect.center().y - 12.0),
        egui::pos2(cell_rect.right() - 6.0, cell_rect.center().y + 12.0),
    );
    let response = ui.interact(
        button_rect,
        ui.make_persistent_id(("relation_jump", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let emphasized = hovered || selected;
    let fill = if emphasized {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::ACCENT_TEAL)
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(button_rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        button_rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let icon_color = if emphasized {
        theme::ACCENT_TEAL
    } else {
        theme::with_alpha(theme::ACCENT_TEAL, 190)
    };
    let center = button_rect.center();
    let tip = egui::pos2(center.x + 3.0, center.y);
    ui.painter().line_segment(
        [egui::pos2(center.x - 3.5, center.y - 5.5), tip],
        Stroke::new(2.2, icon_color),
    );
    ui.painter().line_segment(
        [tip, egui::pos2(center.x - 3.5, center.y + 5.5)],
        Stroke::new(2.2, icon_color),
    );

    if hovered {
        set_pointing_cursor_on_hover(ui, &response, true);
    }
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("data_relation_open"),
    );

    response.clicked()
}

fn open_related_data(state: &mut AppState, bridge: &DbBridge, fk: &ForeignKey, filter: DataFilter) {
    let Some(conn_id) = state.active_data_source().map(|source| source.conn_id) else {
        return;
    };

    state.active_connection = Some(conn_id);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    state.last_error = None;

    let title = relation_tab_title(fk, &filter.display_value);
    state.open_data_workspace_view(
        title,
        fk.target_schema.clone(),
        fk.target_table.clone(),
        Some(filter.clone()),
    );
    state.begin_data_edit_with_filter(
        conn_id,
        &fk.target_schema,
        &fk.target_table,
        Some(filter.clone()),
    );
    request_table_columns_for_data(state, bridge, conn_id, &fk.target_schema, &fk.target_table);
    request_foreign_keys_for_schema(state, bridge, conn_id, &fk.target_schema);

    let source = DataSource {
        conn_id,
        schema: fk.target_schema.clone(),
        table: fk.target_table.clone(),
        filter: Some(filter),
    };
    let limit = state.data_edit.page_limit;
    let columns = state.data_columns_for_source(&source);
    bridge.send(DbCommand::ExecuteQuery {
        conn_id,
        sql: build_data_select_sql_with_columns(&source, &state.data_edit.sort, limit, 0, &columns),
        row_limit: Some(limit),
    });
}

fn relation_tab_title(fk: &ForeignKey, value: &str) -> String {
    format!(
        "{}.{} · {}",
        fk.target_schema,
        fk.target_table,
        compact_relation_value(value)
    )
}

fn compact_relation_value(value: &str) -> String {
    const MAX_CHARS: usize = 22;
    if value.chars().count() <= MAX_CHARS {
        value.to_string()
    } else {
        let mut compact = value.chars().take(MAX_CHARS).collect::<String>();
        compact.push_str("...");
        compact
    }
}

fn render_editable_display_cell(
    ui: &mut egui::Ui,
    edit: &crate::state::EditableCell,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    if edit.is_null {
        passive_value_pill(ui, "NULL", theme::text_muted());
        return;
    }

    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let value = parse_bool(&edit.value).unwrap_or(false);
            let (text, color) = if value {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            passive_value_pill(ui, text, color);
        }
        EditKind::Number => {
            render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT)
        }
        EditKind::Json => render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_TEAL),
        EditKind::Date | EditKind::DateTime => {
            render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_BLUE)
        }
        EditKind::Uuid => render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT),
        EditKind::Bytes => render_passive_copyable_cell(ui, &edit.value, theme::text_muted()),
        EditKind::Text => render_passive_copyable_cell(ui, &edit.value, theme::text_primary()),
    }
}

fn render_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) -> bool {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    if let Some(value) = dark_select_control(
        ui,
        ("enum_cell", row_idx, col_idx),
        &selected,
        enum_values,
        ui.available_width().max(96.0),
    ) {
        edit.value = value;
        true
    } else {
        false
    }
}

fn render_date_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    include_time: bool,
    timezone: &str,
    error: Option<&str>,
) -> bool {
    let (mut date, mut time) = split_datetime_value(&edit.value);
    let mut close_editor = false;
    let mut changed = false;
    let mut use_now = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
        let date_response = ui.add(
            theme::mono_text_input(&mut date)
                .desired_width(108.0)
                .hint_text("YYYY-MM-DD"),
        );
        changed |= date_response.changed();
        close_editor |= date_response.lost_focus() && enter_pressed(ui);
        if let Some(error) = error {
            show_dark_hover_tooltip(ui, date_response.id.with("error"), &date_response, error);
        }
        changed |= render_date_picker_button(ui, &mut date);

        if include_time {
            let time_response = ui.add(
                theme::mono_text_input(&mut time)
                    .desired_width(92.0)
                    .hint_text("HH:MM:SS"),
            );
            changed |= time_response.changed();
            close_editor |= time_response.lost_focus() && enter_pressed(ui);
            if let Some(error) = error {
                show_dark_hover_tooltip(ui, time_response.id.with("error"), &time_response, error);
            }
            changed |= render_time_picker_button(ui, &mut time);
        }

        use_now = inline_dark_text_button(ui, &t("grid_now")).clicked();
    });

    if changed {
        edit.value = compose_datetime_edit_value(&date, &time, include_time);
    }

    if use_now {
        let now_utc = chrono::Utc::now();
        edit.value = if include_time {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d").to_string())
        };
        close_editor = true;
    }

    close_editor
}

fn compose_datetime_edit_value(date: &str, time: &str, include_time: bool) -> String {
    if include_time {
        format!("{} {}", date.trim(), time.trim())
            .trim()
            .to_string()
    } else {
        date.trim().to_string()
    }
}

fn render_date_picker_button(ui: &mut egui::Ui, date: &mut String) -> bool {
    let selected = parse_picker_date(date).unwrap_or_else(default_picker_date);
    let response = picker_icon_button(
        ui,
        crate::ui::icons_svg::CALENDAR,
        "date_picker_icon",
        &t("grid_pick_date"),
    );
    let popup_id = response.id.with("date_picker");
    if response.clicked() {
        let opening = !ui.memory(|memory| memory.is_popup_open(popup_id));
        ui.memory_mut(|memory| {
            if opening {
                memory.data.insert_temp(
                    popup_id.with("visible_month"),
                    (selected.year(), selected.month()),
                );
            }
            memory.toggle_popup(popup_id);
        });
    }

    let mut changed = false;
    show_dark_popup_below(ui, popup_id, &response, 238.0, theme::SPACE_MD_I, |ui| {
        changed |= render_date_picker_calendar(ui, popup_id, selected, date);
    });
    changed
}

fn render_date_picker_calendar(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    selected: chrono::NaiveDate,
    date: &mut String,
) -> bool {
    let visible_id = popup_id.with("visible_month");
    let (mut year, mut month) = ui
        .memory(|memory| memory.data.get_temp::<(i32, u32)>(visible_id))
        .unwrap_or((selected.year(), selected.month()));
    let mut changed = false;

    ui.horizontal(|ui| {
        if picker_nav_button(ui, "<", &t("grid_prev_month")).clicked() {
            (year, month) = shifted_year_month(year, month, -1);
            ui.memory_mut(|memory| memory.data.insert_temp(visible_id, (year, month)));
        }
        ui.add_space(theme::SPACE_XS);
        ui.label(
            RichText::new(format!("{year:04}-{month:02}"))
                .color(theme::text_primary())
                .monospace()
                .strong()
                .size(13.0),
        );
        ui.add_space(theme::SPACE_XS);
        if picker_nav_button(ui, ">", &t("grid_next_month")).clicked() {
            (year, month) = shifted_year_month(year, month, 1);
            ui.memory_mut(|memory| memory.data.insert_temp(visible_id, (year, month)));
        }
    });

    ui.add_space(theme::SPACE_SM);
    let labels = date_picker_weekday_labels();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for label in labels {
            date_picker_label_cell(ui, &label);
        }
    });

    let Some(first_day) = chrono::NaiveDate::from_ymd_opt(year, month, 1) else {
        return false;
    };
    let leading = first_day.weekday().num_days_from_monday() as i32;
    let days = days_in_month(year, month) as i32;
    let today = chrono::Utc::now().date_naive();
    let mut day = 1;

    for week in 0..6 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for weekday in 0..7 {
                let cell_index = week * 7 + weekday;
                if cell_index < leading || day > days {
                    ui.allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::hover());
                    continue;
                }

                let candidate =
                    chrono::NaiveDate::from_ymd_opt(year, month, day as u32).unwrap_or(selected);
                if date_picker_day_cell(ui, day as u32, candidate == selected, candidate == today)
                    .clicked()
                {
                    *date = candidate.format("%Y-%m-%d").to_string();
                    changed = true;
                    ui.memory_mut(|memory| memory.close_popup());
                }
                day += 1;
            }
        });
        if day > days {
            break;
        }
    }

    changed
}

fn render_time_picker_button(ui: &mut egui::Ui, time: &mut String) -> bool {
    let response = picker_icon_button(
        ui,
        crate::ui::icons_svg::CLOCK,
        "time_picker_icon",
        &t("grid_pick_time"),
    );
    let popup_id = response.id.with("time_picker");
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    let mut changed = false;
    show_dark_popup_below(ui, popup_id, &response, 196.0, theme::SPACE_MD_I, |ui| {
        changed |= render_time_picker(ui, time);
    });
    changed
}

fn render_time_picker(ui: &mut egui::Ui, time: &mut String) -> bool {
    let mut parsed = parse_picker_time(time).unwrap_or_else(default_picker_time);
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SPACE_MD;
        changed |= render_time_unit_picker(ui, &t("grid_hour"), &mut parsed.0, 23);
        changed |= render_time_unit_picker(ui, &t("grid_minute"), &mut parsed.1, 59);
        changed |= render_time_unit_picker(ui, &t("grid_second"), &mut parsed.2, 59);
    });

    if changed {
        *time = format!("{:02}:{:02}:{:02}", parsed.0, parsed.1, parsed.2);
    }
    changed
}

fn render_time_unit_picker(ui: &mut egui::Ui, label: &str, value: &mut u32, max: u32) -> bool {
    let mut changed = false;
    ui.allocate_ui_with_layout(
        egui::vec2(48.0, 118.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.label(
                RichText::new(label)
                    .color(theme::text_muted())
                    .strong()
                    .size(10.0),
            );
            if picker_step_button(ui, "+").clicked() {
                *value = if *value >= max { 0 } else { *value + 1 };
                changed = true;
            }
            let value_response = time_value_cell(ui, *value);
            if value_response.hovered() {
                let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
                if scroll_y > 4.0 {
                    *value = if *value >= max { 0 } else { *value + 1 };
                    changed = true;
                } else if scroll_y < -4.0 {
                    *value = if *value == 0 { max } else { *value - 1 };
                    changed = true;
                }
            }
            if picker_step_button(ui, "-").clicked() {
                *value = if *value == 0 { max } else { *value - 1 };
                changed = true;
            }
        },
    );
    changed
}

fn picker_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 170))
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
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(14.0, 14.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.set_clip_rect(icon_rect);
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icon_svg,
                icon_name,
                14.0,
                theme::ACCENT_BLUE,
            ));
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(ui, response.id.with("picker_tooltip"), &response, tooltip);
    response
}

fn picker_nav_button(ui: &mut egui::Ui, label: &str, tooltip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 26.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_darkest()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(13.0),
        theme::text_secondary(),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    response
}

fn date_picker_label_cell(ui: &mut egui::Ui, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 20.0), egui::Sense::hover());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        theme::text_muted(),
    );
}

fn date_picker_day_cell(
    ui: &mut egui::Ui,
    day: u32,
    selected: bool,
    today: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 54)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }
    let stroke = if selected {
        Some(Stroke::new(1.0, theme::ACCENT_TEAL))
    } else if today {
        Some(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 150)))
    } else {
        None
    };
    if let Some(stroke) = stroke {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(theme::RADIUS_MD),
            stroke,
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        day.to_string(),
        egui::FontId::monospace(12.0),
        if selected {
            theme::ACCENT_TEAL
        } else {
            theme::text_secondary()
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn picker_step_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(44.0, 22.0), egui::Sense::click());
    let fill = if response.hovered() {
        theme::bg_light()
    } else {
        theme::bg_darkest()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(13.0),
        theme::text_secondary(),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn time_value_cell(ui: &mut egui::Ui, value: u32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(44.0, 30.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::input_bg(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(
            1.0,
            if response.hovered() {
                theme::ACCENT_BLUE
            } else {
                theme::border_default()
            },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{value:02}"),
        egui::FontId::monospace(14.0),
        theme::text_primary(),
    );
    response
}

fn parse_picker_date(date: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()
}

fn default_picker_date() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

fn parse_picker_time(time: &str) -> Option<(u32, u32, u32)> {
    let trimmed = time.trim();
    let cleaned = trimmed
        .split(['+', '-', '.', 'Z'])
        .next()
        .unwrap_or(trimmed)
        .trim();
    let parsed = chrono::NaiveTime::parse_from_str(cleaned, "%H:%M:%S")
        .or_else(|_| chrono::NaiveTime::parse_from_str(cleaned, "%H:%M"))
        .ok()?;
    Some((parsed.hour(), parsed.minute(), parsed.second()))
}

fn default_picker_time() -> (u32, u32, u32) {
    let now = chrono::Utc::now().time();
    (now.hour(), now.minute(), now.second())
}

fn shifted_year_month(year: i32, month: u32, delta_months: i32) -> (i32, u32) {
    let month_index = year * 12 + month as i32 - 1 + delta_months;
    (
        month_index.div_euclid(12),
        (month_index.rem_euclid(12) + 1) as u32,
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = shifted_year_month(year, month, 1);
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .unwrap_or(31)
}

fn date_picker_weekday_labels() -> [String; 7] {
    [
        t("grid_weekday_mon"),
        t("grid_weekday_tue"),
        t("grid_weekday_wed"),
        t("grid_weekday_thu"),
        t("grid_weekday_fri"),
        t("grid_weekday_sat"),
        t("grid_weekday_sun"),
    ]
}

fn inline_dark_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = theme::text_secondary();
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let width = (text_width + 20.0).max(46.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
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
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn set_pointing_cursor_on_hover(ui: &mut egui::Ui, response: &egui::Response, enabled: bool) {
    if enabled && response.hovered() {
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn split_datetime_value(value: &str) -> (String, String) {
    let value = value.trim();
    if value.is_empty() {
        return ("".to_string(), "".to_string());
    }

    if let Some((date, time)) = value.split_once(' ') {
        return (date.to_string(), time.to_string());
    }
    if let Some((date, time)) = value.split_once('T') {
        return (date.to_string(), time.trim_end_matches('Z').to_string());
    }
    (value.to_string(), "00:00:00".to_string())
}

fn enter_pressed(ui: &egui::Ui) -> bool {
    ui.input(|i| i.key_pressed(egui::Key::Enter))
}

fn render_cell(ui: &mut egui::Ui, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 18.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(theme::RADIUS_MD),
                theme::with_alpha(theme::text_muted(), 24),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect.shrink(2.0)), |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::NULL_MARKER, "null", 12.0);
            });
            show_dark_hover_tooltip(ui, resp.id.with("tooltip"), &resp, &t("grid_null_value"));
        }
        CellValue::Bool(v) => {
            let (text, color) = if *v {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            value_pill(ui, text, color);
        }
        CellValue::Json(v) => {
            render_copyable_cell(ui, &v.to_string(), theme::ACCENT_TEAL);
        }
        CellValue::Timestamp(v) => {
            render_copyable_cell(ui, v, theme::ACCENT_BLUE);
        }
        CellValue::Uuid(v) => {
            render_copyable_cell(ui, &v.to_string(), theme::ACCENT_COPPER_LIGHT);
        }
        CellValue::Bytes(v) => {
            render_copyable_cell(ui, &format!("\\x{}", hex_encode(v)), theme::text_muted());
        }
        other => {
            let text = other.to_string();
            render_copyable_cell(ui, &text, theme::text_primary());
        }
    }
}

fn render_passive_cell(ui: &mut egui::Ui, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            passive_value_pill(ui, "NULL", theme::text_muted());
        }
        CellValue::Bool(v) => {
            let (text, color) = if *v {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            passive_value_pill(ui, text, color);
        }
        CellValue::Json(v) => {
            render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_TEAL);
        }
        CellValue::Timestamp(v) => {
            render_passive_copyable_cell(ui, v, theme::ACCENT_BLUE);
        }
        CellValue::Uuid(v) => {
            render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_COPPER_LIGHT);
        }
        CellValue::Bytes(v) => {
            render_passive_copyable_cell(ui, &format!("\\x{}", hex_encode(v)), theme::text_muted());
        }
        other => {
            let text = other.to_string();
            render_passive_copyable_cell(ui, &text, theme::text_primary());
        }
    }
}

fn value_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    value_pill_with_interaction(ui, text, color, true);
}

fn passive_value_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    value_pill_with_interaction(ui, text, color, false);
}

fn value_pill_with_interaction(ui: &mut egui::Ui, text: &str, color: Color32, interactive: bool) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), egui::FontId::monospace(11.0), color);
    let sense = if interactive {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(galley.rect.width() + 12.0, 18.0), sense);
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::with_alpha(color, if resp.hovered() { 38 } else { 24 }),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::monospace(11.0),
        color,
    );
    if interactive {
        show_cell_copy_context_menu(&resp, text);
    }
}

fn render_copyable_cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    render_copyable_cell_with_interaction(ui, text, color, true);
}

fn render_passive_copyable_cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    render_copyable_cell_with_interaction(ui, text, color, false);
}

fn render_copyable_cell_with_interaction(
    ui: &mut egui::Ui,
    text: &str,
    color: Color32,
    interactive: bool,
) {
    let font = egui::FontId::monospace(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), color);
    let available_width = ui.available_width().max(1.0);
    let width = galley.rect.width().min(available_width).max(1.0);
    let sense = if interactive {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 24.0), sense);
    let text_rect = rect.shrink2(egui::vec2(0.0, 1.0));
    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
    if galley.rect.width() > text_rect.width() + 1.0 {
        show_dark_hover_tooltip(ui, resp.id.with("full_value"), &resp, text);
    }
    if interactive {
        show_cell_copy_context_menu(&resp, text);
    }
}

fn show_cell_copy_context_menu(response: &egui::Response, text: &str) {
    response.context_menu(|ui| {
        let copy_resp = ui.add(theme::ghost_icon_button(
            crate::ui::icon_image_tinted(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_cell_v",
                10.0,
                theme::ACCENT_BLUE,
            ),
            t("grid_copy_value"),
        ));
        if copy_resp.clicked() {
            ui.ctx().copy_text(text.to_string());
            ui.close_menu();
        }
    });
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn set_sort_clause(
    state: &mut AppState,
    bridge: &DbBridge,
    column: &str,
    direction: DataSortDirection,
) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    if let Some(clause) = state
        .data_edit
        .sort
        .iter_mut()
        .find(|clause| clause.column == column)
    {
        clause.direction = direction;
    } else {
        state.data_edit.sort.push(DataSortClause {
            column: column.to_string(),
            direction,
        });
    }
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

fn remove_sort_clause(state: &mut AppState, bridge: &DbBridge, column: &str) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state
        .data_edit
        .sort
        .retain(|clause| clause.column != column);
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

fn clear_sort_clauses(state: &mut AppState, bridge: &DbBridge) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state.data_edit.sort.clear();
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

fn set_data_page_index(state: &mut AppState, bridge: &DbBridge, page_index: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    state.data_edit.page_index = page_index;
    state.data_edit.page_index_input = (page_index + 1).to_string();
    reload_data_source(state, bridge);
}

fn apply_data_page_input(state: &mut AppState, bridge: &DbBridge) -> bool {
    let raw = state.data_edit.page_index_input.trim().replace(',', "");
    match raw.parse::<usize>() {
        Ok(page) => {
            let page_index = page.max(1) - 1;
            set_data_page_index(state, bridge, page_index);
            true
        }
        Err(_) => {
            state.last_error = Some(t("grid_limit_error"));
            false
        }
    }
}

fn apply_data_limit_input(state: &mut AppState, bridge: &DbBridge) -> bool {
    let raw = state.data_edit.page_limit_input.trim().replace(',', "");
    match raw.parse::<usize>() {
        Ok(limit) => {
            set_data_limit(state, bridge, limit);
            true
        }
        Err(_) => {
            state.last_error = Some(t("grid_limit_error"));
            false
        }
    }
}

fn set_data_limit(state: &mut AppState, bridge: &DbBridge, limit: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    let limit = limit.clamp(1, MAX_DATA_PAGE_LIMIT);
    state.data_edit.page_limit = limit;
    state.data_edit.page_limit_input = limit.to_string();
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

fn reload_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    state.last_error = None;
    state.data_edit.selected_cell = None;
    state.data_edit.editing_cell = None;
    bridge.send(DbCommand::ExecuteQuery {
        conn_id: source.conn_id,
        sql: build_data_select_sql_with_columns(
            &source,
            &state.data_edit.sort,
            limit,
            offset,
            &state.data_columns_for_source(&source),
        ),
        row_limit: Some(limit),
    });
}

pub(super) fn normalized_data_limit(state: &AppState) -> usize {
    state.data_edit.page_limit.clamp(1, MAX_DATA_PAGE_LIMIT)
}

pub(super) fn data_page_offset(state: &AppState) -> usize {
    state
        .data_edit
        .page_index
        .saturating_mul(normalized_data_limit(state))
}

fn has_dirty_data_edits(state: &AppState) -> bool {
    state.data_edit.cells.values().any(|cell| cell.is_dirty())
}

#[derive(Clone, Copy)]
enum EditKind {
    Bool,
    Number,
    Json,
    Date,
    DateTime,
    Uuid,
    Bytes,
    Text,
}

struct DataEditSummary {
    conn_id: crate::types::ConnectionId,
    dirty_count: usize,
    can_apply: bool,
    blocked_reason: Option<String>,
    color: Color32,
}

fn data_edit_summary(state: &AppState) -> Option<DataEditSummary> {
    if state.active_main_view != MainView::Data {
        return None;
    }

    let source = state.active_data_source()?;
    let dirty_count = state
        .data_edit
        .cells
        .values()
        .filter(|cell| cell.is_dirty())
        .count();
    if dirty_count == 0 {
        return None;
    }

    let pk_columns = primary_key_columns(state);
    let invalid_count = count_invalid_edits(state);
    // Plan v7 §10 / Phase 1.2 — PK 컬럼 타입 화이트리스트 가드.
    // 비허용 타입 (json/jsonb, array, range, composite 등) 인 PK 가 하나라도 있으면
    // mutation UI hard-disable + 배너 (silent 데이터 손상 차단).
    let unsupported_pk = pk_columns
        .iter()
        .find(|col| !crate::db::row_key::is_pk_type_allowed(&col.data_type));
    let blocked_reason = if pk_columns.is_empty() {
        Some(t("grid_pk_required"))
    } else if let Some(col) = unsupported_pk {
        Some(format!(
            "PK column '{}' has unsupported type '{}' for safe row identity \
             — editing disabled to prevent data loss.",
            col.name, col.data_type
        ))
    } else if invalid_count > 0 {
        Some(tf("grid_invalid_values", &[&invalid_count.to_string()]))
    } else {
        None
    };
    let can_apply = blocked_reason.is_none() && !state.data_edit.applying;
    let color = if blocked_reason.is_some() {
        theme::ACCENT_YELLOW
    } else {
        theme::ACCENT_COPPER
    };

    Some(DataEditSummary {
        conn_id: source.conn_id,
        dirty_count,
        can_apply,
        blocked_reason,
        color,
    })
}

fn data_column_info<'a>(state: &'a AppState, column_name: &str) -> Option<&'a ColumnInfo> {
    let source = state.active_data_source()?;
    state
        .connections
        .get(&source.conn_id)?
        .columns
        .get(&(source.schema, source.table))?
        .iter()
        .find(|col| col.name == column_name)
}

fn has_table_column_metadata(state: &AppState) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .is_some()
}

fn table_columns(state: &AppState) -> Vec<ColumnInfo> {
    let Some(source) = state.active_data_source() else {
        return Vec::new();
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .cloned()
        .unwrap_or_default()
}

fn primary_key_columns(state: &AppState) -> Vec<ColumnInfo> {
    table_columns(state)
        .into_iter()
        .filter(|col| col.is_primary_key)
        .collect()
}

fn build_data_edits(state: &AppState) -> Result<Vec<crate::db::edits::RowEditOp>, String> {
    use crate::db::edits::{ColumnAssignment, EditValue, PkColumn, RowEditOp};
    let source = state
        .active_data_source()
        .ok_or_else(|| t("grid_no_active_data_source"))?;
    let result = state
        .current_result
        .as_ref()
        .ok_or_else(|| t("grid_no_result_set"))?;
    let table_columns = table_columns(state);
    let pk_columns: Vec<ColumnInfo> = table_columns
        .iter()
        .filter(|col| col.is_primary_key)
        .cloned()
        .collect();
    if pk_columns.is_empty() {
        return Err(t("grid_pk_required"));
    }

    let mut edits = Vec::new();
    for ((row_idx, col_idx), cell) in &state.data_edit.cells {
        if !cell.is_dirty() {
            continue;
        }
        let column = result
            .columns
            .get(*col_idx)
            .ok_or_else(|| t("grid_column_missing"))?;
        let column_info = table_columns
            .iter()
            .find(|info| info.name == column.name)
            .cloned();
        if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
            continue;
        }
        let column_type = column_info
            .as_ref()
            .map(|info| info.data_type.clone())
            .unwrap_or_else(|| column.type_name.clone());
        let nullable = column_info
            .as_ref()
            .map(|info| info.is_nullable)
            .unwrap_or(true);
        let enum_values = column_info
            .as_ref()
            .map(|info| info.enum_values.as_slice())
            .unwrap_or(&[]);
        if let Some(error) = validate_edit_value(cell, &column_type, nullable, enum_values) {
            return Err(error);
        }

        let mut pk = Vec::new();
        for pk_col in &pk_columns {
            let pk_idx = result
                .columns
                .iter()
                .position(|col| col.name == pk_col.name)
                .ok_or_else(|| tf("grid_pk_missing", &[&pk_col.name]))?;
            let original = state
                .data_edit
                .cells
                .get(&(*row_idx, pk_idx))
                .map(|cell| cell.original.clone())
                .or_else(|| {
                    result
                        .rows
                        .get(*row_idx)
                        .and_then(|row| row.get(pk_idx))
                        .cloned()
                })
                .ok_or_else(|| t("grid_pk_value_missing"))?;
            pk.push(PkColumn {
                column: pk_col.name.clone(),
                column_type: pk_col.data_type.clone(),
                value: original,
            });
        }

        let value = if cell.is_null {
            EditValue::Null
        } else if is_timestamptz_type(&column_type) {
            EditValue::Text(
                timestamp_display_to_utc(&cell.value, &state.data_timezone)
                    .unwrap_or_else(|| cell.value.clone()),
            )
        } else if is_timestamp_without_timezone_type(&column_type) {
            EditValue::Text(
                timestamp_display_to_utc_naive(&cell.value, &state.data_timezone)
                    .unwrap_or_else(|| cell.value.clone()),
            )
        } else {
            EditValue::Text(cell.value.clone())
        };

        edits.push(RowEditOp::Update {
            schema: source.schema.clone(),
            table: source.table.clone(),
            column: ColumnAssignment {
                column: column.name.clone(),
                column_type,
                value,
            },
            pk,
        });
    }

    Ok(edits)
}

fn count_invalid_edits(state: &AppState) -> usize {
    let Some(result) = state.current_result.as_ref() else {
        return 0;
    };
    state
        .data_edit
        .cells
        .iter()
        .filter(|((_, col_idx), cell)| {
            if !cell.is_dirty() {
                return false;
            }
            let Some(column) = result.columns.get(*col_idx) else {
                return true;
            };
            let info = data_column_info(state, &column.name);
            let type_name = info
                .map(|info| info.data_type.as_str())
                .unwrap_or(column.type_name.as_str());
            let nullable = info.map(|info| info.is_nullable).unwrap_or(true);
            let enum_values = info.map(|info| info.enum_values.as_slice()).unwrap_or(&[]);
            validate_edit_value(cell, type_name, nullable, enum_values).is_some()
        })
        .count()
}

fn revert_data_edits(state: &mut AppState) {
    let column_types = state
        .current_result
        .as_ref()
        .map(|result| {
            result
                .columns
                .iter()
                .map(|column| column.type_name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for ((_, col_idx), cell) in state.data_edit.cells.iter_mut() {
        let type_name = column_types
            .get(*col_idx)
            .map(String::as_str)
            .unwrap_or_default();
        cell.value = cell_edit_text_for_type(&cell.original, type_name, &state.data_timezone);
        cell.original_text = cell.value.clone();
        cell.is_null = matches!(cell.original, CellValue::Null);
    }
    state.data_edit.editing_cell = None;
}

fn validate_edit_value(
    cell: &crate::state::EditableCell,
    type_name: &str,
    nullable: bool,
    enum_values: &[String],
) -> Option<String> {
    if cell.is_null {
        return (!nullable).then(|| t("grid_not_null"));
    }

    if !enum_values.is_empty() && !enum_values.iter().any(|value| value == cell.value.trim()) {
        return Some(t("grid_enum_error"));
    }

    match edit_kind(type_name, &cell.original) {
        EditKind::Bool => parse_bool(&cell.value)
            .is_none()
            .then(|| t("grid_bool_error")),
        EditKind::Number => cell
            .value
            .trim()
            .parse::<f64>()
            .is_err()
            .then(|| t("grid_number_error")),
        EditKind::Json => serde_json::from_str::<serde_json::Value>(&cell.value)
            .is_err()
            .then(|| t("grid_json_error")),
        EditKind::Date => (!is_valid_date(&cell.value)).then(|| t("grid_date_error")),
        EditKind::DateTime => (!is_valid_datetime(&cell.value)).then(|| t("grid_datetime_error")),
        EditKind::Uuid => uuid::Uuid::parse_str(cell.value.trim())
            .is_err()
            .then(|| t("grid_uuid_error")),
        EditKind::Bytes => {
            let value = cell
                .value
                .trim()
                .strip_prefix("\\x")
                .unwrap_or(cell.value.trim());
            (!value.chars().all(|ch| ch.is_ascii_hexdigit()) || !value.len().is_multiple_of(2))
                .then(|| t("grid_bytes_error"))
        }
        EditKind::Text => None,
    }
}

fn edit_kind(type_name: &str, cell: &CellValue) -> EditKind {
    let lower = type_name.to_ascii_lowercase();
    if matches!(cell, CellValue::Bool(_)) || matches!(lower.as_str(), "bool" | "boolean") {
        EditKind::Bool
    } else if matches!(cell, CellValue::Int(_) | CellValue::Float(_))
        || matches!(
            lower.as_str(),
            "smallint"
                | "integer"
                | "bigint"
                | "int2"
                | "int4"
                | "int8"
                | "real"
                | "double precision"
                | "float4"
                | "float8"
                | "numeric"
                | "decimal"
        )
    {
        EditKind::Number
    } else if matches!(cell, CellValue::Json(_)) || matches!(lower.as_str(), "json" | "jsonb") {
        EditKind::Json
    } else if lower == "date" {
        EditKind::Date
    } else if matches!(cell, CellValue::Timestamp(_))
        || matches!(
            lower.as_str(),
            "timestamp"
                | "timestamptz"
                | "timestamp without time zone"
                | "timestamp with time zone"
        )
    {
        EditKind::DateTime
    } else if matches!(cell, CellValue::Uuid(_)) || lower == "uuid" {
        EditKind::Uuid
    } else if matches!(cell, CellValue::Bytes(_)) || lower == "bytea" {
        EditKind::Bytes
    } else {
        EditKind::Text
    }
}

fn is_valid_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").is_ok()
}

fn is_valid_datetime(value: &str) -> bool {
    let value = value.trim();
    if chrono::DateTime::parse_from_rfc3339(value).is_ok() {
        return true;
    }

    [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
    ]
    .iter()
    .any(|format| chrono::NaiveDateTime::parse_from_str(value, format).is_ok())
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------

fn result_to_tsv(result: &crate::types::QueryResult) -> String {
    let mut out = String::new();
    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    out.push_str(&headers.join("\t"));
    out.push('\n');
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}

fn export_csv(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let task = rfd::AsyncFileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("query_result.csv")
        .save_file();

    let columns = result.columns.clone();
    let rows = result.rows.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let mut wtr = csv::Writer::from_writer(Vec::new());
                let headers: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
                let _ = wtr.write_record(&headers);
                for row in &rows {
                    let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                    let _ = wtr.write_record(&cells);
                }
                if let Ok(data) = wtr.into_inner() {
                    let _ = handle.write(&data).await;
                }
            }
        });
    });
}
