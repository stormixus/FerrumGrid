use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, StrokeKind};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{main_view_title, AppState, ConnectionStatus, MainView};
use crate::ui::{editor, grid, icons_svg, theme, titlebar, tree_browser};

pub fn render_panels(
    ctx: &egui::Context,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut crate::storage::settings::AppSettings,
) {
    // Only apply the appearance if no draft preview is active to prevent overwriting the interactive preview.
    // Also, only run it when the system theme or accent color has actually changed to save frame overhead.
    if state.settings_draft.is_none() {
        let current_accent = theme::accent_color_name();
        let current_dark = theme::is_dark();
        let system_dark = !matches!(ctx.system_theme(), Some(egui::Theme::Light));
        let target_dark = match settings.appearance.as_str() {
            "light" => false,
            "dark" => true,
            _ => system_dark,
        };
        if current_accent != settings.accent_color || current_dark != target_dark {
            settings.dark_mode = theme::apply_appearance(ctx, &settings.appearance, &settings.accent_color);
        }
    }
    titlebar::render_titlebar(ctx, state, settings);
    render_main_toolbar(ctx, state);
    render_status_bar(ctx, state);

    // ⌘K shortcut to toggle command palette
    if ctx.input(|i| (i.modifiers.mac_cmd || i.modifiers.ctrl) && i.key_pressed(egui::Key::K)) {
        state.show_command_palette = !state.show_command_palette;
        if !state.show_command_palette {
            state.command_palette_search.clear();
            state.command_palette_selected = 0;
        }
    }
    if let Some(action) = crate::ui::command_palette::render_command_palette(ctx, state) {
        crate::ui::command_palette::execute_palette_action(action, state, bridge);
    }

    // Diagnostics panel (above status bar)
    if state.diagnostics_panel.visible || state.diagnostics_panel.unsafe_ctid_active {
        let panel_height = if state.diagnostics_panel.visible {
            160.0
        } else {
            28.0
        };
        egui::TopBottomPanel::bottom("diagnostics_panel")
            .min_height(28.0)
            .default_height(panel_height)
            .max_height(300.0)
            .resizable(state.diagnostics_panel.visible)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_dark())
                    .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_SM_I))
                    .stroke(Stroke::new(1.0, theme::border_subtle())),
            )
            .show(ctx, |ui| {
                state.diagnostics_panel.render(ui);
            });
    }

    // Left panel: database tree
    if state.show_tree_panel {
        egui::SidePanel::left("tree_panel")
            .default_width(286.0)
            .min_width(220.0)
            .max_width(440.0)
            .resizable(true)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_shell())
                    .inner_margin(Margin::ZERO)
                    .stroke(Stroke::new(1.0, theme::border_subtle())),
            )
            .show(ctx, |ui| {
                render_tree_panel_header(ui, state);
                egui::ScrollArea::both()
                    .id_salt("tree_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(theme::SPACE_SM);
                        tree_browser::render_tree(ui, state, bridge);
                        ui.add_space(theme::SPACE_LG);
                    });
            });
    }

    // Bottom panel: result grid — animate slide in/out using exact_height.
    let want_result_visible = state.show_result_panel
        && state.active_main_view != MainView::Data
        && state.current_result.is_some();
    let result_t = ctx.animate_bool_with_time(
        egui::Id::new("result_panel_slide"),
        want_result_visible,
        0.22,
    );
    if result_t > 0.001 {
        let target_height = 282.0_f32;
        if (result_t - 1.0).abs() < 0.001 {
            egui::TopBottomPanel::bottom("result_panel")
                .default_height(target_height)
                .min_height(108.0)
                .resizable(true)
                .show_separator_line(false)
                .frame(
                    egui::Frame::new()
                        .fill(theme::bg_dark())
                        .inner_margin(Margin::ZERO)
                        .stroke(Stroke::new(1.0, theme::border_subtle())),
                )
                .show(ctx, |ui| {
                    grid::render_grid(ui, state, bridge);
                });
        } else {
            let height = (target_height * result_t).max(2.0);
            egui::TopBottomPanel::bottom("result_panel_anim")
                .exact_height(height)
                .resizable(false)
                .show_separator_line(false)
                .frame(
                    egui::Frame::new()
                        .fill(theme::bg_dark())
                        .inner_margin(Margin::ZERO)
                        .stroke(Stroke::new(1.0, theme::border_subtle())),
                )
                .show(ctx, |ui| {
                    ui.set_clip_rect(ui.max_rect());
                    grid::render_grid(ui, state, bridge);
                });
            ctx.request_repaint();
        }
    }

    // Right panel: Info / Properties
    if state.show_info_panel {
        egui::SidePanel::right("info_panel")
            .default_width(280.0)
            .min_width(180.0)
            .resizable(true)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_shell())
                    .inner_margin(Margin::ZERO)
                    .stroke(Stroke::new(1.0, theme::border_subtle())),
            )
            .show(ctx, |ui| {
                grid::render_info_panel(ui, state, bridge);
            });
    }

    // Center: SQL editor or Objects view
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::bg_darkest())
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), CornerRadius::ZERO, theme::bg_darkest());
            render_workspace_tabs(ui, state, bridge);

            match state.active_main_view {
                crate::state::MainView::Table
                | crate::state::MainView::View
                | crate::state::MainView::MaterializedView
                | crate::state::MainView::Function
                | crate::state::MainView::User
                | crate::state::MainView::Backup
                | crate::state::MainView::Automation
                | crate::state::MainView::Model
                | crate::state::MainView::BI => {
                    crate::ui::objects::render_objects_view(ui, state, bridge, settings);
                }
                crate::state::MainView::Connection | crate::state::MainView::Query => {
                    editor::render_editor(ui, state, bridge, settings);
                }
                crate::state::MainView::Data => {
                    grid::render_grid(ui, state, bridge);
                }
            }
        });

    stabilize_info_panel_resize_cursor(ctx, state);
}

