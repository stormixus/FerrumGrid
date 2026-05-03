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
        render_test_result(ui, &result);
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
    if dialog.clipboard_import_checked || dialog.editing_id.is_some() {
        return;
    }

    dialog.clipboard_import_checked = true;

    let Some(text) = read_clipboard_text() else {
        return;
    };

    if let Some(candidate) = parse_postgres_connection_url(&text) {
        dialog.pending_clipboard_import = Some(candidate);
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
        .fill(theme::with_alpha(theme::ACCENT_TEAL, 18))
        .stroke(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 86)))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_LG as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                crate::ui::icon_img(
                    ui,
                    crate::ui::icons_svg::CONNECTION,
                    "clipboard_postgres_url",
                    14.0,
                );
                ui.add_space(theme::SPACE_SM);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(t("connection_clipboard_title"))
                            .color(theme::text_primary())
                            .size(12.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(t("connection_clipboard_message"))
                            .color(theme::text_secondary())
                            .size(11.0),
                    );
                });
            });

            ui.add_space(theme::SPACE_SM);
            ui.horizontal_wrapped(|ui| {
                clipboard_import_chip(ui, format!("{}:{}", candidate.host, candidate.port));
                clipboard_import_chip(ui, candidate.database.clone());
                clipboard_import_chip(ui, candidate.username.clone());
                clipboard_import_chip(ui, password_label.clone());
                if candidate.use_tls {
                    clipboard_import_chip(ui, "TLS".to_string());
                }
            });

            ui.add_space(theme::SPACE_SM);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 28.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui
                        .add(theme::primary_button(&t("connection_clipboard_apply")))
                        .clicked()
                    {
                        action = ClipboardImportAction::Apply;
                    }
                    if ui
                        .add(theme::ghost_button(&t("connection_clipboard_ignore")))
                        .clicked()
                    {
                        action = ClipboardImportAction::Ignore;
                    }
                },
            );
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

fn clipboard_import_chip(ui: &mut egui::Ui, text: String) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 18))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::symmetric(
            theme::SPACE_MD as i8,
            theme::SPACE_XS as i8,
        ))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .color(theme::text_secondary())
                    .font(egui::FontId::monospace(11.0)),
            );
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

                for i in 0..count {
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
                                crate::ui::icon_img(
                                    ui,
                                    crate::ui::icons_svg::DATABASE,
                                    "saved_db",
                                    10.0,
                                );
                                ui.add_space(2.0);

                                let resp = ui.add(
                                    egui::Label::new(
                                        RichText::new(&name)
                                            .color(theme::text_primary())
                                            .size(12.0),
                                    )
                                    .sense(egui::Sense::click()),
                                );
                                if resp.clicked() {
                                    load_idx = Some(i);
                                }
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                ui.label(
                                    RichText::new(format!("{}:{}/{}", host, port, database))
                                        .color(theme::text_muted())
                                        .size(11.0),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
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
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if del_resp.clicked() {
                                            delete_idx = Some(i);
                                        }
                                    },
                                );
                            });
                        });

                    if i < count - 1 {
                        ui.painter().hline(
                            ui.available_rect_before_wrap().x_range(),
                            ui.cursor().top(),
                            Stroke::new(1.0, theme::border_subtle()),
                        );
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

// ---------------------------------------------------------------------------
// Form fields
// ---------------------------------------------------------------------------

fn render_form_fields(ui: &mut egui::Ui, dialog: &mut ConnectionDialogState) {
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

            field_label(ui, t("connection_ssh_tunnel"));
            ui.add_enabled(
                false,
                egui::Button::new(
                    RichText::new(t("connection_coming_soon"))
                        .color(theme::text_disabled())
                        .size(11.0),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, theme::border_subtle())),
            );
            ui.end_row();
        });
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
                .fill(Color32::from_rgba_unmultiplied(78, 190, 100, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(78, 190, 100, 80),
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
                .fill(Color32::from_rgba_unmultiplied(210, 70, 70, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(210, 70, 70, 80),
                ))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        crate::ui::icon_img(ui, crate::ui::icons_svg::ERROR, "test_err", 12.0);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(msg)
                                .color(Color32::from_rgb(220, 150, 150))
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
            egui::Button::new(format!("      {}", t("connection_test")))
                .fill(theme::bg_light())
                .stroke(Stroke::new(1.0, theme::border_strong())),
        );
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                test_btn
                    .rect
                    .shrink2(egui::vec2(test_btn.rect.width() - 24.0, 0.0)),
            ),
            |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::REFRESH, "test_action", 12.0);
            },
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
        let connect_btn = ui.add(theme::primary_button(&format!(
            "      {}",
            t("connection_connect")
        )));
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                connect_btn
                    .rect
                    .shrink2(egui::vec2(connect_btn.rect.width() - 24.0, 0.0)),
            ),
            |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::CONNECTION, "conn_action", 12.0);
            },
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

// ---------------------------------------------------------------------------
// Connect action
// ---------------------------------------------------------------------------

fn do_connect(state: &mut AppState, bridge: &DbBridge) {
    let config = state.connection_dialog.to_config();
    let conn_id = config.id;

    if let Some(saved) = state.saved_connections.iter_mut().find(|c| {
        c.id == config.id
            || (c.host == config.host
                && c.port == config.port
                && c.database == config.database
                && c.username == config.username)
    }) {
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
