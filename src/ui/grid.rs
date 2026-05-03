use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, cell_edit_text_for_type, data_timezone_offset_seconds,
    is_timestamptz_type, timestamp_display_to_utc, AppState, DataSortClause, DataSortDirection,
    MainView, MAX_DATA_PAGE_LIMIT,
};
use crate::types::{CellValue, ColumnInfo, DataCellEdit, DataEditValue, DataKeyValue};
use crate::ui::theme;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_grid(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if let Some(ref error) = state.last_error.clone() {
        render_error_bar(ui, error);
    }

    match &state.current_result {
        None => render_empty_state(ui, state.query_running),
        Some(_) => {
            render_result_header(ui, state, bridge);
            render_table(ui, state, bridge);
        }
    }
}

// ---------------------------------------------------------------------------
// Error bar
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Empty / loading state
// ---------------------------------------------------------------------------

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

fn render_result_header(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;
    let data_edit_summary = data_edit_summary(state);

    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Result")
                    .color(theme::text_primary())
                    .strong()
                    .size(13.0),
            );
            ui.add_space(theme::SPACE_MD);
            metric_chip(
                ui,
                &format!(
                    "{} {}",
                    row_count,
                    if row_count == 1 { "row" } else { "rows" }
                ),
                theme::ACCENT_TEAL,
            );
            metric_chip(
                ui,
                &format!(
                    "{} {}",
                    col_count,
                    if col_count == 1 { "col" } else { "cols" }
                ),
                theme::ACCENT_BLUE,
            );
            metric_chip(ui, &format!("{exec_ms}ms"), theme::ACCENT_COPPER);

            if truncated {
                metric_chip_svg(
                    ui,
                    "truncated",
                    crate::ui::icons_svg::TRUNCATED,
                    "truncated_icon",
                    theme::ACCENT_YELLOW,
                );
            }

            render_data_pager(ui, state, bridge, truncated, row_count);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(summary) = &data_edit_summary {
                    let can_apply = summary.can_apply && !state.data_edit.applying;
                    if ui
                        .add_enabled(can_apply, theme::primary_button(&t("button_apply")))
                        .clicked()
                    {
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

                let csv_btn = ui.add(theme::ghost_button("      CSV"));
                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(
                        csv_btn
                            .rect
                            .shrink2(egui::vec2(csv_btn.rect.width() - 20.0, 0.0)),
                    ),
                    |ui| {
                        crate::ui::icon_img(ui, crate::ui::icons_svg::EXPORT, "export_csv", 12.0);
                    },
                );

                if csv_btn.clicked() {
                    export_csv(state);
                }

                ui.add_space(theme::SPACE_SM);

                let tsv_btn = ui.add(theme::ghost_button("      Copy TSV"));
                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(
                        tsv_btn
                            .rect
                            .shrink2(egui::vec2(tsv_btn.rect.width() - 20.0, 0.0)),
                    ),
                    |ui| {
                        crate::ui::icon_img(ui, crate::ui::icons_svg::COPY, "copy_tsv", 12.0);
                    },
                );

                if tsv_btn.clicked() {
                    if let Some(ref result) = state.current_result {
                        let tsv = result_to_tsv(result);
                        ui.ctx().copy_text(tsv);
                    }
                }
            });
        });
    });
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

    ui.add_space(theme::SPACE_LG);

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

    ui.menu_button(
        RichText::new(page_label).color(theme::text_secondary()),
        |ui| {
            let mut page_value = (state.data_edit.page_index + 1) as i64;
            ui.horizontal(|ui| {
                ui.label(t("grid_page"));
                ui.add(
                    egui::DragValue::new(&mut page_value)
                        .range(1..=1_000_000_i64)
                        .speed(1)
                        .fixed_decimals(0),
                );
            });
            ui.add_space(theme::SPACE_SM);
            if ui.button(t("button_apply")).clicked() {
                set_data_page_index(state, bridge, page_value.max(1) as usize - 1);
                ui.close_menu();
            }
        },
    );

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

    ui.menu_button(
        RichText::new(limit_label).color(theme::text_secondary()),
        |ui| {
            ui.horizontal(|ui| {
                ui.label(t("grid_limit"));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.data_edit.page_limit_input)
                        .font(egui::FontId::monospace(12.0))
                        .desired_width(96.0),
                );
                if response.lost_focus() && enter_pressed(ui) {
                    if apply_data_limit_input(state, bridge) {
                        ui.close_menu();
                    }
                }
            });
            ui.add_space(theme::SPACE_SM);
            if ui.button(t("button_apply")).clicked() {
                if apply_data_limit_input(state, bridge) {
                    ui.close_menu();
                }
            }
        },
    );

    ui.label(
        RichText::new(range_label)
            .color(theme::text_muted())
            .size(11.0),
    );
}