fn stabilize_info_panel_resize_cursor(ctx: &egui::Context, state: &AppState) {
    if !state.show_info_panel {
        return;
    }

    let panel_id = egui::Id::new("info_panel");
    let resize_id = panel_id.with("__resize");
    let resize_active = ctx
        .read_response(resize_id)
        .is_some_and(|response| response.hovered() || response.dragged());
    let pointer_near_splitter = egui::containers::panel::PanelState::load(ctx, panel_id)
        .zip(ctx.input(|input| input.pointer.hover_pos()))
        .is_some_and(|(panel, pos)| {
            let grab_radius = ctx.style().interaction.resize_grab_radius_side.max(10.0);
            let splitter = panel.rect.left();
            panel.rect.y_range().contains(pos.y) && (pos.x - splitter).abs() <= grab_radius
        });

    if resize_active || pointer_near_splitter {
        ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
}

fn render_workspace_tabs(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if state.workspace_tabs.is_empty() {
        state.open_workspace_main_view(state.active_main_view);
    }

    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(theme::SPACE_LG_I, theme::SPACE_XS_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    let tabs = state.workspace_tabs.clone();
    let active = state.active_workspace_tab;
    let mut activate: Option<usize> = None;
    let mut close: Option<usize> = None;

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(36.0);
        egui::ScrollArea::horizontal()
            .id_salt("workspace_tabs_scroll")
            .max_width(ui.available_width())
            .show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_XS;

            for (index, tab) in tabs.iter().enumerate() {
                let selected = index == active;
                let response = render_workspace_tab(ui, tab.view, &tab.title, selected);
                if response.clicked() {
                    activate = Some(index);
                }

                let tab_paint_cy = response.rect.top() + 4.0 + 16.0;
                let close_rect = egui::Rect::from_center_size(
                    egui::pos2(response.rect.right() - 13.0, tab_paint_cy),
                    egui::vec2(16.0, 16.0),
                );
                let close_resp =
                    ui.interact(close_rect, response.id.with("close"), egui::Sense::click());
                let close_color = if close_resp.hovered() {
                    theme::ACCENT_RED
                } else if selected {
                    theme::text_muted()
                } else {
                    theme::text_disabled()
                };
                ui.painter().text(
                    close_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "\u{00d7}",
                    egui::FontId::proportional(13.0),
                    close_color,
                );
                if close_resp.clicked() {
                    close = Some(index);
                }

                response.context_menu(|ui| {
                    if ui.button(t("workspace_close_tab")).clicked() {
                        close = Some(index);
                        ui.close_menu();
                    }
                });
            }

            let new_query = render_workspace_add_tab_button(ui);
            show_dark_hover_tooltip(
                ui,
                new_query.id.with("tooltip"),
                &new_query,
                &t("workspace_new_query"),
            );
            if new_query.clicked() {
                let n = state.editor_tabs.len() + 1;
                state
                    .editor_tabs
                    .push(crate::types::EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
                state.open_workspace_main_view(MainView::Query);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(main_view_title(state.active_main_view))
                        .color(theme::text_muted())
                        .size(11.0),
                );
            });
        });
        }); // close ScrollArea
    });

    if let Some(index) = close {
        state.close_workspace_tab(index);
        grid::restore_active_data_tab(state, bridge);
    } else if let Some(index) = activate {
        let active_before = state.active_workspace_tab;
        state.activate_workspace_tab(index);
        if active_before != state.active_workspace_tab {
            grid::restore_active_data_tab(state, bridge);
        }
    }
}

