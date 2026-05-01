use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::AppState;
use crate::types::EditorTab;
use crate::ui::{icons, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_editor(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    render_tab_bar(ui, state);
    render_toolbar(ui, state, bridge);
    render_editor_body(ui, state);
}

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let tab_frame = egui::Frame::new()
        .fill(theme::BG_DARKEST)
        .inner_margin(Margin::symmetric(theme::SPACE_MD as i8, 0))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(30.0);

        ui.horizontal(|ui| {
            let mut tab_to_close: Option<usize> = None;

            for i in 0..state.editor_tabs.len() {
                let selected = i == state.active_tab;
                let title = state.editor_tabs[i].title.clone();
                render_tab(ui, i, &title, selected, &mut tab_to_close, &mut state.active_tab);
            }

            if let Some(idx) = tab_to_close {
                if state.editor_tabs.len() > 1 {
                    state.editor_tabs.remove(idx);
                    if state.active_tab >= state.editor_tabs.len() {
                        state.active_tab = state.editor_tabs.len() - 1;
                    }
                }
            }

            // New tab button
            ui.add_space(theme::SPACE_SM);
            let new_tab_btn = egui::Button::new(
                RichText::new(icons::PLUS)
                    .color(theme::TEXT_MUTED)
                    .size(14.0),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(24.0, 24.0));

            if ui.add(new_tab_btn).clicked() {
                let n = state.editor_tabs.len() + 1;
                state.editor_tabs.push(EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
            }
        });
    });
}

fn render_tab(
    ui: &mut egui::Ui,
    index: usize,
    title: &str,
    selected: bool,
    tab_to_close: &mut Option<usize>,
    active_tab: &mut usize,
) {
    let tab_height = 30.0;
    let tab_padding_x = theme::SPACE_LG;
    let close_btn_width = 18.0;

    let galley = ui.painter().layout_no_wrap(
        title.to_string(),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    let total_width =
        galley.rect.width() + tab_padding_x * 2.0 + close_btn_width + theme::SPACE_SM;

    let (tab_rect, resp) =
        ui.allocate_exact_size(egui::vec2(total_width, tab_height), egui::Sense::click());

    if resp.clicked() {
        *active_tab = index;
    }

    // Background
    let bg = if selected {
        theme::BG_DARK
    } else if resp.hovered() {
        theme::BG_MEDIUM
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(tab_rect, 0.0, bg);

    // Active tab bottom copper accent line
    if selected {
        let line = egui::Rect::from_min_size(
            egui::pos2(tab_rect.min.x, tab_rect.max.y - 2.0),
            egui::vec2(tab_rect.width(), 2.0),
        );
        ui.painter().rect_filled(line, 0.0, theme::ACCENT_COPPER);
    }

    // Right separator
    ui.painter().vline(
        tab_rect.max.x,
        tab_rect.y_range(),
        Stroke::new(1.0, theme::BORDER_SUBTLE),
    );

    // Tab label
    ui.painter().text(
        egui::pos2(tab_rect.min.x + tab_padding_x, tab_rect.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        if selected { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED },
    );

    // Close button ×
    let close_center = egui::pos2(
        tab_rect.max.x - close_btn_width / 2.0 - theme::SPACE_SM,
        tab_rect.center().y,
    );
    let close_rect =
        egui::Rect::from_center_size(close_center, egui::vec2(16.0, 16.0));
    let close_resp =
        ui.interact(close_rect, resp.id.with("close"), egui::Sense::click());

    let close_color = if close_resp.hovered() {
        theme::ACCENT_RED
    } else if selected {
        theme::TEXT_MUTED
    } else {
        Color32::TRANSPARENT
    };

    ui.painter().text(
        close_rect.center(),
        egui::Align2::CENTER_CENTER,
        icons::CLOSE,
        egui::FontId::proportional(11.0),
        close_color,
    );

    if close_resp.clicked() {
        *tab_to_close = Some(index);
    }

    resp.context_menu(|ui| {
        if ui.button("Close Tab").clicked() {
            *tab_to_close = Some(index);
            ui.close_menu();
        }
    });
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

fn render_toolbar(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let toolbar_frame = egui::Frame::new()
        .fill(theme::BG_DARK)
        .inner_margin(Margin::symmetric(theme::SPACE_MD as i8, theme::SPACE_SM as i8))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    toolbar_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            let has_connection = state.active_connection.is_some();
            let can_execute = has_connection && !state.query_running;

            let execute_label = format!("{} Execute", icons::EXECUTE);
            let execute_btn = if can_execute {
                theme::primary_button(&execute_label)
            } else {
                egui::Button::new(
                    RichText::new(&execute_label)
                        .color(theme::TEXT_DISABLED)
                        .size(12.0),
                )
                .fill(theme::BG_LIGHT)
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM))
            };

            let exec_resp = ui.add_enabled(can_execute, execute_btn);

            let shortcut_fired = can_execute
                && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));

            if exec_resp.clicked() || shortcut_fired {
                execute_current_query(state, bridge);
            }

            if exec_resp.hovered() {
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("exec_tip"),
                    |ui| {
                        ui.label(
                            RichText::new("Cmd+Return")
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );
                    },
                );
            }

            if state.query_running {
                ui.add_space(theme::SPACE_MD);
                ui.spinner();
                ui.label(
                    RichText::new("Running\u{2026}")
                        .color(theme::ACCENT_YELLOW)
                        .size(12.0),
                );

                let cancel_btn = egui::Button::new(
                    RichText::new(format!("{} Cancel", icons::CANCEL))
                        .color(theme::TEXT_SECONDARY)
                        .size(11.0),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM));

                if ui.add(cancel_btn).clicked() {
                    if let Some(conn_id) = state.active_connection {
                        bridge.send(DbCommand::CancelQuery { conn_id });
                    }
                }
            }

            // Right side: active connection name
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(8.0, 8.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            theme::ACCENT_GREEN,
                        );
                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(theme::TEXT_SECONDARY)
                                .size(11.0),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new("No connection \u{2014} open one from the sidebar")
                            .color(theme::TEXT_DISABLED)
                            .size(11.0),
                    );
                }
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Editor body
// ---------------------------------------------------------------------------

