//! Table painting — `render_table` + column-width math + cell rendering.
//!
//! Extracted from `render.rs` to keep that file ≤800 lines. Pure refactor;
//! behavior unchanged. The high-level `render_grid` orchestrator stays in
//! `render.rs` and calls back into this module.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{AppState, MainView};
use crate::types::CellValue;
use crate::ui::theme;

use super::render::render_header_cell;
use super::*;

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
    let row_number_width = row_number_gutter_width(result.rows.len());
    let content_width =
        (row_number_width + column_widths.iter().sum::<f32>()).max(available_width);
    let row_height = 26.0;
    let header_height = 28.0;
    let header_bg = theme::bg_shell();

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

                // Row number gutter column (fixed, non-resizable)
                table = table.column(
                    Column::exact(row_number_width).clip(true),
                );

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
                        // Row number header
                        header.col(|ui| {
                            let rect = ui.available_rect_before_wrap();
                            ui.painter().rect_filled(rect, 0.0, header_bg);
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("#")
                                        .color(theme::text_muted())
                                        .size(11.0)
                                        .monospace(),
                                );
                            });
                        });
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
                            let is_deleted = state.data_edit.pending_deletes.contains(&row_idx);
                            let is_inserted = state.data_edit.inserted_rows.contains(&row_idx);
                            let row_data = &result.rows[row_idx];

                            // Row number cell
                            row.col(|ui| {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(rect, 0.0, theme::bg_shell());
                                let label = if is_inserted { "*" } else { "" };
                                let num_text = format!("{}{}", row_idx + 1, label);
                                let color = if is_deleted {
                                    theme::ACCENT_RED
                                } else if is_inserted {
                                    theme::accent_color()
                                } else {
                                    theme::text_disabled()
                                };
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(num_text)
                                            .color(color)
                                            .size(10.5)
                                            .monospace(),
                                    );
                                });
                            });

                            for (col_idx, cell) in row_data.iter().enumerate() {
                                row.col(|ui| {
                                    if is_deleted {
                                        ui.set_opacity(0.35);
                                    }
                                    let cell_rect = ui.available_rect_before_wrap();
                                    if !is_deleted && ui.rect_contains_pointer(cell_rect) {
                                        ui.painter().rect_filled(
                                            cell_rect,
                                            0.0,
                                            theme::with_alpha(Color32::WHITE, 4),
                                        );
                                    }
                                    ui.add_space(GRID_CELL_LEFT_PAD);
                                    if state.active_main_view == MainView::Data && !is_deleted {
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

                // Empty area below rows — click to deselect
                let remaining = ui.available_size();
                if remaining.y > 0.0 {
                    let (rect, resp) = ui.allocate_exact_size(remaining, egui::Sense::click());
                    ui.painter().rect_filled(rect, 0.0, theme::bg_darkest());
                    if resp.clicked() {
                        state.data_edit.selected_cell = None;
                        state.data_edit.editing_cell = None;
                    }
                }
            });
        });
}

fn row_number_gutter_width(_row_count: usize) -> f32 {
    48.0
}

fn apply_grid_table_visuals(ui: &mut egui::Ui) {
    let sep = Stroke::new(0.5, grid_separator_color());
    let mut style = (**ui.style()).clone();
    style.visuals.widgets.noninteractive.bg_stroke = sep;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(0.5, grid_separator_hover_color());
    style.visuals.widgets.active.bg_stroke = Stroke::new(0.5, grid_separator_active_color());
    style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    ui.set_style(style);
}

fn grid_separator_color() -> Color32 {
    theme::border_subtle()
}

fn grid_separator_hover_color() -> Color32 {
    if theme::is_dark() {
        theme::with_alpha(Color32::WHITE, 30)
    } else {
        theme::border_default()
    }
}

fn grid_separator_active_color() -> Color32 {
    if theme::is_dark() {
        theme::with_alpha(Color32::WHITE, 50)
    } else {
        theme::border_strong()
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
            render_copyable_cell(ui, &v.to_string(), theme::ACCENT_PURPLE);
        }
        CellValue::Timestamp(v) => {
            render_copyable_cell(ui, v, theme::text_secondary());
        }
        CellValue::Int(v) => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_copyable_cell(ui, &v.to_string(), theme::ACCENT_YELLOW);
            });
        }
        CellValue::Float(v) => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_copyable_cell(ui, &v.to_string(), theme::ACCENT_YELLOW);
            });
        }
        CellValue::Uuid(v) => {
            render_copyable_cell(ui, &v.to_string(), theme::text_muted());
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
            render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_PURPLE);
        }
        CellValue::Timestamp(v) => {
            render_passive_copyable_cell(ui, v, theme::text_secondary());
        }
        CellValue::Int(v) => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_YELLOW);
            });
        }
        CellValue::Float(v) => {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_passive_copyable_cell(ui, &v.to_string(), theme::ACCENT_YELLOW);
            });
        }
        CellValue::Uuid(v) => {
            render_passive_copyable_cell(ui, &v.to_string(), theme::text_muted());
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
