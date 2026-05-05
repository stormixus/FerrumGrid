use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{AppState, ConnectionStatus};
use crate::storage::settings::AppSettings;
use crate::types::EditorTab;
use crate::ui::theme;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_editor(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &AppSettings,
) {
    render_tab_bar(ui, state);
    render_toolbar(ui, state, bridge);
    render_editor_body(ui, state, bridge, settings);
}

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let tab_frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(34.0);

        ui.horizontal(|ui| {
            let mut tab_to_close: Option<usize> = None;

            for i in 0..state.editor_tabs.len() {
                let selected = i == state.active_tab;
                let title = state.editor_tabs[i].title.clone();
                render_tab(
                    ui,
                    i,
                    &title,
                    selected,
                    &mut tab_to_close,
                    &mut state.active_tab,
                );
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
            let new_resp = render_query_add_tab_button(ui);

            if new_resp.clicked() {
                let n = state.editor_tabs.len() + 1;
                state.editor_tabs.push(EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
                state.open_workspace_main_view(crate::state::MainView::Query);
            }
        });
    });
}

fn render_query_add_tab_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let paint_rect = rect.shrink(1.0);
    let fill = if response.hovered() {
        theme::with_alpha(theme::ACCENT_TEAL, 34)
    } else {
        theme::with_alpha(theme::ACCENT_TEAL, 18)
    };
    let stroke = Stroke::new(
        1.0,
        if response.hovered() {
            theme::ACCENT_TEAL
        } else {
            theme::with_alpha(theme::ACCENT_TEAL, 95)
        },
    );
    ui.painter()
        .rect_filled(paint_rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        paint_rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );

    let center = paint_rect.center();
    let arm = 7.0;
    let plus_stroke = Stroke::new(1.65, theme::ACCENT_TEAL);
    ui.painter().line_segment(
        [
            egui::pos2(center.x - arm, center.y),
            egui::pos2(center.x + arm, center.y),
        ],
        plus_stroke,
    );
    ui.painter().line_segment(
        [
            egui::pos2(center.x, center.y - arm),
            egui::pos2(center.x, center.y + arm),
        ],
        plus_stroke,
    );

    response
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
    let display_title = truncate_label(title, 28);

    let galley = ui.painter().layout_no_wrap(
        display_title.clone(),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    let total_width =
        (galley.rect.width() + tab_padding_x * 2.0 + close_btn_width + theme::SPACE_SM)
            .clamp(94.0, 220.0);

    let (tab_rect, resp) =
        ui.allocate_exact_size(egui::vec2(total_width, tab_height), egui::Sense::click());

    if resp.clicked() {
        *active_tab = index;
    }

    // Background
    let bg = if selected {
        theme::bg_dark()
    } else if resp.hovered() {
        theme::with_alpha(theme::ACCENT_TEAL, 18)
    } else {
        Color32::TRANSPARENT
    };
    let paint_rect = tab_rect.shrink2(egui::vec2(1.0, 3.0));
    ui.painter()
        .rect_filled(paint_rect, CornerRadius::same(theme::RADIUS_LG), bg);

    // Active tab bottom copper accent line
    if selected {
        let line = egui::Rect::from_min_size(
            egui::pos2(paint_rect.min.x + 8.0, paint_rect.max.y - 2.0),
            egui::vec2((paint_rect.width() - 16.0).max(10.0), 2.0),
        );
        ui.painter().rect_filled(
            line,
            CornerRadius::same(theme::RADIUS_SM),
            theme::ACCENT_COPPER,
        );
    }

    // Tab label
    ui.painter().text(
        egui::pos2(tab_rect.min.x + tab_padding_x, tab_rect.center().y),
        egui::Align2::LEFT_CENTER,
        display_title,
        egui::FontId::proportional(12.0),
        if selected {
            theme::text_primary()
        } else {
            theme::text_muted()
        },
    );

    // Close button ×
    let close_center = egui::pos2(
        tab_rect.max.x - close_btn_width / 2.0 - theme::SPACE_SM,
        tab_rect.center().y,
    );
    let close_rect = egui::Rect::from_center_size(close_center, egui::vec2(16.0, 16.0));
    let close_resp = ui.interact(close_rect, resp.id.with("close"), egui::Sense::click());

    let close_color = if close_resp.hovered() {
        theme::ACCENT_RED
    } else if selected {
        theme::text_muted()
    } else {
        Color32::TRANSPARENT
    };

    ui.painter().text(
        close_rect.center(),
        egui::Align2::CENTER_CENTER,
        "\u{00d7}",
        egui::FontId::proportional(13.0),
        close_color,
    );

    if close_resp.clicked() {
        *tab_to_close = Some(index);
    }

    resp.context_menu(|ui| {
        if ui.button(t("workspace_close_tab")).clicked() {
            *tab_to_close = Some(index);
            ui.close_menu();
        }
    });
}

