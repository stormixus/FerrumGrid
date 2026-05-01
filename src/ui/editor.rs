use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::AppState;
use crate::types::EditorTab;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, BtnKind, Tokens};
use crate::ui::icons;

// ---------------------------------------------------------------------------
// Public entry called from CentralPanel
// ---------------------------------------------------------------------------

pub fn render_workspace(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
) {
    render_tab_bar(ui, t, state);
    render_toolbar(ui, t, state, bridge);
    render_body(ui, t, state);
}

/// Backward-compat entry used by older callers / tests.
pub fn render_editor(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let t = Tokens::current(ui.ctx());
    render_workspace(ui, t, state, bridge);
}

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

fn render_tab_bar(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(t.bg_app)
        .inner_margin(Margin::symmetric(theme::SPACE_MD_I, 0))
        .stroke(Stroke::new(1.0, t.border_subtle));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(32.0);

        ui.horizontal(|ui| {
            let mut tab_to_close: Option<usize> = None;

            for i in 0..state.editor_tabs.len() {
                let selected = i == state.active_tab;
                let title = state.editor_tabs[i].title.clone();
                render_tab(ui, t, i, &title, selected, &mut tab_to_close, &mut state.active_tab);
            }

            if let Some(idx) = tab_to_close {
                if state.editor_tabs.len() > 1 {
                    state.editor_tabs.remove(idx);
                    if state.active_tab >= state.editor_tabs.len() {
                        state.active_tab = state.editor_tabs.len() - 1;
                    }
                }
            }

            ui.add_space(theme::SPACE_SM);
            if theme::icon_only_button(ui, Icon::Plus, t, t.text_muted, 14.0).clicked() {
                let n = state.editor_tabs.len() + 1;
                state.editor_tabs.push(EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
            }
        });
    });
}

fn render_tab(
    ui: &mut egui::Ui,
    t: Tokens,
    index: usize,
    title: &str,
    selected: bool,
    tab_to_close: &mut Option<usize>,
    active_tab: &mut usize,
) {
    let tab_height = 32.0;
    let pad_x = theme::SPACE_LG;
    let close_w = 18.0;

    // Monogram chip + title width
    let chip_w = 14.0;
    let title_galley = ui.painter().layout_no_wrap(
        title.to_string(),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    let total_w = pad_x + chip_w + 8.0 + title_galley.rect.width() + 12.0 + close_w + pad_x;

    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(total_w, tab_height), Sense::click());

    if resp.clicked() {
        *active_tab = index;
    }

    let bg = if selected {
        t.bg_surface
    } else if resp.hovered() {
        t.bg_elev
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 0.0, bg);

    if selected {
        let line = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.max.y - 2.0),
            egui::vec2(rect.width(), 2.0),
        );
        ui.painter().rect_filled(line, 0.0, t.accent);
    }

    ui.painter().vline(
        rect.max.x,
        rect.y_range(),
        Stroke::new(1.0, t.border_subtle),
    );

    // Chip
    let chip_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + pad_x, rect.center().y - chip_w / 2.0),
        egui::vec2(chip_w, chip_w),
    );
    ui.painter().rect_filled(
        chip_rect,
        CornerRadius::same(theme::RADIUS_SM),
        crate::ui::theme::monogram_bg(t.chip_query),
    );
    ui.painter().text(
        chip_rect.center(),
        egui::Align2::CENTER_CENTER,
        icons::MONO_QUERY,
        egui::FontId::proportional(9.0),
        t.chip_query,
    );

    ui.painter().text(
        egui::pos2(rect.min.x + pad_x + chip_w + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        if selected { t.text_primary } else { t.text_secondary },
    );

    let close_center = egui::pos2(
        rect.max.x - close_w / 2.0 - theme::SPACE_SM,
        rect.center().y,
    );
    let close_rect = egui::Rect::from_center_size(close_center, egui::vec2(16.0, 16.0));
    let close_resp =
        ui.interact(close_rect, resp.id.with("close"), Sense::click());

    let close_color = if close_resp.hovered() {
        t.danger
    } else if selected {
        t.text_muted
    } else {
        Color32::TRANSPARENT
    };
    icons::icon_at(ui.painter(), Icon::Close, close_rect, close_color);

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
// Editor toolbar (36px)
// ---------------------------------------------------------------------------

fn render_toolbar(ui: &mut egui::Ui, t: Tokens, state: &mut AppState, bridge: &DbBridge) {
    let frame = egui::Frame::new()
        .fill(t.bg_surface)
        .inner_margin(Margin::symmetric(theme::SPACE_MD_I, theme::SPACE_SM_I))
        .stroke(Stroke::new(1.0, t.border_subtle));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            let has_conn = state.active_connection.is_some();
            let can_exec = has_conn && !state.query_running;

            let run_resp = theme::icon_button(ui, BtnKind::Primary, Icon::Play, "Run", t, can_exec);
            let shortcut_fired = can_exec
                && ui.input(|i| {
                    i.modifiers.command && i.key_pressed(egui::Key::Enter)
                });
            if run_resp.clicked() || shortcut_fired {
                execute_current_query(state, bridge);
            }

            if run_resp.hovered() {
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    ui.layer_id(),
                    egui::Id::new("exec_tip"),
                    |ui| {
                        ui.label(
                            RichText::new("\u{2318}\u{21A9}  Execute")
                                .color(t.text_muted)
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
                        .color(t.warn)
                        .size(12.0),
                );
                if theme::icon_button(ui, BtnKind::Secondary, Icon::Stop, "Stop", t, true).clicked() {
                    if let Some(conn_id) = state.active_connection {
                        bridge.send(DbCommand::CancelQuery { conn_id });
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 3.5, t.success);
                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(t.text_secondary)
                                .size(11.0),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new("No connection \u{2014} pick one in the sidebar")
                            .color(t.text_disabled)
                            .size(11.0),
                    );
                }
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Editor body (SQL multiline)
// ---------------------------------------------------------------------------

fn render_body(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(t.bg_surface)
        .inner_margin(Margin::same(theme::SPACE_MD_I));

    frame.show(ui, |ui| {
        if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
            let tokens = t;
            let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let job = highlight_sql(tokens, text, wrap_width);
                ui.fonts(|f| f.layout_job(job))
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
// SQL syntax highlighting (theme-aware)
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
    "CHECK", "DEFAULT", "EXPLAIN", "ANALYZE", "SHOW",
    "BEGIN", "COMMIT", "ROLLBACK", "GRANT", "REVOKE",
];

fn highlight_sql(t: Tokens, text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(13.0);
    let default_color = t.text_primary;

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
            job.append(&s, 0.0, fmt(font_id.clone(), t.syntax_comment, true));
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
            job.append(&s, 0.0, fmt(font_id.clone(), t.syntax_comment, true));
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
            job.append(&s, 0.0, fmt(font_id.clone(), t.syntax_string, false));
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
            job.append(&s, 0.0, fmt(font_id.clone(), t.syntax_number, false));
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
                t.syntax_keyword
            } else {
                default_color
            };
            job.append(&word, 0.0, fmt(font_id.clone(), color, false));
            continue;
        }

        job.append(&ch.to_string(), 0.0, fmt(font_id.clone(), default_color, false));
        i += 1;
    }

    job
}

#[inline]
fn fmt(font_id: egui::FontId, color: Color32, italic: bool) -> egui::text::TextFormat {
    let mut f = egui::text::TextFormat::simple(font_id, color);
    f.italics = italic;
    f
}
