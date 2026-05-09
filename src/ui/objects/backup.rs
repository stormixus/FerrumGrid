//! Backup tools view.
//!
//! Plan v7 Phase 1.95b3d cut-over (from `super::mod.rs`). Phase 4a 에서
//! `BackupInfoV1` schema 통합 (status / progress / eta) + DiagnosticsPanel
//! 합류 진행.

use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use eframe::egui::{self, CornerRadius, Margin, RichText, ScrollArea, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::AppState;
use crate::storage::settings::AppSettings;
use crate::types::{BackupFormat, BackupRecord, BackupRequest, ConnectionConfig, ConnectionId};
use crate::ui::{icons_svg, theme};

use super::{
    active_conn, render_no_connection, show_dark_hover_tooltip, type_chip, ObjectAction,
};

// ---------------------------------------------------------------------------
// Backup file browser types
// ---------------------------------------------------------------------------

const BACKUP_EXTENSIONS: &[&str] = &["sql", "dump", "tar", "gz", "backup", "bak"];
const MAX_DISPLAY_FILES: usize = 100;

#[derive(Clone, Debug)]
struct BackupFileEntry {
    name: String,
    path: PathBuf,
    size_bytes: u64,
    created: Option<String>,
    modified: Option<String>,
    /// Raw modified timestamp for sorting (epoch seconds).
    modified_epoch: i64,
}

/// Cached scan result stored in egui temp memory.
#[derive(Clone, Debug)]
struct BackupFilesCache {
    directory: String,
    entries: Vec<BackupFileEntry>,
}

fn format_system_time(st: SystemTime) -> String {
    let dt: DateTime<Local> = st.into();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn modified_epoch(st: SystemTime) -> i64 {
    let dt: DateTime<Local> = st.into();
    dt.timestamp()
}

fn scan_backup_directory(dir: &str) -> Vec<BackupFileEntry> {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut entries: Vec<BackupFileEntry> = read_dir
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?.to_ascii_lowercase();
            if !BACKUP_EXTENSIONS.contains(&ext.as_str()) {
                return None;
            }
            let meta = std::fs::metadata(&path).ok()?;
            let created = meta.created().ok().map(format_system_time);
            let modified = meta.modified().ok().map(format_system_time);
            let mod_epoch = meta.modified().ok().map(modified_epoch).unwrap_or(0);
            Some(BackupFileEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path,
                size_bytes: meta.len(),
                created,
                modified,
                modified_epoch: mod_epoch,
            })
        })
        .collect();

    entries.sort_by_key(|b| std::cmp::Reverse(b.modified_epoch));
    entries.truncate(MAX_DISPLAY_FILES);
    entries
}

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
    render_backup_files(ui, settings, state.backup_history.len());
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
                        .size(12.0)
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
                            .size(12.0)
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
                for format in BackupFormat::BACKUP_TAB_OPTIONS {
                    backup_format_button(ui, &mut state.backup_format, format);
                }
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
                        RichText::new(backup_running_detail(state.backup_format))
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
        BackupFormat::Tar => t("backup_tar_archive"),
        BackupFormat::SqlOnly => t("backup_builtin_sql"),
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

fn backup_running_detail(format: BackupFormat) -> String {
    match format {
        BackupFormat::SqlOnly => t("backup_builtin_running"),
        BackupFormat::Custom | BackupFormat::Plain | BackupFormat::Tar => {
            t("backup_pg_dump_running")
        }
    }
}

// ---------------------------------------------------------------------------
// Backup file browser card
// ---------------------------------------------------------------------------

