use eframe::egui::{self, Color32, Margin, RichText, Sense, Stroke};

use crate::db::bridge::DbBridge;
use crate::state::{AppState, ConnectionStatus, ObjectFilter};
use crate::storage::history::HistoryEntry;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, BtnKind, FerrumTheme, ThemeMode, Tokens};
use crate::ui::{command_palette, editor, grid, icons, tree_browser};

// ============================================================================
// Public entry
// ============================================================================

pub fn render_app(
    ctx: &egui::Context,
    state: &mut AppState,
    bridge: &DbBridge,
    history: &[HistoryEntry],
) {
    FerrumTheme::apply_mode(ctx, state.theme_mode);
    handle_shortcuts(ctx, state);

    let t = Tokens::current(ctx);

    render_header(ctx, t, state);
    render_object_toolbar(ctx, t, state);
    render_status_bar(ctx, t, state);

    if state.sidebar_visible {
        render_sidebar(ctx, t, state, bridge, history);
    }
    if state.result_panel_visible {
        render_result_panel(ctx, t, state);
    }
    render_workspace(ctx, t, state, bridge);

    command_palette::render(ctx, t, state, bridge, history);
}

/// Backward-compat alias kept so callers without history still build.
pub fn render_panels(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    render_app(ctx, state, bridge, &[]);
}

fn handle_shortcuts(ctx: &egui::Context, state: &mut AppState) {
    let cmd = egui::Modifiers::COMMAND;
    ctx.input_mut(|i| {
        if i.consume_key(cmd, egui::Key::K) {
            state.command_palette.open();
        }
        if i.consume_key(cmd, egui::Key::B) {
            state.sidebar_visible = !state.sidebar_visible;
        }
        if i.consume_key(cmd, egui::Key::J) {
            state.result_panel_visible = !state.result_panel_visible;
        }
        if i.consume_key(cmd, egui::Key::T) {
            let n = state.editor_tabs.len() + 1;
            state
                .editor_tabs
                .push(crate::types::EditorTab::new(format!("Query {n}")));
            state.active_tab = state.editor_tabs.len() - 1;
        }
    });
}

// ============================================================================
// Header (36px)
// ============================================================================

fn render_header(ctx: &egui::Context, t: Tokens, state: &mut AppState) {
    egui::TopBottomPanel::top("ferrum_header")
        .exact_height(36.0)
        .frame(
            egui::Frame::new()
                .fill(t.bg_app)
                .inner_margin(Margin::symmetric(theme::SPACE_MD_I, 0))
                .stroke(Stroke::new(1.0, t.border_subtle)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(36.0);
            ui.horizontal_centered(|ui| {
                // Logo + wordmark
                let (logo_rect, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());
                icons::icon_at(ui.painter(), Icon::Logo, logo_rect, t.accent);
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new("FerrumGrid")
                        .color(t.text_primary)
                        .size(13.0)
                        .strong(),
                );

                ui.add_space(theme::SPACE_LG);

                // Cmd+K search pill — custom layout with vector icons
                let pill_w = 240.0_f32;
                let pill_h = 24.0_f32;
                let (pill_rect, pill_resp) = ui.allocate_exact_size(
                    egui::vec2(pill_w, pill_h),
                    Sense::click(),
                );
                let pill_bg = if pill_resp.hovered() { t.bg_surface } else { t.bg_elev };
                ui.painter().rect_filled(
                    pill_rect,
                    eframe::egui::CornerRadius::same(theme::RADIUS_MD),
                    pill_bg,
                );
                ui.painter().rect_stroke(
                    pill_rect,
                    eframe::egui::CornerRadius::same(theme::RADIUS_MD),
                    Stroke::new(1.0, t.border_subtle),
                    eframe::egui::epaint::StrokeKind::Inside,
                );
                // Search icon on left
                let search_rect = egui::Rect::from_center_size(
                    egui::pos2(pill_rect.left() + 14.0, pill_rect.center().y),
                    egui::vec2(12.0, 12.0),
                );
                icons::icon_at(ui.painter(), Icon::Search, search_rect, t.text_muted);
                // "Search…" text
                ui.painter().text(
                    egui::pos2(pill_rect.left() + 26.0, pill_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Search\u{2026}",
                    egui::FontId::proportional(12.0),
                    t.text_muted,
                );
                // Command icon + "K" on right
                let cmd_rect = egui::Rect::from_center_size(
                    egui::pos2(pill_rect.right() - 22.0, pill_rect.center().y),
                    egui::vec2(10.0, 10.0),
                );
                icons::icon_at(ui.painter(), Icon::Command, cmd_rect, t.text_muted);
                ui.painter().text(
                    egui::pos2(pill_rect.right() - 10.0, pill_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    "K",
                    egui::FontId::proportional(11.0),
                    t.text_muted,
                );
                if pill_resp.clicked() {
                    state.command_palette.open();
                }

                // Right side: theme + connection chip
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        render_connection_chip(ui, t, state);
                        ui.add_space(theme::SPACE_SM);
                        render_theme_switcher(ui, t, state);
                    },
                );
            });
        });
}