fn pager_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
    enabled: bool,
) -> egui::Response {
    let response = ui
        .add_enabled(enabled, theme::ghost_button("     "))
        .on_hover_text(tooltip);
    let color = if enabled {
        theme::text_secondary()
    } else {
        theme::text_disabled()
    };
    let icon_rect = egui::Rect::from_center_size(response.rect.center(), egui::vec2(13.0, 13.0));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
        crate::ui::icon_img_tinted(ui, icon_svg, icon_name, 13.0, color);
    });
    response
}

fn metric_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
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

fn metric_chip_svg(ui: &mut egui::Ui, text: &str, svg: &str, name: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::text_primary(),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 24.0, 20.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::with_alpha(color, 24),
    );

    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            rect.left_center() + egui::vec2(10.0, 0.0),
            egui::vec2(12.0, 12.0),
        )),
        |ui| {
            crate::ui::icon_img(ui, svg, name, 10.0);
        },
    );

    ui.painter().text(
        rect.left_center() + egui::vec2(18.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(11.0),
        theme::text_secondary(),
    );
}

// ---------------------------------------------------------------------------
// Result table
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
            ui.add_space(theme::SPACE_SM);
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
    let sort_text = sort_index
        .map(|idx| format!("  {}", idx + 1))
        .unwrap_or_else(|| "   ".to_string());

    let sort_response = ui.menu_button(
        RichText::new(sort_text).color(icon_color).size(11.0),
        |ui| {
            if icon_menu_button(
                ui,
                crate::ui::icons_svg::SORT_ASC,
                "sort_menu_asc",
                &t("grid_sort_asc"),
                theme::ACCENT_TEAL,
                true,
            )
            .clicked()
            {
                set_sort_clause(state, bridge, column_name, DataSortDirection::Asc);
                ui.close_menu();
            }
            if icon_menu_button(
                ui,
                crate::ui::icons_svg::SORT_DESC,
                "sort_menu_desc",
                &t("grid_sort_desc"),
                theme::ACCENT_COPPER_LIGHT,
                true,
            )
            .clicked()
            {
                set_sort_clause(state, bridge, column_name, DataSortDirection::Desc);
                ui.close_menu();
            }
            ui.separator();
            if icon_menu_button(
                ui,
                crate::ui::icons_svg::SORT,
                "sort_menu_remove",
                &t("grid_sort_remove"),
                theme::text_muted(),
                sort_index.is_some(),
            )
            .clicked()
            {
                remove_sort_clause(state, bridge, column_name);
                ui.close_menu();
            }
            if icon_menu_button(
                ui,
                crate::ui::icons_svg::CLOSE,
                "sort_menu_clear",
                &t("grid_sort_clear_all"),
                theme::ACCENT_RED,
                !state.data_edit.sort.is_empty(),
            )
            .clicked()
            {
                clear_sort_clauses(state, bridge);
                ui.close_menu();
            }
        },
    );

    let icon_rect = egui::Rect::from_center_size(
        sort_response.response.rect.left_center() + egui::vec2(13.0, 0.0),
        egui::vec2(13.0, 13.0),
    );
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
        crate::ui::icon_img_tinted(ui, icon, icon_name, 13.0, icon_color);
    });
}

