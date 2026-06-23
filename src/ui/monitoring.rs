//! Slow-query monitoring dashboard.
//!
//! Reads query history from `AppState::query_history` and shows a sortable table
//! of the slowest queries above a configurable threshold (default 500 ms).
//!
//! The window is opened from the main menu (Diagnostics → Slow Queries).

use std::cmp::Reverse;

use eframe::egui::{self, Margin, RichText, Stroke};

use crate::i18n::t;
use crate::state::AppState;
use crate::storage::history::HistoryEntry;
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    #[default]
    Duration,
    Timestamp,
    Rows,
}

/// Default slow-query threshold (ms) — used by the Clear button to reset.
const DEFAULT_THRESHOLD_MS: u64 = 500;

pub fn render_monitoring_window(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_monitoring_window {
        return;
    }

    let mut open = true;
    egui::Window::new(t("monitoring_title"))
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(720.0)
        .default_height(420.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_LG as i8)),
        )
        .show(ctx, |ui| {
            // Threshold slider
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(t("monitoring_threshold"))
                        .color(theme::text_secondary())
                        .size(12.0),
                );
                let mut threshold_ms = state.diag_slow_query_ms;
                if ui
                    .add(
                        egui::DragValue::new(&mut threshold_ms)
                            .range(50..=10_000)
                            .suffix(" ms"),
                    )
                    .changed()
                {
                    state.diag_slow_query_ms = threshold_ms;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t("button_clear")).clicked() {
                        state.diag_slow_query_ms = DEFAULT_THRESHOLD_MS;
                    }
                });
            });
            ui.add_space(theme::SPACE_SM);

            // Render header row — sort state persists in AppState across frames.
            egui::Frame::new()
                .fill(theme::bg_light())
                .inner_margin(Margin::symmetric(
                    theme::SPACE_SM as i8,
                    theme::SPACE_XS as i8,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                state.monitoring_sort == SortColumn::Timestamp,
                                format!("{} ↕", t("monitoring_time")),
                            )
                            .clicked()
                        {
                            state.monitoring_sort = SortColumn::Timestamp;
                        }
                        if ui
                            .selectable_label(
                                state.monitoring_sort == SortColumn::Duration,
                                format!("{} ↕", t("monitoring_duration")),
                            )
                            .clicked()
                        {
                            state.monitoring_sort = SortColumn::Duration;
                        }
                        if ui
                            .selectable_label(
                                state.monitoring_sort == SortColumn::Rows,
                                format!("{} ↕", t("monitoring_rows")),
                            )
                            .clicked()
                        {
                            state.monitoring_sort = SortColumn::Rows;
                        }
                    });
                });

            // Collect and sort entries
            let mut entries: Vec<&HistoryEntry> = state
                .query_history
                .iter()
                .filter(|e| e.duration_ms >= state.diag_slow_query_ms as u128)
                .collect();
            match state.monitoring_sort {
                SortColumn::Timestamp => entries.sort_by_key(|e| Reverse(e.timestamp)),
                SortColumn::Duration => entries.sort_by_key(|e| Reverse(e.duration_ms)),
                SortColumn::Rows => entries.sort_by_key(|e| Reverse(e.row_count)),
            }

            egui::ScrollArea::vertical()
                .id_salt("monitoring_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if entries.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                RichText::new(t("monitoring_empty"))
                                    .color(theme::text_muted())
                                    .size(12.0),
                            );
                        });
                        return;
                    }
                    for entry in entries {
                        let time = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                        let preview: String = entry
                            .query
                            .chars()
                            .take(200)
                            .collect::<String>()
                            .replace('\n', " ");
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(time)
                                    .color(theme::text_disabled())
                                    .size(10.5)
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("{} ms", entry.duration_ms))
                                    .color(theme::ACCENT_RED)
                                    .strong()
                                    .size(11.0)
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("{}", entry.row_count))
                                    .color(theme::text_muted())
                                    .size(11.0)
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(preview)
                                    .color(theme::text_secondary())
                                    .size(11.0)
                                    .monospace(),
                            );
                        });
                        ui.add_space(2.0);
                    }
                });
        });

    if !open {
        state.show_monitoring_window = false;
    }
}
