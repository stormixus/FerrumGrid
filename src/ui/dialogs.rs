use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::{AppState, ConnectionDialogState, ConnectionState, ConnectionStatus};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, BtnKind, Tokens};

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn render_connection_dialog(
    ctx: &egui::Context,
    state: &mut AppState,
    bridge: &DbBridge,
) {
    if !state.show_connection_dialog {
        return;
    }

    let t = Tokens::current(ctx);
    let mut open = true;

    egui::Window::new("connection_dialog")
        .title_bar(false)
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .min_width(720.0)
        .max_width(720.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(t.bg_surface)
                .stroke(Stroke::new(1.0, t.border_default))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::ZERO)
                .shadow(egui::Shadow {
                    offset: [0, 12],
                    blur: 32,
                    spread: 0,
                    color: Color32::from_black_alpha(80),
                }),
        )
        .show(ctx, |ui| {
            render_dialog_header(ui, t, state);
            ui.painter().hline(
                ui.available_rect_before_wrap().x_range(),
                ui.cursor().top(),
                Stroke::new(1.0, t.border_subtle),
            );

            ui.horizontal_top(|ui| {
                render_saved_panel(ui, t, state);
                ui.painter().vline(
                    ui.cursor().left(),
                    ui.available_rect_before_wrap().y_range(),
                    Stroke::new(1.0, t.border_subtle),
                );
                render_form_panel(ui, t, state, bridge);
            });
        });

    if !open {
        state.show_connection_dialog = false;
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn render_dialog_header(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(t.bg_surface)
        .inner_margin(Margin::symmetric(theme::SPACE_XL_I, theme::SPACE_LG_I));
    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Connect to PostgreSQL")
                        .color(t.text_primary)
                        .size(16.0)
                        .strong(),
                );
                ui.label(
                    RichText::new("Credentials are stored in your OS keychain.")
                        .color(t.text_muted)
                        .size(11.0),
                );
            });
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if theme::icon_only_button(ui, Icon::Close, t, t.text_muted, 14.0)
                        .clicked()
                    {
                        state.show_connection_dialog = false;
                    }
                },
            );
        });
    });
}

// ---------------------------------------------------------------------------
// Left: saved connections panel (45%)
// ---------------------------------------------------------------------------

fn render_saved_panel(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let panel_w = 280.0;
    let frame = egui::Frame::new()
        .fill(t.bg_sidebar)
        .inner_margin(Margin::same(theme::SPACE_LG_I));

    egui::Resize::default()
        .fixed_size(egui::vec2(panel_w, 380.0))
        .show(ui, |ui| {
            frame.show(ui, |ui| {
                ui.set_min_width(panel_w - 24.0);

                ui.label(
                    RichText::new("SAVED")
                        .color(t.text_muted)
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(theme::SPACE_SM);

                if state.saved_connections.is_empty() {
                    ui.add_space(theme::SPACE_LG);
                    ui.vertical_centered(|ui| {
                        icons::icon(ui, Icon::Database, 28.0, t.text_disabled);
                        ui.add_space(theme::SPACE_SM);
                        ui.label(
                            RichText::new("No saved connections")
                                .color(t.text_muted)
                                .size(12.0),
                        );
                        ui.label(
                            RichText::new("Fill the form on the right")
                                .color(t.text_disabled)
                                .size(11.0),
                        );
                    });
                    return;
                }

                egui::ScrollArea::vertical()
                    .id_salt("saved_conns")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut load_idx: Option<usize> = None;
                        let mut delete_idx: Option<usize> = None;
                        let count = state.saved_connections.len();
                        for i in 0..count {
                            render_saved_row(
                                ui,
                                t,
                                &state.saved_connections[i],
                                &mut load_idx,
                                &mut delete_idx,
                                i,
                            );
                        }
                        if let Some(i) = load_idx {
                            state.connection_dialog =
                                ConnectionDialogState::from_config(
                                    &state.saved_connections[i],
                                );
                        }
                        if let Some(i) = delete_idx {
                            state.saved_connections.remove(i);
                        }
                    });
            });
        });
}

fn render_saved_row(
    ui: &mut egui::Ui,
    t: Tokens,
    config: &crate::types::ConnectionConfig,
    load_idx: &mut Option<usize>,
    delete_idx: &mut Option<usize>,
    i: usize,
) {
    let row_h = 44.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_h),
        Sense::click(),
    );
    let bg = if resp.hovered() {
        t.bg_elev
    } else {
        t.bg_surface
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_SM), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(1.0, t.border_subtle),
        egui::epaint::StrokeKind::Inside,
    );

    if resp.clicked() {
        *load_idx = Some(i);
    }

    // Status dot
    let dot_center = egui::pos2(
        rect.left() + theme::SPACE_LG,
        rect.center().y,
    );
    ui.painter().circle_filled(dot_center, 4.0, t.text_muted);

    ui.painter().text(
        egui::pos2(rect.left() + theme::SPACE_LG + 12.0, rect.top() + 8.0),
        egui::Align2::LEFT_TOP,
        &config.display_name,
        egui::FontId::proportional(13.0),
        t.text_primary,
    );
    ui.painter().text(
        egui::pos2(rect.left() + theme::SPACE_LG + 12.0, rect.top() + 24.0),
        egui::Align2::LEFT_TOP,
        format!("{}:{}/{}", config.host, config.port, config.database),
        egui::FontId::monospace(11.0),
        t.text_muted,
    );

    let del_rect = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 14.0, rect.center().y),
        egui::vec2(20.0, 20.0),
    );
    let del_resp = ui.interact(del_rect, ui.id().with(("del", i)), Sense::click());
    let del_color = if del_resp.hovered() {
        t.danger
    } else {
        t.text_muted
    };
    let del_icon_rect =
        egui::Rect::from_center_size(del_rect.center(), egui::vec2(12.0, 12.0));
    icons::icon_at(ui.painter(), Icon::Close, del_icon_rect, del_color);
    if del_resp.clicked() {
        *delete_idx = Some(i);
    }

    ui.add_space(theme::SPACE_XS);
}

