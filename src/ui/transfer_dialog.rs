use eframe::egui::{self, RichText};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::transfer::{IfExists, TransferTableStatus};
use crate::state::AppState;
use crate::types::ConnectionId;
use crate::ui::theme;

pub fn render_transfer_dialog(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.transfer.show {
        return;
    }

    let mut close = false;
    let mut start_transfer = false;

    egui::Window::new(t("transfer_title"))
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .default_width(560.0)
        .show(ctx, |ui| {
            ui.set_min_width(480.0);
            ui.set_max_width(700.0);

            render_connection_pair(ui, state);
            ui.add_space(theme::SPACE_MD);
            ui.separator();
            ui.add_space(theme::SPACE_SM);

            render_table_list(ui, state);
            ui.add_space(theme::SPACE_MD);
            ui.separator();
            ui.add_space(theme::SPACE_SM);

            render_options(ui, state);
            ui.add_space(theme::SPACE_MD);

            if let Some(progress) = &state.transfer.progress {
                render_progress(ui, progress);
                ui.add_space(theme::SPACE_MD);
            }

            if let Some(result) = &state.transfer.result {
                render_result(ui, result);
                ui.add_space(theme::SPACE_MD);
            }

            if let Some(err) = &state.transfer.error {
                ui.label(
                    RichText::new(err.as_str())
                        .color(theme::ACCENT_RED)
                        .size(11.0),
                );
                ui.add_space(theme::SPACE_SM);
            }

            ui.horizontal(|ui| {
                let can_transfer = state.transfer.source_conn.is_some()
                    && state.transfer.target_conn.is_some()
                    && state.transfer.tables.iter().any(|t| t.selected)
                    && !state.transfer.is_transferring()
                    && state.transfer.result.is_none();

                if ui
                    .add_enabled(can_transfer, egui::Button::new(t("transfer_start")))
                    .clicked()
                {
                    start_transfer = true;
                }

                if ui.button(t("transfer_cancel")).clicked() {
                    close = true;
                }
            });
        });

    if start_transfer {
        if let (Some(src_id), Some(tgt_id)) =
            (state.transfer.source_conn, state.transfer.target_conn)
        {
            let src_config = state
                .connections
                .get(&src_id)
                .map(|c| c.config.clone());
            let tgt_config = state
                .connections
                .get(&tgt_id)
                .map(|c| c.config.clone());

            if let (Some(src_config), Some(tgt_config)) = (src_config, tgt_config) {
                let tables: Vec<String> = state
                    .transfer
                    .tables
                    .iter()
                    .filter(|t| t.selected)
                    .map(|t| t.name.clone())
                    .collect();

                let request = crate::state::transfer::TransferRequest {
                    source_config: src_config,
                    target_config: tgt_config,
                    source_schema: state.transfer.source_schema.clone(),
                    target_schema: state.transfer.target_schema.clone(),
                    tables,
                    options: state.transfer.options.clone(),
                };

                bridge.send(crate::db::bridge::DbCommand::TransferTables { request });

                state.transfer.progress = Some(
                    crate::state::transfer::TransferProgress {
                        current_table: String::new(),
                        current_table_index: 0,
                        total_tables: state.transfer.tables.len(),
                        rows_transferred: 0,
                        rows_total: None,
                        bytes_transferred: 0,
                    },
                );
            }
        }
    }

    if close {
        state.transfer.reset();
    }
}

fn render_connection_pair(ui: &mut egui::Ui, state: &AppState) {
    let source_name = connection_display_name(state, state.transfer.source_conn);
    let target_name = connection_display_name(state, state.transfer.target_conn);

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(t("transfer_source"))
                .color(theme::text_muted())
                .size(11.0),
        );
        ui.label(
            RichText::new(format!("{} / {}", source_name, state.transfer.source_schema))
                .color(theme::accent_color())
                .size(12.0)
                .strong(),
        );
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(t("transfer_target"))
                .color(theme::text_muted())
                .size(11.0),
        );
        ui.label(
            RichText::new(format!("{} / {}", target_name, state.transfer.target_schema))
                .color(theme::ACCENT_BLUE)
                .size(12.0)
                .strong(),
        );
    });
}

