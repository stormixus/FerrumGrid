use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
#[cfg(target_os = "macos")]
use eframe::egui::Sense;
use std::path::PathBuf;

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{get_language, Language};
use crate::state::{AppState, BackupWizardState};
use crate::storage::settings::AppSettings;
use crate::types::{BackupFormat, BackupRequest};
use crate::ui::theme;

// ---------------------------------------------------------------------------
// Translation Helpers
// ---------------------------------------------------------------------------

fn translate(ko: &str, en: &str) -> String {
    match get_language() {
        Language::Korean => ko.to_string(),
        _ => en.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Render Backup Wizard
// ---------------------------------------------------------------------------

pub fn render_backup_wizard(
    ctx: &egui::Context,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &AppSettings,
) {
    if !state.show_backup_wizard {
        return;
    }

    // Initialize state if missing
    if state.backup_wizard_state.is_none() {
        // Retrieve active tab's schema filter to check if a specific schema was selected in the tree sidebar
        let active_tab_idx = state.active_workspace_tab;
        let active_schema_filter = state.workspace_tabs.get(active_tab_idx)
            .map(|tab| tab.schema_filter.clone())
            .unwrap_or_default();

        let (initial_step, initial_scope) = if !active_schema_filter.is_empty() {
            // A specific schema is selected from tree browser: skip step 0 (Scope Selection)
            (1, Some(active_schema_filter))
        } else {
            // Full database/no specific schema selected: start at step 0
            (0, None)
        };

        state.backup_wizard_state = Some(std::sync::Arc::new(std::sync::Mutex::new(BackupWizardState {
            step: initial_step,
            schema_scope: initial_scope,
            format: BackupFormat::Fgb,
            running: false,
            progress: 0.0,
            current_table: String::new(),
            completed: false,
            error: None,
            closed: false,
        })));
    }

    let wizard_arc = state.backup_wizard_state.clone().unwrap();

    let active_conn_id = state.active_connection;
    let is_active_conn_valid = active_conn_id.is_some();

    let active_tab_idx = state.active_workspace_tab;
    let has_predefined_schema = state.workspace_tabs.get(active_tab_idx)
        .map(|tab| !tab.schema_filter.is_empty())
        .unwrap_or(false);

    let conn_config = active_conn_id
        .and_then(|id| state.connections.get(&id))
        .map(|conn| conn.config.clone());

    let schemas = active_conn_id
        .and_then(|id| state.connections.get(&id))
        .map(|conn| conn.schemas.clone())
        .unwrap_or_default();

    let database_name = active_conn_id
        .and_then(|id| state.connections.get(&id))
        .map(|conn| conn.config.database.clone())
        .unwrap_or_else(|| "postgres".to_string());

    let last_record = state.backup_history.last().cloned();
    let backup_directory = std::path::PathBuf::from(&settings.backup_directory);
    let cmd_tx = bridge.cmd_sender();

    let viewport_id = egui::ViewportId::from_hash_of("backup_wizard_viewport");
    #[allow(unused_mut)]
    let mut builder = egui::ViewportBuilder::default()
        .with_title(translate("데이터베이스 백업 위자드", "Database Backup Wizard"))
        .with_inner_size(egui::vec2(550.0, 480.0))
        .with_resizable(false)
        .with_minimize_button(false)
        .with_maximize_button(false);

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .with_fullsize_content_view(true)
            .with_title_shown(false);
    }

    ctx.show_viewport_immediate(
        viewport_id,
        builder,
        move |ctx, _class| {
            let mut wizard = wizard_arc.lock().unwrap();

            if ctx.input(|i| i.viewport().close_requested()) {
                wizard.closed = true;
            }

            #[cfg(target_os = "macos")]
            {
                crate::ui::titlebar::configure_macos_titlebar();

                let top_frame = egui::Frame::NONE
                    .fill(theme::bg_shell())
                    .inner_margin(Margin::symmetric(16, 0));

                egui::TopBottomPanel::top("wizard_titlebar")
                    .exact_height(32.0)
                    .frame(top_frame)
                    .show_separator_line(false)
                    .show(ctx, |ui| {
                        let full_rect = ui.max_rect();
                        
                        let drag_response = ui.interact(
                            full_rect,
                            ui.id().with("wizard_titlebar_drag"),
                            Sense::click_and_drag(),
                        );
                        if drag_response.drag_started_by(egui::PointerButton::Primary) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }

                        let painter = ui.painter();
                        let title_text = translate("데이터베이스 백업 위자드", "Database Backup Wizard");
                        let font_id = egui::FontId::proportional(11.5);
                        let color = theme::text_secondary();
                        
                        let galley = painter.layout_no_wrap(title_text, font_id, color);
                        let text_pos = egui::pos2(80.0, full_rect.center().y - galley.size().y * 0.5);
                        painter.galley(text_pos, galley, color);
                    });
            }

            egui::CentralPanel::default()
                .frame(
                    egui::Frame::NONE
                        .fill(theme::bg_medium())
                        .inner_margin(Margin::same(theme::SPACE_XXL as i8))
                )
                .show(ctx, |ui| {
                    let step = wizard.step;

                    // 1. Step Indicator
                    render_step_indicator(ui, step);
                    ui.add_space(theme::SPACE_LG);
                    ui.separator();
                    ui.add_space(theme::SPACE_LG);

                    // 2. Main Content Area depending on Step
                    match step {
                        0 => render_step_scope(ui, &schemas, &database_name, &mut wizard),
                        1 => {
                            render_step_format(ui, &mut wizard);
                        }
                        2 => {
                            if let (Some(active_conn_id), Some(conn_config)) = (active_conn_id, conn_config.clone()) {
                                render_step_execute(
                                    ui,
                                    active_conn_id,
                                    conn_config,
                                    last_record.clone(),
                                    backup_directory.clone(),
                                    cmd_tx.clone(),
                                    &mut wizard,
                                );
                            } else {
                                ui.label(translate("활성화된 연결이 없습니다.", "No active connection available."));
                            }
                        }
                        _ => {}
                    }

                    ui.add_space(theme::SPACE_XL);
                    ui.separator();
                    ui.add_space(theme::SPACE_MD);

                    // 3. Navigation Controls
                    ui.horizontal(|ui| {
                        let mut step_decremented = false;
                        let mut step_incremented = false;
                        let mut done_clicked = false;
                        let mut cancel_clicked = false;

                        let w_step = wizard.step;
                        let w_running = wizard.running;
                        let w_completed = wizard.completed;

                        let min_step = if has_predefined_schema { 1 } else { 0 };

                        if w_step > min_step && w_step < 2 && !w_running {
                            let btn = ui.button(translate("이전", "Back"));
                            if btn.clicked() {
                                step_decremented = true;
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if w_step == 2 && w_completed {
                                let btn = ui.button(translate("완료", "Done"));
                                if btn.clicked() {
                                    done_clicked = true;
                                }
                            } else if w_step < 2 {
                                let next_text = if w_step == 1 {
                                    translate("백업 시작", "Start Backup")
                                } else {
                                    translate("다음", "Next")
                                };

                                let next_btn = ui.add_enabled(
                                    is_active_conn_valid,
                                    egui::Button::new(RichText::new(next_text).color(theme::text_primary()))
                                        .fill(theme::accent_color())
                                        .min_size(egui::vec2(80.0, 24.0)),
                                );

                                if next_btn.clicked() {
                                    step_incremented = true;
                                }
                            }

                            if !w_running {
                                let cancel_btn = ui.button(translate("취소", "Cancel"));
                                if cancel_btn.clicked() {
                                    cancel_clicked = true;
                                }
                            }
                        });

                        if step_decremented {
                            wizard.step -= 1;
                        }
                        if step_incremented {
                            wizard.step += 1;
                        }
                        if done_clicked {
                            wizard.closed = true;
                        }
                        if cancel_clicked {
                            wizard.closed = true;
                        }
                    });
                });
        }
    );

    let should_close = {
        if let Some(ref wizard_lock) = state.backup_wizard_state {
            let wizard = wizard_lock.lock().unwrap();
            wizard.closed
        } else {
            false
        }
    };

    if should_close {
        state.show_backup_wizard = false;
        state.backup_wizard_state = None;
    }
}

