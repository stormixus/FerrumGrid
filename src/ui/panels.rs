use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, StrokeKind};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{main_view_title, AppState, ConnectionStatus, MainView};
use crate::ui::{editor, grid, icons_svg, theme, tree_browser};

pub fn render_panels(
    ctx: &egui::Context,
    state: &mut AppState,
    bridge: &DbBridge,
    settings: &mut crate::storage::settings::AppSettings,
) {
    render_main_toolbar(ctx, state);
    render_status_bar(ctx, state);

    // Diagnostics panel (above status bar)
    if state.diagnostics_panel.visible || state.diagnostics_panel.unsafe_ctid_active {
        let panel_height = if state.diagnostics_panel.visible {
            140.0
        } else {
            28.0
        };
        egui::TopBottomPanel::bottom("diagnostics_panel")
            .min_height(28.0)
            .default_height(panel_height)
            .max_height(300.0)
            .resizable(state.diagnostics_panel.visible)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_darkest())
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
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_shell())
                    .inner_margin(Margin::ZERO),
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

    // Bottom panel: result grid
    if state.show_result_panel && state.active_main_view != MainView::Data {
        egui::TopBottomPanel::bottom("result_panel")
            .default_height(282.0)
            .min_height(108.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_darkest())
                    .inner_margin(Margin::ZERO),
            )
            .show(ctx, |ui| {
                grid::render_grid(ui, state, bridge);
            });
    }

    // Right panel: Info / Properties
    if state.show_info_panel {
        egui::SidePanel::right("info_panel")
            .default_width(240.0)
            .min_width(180.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::bg_shell())
                    .inner_margin(Margin::ZERO),
            )
            .show(ctx, |ui| {
                grid::render_info_panel(ui, state, bridge);
            });
    }

    // Center: SQL editor or Objects view
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::bg_dark())
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
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
        .and_then(|panel| {
            ctx.input(|input| input.pointer.hover_pos())
                .map(|pos| (panel, pos))
        })
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
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;

            for (index, tab) in tabs.iter().enumerate() {
                let selected = index == active;
                let response = render_workspace_tab(ui, tab.view, &tab.title, selected);
                if response.clicked() {
                    activate = Some(index);
                }

                let close_rect = egui::Rect::from_center_size(
                    egui::pos2(response.rect.right() - 13.0, response.rect.center().y),
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
    let color = workspace_tab_color(view);
    let label = truncate_tab_label(title, 26);
    let width = tab_width(ui, &label).clamp(96.0, 240.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::click());

    let bg = if selected {
        theme::bg_dark()
    } else if response.hovered() {
        theme::with_alpha(color, 22)
    } else {
        theme::bg_darkest()
    };
    let border = if selected {
        theme::with_alpha(color, 150)
    } else {
        theme::border_subtle()
    };
    let paint_rect = rect.shrink2(egui::vec2(0.0, 1.0));
    ui.painter()
        .rect_filled(paint_rect, CornerRadius::same(theme::RADIUS_MD), bg);
    ui.painter().rect_stroke(
        paint_rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(paint_rect.left() + 8.0, paint_rect.bottom() - 2.0),
                egui::vec2((paint_rect.width() - 16.0).max(12.0), 2.0),
            ),
            CornerRadius::same(theme::RADIUS_SM),
            color,
        );
    }

    let dot_center = egui::pos2(rect.left() + 12.0, rect.center().y);
    ui.painter().circle_filled(dot_center, 3.5, color);
    ui.painter().text(
        egui::pos2(rect.left() + 22.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        if selected {
            theme::text_primary()
        } else {
            theme::text_secondary()
        },
    );

    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, title);
    response
}

fn render_workspace_add_tab_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(30.0, 28.0), egui::Sense::click());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let paint_rect = rect.shrink2(egui::vec2(1.0, 1.0));
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
        StrokeKind::Inside,
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

fn tab_width(ui: &egui::Ui, label: &str) -> f32 {
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(12.0),
        theme::text_primary(),
    );
    galley.rect.width() + 48.0
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

