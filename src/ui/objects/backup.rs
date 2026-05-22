//! Premium Navicat-style backup manager.
//! Replaces legacy configs and folder scans with an integrated action toolbar,
//! a search-filtered persistent backup history list, live file availability checks,
//! and triggers for the new FGB wizard/restore modal dialogs.

use eframe::egui::{self, CornerRadius, Margin, RichText, ScrollArea, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{get_language, Language};
use crate::state::{AppState, RestoreConfirmState};
use crate::storage::settings::AppSettings;
use crate::types::BackupRecord;
use crate::ui::{icons_svg, theme};

use super::{active_conn, render_no_connection, ObjectAction};

// ---------------------------------------------------------------------------
// Translation Helper
// ---------------------------------------------------------------------------

fn translate(ko: &str, en: &str) -> String {
    match get_language() {
        Language::Korean => ko.to_string(),
        _ => en.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Main Render Entry Point
// ---------------------------------------------------------------------------

pub(super) fn render_backup_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut AppSettings,
) -> Option<ObjectAction> {
    let Some(_conn_id) = active_conn(state) else {
        return render_no_connection(ui);
    };

    ui.add_space(theme::SPACE_MD);

    // 1. Render Top Action Toolbar
    render_action_toolbar(ui, state, settings);

    ui.add_space(theme::SPACE_LG);

    // 2. Render Main Backup List
    render_backup_list_view(ui, state, bridge, settings);

    None
}

// ---------------------------------------------------------------------------
// Action Toolbar Rendering
// ---------------------------------------------------------------------------

fn render_action_toolbar(ui: &mut egui::Ui, state: &mut AppState, settings: &AppSettings) {
    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, theme::SPACE_MD as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Icon and Title
                crate::ui::icon_img(ui, icons_svg::BACKUP, "backup_header_icon", 16.0);
                ui.label(
                    RichText::new(translate("백업 관리자", "Backup Manager"))
                        .color(theme::text_primary())
                        .size(13.0)
                        .strong(),
                );
                
                ui.add_space(theme::SPACE_LG);

                // [+ New Backup] Green Pill Button
                let new_btn = egui::Button::new(
                    RichText::new(translate("+ 새 백업 생성", "+ New Backup"))
                        .color(theme::text_primary())
                        .strong()
                        .size(11.5),
                )
                .fill(theme::accent_color())
                .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                .min_size(egui::vec2(100.0, 24.0));

                if ui.add(new_btn).clicked() {
                    state.show_backup_wizard = true;
                    state.backup_wizard_state = None; // Reset wizard to step 0
                }

                // [Open Backup Folder] Ghost Button
                let folder_set = !settings.backup_directory.trim().is_empty();
                let open_btn = egui::Button::new(
                    RichText::new(translate("백업 폴더 열기", "Open Folder"))
                        .color(theme::text_secondary())
                        .size(11.0),
                )
                .fill(theme::bg_darkest())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(theme::RADIUS_MD));

                if ui.add_enabled(folder_set, open_btn).clicked() {
                    open_backup_folder(&settings.backup_directory);
                }

                // Search Filter Input Box (Right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.set_max_width(200.0);
                    
                    // Store search query in UI temp storage or reuse an existing workspace tab filter
                    let mut search_query = ui.data_mut(|d| {
                        d.get_temp::<String>(egui::Id::new("backup_search_query"))
                            .unwrap_or_default()
                    });

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut search_query)
                            .hint_text(translate("검색 (파일명, DB명...)", "Search backups..."))
                            .margin(Margin::symmetric(8, 4))
                    );

                    if response.changed() {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("backup_search_query"), search_query);
                        });
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Main List & Table View
// ---------------------------------------------------------------------------