fn icon_menu_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    label: &str,
    color: Color32,
    enabled: bool,
) -> egui::Response {
    let response = ui.add_enabled(enabled, egui::Button::new(format!("      {label}")));
    let icon_rect = egui::Rect::from_center_size(
        response.rect.left_center() + egui::vec2(14.0, 0.0),
        egui::vec2(13.0, 13.0),
    );
    let icon_color = if enabled {
        color
    } else {
        theme::text_disabled()
    };
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
        crate::ui::icon_img_tinted(ui, icon_svg, icon_name, 13.0, icon_color);
    });
    response
}

fn render_table(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
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

    let table_id = grid_table_id(state, &result, &column_widths);
    egui::ScrollArea::horizontal()
        .id_salt(format!("grid_hscroll_{table_id}"))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(content_width);
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
                                ui.add_space(theme::SPACE_SM);
                                if state.active_main_view == MainView::Data {
                                    let column = result.columns.get(col_idx);
                                    render_editable_cell(ui, state, row_idx, col_idx, cell, column);
                                } else {
                                    render_cell(ui, cell);
                                }
                            });
                        }
                    });
                });
        });
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
        .map(|source| format!("{}_{}_{}", source.conn_id, source.schema, source.table))
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

fn render_editable_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    column: Option<&crate::types::ColumnMeta>,
) {
    if !has_table_column_metadata(state) {
        render_cell(ui, fallback_cell);
        return;
    }

    if !state.data_edit.cells.contains_key(&(row_idx, col_idx)) {
        state.data_edit.cells.insert(
            (row_idx, col_idx),
            crate::state::EditableCell::from_cell_for_type(
                fallback_cell,
                column.map(|col| col.type_name.as_str()).unwrap_or_default(),
                &state.data_timezone,
            ),
        );
    }

    let column_info = column.and_then(|col| data_column_info(state, &col.name).cloned());
    if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
        render_cell(ui, fallback_cell);
        return;
    }

    let cell_key = (row_idx, col_idx);
    let type_name = column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .or_else(|| column.map(|col| col.type_name.clone()))
        .unwrap_or_default();
    let nullable = column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();

    let Some(snapshot) = state.data_edit.cells.get(&cell_key).cloned() else {
        render_cell(ui, fallback_cell);
        return;
    };

    let dirty = snapshot.is_dirty();
    let error = validate_edit_value(&snapshot, &type_name, nullable, &enum_values);
    let rect = ui.available_rect_before_wrap();
    if dirty {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_COPPER, 30),
        );
    } else if error.is_some() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_RED, 28),
        );
    }

    let is_editing = state.data_edit.editing_cell == Some(cell_key);
    if !is_editing {
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("data_cell", row_idx, col_idx)),
            egui::Sense::click(),
        );
        ui.horizontal(|ui| {
            render_editable_display_cell(ui, &snapshot, fallback_cell, &type_name);
            if dirty {
                ui.add_space(2.0);
                ui.painter().circle_filled(
                    ui.cursor().left_center() + egui::vec2(4.0, 0.0),
                    2.0,
                    theme::ACCENT_COPPER,
                );
                ui.add_space(8.0);
            }
        });
        if response.clicked() {
            state.data_edit.editing_cell = Some(cell_key);
        }
        if let Some(error) = error {
            response.on_hover_text(error);
        }
        return;
    }

    let mut close_editor = false;
    let data_timezone = state.data_timezone.clone();
    let Some(edit) = state.data_edit.cells.get_mut(&cell_key) else {
        render_cell(ui, fallback_cell);
        return;
    };

    ui.horizontal(|ui| {
        if nullable {
            let null_resp = ui
                .selectable_label(edit.is_null, RichText::new("NULL").size(9.5))
                .on_hover_text(t("grid_toggle_null"));
            if null_resp.clicked() {
                edit.is_null = !edit.is_null;
            }
            ui.add_space(2.0);
        }

        if edit.is_null {
            value_pill(ui, "NULL", theme::text_muted());
            return;
        }

        if !enum_values.is_empty() {
            close_editor |= render_enum_editor(ui, edit, row_idx, col_idx, &enum_values);
            return;
        }

        match edit_kind(&type_name, fallback_cell) {
            EditKind::Bool => {
                let mut checked = parse_bool(&edit.value).unwrap_or(false);
                if ui.checkbox(&mut checked, "").changed() {
                    edit.value = checked.to_string();
                }
            }
            EditKind::Date => {
                close_editor |=
                    render_date_editor(ui, edit, false, &data_timezone, error.as_deref());
            }
            EditKind::DateTime => {
                close_editor |=
                    render_date_editor(ui, edit, true, &data_timezone, error.as_deref());
            }
            EditKind::Number => {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut edit.value)
                        .font(egui::FontId::monospace(12.0))
                        .desired_width(ui.available_width().max(72.0)),
                );
                close_editor |= response.lost_focus() && enter_pressed(ui);
                if let Some(error) = &error {
                    response.on_hover_text(error);
                }
            }
            EditKind::Json => {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut edit.value)
                        .font(egui::FontId::monospace(12.0))
                        .desired_width(ui.available_width().max(120.0)),
                );
                close_editor |= response.lost_focus() && enter_pressed(ui);
                if let Some(error) = &error {
                    response.on_hover_text(error);
                }
            }
            EditKind::Uuid | EditKind::Bytes | EditKind::Text => {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut edit.value)
                        .font(egui::FontId::monospace(12.0))
                        .desired_width(ui.available_width().max(90.0)),
                );
                close_editor |= response.lost_focus() && enter_pressed(ui);
                if let Some(error) = &error {
                    response.on_hover_text(error);
                }
            }
        }
    });

    if close_editor || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.data_edit.editing_cell = None;
    }
}

