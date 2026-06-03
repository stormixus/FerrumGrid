//! Grid footer (active SQL preview + Copy SQL action) + grid body sizing.
//!
//! Plan v7 Phase 1.95c3b cut-over (from `super::mod.rs`).

use eframe::egui::{self, CornerRadius, Stroke};

use crate::i18n::t;
use crate::state::{build_data_select_sql_with_columns, AppState, MainView};
use crate::ui::theme;

use super::{
    data_page_offset, normalized_data_limit, result_toolbar_action_button,
    result_toolbar_action_width, show_dark_hover_tooltip,
};

pub(super) fn should_show_data_query_footer(state: &AppState) -> bool {
    state.active_main_view == MainView::Data && state.active_data_source().is_some()
}

pub(super) fn render_grid_body_with_reserved_footer(
    ui: &mut egui::Ui,
    add_body: impl FnOnce(&mut egui::Ui),
) {
    let footer_height = data_query_footer_height();
    let available = ui.available_size();
    let body_height = (available.y - footer_height).max(0.0);
    if body_height <= 0.0 {
        return;
    }

    ui.allocate_ui_with_layout(
        egui::vec2(available.x, body_height),
        egui::Layout::top_down(egui::Align::Min),
        add_body,
    );
}

pub(super) fn render_data_query_footer(ui: &mut egui::Ui, state: &AppState) {
    let Some(sql) = active_data_query_sql(state) else {
        return;
    };

    let height = data_query_footer_height();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, theme::bg_shell());
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, theme::border_subtle()),
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, theme::SPACE_SM));
    let copy_label = t("grid_copy_sql");
    let copy_width = result_toolbar_action_width(ui, &copy_label);
    let copy_rect = egui::Rect::from_min_max(
        egui::pos2(inner.right() - copy_width, inner.top()),
        egui::pos2(inner.right(), inner.bottom()),
    );
    let sql_rect = egui::Rect::from_min_max(
        inner.left_top(),
        egui::pos2(copy_rect.left() - theme::SPACE_MD, inner.bottom()),
    );

    let aggregate = selection_aggregate_label(state);
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(sql_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(sql_rect);
            match &aggregate {
                Some(label) => {
                    ui.label(
                        egui::RichText::new(label)
                            .monospace()
                            .color(theme::accent_color())
                            .size(11.0),
                    );
                }
                None => render_data_query_preview(ui, &sql),
            }
        },
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(copy_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(copy_rect);
            let response = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_data_sql",
                &copy_label,
                true,
            );
            if response.clicked() {
                ui.ctx().copy_text(sql);
            }
        },
    );
}

fn render_data_query_preview(ui: &mut egui::Ui, sql: &str) {
    let available = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available, 32.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::bg_darkest(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_subtle()),
        egui::StrokeKind::Inside,
    );

    let label_rect = egui::Rect::from_min_size(rect.left_top(), egui::vec2(42.0, rect.height()))
        .shrink2(egui::vec2(theme::SPACE_SM, 0.0));
    ui.painter().text(
        label_rect.center(),
        egui::Align2::CENTER_CENTER,
        "SQL",
        egui::FontId::proportional(10.5),
        theme::accent_color(),
    );

    let text_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 46.0, rect.top()),
        rect.right_bottom(),
    )
    .shrink2(egui::vec2(theme::SPACE_SM, 0.0));
    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        sql,
        egui::FontId::monospace(11.0),
        theme::text_secondary(),
    );

    show_dark_hover_tooltip(ui, response.id.with("sql_preview_tooltip"), &response, sql);
}

fn active_data_query_sql(state: &AppState) -> Option<String> {
    let source = state.active_data_source()?;
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    Some(build_data_select_sql_with_columns(
        &source,
        &state.data_edit.sort,
        limit,
        offset,
        &state.data_columns_for_source(&source),
    ))
}

/// 선택 영역(직사각형) 내 셀에 대한 집계 라벨. 숫자 셀이 있으면 sum/avg/min/max
/// 도 포함. 영역이 없으면 None (→ 푸터는 SQL 미리보기를 표시).
fn selection_aggregate_label(state: &AppState) -> Option<String> {
    use crate::types::CellValue;
    let (a, b) = state.data_edit.selection_range?;
    let result = state.current_result.as_ref()?;
    let (r0, r1) = (a.0.min(b.0), a.0.max(b.0));
    let (c0, c1) = (a.1.min(b.1), a.1.max(b.1));

    let mut count = 0usize;
    let mut n_num = 0usize;
    let mut sum = 0.0f64;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for r in r0..=r1 {
        let Some(row) = result.rows.get(r) else { continue };
        for c in c0..=c1 {
            let Some(cell) = row.get(c) else { continue };
            count += 1;
            let v = match cell {
                CellValue::Int(i) => Some(*i as f64),
                CellValue::Float(f) => Some(*f),
                _ => None,
            };
            if let Some(v) = v {
                n_num += 1;
                sum += v;
                min = min.min(v);
                max = max.max(v);
            }
        }
    }
    if count == 0 {
        return None;
    }
    if n_num == 0 {
        return Some(format!("{count} cells"));
    }
    let avg = sum / n_num as f64;
    Some(format!(
        "{count} cells  ·  sum {sum:.4}  ·  avg {avg:.4}  ·  min {min:.4}  ·  max {max:.4}",
    ))
}

fn data_query_footer_height() -> f32 {
    48.0
}
