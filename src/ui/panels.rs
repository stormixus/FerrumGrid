use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::state::{AppState, ConnectionStatus};
use crate::ui::{editor, grid, icons, theme, tree_browser};

pub fn render_panels(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    render_menu_bar(ctx, state);
    render_status_bar(ctx, state);

    // Left panel: database tree
    egui::SidePanel::left("tree_panel")
        .default_width(260.0)
        .min_width(180.0)
        .max_width(420.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARKEST)
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
        .default_height(260.0)
        .min_height(80.0)
        .resizable(true)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARK)
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
        .exact_height(28.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARKEST)
                .inner_margin(Margin::symmetric(theme::SPACE_MD as i8, 0))
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(28.0);
            egui::menu::bar(ui, |ui| {
                // App wordmark
                ui.label(
                    RichText::new("FerrumGrid")
                        .color(theme::ACCENT_COPPER)
                        .strong()
                        .size(13.0),
                );

                ui.add_space(theme::SPACE_LG);

                ui.menu_button("File", |ui| {
                    if ui
                        .button(format!("{} New Connection", icons::PLUS))
                        .clicked()
                    {
                        state.show_connection_dialog = true;
                        state.connection_dialog = Default::default();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Query", |ui| {
                    if ui
                        .button(format!("{} Execute  Cmd+Return", icons::EXECUTE))
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("{} New Tab", icons::PLUS))
                        .clicked()
                    {
                        let n = state.editor_tabs.len() + 1;
                        state.editor_tabs.push(
                            crate::types::EditorTab::new(format!("Query {n}")),
                        );
                        state.active_tab = state.editor_tabs.len() - 1;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    let dark = ui.visuals().dark_mode;
                    if ui
                        .button(if dark { "Light Mode" } else { "Dark Mode" })
                        .clicked()
                    {
                        if dark {
                            ctx.set_visuals(egui::Visuals::light());
                        } else {
                            crate::ui::theme::FerrumTheme::apply_dark(ctx);
                        }
                        ui.close_menu();
                    }
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(ctx: &egui::Context, state: &AppState) {
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(22.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DARKEST)
                .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
                .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE)),
        )
        .show(ctx, |ui| {
            ui.set_min_height(22.0);
            ui.horizontal(|ui| {
                // Connection status dot + name
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let (dot_color, label) = match &conn.status {
                            ConnectionStatus::Connected { server_version } => (
                                theme::ACCENT_GREEN,
                                format!("PG {}", server_version),
                            ),
                            ConnectionStatus::Connecting => {
                                (theme::ACCENT_YELLOW, "Connecting\u{2026}".to_string())
                            }
                            ConnectionStatus::Disconnected => {
                                (theme::ACCENT_RED, "Disconnected".to_string())
                            }
                        };

                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(8.0, 8.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(dot_rect.center(), 3.5, dot_color);

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
                    }
                } else {
                    let (dot_rect, _) = ui.allocate_exact_size(
                        egui::vec2(8.0, 8.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().circle_filled(
                        dot_rect.center(),
                        3.5,
                        theme::TEXT_DISABLED,
                    );
                    ui.label(
                        RichText::new("No connection")
                            .color(theme::TEXT_MUTED)
                            .size(11.0),
                    );
                }

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if let Some(ref result) = state.current_result {
                            let row_s =
                                if result.rows.len() == 1 { "row" } else { "rows" };
                            ui.label(
                                RichText::new(format!(
                                    "{}ms  \u{2502}  {} {}",
                                    result.execution_time_ms,
                                    result.rows.len(),
                                    row_s
                                ))
                                .color(theme::TEXT_MUTED)
                                .size(11.0),
                            );
                        }

                        if state.query_running {
                            ui.spinner();
                            ui.label(
                                RichText::new("Running\u{2026}")
                                    .color(theme::ACCENT_YELLOW)
                                    .size(11.0),
                            );
                        }
                    },
                );
            });
        });
}

// ---------------------------------------------------------------------------
// Tree panel header
// ---------------------------------------------------------------------------

fn render_tree_panel_header(ui: &mut egui::Ui, state: &mut AppState) {
    let frame = egui::Frame::new()
        .fill(theme::BG_DARKEST)
        .inner_margin(Margin::symmetric(theme::SPACE_MD as i8, theme::SPACE_SM as i8))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} Explorer", icons::DATABASE))
                    .color(theme::TEXT_SECONDARY)
                    .size(11.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn = egui::Button::new(
                    RichText::new(format!("{} New", icons::PLUS))
                        .color(theme::TEXT_PRIMARY)
                        .size(11.0),
                )
                .fill(theme::BG_LIGHT)
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                .corner_radius(CornerRadius::same(theme::RADIUS_SM));

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
