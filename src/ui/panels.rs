use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{get_language, set_language, t, Language};
use crate::state::{AppState, ConnectionStatus};
use crate::ui::{editor, grid, icons, theme, tree_browser};

pub fn render_panels(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    render_menu_bar(ctx, state);
    render_status_bar(ctx, state);

    // Left panel: database tree
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

    // Bottom panel: result grid
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

    // Center: SQL editor
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARK)
                .inner_margin(Margin::ZERO),
        )
        .show(ctx, |ui| {
            editor::render_editor(ui, state, bridge);
        });
}

// ---------------------------------------------------------------------------
// Menu bar
// ---------------------------------------------------------------------------

fn render_menu_bar(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("menu_bar")
        .exact_height(40.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_SHELL)
                .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(40.0);
            ui.horizontal_centered(|ui| {
                render_brand(ui);
                ui.add_space(theme::SPACE_XL);

                egui::menu::bar(ui, |ui| {
                    ui.menu_button(t("menu_file"), |ui| {
                        if ui
                            .button(format!("{} {}", icons::PLUS, t("menu_new_connection")))
                            .clicked()
                        {
                            state.show_connection_dialog = true;
                            state.connection_dialog = Default::default();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t("menu_quit")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button(t("menu_query"), |ui| {
                        if ui
                            .button(format!("{} {}", icons::EXECUTE, t("menu_execute")))
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{} {}", icons::PLUS, t("menu_new_tab")))
                            .clicked()
                        {
                            let n = state.editor_tabs.len() + 1;
                            state
                                .editor_tabs
                                .push(crate::types::EditorTab::new(format!("Query {n}")));
                            state.active_tab = state.editor_tabs.len() - 1;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button(t("menu_view"), |ui| {
                        let dark = ui.visuals().dark_mode;
                        if ui
                            .button(if dark {
                                t("menu_light_mode")
                            } else {
                                t("menu_dark_mode")
                            })
                            .clicked()
                        {
                            if dark {
                                crate::ui::theme::FerrumTheme::apply_light(ctx);
                            } else {
                                crate::ui::theme::FerrumTheme::apply_dark(ctx);
                            }
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t("menu_er_diagram")).clicked() {
                            state.er_diagram.show_diagram = !state.er_diagram.show_diagram;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t("menu_table_designer")).clicked() {
                            crate::ui::table_designer::open_for_new_table(state);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t("menu_prisma")).clicked() {
                            crate::prisma::ui::open_prisma_window(state);
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(t("menu_language"), |ui| {
                            let current = get_language();
                            for lang in Language::all() {
                                let selected = current == lang;
                                if ui.selectable_label(selected, lang.name()).clicked() {
                                    set_language(lang);
                                    ui.close_menu();
                                }
                            }
                        });
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    render_top_status_pill(ui, state);
                    ui.add_space(theme::SPACE_MD);

                    if ui
                        .add(theme::primary_button(&format!(
                            "{} {}",
                            icons::PLUS,
                            t("explorer_new")
                        )))
                        .clicked()
                    {
                        state.show_connection_dialog = true;
                        state.connection_dialog = Default::default();
                    }
                });
            });
        });
}

fn render_brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (mark_rect, _) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
        ui.painter().rect_filled(
            mark_rect,
            CornerRadius::same(theme::RADIUS_LG),
            theme::BG_LIGHT,
        );
        ui.painter().rect_filled(
            mark_rect.shrink(5.0),
            CornerRadius::same(theme::RADIUS_SM),
            theme::ACCENT_COPPER,
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(mark_rect.min + egui::vec2(12.0, 4.0), egui::vec2(6.0, 6.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::ACCENT_TEAL,
        );

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
    ui.painter()
        .circle_filled(rect.left_center() + egui::vec2(13.0, 0.0), 3.5, color);
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

                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.5, dot_color);

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
                ui.label(
                    RichText::new(format!("{} {}", icons::DATABASE, t("explorer_title")))
                        .color(theme::TEXT_PRIMARY)
                        .size(13.0)
                        .strong(),
                );
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
                let new_label = format!("{} {}", icons::PLUS, t("explorer_new"));
                let btn = theme::secondary_button(&new_label);

                if ui.add(btn).clicked() {
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
