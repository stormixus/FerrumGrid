//! Grid render entry — top-level render_grid + error/empty states + all
//! rendering/painting functions (result toolbar, header/sort, table, cells).
//!
//! Plan v7 Phase 1.95c3c cut-over (from `super::mod.rs`). Subsequent
//! cut-over moved render_result_header, render_table, header/sort, and
//! cell rendering helpers here.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{AppState, DataSortDirection, MainView};
use crate::types::CellValue;
use crate::ui::theme;

use super::*;

pub fn render_grid(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if let Some(ref error) = state.last_error.clone() {
        render_error_bar(ui, error);
    }

    match &state.current_result {
        None => {
            if should_show_data_query_footer(state) {
                render_grid_body_with_reserved_footer(ui, |ui| {
                    render_empty_state(ui, state.query_running);
                });
                render_data_query_footer(ui, state);
            } else {
                render_empty_state(ui, state.query_running);
            }
        }
        Some(_) => {
            render_result_header(ui, state, bridge);
            if should_show_data_query_footer(state) {
                render_grid_body_with_reserved_footer(ui, |ui| {
                    render_table(ui, state, bridge);
                });
                render_data_query_footer(ui, state);
            } else {
                render_table(ui, state, bridge);
            }
        }
    }
}

fn render_error_bar(ui: &mut egui::Ui, error: &str) {
    let frame = egui::Frame::new()
        .fill(theme::with_alpha(theme::ACCENT_RED, 28))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_RED, 86)));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            crate::ui::icon_img(ui, crate::ui::icons_svg::ERROR, "grid_err", 12.0);
            ui.add_space(4.0);
            ui.label(
                RichText::new("Error")
                    .color(theme::ACCENT_RED)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new(error)
                    .color(Color32::from_rgb(220, 150, 150))
                    .size(12.0),
            );
        });
    });
}

