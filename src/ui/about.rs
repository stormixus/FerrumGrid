use chrono::{Datelike, Local};
use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Stroke, StrokeKind};

use crate::i18n::t;
use crate::state::AppState;

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
                .corner_radius(CornerRadius::same(12))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(about_size, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            // 1. Render procedural background layers and grid animations
            paint_premium_background(&painter, rect, ui);
            paint_brand_logo(&painter, rect, ui);
            paint_premium_title(&painter, rect);
            
            // 2. Render copyable version information panel (selectable layers)
            render_copyable_version_card(ui, rect);

            // 3. Render copyable support email row with click-to-clipboard action
            render_copyable_email_row(ui, rect, &painter);

            // 4. Render footer and close button
            paint_badges(&painter, rect);
            paint_footer(ui, rect);

            // Close button (Top Right)
            let close_rect = egui::Rect::from_min_size(
                rect.right_top() + egui::vec2(-38.0, 14.0),
                egui::vec2(24.0, 24.0),
            );
            let close_resp = ui.interact(
                close_rect,
                ui.id().with("about_close"),
                egui::Sense::click(),
            );
            let close_fill = if close_resp.hovered() {
                Color32::from_rgba_unmultiplied(229, 72, 77, 40) // Ruby red glow on close hover
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 12)
            };
            let close_stroke = if close_resp.hovered() {
                Stroke::new(1.0, Color32::from_rgb(229, 72, 77))
            } else {
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20))
            };
            painter.rect(
                close_rect,
                CornerRadius::same(6),
                close_fill,
                close_stroke,
                StrokeKind::Inside,
            );
            painter.text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "\u{00D7}",
                FontId::proportional(16.0),
                if close_resp.hovered() { Color32::from_rgb(255, 145, 145) } else { Color32::from_rgb(214, 226, 246) },
            );

            if close_resp.clicked() || ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                state.show_about_dialog = false;
            }
        });

    state.show_about_dialog &= open;
}

fn paint_premium_background(painter: &egui::Painter, rect: egui::Rect, ui: &egui::Ui) {
    // Canvas Base: Deep Slate Solid Charcoal Canvas
    painter.rect_filled(rect, CornerRadius::same(12), Color32::from_rgb(11, 12, 14));

    // Glowing Concentric Top Halo (Procedural Emerald Glow behind Logo)
    let center_top = rect.center_top() + egui::vec2(0.0, 95.0);
    for i in 0..8 {
        let radius = 160.0 - (i as f32 * 14.0);
        let alpha = (i as f32 * 2.2) as u8;
        painter.circle_filled(
            center_top,
            radius,
            Color32::from_rgba_unmultiplied(62, 207, 142, alpha),
        );
    }

    // Outer premium double borders
    painter.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(62, 207, 142, 100)), // Subtle emerald outer border
        StrokeKind::Inside,
    );
    let inner_rect = rect.shrink(6.0);
    painter.rect_stroke(
        inner_rect,
        CornerRadius::same(9),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 10)),
        StrokeKind::Inside,
    );

    // Procedural Crystalline Isometric "Ferrum Grid" Mesh
    let time = ui.input(|i| i.time) as f32;
    let iso_x = |x: f32, y: f32| x - y;
    let iso_y = |x: f32, y: f32| (x + y) * 0.55;
    let scale = 25.0;

    // Draw isometric grid lines
    for i in -2..=2 {
        for j in -2..=2 {
            let x1 = i as f32 * scale;
            let y1 = j as f32 * scale;
            
            if i < 2 {
                let x2 = (i + 1) as f32 * scale;
                let y2 = j as f32 * scale;
                let p1 = center_top + egui::vec2(iso_x(x1, y1), iso_y(x1, y1));
                let p2 = center_top + egui::vec2(iso_x(x2, y2), iso_y(x2, y2));
                painter.line_segment(
                    [p1, p2],
                    Stroke::new(0.6, Color32::from_rgba_unmultiplied(62, 207, 142, 22)),
                );
            }
            if j < 2 {
                let x2 = i as f32 * scale;
                let y2 = (j + 1) as f32 * scale;
                let p1 = center_top + egui::vec2(iso_x(x1, y1), iso_y(x1, y1));
                let p2 = center_top + egui::vec2(iso_x(x2, y2), iso_y(x2, y2));
                painter.line_segment(
                    [p1, p2],
                    Stroke::new(0.6, Color32::from_rgba_unmultiplied(62, 207, 142, 22)),
                );
            }
        }
    }

    // Draw pulsing grid junctions
    for i in -2..=2 {
        for j in -2..=2 {
            let x = i as f32 * scale;
            let y = j as f32 * scale;
            let pos = center_top + egui::vec2(iso_x(x, y), iso_y(x, y));

            let pulse_delay = (i as f32 + j as f32) * 0.5;
            let wave = ((time * 2.0) + pulse_delay).sin();
            let radius = 1.8 + (wave * 0.7 + 0.7);
            let glow_radius = 4.5 + (wave * 2.0 + 2.0);

            painter.circle_filled(
                pos,
                glow_radius,
                Color32::from_rgba_unmultiplied(62, 207, 142, 18),
            );
            painter.circle_filled(
                pos,
                radius,
                Color32::from_rgb(62, 207, 142),
            );
        }
    }
}