fn render_editable_display_cell(
    ui: &mut egui::Ui,
    edit: &crate::state::EditableCell,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    if edit.is_null {
        value_pill(ui, "NULL", theme::text_muted());
        return;
    }

    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let value = parse_bool(&edit.value).unwrap_or(false);
            let (text, color) = if value {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            value_pill(ui, text, color);
        }
        EditKind::Number => render_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT),
        EditKind::Json => render_copyable_cell(ui, &edit.value, theme::ACCENT_TEAL),
        EditKind::Date | EditKind::DateTime => {
            render_copyable_cell(ui, &edit.value, theme::ACCENT_BLUE)
        }
        EditKind::Uuid => render_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT),
        EditKind::Bytes => render_copyable_cell(ui, &edit.value, theme::text_muted()),
        EditKind::Text => render_copyable_cell(ui, &edit.value, theme::text_primary()),
    }
}

fn render_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) -> bool {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    let mut changed = false;
    egui::ComboBox::from_id_salt(("enum_cell", row_idx, col_idx))
        .width(ui.available_width().max(96.0))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for value in enum_values {
                if ui
                    .selectable_value(&mut edit.value, value.clone(), value)
                    .clicked()
                {
                    changed = true;
                }
            }
        });
    changed
}

fn render_date_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    include_time: bool,
    timezone: &str,
    error: Option<&str>,
) -> bool {
    let (mut date, mut time) = split_datetime_value(&edit.value);
    let date_response = ui.add(
        egui::TextEdit::singleline(&mut date)
            .font(egui::FontId::monospace(12.0))
            .desired_width(86.0)
            .hint_text("YYYY-MM-DD"),
    );

    let mut close_editor = date_response.lost_focus() && enter_pressed(ui);

    if include_time {
        ui.label(RichText::new("·").color(theme::text_muted()));
        let time_response = ui.add(
            egui::TextEdit::singleline(&mut time)
                .font(egui::FontId::monospace(12.0))
                .desired_width(74.0)
                .hint_text("HH:MM:SS"),
        );
        close_editor |= time_response.lost_focus() && enter_pressed(ui);
        if time_response.changed() || date_response.changed() {
            edit.value = format!("{} {}", date.trim(), time.trim())
                .trim()
                .to_string();
        }
        if let Some(error) = error {
            time_response.on_hover_text(error);
        }
    } else if date_response.changed() {
        edit.value = date.trim().to_string();
    }

    if ui.small_button(t("grid_now")).clicked() {
        let now_utc = chrono::Utc::now();
        edit.value = if include_time {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d").to_string())
        };
        close_editor = true;
    }

    if let Some(error) = error {
        date_response.on_hover_text(error);
    }

    close_editor
}