fn render_backup_files(ui: &mut egui::Ui, settings: &AppSettings, history_len: usize) {
    let dir = settings.backup_directory.trim();
    let folder_set = !dir.is_empty();

    let cache_id = egui::Id::new("backup_files_cache");
    let refresh_id = egui::Id::new("backup_files_refresh");
    let delete_confirm_id = egui::Id::new("backup_file_delete_confirm");
    let history_len_id = egui::Id::new("backup_files_history_len");

    // Detect new backup completion by comparing history length.
    let history_changed = ui.data_mut(|d| {
        let prev: usize = d.get_temp(history_len_id).unwrap_or(0);
        d.insert_temp(history_len_id, history_len);
        history_len != prev && prev != 0
    });

    // Determine whether we need a (re-)scan.
    let needs_scan = history_changed
        || ui.data_mut(|d| {
            if d.get_temp::<bool>(refresh_id).unwrap_or(false) {
                d.insert_temp(refresh_id, false);
                return true;
            }
            match d.get_temp::<BackupFilesCache>(cache_id) {
                Some(cache) => cache.directory != dir,
                None => true,
            }
        });

    if needs_scan && folder_set {
        let entries = scan_backup_directory(dir);
        ui.data_mut(|d| {
            d.insert_temp(
                cache_id,
                BackupFilesCache {
                    directory: dir.to_owned(),
                    entries,
                },
            );
        });
    }

    let entries: Vec<BackupFileEntry> = if folder_set {
        ui.data_mut(|d| {
            d.get_temp::<BackupFilesCache>(cache_id)
                .map(|c| c.entries.clone())
                .unwrap_or_default()
        })
    } else {
        Vec::new()
    };

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            // Header row
            ui.horizontal(|ui| {
                let header = if folder_set {
                    tf("backup_files_title_count", &[&entries.len().to_string()])
                } else {
                    t("backup_files_title")
                };
                ui.label(
                    RichText::new(header)
                        .color(theme::text_primary())
                        .size(12.0)
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if folder_set
                        && ui
                            .add(theme::ghost_button(&t("backup_files_refresh")))
                            .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(refresh_id, true));
                    }
                });
            });

            ui.add_space(theme::SPACE_MD);

            if !folder_set {
                ui.label(
                    RichText::new(t("backup_files_set_folder"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                return;
            }

            if entries.is_empty() {
                ui.label(
                    RichText::new(t("backup_files_empty"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                return;
            }

            // Column header
            ui.horizontal(|ui| {
                let header_color = theme::text_muted();
                let w_name = 220.0;
                let w_size = 70.0;
                let w_date = 130.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 16.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add_sized(
                            [w_name, 16.0],
                            egui::Label::new(
                                RichText::new(t("backup_files_col_name"))
                                    .color(header_color)
                                    .size(10.5)
                                    .strong(),
                            ),
                        );
                        ui.add_sized(
                            [w_size, 16.0],
                            egui::Label::new(
                                RichText::new(t("backup_files_col_size"))
                                    .color(header_color)
                                    .size(10.5)
                                    .strong(),
                            ),
                        );
                        ui.add_sized(
                            [w_date, 16.0],
                            egui::Label::new(
                                RichText::new(t("backup_files_col_created"))
                                    .color(header_color)
                                    .size(10.5)
                                    .strong(),
                            ),
                        );
                        ui.add_sized(
                            [w_date, 16.0],
                            egui::Label::new(
                                RichText::new(t("backup_files_col_modified"))
                                    .color(header_color)
                                    .size(10.5)
                                    .strong(),
                            ),
                        );
                        ui.label(
                            RichText::new(t("backup_files_col_actions"))
                                .color(header_color)
                                .size(10.5)
                                .strong(),
                        );
                    },
                );
            });

            ui.add_space(theme::SPACE_XS);

            // Separator
            ui.separator();

            // File rows inside a scroll area
            ScrollArea::vertical()
                .id_salt("backup_files_scroll")
                .max_height(300.0)
                .show(ui, |ui| {
                    for (idx, entry) in entries.iter().enumerate() {
                        let row_bg = if idx % 2 == 0 {
                            theme::bg_dark()
                        } else {
                            theme::bg_medium()
                        };

                        egui::Frame::new()
                            .fill(row_bg)
                            .corner_radius(CornerRadius::same(theme::RADIUS_SM))
                            .inner_margin(Margin::symmetric(
                                theme::SPACE_SM as i8,
                                theme::SPACE_XS as i8,
                            ))
                            .show(ui, |ui| {
                                let w_name = 220.0;
                                let w_size = 70.0;
                                let w_date = 130.0;

                                ui.horizontal(|ui| {
                                    // Name (monospace)
                                    ui.add_sized(
                                        [w_name, 16.0],
                                        egui::Label::new(
                                            RichText::new(&entry.name)
                                                .color(theme::text_primary())
                                                .size(11.0)
                                                .monospace(),
                                        )
                                        .truncate(),
                                    );

                                    // Size
                                    ui.add_sized(
                                        [w_size, 16.0],
                                        egui::Label::new(
                                            RichText::new(format_size(entry.size_bytes))
                                                .color(theme::text_secondary())
                                                .size(10.5),
                                        ),
                                    );

                                    // Created
                                    ui.add_sized(
                                        [w_date, 16.0],
                                        egui::Label::new(
                                            RichText::new(
                                                entry
                                                    .created
                                                    .as_deref()
                                                    .unwrap_or("—"),
                                            )
                                            .color(theme::text_muted())
                                            .size(10.5),
                                        ),
                                    );

                                    // Modified
                                    ui.add_sized(
                                        [w_date, 16.0],
                                        egui::Label::new(
                                            RichText::new(
                                                entry
                                                    .modified
                                                    .as_deref()
                                                    .unwrap_or("—"),
                                            )
                                            .color(theme::text_muted())
                                            .size(10.5),
                                        ),
                                    );

                                    // Actions
                                    if ui
                                        .add(theme::ghost_button(&t("backup_files_show")))
                                        .clicked()
                                    {
                                        reveal_in_finder(&entry.path);
                                    }

                                    // Delete with inline confirmation
                                    let row_delete_id =
                                        delete_confirm_id.with(idx);
                                    let confirming = ui.data_mut(|d| {
                                        d.get_temp::<bool>(row_delete_id)
                                            .unwrap_or(false)
                                    });

                                    if confirming {
                                        ui.label(
                                            RichText::new(t("backup_files_delete_confirm"))
                                                .color(theme::ACCENT_RED)
                                                .size(10.5),
                                        );
                                        let yes_btn = egui::Button::new(
                                            RichText::new(t("backup_files_yes"))
                                                .color(theme::text_primary())
                                                .size(10.5),
                                        )
                                        .fill(theme::ACCENT_RED)
                                        .corner_radius(CornerRadius::same(
                                            theme::RADIUS_SM,
                                        ));
                                        if ui.add(yes_btn).clicked() {
                                            let _ =
                                                std::fs::remove_file(&entry.path);
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    row_delete_id,
                                                    false,
                                                );
                                                d.insert_temp(refresh_id, true);
                                            });
                                        }
                                        let no_btn = egui::Button::new(
                                            RichText::new(t("backup_files_no"))
                                                .color(theme::text_secondary())
                                                .size(10.5),
                                        )
                                        .fill(theme::bg_darkest())
                                        .corner_radius(CornerRadius::same(
                                            theme::RADIUS_SM,
                                        ));
                                        if ui.add(no_btn).clicked() {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    row_delete_id,
                                                    false,
                                                );
                                            });
                                        }
                                    } else {
                                        let del_btn = egui::Button::new(
                                            RichText::new(t("backup_files_delete"))
                                                .color(theme::ACCENT_RED)
                                                .size(10.5),
                                        )
                                        .fill(theme::bg_darkest())
                                        .stroke(Stroke::new(
                                            1.0,
                                            theme::ACCENT_RED,
                                        ))
                                        .corner_radius(CornerRadius::same(
                                            theme::RADIUS_SM,
                                        ));
                                        if ui.add(del_btn).clicked() {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    row_delete_id,
                                                    true,
                                                );
                                            });
                                        }
                                    }
                                });
                            });

                        if idx < entries.len() - 1 {
                            ui.add_space(theme::SPACE_XS);
                        }
                    }
                });
        });
}

fn reveal_in_finder(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
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
                    .size(12.0)
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