fn render_table_list(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new(t("transfer_tables_header"))
            .color(theme::text_primary())
            .strong()
            .size(12.0),
    );
    ui.add_space(theme::SPACE_XS);

    let selected_count = state.transfer.tables.iter().filter(|t| t.selected).count();
    let total_count = state.transfer.tables.len();

    ui.horizontal(|ui| {
        if ui
            .small_button(if selected_count == total_count {
                t("transfer_deselect_all")
            } else {
                t("transfer_select_all")
            })
            .clicked()
        {
            let new_val = selected_count != total_count;
            for entry in &mut state.transfer.tables {
                entry.selected = new_val;
            }
        }
        ui.label(
            RichText::new(format!("{selected_count}/{total_count}"))
                .color(theme::text_muted())
                .size(11.0),
        );
    });

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for entry in &mut state.transfer.tables {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut entry.selected, "");

                    let status_color = match entry.status {
                        TransferTableStatus::Done => theme::ACCENT_GREEN,
                        TransferTableStatus::Error => theme::ACCENT_RED,
                        TransferTableStatus::InProgress => theme::ACCENT_YELLOW,
                        TransferTableStatus::Skipped => theme::text_disabled(),
                        TransferTableStatus::Pending => theme::text_primary(),
                    };
                    ui.label(
                        RichText::new(&entry.name)
                            .color(status_color)
                            .size(12.0),
                    );

                    if let Some(count) = entry.row_count {
                        ui.label(
                            RichText::new(format_row_count(count))
                                .color(theme::text_muted())
                                .size(10.0),
                        );
                    }

                    if !entry.dependencies.is_empty() {
                        ui.label(
                            RichText::new(format!("→ {}", entry.dependencies.join(", ")))
                                .color(theme::text_disabled())
                                .size(10.0),
                        );
                    }

                    if let Some(err) = &entry.error {
                        ui.label(
                            RichText::new(err.as_str())
                                .color(theme::ACCENT_RED)
                                .size(10.0),
                        );
                    }
                });
            }
        });
}

fn render_options(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.transfer.options.include_data, t("transfer_include_data"));
        ui.add_space(theme::SPACE_MD);
        ui.label(
            RichText::new(t("transfer_if_exists"))
                .color(theme::text_muted())
                .size(11.0),
        );
        egui::ComboBox::from_id_salt("transfer_if_exists_combo")
            .selected_text(if_exists_label(state.transfer.options.if_exists))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.transfer.options.if_exists,
                    IfExists::Drop,
                    if_exists_label(IfExists::Drop),
                );
                ui.selectable_value(
                    &mut state.transfer.options.if_exists,
                    IfExists::Skip,
                    if_exists_label(IfExists::Skip),
                );
                ui.selectable_value(
                    &mut state.transfer.options.if_exists,
                    IfExists::Truncate,
                    if_exists_label(IfExists::Truncate),
                );
            });
    });
}

fn render_progress(
    ui: &mut egui::Ui,
    progress: &crate::state::transfer::TransferProgress,
) {
    ui.label(
        RichText::new(format!(
            "{} ({}/{})",
            progress.current_table,
            progress.current_table_index + 1,
            progress.total_tables
        ))
        .color(theme::ACCENT_YELLOW)
        .size(12.0),
    );

    let fraction = if let Some(total) = progress.rows_total {
        if total > 0 {
            progress.rows_transferred as f32 / total as f32
        } else {
            0.0
        }
    } else {
        0.0
    };

    let bar = egui::ProgressBar::new(fraction)
        .text(format!("{} rows", progress.rows_transferred));
    ui.add(bar);
}

fn render_result(ui: &mut egui::Ui, result: &crate::state::transfer::TransferResult) {
    let color = if result.tables_failed == 0 {
        theme::ACCENT_GREEN
    } else {
        theme::ACCENT_YELLOW
    };

    ui.label(
        RichText::new(format!(
            "Done: {} succeeded, {} failed, {} skipped ({} rows, {:.1}s)",
            result.tables_success,
            result.tables_failed,
            result.tables_skipped,
            result.total_rows,
            result.duration_ms as f64 / 1000.0,
        ))
        .color(color)
        .size(12.0),
    );

    if !result.errors.is_empty() {
        egui::ScrollArea::vertical()
            .max_height(100.0)
            .show(ui, |ui| {
                for (table, err) in &result.errors {
                    ui.label(
                        RichText::new(format!("{table}: {err}"))
                            .color(theme::ACCENT_RED)
                            .size(10.0),
                    );
                }
            });
    }
}

fn connection_display_name(state: &AppState, conn_id: Option<ConnectionId>) -> String {
    let Some(id) = conn_id else {
        return "—".to_string();
    };
    state
        .connections
        .get(&id)
        .map(|c| {
            if c.config.display_name.is_empty() {
                format!("{}@{}", c.config.username, c.config.host)
            } else {
                c.config.display_name.clone()
            }
        })
        .unwrap_or_else(|| "—".to_string())
}

fn if_exists_label(mode: IfExists) -> &'static str {
    match mode {
        IfExists::Drop => "Drop & recreate",
        IfExists::Skip => "Skip existing",
        IfExists::Truncate => "Truncate first",
    }
}

fn format_row_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M rows", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K rows", count as f64 / 1_000.0)
    } else {
        format!("{count} rows")
    }
}