fn split_datetime_value(value: &str) -> (String, String) {
    let value = value.trim();
    if value.is_empty() {
        return ("".to_string(), "".to_string());
    }

    if let Some((date, time)) = value.split_once(' ') {
        return (date.to_string(), time.to_string());
    }
    if let Some((date, time)) = value.split_once('T') {
        return (date.to_string(), time.trim_end_matches('Z').to_string());
    }
    (value.to_string(), "00:00:00".to_string())
}

fn enter_pressed(ui: &egui::Ui) -> bool {
    ui.input(|i| i.key_pressed(egui::Key::Enter))
}

fn render_cell(ui: &mut egui::Ui, cell: &CellValue) {
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
            resp.on_hover_text(t("grid_null_value"));
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

fn value_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_string(), egui::FontId::monospace(11.0), color);
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(galley.rect.width() + 12.0, 18.0),
        egui::Sense::click(),
    );
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
    resp.context_menu(|ui| {
        let copy_resp = ui.button(format!("      {}", t("grid_copy_value")));
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                copy_resp
                    .rect
                    .shrink2(egui::vec2(copy_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::COPY, "copy_cell", 10.0);
            },
        );
        if copy_resp.clicked() {
            ui.ctx().copy_text(text.to_string());
            ui.close_menu();
        }
    });
}

fn render_copyable_cell(ui: &mut egui::Ui, text: &str, color: Color32) {
    let resp = ui.label(
        RichText::new(text)
            .font(egui::FontId::monospace(12.0))
            .color(color),
    );
    resp.on_hover_text(text).context_menu(|ui| {
        let copy_resp = ui.button(format!("      {}", t("grid_copy_value")));
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(
                copy_resp
                    .rect
                    .shrink2(egui::vec2(copy_resp.rect.width() - 20.0, 0.0)),
            ),
            |ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::COPY, "copy_cell_v", 10.0);
            },
        );
        if copy_resp.clicked() {
            ui.ctx().copy_text(text.to_string());
            ui.close_menu();
        }
    });
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn set_sort_clause(
    state: &mut AppState,
    bridge: &DbBridge,
    column: &str,
    direction: DataSortDirection,
) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    if let Some(clause) = state
        .data_edit
        .sort
        .iter_mut()
        .find(|clause| clause.column == column)
    {
        clause.direction = direction;
    } else {
        state.data_edit.sort.push(DataSortClause {
            column: column.to_string(),
            direction,
        });
    }
    state.data_edit.page_index = 0;
    reload_data_source(state, bridge);
}

fn remove_sort_clause(state: &mut AppState, bridge: &DbBridge, column: &str) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state
        .data_edit
        .sort
        .retain(|clause| clause.column != column);
    state.data_edit.page_index = 0;
    reload_data_source(state, bridge);
}

fn clear_sort_clauses(state: &mut AppState, bridge: &DbBridge) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state.data_edit.sort.clear();
    state.data_edit.page_index = 0;
    reload_data_source(state, bridge);
}

fn set_data_page_index(state: &mut AppState, bridge: &DbBridge, page_index: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    state.data_edit.page_index = page_index;
    reload_data_source(state, bridge);
}

fn apply_data_limit_input(state: &mut AppState, bridge: &DbBridge) -> bool {
    let raw = state.data_edit.page_limit_input.trim().replace(',', "");
    match raw.parse::<usize>() {
        Ok(limit) => {
            set_data_limit(state, bridge, limit);
            true
        }
        Err(_) => {
            state.last_error = Some(t("grid_limit_error"));
            false
        }
    }
}