fn workspace_tab_color(view: MainView) -> Color32 {
    match view {
        MainView::Connection => theme::ACCENT_GREEN,
        MainView::Table => theme::ACCENT_COPPER,
        MainView::View => theme::ACCENT_BLUE,
        MainView::MaterializedView => theme::ACCENT_TEAL,
        MainView::Function => theme::ACCENT_YELLOW,
        MainView::User => theme::ACCENT_COPPER_LIGHT,
        MainView::Query => theme::ACCENT_BLUE,
        MainView::Data => theme::ACCENT_TEAL,
        MainView::Backup => theme::text_muted(),
        MainView::Automation => theme::ACCENT_TEAL,
        MainView::Model => theme::ACCENT_GREEN,
        MainView::BI => theme::ACCENT_RED,
    }
}

// ---------------------------------------------------------------------------
// Main Toolbar (Navicat style)
// ---------------------------------------------------------------------------

fn render_main_toolbar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("main_toolbar")
        .exact_height(84.0)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_darkest())
                .inner_margin(Margin::symmetric(theme::SPACE_XL_I, 0))
                .stroke(Stroke::new(1.0, theme::border_subtle())),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_XXL;

                render_toolbar_item(
                    ui,
                    state,
                    MainView::Connection,
                    t("toolbar_connection"),
                    theme::ACCENT_GREEN,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Table,
                    t("toolbar_table"),
                    theme::ACCENT_COPPER,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::View,
                    t("toolbar_view"),
                    theme::ACCENT_BLUE,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::MaterializedView,
                    t("toolbar_materialized_view"),
                    theme::ACCENT_TEAL,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Function,
                    t("toolbar_function"),
                    theme::ACCENT_YELLOW,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::User,
                    t("toolbar_user"),
                    theme::ACCENT_COPPER_LIGHT,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Query,
                    t("toolbar_query"),
                    theme::ACCENT_BLUE,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Backup,
                    t("toolbar_backup"),
                    theme::text_muted(),
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Automation,
                    t("toolbar_automation"),
                    theme::ACCENT_TEAL,
                );
                render_toolbar_item(
                    ui,
                    state,
                    MainView::Model,
                    t("toolbar_model"),
                    theme::ACCENT_GREEN,
                );
                render_toolbar_item(ui, state, MainView::BI, t("toolbar_bi"), theme::ACCENT_RED);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(theme::SPACE_MD);

                    let settings_resp = ui.add_sized(
                        egui::vec2(32.0, 32.0),
                        egui::Button::new(
                            RichText::new("\u{2699}")
                                .size(16.0)
                                .color(theme::text_primary()),
                        )
                        .fill(theme::bg_light())
                        .stroke(Stroke::new(1.0, theme::border_default()))
                        .corner_radius(CornerRadius::same(theme::RADIUS_LG)),
                    );
                    show_dark_hover_tooltip(
                        ui,
                        settings_resp.id.with("tooltip"),
                        &settings_resp,
                        &t("settings_title"),
                    );
                    if settings_resp.clicked() {
                        state.show_settings_dialog = true;
                    }

                    ui.add_space(theme::SPACE_XL);

                    render_pane_toggles(ui, state);
                });
            });
        });
}

#[derive(Clone, Copy)]
enum PaneToggle {
    Navigator,
    Results,
    Info,
}

fn render_pane_toggles(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Frame::new()
        .fill(theme::bg_shell())
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::symmetric(theme::SPACE_SM_I, theme::SPACE_XS_I))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_XS;
            pane_toggle_button(
                ui,
                PaneToggle::Info,
                &mut state.show_info_panel,
                t("view_toggle_info"),
            );
            pane_toggle_button(
                ui,
                PaneToggle::Results,
                &mut state.show_result_panel,
                t("view_toggle_results"),
            );
            pane_toggle_button(
                ui,
                PaneToggle::Navigator,
                &mut state.show_tree_panel,
                t("view_toggle_navigator"),
            );
        });
}