fn render_editor_body(ui: &mut egui::Ui, state: &mut AppState) {
    let editor_frame = egui::Frame::new()
        .fill(theme::BG_DARKEST)
        .inner_margin(Margin::same(theme::SPACE_MD as i8));

    editor_frame.show(ui, |ui| {
        if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let layout_job = highlight_sql(ui, text, wrap_width);
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::ScrollArea::vertical()
                .id_salt("editor_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut tab.content)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(12)
                            .frame(false)
                            .layouter(&mut layouter),
                    );
                });
        }
    });
}

// ---------------------------------------------------------------------------
// Query execution
// ---------------------------------------------------------------------------

fn execute_current_query(state: &mut AppState, bridge: &DbBridge) {
    if let Some(conn_id) = state.active_connection {
        if let Some(tab) = state.editor_tabs.get(state.active_tab) {
            let sql = tab.content.trim().to_string();
            if !sql.is_empty() {
                state.query_running = true;
                state.last_error = None;
                bridge.send(DbCommand::ExecuteQuery {
                    conn_id,
                    sql,
                    row_limit: Some(state.default_row_limit),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SQL syntax highlighting
// ---------------------------------------------------------------------------

const SQL_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT", "IN", "IS", "NULL",
    "INSERT", "INTO", "VALUES", "UPDATE", "SET", "DELETE", "CREATE",
    "ALTER", "DROP", "TABLE", "INDEX", "VIEW", "SCHEMA", "DATABASE",
    "JOIN", "LEFT", "RIGHT", "INNER", "OUTER", "FULL", "CROSS", "ON",
    "AS", "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET",
    "UNION", "ALL", "DISTINCT", "EXISTS", "BETWEEN", "LIKE", "ILIKE",
    "CASE", "WHEN", "THEN", "ELSE", "END", "WITH", "RECURSIVE",
    "TRUE", "FALSE", "ASC", "DESC", "NULLS", "FIRST", "LAST",
    "COUNT", "SUM", "AVG", "MIN", "MAX", "COALESCE", "CAST",
    "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "UNIQUE",
    "CHECK", "DEFAULT", "NOT", "EXPLAIN", "ANALYZE", "SHOW",
    "BEGIN", "COMMIT", "ROLLBACK", "GRANT", "REVOKE",
];

fn highlight_sql(ui: &egui::Ui, text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(13.0);
    let default_color = theme::TEXT_PRIMARY;

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Single-line comment
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            let start = i;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(font_id.clone(), theme::COMMENT_COLOR));
            continue;
        }

        // Block comment
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(font_id.clone(), theme::COMMENT_COLOR));
            continue;
        }

        // String literal
        if ch == '\'' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\'' {
                    i += 1;
                    if i < chars.len() && chars[i] == '\'' {
                        i += 1;
                        continue;
                    }
                    break;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(font_id.clone(), theme::STRING_COLOR));
            continue;
        }

        // Number
        if ch.is_ascii_digit()
            || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(font_id.clone(), theme::NUMBER_COLOR));
            continue;
        }

        // Identifier / keyword
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let upper = word.to_uppercase();
            let color = if SQL_KEYWORDS.contains(&upper.as_str()) {
                theme::KEYWORD_COLOR
            } else {
                default_color
            };
            job.append(&word, 0.0, fmt(font_id.clone(), color));
            continue;
        }

        job.append(&ch.to_string(), 0.0, fmt(font_id.clone(), default_color));
        i += 1;
    }

    job
}

#[inline]
fn fmt(font_id: egui::FontId, color: Color32) -> egui::text::TextFormat {
    egui::text::TextFormat::simple(font_id, color)
}