fn paint_brand_logo(painter: &egui::Painter, rect: egui::Rect, _ui: &egui::Ui) {
    let center = rect.center_top() + egui::vec2(0.0, 95.0);

    // Glowing orbital rings around gemstone core
    painter.circle_stroke(
        center,
        38.0,
        Stroke::new(2.0, Color32::from_rgb(62, 207, 142)),
    );
    painter.circle_stroke(
        center,
        45.0,
        Stroke::new(0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 35)),
    );

    // Central Emerald Crystalline Core (Procedural convex shape)
    let gem_points = vec![
        center + egui::vec2(0.0, -16.0),
        center + egui::vec2(13.0, 0.0),
        center + egui::vec2(0.0, 16.0),
        center + egui::vec2(-13.0, 0.0),
    ];
    painter.add(egui::Shape::convex_polygon(
        gem_points,
        Color32::from_rgb(10, 24, 18),
        Stroke::new(2.2, Color32::from_rgb(62, 207, 142)),
    ));

    // Dynamic inner core light reflect
    painter.circle_filled(
        center + egui::vec2(-3.0, -3.0),
        2.5,
        Color32::from_rgba_unmultiplied(255, 255, 255, 200),
    );
}

fn paint_premium_title(painter: &egui::Painter, rect: egui::Rect) {
    let title_y = rect.top() + 165.0;

    // Header 1: FerrumGrid
    painter.text(
        egui::pos2(rect.center().x, title_y),
        egui::Align2::CENTER_CENTER,
        "FerrumGrid Studio",
        FontId::proportional(26.0),
        Color32::WHITE,
    );

    // Header 2: Subtitle database workbench description
    let sub_y = title_y + 24.0;
    let label1 = "PostgreSQL ";
    let label2 = "Native Workbench";
    let l1_galley = painter.layout_no_wrap(label1.to_owned(), FontId::proportional(13.0), Color32::from_rgb(62, 207, 142));
    let l2_galley = painter.layout_no_wrap(label2.to_owned(), FontId::proportional(13.0), Color32::from_rgb(160, 170, 185));
    let total_w = l1_galley.rect.width() + l2_galley.rect.width();

    let start_x = rect.center().x - total_w / 2.0;
    painter.galley(
        egui::pos2(start_x, sub_y - l1_galley.rect.height() / 2.0),
        l1_galley,
        Color32::from_rgb(62, 207, 142),
    );
    painter.galley(
        egui::pos2(start_x + total_w - l2_galley.rect.width(), sub_y - l2_galley.rect.height() / 2.0),
        l2_galley,
        Color32::from_rgb(160, 170, 185),
    );
}

fn render_copyable_version_card(ui: &mut egui::Ui, rect: egui::Rect) {
    let version = env!("CARGO_PKG_VERSION");

    let card_rect = egui::Rect::from_min_size(
        rect.center_top() + egui::vec2(-190.0, 202.0),
        egui::vec2(380.0, 146.0),
    );

    // Vector Glass Panel Background
    ui.painter().rect(
        card_rect,
        CornerRadius::same(8),
        Color32::from_rgb(18, 19, 21), // Slate surface
        Stroke::new(1.0, Color32::from_rgb(38, 38, 42)), // Precise border
        StrokeKind::Inside,
    );

    // Render selectable labels inside absolute Child UI area
    let child_rect = card_rect.shrink(12.0);
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(child_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    egui::Grid::new("about_grid")
        .num_columns(2)
        .spacing(egui::vec2(16.0, 6.0))
        .show(&mut child_ui, |ui| {
            // Row 1: Version with status pill
            ui.label(egui::RichText::new("Version").color(Color32::from_rgb(120, 130, 145)).size(12.0));
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(version).color(Color32::WHITE).size(12.0).strong())
                        .selectable(true),
                );
                
                // Latest build indicator
                let (pill_rect, _) = ui.allocate_exact_size(egui::vec2(62.0, 16.0), egui::Sense::hover());
                ui.painter().rect(
                    pill_rect,
                    CornerRadius::same(8),
                    Color32::from_rgba_unmultiplied(62, 207, 142, 20),
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(62, 207, 142, 80)),
                    StrokeKind::Inside,
                );
                ui.painter().text(
                    pill_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Latest Build",
                    FontId::proportional(9.0),
                    Color32::from_rgb(62, 207, 142),
                );
            });
            ui.end_row();

            // Row 2: Edition
            ui.label(egui::RichText::new("Edition").color(Color32::from_rgb(120, 130, 145)).size(12.0));
            ui.add(
                egui::Label::new(egui::RichText::new(t("about_edition")).color(Color32::WHITE).size(12.0))
                    .selectable(true),
            );
            ui.end_row();

            // Row 3: Offline database Prisma engine
            ui.label(egui::RichText::new("Engine").color(Color32::from_rgb(120, 130, 145)).size(12.0));
            ui.add(
                egui::Label::new(egui::RichText::new("Rust Native Offline").color(Color32::WHITE).size(12.0))
                    .selectable(true),
            );
            ui.end_row();

            // Row 4: MIT License details
            ui.label(egui::RichText::new("License").color(Color32::from_rgb(120, 130, 145)).size(12.0));
            ui.add(
                egui::Label::new(egui::RichText::new("MIT License").color(Color32::WHITE).size(12.0))
                    .selectable(true),
            );
            ui.end_row();
        });
}