fn render_empty_state(ui: &mut egui::Ui, running: bool) {
    ui.centered_and_justified(|ui| {
        if running {
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(theme::SPACE_MD);
                ui.label(
                    RichText::new("Executing query...")
                        .color(theme::text_muted())
                        .size(12.0),
                );
            });
        } else {
            ui.vertical_centered(|ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::TABLE, "grid_empty", 34.0);
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new("No result set")
                        .color(theme::text_muted())
                        .strong()
                        .size(13.0),
                );
                ui.label(
                    RichText::new("Run a query to populate the grid")
                        .color(theme::text_disabled())
                        .size(11.0),
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Result info header strip
// ---------------------------------------------------------------------------

pub fn render_result_header(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;
    let data_edit_summary = data_edit_summary(state);

    let header_height = 56.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), header_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, theme::bg_shell());
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, theme::border_subtle()),
    );

    let inner = rect.shrink2(egui::vec2(theme::SPACE_LG, 0.0));
    let content_rect = egui::Rect::from_center_size(
        inner.center(),
        egui::vec2(inner.width(), theme::BUTTON_HEIGHT),
    );
    let tsv_width = result_toolbar_action_width(ui, "Copy TSV");
    let csv_width = result_toolbar_action_width(ui, "CSV");
    let mut right_width = tsv_width + csv_width + theme::SPACE_SM;
    if data_edit_summary.is_some() {
        right_width += 330.0;
    }
    right_width = right_width.min(content_rect.width() * 0.46);

    let meta_width = result_meta_group_width(ui, row_count, col_count, exec_ms, truncated)
        .min((content_rect.width() - right_width - theme::SPACE_LG).max(120.0));
    let right_rect = egui::Rect::from_min_max(
        egui::pos2(content_rect.right() - right_width, content_rect.top()),
        content_rect.right_bottom(),
    );
    let meta_rect = egui::Rect::from_min_size(
        content_rect.left_top(),
        egui::vec2(meta_width, content_rect.height()),
    );
    let middle_rect = egui::Rect::from_min_max(
        egui::pos2(meta_rect.right() + theme::SPACE_LG, content_rect.top()),
        egui::pos2(right_rect.left() - theme::SPACE_LG, content_rect.bottom()),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(meta_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(meta_rect);
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
            ui.label(
                RichText::new("Result")
                    .color(theme::text_primary())
                    .strong()
                    .size(13.0),
            );
            ui.add_space(theme::SPACE_MD);
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    row_count,
                    if row_count == 1 { "row" } else { "rows" }
                ),
                theme::ACCENT_TEAL,
            );
            result_meta_chip(
                ui,
                &format!(
                    "{} {}",
                    col_count,
                    if col_count == 1 { "col" } else { "cols" }
                ),
                theme::ACCENT_BLUE,
            );
            result_meta_chip(ui, &format!("{exec_ms}ms"), theme::ACCENT_COPPER);

            if truncated {
                result_meta_chip_svg(
                    ui,
                    "trunc",
                    crate::ui::icons_svg::TRUNCATED,
                    "truncated_icon",
                    theme::ACCENT_YELLOW,
                );
            }
        },
    );

    if middle_rect.width() > 120.0 && state.active_main_view == MainView::Data {
        let pager_width = 488.0_f32.min(middle_rect.width());
        let pager_rect = egui::Rect::from_center_size(
            middle_rect.center(),
            egui::vec2(pager_width, content_rect.height()),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(pager_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(pager_rect);
                render_data_pager(ui, state, bridge, truncated, row_count);
            },
        );
    }

    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.set_clip_rect(right_rect);
            if let Some(summary) = &data_edit_summary {
                let can_apply = summary.can_apply && !state.data_edit.applying;
                let apply_label = t("button_apply");
                let apply_button = if can_apply {
                    theme::primary_button(&apply_label)
                } else {
                    theme::secondary_button(&apply_label)
                };
                if ui.add_enabled(can_apply, apply_button).clicked() {
                    match build_data_edits(state) {
                        Ok(edits) => {
                            state.data_edit.applying = true;
                            state.last_error = None;
                            bridge.send(DbCommand::ApplyDataEdits {
                                conn_id: summary.conn_id,
                                edits,
                            });
                        }
                        Err(err) => {
                            state.last_error = Some(err);
                        }
                    }
                }

                ui.add_space(theme::SPACE_SM);

                if ui
                    .add_enabled(
                        !state.data_edit.applying,
                        theme::ghost_button(&t("grid_revert")),
                    )
                    .clicked()
                {
                    revert_data_edits(state);
                }

                ui.add_space(theme::SPACE_MD);
                metric_chip(
                    ui,
                    &tf("grid_edits", &[&summary.dirty_count.to_string()]),
                    summary.color,
                );

                if let Some(reason) = &summary.blocked_reason {
                    ui.label(RichText::new(reason).color(theme::ACCENT_YELLOW).size(11.0));
                }

                ui.add_space(theme::SPACE_LG);
            }

            let csv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::EXPORT,
                "export_csv",
                "CSV",
                true,
            );

            if csv_btn.clicked() {
                export_csv(state);
            }

            ui.add_space(theme::SPACE_SM);

            let tsv_btn = result_toolbar_action_button(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_tsv",
                "Copy TSV",
                true,
            );

            if tsv_btn.clicked() {
                if let Some(ref result) = state.current_result {
                    let tsv = result_to_tsv(result);
                    ui.ctx().copy_text(tsv);
                }
            }
        },
    );
}

