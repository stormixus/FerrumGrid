use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, StrokeKind};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{AppState, ConnectionStatus, MainView};
use crate::ui::{editor, grid, icons_svg, theme, tree_browser};

pub fn render_panels(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    render_main_toolbar(ctx, state);
    render_status_bar(ctx, state);

    // Left panel: database tree
    if state.show_tree_panel {
        egui::SidePanel::left("tree_panel")
            .default_width(286.0)
            .min_width(220.0)
            .max_width(440.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_SHELL)
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
    if state.show_result_panel {
        egui::TopBottomPanel::bottom("result_panel")
            .default_height(282.0)
            .min_height(108.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_DARKEST)
                    .inner_margin(Margin::ZERO),
            )
            .show(ctx, |ui| {
                grid::render_grid(ui, state);
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
                    .fill(theme::BG_SHELL)
                    .inner_margin(Margin::ZERO),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(theme::BG_SHELL)
                    .inner_margin(Margin::same(theme::SPACE_LG_I))
                    .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            crate::ui::icon_img(ui, icons_svg::INFO, "info", 14.0);
                            ui.add_space(4.0);
                            ui.label(RichText::new("Info").color(theme::TEXT_PRIMARY).strong());
                        });
                    });

                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.label(RichText::new("No Info").color(theme::TEXT_MUTED).size(14.0));
                });
            });
    }

    // Center: SQL editor or Objects view
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARK)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| match state.active_main_view {
            crate::state::MainView::Table
            | crate::state::MainView::View
            | crate::state::MainView::MaterializedView
            | crate::state::MainView::Function
            | crate::state::MainView::User
            | crate::state::MainView::Backup
            | crate::state::MainView::Automation
            | crate::state::MainView::Model
            | crate::state::MainView::BI => {
                crate::ui::objects::render_objects_view(ui, state, bridge);
            }
            crate::state::MainView::Connection | crate::state::MainView::Query => {
                editor::render_editor(ui, state, bridge);
            }
        });
}

// ---------------------------------------------------------------------------
// Main Toolbar (Navicat style)
// ---------------------------------------------------------------------------

fn render_main_toolbar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("main_toolbar")
        .exact_height(84.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARKEST)
                .inner_margin(Margin::symmetric(theme::SPACE_XL_I, 0))
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
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
                    theme::TEXT_MUTED,
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

                    let settings_resp = ui
                        .add_sized(
                            egui::vec2(32.0, 32.0),
                            egui::Button::new(
                                RichText::new("\u{2699}")
                                    .size(16.0)
                                    .color(theme::TEXT_PRIMARY),
                            )
                            .fill(theme::BG_LIGHT)
                            .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                            .corner_radius(CornerRadius::same(theme::RADIUS_LG)),
                        )
                        .on_hover_text(t("settings_title"));
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
        .fill(theme::BG_SHELL)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
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
        toolbar_alpha(theme::TEXT_MUTED, 22)
    } else {
        Color32::TRANSPARENT
    };
    let border = if *visible {
        toolbar_alpha(theme::ACCENT_BLUE, 160)
    } else {
        theme::BORDER_DEFAULT
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
            theme::TEXT_MUTED
        },
        *visible,
    );
    response.on_hover_text(tooltip);
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
    response.on_hover_text(label.clone());

    if clicked {
        state.active_main_view = view;
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
        theme::TEXT_SECONDARY
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
            for angle in [0.0_f32, 1.57, 3.14, 4.71] {
                let dir = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment([r.center() + dir * 10.0, r.center() + dir * 13.0], stroke);
            }
        }
    }
}

fn render_brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        crate::ui::icon_img(ui, icons_svg::DATABASE, "brand_logo", 24.0);

        ui.add_space(theme::SPACE_SM);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("FerrumGrid")
                        .color(theme::TEXT_PRIMARY)
                        .strong()
                        .size(13.0),
                );
                ui.label(
                    RichText::new("POSTGRES WORKBENCH")
                        .color(theme::TEXT_MUTED)
                        .size(8.5),
                );
            });
        });
    });
}