fn set_data_limit(state: &mut AppState, bridge: &DbBridge, limit: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    let limit = limit.clamp(1, MAX_DATA_PAGE_LIMIT);
    state.data_edit.page_limit = limit;
    state.data_edit.page_limit_input = limit.to_string();
    state.data_edit.page_index = 0;
    reload_data_source(state, bridge);
}

fn reload_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    state.last_error = None;
    bridge.send(DbCommand::ExecuteQuery {
        conn_id: source.conn_id,
        sql: build_data_select_sql_with_columns(
            &source,
            &state.data_edit.sort,
            limit,
            offset,
            &state.data_columns_for_source(&source),
        ),
        row_limit: Some(limit),
    });
}

fn normalized_data_limit(state: &AppState) -> usize {
    state.data_edit.page_limit.clamp(1, MAX_DATA_PAGE_LIMIT)
}

fn data_page_offset(state: &AppState) -> usize {
    state
        .data_edit
        .page_index
        .saturating_mul(normalized_data_limit(state))
}

fn has_dirty_data_edits(state: &AppState) -> bool {
    state.data_edit.cells.values().any(|cell| cell.is_dirty())
}

#[derive(Clone, Copy)]
enum EditKind {
    Bool,
    Number,
    Json,
    Date,
    DateTime,
    Uuid,
    Bytes,
    Text,
}

struct DataEditSummary {
    conn_id: crate::types::ConnectionId,
    dirty_count: usize,
    can_apply: bool,
    blocked_reason: Option<String>,
    color: Color32,
}

fn data_edit_summary(state: &AppState) -> Option<DataEditSummary> {
    if state.active_main_view != MainView::Data {
        return None;
    }

    let source = state.active_data_source()?;
    let dirty_count = state
        .data_edit
        .cells
        .values()
        .filter(|cell| cell.is_dirty())
        .count();
    if dirty_count == 0 {
        return None;
    }

    let pk_columns = primary_key_columns(state);
    let invalid_count = count_invalid_edits(state);
    let blocked_reason = if pk_columns.is_empty() {
        Some(t("grid_pk_required"))
    } else if invalid_count > 0 {
        Some(tf("grid_invalid_values", &[&invalid_count.to_string()]))
    } else {
        None
    };
    let can_apply = blocked_reason.is_none() && !state.data_edit.applying;
    let color = if blocked_reason.is_some() {
        theme::ACCENT_YELLOW
    } else {
        theme::ACCENT_COPPER
    };

    Some(DataEditSummary {
        conn_id: source.conn_id,
        dirty_count,
        can_apply,
        blocked_reason,
        color,
    })
}

fn data_column_info<'a>(state: &'a AppState, column_name: &str) -> Option<&'a ColumnInfo> {
    let source = state.active_data_source()?;
    state
        .connections
        .get(&source.conn_id)?
        .columns
        .get(&(source.schema, source.table))?
        .iter()
        .find(|col| col.name == column_name)
}

fn has_table_column_metadata(state: &AppState) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .is_some()
}

