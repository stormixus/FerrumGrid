//! Custom titlebar strip — borderless window top drag region + macOS native
//! traffic lights preservation.

use eframe::egui::{self, CornerRadius, Margin, Sense};

use crate::state::{AppState, ConnectionStatus};
use super::theme;

pub const TITLEBAR_HEIGHT: f32 = 32.0;

pub const MAC_TRAFFIC_LIGHTS_WIDTH: f32 = 78.0;

pub fn render_titlebar(ctx: &egui::Context, state: &AppState, settings: &mut crate::storage::settings::AppSettings) {
    egui::TopBottomPanel::top("custom_titlebar")
        .exact_height(TITLEBAR_HEIGHT)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::ZERO),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            let drag_rect = drag_region_rect(full_rect);

            let drag_response = ui.interact(
                drag_rect,
                ui.id().with("titlebar_drag"),
                Sense::click_and_drag(),
            );
            if drag_response.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if drag_response.double_clicked() {
                let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            let painter = ui.painter_at(full_rect);

            // Center: connection info
            let title = build_titlebar_text(state);
            let galley = painter.layout_no_wrap(
                title,
                egui::FontId::proportional(12.0),
                theme::text_secondary(),
            );
            let center = egui::pos2(
                full_rect.center().x - galley.size().x * 0.5,
                full_rect.center().y - galley.size().y * 0.5,
            );
            painter.galley(center, galley, theme::text_secondary());

            // Right: Dark/Light theme toggle
            let toggle_right = full_rect.right() - 12.0;
            let toggle_y = full_rect.center().y;

            // Pill container background
            let pill_w = 78.0;
            let pill_h = 22.0;
            let pill_rect = egui::Rect::from_min_size(
                egui::pos2(toggle_right - pill_w, toggle_y - pill_h / 2.0),
                egui::vec2(pill_w, pill_h),
            );
            painter.rect_filled(
                pill_rect,
                CornerRadius::same(theme::RADIUS_MD),
                theme::bg_light(),
            );

            for (i, (label, is_dark)) in [("Dark", true), ("Light", false)].iter().enumerate() {
                let w = 38.0;
                let x = pill_rect.left() + i as f32 * w;
                let rect = egui::Rect::from_min_size(
                    egui::pos2(x, pill_rect.top()),
                    egui::vec2(w, pill_h),
                );
                let active = settings.dark_mode == *is_dark;
                let response = ui.interact(rect, ui.id().with(label), Sense::click());

                if active {
                    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), theme::with_alpha(theme::ACCENT_EMERALD, 40));
                }
                let text_color = if active {
                    theme::ACCENT_EMERALD
                } else {
                    theme::text_muted()
                };

                let seg_galley = painter.layout_no_wrap(
                    label.to_string(),
                    egui::FontId::proportional(11.0),
                    text_color,
                );
                painter.galley(
                    egui::pos2(
                        rect.center().x - seg_galley.size().x / 2.0,
                        rect.center().y - seg_galley.size().y / 2.0,
                    ),
                    seg_galley,
                    text_color,
                );

                if response.clicked() && settings.dark_mode != *is_dark {
                    settings.dark_mode = *is_dark;
                    settings.appearance = if *is_dark { "dark" } else { "light" }.to_string();
                    theme::apply_appearance(ui.ctx(), &settings.appearance);
                    crate::storage::settings::save_settings(settings);
                }
            }
        });
}

fn build_titlebar_text(state: &AppState) -> String {
    let mut parts = vec!["FerrumGrid".to_string()];
    if let Some(conn_id) = state.active_connection {
        if let Some(conn) = state.connections.get(&conn_id) {
            parts.push(" \u{2014} ".to_string());
            let dot = match &conn.status {
                ConnectionStatus::Connected { .. } => "\u{25CF} ",
                ConnectionStatus::Connecting => "\u{25CB} ",
                ConnectionStatus::Disconnected => "\u{25CB} ",
            };
            parts.push(dot.to_string());
            parts.push(conn.config.display_name.clone());
            parts.push(" \u{00B7} ".to_string());
            parts.push(conn.config.database.clone());
        }
    }
    parts.join("")
}

/// 신호등 영역 (좌측 [`MAC_TRAFFIC_LIGHTS_WIDTH`]) 을 제외한 drag region.
/// macOS 가 아닌 경우 전체 strip 이 drag region.
pub fn drag_region_rect(full: egui::Rect) -> egui::Rect {
    if cfg!(target_os = "macos") {
        egui::Rect::from_min_max(
            egui::pos2(full.min.x + MAC_TRAFFIC_LIGHTS_WIDTH, full.min.y),
            full.max,
        )
    } else {
        full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(w: f32) -> egui::Rect {
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w, TITLEBAR_HEIGHT))
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn drag_region_excludes_traffic_lights_area_on_macos() {
        let dr = drag_region_rect(rect(1000.0));
        assert_eq!(dr.min.x, MAC_TRAFFIC_LIGHTS_WIDTH);
        assert_eq!(dr.max.x, 1000.0);
        assert_eq!(dr.min.y, 0.0);
        assert_eq!(dr.max.y, TITLEBAR_HEIGHT);
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn drag_region_covers_full_strip_on_non_macos() {
        let dr = drag_region_rect(rect(1000.0));
        assert_eq!(dr, rect(1000.0));
    }

    #[test]
    fn drag_region_width_positive_for_typical_window_size() {
        // 800px (min_inner_size) → 신호등 78px 제외 후 722px 남음 (macOS).
        // 비-macOS 는 전체 800px 가 drag region.
        let dr = drag_region_rect(rect(800.0));
        assert!(dr.width() > 0.0);
        if cfg!(target_os = "macos") {
            assert!((dr.width() - (800.0 - MAC_TRAFFIC_LIGHTS_WIDTH)).abs() < f32::EPSILON);
        } else {
            assert!((dr.width() - 800.0).abs() < f32::EPSILON);
        }
    }
}
