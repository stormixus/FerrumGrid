//! Backup tools view.
//!
//! Plan v7 Phase 1.95b3d cut-over (from `super::mod.rs`). Phase 4a 에서
//! `BackupInfoV1` schema 통합 (status / progress / eta) + DiagnosticsPanel
//! 합류 진행.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::AppState;
use crate::storage::settings::AppSettings;
use crate::types::{BackupFormat, BackupRecord, BackupRequest, ConnectionConfig, ConnectionId};
use crate::ui::{icons_svg, theme};

use super::{
    active_conn, render_no_connection, show_dark_hover_tooltip, type_chip, ObjectAction,
};

pub(super) fn render_backup_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
) -> Option<ObjectAction> {
    let Some(conn_id) = active_conn(state) else {
        return render_no_connection(ui);
    };
    let cfg = state.connections.get(&conn_id)?.config.clone();
    let schema =
        (!state.objects_schema_filter.is_empty()).then(|| state.objects_schema_filter.clone());

    ui.add_space(theme::SPACE_XL);
    render_backup_scope_card(ui, &cfg, schema.as_deref());
    ui.add_space(theme::SPACE_LG);
    render_backup_repository_card(
        ui,
        state,
        bridge,
        settings,
        conn_id,
        &cfg,
        schema.as_deref(),
    );
    ui.add_space(theme::SPACE_LG);
    render_backup_history(ui, state);
    None
}

fn render_backup_scope_card(ui: &mut egui::Ui, cfg: &ConnectionConfig, schema: Option<&str>) {
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::BACKUP, "backup_scope", 24.0);
                ui.add_space(theme::SPACE_SM);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(match schema {
                            Some(_) => t("backup_schema"),
                            None => t("backup_full_database"),
                        })
                        .color(theme::text_primary())
                        .size(15.0)
                        .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}  {}:{} / {}",
                            cfg.display_name, cfg.host, cfg.port, cfg.database
                        ))
                        .color(theme::text_muted())
                        .size(11.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(schema) = schema {
                        type_chip(ui, schema, theme::ACCENT_TEAL);
                    } else {
                        type_chip(ui, "FULL", theme::ACCENT_COPPER);
                    }
                });
            });
        });
}

fn render_backup_repository_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
    conn_id: ConnectionId,
    cfg: &ConnectionConfig,
    schema: Option<&str>,
) {
    let folder_set = !settings.backup_directory.trim().is_empty();
    let folder_label = (if folder_set {
        settings.backup_directory.clone()
    } else {
        t("backup_no_folder_selected")
    })
    .to_string();

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t("backup_folder_title"))
                            .color(theme::text_primary())
                            .size(15.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(folder_label)
                            .color(if folder_set {
                                theme::text_secondary()
                            } else {
                                theme::text_muted()
                            })
                            .size(11.0),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(theme::secondary_button(&t("backup_choose_folder")))
                        .clicked()
                    {
                        let mut dialog = rfd::FileDialog::new();
                        if folder_set {
                            dialog = dialog.set_directory(&settings.backup_directory);
                        }
                        if let Some(path) = dialog.pick_folder() {
                            settings.backup_directory = path.display().to_string();
                            crate::storage::settings::save_settings(settings);
                            state.status_message = t("backup_folder_updated");
                        }
                    }

                    if ui
                        .add_enabled(folder_set, theme::ghost_button(&t("backup_open_folder")))
                        .clicked()
                    {
                        open_backup_folder(&settings.backup_directory);
                    }
                });
            });

            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(t("backup_format"))
                        .color(theme::text_muted())
                        .size(11.0)
                        .strong(),
                );
                backup_format_button(ui, &mut state.backup_format, BackupFormat::Custom);
                backup_format_button(ui, &mut state.backup_format, BackupFormat::Plain);
                backup_format_button(ui, &mut state.backup_format, BackupFormat::Tar);
            });

            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                let can_run = folder_set && !state.backup_running;
                let run_label = if state.backup_running {
                    t("backup_running_label")
                } else {
                    t("backup_run")
                };
                let run_button = if can_run {
                    theme::primary_button(&run_label)
                } else {
                    theme::secondary_button(&run_label)
                };
                if ui.add_enabled(can_run, run_button).clicked() {
                    let request = BackupRequest {
                        conn_id,
                        config: cfg.clone(),
                        output_dir: std::path::PathBuf::from(&settings.backup_directory),
                        schema: schema.map(ToOwned::to_owned),
                        format: state.backup_format,
                    };
                    state.backup_running = true;
                    state.backup_last_error = None;
                    state.status_message = tf("backup_running_status", &[&cfg.display_name]);
                    bridge.send(DbCommand::RunBackup { request });
                }

                if state.backup_running {
                    ui.spinner();
                    ui.label(
                        RichText::new(t("backup_pg_dump_running"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                }
            });

            if let Some(error) = &state.backup_last_error {
                ui.add_space(theme::SPACE_MD);
                ui.label(RichText::new(error).color(theme::ACCENT_RED).size(11.0));
            }
        });
}

fn backup_format_button(ui: &mut egui::Ui, value: &mut BackupFormat, format: BackupFormat) {
    let selected = *value == format;
    let label = match format {
        BackupFormat::Custom => t("backup_custom_archive"),
        BackupFormat::Plain => t("backup_plain_sql"),
        BackupFormat::Tar => "Tar archive".to_string(),
    };
    let button = egui::Button::new(RichText::new(label).color(if selected {
        theme::text_primary()
    } else {
        theme::text_secondary()
    }))
    .fill(if selected {
        theme::bg_darkest()
    } else {
        theme::bg_medium()
    })
    .stroke(Stroke::new(
        1.0,
        if selected {
            theme::ACCENT_EMERALD
        } else {
            theme::border_default()
        },
    ))
    .corner_radius(CornerRadius::same(theme::RADIUS_MD));

    if ui.add(button).clicked() {
        *value = format;
    }
}

fn render_backup_history(ui: &mut egui::Ui, state: &AppState) {
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(t("backup_recent"))
                    .color(theme::text_primary())
                    .size(15.0)
                    .strong(),
            );
            ui.add_space(theme::SPACE_MD);

            if state.backup_history.is_empty() {
                ui.label(
                    RichText::new(t("backup_no_session"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                return;
            }

            for record in &state.backup_history {
                render_backup_record(ui, record);
            }
        });
}

fn render_backup_record(ui: &mut egui::Ui, record: &BackupRecord) {
    let response = egui::Frame::new()
        .fill(theme::bg_dark())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::BACKUP, "backup_record", 15.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(
                            record
                                .file_path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("backup"),
                        )
                        .color(theme::text_primary())
                        .size(12.0)
                        .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} / {} / {} / {}",
                            record.connection_name,
                            record.database,
                            record.schema.as_deref().unwrap_or("full"),
                            record.format.label()
                        ))
                        .color(theme::text_muted())
                        .size(10.5),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} ms", record.duration_ms))
                            .color(theme::text_muted())
                            .size(10.5),
                    );
                    ui.label(
                        RichText::new(format_size(record.size_bytes))
                            .color(theme::text_secondary())
                            .size(10.5),
                    );
                    ui.label(
                        RichText::new(&record.completed_at)
                            .color(theme::text_muted())
                            .size(10.5),
                    );
                });
            });
        })
        .response;
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &format!("Connection ID: {}", record.conn_id),
    );
    ui.add_space(theme::SPACE_SM);
}

fn open_backup_folder(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = bytes as f64;

    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}
