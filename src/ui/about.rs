use chrono::{Datelike, Local};
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Pos2, Stroke, StrokeKind};

use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

const ABOUT_WIDTH: f32 = 560.0;
const ABOUT_HEIGHT: f32 = 468.0;

pub fn render_about_window(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_about_dialog {
        return;
    }

    let mut open = state.show_about_dialog;
    let about_size = egui::vec2(ABOUT_WIDTH, ABOUT_HEIGHT);

    egui::Window::new(t("menu_about"))
        .open(&mut open)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_size(about_size)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(Color32::TRANSPARENT)
                .inner_margin(Margin::ZERO)
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(1.0, Color32::from_rgb(64, 106, 170))),
        )
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(about_size, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            paint_background(&painter, rect);
            paint_logo(&painter, rect);
            paint_title(ui, rect);
            paint_version(ui, rect);
            paint_badges(&painter, rect);
            paint_footer(ui, rect);

            let close_rect = egui::Rect::from_min_size(
                rect.right_top() + egui::vec2(-38.0, 12.0),
                egui::vec2(24.0, 24.0),
            );
            let close_resp = ui.interact(
                close_rect,
                ui.id().with("about_close"),
                egui::Sense::click(),
            );
            let close_fill = if close_resp.hovered() {
                Color32::from_rgba_unmultiplied(255, 255, 255, 42)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 18)
            };
            painter.rect_filled(close_rect, CornerRadius::same(12), close_fill);
            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "\u{00D7}",
                FontId::proportional(18.0),
                Color32::from_rgb(224, 234, 255),
            );

            if close_resp.clicked() || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                state.show_about_dialog = false;
            }
        });

    state.show_about_dialog &= open;
}

fn paint_background(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(4, 23, 54));
    painter.rect_filled(
        egui::Rect::from_min_max(rect.min, rect.min + egui::vec2(rect.width(), 188.0)),
        CornerRadius::ZERO,
        Color32::from_rgb(3, 29, 69),
    );

    let wave_a = vec![
        rect.left_top() + egui::vec2(0.0, 264.0),
        rect.left_top() + egui::vec2(48.0, 250.0),
        rect.left_top() + egui::vec2(132.0, 260.0),
        rect.left_top() + egui::vec2(226.0, 252.0),
        rect.left_top() + egui::vec2(330.0, 263.0),
        rect.left_top() + egui::vec2(454.0, 251.0),
        rect.right_top() + egui::vec2(0.0, 264.0),
        rect.right_bottom(),
        rect.left_bottom(),
    ];
    painter.add(egui::Shape::convex_polygon(
        wave_a,
        Color32::from_rgb(12, 50, 101),
        Stroke::NONE,
    ));

    let wave_b = vec![
        rect.left_top() + egui::vec2(0.0, 300.0),
        rect.left_top() + egui::vec2(58.0, 278.0),
        rect.left_top() + egui::vec2(156.0, 294.0),
        rect.left_top() + egui::vec2(262.0, 284.0),
        rect.left_top() + egui::vec2(374.0, 296.0),
        rect.left_top() + egui::vec2(476.0, 282.0),
        rect.right_top() + egui::vec2(0.0, 292.0),
        rect.right_bottom(),
        rect.left_bottom(),
    ];
    painter.add(egui::Shape::convex_polygon(
        wave_b,
        Color32::from_rgb(17, 59, 118),
        Stroke::NONE,
    ));

    let glow = egui::Rect::from_center_size(
        rect.center_top() + egui::vec2(0.0, 128.0),
        egui::vec2(280.0, 152.0),
    );
    painter.rect_filled(
        glow,
        CornerRadius::same(76),
        Color32::from_rgba_unmultiplied(73, 131, 255, 24),
    );
}

