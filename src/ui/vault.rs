use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};
use zeroize::Zeroize;

use crate::i18n::t;
use crate::state::{AppState, VaultStatus};
use crate::ui::theme;

pub fn render_vault_window(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::bg_dark()))
        .show(ctx, |_ui| {});

    egui::Window::new(t("vault_title"))
        .resizable(false)
        .collapsible(false)
        .min_width(460.0)
        .max_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_XXL as i8)),
        )
        .show(ctx, |ui| {
            render_header(ui, state);
            ui.add_space(theme::SPACE_XL);
            render_body(ui, state);
        });
}

fn render_header(ui: &mut egui::Ui, state: &AppState) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(theme::with_alpha(theme::accent_color(), 24))
            .stroke(Stroke::new(1.0, theme::with_alpha(theme::accent_color(), 96)))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_MD as i8))
            .show(ui, |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::BACKUP, "vault_lock", 22.0);
            });

        ui.add_space(theme::SPACE_MD);
        ui.vertical(|ui| {
            let title = match state.vault.status {
                VaultStatus::SetupRequired => t("vault_setup_title"),
                VaultStatus::Locked => t("vault_unlock_title"),
                VaultStatus::Unlocked => t("vault_unlocked_title"),
            };
            ui.label(
                RichText::new(title)
                    .color(theme::text_primary())
                    .size(18.0)
                    .strong(),
            );
            ui.label(
                RichText::new(t("vault_subtitle"))
                    .color(theme::text_secondary())
                    .size(12.0),
            );
        });
    });
}

fn render_body(ui: &mut egui::Ui, state: &mut AppState) {
    let status = state.vault.status.clone();
    match status {
        VaultStatus::SetupRequired => render_setup(ui, state),
        VaultStatus::Locked => render_unlock(ui, state),
        VaultStatus::Unlocked => {}
    }
}

fn render_setup(ui: &mut egui::Ui, state: &mut AppState) {
    let legacy_count = state.vault.legacy_connections.len();
    if legacy_count > 0 {
        render_notice(
            ui,
            format!(
                "{} {}",
                legacy_count,
                t("vault_legacy_connections_will_migrate")
            ),
        );
        ui.add_space(theme::SPACE_MD);
    }

    password_field(ui, state, t("vault_master_password"));
    ui.add_space(theme::SPACE_MD);
    confirm_field(ui, state);
    ui.add_space(theme::SPACE_MD);
    render_error(ui, state);
    ui.add_space(theme::SPACE_LG);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 30.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            if ui
                .add(theme::primary_button(&t("vault_create_button")))
                .clicked()
            {
                create_vault(state);
            }
        },
    );
}

fn render_unlock(ui: &mut egui::Ui, state: &mut AppState) {
    render_notice(ui, format!("{}: {}", t("vault_name"), state.vault.name));
    ui.add_space(theme::SPACE_MD);
    password_field(ui, state, t("vault_master_password"));
    ui.add_space(theme::SPACE_MD);
    render_error(ui, state);
    ui.add_space(theme::SPACE_LG);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 30.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            if ui
                .add(theme::primary_button(&t("vault_unlock_button")))
                .clicked()
            {
                unlock_vault(state);
            }
        },
    );
}

fn password_field(ui: &mut egui::Ui, state: &mut AppState, label: String) {
    ui.label(
        RichText::new(label)
            .color(theme::text_secondary())
            .size(11.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_XS);
    ui.horizontal(|ui| {
        let response = ui.add(
            theme::password_input(&mut state.vault.master_password, state.vault.show_password)
                .desired_width(f32::INFINITY),
        );
        if response.changed() {
            crate::korean_keyboard::normalize_password_input(&mut state.vault.master_password);
        }
        if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            match state.vault.status {
                VaultStatus::SetupRequired => create_vault(state),
                VaultStatus::Locked => unlock_vault(state),
                VaultStatus::Unlocked => {}
            }
        }

        let label = if state.vault.show_password {
            t("vault_hide_password")
        } else {
            t("vault_show_password")
        };
        if ui
            .add_sized(
                egui::vec2(54.0, theme::INPUT_HEIGHT),
                egui::Button::new(
                    RichText::new(label)
                        .color(theme::text_secondary())
                        .size(11.0),
                )
                .fill(theme::bg_light())
                .stroke(Stroke::new(1.0, theme::border_default())),
            )
            .clicked()
        {
            state.vault.show_password = !state.vault.show_password;
        }
    });
}

fn confirm_field(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(
        RichText::new(t("vault_confirm_password"))
            .color(theme::text_secondary())
            .size(11.0)
            .strong(),
    );
    ui.add_space(theme::SPACE_XS);
    let response = ui.add(
        theme::password_input(&mut state.vault.confirm_password, state.vault.show_password)
            .desired_width(f32::INFINITY),
    );
    if response.changed() {
        crate::korean_keyboard::normalize_password_input(&mut state.vault.confirm_password);
    }
}

fn render_notice(ui: &mut egui::Ui, text: String) {
    egui::Frame::new()
        .fill(theme::with_alpha(theme::ACCENT_BLUE, 18))
        .stroke(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 70)))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(text)
                    .color(theme::text_secondary())
                    .size(12.0),
            );
        });
}

fn render_error(ui: &mut egui::Ui, state: &AppState) {
    let Some(error) = state.vault.error.as_ref() else {
        return;
    };

    egui::Frame::new()
        .fill(theme::with_alpha(theme::ACCENT_RED, 20))
        .stroke(Stroke::new(
            1.0,
            theme::with_alpha(theme::ACCENT_RED, 80),
        ))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(error).color(theme::text_primary()).size(12.0));
        });
}

fn create_vault(state: &mut AppState) {
    state.vault.error = None;

    if state.vault.master_password.len() < 8 {
        state.vault.error = Some(t("vault_error_short_password"));
        return;
    }
    if state.vault.master_password != state.vault.confirm_password {
        state.vault.error = Some(t("vault_error_password_mismatch"));
        return;
    }

    let legacy_connections = state.vault.legacy_connections.clone();
    match crate::storage::connections::setup_vault(
        &state.vault.master_password,
        &legacy_connections,
    ) {
        Ok(session) => {
            state.saved_connections = legacy_connections;
            state.vault.session = Some(session);
            state.vault.status = VaultStatus::Unlocked;
            state.vault.master_password.zeroize();
            state.vault.confirm_password.zeroize();
            state.vault.legacy_connections.clear();
            state.show_connection_dialog = state.saved_connections.is_empty();
            state.status_message = t("vault_unlocked_status");
        }
        Err(err) => {
            state.vault.error = Some(err.to_string());
        }
    }
}

fn unlock_vault(state: &mut AppState) {
    state.vault.error = None;

    match crate::storage::connections::unlock_vault(&state.vault.master_password) {
        Ok((connections, session)) => {
            state.saved_connections = connections;
            state.vault.name = session.name.clone();
            state.vault.session = Some(session);
            state.vault.status = VaultStatus::Unlocked;
            state.vault.master_password.zeroize();
            state.vault.confirm_password.zeroize();
            state.show_connection_dialog = state.saved_connections.is_empty();
            state.status_message = t("vault_unlocked_status");
        }
        Err(err) => {
            state.vault.error = Some(err.to_string());
        }
    }
}