fn truncate_label(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let mut label = text
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect::<String>();
        label.push_str("...");
        label
    }
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

fn render_toolbar(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let toolbar_frame = egui::Frame::new()
        .fill(theme::bg_dark())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    toolbar_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            let has_connection = state.active_connection.is_some();
            let can_execute = has_connection && !state.query_running;
            let action_enabled = can_execute || state.query_running;
            let action_label = if state.query_running {
                t("query_cancel")
            } else {
                t("query_execute")
            };
            let exec_resp = editor_toolbar_action_button(
                ui,
                if state.query_running {
                    crate::ui::icons_svg::CANCEL
                } else {
                    crate::ui::icons_svg::EXECUTE
                },
                &action_label,
                can_execute.then_some("⌘↵"),
                action_enabled,
                can_execute,
                state.query_running,
            );

            let shortcut_fired =
                can_execute && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));

            if shortcut_fired || (exec_resp.clicked() && can_execute) {
                execute_current_query(state, bridge);
            }
            if exec_resp.clicked() && state.query_running {
                if let Some(conn_id) = state.active_connection {
                    bridge.send(DbCommand::CancelQuery { conn_id });
                }
            }

            if state.query_running {
                ui.add_space(theme::SPACE_MD);
                ui.spinner();
                ui.label(
                    RichText::new("Running...")
                        .color(theme::ACCENT_YELLOW)
                        .size(12.0),
                );
            }

            // Right side: active connection name
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        ui.allocate_new_ui(
                            egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
                                ui.next_widget_position() + egui::vec2(6.0, 11.0),
                                egui::vec2(12.0, 12.0),
                            )),
                            |ui| {
                                crate::ui::icon_img(
                                    ui,
                                    crate::ui::icons_svg::CONNECT,
                                    "editor_status_conn",
                                    10.0,
                                );
                            },
                        );
                        ui.add_space(14.0);

                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(theme::text_secondary())
                                .size(11.0),
                        );
                    }
                } else {
                    ui.label(
                        RichText::new("No connection")
                            .color(theme::text_disabled())
                            .size(11.0),
                    );
                }
            });
        });
    });
}

fn editor_toolbar_action_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    label: &str,
    shortcut: Option<&str>,
    enabled: bool,
    primary: bool,
    destructive: bool,
) -> egui::Response {
    let font = egui::FontId::proportional(12.5);
    let text_color = if enabled {
        theme::text_primary()
    } else {
        theme::text_disabled()
    };
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let shortcut_font = egui::FontId::monospace(10.0);
    let shortcut_width = shortcut
        .map(|shortcut| {
            ui.painter()
                .layout_no_wrap(
                    shortcut.to_string(),
                    shortcut_font.clone(),
                    theme::text_muted(),
                )
                .rect
                .width()
                + 14.0
        })
        .unwrap_or(0.0);
    let shortcut_gap = if shortcut.is_some() { 10.0 } else { 0.0 };
    let width = (text_width + shortcut_width + shortcut_gap + 42.0).max(86.0);
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), sense);
    let hovered = enabled && response.hovered();
    let accent = if destructive {
        theme::ACCENT_RED
    } else {
        theme::ACCENT_TEAL
    };
    let fill = if !enabled {
        theme::bg_darkest()
    } else if primary {
        theme::with_alpha(accent, if hovered { 44 } else { 30 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if enabled && (hovered || primary || destructive) {
        Stroke::new(1.0, accent)
    } else if !enabled {
        Stroke::new(1.0, theme::border_subtle())
    } else {
        Stroke::new(1.0, theme::border_default())
    };

    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_LG), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        stroke,
        egui::StrokeKind::Inside,
    );

    let icon_color = if enabled {
        accent
    } else {
        theme::text_disabled()
    };
    let icon_name = if destructive {
        "query_cancel_main"
    } else {
        "query_execute_main"
    };
    let icon_rect = egui::Rect::from_center_size(
        rect.left_center() + egui::vec2(16.0, 0.0),
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
                ui, icon_svg, icon_name, 12.0, icon_color,
            ));
        },
    );
    let label_pos = rect.left_center() + egui::vec2(31.0, 0.0);
    let label_clip = egui::Rect::from_min_max(
        egui::pos2(label_pos.x, rect.top()),
        egui::pos2(
            rect.right() - shortcut_width - shortcut_gap - 8.0,
            rect.bottom(),
        ),
    );
    ui.painter().with_clip_rect(label_clip).text(
        label_pos,
        egui::Align2::LEFT_CENTER,
        label,
        font,
        text_color,
    );

    if let Some(shortcut) = shortcut {
        let key_rect = egui::Rect::from_center_size(
            rect.right_center() - egui::vec2(shortcut_width / 2.0 + 7.0, 0.0),
            egui::vec2(shortcut_width, 18.0),
        );
        ui.painter().rect_filled(
            key_rect,
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::bg_light(), if enabled { 190 } else { 110 }),
        );
        ui.painter().rect_stroke(
            key_rect,
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::border_subtle()),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            key_rect.center(),
            egui::Align2::CENTER_CENTER,
            shortcut,
            shortcut_font,
            if enabled {
                theme::text_muted()
            } else {
                theme::text_disabled()
            },
        );
    }

    response
}