// ---------------------------------------------------------------------------
// Backup Wizard Steps
// ---------------------------------------------------------------------------

fn render_step_indicator(ui: &mut egui::Ui, current_step: usize) {
    let steps = [
        translate("백업 범위", "Scope Selection"),
        translate("형식 지정", "Format Selection"),
        translate("실행 및 결과", "Run & Progress"),
    ];

    ui.horizontal(|ui| {
        for (i, name) in steps.iter().enumerate() {
            let is_active = i == current_step;
            let is_past = i < current_step;

            let color = if is_active {
                theme::accent_color()
            } else if is_past {
                theme::text_primary()
            } else {
                theme::text_muted()
            };

            let badge_text = if is_past {
                "✓".to_string()
            } else {
                format!("{}", i + 1)
            };

            ui.horizontal(|ui| {
                // Circle Badge
                let rect = ui.allocate_space(egui::vec2(20.0, 20.0)).1;
                let bg_color = if is_active || is_past {
                    theme::accent_color_dim()
                } else {
                    theme::bg_light()
                };

                let stroke_color = if is_active || is_past {
                    theme::accent_color()
                } else {
                    theme::border_default()
                };

                ui.painter().circle(
                    rect.center(),
                    10.0,
                    bg_color,
                    Stroke::new(1.0, stroke_color),
                );

                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    badge_text,
                    egui::FontId::monospace(10.0),
                    color,
                );

                ui.add_space(theme::SPACE_XS);

                // Step Name
                ui.label(
                    RichText::new(name)
                        .color(color)
                        .strong()
                        .size(12.0),
                );

                if i < steps.len() - 1 {
                    ui.add_space(theme::SPACE_MD);
                    ui.label(RichText::new("→").color(theme::text_muted()));
                    ui.add_space(theme::SPACE_MD);
                }
            });
        }
    });
}

