use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Sense, Stroke};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::AppState;
use crate::storage::history::HistoryEntry;
use crate::types::{ConnectionId, EditorTab};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, ThemeMode, Tokens};

// ============================================================================
// Item / action
// ============================================================================

#[derive(Clone)]
enum Action {
    SwitchConnection(ConnectionId),
    OpenTable {
        conn_id: ConnectionId,
        schema: String,
        table: String,
    },
    LoadHistory(usize),
    SetTheme(ThemeMode),
    ToggleSidebar,
    ToggleResults,
    NewConnection,
    NewQueryTab,
}

#[derive(Clone)]
struct Item {
    label: String,
    sub: String,
    icon: Icon,
    icon_color: ColorKey,
    action: Action,
}

#[derive(Clone, Copy)]
enum ColorKey {
    Accent,
    #[allow(dead_code)]
    Info,
    #[allow(dead_code)]
    Warn,
    Success,
    Muted,
}

impl ColorKey {
    fn resolve(self, t: Tokens) -> Color32 {
        match self {
            ColorKey::Accent => t.accent,
            ColorKey::Info => t.info,
            ColorKey::Warn => t.warn,
            ColorKey::Success => t.success,
            ColorKey::Muted => t.text_muted,
        }
    }
}

// ============================================================================
// Public entry
// ============================================================================

pub fn render(
    ctx: &egui::Context,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
    history: &[HistoryEntry],
) {
    if !state.command_palette.open {
        return;
    }

    // Esc closes
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.command_palette.close();
        return;
    }

    let items = build_items(state, history);
    let filtered = filter_items(&items, &state.command_palette.query);

    if state.command_palette.selected_index >= filtered.len() {
        state.command_palette.selected_index =
            filtered.len().saturating_sub(1).max(0);
    }

    // Arrow keys
    let len = filtered.len();
    ctx.input_mut(|i| {
        if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) && len > 0 {
            state.command_palette.selected_index =
                (state.command_palette.selected_index + 1) % len;
        }
        if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) && len > 0 {
            state.command_palette.selected_index =
                (state.command_palette.selected_index + len - 1) % len;
        }
    });

    // Enter executes
    let enter_pressed = ctx.input(|i| i.key_pressed(egui::Key::Enter));

    let screen = ctx.screen_rect();
    let win_w = 640.0;
    let win_h = 440.0;
    let pos = egui::pos2(
        screen.center().x - win_w / 2.0,
        screen.top() + 96.0,
    );

    let mut close_requested = false;
    let mut chosen: Option<Action> = None;

    egui::Area::new(egui::Id::new("ferrum_command_palette_overlay"))
        .order(egui::Order::Foreground)
        .interactable(true)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            // Dimmed backdrop
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, t.bg_overlay);
        });

    egui::Window::new("ferrum_command_palette")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_pos(pos)
        .fixed_size(egui::vec2(win_w, win_h))
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(t.bg_surface)
                .stroke(Stroke::new(1.0, t.border_default))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::ZERO)
                .shadow(egui::Shadow {
                    offset: [0, 16],
                    blur: 40,
                    spread: 0,
                    color: Color32::from_black_alpha(120),
                }),
        )
        .show(ctx, |ui| {
            // Search input
            let header = egui::Frame::new()
                .fill(t.bg_surface)
                .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_MD_I))
                .stroke(Stroke::new(1.0, t.border_subtle));
            header.show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    icons::icon(ui, Icon::Search, 16.0, t.text_muted);
                    let text_edit = egui::TextEdit::singleline(
                        &mut state.command_palette.query,
                    )
                    .hint_text(
                        "Type a table, query, command\u{2026}",
                    )
                    .desired_width(f32::INFINITY)
                    .frame(false)
                    .font(egui::FontId::proportional(14.0));
                    let resp = ui.add(text_edit);
                    if state.command_palette.focus_requested {
                        resp.request_focus();
                        state.command_palette.focus_requested = false;
                    }
                });
            });

            // List body
            egui::ScrollArea::vertical()
                .id_salt("palette_list")
                .max_height(win_h - 60.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.add_space(theme::SPACE_XL);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("No results")
                                    .color(t.text_muted)
                                    .size(13.0),
                            );
                        });
                        return;
                    }
                    for (i, item) in filtered.iter().enumerate() {
                        let active = i == state.command_palette.selected_index;
                        let row_h = 40.0;
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), row_h),
                            Sense::click(),
                        );
                        if resp.hovered() {
                            state.command_palette.selected_index = i;
                        }
                        let bg = if active {
                            t.accent_soft
                        } else if resp.hovered() {
                            t.bg_elev
                        } else {
                            Color32::TRANSPARENT
                        };
                        ui.painter().rect_filled(rect, 0.0, bg);
                        if active {
                            let stripe = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(2.0, rect.height()),
                            );
                            ui.painter().rect_filled(stripe, 0.0, t.accent);
                        }

                        // Vector icon
                        let icon_rect = egui::Rect::from_center_size(
                            egui::pos2(
                                rect.left() + theme::SPACE_LG + 8.0,
                                rect.center().y,
                            ),
                            egui::vec2(16.0, 16.0),
                        );
                        icons::icon_at(
                            ui.painter(),
                            item.icon,
                            icon_rect,
                            item.icon_color.resolve(t),
                        );
                        // Label
                        ui.painter().text(
                            rect.min + egui::vec2(40.0, rect.height() / 2.0 - 7.0),
                            egui::Align2::LEFT_TOP,
                            &item.label,
                            egui::FontId::proportional(13.0),
                            t.text_primary,
                        );
                        if !item.sub.is_empty() {
                            ui.painter().text(
                                rect.right_center()
                                    - egui::vec2(theme::SPACE_LG, 0.0),
                                egui::Align2::RIGHT_CENTER,
                                &item.sub,
                                egui::FontId::proportional(11.0),
                                t.text_muted,
                            );
                        }

                        if resp.clicked() {
                            chosen = Some(item.action.clone());
                            close_requested = true;
                        }
                    }
                });

            if enter_pressed && !filtered.is_empty() {
                let i = state.command_palette.selected_index;
                if let Some(item) = filtered.get(i) {
                    chosen = Some(item.action.clone());
                    close_requested = true;
                }
            }
        });

    if let Some(action) = chosen {
        execute_action(action, state, bridge, history);
    }
    if close_requested {
        state.command_palette.close();
    }
}

