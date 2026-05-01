use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::state::AppState;
use crate::types::CellValue;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, BtnKind, Tokens};

// ---------------------------------------------------------------------------
// Public entry — called from result TopBottomPanel
// ---------------------------------------------------------------------------

pub fn render_result_panel(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    render_action_bar(ui, t, state);
    if let Some(ref err) = state.last_error.clone() {
        render_error_bar(ui, t, err);
    }
    match &state.current_result {
        None => render_empty_state(ui, t, state.query_running),
        Some(_) => {
            if state.result_view_form {
                render_form_view(ui, t, state);
            } else {
                render_grid_view(ui, t, state);
            }
        }
    }
}

/// Backward-compat: old `render_grid` signature used by tests / older callers.
pub fn render_grid(ui: &mut egui::Ui, state: &mut AppState) {
    let t = Tokens::current(ui.ctx());
    render_result_panel(ui, t, state);
}

// ---------------------------------------------------------------------------
// Action bar (32px) — Form/Grid toggle, filter, export, copy
// ---------------------------------------------------------------------------

fn render_action_bar(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(t.bg_app)
        .inner_margin(Margin::symmetric(theme::SPACE_MD_I, theme::SPACE_SM_I))
        .stroke(Stroke::new(1.0, t.border_subtle));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            // View toggle: Grid / Form
            if view_segment(ui, t, "Grid", Icon::Grid, !state.result_view_form) {
                state.result_view_form = false;
            }
            if view_segment(ui, t, "Form", Icon::Form, state.result_view_form) {
                state.result_view_form = true;
            }

            ui.add_space(theme::SPACE_LG);

            // Inline filter — vector filter icon + textbox
            icons::icon(ui, Icon::Filter, 12.0, t.text_muted);
            let filter_w = 200.0;
            let filter = egui::TextEdit::singleline(&mut state.result_filter)
                .hint_text("Filter rows\u{2026}")
                .desired_width(filter_w)
                .margin(egui::vec2(8.0, 4.0));
            ui.add(filter);

            // Result counts (mid)
            if let Some(ref result) = state.current_result {
                let total = result.rows.len();
                let shown = filtered_row_count(&state.result_filter, result);
                let row_label = if total == 1 { "row" } else { "rows" };
                let count_text = if state.result_filter.is_empty() {
                    format!("{} {}", total, row_label)
                } else {
                    format!("{}/{} {}", shown, total, row_label)
                };
                ui.label(
                    RichText::new(count_text)
                        .color(t.text_primary)
                        .size(12.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!(
                        "  \u{00B7}  {} cols  \u{00B7}  {}ms",
                        result.columns.len(),
                        result.execution_time_ms
                    ))
                    .color(t.text_muted)
                    .size(11.0),
                );
                if state.current_result_truncated {
                    ui.add_space(theme::SPACE_SM);
                    icons::icon(ui, Icon::Warning, 12.0, t.warn);
                    ui.label(
                        RichText::new("truncated").color(t.warn).size(11.0),
                    );
                }
            }

            // Right side: Copy, Export
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::icon_button_sm(ui, BtnKind::Secondary, Icon::Export, "CSV", t, true)
                    .clicked()
                {
                    export_csv(state);
                }
                ui.add_space(theme::SPACE_SM);
                if theme::icon_button_sm(ui, BtnKind::Secondary, Icon::Copy, "Copy TSV", t, true)
                    .clicked()
                {
                    if let Some(ref result) = state.current_result {
                        ui.ctx().copy_text(result_to_tsv(result));
                    }
                }
            });
        });
    });
}

fn view_segment(
    ui: &mut egui::Ui,
    t: Tokens,
    label: &str,
    icon: Icon,
    active: bool,
) -> bool {
    let icon_size = 12.0;
    let pad_x = 10.0;
    let gap = 6.0;
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );
    let total_w = pad_x + icon_size + gap + galley.rect.width() + pad_x;
    let h = 22.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(total_w, h), Sense::click());

    let bg = if active { t.accent } else { t.bg_elev };
    let fg = if active { t.text_inverse } else { t.text_secondary };
    let border = if active { t.accent_hot } else { t.border_default };
    ui.painter().rect_filled(rect, CornerRadius::same(theme::RADIUS_SM), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(1.0, border),
        egui::epaint::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + pad_x, rect.center().y - icon_size / 2.0),
        egui::vec2(icon_size, icon_size),
    );
    icons::icon_at(ui.painter(), icon, icon_rect, fg);
    ui.painter().text(
        egui::pos2(rect.left() + pad_x + icon_size + gap, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        fg,
    );

    resp.clicked()
}