fn render_step_scope(
    ui: &mut egui::Ui,
    schemas: &[String],
    database_name: &str,
    wizard: &mut BackupWizardState,
) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(translate(
                "백업할 스키마 범위를 선택하세요.",
                "Choose the scope of schemas to back up.",
            ))
            .color(theme::text_secondary())
            .size(13.0),
        );
        ui.add_space(theme::SPACE_LG);

        // Card style Selection
        // Option A: Full Database
        let is_full = wizard.schema_scope.is_none();
        let card_fill = if is_full { theme::bg_light() } else { theme::bg_darkest() };
        let card_stroke = if is_full { Stroke::new(1.0, theme::accent_color()) } else { Stroke::new(1.0, theme::border_default()) };
        
        let response = egui::Frame::NONE
            .fill(card_fill)
            .stroke(card_stroke)
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_MD_I))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let radio_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                    ui.painter().circle(radio_rect.center(), 8.0, theme::bg_medium(), Stroke::new(1.0, if is_full { theme::accent_color() } else { theme::text_muted() }));
                    if is_full {
                        ui.painter().circle_filled(radio_rect.center(), 4.0, theme::accent_color());
                    }
                    ui.add_space(theme::SPACE_SM);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(translate("전체 데이터베이스", "Full Database")).strong().color(theme::text_primary()));
                        ui.label(RichText::new(format!(
                            "{} '{}' - {}", 
                            translate("데이터베이스", "Database"), 
                            database_name,
                            translate("모든 스키마 및 테이블 포함", "All schemas and tables included")
                        )).size(11.0).color(theme::text_muted()));
                    });
                });
            }).response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        if response.clicked() {
            wizard.schema_scope = None;
        }

        ui.add_space(theme::SPACE_MD);

        // Option B: Schema Scope
        let is_schema = wizard.schema_scope.is_some();
        let card_fill = if is_schema { theme::bg_light() } else { theme::bg_darkest() };
        let card_stroke = if is_schema { Stroke::new(1.0, theme::accent_color()) } else { Stroke::new(1.0, theme::border_default()) };
        
        let response = egui::Frame::NONE
            .fill(card_fill)
            .stroke(card_stroke)
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_MD_I))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let radio_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                    ui.painter().circle(radio_rect.center(), 8.0, theme::bg_medium(), Stroke::new(1.0, if is_schema { theme::accent_color() } else { theme::text_muted() }));
                    if is_schema {
                        ui.painter().circle_filled(radio_rect.center(), 4.0, theme::accent_color());
                    }
                    ui.add_space(theme::SPACE_SM);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(translate("특정 스키마 백업", "Specific Schema")).strong().color(theme::text_primary()));
                        ui.label(RichText::new(translate(
                            "단일 스키마 내의 구조 및 데이터만 백업합니다.",
                            "Only back up structures and data inside a single schema."
                        )).size(11.0).color(theme::text_muted()));
                    });
                });
            }).response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        if response.clicked() && wizard.schema_scope.is_none() {
            wizard.schema_scope = Some(schemas.first().cloned().unwrap_or_else(|| "public".to_string()));
        }

        if is_schema {
            ui.add_space(theme::SPACE_SM);
            ui.horizontal(|ui| {
                ui.add_space(theme::SPACE_XXL);
                ui.label(RichText::new(translate("대상 스키마:", "Target Schema:")).color(theme::text_secondary()));
                
                let current_schema = wizard.schema_scope.as_deref().unwrap_or("public");
                egui::ComboBox::from_id_salt("backup_schema_select")
                    .selected_text(current_schema)
                    .show_ui(ui, |ui| {
                        for schema in schemas {
                            let is_selected = Some(schema.clone()) == wizard.schema_scope;
                            if ui.selectable_label(is_selected, schema).clicked() {
                                wizard.schema_scope = Some(schema.clone());
                            }
                        }
                    });
            });
        }
    });
}

