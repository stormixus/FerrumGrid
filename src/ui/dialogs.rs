use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::connection_url::{parse_postgres_connection_url, PostgresConnectionUrl};
use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{AppState, ConnectionDialogState, ConnectionState, ConnectionStatus};
use crate::ui::theme;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_connection_dialog(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.show_connection_dialog {
        return;
    }

    if state.connection_dialog.clipboard_import_enabled
        && state.connection_dialog.editing_id.is_none()
    {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }

    let mut open = true;

    egui::Window::new(t("connection_dialog_title"))
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .min_width(440.0)
        .max_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_XXL as i8)),
        )
        .show(ctx, |ui| {
            render_dialog_content(ui, state, bridge);
        });

    if !open {
        state.show_connection_dialog = false;
    }
}

// ---------------------------------------------------------------------------
// Dialog body
// ---------------------------------------------------------------------------

fn render_dialog_content(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    detect_clipboard_import(&mut state.connection_dialog);

    if !state.saved_connections.is_empty() {
        render_saved_connections(ui, state);
        ui.add_space(theme::SPACE_MD);
        ui.painter().hline(
            ui.available_rect_before_wrap().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0, theme::border_subtle()),
        );
        ui.add_space(theme::SPACE_MD);
    }

    render_clipboard_import_prompt(ui, &mut state.connection_dialog);

    ui.label(
        RichText::new(t("connection_details"))
            .color(theme::text_secondary())
            .size(11.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_MD);

    let dialog = &mut state.connection_dialog;
    render_form_fields(ui, dialog);

    ui.add_space(theme::SPACE_MD);
    if let Some(ref result) = dialog.test_result.clone() {
        render_test_result(ui, result);
        ui.add_space(theme::SPACE_SM);
    }

    if dialog.testing {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                RichText::new(t("connection_testing"))
                    .color(theme::text_muted())
                    .size(12.0),
            );
        });
        ui.add_space(theme::SPACE_SM);
    }

    ui.add_space(theme::SPACE_MD);
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().top(),
        Stroke::new(1.0, theme::border_subtle()),
    );
    ui.add_space(theme::SPACE_LG);

    render_action_buttons(ui, state, bridge);
}

// ---------------------------------------------------------------------------
// Clipboard PostgreSQL URL import
// ---------------------------------------------------------------------------

fn detect_clipboard_import(dialog: &mut ConnectionDialogState) {
    if !dialog.clipboard_import_enabled || dialog.editing_id.is_some() {
        return;
    }

    let now = std::time::Instant::now();
    if dialog
        .last_clipboard_scan
        .is_some_and(|last| now.duration_since(last) < std::time::Duration::from_millis(500))
    {
        return;
    }
    dialog.last_clipboard_scan = Some(now);

    let Some(text) = read_clipboard_text() else {
        return;
    };
    if dialog.last_clipboard_text.as_deref() == Some(text.as_str()) {
        return;
    }
    dialog.last_clipboard_text = Some(text.clone());

    if let Some(candidate) = parse_postgres_connection_url(&text) {
        if dialog.pending_clipboard_import.as_ref() != Some(&candidate) {
            dialog.pending_clipboard_import = Some(candidate);
        }
    }
}

fn read_clipboard_text() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

