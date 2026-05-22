use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{AppState, ConnectionStatus};
use crate::storage::settings::AppSettings;
use crate::types::EditorTab;
use crate::ui::{icons_svg, theme};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn render_editor(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &AppSettings,
) {
    render_tab_bar(ui, state, bridge);
    render_toolbar(ui, state, bridge);
    render_editor_body(ui, state, bridge, settings);
}

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
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
                    // US-J2 — explicit tx active 시 tab close 전 ROLLBACK 발사.
                    if state.explicit_tx_active {
                        if let Some(conn_id) = state.active_connection {
                            bridge.send(DbCommand::ExecuteQuery {
                                conn_id,
                                sql: "ROLLBACK".to_string(),
                                row_limit: None,
                            });
                        }
                        state.explicit_tx_active = false;
                        state.explicit_tx_started = None;
                        state.explicit_tx_warned = false;
                    }
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
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 22.0), egui::Sense::click());

    let fill = if response.hovered() {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);

    let center = rect.center();
    let arm = 5.0;
    let color = if response.hovered() {
        theme::text_primary()
    } else {
        theme::text_muted()
    };
    let plus_stroke = Stroke::new(1.5, color);
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
        theme::with_alpha(theme::accent_color(), 18)
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
            theme::accent_color(),
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
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(18, theme::SPACE_XS_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    toolbar_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(30.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;

            let has_connection = state.active_connection.is_some();
            let can_execute = has_connection && !state.query_running;

            // Run / Cancel button (primary pill with icon)
            if state.query_running {
                let cancel_btn = ui.add(
                    theme::primary_icon_button(
                        crate::ui::icon_image_tinted(ui, icons_svg::CLOSE, "ed_cancel2", 12.0, Color32::WHITE),
                        t("query_cancel"),
                    )
                    .stroke(Stroke::new(1.0, theme::ACCENT_RED))
                );
                if cancel_btn.clicked() {
                    if let Some(conn_id) = state.active_connection {
                        bridge.send(DbCommand::CancelQuery { conn_id });
                    }
                }
                ui.spinner();
            } else {
                let run_label = format!("  {}  \u{2318}\u{21B5}  ", t("query_execute"));
                let run_btn = ui.add(theme::primary_icon_button(
                    crate::ui::icon_image_tinted(ui, icons_svg::PLAY_SM, "ed_play2", 12.0, Color32::WHITE),
                    run_label,
                ));
                let shortcut_fired = can_execute
                    && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
                if shortcut_fired || (run_btn.clicked() && can_execute) {
                    execute_current_query(state, bridge);
                }
            }

            // Save button (secondary with icon)
            ui.add(theme::secondary_icon_button(
                crate::ui::icon_image_tinted(ui, icons_svg::SAVE, "ed_save2", 12.0, theme::text_secondary()),
                "Save",
            ));

            // History button (ghost with icon)
            let history_label = if state.show_history_panel {
                "History \u{2713}"
            } else {
                "History"
            };
            if ui
                .add(theme::ghost_icon_button(
                    crate::ui::icon_image_tinted(ui, icons_svg::HISTORY, "ed_history2", 12.0, theme::text_muted()),
                    history_label,
                ))
                .clicked()
            {
                state.show_history_panel = !state.show_history_panel;
            }

            // Separator
            let sep_rect = ui
                .allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover())
                .0;
            ui.painter()
                .rect_filled(sep_rect, CornerRadius::ZERO, theme::border_subtle());

            // Status badges
            let stmt_count = state
                .editor_tabs
                .get(state.active_tab)
                .map(|tab| {
                    tab.content
                        .split(';')
                        .filter(|s| !s.trim().is_empty())
                        .count()
                        .max(1)
                })
                .unwrap_or(1);
            render_badge_muted(ui, &format!("statement 1 of {stmt_count}"));

            if state.explicit_tx_active {
                render_badge_info(ui, "transaction active");
            } else {
                render_badge_info(ui, "auto-commit");
            }

            // Right side: cursor info + Format + EXPLAIN
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_SM;

                ui.add(theme::ghost_icon_button(
                    crate::ui::icon_image_tinted(ui, icons_svg::BRAIN, "ed_brain2", 12.0, theme::text_muted()),
                    "EXPLAIN",
                ));

                ui.add(theme::ghost_icon_button(
                    crate::ui::icon_image_tinted(ui, icons_svg::SIGMA, "ed_sigma2", 12.0, theme::text_muted()),
                    "Format",
                ));

                let sep_rect = ui
                    .allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover())
                    .0;
                ui.painter()
                    .rect_filled(sep_rect, CornerRadius::ZERO, theme::border_subtle());

                ui.label(
                    RichText::new("cursor 1:1")
                        .color(theme::text_muted())
                        .monospace()
                        .size(11.0),
                );
            });
        });
    });
}

