//! BI (Business Intelligence) cards view.
//!
//! Plan v7 Phase 1.95b2 cut-over (from `super::mod.rs`). Phase 4c 에서
//! 그룹/피벗/차트 카드 + `QueryResult` 캐시 모델 변경 예정.

use eframe::egui::{self, CornerRadius, RichText, ScrollArea};

use crate::bi::aggregate::{compute_column_stats, group_by, AggregateOp};
use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{build_data_select_sql_with_columns, AppState};
use crate::ui::theme;

use super::{
    active_conn, cell_label, data_row, empty_state, format_number, render_count_strip,
    selected_schema_or_public, table_header, type_chip, ObjectAction, BI_COLUMNS,
};

pub(super) fn render_bi_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    ui.add_space(theme::SPACE_LG);
    let Some(result) = state.current_result.as_ref() else {
        return render_bi_empty(ui, state, bridge);
    };

    render_count_strip(
        ui,
        result.rows.len(),
        &format!("rows, {} columns", result.columns.len()),
    );
    ui.add_space(theme::SPACE_MD);

    ScrollArea::vertical().id_salt("bi_summary").show(ui, |ui| {
        // Column stats table
        table_header(
            ui,
            &BI_COLUMNS,
            &[
                t("objects_column"),
                t("objects_type"),
                t("objects_non_null"),
                t("objects_min"),
                t("objects_max"),
                t("objects_average"),
            ],
        );
        for idx in 0..result.columns.len() {
            let Some(stats) = compute_column_stats(result, idx) else {
                continue;
            };
            let min = stats.min.map(format_number).unwrap_or_else(|| "-".to_string());
            let max = stats.max.map(format_number).unwrap_or_else(|| "-".to_string());
            let avg = stats.avg.map(format_number).unwrap_or_else(|| "-".to_string());

            data_row(ui, &BI_COLUMNS, |row| {
                row.col(|ui| cell_label(ui, &stats.name, theme::text_primary(), 12.0, false));
                row.col(|ui| type_chip(ui, &stats.type_name, theme::ACCENT_BLUE));
                row.col(|ui| {
                    cell_label(ui, &stats.non_null.to_string(), theme::text_secondary(), 12.0, false)
                });
                row.col(|ui| cell_label(ui, &min, theme::text_muted(), 12.0, false));
                row.col(|ui| cell_label(ui, &max, theme::text_muted(), 12.0, false));
                row.col(|ui| cell_label(ui, &avg, theme::text_muted(), 12.0, false));
            });
        }

        // Group Analysis section
        ui.add_space(theme::SPACE_XXL);
        render_group_analysis_section(ui, result);
    });

    None
}

fn render_bi_empty(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = active_conn(state);
    let schema = selected_schema_or_public(state);
    let table = if !state.objects_search.is_empty() {
        state.objects_search.clone()
    } else {
        String::new()
    };

    if conn_id.is_some() && !table.is_empty() {
        ui.add_space(theme::SPACE_XXL);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(format!("Analyze: {schema}.{table}"))
                    .color(theme::text_primary())
                    .size(14.0)
                    .strong(),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new("Load table data to see column statistics and group analysis.")
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_LG);
            if ui.add(theme::primary_button("Load Data")).clicked() {
                let conn_id = conn_id.unwrap();
                let source = crate::state::DataSource {
                    conn_id,
                    schema: schema.clone(),
                    table: table.clone(),
                    filter: None,
                };
                let columns = state.data_columns_for_source(&source);
                state.query_running = true;
                bridge.send(DbCommand::ExecuteQuery {
                    conn_id,
                    sql: build_data_select_sql_with_columns(
                        &source,
                        &[],
                        1000,
                        0,
                        &columns,
                    ),
                    row_limit: Some(1000),
                });
            }
        });
        if state.query_running {
            ui.add_space(theme::SPACE_MD);
            ui.vertical_centered(|ui| {
                ui.spinner();
            });
        }
    } else {
        empty_state(
            ui,
            "No table selected",
            "Select a table from the navigator, then switch to BI to analyze its data.",
        );
    }
    None
}

