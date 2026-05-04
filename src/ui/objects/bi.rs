//! BI (Business Intelligence) cards view.
//!
//! Plan v7 Phase 1.95b2 cut-over (from `super::mod.rs`). Phase 4c 에서
//! 그룹/피벗/차트 카드 + `QueryResult` 캐시 모델 변경 예정.

use eframe::egui::{self, ScrollArea};

use crate::bi::aggregate::compute_column_stats;
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

use super::{
    cell_label, data_row, empty_state, format_number, render_count_strip, table_header, type_chip,
    ObjectAction, BI_COLUMNS,
};

pub(super) fn render_bi_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    ui.add_space(theme::SPACE_LG);
    let Some(result) = state.current_result.as_ref() else {
        empty_state(
            ui,
            "No result set",
            "Run a query first, then BI will summarize rows and numeric columns here.",
        );
        return None;
    };

    render_count_strip(
        ui,
        result.rows.len(),
        &format!("rows, {} columns", result.columns.len()),
    );
    ui.add_space(theme::SPACE_MD);

    ScrollArea::vertical().id_salt("bi_summary").show(ui, |ui| {
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
    });

    None
}