fn render_connection_chip(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let (label, dot_color) = match state.active_connection {
        Some(id) => match state.connections.get(&id) {
            Some(conn) => {
                let color = match conn.status {
                    ConnectionStatus::Connected { .. } => t.success,
                    ConnectionStatus::Connecting => t.warn,
                    ConnectionStatus::Disconnected => t.danger,
                };
                let lbl = format!(
                    "{}  /  {}",
                    conn.config.display_name, conn.config.database
                );
                (lbl, color)
            }
            None => ("No connection".to_string(), t.text_disabled),
        },
        None => ("No connection".to_string(), t.text_disabled),
    };

    let response = egui::menu::menu_custom_button(
        ui,
        egui::Button::new(
            RichText::new(format!("    {label}    \u{25BE}"))
                .color(t.text_secondary)
                .size(12.0),
        )
        .fill(t.bg_elev)
        .stroke(Stroke::new(1.0, t.border_subtle))
        .corner_radius(eframe::egui::CornerRadius::same(theme::RADIUS_MD))
        .min_size(egui::vec2(0.0, 24.0)),
        |ui| {
            ui.set_min_width(280.0);
            ui.label(
                RichText::new("Connections")
                    .color(t.text_muted)
                    .size(11.0)
                    .strong(),
            );
            ui.separator();
            let conn_ids: Vec<_> = state.connections.keys().copied().collect();
            if conn_ids.is_empty() {
                ui.label(
                    RichText::new("No active connections")
                        .color(t.text_disabled)
                        .size(12.0),
                );
            }
            for id in conn_ids {
                let (name, status_color, db) = {
                    let c = state.connections.get(&id).unwrap();
                    let color = match c.status {
                        ConnectionStatus::Connected { .. } => t.success,
                        ConnectionStatus::Connecting => t.warn,
                        ConnectionStatus::Disconnected => t.danger,
                    };
                    (
                        c.config.display_name.clone(),
                        color,
                        c.config.database.clone(),
                    )
                };
                let active = state.active_connection == Some(id);
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(rect.center(), 3.5, status_color);
                    let label_text = format!("{name}    \u{00B7}/{db}");
                    let resp = ui.add(
                        egui::Label::new(
                            RichText::new(label_text)
                                .color(if active {
                                    t.text_primary
                                } else {
                                    t.text_secondary
                                })
                                .size(12.0),
                        )
                        .sense(Sense::click()),
                    );
                    if resp.clicked() {
                        state.active_connection = Some(id);
                        ui.close_menu();
                    }
                });
            }
            ui.separator();
            if theme::icon_button_sm(ui, BtnKind::Ghost, Icon::Plus, "New Connection", t, true)
                .clicked()
            {
                state.show_connection_dialog = true;
                state.connection_dialog = Default::default();
                ui.close_menu();
            }
        },
    );

    // Paint the status dot inside the chip on the left
    let rect = response.response.rect;
    let dot_center = egui::pos2(rect.left() + 10.0, rect.center().y);
    ui.painter().circle_filled(dot_center, 4.0, dot_color);
}