// ============================================================================
// Item building
// ============================================================================

fn build_items(state: &AppState, history: &[HistoryEntry]) -> Vec<Item> {
    let mut items = Vec::new();

    // Connections (switch)
    for (id, conn) in &state.connections {
        items.push(Item {
            label: format!("Switch to {}", conn.config.display_name),
            sub: format!("{}/{}", conn.config.host, conn.config.database),
            icon: Icon::Connection,
            icon_color: ColorKey::Success,
            action: Action::SwitchConnection(*id),
        });
    }

    // Tables across connections
    for (conn_id, conn) in &state.connections {
        for (schema, tables) in &conn.tables {
            for tbl in tables {
                items.push(Item {
                    label: format!("{}.{}", schema, tbl.name),
                    sub: format!(
                        "{}  /  {}",
                        conn.config.display_name, tbl.table_type
                    ),
                    icon: Icon::Table,
                    icon_color: ColorKey::Accent,
                    action: Action::OpenTable {
                        conn_id: *conn_id,
                        schema: schema.clone(),
                        table: tbl.name.clone(),
                    },
                });
            }
        }
    }

    // History (recent first, dedup)
    let mut seen = std::collections::HashSet::new();
    for (i, entry) in history.iter().rev().enumerate() {
        let key = entry.query.clone();
        if !seen.insert(key.clone()) {
            continue;
        }
        let preview = entry
            .query
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(70)
            .collect::<String>();
        let when = entry.timestamp.format("%H:%M:%S").to_string();
        items.push(Item {
            label: preview,
            sub: format!("{}  \u{00B7} {}ms", when, entry.duration_ms),
            icon: Icon::Clock,
            icon_color: ColorKey::Muted,
            action: Action::LoadHistory(history.len() - 1 - i),
        });
        if i > 25 {
            break;
        }
    }

    // Built-in actions
    items.push(Item {
        label: "New connection\u{2026}".to_string(),
        sub: "\u{2318}N".to_string(),
        icon: Icon::Plus,
        icon_color: ColorKey::Accent,
        action: Action::NewConnection,
    });
    items.push(Item {
        label: "New query tab".to_string(),
        sub: "\u{2318}T".to_string(),
        icon: Icon::Plus,
        icon_color: ColorKey::Muted,
        action: Action::NewQueryTab,
    });
    items.push(Item {
        label: "Toggle sidebar".to_string(),
        sub: "\u{2318}B".to_string(),
        icon: Icon::Form,
        icon_color: ColorKey::Muted,
        action: Action::ToggleSidebar,
    });
    items.push(Item {
        label: "Toggle result panel".to_string(),
        sub: "\u{2318}J".to_string(),
        icon: Icon::Grid,
        icon_color: ColorKey::Muted,
        action: Action::ToggleResults,
    });

    for mode in [ThemeMode::Auto, ThemeMode::Light, ThemeMode::Dark] {
        items.push(Item {
            label: format!("Theme: {}", mode.label()),
            sub: String::new(),
            icon: Icon::Settings,
            icon_color: ColorKey::Muted,
            action: Action::SetTheme(mode),
        });
    }

    items
}