fn render_clipboard_import_prompt(ui: &mut egui::Ui, dialog: &mut ConnectionDialogState) {
    let Some(candidate) = dialog.pending_clipboard_import.clone() else {
        return;
    };

    let mut action = ClipboardImportAction::None;
    let password_label = if candidate.has_password() {
        t("connection_clipboard_password_present")
    } else {
        t("connection_clipboard_password_empty")
    };

    egui::Frame::new()
        .fill(theme::bg_medium())
        .stroke(Stroke::new(1.0, theme::border_default()))
        .corner_radius(CornerRadius::same(theme::RADIUS_LG))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                egui::Frame::new()
                    .fill(theme::with_alpha(theme::accent_color(), 22))
                    .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                    .inner_margin(Margin::same(theme::SPACE_SM as i8))
                    .show(ui, |ui| {
                        crate::ui::icon_img(
                            ui,
                            crate::ui::icons_svg::CONNECTION,
                            "clipboard_postgres_url",
                            16.0,
                        );
                    });
                ui.add_space(theme::SPACE_MD);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t("connection_clipboard_title"))
                            .color(theme::text_primary())
                            .size(13.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(t("connection_clipboard_message"))
                            .color(theme::text_secondary())
                            .size(11.0),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            egui::vec2(62.0, 30.0),
                            theme::primary_button(&t("connection_clipboard_apply")),
                        )
                        .clicked()
                    {
                        action = ClipboardImportAction::Apply;
                    }
                    if ui
                        .add_sized(
                            egui::vec2(62.0, 30.0),
                            theme::ghost_button(&t("connection_clipboard_ignore")),
                        )
                        .clicked()
                    {
                        action = ClipboardImportAction::Ignore;
                    }
                });
            });

            ui.add_space(theme::SPACE_MD);
            ui.horizontal_wrapped(|ui| {
                clipboard_import_field(
                    ui,
                    t("connection_host"),
                    format!("{}:{}", candidate.host, candidate.port),
                    true,
                );
                clipboard_import_field(
                    ui,
                    t("connection_database"),
                    candidate.database.clone(),
                    true,
                );
                clipboard_import_field(
                    ui,
                    t("connection_username"),
                    candidate.username.clone(),
                    true,
                );
                clipboard_import_field(ui, t("connection_password"), password_label.clone(), false);
                clipboard_import_field(
                    ui,
                    t("connection_use_tls"),
                    if candidate.use_tls {
                        t("connection_encrypted")
                    } else {
                        t("connection_unencrypted")
                    },
                    false,
                );
            });
        });

    ui.add_space(theme::SPACE_MD);

    match action {
        ClipboardImportAction::Apply => {
            apply_clipboard_import(dialog, &candidate);
            dialog.pending_clipboard_import = None;
        }
        ClipboardImportAction::Ignore => {
            dialog.pending_clipboard_import = None;
        }
        ClipboardImportAction::None => {}
    }
}

fn clipboard_import_field(ui: &mut egui::Ui, label: String, value: String, monospace: bool) {
    egui::Frame::new()
        .fill(theme::bg_dark())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(
            theme::SPACE_MD as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.set_min_width(104.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(label)
                        .color(theme::text_muted())
                        .size(9.5)
                        .strong(),
                );
                let text = RichText::new(value)
                    .color(theme::text_secondary())
                    .size(11.0);
                if monospace {
                    ui.label(text.font(egui::FontId::monospace(11.0)));
                } else {
                    ui.label(text);
                }
            });
        });
}

fn apply_clipboard_import(dialog: &mut ConnectionDialogState, candidate: &PostgresConnectionUrl) {
    dialog.display_name = candidate.suggested_display_name();
    dialog.host = candidate.host.clone();
    dialog.port = candidate.port.to_string();
    dialog.database = candidate.database.clone();
    dialog.username = candidate.username.clone();
    dialog.password = candidate.password.clone();
    dialog.use_tls = candidate.use_tls;
    dialog.test_result = None;
}

enum ClipboardImportAction {
    None,
    Apply,
    Ignore,
}

// ---------------------------------------------------------------------------
// Saved connections list
// ---------------------------------------------------------------------------