fn render_badge_muted(ui: &mut egui::Ui, text: &str) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(10.5),
        theme::text_muted(),
    );
    let size = egui::vec2(galley.rect.width() + 14.0, 20.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(255),
        theme::bg_light(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(255),
        Stroke::new(1.0, theme::border_subtle()),
        eframe::egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.rect.width() / 2.0,
            rect.center().y - galley.rect.height() / 2.0,
        ),
        galley,
        theme::text_muted(),
    );
}

fn render_badge_info(ui: &mut egui::Ui, text: &str) {
    let color = theme::ACCENT_BLUE;
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(10.5),
        color,
    );
    let size = egui::vec2(galley.rect.width() + 14.0, 20.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(255),
        theme::with_alpha(color, 20),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(255),
        Stroke::new(1.0, theme::with_alpha(color, 50)),
        eframe::egui::StrokeKind::Inside,
    );
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.rect.width() / 2.0,
            rect.center().y - galley.rect.height() / 2.0,
        ),
        galley,
        color,
    );
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

    if state.show_history_panel {
        let panel_width = 280.0_f32.min(ui.available_width() * 0.35);
        egui::SidePanel::right("query_history_panel")
            .exact_width(panel_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_dark())
                    .inner_margin(Margin::same(theme::SPACE_SM as i8))
                    .stroke(Stroke::new(1.0, theme::border_subtle())),
            )
            .show_inside(ui, |ui| {
                render_history_panel(ui, state);
            });
    }

    let editor_frame = egui::Frame::new()
        .fill(theme::bg_editor())
        .inner_margin(Margin::ZERO);

    editor_frame.show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        ui.visuals_mut().faint_bg_color = theme::bg_editor();
        ui.visuals_mut().extreme_bg_color = theme::bg_editor();
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

        let mut dropped_payload: Option<String> = None;
        {
            let tab = &mut state.editor_tabs[active_tab];
            let editor_font_size = settings.font_size;
            let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                let layout_job = highlight_sql(text, wrap_width, editor_font_size);
                ui.fonts(|f| f.layout_job(layout_job))
            };

            let frame = egui::Frame::new()
                .fill(theme::bg_editor())
                .inner_margin(Margin::ZERO);
            let avail_h = ui.available_height();
            let (_inner, dropped) = ui.dnd_drop_zone::<crate::ui::TableDragPayload, ()>(
                frame,
                |ui| {
                    ui.set_min_height(avail_h);
                    // Paint full area with bg_editor first
                    ui.painter().rect_filled(ui.max_rect(), CornerRadius::ZERO, theme::bg_editor());
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        // Gutter: line numbers
                        let line_count = tab.content.lines().count().max(1);
                        let gutter_width = 44.0_f32;
                        let (gutter_rect, _) = ui.allocate_exact_size(
                            egui::vec2(gutter_width, avail_h),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(gutter_rect, CornerRadius::ZERO, theme::bg_editor());
                        ui.painter().vline(
                            gutter_rect.right(),
                            gutter_rect.y_range(),
                            Stroke::new(1.0, theme::border_subtle()),
                        );
                        let line_height = editor_font_size * 1.6;
                        for i in 0..line_count {
                            let y = gutter_rect.top() + 8.0 + i as f32 * line_height;
                            if y > gutter_rect.bottom() {
                                break;
                            }
                            ui.painter().text(
                                egui::pos2(gutter_rect.right() - 10.0, y),
                                egui::Align2::RIGHT_TOP,
                                format!("{}", i + 1),
                                egui::FontId::monospace(11.5),
                                theme::text_disabled(),
                            );
                        }

                        // Editor
                        egui::ScrollArea::vertical()
                            .id_salt(("editor_scroll", tab.id))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_min_height(avail_h - 4.0);
                                let te_id = egui::Id::new(("sql_editor", tab.id));
                                let output = egui::TextEdit::multiline(&mut tab.content)
                                    .id(te_id)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(100)
                                    .hint_text("SELECT *\nFROM public.table_name\nLIMIT 100;")
                                    .background_color(theme::bg_editor())
                                    .text_color(theme::text_primary())
                                    .margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_MD_I))
                                    .layouter(&mut layouter)
                                    .show(ui);
                                editor_rect = output.response.rect;
                                cursor_index =
                                    output.cursor_range.map(|range| range.primary.ccursor.index);
                                content_snapshot = tab.content.clone();
                            });
                    });
                },
            );
            if let Some(payload) = dropped {
                dropped_payload = Some(payload.text.clone());
            }
        }

        if let Some(insert) = dropped_payload {
            if let Some(tab) = state.editor_tabs.get_mut(active_tab) {
                let pos = cursor_index
                    .map(|idx| byte_index_for_char(&tab.content, idx))
                    .unwrap_or_else(|| tab.content.len());
                tab.content.insert_str(pos, &insert);
                let new_char_pos = cursor_index.unwrap_or(tab.content.chars().count())
                    + insert.chars().count();
                let te_id = egui::Id::new(("sql_editor", tab.id));
                if let Some(mut te_state) = egui::TextEdit::load_state(ui.ctx(), te_id) {
                    use egui::text::{CCursor, CCursorRange};
                    te_state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(CCursor::new(new_char_pos))));
                    te_state.store(ui.ctx(), te_id);
                }
                content_snapshot = tab.content.clone();
            }
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
                    let new_cursor_pos = completion_cursor_pos(&insert);
                    apply_completion(&mut tab.content, &insert);
                    let snapshot = tab.content.clone();
                    ui.data_mut(|d| {
                        d.insert_persisted(
                            egui::Id::new(("sql_completion_applied", tab_id)).with("content"),
                            snapshot,
                        )
                    });
                    let te_id = egui::Id::new(("sql_editor", tab_id));
                    if let Some(mut te_state) =
                        egui::TextEdit::load_state(ui.ctx(), te_id)
                    {
                        use egui::text::{CCursor, CCursorRange};
                        te_state
                            .cursor
                            .set_char_range(Some(CCursorRange::one(CCursor::new(new_cursor_pos))));
                        te_state.store(ui.ctx(), te_id);
                    }
                }
            }
        }
    });
}

