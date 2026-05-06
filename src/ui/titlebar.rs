//! Custom titlebar strip — borderless 윈도우 상단의 drag region + macOS native
//! traffic lights 영역 보존.
//!
//! macOS: `with_fullsize_content_view(true)` + `with_title_shown(false)` 와
//! 함께 사용. 콘텐츠가 native titlebar 아래로 확장되지만 신호등 (close/min/max)
//! 은 native 그대로 좌상단에 표시된다. 본 strip 은 신호등 우측을 drag region
//! 으로 처리하고 좌측 [`MAC_TRAFFIC_LIGHTS_WIDTH`] 만큼은 비워둔다.

use eframe::egui::{self, Margin, Sense, Stroke};

use super::theme;

/// 타이틀바 strip 의 총 높이 (px).
pub const TITLEBAR_HEIGHT: f32 = 28.0;

/// macOS native traffic lights 가 차지하는 좌측 영역 폭 (close/min/max 3개 +
/// 좌측 padding). 본 영역은 drag region 에서 제외해야 신호등 클릭이 hit.
pub const MAC_TRAFFIC_LIGHTS_WIDTH: f32 = 78.0;

/// `panels::render_panels` 가 가장 먼저 호출 — main_toolbar 위에 위치한다.
/// 빈 strip 위에 pointer drag 시 `ViewportCommand::StartDrag` 발사.
pub fn render_titlebar(ctx: &egui::Context) {
    egui::TopBottomPanel::top("custom_titlebar")
        .exact_height(TITLEBAR_HEIGHT)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::ZERO)
                .stroke(Stroke::new(1.0, theme::border_subtle())),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            let full_rect = ui.max_rect();
            let drag_rect = drag_region_rect(full_rect);

            // Drag region — pointer drag 시 윈도우 이동.
            let drag_response = ui.interact(
                drag_rect,
                ui.id().with("titlebar_drag"),
                Sense::click_and_drag(),
            );
            if drag_response.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            // 더블클릭 시 maximize 토글 (macOS native 동작 모방).
            if drag_response.double_clicked() {
                let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            let painter = ui.painter_at(full_rect);
            let title = "FerrumGrid";
            let galley = painter.layout_no_wrap(
                title.to_string(),
                egui::FontId::proportional(12.0),
                theme::text_muted(),
            );
            if cfg!(target_os = "macos") {
                let pos = egui::pos2(
                    full_rect.min.x + MAC_TRAFFIC_LIGHTS_WIDTH + 4.0,
                    full_rect.center().y - galley.size().y * 0.5,
                );
                painter.galley(pos, galley, theme::text_muted());
            } else {
                let center = full_rect.center() - galley.size() * 0.5;
                painter.galley(center, galley, theme::text_muted());
            }
        });
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
