use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::i18n::t;
use crate::state::{AppState, MainView};
use crate::ui::{icons_svg, theme};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaletteAction {
    RunQuery,
    NewQueryTab,
    OpenQueryHistory,
    GoToTable,
    OpenERDiagram,
    OpenBI,
    OpenVault,
    OpenSettings,
    ToggleFilterRow,
    ExportCsv,
    RefreshSchema,
}

struct CmdSection {
    /// i18n key for the section label
    label_key: &'static str,
    items: &'static [CmdItem],
}

struct CmdItem {
    icon: &'static str,
    /// i18n key for the item label
    label_key: &'static str,
    /// i18n key for the hint (empty string = no hint)
    hint_key: &'static str,
    kbd: &'static str,
    action: PaletteAction,
}

static SECTIONS: &[CmdSection] = &[
    CmdSection {
        label_key: "cmd_sec_workspace",
        items: &[
            CmdItem { icon: icons_svg::PLAY_SM, label_key: "cmd_run_query", hint_key: "cmd_hint_run_query", kbd: "\u{2318}\u{23CE}", action: PaletteAction::RunQuery },
            CmdItem { icon: icons_svg::PLAY_SM, label_key: "cmd_new_tab", hint_key: "cmd_hint_new_tab", kbd: "\u{2318}T", action: PaletteAction::NewQueryTab },
            CmdItem { icon: icons_svg::HISTORY, label_key: "cmd_open_history", hint_key: "cmd_hint_open_history", kbd: "\u{2318}\u{21E7}H", action: PaletteAction::OpenQueryHistory },
        ],
    },
    CmdSection {
        label_key: "cmd_sec_navigate",
        items: &[
            CmdItem { icon: icons_svg::TABLE_SM, label_key: "cmd_go_table", hint_key: "cmd_hint_go_table", kbd: "\u{2318}O", action: PaletteAction::GoToTable },
            CmdItem { icon: icons_svg::DIAGRAM, label_key: "cmd_open_er", hint_key: "cmd_hint_open_er", kbd: "\u{2318}D", action: PaletteAction::OpenERDiagram },
            CmdItem { icon: icons_svg::CHART, label_key: "cmd_open_bi", hint_key: "cmd_hint_open_bi", kbd: "\u{2318}B", action: PaletteAction::OpenBI },
            CmdItem { icon: icons_svg::VAULT, label_key: "cmd_open_vault", hint_key: "cmd_hint_open_vault", kbd: "\u{2318}\u{21E7}V", action: PaletteAction::OpenVault },
            CmdItem { icon: icons_svg::COG, label_key: "cmd_open_settings", hint_key: "cmd_hint_open_settings", kbd: "\u{2318},", action: PaletteAction::OpenSettings },
        ],
    },
    CmdSection {
        label_key: "cmd_sec_data",
        items: &[
            CmdItem { icon: icons_svg::FILTER, label_key: "cmd_toggle_filter", hint_key: "cmd_hint_toggle_filter", kbd: "\u{2318}F", action: PaletteAction::ToggleFilterRow },
            CmdItem { icon: icons_svg::DOWNLOAD, label_key: "cmd_export_csv", hint_key: "cmd_hint_export_csv", kbd: "\u{2318}E", action: PaletteAction::ExportCsv },
        ],
    },
    CmdSection {
        label_key: "cmd_sec_database",
        items: &[
            CmdItem { icon: icons_svg::COG, label_key: "cmd_refresh_schema", hint_key: "cmd_hint_refresh_schema", kbd: "F5", action: PaletteAction::RefreshSchema },
        ],
    },
];

/// Resolved command item with translated strings for display and search.
struct ResolvedCmd {
    section: String,
    label: String,
    hint: String,
    item: &'static CmdItem,
}