fn byte_index_for_char(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(byte, _)| byte)
        .unwrap_or_else(|| s.len())
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

#[allow(clippy::too_many_arguments)]
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

    let applied_id = egui::Id::new(("sql_completion_applied", tab_id));
    let applied_content: String =
        ui.data_mut(|d| d.get_persisted(applied_id.with("content")).unwrap_or_default());
    if !applied_content.is_empty() && applied_content == content {
        return None;
    }
    if !applied_content.is_empty() && applied_content != content {
        ui.data_mut(|d| d.insert_persisted(applied_id.with("content"), String::new()));
    }

    let cursor = cursor_index.unwrap_or_else(|| content.chars().count());
    let context = completion_context(content, cursor)?;
    let suggestions = collect_completions(state, &context);
    let popup_id = egui::Id::new(("sql_completion_sel", tab_id));

    if suggestions.is_empty() {
        ui.data_mut(|d| d.insert_persisted(popup_id, usize::MAX));
        return None;
    }

    // Hide popup when typed token exactly matches the only suggestion
    if suggestions.len() == 1
        && suggestions[0].insert_text.eq_ignore_ascii_case(&context.fragment)
    {
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
        theme::with_alpha(theme::accent_color(), 45)
    } else if response.hovered() {
        theme::with_alpha(theme::accent_color(), 28)
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
                theme::accent_color(),
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
                        color: theme::accent_color_light(),
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
        "MATERIALIZED VIEW" => theme::accent_color_light(),
        _ => theme::accent_color(),
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
    let with_space = format!("{} ", insert.text);
    content.replace_range(start..end, &with_space);
}