fn pane_toggle_button(ui: &mut egui::Ui, pane: PaneToggle, visible: &mut bool, tooltip: String) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(25.0, 25.0), egui::Sense::click());
    if response.clicked() {
        *visible = !*visible;
    }

    let hovered = response.hovered();
    let bg = if *visible {
        toolbar_alpha(theme::ACCENT_BLUE, if hovered { 45 } else { 30 })
    } else if hovered {
        toolbar_alpha(theme::text_muted(), 22)
    } else {
        Color32::TRANSPARENT
    };
    let border = if *visible {
        toolbar_alpha(theme::ACCENT_BLUE, 160)
    } else {
        theme::border_default()
    };

    ui.painter()
        .rect_filled(rect.shrink(1.0), CornerRadius::same(theme::RADIUS_MD), bg);
    ui.painter().rect_stroke(
        rect.shrink(1.0),
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    paint_pane_icon(
        ui.painter(),
        rect.shrink(5.0),
        pane,
        if *visible {
            theme::ACCENT_BLUE
        } else {
            theme::text_muted()
        },
        *visible,
    );
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, &tooltip);
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
        toolbar_alpha(color, 48)
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

fn render_toolbar_item(
    ui: &mut egui::Ui,
    state: &mut AppState,
    view: MainView,
    label: String,
    color: Color32,
) {
    let selected = state.active_main_view == view;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(72.0, 72.0), egui::Sense::click());
    let hovered = response.hovered();
    let clicked = response.clicked();
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, &label);

    if clicked {
        state.open_workspace_main_view(view);
        if view == MainView::Connection {
            state.show_connection_dialog = true;
            state.connection_dialog = Default::default();
        }
    }

    let card_rect = rect.shrink2(egui::vec2(3.0, 4.0));
    if selected {
        ui.painter().rect_filled(
            card_rect,
            CornerRadius::same(theme::RADIUS_LG),
            toolbar_alpha(color, 225),
        );
        ui.painter().rect_stroke(
            card_rect,
            CornerRadius::same(theme::RADIUS_LG),
            Stroke::new(1.0, toolbar_alpha(Color32::WHITE, 75)),
            StrokeKind::Inside,
        );
    } else if hovered {
        ui.painter().rect_filled(
            card_rect,
            CornerRadius::same(theme::RADIUS_LG),
            toolbar_alpha(color, 26),
        );
        ui.painter().rect_stroke(
            card_rect,
            CornerRadius::same(theme::RADIUS_LG),
            Stroke::new(1.0, toolbar_alpha(color, 90)),
            StrokeKind::Inside,
        );
    }

    let icon_color = if selected { Color32::WHITE } else { color };
    let label_color = if selected {
        Color32::WHITE
    } else {
        theme::text_secondary()
    };
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, rect.min.y + 29.0),
        egui::vec2(31.0, 31.0),
    );

    paint_toolbar_icon(ui.painter(), view, icon_rect, icon_color, selected);
    paint_toolbar_label(ui, rect, &label, label_color);
}

fn toolbar_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn paint_toolbar_label(ui: &egui::Ui, rect: egui::Rect, label: &str, color: Color32) {
    let max_width = rect.width() - 8.0;
    let mut font_size = 10.5;
    let mut galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(font_size),
        color,
    );

    if galley.rect.width() > max_width {
        font_size = (font_size * max_width / galley.rect.width()).clamp(8.0, 10.5);
        galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(font_size),
            color,
        );
    }

    let pos = egui::pos2(
        rect.center().x - galley.rect.width() / 2.0,
        rect.min.y + 57.0 - galley.rect.height() / 2.0,
    );
    ui.painter().galley(pos, galley, color);
}