fn filtered_items(search: &str) -> Vec<ResolvedCmd> {
    let query = search.to_lowercase();
    let mut results = Vec::new();
    for sec in SECTIONS {
        let section_label = t(sec.label_key);
        for item in sec.items {
            let label = t(item.label_key);
            let hint = t(item.hint_key);
            if query.is_empty() || label.to_lowercase().contains(&query) || hint.to_lowercase().contains(&query) {
                results.push(ResolvedCmd {
                    section: section_label.clone(),
                    label,
                    hint,
                    item,
                });
            }
        }
    }
    results
}

pub fn render_command_palette(ctx: &egui::Context, state: &mut AppState) -> Option<PaletteAction> {
    if !state.show_command_palette {
        return None;
    }

    let mut close = false;
    let mut triggered_action: Option<PaletteAction> = None;

    let screen = ctx.screen_rect();
    let overlay_layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("cmd_overlay"));
    let painter = ctx.layer_painter(overlay_layer);
    painter.rect_filled(screen, 0.0, Color32::from_black_alpha(120));

    let overlay_resp = egui::Area::new(egui::Id::new("cmd_overlay_area"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (_, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            resp
        });
    if overlay_resp.inner.clicked() {
        close = true;
    }

    egui::Area::new(egui::Id::new("command_palette"))
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::bg_elevated())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::ZERO)
                .show(ui, |ui| {
                    ui.set_width(520.0);

                    egui::Frame::new()
                        .inner_margin(Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                crate::ui::icon_img_tinted(
                                    ui, icons_svg::SEARCH, "cmd_search_ic", 14.0, theme::text_muted(),
                                );
                                ui.add_space(6.0);
                                let te = ui.add(
                                    egui::TextEdit::singleline(&mut state.command_palette_search)
                                        .hint_text(t("cmd_search_placeholder"))
                                        .frame(false)
                                        .background_color(Color32::TRANSPARENT)
                                        .text_color(theme::text_primary())
                                        .font(egui::FontId::proportional(14.0))
                                        .min_size(egui::vec2(400.0, 22.0))
                                        .margin(Margin::ZERO),
                                );
                                if !te.has_focus() {
                                    te.request_focus();
                                }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new("\u{2318}K")
                                            .monospace()
                                            .size(10.0)
                                            .color(theme::text_disabled()),
                                    );
                                });
                            });
                        });

                    ui.painter().hline(
                        ui.available_rect_before_wrap().x_range(),
                        ui.available_rect_before_wrap().top(),
                        Stroke::new(1.0, theme::border_subtle()),
                    );
                    ui.add_space(1.0);

                    let items = filtered_items(&state.command_palette_search);
                    let max_sel = items.len().saturating_sub(1);
                    state.command_palette_selected = state.command_palette_selected.min(max_sel);

                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            let mut visual_idx = 0usize;
                            let mut prev_section = String::new();
                            for resolved in &items {
                                if resolved.section != prev_section {
                                    ui.add_space(2.0);
                                    egui::Frame::new()
                                        .inner_margin(Margin::symmetric(16, 4))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(resolved.section.to_uppercase())
                                                    .size(10.0)
                                                    .color(theme::text_muted())
                                                    .monospace(),
                                            );
                                        });
                                    prev_section = resolved.section.clone();
                                }

                                let selected = visual_idx == state.command_palette_selected;
                                let bg = if selected {
                                    theme::with_alpha(theme::ACCENT_EMERALD, 16)
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let (row_rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 28.0),
                                    egui::Sense::click(),
                                );

                                let hover_bg = if resp.hovered() && !selected {
                                    theme::bg_light()
                                } else {
                                    bg
                                };
                                ui.painter().rect_filled(row_rect, 0.0, hover_bg);

                                let icon_rect = egui::Rect::from_min_size(
                                    egui::pos2(row_rect.left() + 16.0, row_rect.center().y - 7.0),
                                    egui::vec2(14.0, 14.0),
                                );
                                let icon_img = crate::ui::icon_image_tinted(
                                    ui, resolved.item.icon, &format!("cmd_{}", resolved.item.label_key), 14.0, theme::text_muted(),
                                );
                                icon_img.paint_at(ui, icon_rect);

                                ui.painter().text(
                                    egui::pos2(row_rect.left() + 40.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &resolved.label,
                                    egui::FontId::proportional(12.5),
                                    theme::text_primary(),
                                );

                                if !resolved.hint.is_empty() {
                                    ui.painter().text(
                                        egui::pos2(row_rect.right() - 80.0, row_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        &resolved.hint,
                                        egui::FontId::proportional(11.0),
                                        theme::text_muted(),
                                    );
                                }

                                if !resolved.item.kbd.is_empty() {
                                    ui.painter().text(
                                        egui::pos2(row_rect.right() - 16.0, row_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        resolved.item.kbd,
                                        egui::FontId::monospace(10.0),
                                        theme::text_disabled(),
                                    );
                                }

                                if resp.clicked() {
                                    triggered_action = Some(resolved.item.action);
                                    close = true;
                                }

                                visual_idx += 1;
                            }

                            if items.is_empty() {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        RichText::new(t("cmd_no_match"))
                                            .color(theme::text_muted())
                                            .size(12.0),
                                    );
                                });
                            }
                            ui.add_space(4.0);
                        });
                });
        });

    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            close = true;
        }
        if i.key_pressed(egui::Key::ArrowDown) {
            let max = filtered_items(&state.command_palette_search).len().saturating_sub(1);
            state.command_palette_selected = (state.command_palette_selected + 1).min(max);
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            state.command_palette_selected = state.command_palette_selected.saturating_sub(1);
        }
        if i.key_pressed(egui::Key::Enter) && triggered_action.is_none() {
            let items = filtered_items(&state.command_palette_search);
            if let Some(resolved) = items.get(state.command_palette_selected) {
                triggered_action = Some(resolved.item.action);
            }
            close = true;
        }
    });

    if close {
        state.show_command_palette = false;
        state.command_palette_search.clear();
        state.command_palette_selected = 0;
    }

    triggered_action
}