// ---------------------------------------------------------------------------
// Editor body
// ---------------------------------------------------------------------------

fn render_editor_body(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &AppSettings,
) {
    if settings.enable_code_completion {
        ensure_completion_metadata(state, bridge);
    }

    let editor_frame = egui::Frame::new()
        .fill(theme::bg_editor())
        .inner_margin(Margin::ZERO);

    editor_frame.show(ui, |ui| {
        let active_tab = state.active_tab;
        if active_tab >= state.editor_tabs.len() {
            return;
        }

        let (title, line_count, char_count) = {
            let tab = &state.editor_tabs[active_tab];
            let line_count = if tab.content.is_empty() {
                1
            } else {
                tab.content.lines().count()
            };
            let char_count = tab.content.chars().count();
            (tab.title.clone(), line_count, char_count)
        };
        render_editor_meta(ui, &title, line_count, char_count);

        let mut editor_rect = egui::Rect::NOTHING;
        let mut cursor_index = None;
        let mut content_snapshot = String::new();
        let tab_id = state.editor_tabs[active_tab].id;

        // Intercept completion keys BEFORE TextEdit consumes them
        let popup_sel_id = egui::Id::new(("sql_completion_sel", tab_id));
        let popup_active = ui
            .data_mut(|d| d.get_persisted::<usize>(popup_sel_id))
            .is_some_and(|v| v != usize::MAX);
        let mut comp_accept = false;
        let mut comp_up = false;
        let mut comp_down = false;
        if popup_active && settings.enable_code_completion && settings.code_completion_popup {
            comp_accept = ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                    || i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
            });
            comp_up = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
            comp_down =
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        }

        {
            let tab = &mut state.editor_tabs[active_tab];
            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let layout_job = highlight_sql(text, wrap_width);
                ui.fonts(|f| f.layout_job(layout_job))
            };

            egui::Frame::new()
                .fill(theme::bg_editor())
                .inner_margin(Margin::same(theme::SPACE_LG as i8))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(("editor_scroll", tab.id))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let output = egui::TextEdit::multiline(&mut tab.content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(12)
                                .hint_text("SELECT *\nFROM public.table_name\nLIMIT 100;")
                                .frame(false)
                                .layouter(&mut layouter)
                                .show(ui);
                            editor_rect = output.response.rect;
                            cursor_index =
                                output.cursor_range.map(|range| range.primary.ccursor.index);
                            content_snapshot = tab.content.clone();
                        });
                });
        }

        if settings.enable_code_completion && settings.code_completion_popup {
            if let Some(insert) = render_completion_popup(
                ui,
                state,
                tab_id,
                editor_rect,
                &content_snapshot,
                cursor_index,
                comp_accept,
                comp_up,
                comp_down,
            ) {
                if let Some(tab) = state.editor_tabs.get_mut(active_tab) {
                    apply_completion(&mut tab.content, &insert);
                }
            }
        }
    });
}