fn filter_items(items: &[Item], query: &str) -> Vec<Item> {
    if query.is_empty() {
        return items.iter().take(40).cloned().collect();
    }
    let needle = query.to_ascii_lowercase();
    let mut scored: Vec<(usize, Item)> = items
        .iter()
        .filter_map(|it| {
            let hay = format!(
                "{} {}",
                it.label.to_ascii_lowercase(),
                it.sub.to_ascii_lowercase()
            );
            let pos = hay.find(&needle)?;
            // earlier match → lower score (better)
            Some((pos, it.clone()))
        })
        .collect();
    scored.sort_by_key(|(p, _)| *p);
    scored.into_iter().take(60).map(|(_, i)| i).collect()
}

// ============================================================================
// Action execution
// ============================================================================

fn execute_action(
    action: Action,
    state: &mut AppState,
    bridge: &DbBridge,
    history: &[HistoryEntry],
) {
    match action {
        Action::SwitchConnection(id) => {
            state.active_connection = Some(id);
        }
        Action::OpenTable {
            conn_id,
            schema,
            table,
        } => {
            let sql = format!(
                "SELECT * FROM \"{}\".\"{}\" LIMIT 100",
                schema.replace('"', "\"\""),
                table.replace('"', "\"\""),
            );
            let title = format!("{}.{}", schema, table);
            let mut tab = EditorTab::new(title);
            tab.content = sql.clone();
            state.editor_tabs.push(tab);
            state.active_tab = state.editor_tabs.len() - 1;
            state.active_connection = Some(conn_id);
            state.query_running = true;
            state.last_error = None;
            bridge.send(DbCommand::ExecuteQuery {
                conn_id,
                sql,
                row_limit: Some(100),
            });
        }
        Action::LoadHistory(idx) => {
            if let Some(entry) = history.get(idx) {
                if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
                    tab.content = entry.query.clone();
                }
            }
        }
        Action::SetTheme(mode) => {
            state.theme_mode = mode;
        }
        Action::ToggleSidebar => {
            state.sidebar_visible = !state.sidebar_visible;
        }
        Action::ToggleResults => {
            state.result_panel_visible = !state.result_panel_visible;
        }
        Action::NewConnection => {
            state.show_connection_dialog = true;
            state.connection_dialog = Default::default();
        }
        Action::NewQueryTab => {
            let n = state.editor_tabs.len() + 1;
            state
                .editor_tabs
                .push(EditorTab::new(format!("Query {n}")));
            state.active_tab = state.editor_tabs.len() - 1;
        }
    }
}