fn paint_toolbar_icon(
    painter: &egui::Painter,
    view: MainView,
    rect: egui::Rect,
    color: Color32,
    selected: bool,
) {
    let stroke = Stroke::new(if selected { 2.35 } else { 2.0 }, color);
    let fine_stroke = Stroke::new(if selected { 1.7 } else { 1.45 }, color);
    let fill = toolbar_alpha(color, if selected { 42 } else { 24 });
    let r = rect.shrink(2.0);
    let cx = r.center().x;
    let cy = r.center().y;

    match view {
        MainView::Connection => {
            let plug =
                egui::Rect::from_center_size(egui::pos2(cx, cy + 4.0), egui::vec2(15.0, 12.0));
            painter.rect_filled(plug, CornerRadius::same(theme::RADIUS_SM), fill);
            painter.rect_stroke(
                plug,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    egui::pos2(cx - 4.0, plug.top()),
                    egui::pos2(cx - 4.0, plug.top() - 7.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(cx + 4.0, plug.top()),
                    egui::pos2(cx + 4.0, plug.top() - 7.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(cx, plug.bottom()),
                    egui::pos2(cx, plug.bottom() + 6.0),
                ],
                stroke,
            );
            painter.circle_stroke(egui::pos2(cx, r.top() + 6.0), 4.8, fine_stroke);
            painter.line_segment(
                [
                    egui::pos2(cx, r.top() + 10.8),
                    egui::pos2(cx, plug.top() - 7.0),
                ],
                fine_stroke,
            );
        }
        MainView::Table => {
            let table = egui::Rect::from_center_size(r.center(), egui::vec2(24.0, 21.0));
            let header =
                egui::Rect::from_min_max(table.min, egui::pos2(table.right(), table.top() + 6.0));
            painter.rect_filled(header, CornerRadius::same(theme::RADIUS_SM), fill);
            painter.rect_stroke(
                table,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
            for y in [table.top() + 7.0, table.top() + 14.0] {
                painter.line_segment(
                    [egui::pos2(table.left(), y), egui::pos2(table.right(), y)],
                    fine_stroke,
                );
            }
            for x in [table.left() + 8.0, table.left() + 16.0] {
                painter.line_segment(
                    [egui::pos2(x, table.top()), egui::pos2(x, table.bottom())],
                    fine_stroke,
                );
            }
        }
        MainView::View => {
            painter.line_segment(
                [
                    egui::pos2(r.left() + 1.0, cy),
                    egui::pos2(cx, r.top() + 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(cx, r.top() + 5.0),
                    egui::pos2(r.right() - 1.0, cy),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(r.right() - 1.0, cy),
                    egui::pos2(cx, r.bottom() - 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(cx, r.bottom() - 5.0),
                    egui::pos2(r.left() + 1.0, cy),
                ],
                stroke,
            );
            painter.circle_filled(r.center(), 3.2, color);
            painter.circle_stroke(r.center(), 6.0, fine_stroke);
        }
        MainView::MaterializedView => {
            let shell = egui::Rect::from_center_size(r.center(), egui::vec2(26.0, 24.0));
            painter.rect_filled(shell, CornerRadius::same(theme::RADIUS_SM), fill);
            painter.rect_stroke(
                shell,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
            let eye_center = shell.center();
            painter.line_segment(
                [
                    eye_center + egui::vec2(-8.0, 0.0),
                    eye_center + egui::vec2(0.0, -5.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    eye_center + egui::vec2(0.0, -5.0),
                    eye_center + egui::vec2(8.0, 0.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    eye_center + egui::vec2(8.0, 0.0),
                    eye_center + egui::vec2(0.0, 5.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    eye_center + egui::vec2(0.0, 5.0),
                    eye_center + egui::vec2(-8.0, 0.0),
                ],
                fine_stroke,
            );
            painter.circle_filled(eye_center, 2.7, color);
        }
        MainView::Function => {
            painter.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                "fn",
                egui::FontId::proportional(17.5),
                color,
            );
            painter.line_segment(
                [
                    egui::pos2(r.left() + 4.0, r.bottom() - 4.0),
                    egui::pos2(r.right() - 4.0, r.bottom() - 4.0),
                ],
                fine_stroke,
            );
        }
        MainView::User => {
            painter.circle_stroke(egui::pos2(cx, r.top() + 8.5), 5.4, stroke);
            let shoulders = egui::Rect::from_center_size(
                egui::pos2(cx, r.bottom() - 6.0),
                egui::vec2(23.0, 11.0),
            );
            painter.rect_filled(shoulders, CornerRadius::same(theme::RADIUS_LG), fill);
            painter.rect_stroke(
                shoulders,
                CornerRadius::same(theme::RADIUS_LG),
                stroke,
                StrokeKind::Inside,
            );
        }
        MainView::Query => {
            let doc = egui::Rect::from_center_size(r.center(), egui::vec2(22.0, 25.0));
            painter.rect_filled(doc, CornerRadius::same(theme::RADIUS_SM), fill);
            painter.rect_stroke(
                doc,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    egui::pos2(doc.left() + 5.0, doc.top() + 8.0),
                    egui::pos2(doc.right() - 5.0, doc.top() + 8.0),
                ],
                fine_stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(doc.left() + 5.0, doc.top() + 14.0),
                    egui::pos2(doc.right() - 7.0, doc.top() + 14.0),
                ],
                fine_stroke,
            );
            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(doc.left() + 7.0, doc.bottom() - 7.0),
                    egui::pos2(doc.left() + 7.0, doc.bottom() - 2.0),
                    egui::pos2(doc.left() + 13.0, doc.bottom() - 4.5),
                ],
                color,
                Stroke::NONE,
            ));
        }
        MainView::Data => {
            let table = egui::Rect::from_center_size(r.center(), egui::vec2(25.0, 22.0));
            painter.rect_filled(table, CornerRadius::same(theme::RADIUS_SM), fill);
            painter.rect_stroke(
                table,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
            for y in [table.top() + 7.0, table.top() + 14.0] {
                painter.line_segment(
                    [egui::pos2(table.left(), y), egui::pos2(table.right(), y)],
                    fine_stroke,
                );
            }
            for x in [table.left() + 8.0, table.left() + 17.0] {
                painter.line_segment(
                    [egui::pos2(x, table.top()), egui::pos2(x, table.bottom())],
                    fine_stroke,
                );
            }
            painter.circle_filled(table.right_bottom() - egui::vec2(5.0, 5.0), 2.4, color);
        }
        MainView::Backup => {
            painter.circle_stroke(r.center(), 9.0, fine_stroke);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(cx + 9.0, cy - 9.0),
                    egui::pos2(cx + 12.5, cy - 2.0),
                    egui::pos2(cx + 5.0, cy - 3.0),
                ],
                color,
                Stroke::NONE,
            ));
            let tray = egui::Rect::from_center_size(
                egui::pos2(cx, r.bottom() - 4.0),
                egui::vec2(22.0, 5.0),
            );
            painter.rect_stroke(
                tray,
                CornerRadius::same(theme::RADIUS_SM),
                stroke,
                StrokeKind::Inside,
            );
        }
        MainView::Model => {
            let nodes = [
                egui::pos2(cx - 9.0, cy - 7.0),
                egui::pos2(cx + 9.0, cy - 7.0),
                egui::pos2(cx, cy + 9.0),
            ];
            painter.line_segment([nodes[0], nodes[1]], fine_stroke);
            painter.line_segment([nodes[0], nodes[2]], fine_stroke);
            painter.line_segment([nodes[1], nodes[2]], fine_stroke);
            for node in nodes {
                painter.circle_filled(
                    node,
                    4.7,
                    toolbar_alpha(color, if selected { 70 } else { 42 }),
                );
                painter.circle_stroke(node, 4.7, stroke);
            }
        }
        MainView::BI => {
            let axis_origin = egui::pos2(r.left() + 4.0, r.bottom() - 4.0);
            painter.line_segment(
                [axis_origin, egui::pos2(r.right() - 2.0, axis_origin.y)],
                stroke,
            );
            painter.line_segment(
                [axis_origin, egui::pos2(axis_origin.x, r.top() + 3.0)],
                stroke,
            );
            let bar_width = 4.6;
            for (idx, height) in [9.0, 16.0, 22.0].into_iter().enumerate() {
                let x = axis_origin.x + 5.5 + idx as f32 * 6.8;
                let bar = egui::Rect::from_min_max(
                    egui::pos2(x, axis_origin.y - height),
                    egui::pos2(x + bar_width, axis_origin.y),
                );
                painter.rect_filled(
                    bar,
                    CornerRadius::same(theme::RADIUS_SM),
                    toolbar_alpha(color, if selected { 105 } else { 70 }),
                );
                painter.rect_stroke(
                    bar,
                    CornerRadius::same(theme::RADIUS_SM),
                    fine_stroke,
                    StrokeKind::Inside,
                );
            }
        }
        MainView::Automation => {
            painter.circle_stroke(r.center(), 8.5, stroke);
            painter.circle_filled(r.center(), 3.0, color);
            for angle in [
                0.0_f32,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
                std::f32::consts::PI + std::f32::consts::FRAC_PI_2,
            ] {
                let dir = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment([r.center() + dir * 10.0, r.center() + dir * 13.0], stroke);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .frame(
            egui::Frame::new()
                .fill(theme::bg_shell())
                .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
                .stroke(Stroke::new(1.0, theme::border_subtle())),
        )
        .show(ctx, |ui| {
            ui.set_min_height(24.0);
            ui.horizontal(|ui| {
                // Connection status dot + name
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let (dot_color, label) = match &conn.status {
                            ConnectionStatus::Connected { server_version } => {
                                (theme::ACCENT_GREEN, format!("PG {}", server_version))
                            }
                            ConnectionStatus::Connecting => {
                                (theme::ACCENT_YELLOW, t("status_connecting"))
                            }
                            ConnectionStatus::Disconnected => {
                                (theme::ACCENT_RED, t("status_disconnected"))
                            }
                        };

                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(12.0, 18.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.8, dot_color);
                        ui.add_space(2.0);

                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(theme::text_primary())
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(format!("  {label}"))
                                .color(theme::text_muted())
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(format!("  {}", state.status_message))
                                .color(theme::text_disabled())
                                .size(11.0),
                        );
                    }
                } else {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 3.5, theme::text_disabled());
                    ui.label(
                        RichText::new(t("no_connection"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                    ui.label(
                        RichText::new(format!("  {}", state.status_message))
                            .color(theme::text_disabled())
                            .size(11.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Diagnostics toggle
                    {
                        let count = state.diagnostics_panel.entry_count();
                        let has_error = count > 0
                            && state.diagnostics_panel.entries().any(|e| {
                                e.severity
                                    == crate::ui::diagnostics_panel::DiagSeverity::Error
                            });
                        let color = if count == 0 {
                            theme::text_disabled()
                        } else if has_error {
                            theme::ACCENT_RED
                        } else {
                            theme::ACCENT_YELLOW
                        };
                        let text = if count > 0 {
                            format!("\u{25b2} {count}")
                        } else {
                            "\u{25b2}".to_string()
                        };
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new(&text).color(color).size(10.0).monospace(),
                            )
                            .fill(if state.diagnostics_panel.visible {
                                theme::with_alpha(color, 20)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(Stroke::NONE)
                            .corner_radius(CornerRadius::same(theme::RADIUS_SM)),
                        );
                        let diag_tooltip = if count > 0 {
                            format!("Diagnostics Panel ({count} entries)")
                        } else {
                            "Diagnostics Panel".to_string()
                        };
                        show_dark_hover_tooltip(
                            ui,
                            btn.id.with("diag_tip"),
                            &btn,
                            &diag_tooltip,
                        );
                        if btn.clicked() {
                            state.diagnostics_panel.visible =
                                !state.diagnostics_panel.visible;
                        }
                        ui.separator();
                    }

                    if let Some(ref result) = state.current_result {
                        ui.label(
                            RichText::new(format!(
                                "{}ms  \u{2502}  {} {}",
                                result.execution_time_ms,
                                result.rows.len(),
                                t("result_rows")
                            ))
                            .color(theme::text_muted())
                            .size(11.0),
                        );
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
    let frame = egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .stroke(Stroke::new(1.0, theme::border_subtle()));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    crate::ui::icon_img(ui, icons_svg::DATABASE, "tree_db", 14.0);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(t("explorer_title"))
                            .color(theme::text_primary())
                            .size(13.0)
                            .strong(),
                    );
                });
                ui.label(
                    RichText::new(format!(
                        "{} saved connections",
                        state.saved_connections.len()
                    ))
                    .color(theme::text_muted())
                    .size(10.0),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn = ui.add(
                    theme::secondary_icon_button(
                        crate::ui::icon_image(ui, icons_svg::PLUS, "explorer_new_icon", 12.0),
                        t("explorer_new"),
                    )
                    .min_size(egui::vec2(74.0, 30.0)),
                );

                if btn.clicked() {
                    state.show_connection_dialog = true;
                    state.connection_dialog = Default::default();
                }
            });
        });
    });
}