fn render_workspace_tab(
    ui: &mut egui::Ui,
    view: MainView,
    title: &str,
    selected: bool,
) -> egui::Response {
    let _color = workspace_tab_color(view);
    let view_suffix = match view {
        MainView::Data => " \u{00B7} Data",
        MainView::Table => " \u{00B7} Object",
        MainView::View => " \u{00B7} Object",
        MainView::MaterializedView => " \u{00B7} Object",
        MainView::Function => " \u{00B7} Object",
        MainView::Model => " \u{00B7} ER",
        MainView::BI => " \u{00B7} Sales",
        _ => "",
    };
    let full_title = format!("{title}{view_suffix}");
    let label = truncate_tab_label(&full_title, 30);
    let width = tab_width(ui, &label).clamp(96.0, 240.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());

    let bg = if selected {
        theme::bg_darkest()
    } else if response.hovered() {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    let border = if selected {
        theme::border_subtle()
    } else {
        Color32::TRANSPARENT
    };
    let paint_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + 4.0),
        egui::vec2(rect.width(), 32.0),
    );
    let tab_rounding = CornerRadius {
        nw: theme::RADIUS_LG,
        ne: theme::RADIUS_LG,
        sw: 0,
        se: 0,
    };
    ui.painter().rect_filled(paint_rect, tab_rounding, bg);
    if selected {
        ui.painter().rect_stroke(
            paint_rect,
            tab_rounding,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                paint_rect.min,
                egui::vec2(paint_rect.width(), 2.0),
            ),
            CornerRadius {
                nw: theme::RADIUS_LG,
                ne: theme::RADIUS_LG,
                sw: 0,
                se: 0,
            },
            theme::accent_color(),
        );
    }

    let tab_cy = paint_rect.center().y;
    let text_color = if selected {
        theme::text_primary()
    } else {
        theme::text_secondary()
    };

    // View-type icon — painted directly, no layout allocation
    let icon_svg = workspace_tab_icon_svg(view);
    let icon_name = format!("wstab_{}_{}", view as u8, if selected { "s" } else { "n" });
    let icon_image = crate::ui::icon_image_tinted(ui, icon_svg, &icon_name, 12.0, text_color);
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(paint_rect.left() + 14.0, tab_cy),
        egui::vec2(12.0, 12.0),
    );
    icon_image.paint_at(ui, icon_rect);

    // Text — clipped before close button
    let text_clip = egui::Rect::from_min_max(
        egui::pos2(paint_rect.left() + 26.0, paint_rect.top()),
        egui::pos2(paint_rect.right() - 22.0, paint_rect.bottom()),
    );
    ui.painter().with_clip_rect(text_clip).text(
        egui::pos2(paint_rect.left() + 26.0, tab_cy),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, title);
    response
}

fn render_workspace_add_tab_button(ui: &mut egui::Ui) -> egui::Response {
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

fn tab_width(ui: &egui::Ui, label: &str) -> f32 {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.0),
        theme::text_primary(),
    );
    galley.rect.width() + 62.0 // icon(14) + gap(12) + close(16) + padding(20)
}

fn truncate_tab_label(label: &str, max_chars: usize) -> String {
    if label.chars().count() <= max_chars {
        label.to_string()
    } else {
        let mut truncated = label
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }
}

fn workspace_tab_icon_svg(view: MainView) -> &'static str {
    match view {
        MainView::Connection => icons_svg::VAULT,
        MainView::Table => icons_svg::TABLE_SM,
        MainView::View => icons_svg::TABLE_SM,
        MainView::MaterializedView => icons_svg::TABLE_SM,
        MainView::Function => icons_svg::COG,
        MainView::User => icons_svg::COG,
        MainView::Query => icons_svg::PLAY_SM,
        MainView::Data => icons_svg::TABLE_SM,
        MainView::Backup => icons_svg::DOWNLOAD,
        MainView::Automation => icons_svg::COG,
        MainView::Model => icons_svg::DIAGRAM,
        MainView::BI => icons_svg::CHART,
    }
}