fn result_meta_group_width(
    ui: &egui::Ui,
    row_count: usize,
    col_count: usize,
    exec_ms: u128,
    truncated: bool,
) -> f32 {
    let title_width = ui
        .painter()
        .layout_no_wrap(
            "Result".to_string(),
            egui::FontId::proportional(13.0),
            theme::text_primary(),
        )
        .rect
        .width();
    let row_text = format!(
        "{} {}",
        row_count,
        if row_count == 1 { "row" } else { "rows" }
    );
    let col_text = format!(
        "{} {}",
        col_count,
        if col_count == 1 { "col" } else { "cols" }
    );
    let mut width = title_width
        + theme::SPACE_MD
        + result_meta_chip_width(ui, &row_text)
        + result_meta_chip_width(ui, &col_text)
        + result_meta_chip_width(ui, &format!("{exec_ms}ms"))
        + theme::SPACE_SM * 4.0;
    if truncated {
        width += result_meta_chip_svg_width(ui, "trunc") + theme::SPACE_SM;
    }
    width
}

pub fn result_toolbar_action_width(ui: &egui::Ui, label: &str) -> f32 {
    let width = ui
        .painter()
        .layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(12.0),
            theme::text_secondary(),
        )
        .rect
        .width();
    (width + 38.0).max(58.0)
}

fn result_meta_chip_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width()
        + 18.0
}

fn result_meta_chip_svg_width(ui: &egui::Ui, text: &str) -> f32 {
    let text_width = ui
        .painter()
        .layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            theme::text_primary(),
        )
        .rect
        .width();
    (text_width + 34.0).max(74.0)
}

fn render_data_pager(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    has_next_page: bool,
    visible_rows: usize,
) {
    if state.active_main_view != MainView::Data {
        return;
    }

    let page_index = state.data_edit.page_index;
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    let page_start = if visible_rows == 0 { 0 } else { offset + 1 };
    let page_end = if visible_rows == 0 {
        0
    } else {
        offset + visible_rows
    };
    let page_label = tf("grid_page_n", &[&(page_index + 1).to_string()]);
    let limit_label = tf("grid_limit_n", &[&limit.to_string()]);
    let range_label = tf(
        "grid_visible_range",
        &[&page_start.to_string(), &page_end.to_string()],
    );

    egui::Frame::new()
        .fill(theme::bg_darkest())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(theme::SPACE_SM_I, theme::SPACE_XS_I))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
                let first = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_DOUBLE_LEFT,
                    "data_first_page",
                    &t("grid_first_page"),
                    page_index > 0 && !state.query_running,
                );
                if first.clicked() {
                    set_data_page_index(state, bridge, 0);
                }

                let prev = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_LEFT,
                    "data_prev_page",
                    &t("grid_prev_page"),
                    page_index > 0 && !state.query_running,
                );
                if prev.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_sub(1));
                }

                result_toolbar_menu_button(ui, "page", &page_label, 64.0, |ui| {
                    result_popup_field_row(ui, &t("grid_page"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_index_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_page_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_page_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                let next = pager_icon_button(
                    ui,
                    crate::ui::icons_svg::CHEVRON_RIGHT,
                    "data_next_page",
                    &t("grid_next_page"),
                    has_next_page && !state.query_running,
                );
                if next.clicked() {
                    set_data_page_index(state, bridge, page_index.saturating_add(1));
                }

                ui.add_space(theme::SPACE_SM);
                result_toolbar_menu_button(ui, "limit", &limit_label, 78.0, |ui| {
                    result_popup_field_row(ui, &t("grid_limit"), |ui| {
                        let response = ui.add(
                            theme::mono_text_input(&mut state.data_edit.page_limit_input)
                                .desired_width(ui.available_width()),
                        );
                        if response.lost_focus()
                            && enter_pressed(ui)
                            && apply_data_limit_input(state, bridge)
                        {
                            ui.memory_mut(|memory| memory.close_popup());
                        }
                    });
                    ui.add_space(theme::SPACE_SM);
                    if result_popup_apply_button(ui, &t("button_apply"), true).clicked()
                        && apply_data_limit_input(state, bridge)
                    {
                        ui.memory_mut(|memory| memory.close_popup());
                    }
                });

                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(range_label)
                        .color(theme::text_muted())
                        .size(11.0),
                );
            });
        });
}