// ---------------------------------------------------------------------------
// Right: form panel
// ---------------------------------------------------------------------------

fn render_form_panel(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
) {
    let frame = egui::Frame::new()
        .fill(t.bg_surface)
        .inner_margin(Margin::same(theme::SPACE_XL_I));

    frame.show(ui, |ui| {
        ui.set_min_width(420.0);

        ui.label(
            RichText::new("CONNECTION DETAILS")
                .color(t.text_muted)
                .size(10.0)
                .strong(),
        );
        ui.add_space(theme::SPACE_MD);

        let dialog = &mut state.connection_dialog;
        render_form_fields(ui, t, dialog);

        ui.add_space(theme::SPACE_MD);
        if let Some(ref result) = dialog.test_result.clone() {
            render_test_result(ui, t, result);
            ui.add_space(theme::SPACE_SM);
        }
        if dialog.testing {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    RichText::new("Testing connection\u{2026}")
                        .color(t.text_muted)
                        .size(12.0),
                );
            });
            ui.add_space(theme::SPACE_SM);
        }

        ui.add_space(theme::SPACE_MD);
        ui.painter().hline(
            ui.available_rect_before_wrap().x_range(),
            ui.cursor().top(),
            Stroke::new(1.0, t.border_subtle),
        );
        ui.add_space(theme::SPACE_LG);

        render_action_buttons(ui, t, state, bridge);
    });
}

fn render_form_fields(
    ui: &mut egui::Ui,
    t: Tokens,
    dialog: &mut ConnectionDialogState,
) {
    egui::Grid::new("conn_fields")
        .num_columns(2)
        .min_col_width(80.0)
        .spacing([theme::SPACE_LG, theme::SPACE_MD])
        .show(ui, |ui| {
            field_label(ui, t, "Name");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.display_name)
                    .hint_text("My Database")
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Host");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.host)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("localhost")
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Port");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.port)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("5432")
                    .desired_width(80.0)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Database");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.database)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Username");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.username)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("postgres")
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Password");
            ui.add(
                egui::TextEdit::singleline(&mut dialog.password)
                    .password(true)
                    .hint_text(
                        "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}",
                    )
                    .desired_width(f32::INFINITY)
                    .margin(egui::vec2(8.0, 4.0)),
            );
            ui.end_row();

            field_label(ui, t, "Use TLS");
            ui.horizontal(|ui| {
                ui.checkbox(&mut dialog.use_tls, "");
                if dialog.use_tls {
                    ui.label(
                        RichText::new("Encrypted")
                            .color(t.success)
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        RichText::new("Unencrypted")
                            .color(t.text_muted)
                            .size(11.0),
                    );
                }
            });
            ui.end_row();

            field_label(ui, t, "SSH Tunnel");
            ui.add_enabled(
                false,
                egui::Button::new(
                    RichText::new("Coming soon")
                        .color(t.text_disabled)
                        .size(11.0),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, t.border_subtle)),
            );
            ui.end_row();
        });
}

fn field_label(ui: &mut egui::Ui, t: Tokens, text: &str) {
    ui.label(RichText::new(text).color(t.text_secondary).size(12.0));
}

fn render_test_result(ui: &mut egui::Ui, t: Tokens, result: &Result<String, String>) {
    let (color, icon_kind, msg): (Color32, Icon, &str) = match result {
        Ok(m) => (t.success, Icon::Check, m.as_str()),
        Err(m) => (t.danger, Icon::ErrorMark, m.as_str()),
    };
    let bg = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 24);
    egui::Frame::new()
        .fill(bg)
        .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_SM_I))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 80),
        ))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                icons::icon(ui, icon_kind, 14.0, color);
                ui.label(RichText::new(msg).color(color).size(12.0));
            });
        });
}

fn render_action_buttons(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
) {
    let testing = state.connection_dialog.testing;

    ui.horizontal(|ui| {
        if theme::icon_button(ui, BtnKind::Secondary, Icon::Connection, "Test", t, !testing)
            .clicked()
        {
            state.connection_dialog.testing = true;
            state.connection_dialog.test_result = None;
            let config = state.connection_dialog.to_config();
            bridge.send(DbCommand::Connect {
                conn_id: config.id,
                config,
            });
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::icon_button(ui, BtnKind::Primary, Icon::Connection, "Connect", t, true)
                .clicked()
            {
                do_connect(state, bridge);
            }

            ui.add_space(theme::SPACE_SM);

            let cancel_btn = theme::ghost_button(t, "Cancel");
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

    if !state.saved_connections.iter().any(|c| {
        c.host == config.host
            && c.database == config.database
            && c.username == config.username
    }) {
        state.saved_connections.push(config.clone());
    }

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