// ---------------------------------------------------------------------------
// Error bar
// ---------------------------------------------------------------------------

fn render_error_bar(ui: &mut egui::Ui, t: Tokens, error: &str) {
    let frame = egui::Frame::new()
        .fill(Color32::from_rgba_premultiplied(t.danger.r(), t.danger.g(), t.danger.b(), 24))
        .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_SM_I))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_premultiplied(t.danger.r(), t.danger.g(), t.danger.b(), 80),
        ));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            icons::icon(ui, Icon::ErrorMark, 14.0, t.danger);
            ui.label(
                RichText::new("Error")
                    .color(t.danger)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new(error)
                    .color(t.danger)
                    .size(12.0),
            );
        });
    });
}

// ---------------------------------------------------------------------------
// Empty / loading
// ---------------------------------------------------------------------------

fn render_empty_state(ui: &mut egui::Ui, t: Tokens, running: bool) {
    ui.centered_and_justified(|ui| {
        if running {
            ui.vertical_centered(|ui| {
                ui.spinner();
                ui.add_space(theme::SPACE_MD);
                ui.label(
                    RichText::new("Executing query\u{2026}")
                        .color(t.text_muted)
                        .size(12.0),
                );
            });
        } else {
            ui.label(
                RichText::new("Run a query to see results")
                    .color(t.text_disabled)
                    .size(12.0),
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Grid view
// ---------------------------------------------------------------------------

fn filtered_row_indices(filter: &str, result: &crate::types::QueryResult) -> Vec<usize> {
    if filter.is_empty() {
        return (0..result.rows.len()).collect();
    }
    let needle = filter.to_ascii_lowercase();
    result
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            row.iter().any(|cell| cell.to_string().to_ascii_lowercase().contains(&needle))
        })
        .map(|(i, _)| i)
        .collect()
}

fn filtered_row_count(filter: &str, result: &crate::types::QueryResult) -> usize {
    if filter.is_empty() {
        return result.rows.len();
    }
    filtered_row_indices(filter, result).len()
}

fn render_grid_view(ui: &mut egui::Ui, t: Tokens, state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };
    if result.columns.is_empty() {
        return;
    }

    let visible: Vec<usize> = filtered_row_indices(&state.result_filter, result);

    let num_cols = result.columns.len();
    let available_width = ui.available_width();
    let col_width = (available_width / num_cols as f32).clamp(60.0, 320.0);
    let row_height = 22.0;
    let header_height = 28.0;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::initial(col_width).clip(true), num_cols)
        .header(header_height, |mut header| {
            for col in &result.columns {
                header.col(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, t.bg_elev);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, t.border_subtle),
                        egui::epaint::StrokeKind::Inside,
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(theme::SPACE_SM);
                        ui.label(
                            RichText::new(&col.name)
                                .color(t.text_primary)
                                .strong()
                                .size(12.0),
                        );
                        ui.label(
                            RichText::new(&col.type_name)
                                .color(t.text_muted)
                                .size(10.0)
                                .monospace(),
                        );
                    });
                });
            }
        })
        .body(|body| {
            body.rows(row_height, visible.len(), |mut row| {
                let row_data = &result.rows[visible[row.index()]];
                for cell in row_data {
                    row.col(|ui| {
                        ui.add_space(theme::SPACE_SM);
                        render_cell(ui, t, cell);
                    });
                }
            });
        });
}