fn pager_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
    enabled: bool,
) -> egui::Response {
    let color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(26.0, 26.0), enabled);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, color,
            ));
        },
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    response
}

pub fn result_toolbar_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    enabled: bool,
) -> egui::Response {
    let font = egui::FontId::proportional(12.0);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let width = (text_width + 38.0).max(58.0);
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icon_svg,
                icon_name,
                12.0,
                if enabled {
                    theme::ACCENT_TEAL
                } else {
                    theme::text_disabled()
                },
            ));
        },
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(28.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );
    response
}

fn result_toolbar_menu_button<R>(
    ui: &mut egui::Ui,
    id_source: impl Hash,
    label: &str,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    let popup_id = ui.make_persistent_id(("result_toolbar_menu", id_source));
    let response = result_toolbar_text_button(ui, label, width, true);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }
    show_dark_popup_below(
        ui,
        popup_id,
        &response,
        160.0,
        theme::SPACE_MD_I,
        add_contents,
    );
    response
}

pub fn show_dark_popup_below<R>(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    response: &egui::Response,
    min_width: f32,
    margin: i8,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) {
    if !ui.memory(|memory| memory.is_popup_open(popup_id)) {
        return;
    }

    let mut pos = response.rect.left_bottom() + egui::vec2(0.0, 4.0);
    if let Some(to_global) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        pos = to_global * pos;
    }
    let popup = egui::Area::new(popup_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(margin))
                .show(ui, |ui| {
                    ui.set_width(min_width);
                    add_contents(ui);
                });
        });

    let should_close = ui.input(|input| input.key_pressed(egui::Key::Escape))
        || (response.clicked_elsewhere() && popup.response.clicked_elsewhere());
    if should_close {
        ui.memory_mut(|memory| memory.close_popup());
    }
}

fn result_popup_field_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_field: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = COLUMN_GAP;
        ui.allocate_ui_with_layout(
            egui::vec2(LABEL_WIDTH, theme::INPUT_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    RichText::new(label)
                        .color(theme::text_secondary())
                        .size(11.5)
                        .strong(),
                );
            },
        );
        add_field(ui)
    })
    .inner
}

fn result_popup_apply_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    const LABEL_WIDTH: f32 = 44.0;
    const COLUMN_GAP: f32 = 8.0;

    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + COLUMN_GAP);
        let width = ui.available_width().max(68.0);
        result_popup_action_button(ui, label, enabled, width)
    })
    .inner
}

fn result_popup_action_button(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    width: f32,
) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 28.0), enabled);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );
    response
}

fn result_toolbar_text_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(width, 26.0), enabled);
    ui.painter().text(
        rect.center() - egui::vec2(4.0, 0.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(11.5),
        if enabled {
            theme::text_secondary()
        } else {
            theme::text_disabled()
        },
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(10.0, 1.0),
        egui::Align2::CENTER_CENTER,
        "⌄",
        egui::FontId::proportional(10.0),
        theme::text_muted(),
    );
    response
}

fn result_toolbar_button_frame(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    enabled: bool,
) -> (egui::Rect, egui::Response) {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let hovered = enabled && response.hovered();
    let fill = if !enabled {
        theme::bg_darkest()
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 150))
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    set_pointing_cursor_on_hover(ui, &response, enabled);
    (rect, response)
}