fn render_saved_connections(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new(t("connection_saved"))
            .color(theme::text_secondary())
            .size(11.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_SM);

    let frame = egui::Frame::new()
        .fill(theme::bg_dark())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_SM as i8));

    frame.show(ui, |ui| {
        egui::ScrollArea::vertical()
            .max_height(130.0)
            .id_salt("saved_conns")
            .show(ui, |ui| {
                let mut load_idx: Option<usize> = None;
                let mut delete_idx: Option<usize> = None;
                let count = state.saved_connections.len();

                // 인덱스를 group 라벨 별로 묶는다 (첫 등장 순서로 수집 후 정렬:
                // 이름 있는 그룹은 알파벳, 미분류는 맨 뒤).
                let mut groups: Vec<(Option<String>, Vec<usize>)> = Vec::new();
                for i in 0..count {
                    let g = state.saved_connections[i]
                        .group
                        .clone()
                        .filter(|s| !s.trim().is_empty());
                    if let Some(slot) = groups.iter_mut().find(|(name, _)| *name == g) {
                        slot.1.push(i);
                    } else {
                        groups.push((g, vec![i]));
                    }
                }
                groups.sort_by(|a, b| match (&a.0, &b.0) {
                    (Some(x), Some(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                });

                let has_named_group = groups.iter().any(|(name, _)| name.is_some());
                for (group_name, indices) in &groups {
                    match group_name {
                        Some(name) if has_named_group => {
                            egui::CollapsingHeader::new(
                                RichText::new(name)
                                    .color(theme::text_secondary())
                                    .size(11.0)
                                    .strong(),
                            )
                            .id_salt(("conn_group", name.as_str()))
                            .default_open(true)
                            .show(ui, |ui| {
                                for &i in indices {
                                    render_saved_connection_row(
                                        ui,
                                        state,
                                        i,
                                        &mut load_idx,
                                        &mut delete_idx,
                                    );
                                }
                            });
                        }
                        _ => {
                            for &i in indices {
                                render_saved_connection_row(
                                    ui,
                                    state,
                                    i,
                                    &mut load_idx,
                                    &mut delete_idx,
                                );
                            }
                        }
                    }
                }

                if let Some(i) = load_idx {
                    state.connection_dialog =
                        ConnectionDialogState::from_config(&state.saved_connections[i]);
                }
                if let Some(i) = delete_idx {
                    state.saved_connections.remove(i);
                    save_vault_connections(state);
                }
            });
    });
}

/// 저장된 커넥션 한 행 렌더 (그룹 헤더 안/밖 공용). 클릭 시 `load_idx`,
/// 삭제 아이콘 클릭 시 `delete_idx` 설정.
fn render_saved_connection_row(
    ui: &mut egui::Ui,
    state: &AppState,
    i: usize,
    load_idx: &mut Option<usize>,
    delete_idx: &mut Option<usize>,
) {
    let name = state.saved_connections[i].display_name.clone();
    let host = state.saved_connections[i].host.clone();
    let port = state.saved_connections[i].port;
    let database = state.saved_connections[i].database.clone();

    egui::Frame::new()
        .inner_margin(Margin::symmetric(
            theme::SPACE_MD as i8,
            theme::SPACE_SM as i8,
        ))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::DATABASE, "saved_db", 10.0);
                ui.add_space(2.0);

                let resp = ui.add(
                    egui::Label::new(
                        RichText::new(&name).color(theme::text_primary()).size(12.0),
                    )
                    .sense(egui::Sense::click()),
                );
                if resp.clicked() {
                    *load_idx = Some(i);
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.label(
                    RichText::new(format!("{}:{}/{}", host, port, database))
                        .color(theme::text_muted())
                        .size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let del_resp = ui.add(
                        egui::Button::new("")
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE),
                    );
                    ui.allocate_new_ui(
                        egui::UiBuilder::new().max_rect(del_resp.rect),
                        |ui| {
                            crate::ui::icon_img(
                                ui,
                                crate::ui::icons_svg::CLOSE,
                                "del_conn",
                                10.0,
                            );
                        },
                    );

                    if del_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if del_resp.clicked() {
                        *delete_idx = Some(i);
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Form fields
// ---------------------------------------------------------------------------

fn render_form_fields(ui: &mut egui::Ui, dialog: &mut ConnectionDialogState) {
    // URL/DSN 빠른 입력 — 폼과 양방향 변환.
    ui.label(
        RichText::new(t("connection_url"))
            .color(theme::text_muted())
            .size(12.0),
    );
    ui.add(
        theme::mono_text_input(&mut dialog.url_input)
            .hint_text("postgres://user:pass@host:5432/db")
            .desired_width(f32::INFINITY),
    );
    ui.add_space(theme::SPACE_XS);
    ui.horizontal(|ui| {
        if ui.add(theme::secondary_button(&t("connection_url_apply"))).clicked() {
            if let Some(parsed) =
                crate::connection_url::parse_postgres_connection_url(&dialog.url_input)
            {
                dialog.host = parsed.host;
                dialog.port = parsed.port.to_string();
                dialog.database = parsed.database;
                dialog.username = parsed.username;
                if !parsed.password.is_empty() {
                    dialog.password = parsed.password;
                }
                dialog.use_tls = parsed.use_tls;
            }
        }
        if ui.add(theme::secondary_button(&t("connection_url_from_form"))).clicked() {
            dialog.url_input = crate::connection_url::PostgresConnectionUrl {
                host: dialog.host.clone(),
                port: dialog.port.parse().unwrap_or(5432),
                database: dialog.database.clone(),
                username: dialog.username.clone(),
                password: dialog.password.clone(),
                use_tls: dialog.use_tls,
            }
            .to_url();
        }
    });
    ui.add_space(theme::SPACE_SM);

    egui::Grid::new("conn_fields")
        .num_columns(2)
        .min_col_width(80.0)
        .spacing([theme::SPACE_LG, theme::SPACE_MD])
        .show(ui, |ui| {
            field_label(ui, t("connection_name"));
            ui.add(
                theme::text_input(&mut dialog.display_name)
                    .hint_text("My Database")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_group"));
            ui.add(
                theme::text_input(&mut dialog.group)
                    .hint_text("dev / staging / prod")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_host"));
            ui.add(
                theme::mono_text_input(&mut dialog.host)
                    .hint_text("localhost")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_port"));
            ui.add(
                theme::mono_text_input(&mut dialog.port)
                    .hint_text("5432")
                    .desired_width(92.0),
            );
            ui.end_row();

            field_label(ui, t("connection_database"));
            ui.add(
                theme::mono_text_input(&mut dialog.database)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_username"));
            ui.add(
                theme::mono_text_input(&mut dialog.username)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_password"));
            ui.horizontal(|ui| {
                let response = ui.add(
                    theme::password_input(&mut dialog.password, dialog.show_password)
                        .hint_text(
                            "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                        )
                        .desired_width(f32::INFINITY),
                );
                if response.changed() {
                    crate::korean_keyboard::normalize_password_input(&mut dialog.password);
                }
                let toggle_label = if dialog.show_password { "Hide" } else { "Show" };
                if ui
                    .add_sized(
                        egui::vec2(54.0, theme::INPUT_HEIGHT),
                        egui::Button::new(
                            RichText::new(toggle_label)
                                .color(theme::text_secondary())
                                .size(11.0),
                        )
                        .fill(theme::bg_light())
                        .stroke(Stroke::new(1.0, theme::border_default())),
                    )
                    .clicked()
                {
                    dialog.show_password = !dialog.show_password;
                }
            });
            ui.end_row();

            field_label(ui, t("connection_use_tls"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut dialog.use_tls, "");
                if dialog.use_tls {
                    crate::ui::icon_img(ui, crate::ui::icons_svg::CONNECTION, "tls_locked", 10.0);
                    ui.label(
                        RichText::new(t("connection_encrypted"))
                            .color(theme::ACCENT_GREEN)
                            .size(11.0),
                    );
                } else {
                    crate::ui::icon_img(ui, crate::ui::icons_svg::BACKUP, "tls_unlocked", 10.0);
                    ui.label(
                        RichText::new(t("connection_unencrypted"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                }
            });
            ui.end_row();

            if dialog.use_tls {
                field_label(ui, t("connection_sslmode"));
                egui::ComboBox::from_id_salt("conn_sslmode")
                    .selected_text(&dialog.sslmode)
                    .show_ui(ui, |ui| {
                        for m in ["require", "verify-ca", "verify-full"] {
                            ui.selectable_value(&mut dialog.sslmode, m.to_string(), m);
                        }
                    });
                ui.end_row();

                cert_file_row(ui, t("connection_ssl_root_cert"), &mut dialog.ssl_root_cert);
                cert_file_row(ui, t("connection_ssl_client_cert"), &mut dialog.ssl_client_cert);
                cert_file_row(ui, t("connection_ssl_client_key"), &mut dialog.ssl_client_key);
            }

            field_label(ui, t("connection_guardrails"));
            ui.vertical(|ui| {
                ui.checkbox(&mut dialog.read_only, t("connection_read_only"));
                ui.checkbox(&mut dialog.is_production, t("connection_production"));
            });
            ui.end_row();

            field_label(ui, t("connection_ssh_tunnel"));
            ui.checkbox(&mut dialog.ssh_enabled, t("connection_ssh_enable"));
            ui.end_row();

            if dialog.ssh_enabled {
                field_label(ui, t("connection_ssh_host"));
                ui.horizontal(|ui| {
                    ui.add(
                        theme::mono_text_input(&mut dialog.ssh_host)
                            .hint_text("bastion.example.com")
                            .desired_width(150.0),
                    );
                    ui.label(RichText::new(":").color(theme::text_muted()));
                    ui.add(
                        theme::mono_text_input(&mut dialog.ssh_port)
                            .hint_text("22")
                            .desired_width(50.0),
                    );
                });
                ui.end_row();

                field_label(ui, t("connection_ssh_user"));
                ui.add(
                    theme::mono_text_input(&mut dialog.ssh_user)
                        .hint_text("ec2-user")
                        .desired_width(f32::INFINITY),
                );
                ui.end_row();

                cert_file_row(ui, t("connection_ssh_key"), &mut dialog.ssh_key);
            }
        });
}

/// 인증서 파일 경로 입력 행: 텍스트 필드 + 파일 선택(…) + 지우기(×).
fn cert_file_row(ui: &mut egui::Ui, label: String, value: &mut String) {
    field_label(ui, label);
    ui.horizontal(|ui| {
        ui.add(
            theme::mono_text_input(value)
                .hint_text("/path/to.pem")
                .desired_width(190.0),
        );
        if ui.small_button("\u{2026}").clicked() {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("PEM", &["pem", "crt", "cer", "key"])
                .pick_file()
            {
                *value = p.display().to_string();
            }
        }
        if !value.is_empty() && ui.small_button("\u{00d7}").clicked() {
            value.clear();
        }
    });
    ui.end_row();
}

fn field_label(ui: &mut egui::Ui, text: String) {
    ui.label(RichText::new(text).color(theme::text_muted()).size(12.0));
}

// ---------------------------------------------------------------------------
// Test result feedback
// ---------------------------------------------------------------------------

fn render_test_result(ui: &mut egui::Ui, result: &Result<String, String>) {
    match result {
        Ok(msg) => {
            egui::Frame::new()
                .fill(theme::with_alpha(theme::ACCENT_GREEN, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    theme::with_alpha(theme::ACCENT_GREEN, 80),
                ))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        crate::ui::icon_img(ui, crate::ui::icons_svg::SUCCESS, "test_ok", 12.0);
                        ui.add_space(4.0);
                        ui.label(RichText::new(msg).color(theme::ACCENT_GREEN).size(12.0));
                    });
                });
        }
        Err(msg) => {
            egui::Frame::new()
                .fill(theme::with_alpha(theme::ACCENT_RED, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    theme::with_alpha(theme::ACCENT_RED, 80),
                ))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        crate::ui::icon_img(ui, crate::ui::icons_svg::ERROR, "test_err", 12.0);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(msg)
                                .color(theme::accent_red_soft())
                                .size(12.0),
                        );
                    });
                });
        }
    }
}

// ---------------------------------------------------------------------------
// Action buttons
// ---------------------------------------------------------------------------

fn render_action_buttons(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let testing = state.connection_dialog.testing;

    ui.horizontal(|ui| {
        // Test — secondary style
        let test_btn = ui.add_enabled(
            !testing,
            theme::secondary_icon_button(
                crate::ui::icon_image(ui, crate::ui::icons_svg::REFRESH, "test_action", 12.0),
                t("connection_test"),
            )
            .min_size(egui::vec2(160.0, 32.0)),
        );

        if test_btn.clicked() {
            state.connection_dialog.testing = true;
            state.connection_dialog.test_result = None;
            let config = state.connection_dialog.to_config();
            bridge.send(DbCommand::Connect {
                conn_id: config.id,
                config,
            });
        }

        ui.add_space(theme::SPACE_SM);

        // Connect — copper primary
        let connect_btn = ui.add(
            theme::primary_icon_button(
                crate::ui::icon_image(ui, crate::ui::icons_svg::CONNECTION, "conn_action", 12.0),
                t("connection_connect"),
            )
            .min_size(egui::vec2(88.0, 32.0)),
        );

        if connect_btn.clicked() {
            do_connect(state, bridge);
        }

        // Cancel — ghost, right-aligned
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let cancel_btn = egui::Button::new(
                RichText::new(t("connection_cancel"))
                    .color(theme::text_muted())
                    .size(12.0),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE);

            if ui.add(cancel_btn).clicked() {
                state.show_connection_dialog = false;
            }
        });
    });
}