fn workspace_tab_color(view: MainView) -> Color32 {
    match view {
        MainView::Connection => theme::accent_color(),
        MainView::Table => theme::accent_color(),
        MainView::View => theme::ACCENT_BLUE,
        MainView::MaterializedView => theme::accent_color(),
        MainView::Function => theme::ACCENT_YELLOW,
        MainView::User => theme::accent_color_light(),
        MainView::Query => theme::ACCENT_BLUE,
        MainView::Data => theme::accent_color(),
        MainView::Backup => theme::text_muted(),
        MainView::Automation => theme::accent_color_light(),
        MainView::Model => theme::accent_color(),
        MainView::BI => theme::ACCENT_RED,
    }
}

// ---------------------------------------------------------------------------
// Main Toolbar
// ---------------------------------------------------------------------------

fn render_main_toolbar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("main_toolbar")
        .exact_height(38.0)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_medium())
                .inner_margin(Margin::symmetric(theme::SPACE_LG_I, 0))
                .stroke(Stroke::NONE),
        )
        .show(ctx, |ui| {
            let bottom_line = ui.max_rect().x_range();
            ui.painter().hline(
                bottom_line,
                ui.max_rect().bottom(),
                Stroke::new(1.0, theme::border_subtle()),
            );

            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_XS;

                // Left group: panel toggles
                render_toolbar_pane_toggle(ui, &mut state.show_tree_panel, PaneToggle::Navigator);
                render_toolbar_pane_toggle(ui, &mut state.show_info_panel, PaneToggle::Info);

                // Separator
                ui.add_space(theme::SPACE_SM);
                let sep_rect = ui.allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover()).0;
                ui.painter().rect_filled(sep_rect, CornerRadius::ZERO, theme::border_subtle());
                ui.add_space(theme::SPACE_SM);

                // Center group: 6 main view tabs with icons
                ui.spacing_mut().item_spacing.x = theme::SPACE_XS;
                for view in MainView::TOOLBAR_TABS {
                    let selected = state.active_main_view == view;
                    let label = main_view_title(view);
                    let icon_svg = toolbar_tab_icon(view);
                    let response = render_toolbar_tab_button_with_icon(ui, label, icon_svg, selected);
                    if response.clicked() {
                        state.open_workspace_main_view(view);
                    }
                }

                // Right group: Search, Vault, Settings (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = theme::SPACE_XS;

                    let settings_btn = ui.add(
                        egui::Button::image(
                            crate::ui::icon_image_tinted(ui, icons_svg::COG, "tb_cog2", 14.0, theme::text_muted()),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                        .min_size(egui::vec2(26.0, 26.0)),
                    );
                    if settings_btn.clicked() {
                        state.show_settings_dialog = true;
                    }

                    // DBA 세션 모니터 토글.
                    let sessions_btn = ui
                        .add(
                            egui::Button::image(crate::ui::icon_image_tinted(
                                ui,
                                icons_svg::DATABASE,
                                "tb_sessions",
                                14.0,
                                theme::text_muted(),
                            ))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                            .min_size(egui::vec2(26.0, 26.0)),
                        )
                        .on_hover_text(t("sessions_window_title"));
                    if sessions_btn.clicked() {
                        state.show_sessions_window = !state.show_sessions_window;
                        if state.show_sessions_window {
                            state.sessions_needs_fetch = true;
                        }
                    }

                    // 카탈로그(시퀀스/enum/익스텐션) 브라우저 토글.
                    let catalog_btn = ui
                        .add(
                            egui::Button::image(crate::ui::icon_image_tinted(
                                ui,
                                icons_svg::SCHEMA,
                                "tb_catalog",
                                14.0,
                                theme::text_muted(),
                            ))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                            .min_size(egui::vec2(26.0, 26.0)),
                        )
                        .on_hover_text(t("catalog_window_title"));
                    if catalog_btn.clicked() {
                        state.show_catalog_window = !state.show_catalog_window;
                        if state.show_catalog_window {
                            state.catalog_needs_fetch = true;
                        }
                    }

                    // 권한(GRANT/REVOKE) 브라우저 토글.
                    let priv_btn = ui
                        .add(
                            egui::Button::image(crate::ui::icon_image_tinted(
                                ui,
                                icons_svg::VAULT,
                                "tb_privileges",
                                14.0,
                                theme::text_muted(),
                            ))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                            .min_size(egui::vec2(26.0, 26.0)),
                        )
                        .on_hover_text(t("privileges_window_title"));
                    if priv_btn.clicked() {
                        state.show_privileges_window = !state.show_privileges_window;
                        if state.show_privileges_window {
                            state.privileges_needs_fetch = true;
                        }
                    }

                    let vault_btn = ui.add(
                        egui::Button::image_and_text(
                            crate::ui::icon_image_tinted(ui, icons_svg::VAULT, "tb_vault2", 13.0, theme::text_muted()),
                            egui::RichText::new(t("panel_vault")).color(theme::text_muted()).size(12.0),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                        .min_size(egui::vec2(0.0, 26.0)),
                    );
                    if vault_btn.clicked() {
                        state.show_connection_dialog = true;
                        state.connection_dialog = Default::default();
                    }

                    // In RTL: kbd badge renders first (rightmost), then search button
                    render_kbd_badge(ui, "\u{2318}K");
                    let search_btn = ui.add(
                        egui::Button::image_and_text(
                            crate::ui::icon_image_tinted(ui, icons_svg::SEARCH, "tb_search3", 13.0, theme::text_muted()),
                            egui::RichText::new(t("panel_search")).color(theme::text_muted()).size(12.0),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
                        .min_size(egui::vec2(0.0, 26.0)),
                    );
                    if search_btn.clicked() {
                        state.show_command_palette = true;
                    }
                });
            });
        });
}