fn result_meta_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 18.0, 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    painter.circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 2.7, color);
    painter.text(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

fn result_meta_chip_svg(ui: &mut egui::Ui, text: &str, svg: &str, name: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2((galley.rect.width() + 34.0).max(74.0), 22.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        Stroke::new(1.0, theme::with_alpha(color, 48)),
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(11.0, 0.0),
        egui::vec2(12.0, 12.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(ui, svg, name, 12.0, color));
        },
    );
    painter.text(
        rect.left_center() + egui::vec2(22.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

pub fn metric_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 18.0, 20.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );
    ui.painter()
        .circle_filled(rect.left_center() + egui::vec2(9.0, 0.0), 2.5, color);
    ui.painter().text(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

// ---------------------------------------------------------------------------
// Header / Sort
// ---------------------------------------------------------------------------

fn render_header_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    column_name: &str,
    type_name: &str,
) {
    let cell_width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(cell_width, 26.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(GRID_CELL_LEFT_PAD);
            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.label(
                    RichText::new(column_name)
                        .color(theme::text_primary())
                        .strong()
                        .size(12.0),
                );
                ui.label(
                    RichText::new(type_name)
                        .color(theme::text_muted())
                        .size(9.5)
                        .monospace(),
                );
            });

            if state.active_main_view != MainView::Data {
                return;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_sort_menu(ui, state, bridge, column_name);
            });
        },
    );
}

fn render_sort_menu(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge, column_name: &str) {
    let sort_index = state
        .data_edit
        .sort
        .iter()
        .position(|clause| clause.column == column_name);
    let direction = sort_index.map(|idx| state.data_edit.sort[idx].direction);
    let (icon, icon_name, icon_color) = match direction {
        Some(DataSortDirection::Asc) => (
            crate::ui::icons_svg::SORT_ASC,
            "header_sort_asc",
            theme::ACCENT_TEAL,
        ),
        Some(DataSortDirection::Desc) => (
            crate::ui::icons_svg::SORT_DESC,
            "header_sort_desc",
            theme::ACCENT_COPPER_LIGHT,
        ),
        None => (
            crate::ui::icons_svg::SORT,
            "header_sort",
            theme::text_muted(),
        ),
    };
    let popup_id = ui.make_persistent_id(("header_sort_menu", column_name));
    let response = render_header_sort_button(ui, icon, icon_name, icon_color, sort_index);
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    show_dark_popup_below(ui, popup_id, &response, 184.0, theme::SPACE_SM_I, |ui| {
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT_ASC,
            "sort_menu_asc",
            &t("grid_sort_asc"),
            theme::ACCENT_TEAL,
            true,
            direction == Some(DataSortDirection::Asc),
        )
        .clicked()
        {
            set_sort_clause(state, bridge, column_name, DataSortDirection::Asc);
            ui.memory_mut(|memory| memory.close_popup());
        }
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT_DESC,
            "sort_menu_desc",
            &t("grid_sort_desc"),
            theme::ACCENT_TEAL,
            true,
            direction == Some(DataSortDirection::Desc),
        )
        .clicked()
        {
            set_sort_clause(state, bridge, column_name, DataSortDirection::Desc);
            ui.memory_mut(|memory| memory.close_popup());
        }
        sort_menu_separator(ui);
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::SORT,
            "sort_menu_remove",
            &t("grid_sort_remove"),
            theme::text_muted(),
            sort_index.is_some(),
            false,
        )
        .clicked()
        {
            remove_sort_clause(state, bridge, column_name);
            ui.memory_mut(|memory| memory.close_popup());
        }
        if sort_menu_item(
            ui,
            crate::ui::icons_svg::CLOSE,
            "sort_menu_clear",
            &t("grid_sort_clear_all"),
            theme::ACCENT_RED,
            !state.data_edit.sort.is_empty(),
            false,
        )
        .clicked()
        {
            clear_sort_clauses(state, bridge);
            ui.memory_mut(|memory| memory.close_popup());
        }
    });
}

fn render_header_sort_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    color: Color32,
    sort_index: Option<usize>,
) -> egui::Response {
    let (rect, response) = result_toolbar_button_frame(ui, egui::vec2(24.0, 24.0), true);
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(13.0, 13.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, color,
            ));
        },
    );

    if let Some(idx) = sort_index {
        let badge_rect = egui::Rect::from_center_size(
            rect.right_top() + egui::vec2(-4.0, 4.0),
            egui::vec2(11.0, 11.0),
        );
        ui.painter().circle_filled(badge_rect.center(), 5.5, color);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            (idx + 1).to_string(),
            egui::FontId::proportional(8.0),
            theme::bg_darkest(),
        );
    }

    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("grid_sort_asc"),
    );
    response
}