fn render_top_status_pill(ui: &mut egui::Ui, state: &AppState) {
    let connected = state
        .connections
        .values()
        .filter(|conn| matches!(conn.status, ConnectionStatus::Connected { .. }))
        .count();
    let connecting = state
        .connections
        .values()
        .any(|conn| matches!(conn.status, ConnectionStatus::Connecting));

    let (label, color) = if state.query_running {
        ("Query running".to_string(), theme::ACCENT_YELLOW)
    } else if let Some(conn_id) = state.active_connection {
        if let Some(conn) = state.connections.get(&conn_id) {
            (conn.config.display_name.clone(), theme::ACCENT_GREEN)
        } else {
            (state.status_message.clone(), theme::TEXT_MUTED)
        }
    } else if connecting {
        ("Connecting".to_string(), theme::ACCENT_YELLOW)
    } else if connected > 0 {
        (format!("{connected} connected"), theme::ACCENT_GREEN)
    } else {
        ("Offline".to_string(), theme::TEXT_MUTED)
    };

    let galley =
        ui.painter()
            .layout_no_wrap(label.clone(), egui::FontId::proportional(11.0), color);
    let width = (galley.rect.width() + 34.0).clamp(86.0, 220.0);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 24.0), egui::Sense::hover());

    let fill = if resp.hovered() {
        theme::with_alpha(color, 36)
    } else {
        theme::with_alpha(color, 22)
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_LG), fill);

    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
            rect.left_center() + egui::vec2(13.0, 0.0),
            egui::vec2(12.0, 12.0),
        )),
        |ui| {
            let (svg, name) = if state.query_running {
                (icons_svg::QUERY, "status_query")
            } else if connected > 0 {
                (icons_svg::CONNECT, "status_conn")
            } else {
                (icons_svg::DATABASE, "status_offline")
            };
            crate::ui::icon_img(ui, svg, name, 10.0);
        },
    );

    ui.painter().text(
        rect.left_center() + egui::vec2(23.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.0),
        if color == theme::TEXT_MUTED {
            theme::TEXT_SECONDARY
        } else {
            theme::TEXT_PRIMARY
        },
    );
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_SHELL)
                .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
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

                        let (dot_svg, dot_name) = match &conn.status {
                            ConnectionStatus::Connected { .. } => {
                                (icons_svg::CONNECT, "status_bar_conn")
                            }
                            _ => (icons_svg::DATABASE, "status_bar_other"),
                        };

                        ui.allocate_new_ui(
                            egui::UiBuilder::new().max_rect(egui::Rect::from_center_size(
                                ui.next_widget_position() + egui::vec2(6.0, 11.0),
                                egui::vec2(12.0, 12.0),
                            )),
                            |ui| {
                                crate::ui::icon_img(ui, dot_svg, dot_name, 10.0);
                            },
                        );
                        ui.add_space(14.0);

                        ui.label(
                            RichText::new(&conn.config.display_name)
                                .color(theme::TEXT_PRIMARY)
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(format!("  {label}"))
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                        );
                        ui.label(
                            RichText::new(format!("  {}", state.status_message))
                                .color(theme::TEXT_DISABLED)
                                .size(11.0),
                        );
                    }
                } else {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 3.5, theme::TEXT_DISABLED);
                    ui.label(
                        RichText::new(t("no_connection"))
                            .color(theme::TEXT_MUTED)
                            .size(11.0),
                    );
                    ui.label(
                        RichText::new(format!("  {}", state.status_message))
                            .color(theme::TEXT_DISABLED)
                            .size(11.0),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref result) = state.current_result {
                        ui.label(
                            RichText::new(format!(
                                "{}ms  \u{2502}  {} {}",
                                result.execution_time_ms,
                                result.rows.len(),
                                t("result_rows")
                            ))
                            .color(theme::TEXT_MUTED)
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
        .fill(theme::BG_SHELL)
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_MD as i8,
        ))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    crate::ui::icon_img(ui, icons_svg::DATABASE, "tree_db", 14.0);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(t("explorer_title"))
                            .color(theme::TEXT_PRIMARY)
                            .size(13.0)
                            .strong(),
                    );
                });
                ui.label(
                    RichText::new(format!(
                        "{} saved connections",
                        state.saved_connections.len()
                    ))
                    .color(theme::TEXT_MUTED)
                    .size(10.0),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let new_btn_label = "      ".to_owned() + &t("explorer_new");
                let btn = ui.add(theme::secondary_button(&new_btn_label));
                ui.allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(btn.rect.shrink2(egui::vec2(btn.rect.width() - 24.0, 0.0))),
                    |ui| {
                        crate::ui::icon_img(ui, icons_svg::PLUS, "explorer_new_icon", 12.0);
                    },
                );

                if btn.clicked() {
                    state.show_connection_dialog = true;
                    state.connection_dialog = Default::default();
                }
            });
        });
    });
}

pub fn panel_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(Margin::same(theme::SPACE_MD as i8))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
}