fn render_editor_meta(ui: &mut egui::Ui, title: &str, line_count: usize, char_count: usize) {
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .color(theme::text_secondary())
                        .strong()
                        .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{line_count} lines  |  {char_count} chars"))
                            .color(theme::text_muted())
                            .monospace()
                            .size(10.0),
                    );
                });
            });
        });
}

// ---------------------------------------------------------------------------
// SQL completion
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct CompletionItem {
    label: String,
    detail: &'static str,
    insert_text: String,
    color: Color32,
}

struct CompletionContext {
    token: String,
    fragment: String,
    start_char: usize,
    end_char: usize,
}

struct CompletionInsert {
    start_char: usize,
    end_char: usize,
    text: String,
}

fn ensure_completion_metadata(state: &mut AppState, bridge: &DbBridge) {
    let Some(conn_id) = state.active_connection else {
        return;
    };
    let Some(conn) = state.connections.get(&conn_id) else {
        return;
    };
    if !matches!(conn.status, ConnectionStatus::Connected { .. }) {
        return;
    }

    let missing_schemas: Vec<String> = conn
        .schemas
        .iter()
        .filter(|schema| {
            !conn.tables.contains_key(*schema) && !conn.loading_tables.contains(*schema)
        })
        .cloned()
        .collect();

    if missing_schemas.is_empty() {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for schema in &missing_schemas {
            conn.loading_tables.insert(schema.clone());
        }
    }

    for schema in missing_schemas {
        bridge.send(DbCommand::ListTables { conn_id, schema });
    }
}

fn render_completion_popup(
    ui: &mut egui::Ui,
    state: &AppState,
    tab_id: uuid::Uuid,
    editor_rect: egui::Rect,
    content: &str,
    cursor_index: Option<usize>,
    accept: bool,
    nav_up: bool,
    nav_down: bool,
) -> Option<CompletionInsert> {
    if editor_rect == egui::Rect::NOTHING {
        return None;
    }

    let cursor = cursor_index.unwrap_or_else(|| content.chars().count());
    let context = completion_context(content, cursor)?;
    let suggestions = collect_completions(state, &context);
    let popup_id = egui::Id::new(("sql_completion_sel", tab_id));

    if suggestions.is_empty() {
        ui.data_mut(|d| d.insert_persisted(popup_id, usize::MAX));
        return None;
    }

    let mut selected: usize = ui.data_mut(|d| d.get_persisted(popup_id).unwrap_or(0));

    if nav_up {
        selected = selected.saturating_sub(1);
    }
    if nav_down {
        selected = (selected + 1).min(suggestions.len().saturating_sub(1));
    }
    selected = selected.min(suggestions.len().saturating_sub(1));
    ui.data_mut(|d| d.insert_persisted(popup_id, selected));

    if accept {
        ui.data_mut(|d| d.insert_persisted(popup_id, usize::MAX));
        return Some(CompletionInsert {
            start_char: context.start_char,
            end_char: context.end_char,
            text: suggestions[selected].insert_text.clone(),
        });
    }

    let mut picked = None;
    let pos = editor_rect.left_top() + egui::vec2(18.0, 28.0);
    egui::Area::new(egui::Id::new(("sql_completion_popup", tab_id)))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                .inner_margin(Margin::same(theme::SPACE_SM as i8))
                .show(ui, |ui| {
                    ui.set_min_width(430.0);
                    egui::ScrollArea::vertical()
                        .max_height(270.0)
                        .show(ui, |ui| {
                            for (idx, item) in suggestions.iter().enumerate() {
                                if render_completion_item(ui, item, idx == selected).clicked() {
                                    picked = Some(CompletionInsert {
                                        start_char: context.start_char,
                                        end_char: context.end_char,
                                        text: item.insert_text.clone(),
                                    });
                                }
                            }
                        });
                });
        });

    picked
}