fn sort_menu_item(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    color: Color32,
    enabled: bool,
    selected: bool,
) -> egui::Response {
    let full_width = ui.available_width().max(184.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(full_width, 30.0), sense);
    let hovered = enabled && response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 26)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }

    let icon_color = if enabled {
        color
    } else {
        theme::text_disabled()
    };
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(13.0, 13.0),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.add(crate::ui::icon_image_tinted(
                ui, icon_svg, icon_name, 13.0, icon_color,
            ));
        },
    );

    let text_color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(32.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    if selected {
        ui.painter().circle_filled(
            rect.right_center() - egui::vec2(13.0, 0.0),
            3.0,
            theme::ACCENT_TEAL,
        );
    }

    set_pointing_cursor_on_hover(ui, &response, enabled);
    response
}

fn sort_menu_separator(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 9.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0, theme::border_default()),
    );
}

// ---------------------------------------------------------------------------
// Table rendering
// ---------------------------------------------------------------------------

pub fn render_table(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match state.current_result.clone() {
        Some(r) => r,
        None => return,
    };

    if result.columns.is_empty() {
        return;
    }

    let available_width = ui.available_width();
    let column_widths = compute_column_widths(ui, &result);
    let content_width = column_widths.iter().sum::<f32>().max(available_width);
    let row_height = 28.0;
    let header_height = 30.0;
    let header_bg = theme::bg_medium();

    ensure_foreign_keys_for_active_data_source(state, bridge);

    let table_id = grid_table_id(state, &result, &column_widths);
    egui::ScrollArea::horizontal()
        .id_salt(format!("grid_hscroll_{table_id}"))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_width);
            ui.scope(|ui| {
                apply_grid_table_visuals(ui);
                let mut table = TableBuilder::new(ui)
                    .id_salt(table_id)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

                for width in &column_widths {
                    table = table.column(
                        Column::initial(*width)
                            .clip(true)
                            .at_least(64.0)
                            .at_most(560.0),
                    );
                }

                table
                    .header(header_height, |mut header| {
                        for col in &result.columns {
                            header.col(|ui| {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(rect, 0.0, header_bg);
                                render_header_cell(ui, state, bridge, &col.name, &col.type_name);
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(row_height, result.rows.len(), |mut row| {
                            let row_idx = row.index();
                            let row_data = &result.rows[row_idx];
                            for (col_idx, cell) in row_data.iter().enumerate() {
                                row.col(|ui| {
                                    ui.add_space(GRID_CELL_LEFT_PAD);
                                    if state.active_main_view == MainView::Data {
                                        let column = result.columns.get(col_idx);
                                        render_editable_cell(
                                            ui, state, bridge, row_idx, col_idx, cell, column,
                                        );
                                    } else {
                                        render_cell(ui, cell);
                                    }
                                });
                            }
                        });
                    });
            });
        });
}

fn apply_grid_table_visuals(ui: &mut egui::Ui) {
    let mut style = (**ui.style()).clone();
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, grid_separator_color());
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, grid_separator_hover_color());
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, grid_separator_active_color());
    ui.set_style(style);
}

fn grid_separator_color() -> Color32 {
    if theme::is_dark() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 24)
    } else {
        theme::border_default()
    }
}

fn grid_separator_hover_color() -> Color32 {
    if theme::is_dark() {
        Color32::from_rgba_unmultiplied(255, 255, 255, 54)
    } else {
        theme::border_strong()
    }
}

fn grid_separator_active_color() -> Color32 {
    if theme::is_dark() {
        theme::with_alpha(theme::ACCENT_TEAL, 150)
    } else {
        theme::ACCENT_TEAL
    }
}