pub fn execute_palette_action(action: PaletteAction, state: &mut AppState, bridge: &crate::db::bridge::DbBridge) {
    match action {
        PaletteAction::RunQuery => {
            if let Some(conn_id) = state.active_connection {
                if let Some(tab) = state.editor_tabs.get(state.active_tab) {
                    let sql = tab.content.clone();
                    if !sql.trim().is_empty() {
                        state.query_running = true;
                        bridge.send(crate::db::bridge::DbCommand::ExecuteQuery {
                            conn_id,
                            sql,
                            row_limit: Some(state.default_row_limit),
                        });
                    }
                }
            }
        }
        PaletteAction::NewQueryTab => {
            state.open_workspace_main_view(MainView::Query);
        }
        PaletteAction::OpenQueryHistory => {
            state.open_workspace_main_view(MainView::Query);
        }
        PaletteAction::GoToTable => {
            state.open_workspace_main_view(MainView::Data);
        }
        PaletteAction::OpenERDiagram => {
            state.open_workspace_main_view(MainView::Model);
        }
        PaletteAction::OpenBI => {
            state.open_workspace_main_view(MainView::BI);
        }
        PaletteAction::OpenVault => {
            state.show_connection_dialog = true;
            state.connection_dialog = Default::default();
        }
        PaletteAction::OpenSettings => {
            state.show_settings_dialog = true;
        }
        PaletteAction::ToggleFilterRow => {
            state.open_workspace_main_view(MainView::Data);
        }
        PaletteAction::ExportCsv => {
            state.open_workspace_main_view(MainView::Data);
        }
        PaletteAction::RefreshSchema => {
            if let Some(conn_id) = state.active_connection {
                bridge.send(crate::db::bridge::DbCommand::ListSchemas {
                    conn_id,
                });
            }
        }
    }
}