fn render_kbd_badge(ui: &mut egui::Ui, text: &str) {
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        egui::FontId::monospace(10.5),
        theme::text_secondary(),
    );
    let size = egui::vec2(galley.rect.width() + 8.0, 18.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(3),
        theme::bg_light(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, theme::border_default()),
        StrokeKind::Inside,
    );
    ui.painter().galley(
        egui::pos2(
            rect.center().x - galley.rect.width() / 2.0,
            rect.center().y - galley.rect.height() / 2.0,
        ),
        galley,
        theme::text_secondary(),
    );
}

fn render_toolbar_pane_toggle(
    ui: &mut egui::Ui,
    active: &mut bool,
    pane: PaneToggle,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::click());
    let hovered = response.hovered();
    let color = if *active {
        theme::text_primary()
    } else {
        theme::text_muted()
    };
    let bg = if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), bg);
    paint_pane_icon(ui.painter(), rect.shrink(5.0), pane, color, *active);

    if response.clicked() {
        *active = !*active;
    }
    response
}

fn toolbar_tab_icon(view: MainView) -> &'static str {
    match view {
        MainView::Query => icons_svg::PLAY_SM,
        MainView::Data => icons_svg::TABLE_SM,
        MainView::Model => icons_svg::DIAGRAM,
        MainView::BI => icons_svg::CHART,
        MainView::Backup => icons_svg::DOWNLOAD,
        MainView::Automation => icons_svg::COG,
        _ => icons_svg::TABLE_SM,
    }
}

fn render_toolbar_tab_button_with_icon(
    ui: &mut egui::Ui,
    label: &str,
    icon_svg: &str,
    selected: bool,
) -> egui::Response {
    let text_color = if selected {
        theme::text_primary()
    } else {
        theme::text_secondary()
    };
    let fill = if selected {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    let border = if selected {
        Stroke::new(1.0, theme::border_default())
    } else {
        Stroke::NONE
    };
    let icon_name = format!("tb_{}", label.to_lowercase());
    let icon = crate::ui::icon_image_tinted(ui, icon_svg, &icon_name, 14.0, text_color);

    let btn = egui::Button::image_and_text(
        icon,
        egui::RichText::new(label).color(text_color).size(12.0),
    )
    .fill(fill)
    .stroke(border)
    .corner_radius(CornerRadius::same(theme::RADIUS_MD))
    .min_size(egui::vec2(0.0, 26.0));

    ui.add(btn)
}


#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PaneToggle {
    Navigator,
    Results,
    Info,
}