fn compute_column_widths(ui: &egui::Ui, result: &crate::types::QueryResult) -> Vec<f32> {
    result
        .columns
        .iter()
        .enumerate()
        .map(|(col_idx, column)| {
            let header_width = measure_text_width(
                ui,
                &format!("{}  {}", column.name, column.type_name),
                egui::FontId::proportional(12.0),
            ) + 58.0;

            let max_sample_width = result
                .rows
                .iter()
                .take(80)
                .filter_map(|row| row.get(col_idx))
                .map(|cell| {
                    let sample = cell_auto_width_text(cell);
                    let font = if matches!(cell, CellValue::Text(_)) {
                        egui::FontId::proportional(12.0)
                    } else {
                        egui::FontId::monospace(12.0)
                    };
                    measure_text_width(ui, &sample, font) + cell_width_padding(cell)
                })
                .fold(0.0_f32, f32::max);

            let base = header_width.max(max_sample_width);
            let max_width = column_width_cap(&column.type_name);
            base.clamp(72.0, max_width)
        })
        .collect()
}

fn measure_text_width(ui: &egui::Ui, text: &str, font_id: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_string(), font_id, theme::text_primary())
        .rect
        .width()
}

fn cell_auto_width_text(cell: &CellValue) -> String {
    let text = cell.to_string();
    const MAX_SAMPLE_CHARS: usize = 96;
    if text.chars().count() <= MAX_SAMPLE_CHARS {
        text
    } else {
        let mut truncated = text.chars().take(MAX_SAMPLE_CHARS).collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

fn cell_width_padding(cell: &CellValue) -> f32 {
    match cell {
        CellValue::Bool(_) | CellValue::Null => 42.0,
        CellValue::Int(_) | CellValue::Float(_) => 32.0,
        CellValue::Uuid(_) => 26.0,
        CellValue::Timestamp(_) => 34.0,
        CellValue::Json(_) | CellValue::Bytes(_) => 46.0,
        CellValue::Text(_) | CellValue::Unknown(_) => 34.0,
    }
}

fn column_width_cap(type_name: &str) -> f32 {
    match type_name.to_ascii_lowercase().as_str() {
        "uuid" => 310.0,
        "bool" | "boolean" => 110.0,
        "int2" | "int4" | "int8" | "smallint" | "integer" | "bigint" | "numeric" | "decimal"
        | "float4" | "float8" | "real" | "double precision" => 150.0,
        "date"
        | "timestamp"
        | "timestamptz"
        | "timestamp without time zone"
        | "timestamp with time zone" => 230.0,
        "json" | "jsonb" => 520.0,
        "bytea" => 360.0,
        _ => 420.0,
    }
}

fn grid_table_id(
    state: &AppState,
    result: &crate::types::QueryResult,
    column_widths: &[f32],
) -> String {
    let source = state
        .active_data_source()
        .map(|source| {
            let filter = source
                .filter
                .as_ref()
                .map(|filter| format!("_{}_{}", filter.column, filter.sql_value))
                .unwrap_or_default();
            format!(
                "{}_{}_{}{}",
                source.conn_id, source.schema, source.table, filter
            )
        })
        .unwrap_or_else(|| "query_result".to_string());
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    for column in &result.columns {
        column.name.hash(&mut hasher);
        column.type_name.hash(&mut hasher);
    }
    for width in column_widths {
        (*width as u32).hash(&mut hasher);
    }
    format!("grid_{:x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// Cell rendering
// ---------------------------------------------------------------------------

pub fn render_cell(ui: &mut egui::Ui, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 18.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(theme::RADIUS_MD),
                theme::with_alpha(theme::text_muted(), 24),
            );
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect.shrink(2.0)), |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::NULL_MARKER, "null", 12.0);
            });
            show_dark_hover_tooltip(ui, resp.id.with("tooltip"), &resp, &t("grid_null_value"));
        }
        CellValue::Bool(v) => {
            let (text, color) = if *v {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            value_pill(ui, text, color);
        }
        CellValue::Json(v) => {
            render_copyable_cell(ui, &v.to_string(), theme::ACCENT_TEAL);
        }
        CellValue::Timestamp(v) => {
            render_copyable_cell(ui, v, theme::ACCENT_BLUE);
        }
        CellValue::Uuid(v) => {
            render_copyable_cell(ui, &v.to_string(), theme::ACCENT_COPPER_LIGHT);
        }
        CellValue::Bytes(v) => {
            render_copyable_cell(ui, &format!("\\x{}", hex_encode(v)), theme::text_muted());
        }
        other => {
            let text = other.to_string();
            render_copyable_cell(ui, &text, theme::text_primary());
        }
    }
}