fn render_theme_switcher(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    egui::menu::menu_custom_button(
        ui,
        egui::Button::new(
            RichText::new(state.theme_mode.label())
                .color(t.text_secondary)
                .size(12.0),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE),
        |ui| {
            ui.set_min_width(140.0);
            for mode in [ThemeMode::Auto, ThemeMode::Light, ThemeMode::Dark] {
                let active = state.theme_mode == mode;
                let label = if active {
                    format!("\u{2713}  {}", mode.label())
                } else {
                    format!("   {}", mode.label())
                };
                if ui.add(theme::ghost_button(t, &label)).clicked() {
                    state.theme_mode = mode;
                    ui.close_menu();
                }
            }
        },
    );
}

// ============================================================================
// Object toolbar (40px)
// ============================================================================

fn render_object_toolbar(ctx: &egui::Context, t: Tokens, state: &mut AppState) {
    egui::TopBottomPanel::top("object_toolbar")
        .exact_height(40.0)
        .frame(
            egui::Frame::new()
                .fill(t.bg_app)
                .inner_margin(Margin::symmetric(theme::SPACE_MD_I, 0))
                .stroke(Stroke::new(1.0, t.border_subtle)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(40.0);
            ui.horizontal_centered(|ui| {
                // Vector-icon tabs (All, History)
                object_tab_icon(ui, t, ObjectFilter::All, Icon::Database, "All", &mut state.object_filter);
                // Monogram chip tabs (Tables, Views, Functions, Queries)
                object_tab_mono(ui, t, ObjectFilter::Tables, icons::MONO_TABLE, "Tables", &mut state.object_filter);
                object_tab_mono(ui, t, ObjectFilter::Views, icons::MONO_VIEW, "Views", &mut state.object_filter);
                object_tab_mono(ui, t, ObjectFilter::Functions, icons::MONO_FUNCTION, "Functions", &mut state.object_filter);
                object_tab_mono(ui, t, ObjectFilter::Queries, icons::MONO_QUERY, "Queries", &mut state.object_filter);
                object_tab_icon(ui, t, ObjectFilter::History, Icon::Clock, "History", &mut state.object_filter);
            });
        });
}

/// Tab with a vector icon on the left.
fn object_tab_icon(
    ui: &mut egui::Ui,
    t: Tokens,
    filter: ObjectFilter,
    icon: Icon,
    label: &str,
    current: &mut ObjectFilter,
) {
    let active = *current == filter;
    let icon_size = 13.0;
    let gap = 5.0;
    let pad_x = 10.0;
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    let label_w = pad_x + icon_size + gap + galley.rect.width() + pad_x;
    let h = 40.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(label_w, h), Sense::click());

    let bg = if resp.hovered() && !active { t.bg_elev } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, 0.0, bg);
    if active {
        let underline = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 6.0, rect.bottom() - 2.0),
            egui::pos2(rect.right() - 6.0, rect.bottom()),
        );
        ui.painter().rect_filled(underline, 0.0, t.accent);
    }

    let fg = if active { t.text_primary } else { t.text_secondary };
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + pad_x + icon_size / 2.0, rect.center().y),
        egui::vec2(icon_size, icon_size),
    );
    icons::icon_at(ui.painter(), icon, icon_rect, fg);
    ui.painter().text(
        egui::pos2(rect.left() + pad_x + icon_size + gap, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        fg,
    );

    if resp.clicked() {
        *current = filter;
    }
}

