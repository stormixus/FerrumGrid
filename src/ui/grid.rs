use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};
use egui_extras::{Column, TableBuilder};

use crate::state::AppState;
use crate::types::CellValue;
use crate::ui::{icons, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_grid(ui: &mut egui::Ui, state: &mut AppState) {
    if let Some(ref error) = state.last_error.clone() {
        render_error_bar(ui, error);
    }

    match &state.current_result {
        None => render_empty_state(ui, state.query_running),
        Some(_) => {
            render_result_header(ui, state);
            render_table(ui, state);
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
            ui.label(
                RichText::new(format!("{} Error", icons::ERROR))
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
                        .color(theme::TEXT_MUTED)
                        .size(12.0),
                );
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(icons::TABLE)
                        .color(theme::with_alpha(theme::ACCENT_TEAL, 150))
                        .size(34.0),
                );
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new("No result set")
                        .color(theme::TEXT_MUTED)
                        .strong()
                        .size(13.0),
                );
                ui.label(
                    RichText::new("Run a query to populate the grid")
                        .color(theme::TEXT_DISABLED)
                        .size(11.0),
                );
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Result info header strip
// ---------------------------------------------------------------------------

fn render_result_header(ui: &mut egui::Ui, state: &mut AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let row_count = result.rows.len();
    let col_count = result.columns.len();
    let exec_ms = result.execution_time_ms;
    let truncated = state.current_result_truncated;

    let frame = egui::Frame::new()
        .fill(theme::BG_SHELL)
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Result")
                    .color(theme::TEXT_PRIMARY)
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
                metric_chip(
                    ui,
                    &format!("{} truncated", icons::TRUNCATED),
                    theme::ACCENT_YELLOW,
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(theme::ghost_button(&format!("{} CSV", icons::EXPORT)))
                    .clicked()
                {
                    export_csv(state);
                }

                ui.add_space(theme::SPACE_SM);

                if ui
                    .add(theme::ghost_button(&format!("{} Copy TSV", icons::COPY)))
                    .clicked()
                {
                    if let Some(ref result) = state.current_result {
                        let tsv = result_to_tsv(result);
                        ui.ctx().copy_text(tsv);
                    }
                }
            });
        });
    });
}

fn metric_chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(11.0),
        theme::TEXT_PRIMARY,
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
        theme::TEXT_SECONDARY,
    );
}

// ---------------------------------------------------------------------------
// Result table
// ---------------------------------------------------------------------------

fn render_table(ui: &mut egui::Ui, state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    if result.columns.is_empty() {
        return;
    }

    let num_cols = result.columns.len();
    let available_width = ui.available_width();
    let col_width = (available_width / num_cols as f32).clamp(60.0, 320.0);
    let row_height = 24.0;
    let header_height = 30.0;
    let header_bg = theme::BG_MEDIUM;

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .columns(Column::initial(col_width).clip(true), num_cols)
        .header(header_height, |mut header| {
            for col in &result.columns {
                header.col(|ui| {
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, header_bg);
                    ui.horizontal(|ui| {
                        ui.add_space(theme::SPACE_SM);
                        ui.label(
                            RichText::new(&col.name)
                                .color(theme::TEXT_PRIMARY)
                                .strong()
                                .size(12.0),
                        );
                        ui.label(
                            RichText::new(format!(" {}", col.type_name))
                                .color(theme::TEXT_MUTED)
                                .size(10.0)
                                .monospace(),
                        );
                    });
                });
            }
        })
        .body(|body| {
            body.rows(row_height, result.rows.len(), |mut row| {
                let row_data = &result.rows[row.index()];
                for cell in row_data {
                    row.col(|ui| {
                        ui.add_space(theme::SPACE_SM);
                        render_cell(ui, cell);
                    });
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Cell rendering
// ---------------------------------------------------------------------------

fn render_cell(ui: &mut egui::Ui, cell: &CellValue) {
    match cell {
        CellValue::Null => {
            value_pill(ui, icons::NULL_MARKER, theme::TEXT_MUTED);
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
            render_copyable_cell(ui, &format!("\\x{}", hex_encode(v)), theme::TEXT_MUTED);
        }
        other => {
            let text = other.to_string();
            render_copyable_cell(ui, &text, theme::TEXT_PRIMARY);
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
        if ui.button(format!("{} Copy Value", icons::COPY)).clicked() {
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
        if ui.button(format!("{} Copy Value", icons::COPY)).clicked() {
            ui.ctx().copy_text(text.to_string());
            ui.close_menu();
        }
    });
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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