fn paint_pane_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    pane: PaneToggle,
    color: Color32,
    active: bool,
) {
    let stroke = Stroke::new(1.25, color);
    let fill = if active {
        theme::with_alpha(color, 48)
    } else {
        Color32::TRANSPARENT
    };
    let frame = rect.expand(1.0);

    painter.rect_stroke(
        frame,
        CornerRadius::same(theme::RADIUS_SM),
        stroke,
        StrokeKind::Inside,
    );

    match pane {
        PaneToggle::Navigator => {
            let pane_rect =
                egui::Rect::from_min_max(frame.min, egui::pos2(frame.left() + 4.4, frame.bottom()));
            painter.rect_filled(pane_rect.shrink(1.0), CornerRadius::same(1), fill);
            painter.line_segment(
                [
                    egui::pos2(pane_rect.right(), frame.top()),
                    egui::pos2(pane_rect.right(), frame.bottom()),
                ],
                stroke,
            );
        }
        PaneToggle::Results => {
            let pane_rect =
                egui::Rect::from_min_max(egui::pos2(frame.left(), frame.bottom() - 4.4), frame.max);
            painter.rect_filled(pane_rect.shrink(1.0), CornerRadius::same(1), fill);
            painter.line_segment(
                [
                    egui::pos2(frame.left(), pane_rect.top()),
                    egui::pos2(frame.right(), pane_rect.top()),
                ],
                stroke,
            );
        }
        PaneToggle::Info => {
            let pane_rect =
                egui::Rect::from_min_max(egui::pos2(frame.right() - 4.4, frame.top()), frame.max);
            painter.rect_filled(pane_rect.shrink(1.0), CornerRadius::same(1), fill);
            painter.line_segment(
                [
                    egui::pos2(pane_rect.left(), frame.top()),
                    egui::pos2(pane_rect.left(), frame.bottom()),
                ],
                stroke,
            );
        }
    }
}

fn show_dark_hover_tooltip(
    ui: &egui::Ui,
    tooltip_id: egui::Id,
    response: &egui::Response,
    text: &str,
) {
    if !response.hovered() {
        return;
    }

    let pointer = ui
        .ctx()
        .pointer_hover_pos()
        .unwrap_or_else(|| response.rect.left_bottom());
    let max_width = 360.0;
    let pos = smart_tooltip_pos(ui.ctx(), pointer, estimate_tooltip_size(text, max_width));
    egui::Area::new(tooltip_id)
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_strong()))
                .corner_radius(CornerRadius::same(theme::RADIUS_LG))
                .inner_margin(Margin::same(theme::SPACE_MD_I))
                .show(ui, |ui| {
                    ui.set_max_width(max_width);
                    ui.add(
                        egui::Label::new(
                            RichText::new(text)
                                .color(theme::text_secondary())
                                .monospace()
                                .size(11.0),
                        )
                        .wrap(),
                    );
                });
        });
}

fn smart_tooltip_pos(
    ctx: &egui::Context,
    anchor: egui::Pos2,
    estimated_size: egui::Vec2,
) -> egui::Pos2 {
    let bounds = ctx.screen_rect().shrink(8.0);
    let gap = 12.0;
    let right_x = anchor.x + gap;
    let left_x = anchor.x - gap - estimated_size.x;
    let bottom_y = anchor.y + gap;
    let top_y = anchor.y - gap - estimated_size.y;

    let x = if right_x + estimated_size.x <= bounds.right() {
        right_x
    } else if left_x >= bounds.left() {
        left_x
    } else {
        clamp_axis(right_x, bounds.left(), bounds.right() - estimated_size.x)
    };

    let y = if bottom_y + estimated_size.y <= bounds.bottom() {
        bottom_y
    } else if top_y >= bounds.top() {
        top_y
    } else {
        clamp_axis(bottom_y, bounds.top(), bounds.bottom() - estimated_size.y)
    };

    egui::pos2(x, y)
}

fn estimate_tooltip_size(text: &str, max_width: f32) -> egui::Vec2 {
    let char_width = 7.2;
    let content_max = (max_width - theme::SPACE_MD * 2.0).max(80.0);
    let mut visual_lines = 0.0_f32;
    let mut widest = 0.0_f32;

    for line in text.lines().chain((text.is_empty()).then_some("")) {
        let line_width = line.chars().count() as f32 * char_width;
        widest = widest.max(line_width);
        visual_lines += (line_width / content_max).ceil().max(1.0);
    }

    let width = (widest + theme::SPACE_MD * 2.0).clamp(48.0, max_width);
    let height = visual_lines * 15.0 + theme::SPACE_MD * 2.0;
    egui::vec2(width, height)
}

fn clamp_axis(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        min
    } else {
        value.clamp(min, max)
    }
}


// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
                .stroke(Stroke::NONE),
        )
        .show_separator_line(false)
        .show(ctx, |ui| {
            let top_line = ui.max_rect().x_range();
            ui.painter().hline(top_line, ui.max_rect().top(), Stroke::new(1.0, theme::border_subtle()));
            ui.set_min_height(22.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_LG;

                // Left: connection status
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let dot_color = match &conn.status {
                            ConnectionStatus::Connected { .. } => theme::ACCENT_GREEN,
                            ConnectionStatus::Connecting => theme::ACCENT_YELLOW,
                            ConnectionStatus::Disconnected => theme::ACCENT_RED,
                        };
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(10.0, 16.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.0, dot_color);
                        let status_label = match &conn.status {
                            ConnectionStatus::Connected { .. } => {
                                format!("connected \u{00B7} {}", conn.config.display_name)
                            }
                            ConnectionStatus::Connecting => t("status_connecting"),
                            ConnectionStatus::Disconnected => t("status_disconnected"),
                        };
                        ui.label(
                            RichText::new(status_label)
                                .color(theme::text_muted())
                                .size(11.0),
                        );
                        if let ConnectionStatus::Connected { server_version } = &conn.status {
                            ui.label(
                                RichText::new(format!("PG {server_version}"))
                                    .color(theme::text_muted())
                                    .size(11.0),
                            );
                        }
                        if state.explicit_tx_active {
                            let elapsed = state
                                .explicit_tx_started
                                .map(|s| s.elapsed().as_secs())
                                .unwrap_or(0);
                            ui.label(
                                RichText::new(format!("tx: 1 open \u{00B7} {elapsed}s"))
                                    .color(theme::ACCENT_YELLOW)
                                    .size(11.0),
                            );
                        }
                    }
                } else {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 3.0, theme::text_disabled());
                    ui.label(
                        RichText::new(t("no_connection"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                }

                // Right side
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = theme::SPACE_LG;

                    // Version
                    ui.label(
                        RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .color(theme::text_muted())
                            .size(11.0),
                    );

                    // Diagnostics toggle
                    {
                        let count = state.diagnostics_panel.entry_count();
                        let has_error = count > 0
                            && state.diagnostics_panel.entries().any(|e| {
                                e.severity
                                    == crate::ui::diagnostics_panel::DiagSeverity::Error
                            });
                        let color = if count == 0 {
                            theme::text_muted()
                        } else if has_error {
                            theme::ACCENT_RED
                        } else {
                            theme::ACCENT_YELLOW
                        };
                        let text = if count > 0 {
                            format!("Diagnostics \u{25BE} {count}")
                        } else {
                            "Diagnostics \u{25BE}".to_string()
                        };
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new(&text).color(color).size(11.0),
                            )
                            .fill(if state.diagnostics_panel.visible {
                                theme::with_alpha(color, 20)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_SM)),
                        );
                        if btn.clicked() {
                            state.diagnostics_panel.visible =
                                !state.diagnostics_panel.visible;
                        }
                    }

                    // Last query stats
                                        if let Some(ref result) = state.current_result {
                                            ui.label(
                                                RichText::new(format!(
                                                    "last query {} ms · {} rows",
                                                    result.execution_time_ms,
                                                    result.rows.len(),
                                                ))
                                                .color(theme::text_muted())
                                                .size(11.0),
                                            );
                                            render_mini_bar_chart(ui, result);
                                        }

                    if state.query_running {
                        ui.spinner();
                        ui.label(
                            RichText::new(t("loading"))
                                .color(theme::ACCENT_YELLOW)
                                .size(11.0),
                        );
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Tree panel header
// ---------------------------------------------------------------------------

fn render_tree_panel_header(ui: &mut egui::Ui, state: &mut AppState) {
    use crate::state::TreePanelTab;

    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());

        // 1) Connection mini-status (FIRST — matching mockup parts.js:17-21)
        {
            let (dot_color, label_text, version_text) =
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let dot = match &conn.status {
                            ConnectionStatus::Connected { .. } => theme::ACCENT_GREEN,
                            ConnectionStatus::Connecting => theme::ACCENT_YELLOW,
                            ConnectionStatus::Disconnected => theme::ACCENT_RED,
                        };
                        let name = if conn.config.display_name.is_empty() {
                            format!("{}@{}", conn.config.username, conn.config.host)
                        } else {
                            conn.config.display_name.clone()
                        };
                        let ver = match &conn.status {
                            ConnectionStatus::Connected { server_version } => {
                                format!("PG {server_version}")
                            }
                            _ => String::new(),
                        };
                        (dot, name, ver)
                    } else {
                        (theme::text_disabled(), t("panel_no_connection"), String::new())
                    }
                } else {
                    (theme::text_disabled(), t("panel_no_connection"), String::new())
                };

            ui.horizontal(|ui| {
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 3.0, dot_color);
                ui.label(
                    RichText::new(label_text)
                        .color(theme::text_secondary())
                        .monospace()
                        .size(11.0),
                );
                if !version_text.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(version_text)
                                .color(theme::text_disabled())
                                .size(11.0),
                        );
                    });
                }
            });
            ui.add_space(theme::SPACE_SM);
        }

        // 2) Search input (SECOND)
        let search_frame = egui::Frame::new()
            .fill(theme::bg_darkest())
            .stroke(Stroke::new(1.0, theme::border_default()))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::symmetric(theme::SPACE_MD_I, theme::SPACE_SM_I));
        search_frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                crate::ui::icon_img_tinted(ui, icons_svg::INFO, "tree_search_ic", 12.0, theme::text_muted());
                ui.add(
                    egui::TextEdit::singleline(&mut state.tree_search)
                        .hint_text(t("panel_search_schema"))
                        .background_color(Color32::TRANSPARENT)
                        .text_color(theme::text_primary())
                        .margin(Margin::ZERO)
                        .min_size(egui::vec2(0.0, 18.0))
                        .font(egui::FontId::proportional(12.0))
                        .frame(false),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("\u{2318}K")
                            .color(theme::text_disabled())
                            .monospace()
                            .size(10.0),
                    );
                });
            });
        });

        ui.add_space(theme::SPACE_SM);

        // 3) Tab bar: Schema | Roles | History | Snippets (THIRD)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_XS;
            for (tab, label) in [
                (TreePanelTab::Schema, "Schema"),
                (TreePanelTab::Roles, "Roles"),
                (TreePanelTab::History, "History"),
                (TreePanelTab::Snippets, "Snippets"),
            ] {
                let selected = state.tree_panel_tab == tab;
                let text_color = if selected {
                    theme::accent_color()
                } else {
                    theme::text_muted()
                };
                let bg = if selected {
                    theme::with_alpha(theme::accent_color(), 30)
                } else {
                    Color32::TRANSPARENT
                };

                let galley = ui.painter().layout_no_wrap(
                    label.to_owned(),
                    egui::FontId::proportional(10.5),
                    text_color,
                );
                let btn_width = galley.rect.width() + 16.0;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(btn_width, 22.0), egui::Sense::click());

                let fill = if response.hovered() && !selected {
                    theme::bg_light()
                } else {
                    bg
                };
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
                ui.painter().galley(
                    egui::pos2(
                        rect.center().x - galley.rect.width() / 2.0,
                        rect.center().y - galley.rect.height() / 2.0,
                    ),
                    galley,
                    text_color,
                );

                if response.clicked() {
                    state.tree_panel_tab = tab;
                }
            }
        });
    });
}