fn paint_logo(painter: &egui::Painter, rect: egui::Rect) {
    let center = rect.center_top() + egui::vec2(0.0, 112.0);
    let blue = Color32::from_rgb(72, 136, 255);
    let soft_blue = Color32::from_rgb(62, 124, 246);
    let stroke = Stroke::new(10.5, blue);
    let fine = Stroke::new(9.0, soft_blue);

    painter.add(egui::Shape::line(
        arc_points(center + egui::vec2(0.0, 25.0), 48.0, 0.16, 1.86, 56),
        stroke,
    ));
    painter.add(egui::Shape::line(
        arc_points(center + egui::vec2(-30.0, -16.0), 41.0, 0.97, 2.02, 34),
        fine,
    ));
    painter.add(egui::Shape::line(
        arc_points(center + egui::vec2(30.0, -16.0), 41.0, -0.02, 1.03, 34),
        fine,
    ));

    let core = egui::Rect::from_center_size(center + egui::vec2(0.0, 18.0), egui::vec2(94.0, 58.0));
    painter.rect_stroke(
        core,
        CornerRadius::same(29),
        Stroke::new(7.5, Color32::from_rgb(64, 128, 255)),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        egui::Rect::from_center_size(center + egui::vec2(0.0, 19.0), egui::vec2(70.0, 34.0)),
        CornerRadius::same(17),
        Color32::from_rgba_unmultiplied(72, 136, 255, 16),
    );
}

fn paint_title(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let title_y = rect.top() + 202.0;
    painter.text(
        egui::pos2(rect.center().x, title_y),
        egui::Align2::CENTER_CENTER,
        "FerrumGrid",
        FontId::proportional(35.0),
        Color32::WHITE,
    );

    let sub_y = title_y + 28.0;
    let prefix = "for ";
    let product = "PostgreSQL";
    let prefix_galley = painter.layout_no_wrap(
        prefix.to_owned(),
        FontId::proportional(17.0),
        Color32::from_rgb(214, 226, 246),
    );
    let product_galley = painter.layout_no_wrap(
        product.to_owned(),
        FontId::proportional(17.0),
        Color32::from_rgb(70, 139, 255),
    );
    let total = prefix_galley.rect.width() + product_galley.rect.width();
    let start = rect.center().x - total / 2.0;
    painter.galley(
        egui::pos2(start, sub_y - prefix_galley.rect.height() / 2.0),
        prefix_galley,
        Color32::from_rgb(214, 226, 246),
    );
    painter.galley(
        egui::pos2(
            start + total - product_galley.rect.width(),
            sub_y - product_galley.rect.height() / 2.0,
        ),
        product_galley,
        Color32::from_rgb(70, 139, 255),
    );
}

fn paint_version(ui: &egui::Ui, rect: egui::Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let lines = [
        format!("{} {version}", t("about_version")),
        t("about_edition"),
        t("about_engine"),
        t("about_author"),
    ];

    for (index, line) in lines.iter().enumerate() {
        ui.painter().text(
            egui::pos2(rect.center().x, rect.top() + 304.0 + index as f32 * 16.0),
            egui::Align2::CENTER_CENTER,
            line,
            FontId::proportional(13.0),
            Color32::from_rgb(207, 219, 241),
        );
    }
}

fn paint_badges(painter: &egui::Painter, rect: egui::Rect) {
    let labels = ["PG", "SQL", "ER", "BI", "⌘"];
    let colors = [
        theme::ACCENT_BLUE,
        theme::ACCENT_TEAL,
        theme::ACCENT_EMERALD,
        theme::ACCENT_RED,
        theme::TEXT_MUTED,
    ];
    let start_x = rect.center().x - 104.0;
    let y = rect.bottom() - 52.0;

    for (idx, label) in labels.iter().enumerate() {
        let center = egui::pos2(start_x + idx as f32 * 52.0, y);
        painter.circle_filled(center, 14.0, colors[idx]);
        painter.circle_stroke(
            center,
            14.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 90)),
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            *label,
            FontId::proportional(10.5),
            Color32::WHITE,
        );
    }
}

fn paint_footer(ui: &egui::Ui, rect: egui::Rect) {
    let year = Local::now().year();
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - 15.0),
        egui::Align2::CENTER_CENTER,
        format!("Copyright © {year} FerrumGrid. All rights reserved."),
        FontId::proportional(11.0),
        Color32::from_rgb(195, 207, 229),
    );
}

fn arc_points(
    center: Pos2,
    radius: f32,
    start_turns: f32,
    end_turns: f32,
    steps: usize,
) -> Vec<Pos2> {
    let start = start_turns * std::f32::consts::TAU;
    let end = end_turns * std::f32::consts::TAU;
    (0..=steps)
        .map(|index| {
            let t = index as f32 / steps as f32;
            let angle = start + (end - start) * t;
            egui::pos2(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            )
        })
        .collect()
}