/// 프로덕션 연결에서 파괴적 문장 실행 전 typed 확인 모달.
pub fn render_prod_confirm_dialog(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    let Some(sql) = state.pending_prod_confirm.clone() else {
        return;
    };
    let mut run = false;
    let mut cancel = false;

    egui::Window::new(t("guard_prod_confirm_title"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("guard_prod_confirm_body"))
                    .color(theme::ACCENT_RED)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_SM);
            let preview: String = sql.chars().take(200).collect();
            ui.label(
                RichText::new(preview)
                    .monospace()
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_MD);
            ui.add(
                theme::mono_text_input(&mut state.prod_confirm_input)
                    .hint_text("production")
                    .desired_width(220.0),
            );
            ui.add_space(theme::SPACE_SM);
            let ok = state.prod_confirm_input.trim().eq_ignore_ascii_case("production");
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(ok, theme::secondary_button(&t("guard_prod_confirm_run")))
                    .clicked()
                {
                    run = true;
                }
                if ui.add(theme::secondary_button(&t("connection_cancel"))).clicked() {
                    cancel = true;
                }
            });
        });

    if run {
        if let Some(conn_id) = state.active_connection {
            state.query_running = true;
            state.last_error = None;
            bridge.send(DbCommand::ExecuteQuery {
                conn_id,
                sql,
                row_limit: Some(state.default_row_limit),
            });
        }
        state.pending_prod_confirm = None;
        state.prod_confirm_input.clear();
    }
    if cancel {
        state.pending_prod_confirm = None;
        state.prod_confirm_input.clear();
    }
}