fn render_copyable_email_row(ui: &mut egui::Ui, rect: egui::Rect, painter: &egui::Painter) {
    let email_id = ui.id().with("support_email");
    let mut copied = false;
    if let Some(copied_time) = ui.data(|d| d.get_temp::<std::time::Instant>(email_id)) {
        if copied_time.elapsed().as_secs_f32() < 2.0 {
            copied = true;
        }
    }

    let email_rect = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.bottom() - 92.0),
        egui::vec2(290.0, 32.0),
    );

    let email_resp = ui.interact(email_rect, email_id, egui::Sense::click());
    let is_hovered = email_resp.hovered();

    // Render interactive buttons via layers
    let btn_fill = if is_hovered {
        Color32::from_rgba_unmultiplied(62, 207, 142, 14) // Neon glow emerald tint
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 5) // Muted glass
    };
    let btn_stroke = if is_hovered {
        Stroke::new(1.0, Color32::from_rgb(62, 207, 142)) // Interactive glowing emerald border
    } else {
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 15))
    };

    painter.rect(
        email_rect,
        CornerRadius::same(6),
        btn_fill,
        btn_stroke,
        StrokeKind::Inside,
    );

    // Render icon & support email string values
    let (icon, label_text, color) = if copied {
        ("\u{2714}", "Copied: support@ferrumgrid.com", Color32::from_rgb(62, 207, 142))
    } else {
        ("\u{2709}", "support@ferrumgrid.com", Color32::from_rgb(180, 195, 210))
    };

    painter.text(
        email_rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{} {}", icon, label_text),
        FontId::proportional(11.5),
        color,
    );

    // Interactive mouse hover feedback subtitle
    if is_hovered && !copied {
        painter.text(
            egui::pos2(rect.center().x, email_rect.bottom() + 3.0),
            egui::Align2::CENTER_TOP,
            "Click to copy support email address",
            FontId::proportional(9.0),
            Color32::from_rgba_unmultiplied(255, 255, 255, 90),
        );
    }

    if email_resp.clicked() {
        ui.ctx().copy_text("support@ferrumgrid.com".to_string());
        ui.data_mut(|d| d.insert_temp(email_id, std::time::Instant::now()));
    }
}

fn paint_badges(painter: &egui::Painter, rect: egui::Rect) {
    let labels = ["PG", "SQL", "ER", "BI", "⌘"];
    let colors = [
        Color32::from_rgba_unmultiplied(62, 207, 142, 24), // PG is Emerald
        Color32::from_rgba_unmultiplied(72, 156, 255, 24), // SQL is Azure
        Color32::from_rgba_unmultiplied(255, 160, 92, 24), // ER is Coral
        Color32::from_rgba_unmultiplied(255, 82, 82, 24),  // BI is Ruby
        Color32::from_rgba_unmultiplied(168, 176, 190, 16), // ⌘ is gray
    ];
    let border_colors = [
        Color32::from_rgba_unmultiplied(62, 207, 142, 90),
        Color32::from_rgba_unmultiplied(72, 156, 255, 90),
        Color32::from_rgba_unmultiplied(255, 160, 92, 90),
        Color32::from_rgba_unmultiplied(255, 82, 82, 90),
        Color32::from_rgba_unmultiplied(168, 176, 190, 70),
    ];

    let start_x = rect.center().x - 110.0;
    let y = rect.bottom() - 44.0;

    for (idx, label) in labels.iter().enumerate() {
        let badge_rect = egui::Rect::from_center_size(
            egui::pos2(start_x + idx as f32 * 54.0, y),
            egui::vec2(42.0, 24.0),
        );

        // Render sleek vector badge pill
        painter.rect_filled(badge_rect, CornerRadius::same(5), colors[idx]);
        painter.rect_stroke(
            badge_rect,
            CornerRadius::same(5),
            Stroke::new(1.0, border_colors[idx]),
            StrokeKind::Inside,
        );
        
        painter.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            FontId::proportional(10.5),
            Color32::from_rgb(220, 225, 235),
        );
    }
}

fn paint_footer(ui: &egui::Ui, rect: egui::Rect) {
    let year = Local::now().year();
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - 15.0),
        egui::Align2::CENTER_CENTER,
        format!("© Copyright {year} FerrumGrid, Inc. All Rights Reserved."),
        FontId::proportional(9.0),
        Color32::from_rgb(100, 110, 125),
    );
}