fn render_completion_item(ui: &mut egui::Ui, item: &CompletionItem, selected: bool) -> egui::Response {
    let width = ui.available_width().max(360.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 45)
    } else if response.hovered() {
        theme::with_alpha(theme::ACCENT_TEAL, 28)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(
        rect.shrink2(egui::vec2(1.0, 1.0)),
        CornerRadius::same(theme::RADIUS_SM),
        fill,
    );

    let kind_rect = egui::Rect::from_min_size(
        rect.left_center() + egui::vec2(8.0, -8.0),
        egui::vec2(72.0, 16.0),
    );
    ui.painter().rect_filled(
        kind_rect,
        CornerRadius::same(theme::RADIUS_SM),
        theme::with_alpha(item.color, 30),
    );
    ui.painter().text(
        kind_rect.center(),
        egui::Align2::CENTER_CENTER,
        item.detail,
        egui::FontId::monospace(9.5),
        item.color,
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(92.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &item.label,
        egui::FontId::monospace(12.0),
        theme::text_primary(),
    );

    response
}

fn completion_context(content: &str, cursor: usize) -> Option<CompletionContext> {
    let chars: Vec<char> = content.chars().collect();
    let end = cursor.min(chars.len());
    let mut start = end;
    while start > 0 && is_completion_char(chars[start - 1]) {
        start -= 1;
    }

    let token: String = chars[start..end].iter().collect();
    if token.is_empty() {
        return None;
    }

    let fragment = token
        .rsplit_once('.')
        .map(|(_, fragment)| fragment)
        .unwrap_or(&token)
        .trim_matches('"')
        .to_lowercase();

    if fragment.is_empty() && !token.ends_with('.') {
        return None;
    }

    Some(CompletionContext {
        token,
        fragment,
        start_char: start,
        end_char: end,
    })
}

fn is_completion_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '"')
}

fn collect_completions(state: &AppState, context: &CompletionContext) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let qualifier = context
        .token
        .rsplit_once('.')
        .map(|(qualifier, _)| qualifier.trim_matches('"').to_lowercase());
    let has_qualifier = qualifier.is_some();

    if !has_qualifier {
        for keyword in SQL_KEYWORDS {
            if completion_matches(keyword, &context.fragment) {
                items.push(CompletionItem {
                    label: (*keyword).to_string(),
                    detail: "COMMAND",
                    insert_text: format!("{keyword} "),
                    color: theme::KEYWORD_COLOR,
                });
            }
        }
    }

    let Some(conn_id) = state.active_connection else {
        return trim_completion_items(items);
    };
    let Some(conn) = state.connections.get(&conn_id) else {
        return trim_completion_items(items);
    };

    if !has_qualifier {
        for database in &conn.databases {
            push_completion(
                &mut items,
                database,
                "DATABASE",
                sql_ident(database),
                theme::ACCENT_BLUE,
                &context.fragment,
            );
        }

        for schema in &conn.schemas {
            push_completion(
                &mut items,
                schema,
                "SCHEMA",
                sql_ident(schema),
                theme::ACCENT_TEAL,
                &context.fragment,
            );
        }
    }

    for (schema, tables) in &conn.tables {
        for table in tables {
            let table_name = table.name.as_str();
            let qualified_label = format!("{schema}.{table_name}");
            let qualified_insert = format!("{}.{}", sql_ident(schema), sql_ident(table_name));
            let table_matches_qualifier = qualifier
                .as_deref()
                .map(|qualifier| qualifier == schema.to_lowercase())
                .unwrap_or(true);

            if table_matches_qualifier
                && (completion_matches(table_name, &context.fragment)
                    || completion_matches(&qualified_label, &context.fragment))
            {
                items.push(CompletionItem {
                    label: qualified_label.clone(),
                    detail: table.table_type_label(),
                    insert_text: qualified_insert.clone(),
                    color: table_type_completion_color(&table.table_type),
                });
            }

            for ((column_schema, column_table), columns) in &conn.columns {
                if column_schema != schema || column_table != table_name {
                    continue;
                }

                let table_qualifier = table_name.to_lowercase();
                let schema_table_qualifier =
                    format!("{}.{}", schema.to_lowercase(), table_name.to_lowercase());
                let column_matches_qualifier = qualifier
                    .as_deref()
                    .map(|qualifier| {
                        qualifier == table_qualifier || qualifier == schema_table_qualifier
                    })
                    .unwrap_or(false);

                for column in columns {
                    if !completion_matches(&column.name, &context.fragment) {
                        continue;
                    }
                    let insert_text = if column_matches_qualifier {
                        format!("{}.{}", context.token_prefix(), sql_ident(&column.name))
                    } else {
                        sql_ident(&column.name)
                    };
                    items.push(CompletionItem {
                        label: format!("{schema}.{table_name}.{}", column.name),
                        detail: "COLUMN",
                        insert_text,
                        color: theme::ACCENT_COPPER_LIGHT,
                    });
                }
            }
        }
    }

    for (schema, functions) in &conn.functions {
        for function in functions {
            if completion_matches(&function.name, &context.fragment) {
                items.push(CompletionItem {
                    label: format!("{schema}.{}({})", function.name, function.arguments),
                    detail: "FUNCTION",
                    insert_text: format!("{}.{}()", sql_ident(schema), sql_ident(&function.name)),
                    color: theme::ACCENT_YELLOW,
                });
            }
        }
    }

    trim_completion_items(items)
}