fn render_group_analysis_section(ui: &mut egui::Ui, result: &crate::types::QueryResult) {
    // Section header
    ui.horizontal(|ui| {
        ui.add_space(theme::SPACE_LG);
        ui.label(
            RichText::new("Group Analysis")
                .color(theme::text_primary())
                .size(13.0)
                .strong(),
        );
    });
    ui.add_space(theme::SPACE_MD);

    // Retrieve persisted state from egui memory
    let mut group_col = ui.data_mut(|d| d.get_persisted(egui::Id::new("bi_group_col")).unwrap_or(0));
    let mut agg_col = ui.data_mut(|d| d.get_persisted(egui::Id::new("bi_agg_col")).unwrap_or(0));
    let mut agg_op = ui.data_mut(|d| d.get_persisted(egui::Id::new("bi_agg_op")).unwrap_or(0));

    // Clamp to valid column indices
    group_col = group_col.min(result.columns.len().saturating_sub(1));
    agg_col = agg_col.min(result.columns.len().saturating_sub(1));

    // Column selectors and op selector
    ui.horizontal(|ui| {
        ui.add_space(theme::SPACE_LG);

        // Group by column selector
        ui.label(RichText::new("Group by:").color(theme::text_secondary()).size(12.0));
        ui.add_space(theme::SPACE_SM);

        let mut changed = false;
        egui::ComboBox::from_id_salt("bi_group_col_selector")
            .selected_text(&result.columns[group_col].name)
            .width(150.0)
            .show_ui(ui, |ui| {
                for (idx, col) in result.columns.iter().enumerate() {
                    if ui.selectable_value(&mut group_col, idx, &col.name).changed() {
                        changed = true;
                    }
                }
            });

        ui.add_space(theme::SPACE_LG);

        // Aggregate column selector
        ui.label(RichText::new("Aggregate:").color(theme::text_secondary()).size(12.0));
        ui.add_space(theme::SPACE_SM);

        egui::ComboBox::from_id_salt("bi_agg_col_selector")
            .selected_text(&result.columns[agg_col].name)
            .width(150.0)
            .show_ui(ui, |ui| {
                for (idx, col) in result.columns.iter().enumerate() {
                    if ui.selectable_value(&mut agg_col, idx, &col.name).changed() {
                        changed = true;
                    }
                }
            });

        ui.add_space(theme::SPACE_LG);

        // Op selector
        ui.label(RichText::new("Op:").color(theme::text_secondary()).size(12.0));
        ui.add_space(theme::SPACE_SM);

        let op_names = ["Count", "Sum", "Avg", "Min", "Max"];
        egui::ComboBox::from_id_salt("bi_agg_op_selector")
            .selected_text(op_names[agg_op])
            .width(100.0)
            .show_ui(ui, |ui| {
                for (idx, name) in op_names.iter().enumerate() {
                    if ui.selectable_value(&mut agg_op, idx, *name).changed() {
                        changed = true;
                    }
                }
            });

        if changed {
            ui.data_mut(|d| d.insert_persisted(egui::Id::new("bi_group_col"), group_col));
            ui.data_mut(|d| d.insert_persisted(egui::Id::new("bi_agg_col"), agg_col));
            ui.data_mut(|d| d.insert_persisted(egui::Id::new("bi_agg_op"), agg_op));
        }
    });

    ui.add_space(theme::SPACE_LG);

    // Compute group-by results
    let op = match agg_op {
        0 => AggregateOp::Count,
        1 => AggregateOp::Sum,
        2 => AggregateOp::Avg,
        3 => AggregateOp::Min,
        4 => AggregateOp::Max,
        _ => AggregateOp::Count,
    };

    let groups = group_by(result, group_col, agg_col, op);

    if groups.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(theme::SPACE_LG);
            ui.label(
                RichText::new("No data to display")
                    .color(theme::text_muted())
                    .size(11.0),
            );
        });
    } else {
        render_bar_chart(ui, &groups);
    }
}

fn render_bar_chart(ui: &mut egui::Ui, groups: &[(String, f64)]) {
    const MAX_BARS: usize = 20;

    // Find max value for scaling
    let max_value = groups
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0_f64, f64::max)
        .max(1.0);

    // Truncate to max bars
    let display_groups = if groups.len() > MAX_BARS {
        &groups[..MAX_BARS]
    } else {
        groups
    };

    // Render each bar
    for (key, value) in display_groups {
        ui.horizontal(|ui| {
            ui.add_space(theme::SPACE_LG);

            // Group key label (left side)
            let key_label = if key.len() > 24 {
                format!("{}...", &key[..21])
            } else {
                key.clone()
            };
            ui.add_sized(
                egui::vec2(180.0, 24.0),
                egui::Label::new(
                    RichText::new(key_label)
                        .color(theme::text_primary())
                        .size(11.0),
                ),
            );

            ui.add_space(theme::SPACE_SM);

            // Bar chart area
            let available_width = ui.available_width() - 80.0; // Reserve space for value label
            let bar_width = ((value / max_value * available_width as f64).max(2.0)) as f32;

            let (bar_rect, _) = ui.allocate_exact_size(
                egui::vec2(bar_width, 20.0),
                egui::Sense::hover(),
            );

            ui.painter().rect_filled(
                bar_rect,
                CornerRadius::same(theme::RADIUS_SM),
                theme::ACCENT_BLUE,
            );

            // Value label (right side)
            ui.add_space(theme::SPACE_SM);
            ui.label(
                RichText::new(format_number(*value))
                    .color(theme::text_secondary())
                    .size(11.0),
            );
        });

        ui.add_space(theme::SPACE_SM);
    }

    // Show "and N more" if truncated
    if groups.len() > MAX_BARS {
        ui.horizontal(|ui| {
            ui.add_space(theme::SPACE_LG);
            ui.label(
                RichText::new(format!("... and {} more", groups.len() - MAX_BARS))
                    .color(theme::text_muted())
                    .size(10.0)
                    .italics(),
            );
        });
    }
}