// ---------------------------------------------------------------------------
// Connect action
// ---------------------------------------------------------------------------

fn do_connect(state: &mut AppState, bridge: &DbBridge) {
    let mut config = state.connection_dialog.to_config();
    let conn_id = config.id;

    if let Some(saved) = state.saved_connections.iter_mut().find(|c| {
        c.id == config.id
            || (c.host == config.host
                && c.port == config.port
                && c.database == config.database
                && c.username == config.username)
    }) {
        // 다이얼로그는 color_tag 를 편집하지 않으므로 기존 값 보존 (편집 저장 시
        // 색상 태그가 사라지던 버그 수정).
        config.color_tag = saved.color_tag.clone();
        *saved = config.clone();
    } else {
        state.saved_connections.push(config.clone());
    }
    save_vault_connections(state);

    let conn_state = ConnectionState::new(config.clone());
    state.connections.insert(conn_id, conn_state);
    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.status = ConnectionStatus::Connecting;
    }
    state.active_connection = Some(conn_id);
    state.status_message = format!("Connecting to {}\u{2026}", config.display_name);

    bridge.send(DbCommand::Connect { conn_id, config });
    state.show_connection_dialog = false;
}

fn save_vault_connections(state: &mut AppState) {
    let Some(session) = state.vault.session.as_ref() else {
        state.last_error = Some(t("vault_error_locked"));
        return;
    };

    if let Err(err) =
        crate::storage::connections::save_connections(&state.saved_connections, session)
    {
        state.last_error = Some(err.to_string());
        state.status_message = err.to_string();
    }
}