trait TableTypeLabel {
    fn table_type_label(&self) -> &'static str;
}

impl TableTypeLabel for crate::types::TableInfo {
    fn table_type_label(&self) -> &'static str {
        match self.table_type.as_str() {
            "VIEW" => "VIEW",
            "MATERIALIZED VIEW" => "MAT VIEW",
            _ => "TABLE",
        }
    }
}

trait CompletionContextExt {
    fn token_prefix(&self) -> String;
}

impl CompletionContextExt for CompletionContext {
    fn token_prefix(&self) -> String {
        self.token
            .rsplit_once('.')
            .map(|(prefix, _)| prefix.to_string())
            .unwrap_or_default()
    }
}

fn push_completion(
    items: &mut Vec<CompletionItem>,
    label: &str,
    detail: &'static str,
    insert_text: String,
    color: Color32,
    fragment: &str,
) {
    if completion_matches(label, fragment) {
        items.push(CompletionItem {
            label: label.to_string(),
            detail,
            insert_text,
            color,
        });
    }
}

fn completion_matches(candidate: &str, fragment: &str) -> bool {
    fragment.is_empty() || candidate.to_lowercase().contains(fragment)
}

fn trim_completion_items(mut items: Vec<CompletionItem>) -> Vec<CompletionItem> {
    items.sort_by(|a, b| {
        a.detail
            .cmp(b.detail)
            .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
    });
    items.dedup_by(|a, b| a.detail == b.detail && a.label.eq_ignore_ascii_case(&b.label));
    items.truncate(14);
    items
}

fn table_type_completion_color(table_type: &str) -> Color32 {
    match table_type {
        "VIEW" => theme::ACCENT_BLUE,
        "MATERIALIZED VIEW" => theme::ACCENT_TEAL,
        _ => theme::ACCENT_COPPER,
    }
}

fn sql_ident(identifier: &str) -> String {
    let safe = identifier.chars().enumerate().all(|(idx, ch)| {
        if idx == 0 {
            ch.is_ascii_lowercase() || ch == '_'
        } else {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'
        }
    }) && !SQL_KEYWORDS.contains(&identifier.to_uppercase().as_str());

    if safe {
        identifier.to_string()
    } else {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }
}

fn apply_completion(content: &mut String, insert: &CompletionInsert) {
    let start = char_to_byte_index(content, insert.start_char);
    let end = char_to_byte_index(content, insert.end_char);
    content.replace_range(start..end, &insert.text);
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| text.len())
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
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "NOT",
    "IN",
    "IS",
    "NULL",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "ALTER",
    "DROP",
    "TABLE",
    "INDEX",
    "VIEW",
    "SCHEMA",
    "DATABASE",
    "JOIN",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "FULL",
    "CROSS",
    "ON",
    "AS",
    "ORDER",
    "BY",
    "GROUP",
    "HAVING",
    "LIMIT",
    "OFFSET",
    "UNION",
    "ALL",
    "DISTINCT",
    "EXISTS",
    "BETWEEN",
    "LIKE",
    "ILIKE",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "WITH",
    "RECURSIVE",
    "TRUE",
    "FALSE",
    "ASC",
    "DESC",
    "NULLS",
    "FIRST",
    "LAST",
    "COUNT",
    "SUM",
    "AVG",
    "MIN",
    "MAX",
    "COALESCE",
    "CAST",
    "PRIMARY",
    "KEY",
    "FOREIGN",
    "REFERENCES",
    "CONSTRAINT",
    "UNIQUE",
    "CHECK",
    "DEFAULT",
    "NOT",
    "EXPLAIN",
    "ANALYZE",
    "SHOW",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "GRANT",
    "REVOKE",
];

fn highlight_sql(text: &str, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(13.0);
    let default_color = theme::text_primary();

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
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
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