/// Tab 수락 후 egui `CCursor` 가 위치할 char index — 완성된 텍스트 끝 + trailing
/// space 1칸. `CCursor` 는 *char* 기반이므로 `text.chars().count()` 로 계산해야
/// 한다 (byte length 사용 시 non-ASCII identifier 에서 cursor 가 어긋난다).
fn completion_cursor_pos(insert: &CompletionInsert) -> usize {
    insert.start_char + insert.text.chars().count() + 1
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

/// US-J2 — classify_explicit_tx 결과에 따라 state.explicit_tx_* 토글.
/// `execute_current_query` 와 단위 테스트 양쪽에서 호출.
pub(crate) fn toggle_explicit_tx_for_sql(state: &mut AppState, sql: &str) {
    use crate::db::begin_detect::{classify_explicit_tx, ExplicitTxClass};
    match classify_explicit_tx(sql) {
        ExplicitTxClass::Begin => {
            state.explicit_tx_active = true;
            state.explicit_tx_started = Some(std::time::Instant::now());
            state.explicit_tx_warned = false;
        }
        ExplicitTxClass::Commit | ExplicitTxClass::Rollback => {
            state.explicit_tx_active = false;
            state.explicit_tx_started = None;
            state.explicit_tx_warned = false;
        }
        _ => {}
    }
}

fn execute_current_query(state: &mut AppState, bridge: &DbBridge) {
    if let Some(conn_id) = state.active_connection {
        if let Some(tab) = state.editor_tabs.get(state.active_tab) {
            let sql = tab.content.trim().to_string();
            if !sql.is_empty() {
                toggle_explicit_tx_for_sql(state, &sql);
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

fn highlight_sql(text: &str, wrap_width: f32, font_size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::FontId::monospace(font_size);
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

fn render_history_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.strong(RichText::new("Query History").size(12.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("×").clicked() {
                state.show_history_panel = false;
            }
        });
    });
    ui.separator();

    if state.query_history.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("No queries yet")
                    .color(theme::text_muted())
                    .size(11.0),
            );
        });
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("history_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut load_query = None;
            for (idx, entry) in state.query_history.iter().rev().enumerate() {
                let preview: String = entry
                    .query
                    .chars()
                    .take(120)
                    .collect::<String>()
                    .replace('\n', " ");
                let time_str = entry.timestamp.format("%m-%d %H:%M").to_string();

                let frame = egui::Frame::new()
                    .fill(theme::bg_medium())
                    .inner_margin(Margin::same(theme::SPACE_SM as i8))
                    .corner_radius(egui::CornerRadius::same(theme::RADIUS_SM));

                let resp = frame
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&time_str)
                                    .color(theme::text_disabled())
                                    .size(9.5)
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(format!("{}ms", entry.duration_ms))
                                    .color(theme::accent_color())
                                    .size(9.5),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{} {}",
                                    entry.row_count,
                                    if entry.row_count == 1 { "row" } else { "rows" }
                                ))
                                .color(theme::text_muted())
                                .size(9.5),
                            );
                        });
                        ui.label(
                            RichText::new(&preview)
                                .color(theme::text_secondary())
                                .size(11.0)
                                .monospace(),
                        );
                    })
                    .response;

                let resp = ui.interact(
                    resp.rect,
                    ui.id().with(("history_item", idx)),
                    egui::Sense::click(),
                );
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    load_query = Some(entry.query.clone());
                }
                ui.add_space(2.0);
            }

            if let Some(query) = load_query {
                if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
                    tab.content = query;
                }
            }
        });
}

#[inline]
fn fmt(font_id: egui::FontId, color: Color32) -> egui::text::TextFormat {
    egui::text::TextFormat::simple(font_id, color)
}

#[cfg(test)]
mod completion_tests {
    use super::*;

    fn make_insert(start: usize, end: usize, text: &str) -> CompletionInsert {
        CompletionInsert {
            start_char: start,
            end_char: end,
            text: text.to_string(),
        }
    }