/// Tab with a monogram letter chip on the left.
fn object_tab_mono(
    ui: &mut egui::Ui,
    t: Tokens,
    filter: ObjectFilter,
    mono: &str,
    label: &str,
    current: &mut ObjectFilter,
) {
    let active = *current == filter;
    let chip_size = 13.0;
    let gap = 5.0;
    let pad_x = 10.0;
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    let label_w = pad_x + chip_size + gap + galley.rect.width() + pad_x;
    let h = 40.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(label_w, h), Sense::click());

    let bg = if resp.hovered() && !active { t.bg_elev } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, 0.0, bg);
    if active {
        let underline = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 6.0, rect.bottom() - 2.0),
            egui::pos2(rect.right() - 6.0, rect.bottom()),
        );
        ui.painter().rect_filled(underline, 0.0, t.accent);
    }

    let fg = if active { t.text_primary } else { t.text_secondary };
    // Inline monogram letter (no colored chip background in tab bar — just the letter)
    ui.painter().text(
        egui::pos2(rect.left() + pad_x + chip_size / 2.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        mono,
        egui::FontId::proportional(11.0),
        fg,
    );
    ui.painter().text(
        egui::pos2(rect.left() + pad_x + chip_size + gap, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        fg,
    );

    if resp.clicked() {
        *current = filter;
    }
}

// ============================================================================
// Sidebar
// ============================================================================

fn render_sidebar(
    ctx: &egui::Context,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
    history: &[HistoryEntry],
) {
    egui::SidePanel::left("ferrum_sidebar")
        .default_width(280.0)
        .min_width(220.0)
        .max_width(480.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(t.bg_sidebar)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            render_sidebar_header(ui, t, state);
            ui.painter().hline(
                ui.available_rect_before_wrap().x_range(),
                ui.cursor().top(),
                Stroke::new(1.0, t.border_subtle),
            );

            egui::ScrollArea::both()
                .id_salt("sidebar_body")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(theme::SPACE_SM);
                    match state.object_filter {
                        ObjectFilter::History => {
                            render_history_list(ui, t, state, history);
                        }
                        ObjectFilter::Queries => {
                            render_queries_placeholder(ui, t);
                        }
                        _ => {
                            tree_browser::render_tree(ui, state, bridge);
                        }
                    }
                    ui.add_space(theme::SPACE_LG);
                });
        });
}

fn render_sidebar_header(ui: &mut egui::Ui, t: Tokens, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(t.bg_sidebar)
        .inner_margin(Margin::symmetric(theme::SPACE_MD_I, theme::SPACE_SM_I));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(state.object_filter.label().to_uppercase())
                    .color(t.text_muted)
                    .size(10.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::icon_button_sm(ui, BtnKind::Secondary, Icon::Plus, "New", t, true)
                    .clicked()
                {
                    state.show_connection_dialog = true;
                    state.connection_dialog = Default::default();
                }
            });
        });
        ui.add_space(theme::SPACE_XS);
        let search = egui::TextEdit::singleline(&mut state.sidebar_search)
            .hint_text("Filter\u{2026}")
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(8.0, 4.0));
        ui.add(search);
    });
}

fn render_queries_placeholder(ui: &mut egui::Ui, t: Tokens) {
    ui.add_space(theme::SPACE_XXL);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(icons::MONO_QUERY)
                .color(t.text_disabled)
                .size(28.0),
        );
        ui.add_space(theme::SPACE_MD);
        ui.label(
            RichText::new("No saved queries")
                .color(t.text_muted)
                .size(12.0),
        );
        ui.label(
            RichText::new("Save current editor with \u{2318}S")
                .color(t.text_disabled)
                .size(11.0),
        );
    });
}

fn render_history_list(
    ui: &mut egui::Ui,
    t: Tokens,
    state: &mut AppState,
    history: &[HistoryEntry],
) {
    if history.is_empty() {
        ui.add_space(theme::SPACE_XXL);
        ui.vertical_centered(|ui| {
            icons::icon(ui, Icon::Clock, 28.0, t.text_disabled);
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new("No history yet")
                    .color(t.text_muted)
                    .size(12.0),
            );
        });
        return;
    }

    let needle = state.sidebar_search.to_ascii_lowercase();

    for entry in history.iter().rev() {
        if !needle.is_empty()
            && !entry.query.to_ascii_lowercase().contains(&needle)
        {
            continue;
        }

        let preview = entry
            .query
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(80)
            .collect::<String>();
        let when = entry.timestamp.format("%H:%M:%S").to_string();

        let row_h = 38.0;
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_h),
            Sense::click(),
        );
        if resp.hovered() {
            ui.painter().rect_filled(rect, 0.0, t.bg_elev);
        }
        ui.painter().text(
            rect.min + egui::vec2(theme::SPACE_MD, 6.0),
            egui::Align2::LEFT_TOP,
            preview,
            egui::FontId::monospace(11.0),
            t.text_primary,
        );
        ui.painter().text(
            rect.min + egui::vec2(theme::SPACE_MD, 22.0),
            egui::Align2::LEFT_TOP,
            format!(
                "{}  \u{00B7} {}ms  \u{00B7} {} rows",
                when,
                entry.duration_ms,
                entry.row_count
            ),
            egui::FontId::proportional(10.0),
            t.text_muted,
        );

        if resp.clicked() {
            if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
                tab.content = entry.query.clone();
            }
        }
    }
}