/// 결과 첫 컬럼이 숫자면 미니 막대 차트 표시.
fn render_mini_bar_chart(ui: &mut egui::Ui, result: &crate::types::QueryResult) {
    if result.columns.is_empty() || result.rows.is_empty() {
        return;
    }
    let mut values: Vec<f64> = Vec::with_capacity(result.rows.len().min(64));
    let mut max_v = f64::NEG_INFINITY;
    let mut min_v = f64::INFINITY;
    for (i, row) in result.rows.iter().enumerate() {
        if i >= 64 { break; }
        if let Some(val) = row.get(0) {
            if let Some(n) = cell_to_f64(val) {
                if n.is_finite() {
                    values.push(n);
                    if n > max_v { max_v = n; }
                    if n < min_v { min_v = n; }
                }
            }
        }
    }
    if values.len() < 2 || !max_v.is_finite() { return; }
    let span = (max_v - min_v).max(1e-9);
    let height = 40.0;
    let width = ui.available_width().min(360.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(2), theme::bg_dark());
    let bar_w = (width / values.len() as f32).max(1.0);
    for (i, v) in values.iter().enumerate() {
        let norm = ((v - min_v) / span) as f32;
        let h = norm * height;
        let x = rect.left() + i as f32 * bar_w;
        let y = rect.bottom() - h;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_w - 0.5, h)),
            CornerRadius::ZERO,
            theme::accent_color(),
        );
    }
}

fn cell_to_f64(v: &crate::types::CellValue) -> Option<f64> {
    use crate::types::CellValue::*;
    match v {
        Int(i) => Some(*i as f64),
        Float(f) => Some(*f),
        Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Text(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}