fn render_backup_list_view(
    ui: &mut egui::Ui,
    state: &mut AppState,
    _bridge: &DbBridge,
    _settings: &AppSettings,
) {
    let mut to_delete = None;
    let mut to_restore = None;
    // Get search filter from temp storage
    let search_query = ui.data_mut(|d| {
        d.get_temp::<String>(egui::Id::new("backup_search_query"))
            .unwrap_or_default()
    }).to_lowercase();

    // Filters history list
    let filtered_records: Vec<&BackupRecord> = state
        .backup_history
        .iter()
        .filter(|record| {
            if search_query.is_empty() {
                return true;
            }
            let filename = record
                .file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string().to_lowercase())
                .unwrap_or_default();
            let db = record.database.to_lowercase();
            let conn_name = record.connection_name.to_lowercase();
            
            filename.contains(&search_query) || db.contains(&search_query) || conn_name.contains(&search_query)
        })
        .collect();

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_LG as i8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Table Header / Count
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(translate(
                            &format!("보관된 백업 ({})", filtered_records.len()),
                            &format!("Archived Backups ({})", filtered_records.len()),
                        ))
                        .color(theme::text_primary())
                        .size(12.0)
                        .strong(),
                    );
                });

                ui.add_space(theme::SPACE_MD);

                if filtered_records.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.add_space(theme::SPACE_XL);
                        ui.label(
                            RichText::new(translate(
                                "보관되거나 조건에 부합하는 백업 파일이 없습니다.",
                                "No archived backups found matching criteria.",
                            ))
                            .color(theme::text_muted())
                            .size(11.0),
                        );
                        ui.add_space(theme::SPACE_XL);
                    });
                    return;
                }

                // Table Headers Row
                ui.horizontal(|ui| {
                    let text_color = theme::text_muted();
                    let w_name = 180.0;
                    let w_conn = 140.0;
                    let w_size = 70.0;
                    let w_date = 130.0;

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 16.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.add_sized(
                                [w_name, 16.0],
                                egui::Label::new(
                                    RichText::new(translate("백업 파일명", "Backup File Name"))
                                        .color(text_color)
                                        .size(10.5)
                                        .strong(),
                                ),
                            );

                            ui.add_sized(
                                [w_conn, 16.0],
                                egui::Label::new(
                                    RichText::new(translate("접속 / DB 정보", "Connection / DB"))
                                        .color(text_color)
                                        .size(10.5)
                                        .strong(),
                                ),
                            );

                            ui.add_sized(
                                [w_size, 16.0],
                                egui::Label::new(
                                    RichText::new(translate("크기", "Size"))
                                        .color(text_color)
                                        .size(10.5)
                                        .strong(),
                                ),
                            );

                            ui.add_sized(
                                [w_date, 16.0],
                                egui::Label::new(
                                    RichText::new(translate("완료 일시", "Completed Date"))
                                        .color(text_color)
                                        .size(10.5)
                                        .strong(),
                                ),
                            );

                            ui.label(
                                RichText::new(translate("작업 및 도구", "Actions"))
                                    .color(text_color)
                                    .size(10.5)
                                    .strong(),
                            );
                        },
                    );
                });

                ui.add_space(theme::SPACE_XS);
                ui.separator();

                // Rows View inside Scroll Area
                ScrollArea::vertical()
                    .id_salt("backup_history_scroll_view")
                    .max_height(400.0)
                    .show(ui, |ui| {
                        // Reverse chronological ordering (latest on top)
                        for (idx, record) in filtered_records.iter().rev().enumerate() {
                            let row_bg = if idx % 2 == 0 {
                                theme::bg_dark()
                            } else {
                                theme::bg_medium()
                            };

                            let file_exists = record.file_path.exists();

                            egui::Frame::new()
                                .fill(row_bg)
                                .corner_radius(CornerRadius::same(theme::RADIUS_SM))
                                .inner_margin(Margin::symmetric(
                                    theme::SPACE_LG as i8,
                                    theme::SPACE_SM as i8,
                                ))
                                .show(ui, |ui| {
                                    let w_name = 180.0;
                                    let w_conn = 140.0;
                                    let w_size = 70.0;
                                    let w_date = 130.0;

                                    ui.horizontal(|ui| {
                                        // 1. Monospaced File name + Badge if missing
                                        ui.add_sized(
                                            [w_name, 16.0],
                                            egui::Label::new(
                                                RichText::new(
                                                    record
                                                        .file_path
                                                        .file_name()
                                                        .map(|n| n.to_string_lossy().to_string())
                                                        .unwrap_or_else(|| "backup.fgb".to_string()),
                                                )
                                                .color(if file_exists { theme::text_primary() } else { theme::text_muted() })
                                                .size(11.0)
                                                .monospace(),
                                            )
                                            .truncate(),
                                        );

                                        // 2. Connection / DB Name + optional schema context
                                        let schema_context = record.schema.as_deref().unwrap_or("full");
                                        let conn_db_text = format!("{} ({}.{})", record.connection_name, record.database, schema_context);
                                        ui.add_sized(
                                            [w_conn, 16.0],
                                            egui::Label::new(
                                                RichText::new(conn_db_text)
                                                    .color(theme::text_secondary())
                                                    .size(10.5),
                                            )
                                            .truncate(),
                                        );

                                        // 3. Formatted Size
                                        let size_text = if file_exists {
                                            format_size(record.size_bytes)
                                        } else {
                                            translate("누락됨", "Missing").to_string()
                                        };
                                        ui.add_sized(
                                            [w_size, 16.0],
                                            egui::Label::new(
                                                RichText::new(size_text)
                                                    .color(if file_exists { theme::text_secondary() } else { theme::ACCENT_RED })
                                                    .size(10.5)
                                                    .strong(),
                                            ),
                                        );

                                        // 4. Completed At Date
                                        ui.add_sized(
                                            [w_date, 16.0],
                                            egui::Label::new(
                                                RichText::new(&record.completed_at)
                                                    .color(theme::text_muted())
                                                    .size(10.5),
                                            ),
                                        );

                                        // 5. Action Triggers
                                        // [Restore] Button (Enabled only if file exists)
                                        let restore_btn = egui::Button::new(
                                            RichText::new(translate("복원", "Restore"))
                                                .color(if file_exists { theme::text_primary() } else { theme::text_disabled() })
                                                .size(10.0)
                                                .strong(),
                                        )
                                        .fill(if file_exists { theme::accent_color_dim() } else { theme::bg_darkest() })
                                        .stroke(Stroke::new(1.0, if file_exists { theme::accent_color() } else { theme::border_default() }))
                                        .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                                        if ui.add_enabled(file_exists, restore_btn).clicked() {
                                            to_restore = Some((*record).clone());
                                        }

                                        // [Location / Finder] Button (Enabled only if file exists)
                                        let loc_btn = egui::Button::new(
                                            RichText::new(translate("위치", "Locate"))
                                                .color(theme::text_secondary())
                                                .size(10.0),
                                        )
                                        .fill(theme::bg_darkest())
                                        .stroke(Stroke::new(1.0, theme::border_default()))
                                        .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                                        if ui.add_enabled(file_exists, loc_btn).clicked() {
                                            reveal_in_finder(&record.file_path);
                                        }

                                        // [Delete] Button (Double check confirmations inline)
                                        let row_delete_id = egui::Id::new("backup_history_del_confirm").with(idx);
                                        let is_confirming_del = ui.data_mut(|d| {
                                            d.get_temp::<bool>(row_delete_id).unwrap_or(false)
                                        });

                                        if is_confirming_del {
                                            let confirm_del_btn = egui::Button::new(
                                                RichText::new(translate("삭제 확정", "Confirm Delete"))
                                                    .color(theme::text_primary())
                                                    .size(10.0)
                                                    .strong(),
                                            )
                                            .fill(theme::ACCENT_RED)
                                            .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                                            if ui.add(confirm_del_btn).clicked() {
                                                // 1. Delete physical file if it exists
                                                if file_exists {
                                                    let _ = std::fs::remove_file(&record.file_path);
                                                }

                                                // 2. Mark for deletion outside closure
                                                to_delete = Some(record.file_path.clone());

                                                ui.data_mut(|d| {
                                                    d.insert_temp(row_delete_id, false);
                                                });
                                            }

                                            let cancel_del_btn = egui::Button::new(
                                                RichText::new(translate("취소", "Cancel"))
                                                    .color(theme::text_secondary())
                                                    .size(10.0),
                                            )
                                            .fill(theme::bg_darkest())
                                            .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                                            if ui.add(cancel_del_btn).clicked() {
                                                ui.data_mut(|d| {
                                                    d.insert_temp(row_delete_id, false);
                                                });
                                            }
                                        } else {
                                            let del_btn = egui::Button::new(
                                                RichText::new(translate("삭제", "Delete"))
                                                    .color(theme::ACCENT_RED)
                                                    .size(10.0),
                                            )
                                            .fill(theme::bg_darkest())
                                            .stroke(Stroke::new(1.0, theme::ACCENT_RED))
                                            .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                                            if ui.add(del_btn).clicked() {
                                                ui.data_mut(|d| {
                                                    d.insert_temp(row_delete_id, true);
                                                });
                                            }
                                        }
                                    });
                                });

                            ui.add_space(theme::SPACE_XS);
                        }
                    });
            });
        });

    if let Some(target_path) = to_delete {
        state.backup_history.retain(|r| r.file_path != target_path);
        crate::storage::backups::save_backups(&state.backup_history);
    }

    if let Some(record) = to_restore {
        state.restore_confirm_dialog = Some(RestoreConfirmState {
            record,
            running: false,
            progress: 0.0,
            completed: false,
            error: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Shell Helpers / Utilities
// ---------------------------------------------------------------------------

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
