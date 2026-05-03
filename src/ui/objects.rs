use crate::db::bridge::DbBridge;
use crate::state::AppState;
use crate::ui::theme;
use eframe::egui::{self, Margin, RichText, Stroke};

pub fn render_objects_view(ui: &mut egui::Ui, state: &mut AppState, _bridge: &DbBridge) {
    render_tabs(ui);
    render_sub_toolbar(ui, state);
    render_objects_list(ui, state);
}

fn render_tabs(ui: &mut egui::Ui) {
    let tab_frame = egui::Frame::new()
        .fill(theme::BG_SHELL)
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(34.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Objects")
                    .color(theme::TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
            );
        });
    });
}

fn render_sub_toolbar(ui: &mut egui::Ui, _state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(theme::BG_DARK)
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            // Placeholder buttons for Navicat-like sub-toolbar
            crate::ui::icon_img(ui, crate::ui::icons_svg::REFRESH, "refresh", 14.0);
            ui.add_space(theme::SPACE_MD);
            crate::ui::icon_img(ui, crate::ui::icons_svg::PLUS, "new", 14.0);
            ui.add_space(theme::SPACE_MD);
            crate::ui::icon_img(ui, crate::ui::icons_svg::MODEL, "design", 14.0);
            ui.add_space(theme::SPACE_MD);
            crate::ui::icon_img(ui, crate::ui::icons_svg::CLOSE, "delete", 14.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(theme::SPACE_MD);
                let mut search = String::new();
                ui.add(
                    egui::TextEdit::singleline(&mut search)
                        .hint_text("Search")
                        .margin(Margin::symmetric(8, 2)),
                );
            });
        });
    });
}

fn render_objects_list(ui: &mut egui::Ui, _state: &AppState) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(
                RichText::new("No Objects Selected")
                    .color(theme::TEXT_MUTED)
                    .size(16.0),
            );
        });
    });
}