fn table_columns(state: &AppState) -> Vec<ColumnInfo> {
    let Some(source) = state.active_data_source() else {
        return Vec::new();
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .cloned()
        .unwrap_or_default()
}

fn primary_key_columns(state: &AppState) -> Vec<ColumnInfo> {
    table_columns(state)
        .into_iter()
        .filter(|col| col.is_primary_key)
        .collect()
}

fn build_data_edits(state: &AppState) -> Result<Vec<DataCellEdit>, String> {
    let source = state
        .active_data_source()
        .ok_or_else(|| t("grid_no_active_data_source"))?;
    let result = state
        .current_result
        .as_ref()
        .ok_or_else(|| t("grid_no_result_set"))?;
    let table_columns = table_columns(state);
    let pk_columns: Vec<ColumnInfo> = table_columns
        .iter()
        .filter(|col| col.is_primary_key)
        .cloned()
        .collect();
    if pk_columns.is_empty() {
        return Err(t("grid_pk_required"));
    }

    let mut edits = Vec::new();
    for ((row_idx, col_idx), cell) in &state.data_edit.cells {
        if !cell.is_dirty() {
            continue;
        }
        let column = result
            .columns
            .get(*col_idx)
            .ok_or_else(|| t("grid_column_missing"))?;
        let column_info = table_columns
            .iter()
            .find(|info| info.name == column.name)
            .cloned();
        if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
            continue;
        }
        let column_type = column_info
            .as_ref()
            .map(|info| info.data_type.clone())
            .unwrap_or_else(|| column.type_name.clone());
        let nullable = column_info
            .as_ref()
            .map(|info| info.is_nullable)
            .unwrap_or(true);
        let enum_values = column_info
            .as_ref()
            .map(|info| info.enum_values.as_slice())
            .unwrap_or(&[]);
        if let Some(error) = validate_edit_value(cell, &column_type, nullable, enum_values) {
            return Err(error);
        }

        let mut pk = Vec::new();
        for pk_col in &pk_columns {
            let pk_idx = result
                .columns
                .iter()
                .position(|col| col.name == pk_col.name)
                .ok_or_else(|| tf("grid_pk_missing", &[&pk_col.name]))?;
            let original = state
                .data_edit
                .cells
                .get(&(*row_idx, pk_idx))
                .map(|cell| cell.original.clone())
                .or_else(|| {
                    result
                        .rows
                        .get(*row_idx)
                        .and_then(|row| row.get(pk_idx))
                        .cloned()
                })
                .ok_or_else(|| t("grid_pk_value_missing"))?;
            pk.push(DataKeyValue {
                column: pk_col.name.clone(),
                column_type: pk_col.data_type.clone(),
                value: original,
            });
        }

        let value = if cell.is_null {
            DataEditValue::Null
        } else if is_timestamptz_type(&column_type) {
            DataEditValue::Text(
                timestamp_display_to_utc(&cell.value, &state.data_timezone)
                    .unwrap_or_else(|| cell.value.clone()),
            )
        } else {
            DataEditValue::Text(cell.value.clone())
        };

        edits.push(DataCellEdit {
            schema: source.schema.clone(),
            table: source.table.clone(),
            column: column.name.clone(),
            column_type,
            pk,
            value,
        });
    }

    Ok(edits)
}

fn count_invalid_edits(state: &AppState) -> usize {
    let Some(result) = state.current_result.as_ref() else {
        return 0;
    };
    state
        .data_edit
        .cells
        .iter()
        .filter(|((_, col_idx), cell)| {
            if !cell.is_dirty() {
                return false;
            }
            let Some(column) = result.columns.get(*col_idx) else {
                return true;
            };
            let info = data_column_info(state, &column.name);
            let type_name = info
                .map(|info| info.data_type.as_str())
                .unwrap_or(column.type_name.as_str());
            let nullable = info.map(|info| info.is_nullable).unwrap_or(true);
            let enum_values = info.map(|info| info.enum_values.as_slice()).unwrap_or(&[]);
            validate_edit_value(cell, type_name, nullable, enum_values).is_some()
        })
        .count()
}