fn render_cell(ui: &mut egui::Ui, t: Tokens, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            icons::icon(ui, Icon::NullMarker, 12.0, t.null);
        }
        CellValue::Bool(v) => {
            let (text, color) = if *v {
                ("true", t.success)
            } else {
                ("false", t.danger)
            };
            ui.label(RichText::new(text).color(color).size(12.0).monospace());
        }
        CellValue::Int(_) | CellValue::Float(_) => {
            ui.label(
                RichText::new(cell.to_string())
                    .font(egui::FontId::monospace(12.0))
                    .color(t.syntax_number),
            );
        }
        CellValue::Json(_) => {
            let s = cell.to_string();
            let resp = ui.label(
                RichText::new(&s)
                    .font(egui::FontId::monospace(12.0))
                    .color(t.info),
            );
            resp.on_hover_text(&s);
        }
        other => {
            let text = other.to_string();
            let resp = ui.label(
                RichText::new(&text)
                    .font(egui::FontId::monospace(12.0))
                    .color(t.text_primary),
            );
            resp.on_hover_text(&text).context_menu(|ui| {
                if ui.button("Copy Value").clicked() {
                    ui.ctx().copy_text(text.clone());
                    ui.close_menu();
                }
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Form view
// ---------------------------------------------------------------------------

fn render_form_view(ui: &mut egui::Ui, t: Tokens, state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };
    if result.columns.is_empty() {
        return;
    }

    let visible: Vec<usize> = filtered_row_indices(&state.result_filter, result);
    if visible.is_empty() {
        ui.add_space(theme::SPACE_XL);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("No matching rows")
                    .color(t.text_muted)
                    .size(12.0),
            );
        });
        return;
    }

    // Persistent selected row index per session
    let id = ui.make_persistent_id("form_selected_row");
    let mut idx = ui.data(|d| d.get_temp::<usize>(id)).unwrap_or(0);
    if idx >= visible.len() {
        idx = 0;
    }

    // Pager
    let pager_frame = egui::Frame::new()
        .fill(t.bg_app)
        .inner_margin(Margin::symmetric(theme::SPACE_MD_I, theme::SPACE_SM_I))
        .stroke(Stroke::new(1.0, t.border_subtle));

    pager_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            let prev = egui::Button::new(
                RichText::new("\u{25C0}").color(t.text_secondary).size(12.0),
            )
            .fill(t.bg_elev)
            .stroke(Stroke::new(1.0, t.border_default))
            .corner_radius(CornerRadius::same(theme::RADIUS_SM));
            if ui.add_enabled(idx > 0, prev).clicked() {
                idx -= 1;
            }
            let next = egui::Button::new(
                RichText::new("\u{25B6}").color(t.text_secondary).size(12.0),
            )
            .fill(t.bg_elev)
            .stroke(Stroke::new(1.0, t.border_default))
            .corner_radius(CornerRadius::same(theme::RADIUS_SM));
            if ui.add_enabled(idx + 1 < visible.len(), next).clicked() {
                idx += 1;
            }
            ui.label(
                RichText::new(format!(
                    "Row {} of {}",
                    idx + 1,
                    visible.len()
                ))
                .color(t.text_secondary)
                .size(11.0),
            );
        });
    });
    ui.data_mut(|d| d.insert_temp(id, idx));

    let row_idx = visible[idx];
    let row = &result.rows[row_idx];

    egui::ScrollArea::vertical()
        .id_salt("form_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("form_grid")
                .num_columns(2)
                .spacing([theme::SPACE_LG, theme::SPACE_SM])
                .min_col_width(140.0)
                .striped(true)
                .show(ui, |ui| {
                    for (col, cell) in result.columns.iter().zip(row.iter()) {
                        ui.label(
                            RichText::new(&col.name)
                                .color(t.text_secondary)
                                .strong()
                                .size(12.0),
                        );
                        render_cell(ui, t, cell);
                        ui.end_row();
                    }
                });
        });
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
                let headers: Vec<&str> =
                    columns.iter().map(|c| c.name.as_str()).collect();
                let _ = wtr.write_record(&headers);
                for row in &rows {
                    let cells: Vec<String> =
                        row.iter().map(|c| c.to_string()).collect();
                    let _ = wtr.write_record(&cells);
                }
                if let Ok(data) = wtr.into_inner() {
                    let _ = handle.write(&data).await;
                }
            }
        });
    });
}