pub fn render_passive_cell(ui: &mut egui::Ui, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            passive_value_pill(ui, "NULL", theme::text_muted());
        }
        CellValue::Bool(v) => {
            let (text, color) = if *v {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            passive_value_pill(ui, text, color);
        }
        CellValue::Json(v) => {
            render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_TEAL);
        }
        CellValue::Timestamp(v) => {
            render_passive_copyable_cell(ui, v, theme::ACCENT_BLUE);
        }
        CellValue::Uuid(v) => {
            render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_COPPER_LIGHT);
        }
        CellValue::Bytes(v) => {
            render_passive_copyable_cell(ui, &format!("\\x{}", hex_encode(v)), theme::text_muted());
        }
        other => {
            let text = other.to_string();
            render_passive_copyable_cell(ui, &text, theme::text_primary());
        }
    }
}

fn value_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    value_pill_with_interaction(ui, text, color, true);
}

pub fn passive_value_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    value_pill_with_interaction(ui, text, color, false);
}

fn value_pill_with_interaction(ui: &mut egui::Ui, text: &str, color: Color32, interactive: bool) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), egui::FontId::monospace(11.0), color);
    let sense = if interactive {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(galley.rect.width() + 12.0, 18.0), sense);
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::with_alpha(color, if resp.hovered() { 38 } else { 24 }),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::monospace(11.0),
        color,
    );
    if interactive {
        show_cell_copy_context_menu(&resp, text);
    }
}

fn render_copyable_cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    render_copyable_cell_with_interaction(ui, text, color, true);
}

pub fn render_passive_copyable_cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    render_copyable_cell_with_interaction(ui, text, color, false);
}

fn render_copyable_cell_with_interaction(
    ui: &mut egui::Ui,
    text: &str,
    color: Color32,
    interactive: bool,
) {
    let font = egui::FontId::monospace(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font.clone(), color);
    let available_width = ui.available_width().max(1.0);
    let width = galley.rect.width().min(available_width).max(1.0);
    let sense = if interactive {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 24.0), sense);
    let text_rect = rect.shrink2(egui::vec2(0.0, 1.0));
    ui.painter().with_clip_rect(text_rect).text(
        text_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
    if galley.rect.width() > text_rect.width() + 1.0 {
        show_dark_hover_tooltip(ui, resp.id.with("full_value"), &resp, text);
    }
    if interactive {
        show_cell_copy_context_menu(&resp, text);
    }
}

pub fn show_cell_copy_context_menu(response: &egui::Response, text: &str) {
    response.context_menu(|ui| {
        let copy_resp = ui.add(theme::ghost_icon_button(
            crate::ui::icon_image_tinted(
                ui,
                crate::ui::icons_svg::COPY,
                "copy_cell_v",
                10.0,
                theme::ACCENT_BLUE,
            ),
            t("grid_copy_value"),
        ));
        if copy_resp.clicked() {
            ui.ctx().copy_text(text.to_string());
            ui.close_menu();
        }
    });
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