fn revert_data_edits(state: &mut AppState) {
    let column_types = state
        .current_result
        .as_ref()
        .map(|result| {
            result
                .columns
                .iter()
                .map(|column| column.type_name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for ((_, col_idx), cell) in state.data_edit.cells.iter_mut() {
        let type_name = column_types
            .get(*col_idx)
            .map(String::as_str)
            .unwrap_or_default();
        cell.value = cell_edit_text_for_type(&cell.original, type_name, &state.data_timezone);
        cell.original_text = cell.value.clone();
        cell.is_null = matches!(cell.original, CellValue::Null);
    }
    state.data_edit.editing_cell = None;
}

fn validate_edit_value(
    cell: &crate::state::EditableCell,
    type_name: &str,
    nullable: bool,
    enum_values: &[String],
) -> Option<String> {
    if cell.is_null {
        return (!nullable).then(|| t("grid_not_null"));
    }

    if !enum_values.is_empty() && !enum_values.iter().any(|value| value == cell.value.trim()) {
        return Some(t("grid_enum_error"));
    }

    match edit_kind(type_name, &cell.original) {
        EditKind::Bool => parse_bool(&cell.value)
            .is_none()
            .then(|| t("grid_bool_error")),
        EditKind::Number => cell
            .value
            .trim()
            .parse::<f64>()
            .is_err()
            .then(|| t("grid_number_error")),
        EditKind::Json => serde_json::from_str::<serde_json::Value>(&cell.value)
            .is_err()
            .then(|| t("grid_json_error")),
        EditKind::Date => (!is_valid_date(&cell.value)).then(|| t("grid_date_error")),
        EditKind::DateTime => (!is_valid_datetime(&cell.value)).then(|| t("grid_datetime_error")),
        EditKind::Uuid => uuid::Uuid::parse_str(cell.value.trim())
            .is_err()
            .then(|| t("grid_uuid_error")),
        EditKind::Bytes => {
            let value = cell
                .value
                .trim()
                .strip_prefix("\\x")
                .unwrap_or(cell.value.trim());
            (!value.chars().all(|ch| ch.is_ascii_hexdigit()) || value.len() % 2 != 0)
                .then(|| t("grid_bytes_error"))
        }
        EditKind::Text => None,
    }
}

fn edit_kind(type_name: &str, cell: &CellValue) -> EditKind {
    let lower = type_name.to_ascii_lowercase();
    if matches!(cell, CellValue::Bool(_)) || matches!(lower.as_str(), "bool" | "boolean") {
        EditKind::Bool
    } else if matches!(cell, CellValue::Int(_) | CellValue::Float(_))
        || matches!(
            lower.as_str(),
            "smallint"
                | "integer"
                | "bigint"
                | "int2"
                | "int4"
                | "int8"
                | "real"
                | "double precision"
                | "float4"
                | "float8"
                | "numeric"
                | "decimal"
        )
    {
        EditKind::Number
    } else if matches!(cell, CellValue::Json(_)) || matches!(lower.as_str(), "json" | "jsonb") {
        EditKind::Json
    } else if lower == "date" {
        EditKind::Date
    } else if matches!(cell, CellValue::Timestamp(_))
        || matches!(
            lower.as_str(),
            "timestamp"
                | "timestamptz"
                | "timestamp without time zone"
                | "timestamp with time zone"
        )
    {
        EditKind::DateTime
    } else if matches!(cell, CellValue::Uuid(_)) || lower == "uuid" {
        EditKind::Uuid
    } else if matches!(cell, CellValue::Bytes(_)) || lower == "bytea" {
        EditKind::Bytes
    } else {
        EditKind::Text
    }
}

fn is_valid_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").is_ok()
}

fn is_valid_datetime(value: &str) -> bool {
    let value = value.trim();
    if chrono::DateTime::parse_from_rfc3339(value).is_ok() {
        return true;
    }

    [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
    ]
    .iter()
    .any(|format| chrono::NaiveDateTime::parse_from_str(value, format).is_ok())
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Export helpers
// ---------------------------------------------------------------------------

fn result_to_tsv(result: &crate::types::QueryResult) -> String {
    let mut out = String::new();
    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    out.push_str(&headers.join("\t"));
    out.push('\n');
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}

fn export_csv(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let task = rfd::AsyncFileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("query_result.csv")
        .save_file();

    let columns = result.columns.clone();
    let rows = result.rows.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let mut wtr = csv::Writer::from_writer(Vec::new());
                let headers: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
                let _ = wtr.write_record(&headers);
                for row in &rows {
                    let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                    let _ = wtr.write_record(&cells);
                }
                if let Ok(data) = wtr.into_inner() {
                    let _ = handle.write(&data).await;
                }
            }
        });
    });
}