fn render_step_format(ui: &mut egui::Ui, wizard: &mut BackupWizardState) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(translate(
                "원하는 백업 파일 형식을 선택하십시오.",
                "Select the desired backup file format.",
            ))
            .color(theme::text_secondary())
            .size(13.0),
        );
        ui.add_space(theme::SPACE_LG);

        // FGB Format Card (RECOMMENDED)
        let is_fgb = wizard.format == BackupFormat::Fgb;
        let card_fill = if is_fgb { theme::bg_light() } else { theme::bg_darkest() };
        let card_stroke = if is_fgb { Stroke::new(1.2, theme::accent_color()) } else { Stroke::new(1.0, theme::border_default()) };
        
        let response = egui::Frame::NONE
            .fill(card_fill)
            .stroke(card_stroke)
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_MD_I))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let radio_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                    ui.painter().circle(radio_rect.center(), 8.0, theme::bg_medium(), Stroke::new(1.0, if is_fgb { theme::accent_color() } else { theme::text_muted() }));
                    if is_fgb {
                        ui.painter().circle_filled(radio_rect.center(), 4.0, theme::accent_color());
                    }
                    ui.add_space(theme::SPACE_SM);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("FerrumGrid Backup (.fgb)").strong().color(theme::text_primary()));
                            // Recommended Badge
                            let badge_rect = ui.allocate_space(egui::vec2(80.0, 16.0)).1;
                            ui.painter().rect_filled(badge_rect, CornerRadius::same(3), theme::accent_color_dim());
                            ui.painter().text(
                                badge_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                translate("추천 (기본)", "RECOMMENDED"),
                                egui::FontId::monospace(9.0),
                                theme::accent_color(),
                            );
                        });
                        ui.label(RichText::new(translate(
                            "고성능 바이너리 스트리밍 엔진. 복원 시 외래 키를 자동으로 정렬 및 후적용하여 의존성 충돌을 방지하며, 별도 나비캣 스타일로 내장 백업 폴더에 안전하게 저장됩니다.",
                            "High-performance streaming binary engine. Auto-sorts and post-applies FKs to avoid dependency conflicts, and safely saves to the built-in backup folder in a Navicat style."
                        )).size(11.0).color(theme::text_muted()));
                    });
                });
            }).response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        if response.clicked() {
            wizard.format = BackupFormat::Fgb;
        }

        ui.add_space(theme::SPACE_MD);

        // SQL Only Card
        let is_sql = wizard.format == BackupFormat::SqlOnly;
        let card_fill = if is_sql { theme::bg_light() } else { theme::bg_darkest() };
        let card_stroke = if is_sql { Stroke::new(1.0, theme::accent_color()) } else { Stroke::new(1.0, theme::border_default()) };
        
        let response = egui::Frame::NONE
            .fill(card_fill)
            .stroke(card_stroke)
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_MD_I))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let radio_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                    ui.painter().circle(radio_rect.center(), 8.0, theme::bg_medium(), Stroke::new(1.0, if is_sql { theme::accent_color() } else { theme::text_muted() }));
                    if is_sql {
                        ui.painter().circle_filled(radio_rect.center(), 4.0, theme::accent_color());
                    }
                    ui.add_space(theme::SPACE_SM);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(translate("텍스트 SQL 스크립트 (.sql) (내장)", "Plaintext SQL Script (.sql) (Built-in)")).strong().color(theme::text_primary()));
                        ui.label(RichText::new(translate(
                            "데이터베이스 구조와 INSERT/COPY 데이터가 텍스트 SQL 파일로 생성됩니다.",
                            "Database structures and INSERT/COPY statements are generated in a plaintext SQL file."
                        )).size(11.0).color(theme::text_muted()));
                    });
                });
            }).response;

        let response = ui.interact(response.rect, response.id, egui::Sense::click());
        if response.clicked() {
            wizard.format = BackupFormat::SqlOnly;
        }
    });
}

