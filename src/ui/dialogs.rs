use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

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
                .fill(theme::BG_MEDIUM)
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
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
    if !state.saved_connections.is_empty() {
        render_saved_connections(ui, state);
        ui.add_space(theme::SPACE_MD);
        ui.painter().hline(
            ui.available_rect_before_wrap().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0, theme::BORDER_SUBTLE),
        );
        ui.add_space(theme::SPACE_MD);
    }

    ui.label(
        RichText::new(t("connection_details"))
            .color(theme::TEXT_SECONDARY)
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
                    .color(theme::TEXT_MUTED)
                    .size(12.0),
            );
        });
        ui.add_space(theme::SPACE_SM);
    }

    ui.add_space(theme::SPACE_MD);
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().top(),
        Stroke::new(1.0, theme::BORDER_SUBTLE),
    );
    ui.add_space(theme::SPACE_LG);

    render_action_buttons(ui, state, bridge);
}

// ---------------------------------------------------------------------------
// Saved connections list
// ---------------------------------------------------------------------------

fn render_saved_connections(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new(t("connection_saved"))
            .color(theme::TEXT_SECONDARY)
            .size(11.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_SM);

    let frame = egui::Frame::new()
        .fill(theme::BG_DARK)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
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
                                        RichText::new(&name).color(theme::TEXT_PRIMARY).size(12.0),
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
                                        .color(theme::TEXT_MUTED)
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
                            Stroke::new(1.0, theme::BORDER_SUBTLE),
                        );
                    }
                }

                if let Some(i) = load_idx {
                    state.connection_dialog =
                        ConnectionDialogState::from_config(&state.saved_connections[i]);
                }
                if let Some(i) = delete_idx {
                    let removed = state.saved_connections.remove(i);
                    crate::storage::connections::delete_password(&removed.id);
                    crate::storage::connections::save_connections(&state.saved_connections);
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
                egui::TextEdit::singleline(&mut dialog.display_name)
                    .hint_text("My Database")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_host"));
            ui.add(
                egui::TextEdit::singleline(&mut dialog.host)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("localhost")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_port"));
            ui.add(
                egui::TextEdit::singleline(&mut dialog.port)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("5432")
                    .desired_width(72.0),
            );
            ui.end_row();

            field_label(ui, t("connection_database"));
            ui.add(
                egui::TextEdit::singleline(&mut dialog.database)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_username"));
            ui.add(
                egui::TextEdit::singleline(&mut dialog.username)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY),
            );
            ui.end_row();

            field_label(ui, t("connection_password"));
            ui.add(
                egui::TextEdit::singleline(&mut dialog.password)
                    .password(true)
                    .hint_text("\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}")
                    .desired_width(f32::INFINITY),
            );
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
                            .color(theme::TEXT_MUTED)
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
                        .color(theme::TEXT_DISABLED)
                        .size(11.0),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
            );
            ui.end_row();
        });
}

fn field_label(ui: &mut egui::Ui, text: String) {
    ui.label(RichText::new(text).color(theme::TEXT_MUTED).size(12.0));
}

// ---------------------------------------------------------------------------
// Test result feedback
// ---------------------------------------------------------------------------

fn render_test_result(ui: &mut egui::Ui, result: &Result<String, String>) {
    match result {
        Ok(msg) => {
            egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(78, 190, 100, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(78, 190, 100, 80),
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
                .fill(Color32::from_rgba_premultiplied(210, 70, 70, 20))
                .inner_margin(Margin::symmetric(
                    theme::SPACE_LG as i8,
                    theme::SPACE_SM as i8,
                ))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(210, 70, 70, 80),
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
                .fill(theme::BG_LIGHT)
                .stroke(Stroke::new(1.0, theme::BORDER_STRONG)),
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
                    .color(theme::TEXT_MUTED)
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

    if config.password.is_empty() {
        crate::storage::connections::delete_password(&config.id);
    } else {
        crate::storage::connections::store_password(&config.id, &config.password);
    }

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
    crate::storage::connections::save_connections(&state.saved_connections);

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