// ============================================================================
// Status bar (24px)
// ============================================================================

fn render_status_bar(ctx: &egui::Context, t: Tokens, state: &AppState) {
    egui::TopBottomPanel::bottom("ferrum_status")
        .exact_height(24.0)
        .frame(
            egui::Frame::new()
                .fill(t.bg_app)
                .inner_margin(Margin::symmetric(theme::SPACE_LG_I, 0))
                .stroke(Stroke::new(1.0, t.border_subtle)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(24.0);
            ui.horizontal_centered(|ui| {
                if let Some(id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&id) {
                        let (dot, label) = match &conn.status {
                            ConnectionStatus::Connected { server_version } => {
                                (t.success, format!("PG {server_version}"))
                            }
                            ConnectionStatus::Connecting => {
                                (t.warn, "Connecting\u{2026}".to_string())
                            }
                            ConnectionStatus::Disconnected => {
                                (t.danger, "Disconnected".to_string())
                            }
                        };
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 3.5, dot);
                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(t.text_primary)
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(format!("  \u{00B7} {label}"))
                                .color(t.text_muted)
                                .size(11.0),
                        );
                    }
                } else {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(rect.center(), 3.5, t.text_disabled);
                    ui.label(
                        RichText::new("No connection")
                            .color(t.text_muted)
                            .size(11.0),
                    );
                }

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        let now = chrono::Local::now().format("%H:%M:%S").to_string();
                        ui.label(
                            RichText::new(now).color(t.text_disabled).size(11.0),
                        );

                        if let Some(ref result) = state.current_result {
                            let row_label =
                                if result.rows.len() == 1 { "row" } else { "rows" };
                            ui.label(
                                RichText::new(format!(
                                    "  \u{00B7} {}ms  \u{00B7} {} {row_label} ",
                                    result.execution_time_ms,
                                    result.rows.len(),
                                ))
                                .color(t.text_muted)
                                .size(11.0),
                            );
                        }

                        if state.query_running {
                            ui.spinner();
                            ui.label(
                                RichText::new("Running\u{2026}")
                                    .color(t.warn)
                                    .size(11.0),
                            );
                        }
                    },
                );
            });
        });
}

// ============================================================================
// Result panel (bottom dock)
// ============================================================================

fn render_result_panel(ctx: &egui::Context, t: Tokens, state: &mut AppState) {
    egui::TopBottomPanel::bottom("ferrum_results")
        .default_height(280.0)
        .min_height(80.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(t.bg_surface)
                .inner_margin(Margin::ZERO)
                .stroke(Stroke::new(1.0, t.border_subtle)),
        )
        .show(ctx, |ui| {
            grid::render_result_panel(ui, t, state);
        });
}

// ============================================================================
// Workspace (central)
// ============================================================================

fn render_workspace(
    ctx: &egui::Context,
    t: Tokens,
    state: &mut AppState,
    bridge: &DbBridge,
) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(t.bg_app)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            editor::render_workspace(ui, t, state, bridge);
        });
}

// ============================================================================
// Helper exposed to other modules
// ============================================================================

pub fn panel_frame(t: Tokens, fill: Color32) -> egui::Frame {
    let _ = t;
    egui::Frame::new()
        .fill(fill)
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, t.border_subtle))
}