fn render_step_execute(
    ui: &mut egui::Ui,
    active_conn_id: crate::types::ConnectionId,
    conn_config: crate::types::ConnectionConfig,
    last_record: Option<crate::types::BackupRecord>,
    backup_directory: PathBuf,
    cmd_tx: tokio::sync::mpsc::Sender<crate::db::bridge::DbCommand>,
    wizard: &mut BackupWizardState,
) {
    ui.vertical_centered_justified(|ui| {
        if !wizard.running && !wizard.completed {
            // Automatically trigger the backup on entering Step 2
            wizard.running = true;
            wizard.completed = false;
            wizard.error = None;
            wizard.progress = 0.0;
            wizard.current_table = String::new();

            let request = BackupRequest {
                conn_id: active_conn_id,
                config: conn_config,
                output_dir: backup_directory,
                schema: wizard.schema_scope.clone(),
                format: wizard.format,
            };

            // Send command
            if let Err(e) = cmd_tx.try_send(DbCommand::RunBackup { request }) {
                tracing::error!("failed to send command to db bridge: {e}");
            }
        }

        if wizard.running {
            // Keep the frame painting to ensure 60fps animations
            ui.ctx().request_repaint();

            ui.add_space(theme::SPACE_LG);

            // 1. Central Animated Glowing Database Stack
            let (rect, _response) = ui.allocate_exact_size(egui::vec2(160.0, 160.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let center = rect.center();
            let time = ui.input(|i| i.time) as f32;

            // Pulse outer ambient glow using active accent color
            let glow_radius = 50.0 + (time * 2.0).sin() * 6.0;
            let glow_alpha = (25.0 + (time * 2.0).sin() * 10.0) as u8;
            painter.circle_filled(
                center,
                glow_radius,
                theme::with_alpha(theme::accent_color(), glow_alpha),
            );

            // Outer orbital comet/trail ring
            let outer_radius = 60.0;
            let dot_count = 24;
            for i in 0..dot_count {
                let angle = (i as f32 / dot_count as f32) * std::f32::consts::TAU + (time * 1.6);
                let alpha = (i as f32 / dot_count as f32 * 200.0) as u8;
                let dot_color = theme::with_alpha(theme::accent_color(), alpha);
                let dot_pos = center + egui::vec2(angle.cos() * outer_radius, angle.sin() * outer_radius);
                painter.circle_filled(
                    dot_pos,
                    1.2 + 1.8 * (i as f32 / dot_count as f32),
                    dot_color,
                );
            }

            // Glassmorphic central container disk
            let disk_radius = 42.0;
            painter.circle_filled(
                center,
                disk_radius,
                theme::BG_DARKEST,
            );
            painter.circle(
                center,
                disk_radius,
                Color32::TRANSPARENT,
                Stroke::new(1.0, theme::BORDER_DEFAULT),
            );

            // Spindle for database plates
            painter.line_segment(
                [egui::pos2(center.x, center.y - 18.0), egui::pos2(center.x, center.y + 18.0)],
                Stroke::new(1.5, theme::accent_color().linear_multiply(0.3)),
            );

            // Pulse three layers of floating plates (database symbol)
            for layer in 0..3 {
                let ly = center.y + 12.0 - (layer as f32 * 12.0);
                let pulse_offset = layer as f32 * 1.5;
                let layer_pulse = 1.0 + (time * 3.0 - pulse_offset).sin() * 0.06;
                let w = 36.0 * layer_pulse;
                let h = 6.0;
                let plate_rect = egui::Rect::from_center_size(
                    egui::pos2(center.x, ly),
                    egui::vec2(w, h),
                );

                // Glow behind the plate
                painter.rect_filled(
                    plate_rect.expand(2.0),
                    CornerRadius::same(3),
                    theme::with_alpha(theme::accent_color(), 15),
                );
                // Dark core of the plate
                painter.rect_filled(
                    plate_rect,
                    CornerRadius::same(3),
                    theme::BG_DARK,
                );
                // Neon stroke
                painter.rect_stroke(
                    plate_rect,
                    CornerRadius::same(3),
                    Stroke::new(1.2, theme::accent_color().linear_multiply(0.8)),
                    egui::StrokeKind::Inside,
                );

                // LED indicator light on the plate
                let led_center = egui::pos2(center.x - 8.0 + (layer as f32 * 4.0), ly);
                let led_pulse = 0.4 + 0.6 * (time * 5.0 - pulse_offset).sin();
                let led_color = theme::with_alpha(theme::accent_color_light(), (led_pulse * 255.0) as u8);
                painter.circle_filled(led_center, 1.2, led_color);
            }

            ui.add_space(theme::SPACE_MD);

            // 2. Title label
            ui.label(
                RichText::new(translate(
                    "백업 파일을 안전하게 생성하고 있습니다...",
                    "Generating backup file securely...",
                ))
                .strong()
                .size(13.5)
                .color(theme::text_primary()),
            );
            ui.add_space(theme::SPACE_XS);

            // Description label
            ui.label(
                RichText::new(translate(
                    "FGB 엔진: 테이블 스트림 데이터 및 스키마 구조 내보내는 중",
                    "FGB Engine: Exporting table stream data & schema structure",
                ))
                .size(11.0)
                .color(theme::text_muted()),
            );

            ui.add_space(theme::SPACE_LG);

            // 3. Custom Premium Progress Bar Card
            let card_width = 380.0;
            egui::Frame::NONE
                .fill(theme::BG_DARKEST.linear_multiply(0.4))
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_LG as i8))
                .show(ui, |ui| {
                    ui.set_width(card_width);
                    
                    // Progress Bar label (Progress % vs Table name)
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(translate("백업 진행률", "Backup Progress"))
                                .size(11.0)
                                .color(theme::text_secondary())
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{:.0}%", wizard.progress * 100.0))
                                    .strong()
                                    .size(12.0)
                                    .color(theme::accent_color())
                            );
                        });
                    });
                    
                    ui.add_space(theme::SPACE_SM);

                    // Draw custom progress track & glowing fill
                    let bar_height = 6.0;
                    let (bar_rect, _res) = ui.allocate_exact_size(egui::vec2(card_width - 24.0, bar_height), egui::Sense::hover());
                    let b_painter = ui.painter_at(bar_rect);

                    // Track background
                    b_painter.rect_filled(bar_rect, CornerRadius::same(3), theme::BG_DARK);
                    b_painter.rect_stroke(bar_rect, CornerRadius::same(3), Stroke::new(1.0, theme::BORDER_DEFAULT), egui::StrokeKind::Inside);

                    let progress_val = wizard.progress.clamp(0.0, 1.0);
                    if progress_val > 0.0 {
                        let fill_width = (bar_rect.width() * progress_val).max(6.0);
                        let fill_rect = egui::Rect::from_min_size(
                            bar_rect.min,
                            egui::vec2(fill_width, bar_height),
                        );
                        // Vibrant accent color fill
                        b_painter.rect_filled(fill_rect, CornerRadius::same(3), theme::accent_color());

                        // Glowing head pointer
                        let head_pos = egui::pos2(bar_rect.min.x + fill_width, bar_rect.min.y + bar_height * 0.5);
                        b_painter.circle_filled(head_pos, 3.5, theme::accent_color_light());
                        b_painter.circle_filled(head_pos, 7.0, theme::with_alpha(theme::accent_color(), 60));
                    }

                    // Current Active Table Stream Log
                    if !wizard.current_table.is_empty() {
                        ui.add_space(theme::SPACE_MD);
                        
                        egui::Frame::NONE
                            .fill(theme::BG_DARK)
                            .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                            .inner_margin(Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.set_width(card_width - 24.0);
                                ui.horizontal(|ui| {
                                    // Pulsing active dot
                                    let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                                    let dot_center = dot_rect.center();
                                    let dot_pulse = 0.5 + 0.5 * (time * 4.0).sin();
                                    let dot_color = theme::with_alpha(theme::accent_color(), (80.0 + dot_pulse * 175.0) as u8);
                                    ui.painter().circle_filled(dot_center, 2.5 + dot_pulse * 1.2, dot_color);

                                    ui.add_space(theme::SPACE_XS);
                                    ui.label(
                                        RichText::new(format!("Exporting: {}", wizard.current_table))
                                            .monospace()
                                            .size(10.5)
                                            .color(theme::accent_color_light())
                                    );
                                });
                            });
                    }
                });

            ui.add_space(theme::SPACE_LG);
        } else if wizard.completed {
            if let Some(ref err) = wizard.error {
                ui.add_space(theme::SPACE_LG);
                ui.label(
                    RichText::new("⚠️")
                        .size(40.0),
                );
                ui.add_space(theme::SPACE_MD);
                ui.label(
                    RichText::new(translate("백업 생성 중 오류가 발생하였습니다.", "An error occurred during backup creation."))
                        .strong()
                        .color(theme::ACCENT_RED)
                        .size(14.0),
                );
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(err)
                        .size(12.0)
                        .color(theme::text_secondary()),
                );
                ui.add_space(theme::SPACE_LG);
                
                let retry = ui.button(translate("다시 시도", "Retry"));
                if retry.clicked() {
                    wizard.running = false;
                    wizard.completed = false;
                }
            } else {
                ui.add_space(theme::SPACE_LG);
                ui.label(
                    RichText::new("✓")
                        .size(40.0)
                        .color(theme::accent_color())
                        .strong(),
                );
                ui.add_space(theme::SPACE_MD);
                ui.label(
                    RichText::new(translate("백업 성공!", "Backup Completed Successfully!"))
                        .strong()
                        .color(theme::accent_color())
                        .size(16.0),
                );
                ui.add_space(theme::SPACE_LG);

                // Show details from last record in history if matches
                if let Some(record) = &last_record {
                    egui::Grid::new("backup_success_details_grid")
                        .num_columns(2)
                        .spacing([theme::SPACE_LG, theme::SPACE_MD])
                        .show(ui, |ui| {
                            ui.label(RichText::new(translate("저장 경로:", "Output Path:")).color(theme::text_secondary()));
                            ui.label(RichText::new(record.file_path.display().to_string()).color(theme::text_primary()).monospace());
                            ui.end_row();

                            ui.label(RichText::new(translate("파일 크기:", "File Size:")).color(theme::text_secondary()));
                            let mb = record.size_bytes as f64 / 1024.0 / 1024.0;
                            ui.label(RichText::new(format!("{:.2} MB ({} bytes)", mb, record.size_bytes)).color(theme::text_primary()));
                            ui.end_row();

                            ui.label(RichText::new(translate("소요 시간:", "Time Elapsed:")).color(theme::text_secondary()));
                            ui.label(RichText::new(format!("{:.2}s", record.duration_ms as f64 / 1000.0)).color(theme::text_primary()));
                            ui.end_row();
                        });
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Render Restore Confirmation Dialog
// ---------------------------------------------------------------------------

pub fn render_restore_confirm_dialog(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if state.restore_confirm_dialog.is_none() {
        return;
    }

    let mut open = true;
    let mut close_dialog = false;

    egui::Window::new(translate("백업 복원 확인", "Confirm Backup Restore"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .default_width(450.0)
        .min_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_XXL as i8)),
        )
        .show(ctx, |ui| {
            let dialog = state.restore_confirm_dialog.as_mut().unwrap();

            ui.vertical(|ui| {
                if !dialog.running && !dialog.completed {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠️").size(32.0).color(theme::ACCENT_YELLOW));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(translate(
                                    "데이터베이스를 정말 복원하시겠습니까?",
                                    "Are you sure you want to restore the database?",
                                ))
                                .strong()
                                .color(theme::text_primary())
                                .size(14.0),
                            );
                            ui.label(
                                RichText::new(translate(
                                    "경고: 이 백업 파일의 테이블 구조와 데이터가 스키마에 재배포됩니다.\n기존의 동일 이름 테이블이 존재한다면 삭제 후 재구성되므로 주의하세요!",
                                    "WARNING: Table structures and data from this backup will be redeployed.\nExisting tables with the same names will be overwritten!",
                                ))
                                .color(theme::ACCENT_RED)
                                .size(11.0),
                            );
                        });
                    });

                    ui.add_space(theme::SPACE_LG);
                    ui.separator();
                    ui.add_space(theme::SPACE_LG);

                    // Backup Details Grid
                    egui::Grid::new("restore_details_grid")
                        .num_columns(2)
                        .spacing([theme::SPACE_LG, theme::SPACE_MD])
                        .show(ui, |ui| {
                            ui.label(translate("백업 파일:", "Backup File:"));
                            ui.label(RichText::new(dialog.record.file_path.file_name().unwrap_or_default().to_string_lossy()).strong().color(theme::text_primary()));
                            ui.end_row();

                            ui.label(translate("백업 일시:", "Backup Date:"));
                            ui.label(RichText::new(&dialog.record.completed_at).color(theme::text_secondary()));
                            ui.end_row();

                            ui.label(translate("백업 형식:", "Backup Format:"));
                            ui.label(RichText::new(dialog.record.format.label()).color(theme::text_secondary()));
                            ui.end_row();

                            ui.label(translate("파일 크기:", "File Size:"));
                            let mb = dialog.record.size_bytes as f64 / 1024.0 / 1024.0;
                            ui.label(RichText::new(format!("{:.2} MB", mb)).color(theme::text_secondary()));
                            ui.end_row();
                        });

                    ui.add_space(theme::SPACE_XL);
                    ui.separator();
                    ui.add_space(theme::SPACE_MD);

                    // Actions
                    ui.horizontal(|ui| {
                        let cancel_btn = ui.button(translate("취소", "Cancel"));
                        if cancel_btn.clicked() {
                            close_dialog = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let restore_btn = ui.add(
                                egui::Button::new(RichText::new(translate("복원 시작", "Start Restore")).color(theme::text_primary()))
                                    .fill(theme::ACCENT_RED)
                                    .min_size(egui::vec2(100.0, 24.0)),
                            );

                            if restore_btn.clicked() {
                                dialog.running = true;
                                dialog.completed = false;
                                dialog.error = None;

                                // Active Connection Lookup
                                let active_conn_id = state.active_connection.unwrap_or(dialog.record.conn_id);
                                let config_opt = state.connections.get(&active_conn_id)
                                    .map(|c| c.config.clone())
                                    .or_else(|| state.saved_connections.iter().find(|c| c.id == active_conn_id).cloned())
                                    .or_else(|| state.saved_connections.iter().find(|c| c.id == dialog.record.conn_id).cloned());

                                if let Some(config) = config_opt {
                                    bridge.send(DbCommand::RunRestore {
                                        conn_id: active_conn_id,
                                        config,
                                        file_path: dialog.record.file_path.clone(),
                                    });
                                } else {
                                    dialog.running = false;
                                    dialog.completed = true;
                                    dialog.error = Some(translate(
                                        "연결 정보를 찾을 수 없습니다. 다시 접속 후 시도해 주세요.",
                                        "Connection config not found. Please reconnect and try again.",
                                    ));
                                }
                            }
                        });
                    });
                } else if dialog.running {
                    ui.add_space(theme::SPACE_XXL);
                    ui.spinner();
                    ui.add_space(theme::SPACE_MD);
                    ui.vertical_centered_justified(|ui| {
                        ui.label(
                            RichText::new(translate(
                                "데이터베이스 복원이 진행 중입니다...",
                                "Restoring database in progress...",
                            ))
                            .strong()
                            .size(14.0)
                            .color(theme::text_primary()),
                        );
                        ui.add_space(theme::SPACE_SM);
                        ui.label(
                            RichText::new(translate(
                                "테이블 생성 및 대용량 스트리밍 복원 데이터 주입 중...",
                                "Creating tables & injecting large streaming copy data...",
                            ))
                            .size(11.0)
                            .color(theme::text_muted()),
                        );
                    });
                    ui.add_space(theme::SPACE_XXL);
                } else if dialog.completed {
                    ui.vertical_centered_justified(|ui| {
                        if let Some(ref err) = dialog.error {
                            ui.label(RichText::new("⚠️").size(40.0));
                            ui.add_space(theme::SPACE_MD);
                            ui.label(
                                RichText::new(translate("복원 중 오류가 발생하였습니다.", "An error occurred during restore."))
                                    .strong()
                                    .color(theme::ACCENT_RED)
                                    .size(14.0),
                            );
                            ui.add_space(theme::SPACE_SM);
                            ui.label(RichText::new(err).color(theme::text_secondary()).size(12.0));
                        } else {
                            ui.label(
                                RichText::new("✓")
                                    .size(40.0)
                                    .color(theme::accent_color())
                                    .strong(),
                            );
                            ui.add_space(theme::SPACE_MD);
                            ui.label(
                                RichText::new(translate("성공적으로 복원되었습니다!", "Database Restored Successfully!"))
                                    .strong()
                                    .color(theme::accent_color())
                                    .size(16.0),
                            );
                            ui.add_space(theme::SPACE_SM);
                            ui.label(
                                RichText::new(translate(
                                    "구조 DDL, 레코드 데이터 및 외래 키 제약 조건이 완벽히 복구되었습니다.",
                                    "DDL structure, records, and FK constraints have been fully restored.",
                                ))
                                .size(11.0)
                                .color(theme::text_muted()),
                            );
                        }

                        ui.add_space(theme::SPACE_XL);
                        let btn = ui.button(translate("닫기", "Close"));
                        if btn.clicked() {
                            close_dialog = true;
                        }
                    });
                }
            });
        });

    if !open || close_dialog {
        state.restore_confirm_dialog = None;
    }
}