    #[test]
    fn apply_completion_replaces_partial_token_with_full_text_and_space() {
        let mut content = "SELECT * FROM us".to_string();
        let insert = make_insert(14, 16, "users");
        apply_completion(&mut content, &insert);
        assert_eq!(content, "SELECT * FROM users ");
    }

    #[test]
    fn apply_completion_handles_empty_fragment() {
        let mut content = "SELECT ".to_string();
        let insert = make_insert(7, 7, "id");
        apply_completion(&mut content, &insert);
        assert_eq!(content, "SELECT id ");
    }

    #[test]
    fn apply_completion_in_middle_of_buffer() {
        let mut content = "SELECT  FROM t".to_string();
        let insert = make_insert(7, 7, "name");
        apply_completion(&mut content, &insert);
        assert_eq!(content, "SELECT name  FROM t");
    }

    #[test]
    fn cursor_pos_after_ascii_completion_is_text_end_plus_space() {
        let insert = make_insert(14, 16, "users");
        // start=14 + len('users')=5 + 1 (space) = 20
        assert_eq!(completion_cursor_pos(&insert), 20);
    }

    #[test]
    fn cursor_pos_uses_char_count_not_byte_count_for_non_ascii() {
        // Korean column name: 이름 = 2 chars, 6 bytes (UTF-8)
        let insert = make_insert(7, 7, "이름");
        // start=7 + chars('이름')=2 + 1 (space) = 10 (NOT 14 from byte count)
        assert_eq!(completion_cursor_pos(&insert), 10);
    }

    #[test]
    fn cursor_pos_handles_quoted_identifier() {
        let insert = make_insert(7, 9, "\"my col\"");
        // start=7 + chars('"my col"')=8 + 1 (space) = 16
        assert_eq!(completion_cursor_pos(&insert), 16);
    }

    #[test]
    fn apply_completion_preserves_content_after_replacement_window() {
        let mut content = "SELECT us FROM t".to_string();
        let insert = make_insert(7, 9, "users");
        apply_completion(&mut content, &insert);
        assert_eq!(content, "SELECT users  FROM t");
    }

    #[test]
    fn char_to_byte_index_at_end_returns_total_byte_length() {
        let text = "abc";
        assert_eq!(char_to_byte_index(text, 3), 3);
        assert_eq!(char_to_byte_index(text, 100), 3); // out of range → end
    }

    #[test]
    fn char_to_byte_index_for_multibyte() {
        let text = "이름";
        // each Korean char is 3 bytes in UTF-8
        assert_eq!(char_to_byte_index(text, 0), 0);
        assert_eq!(char_to_byte_index(text, 1), 3);
        assert_eq!(char_to_byte_index(text, 2), 6); // beyond last char → end
    }
}

#[cfg(test)]
mod begin_tracking_tests {
    use super::*;

    #[test]
    fn begin_sets_explicit_tx_active() {
        let mut state = AppState::default();
        toggle_explicit_tx_for_sql(&mut state, "BEGIN");
        assert!(state.explicit_tx_active);
        assert!(state.explicit_tx_started.is_some());
    }

    #[test]
    fn begin_select_commit_sequence_toggles_correctly() {
        let mut state = AppState::default();
        toggle_explicit_tx_for_sql(&mut state, "BEGIN");
        assert!(state.explicit_tx_active);
        toggle_explicit_tx_for_sql(&mut state, "SELECT 1");
        assert!(state.explicit_tx_active, "SELECT keeps tx active");
        toggle_explicit_tx_for_sql(&mut state, "COMMIT");
        assert!(!state.explicit_tx_active);
        assert!(state.explicit_tx_started.is_none());
    }

    #[test]
    fn rollback_resets_active_state() {
        let mut state = AppState {
            explicit_tx_active: true,
            explicit_tx_started: Some(std::time::Instant::now()),
            explicit_tx_warned: true,
            ..AppState::default()
        };
        toggle_explicit_tx_for_sql(&mut state, "ROLLBACK");
        assert!(!state.explicit_tx_active);
        assert!(state.explicit_tx_started.is_none());
        assert!(!state.explicit_tx_warned);
    }
}
