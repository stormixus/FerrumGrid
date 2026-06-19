use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, ComboBox, CornerRadius, FontId, Frame, Layout,
    Margin, Rect, RichText, Sense, Stroke, StrokeKind, UiBuilder,
};

use crate::i18n::{set_language, t, Language};
use crate::state::{data_timezone_label, data_timezone_options, AppState};
use crate::storage;
use crate::ui::theme;

const PREF_MAX_WIDTH: f32 = 720.0;
const PREF_MAX_HEIGHT: f32 = 540.0;
const PREF_MIN_WIDTH: f32 = 480.0;
const PREF_MIN_HEIGHT: f32 = 360.0;
const HEADER_HEIGHT: f32 = 48.0;
const FOOTER_HEIGHT: f32 = 48.0;
const NAV_WIDTH: f32 = 200.0;
// New-pane layout constants
const SECTION_LABEL_SIZE: f32 = 11.0;
const ROW_TITLE_SIZE: f32 = 12.0;
const ROW_DESC_SIZE: f32 = 11.0;
const TOGGLE_W: f32 = 32.0;
const TOGGLE_H: f32 = 18.0;
const CHIP_H: f32 = 22.0;

fn window_bg() -> Color32 {
    theme::bg_medium()
}

fn content_bg() -> Color32 {
    theme::bg_light()
}

fn text_color() -> Color32 {
    theme::text_primary()
}

fn active_accent() -> Color32 {
    theme::accent_color()
}

#[derive(Clone, Copy, PartialEq)]
enum CloseAction {
    None,
    Cancel,
    Apply,
    #[allow(dead_code)]
    RestoreDefaults,
}

const NUM_SETTINGS_TABS: usize = 10;

const NAV_ICONS: [&str; NUM_SETTINGS_TABS] = [
    "\u{2699}",  // 0 - cog
    "\u{1D5E7}", // 1 - table-ish
    "\u{25A6}",  // 2 - grid
    "\u{1F310}", // 3 - globe
    "\u{1F512}", // 4 - lock/vault
    "\u{2B07}",  // 5 - download
    "\u{1F9E0}", // 6 - brain
    "\u{1F4CA}", // 7 - chart
    "\u{1F30D}", // 8 - globe
    "\u{1F504}", // 9 - refresh
];

const NAV_LABEL_KEYS: [&str; NUM_SETTINGS_TABS] = [
    "settings_nav_general",
    "settings_nav_editor",
    "settings_nav_data_grid",
    "settings_nav_connections",
    "settings_nav_vault",
    "settings_nav_backup",
    "settings_nav_ai",
    "settings_nav_diagnostics",
    "settings_nav_language",
    "settings_nav_updates",
];

pub fn render_settings_window(
    ctx: &egui::Context,
    state: &mut AppState,
    settings: &mut storage::settings::AppSettings,
) -> bool {
    if !state.show_settings_dialog {
        state.settings_draft = None;
        return false;
    }

    if state.settings_draft.is_none() {
        state.settings_draft = Some(settings.clone());
    }
    state.active_settings_tab = state.active_settings_tab.min(NUM_SETTINGS_TABS - 1);

    let mut open = true;
    let mut close_action = CloseAction::None;

    let screen = ctx.screen_rect();
    let pref_w = (screen.width() * 0.85).clamp(PREF_MIN_WIDTH, PREF_MAX_WIDTH);
    let pref_h = (screen.height() * 0.80).clamp(PREF_MIN_HEIGHT, PREF_MAX_HEIGHT);

    // Dim overlay behind settings
    let overlay_layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("settings_overlay"));
    ctx.layer_painter(overlay_layer)
        .rect_filled(screen, 0.0, Color32::from_black_alpha(115));

    egui::Window::new(t("settings_title"))
        .id(egui::Id::new("settings_preferences_window"))
        .open(&mut open)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_size(vec2(pref_w, pref_h))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Tooltip)
        .frame(
            Frame::window(&ctx.style())
                .fill(window_bg())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(0)),
        )
        .show(ctx, |ui| {
            ui.set_min_size(vec2(pref_w, pref_h));
            render_header(ui, pref_w, state, &mut close_action);

            // Sidebar + Content area
            let body_height = pref_h - HEADER_HEIGHT - FOOTER_HEIGHT;
            let (body_rect, _) =
                ui.allocate_exact_size(vec2(pref_w, body_height), Sense::hover());

            // Left: sidebar navigation
            let nav_rect = Rect::from_min_size(body_rect.left_top(), vec2(NAV_WIDTH, body_height));
            ui.allocate_new_ui(
                UiBuilder::new()
                    .max_rect(nav_rect)
                    .layout(Layout::top_down(Align::LEFT)),
                |ui| {
                    render_sidebar(ui, state);
                },
            );

            // Separator
            ui.painter().vline(
                nav_rect.right(),
                body_rect.y_range(),
                Stroke::new(1.0, theme::border_subtle()),
            );

            // Right: content pane
            let content_rect = Rect::from_min_max(
                pos2(nav_rect.right() + 1.0, body_rect.top()),
                body_rect.right_bottom(),
            );
            ui.allocate_new_ui(
                UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(Layout::top_down(Align::LEFT)),
                |ui| {
                    let active_tab = state.active_settings_tab;
                    if let Some(draft) = state.settings_draft.as_mut() {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_min_width(content_rect.width() - 16.0);
                                Frame::new()
                                    .inner_margin(Margin::symmetric(22, 18))
                                    .fill(theme::bg_shell())
                                    .show(ui, |ui| {
                                        render_content(ui, active_tab, draft);
                                    });
                            });
                    }
                },
            );

            render_footer(ui, &mut close_action);
        });

    preview_draft_appearance(ctx, state);

    if !open && close_action == CloseAction::None {
        close_action = CloseAction::Cancel;
    }

    match close_action {
        CloseAction::None => false,
        CloseAction::Cancel => {
            settings.dark_mode = theme::apply_appearance(ctx, &settings.appearance, &settings.accent_color);
            set_language(Language::from_code(&settings.language));
            state.settings_draft = None;
            state.show_settings_dialog = false;
            ctx.request_repaint();
            false
        }
        CloseAction::RestoreDefaults => {
            let mut defaults = storage::settings::AppSettings::default();
            defaults.dark_mode = theme::apply_appearance(ctx, &defaults.appearance, &defaults.accent_color);
            state.settings_draft = Some(defaults);
            ctx.request_repaint();
            false
        }
        CloseAction::Apply => {
            let old_language = settings.language.clone();
            let mut next = state
                .settings_draft
                .take()
                .unwrap_or_else(|| settings.clone());
            next.normalize();
            next.dark_mode = theme::apply_appearance(ctx, &next.appearance, &next.accent_color);
            state.default_row_limit = next.default_row_limit;
            state.data_timezone = next.data_timezone.clone();
            set_language(Language::from_code(&next.language));

            *settings = next;
            storage::settings::save_settings(settings);
            state.status_message = t("settings_saved");
            state.show_settings_dialog = false;

            old_language != settings.language
        }
    }
}

fn preview_draft_appearance(ctx: &egui::Context, state: &mut AppState) {
    let Some(draft) = state.settings_draft.as_mut() else {
        return;
    };

    let prev_accent = theme::accent_color_name().to_string();
    let preview_dark_mode = theme::apply_appearance(ctx, &draft.appearance, &draft.accent_color);
    if draft.dark_mode != preview_dark_mode || prev_accent != draft.accent_color {
        draft.dark_mode = preview_dark_mode;
        ctx.request_repaint();
    }

    set_language(Language::from_code(&draft.language));
}

fn render_header(ui: &mut egui::Ui, width: f32, _state: &mut AppState, close_action: &mut CloseAction) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, HEADER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, CornerRadius { nw: 10, ne: 10, sw: 0, se: 0 }, theme::bg_shell());
    painter.hline(
        rect.x_range(),
        rect.bottom() - 1.0,
        Stroke::new(1.0, theme::border_subtle()),
    );

    // Settings icon + title
    painter.text(
        pos2(rect.left() + 16.0, rect.center().y),
        Align2::LEFT_CENTER,
        "\u{2699}",
        FontId::proportional(14.0),
        active_accent(),
    );
    painter.text(
        pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        t("settings_header_preferences"),
        FontId::proportional(13.0),
        text_color(),
    );

    // Close button
    let close_rect = Rect::from_center_size(
        pos2(rect.right() - 24.0, rect.center().y),
        vec2(20.0, 20.0),
    );
    let close_resp = ui.interact(close_rect, ui.id().with("settings_close"), Sense::click());
    painter.text(
        close_rect.center(),
        Align2::CENTER_CENTER,
        "\u{00D7}",
        FontId::proportional(16.0),
        if close_resp.hovered() {
            theme::ACCENT_RED
        } else {
            theme::text_muted()
        },
    );
    if close_resp.clicked() {
        *close_action = CloseAction::Cancel;
    }
}

fn render_sidebar(ui: &mut egui::Ui, state: &mut AppState) {
    Frame::new()
        .fill(theme::bg_medium())
        .inner_margin(Margin::symmetric(6, 10))
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.set_min_size(vec2(NAV_WIDTH, ui.available_height()));
            ui.spacing_mut().item_spacing.y = 1.0;
            for (i, (icon, label_key)) in NAV_ICONS.iter().zip(NAV_LABEL_KEYS.iter()).enumerate() {
                let label = t(label_key);
                let active = state.active_settings_tab == i;
                let (rect, response) =
                    ui.allocate_exact_size(vec2(NAV_WIDTH - 12.0, 32.0), Sense::click());
                let bg = if active {
                    theme::with_alpha(active_accent(), 30)
                } else if response.hovered() {
                    theme::bg_light()
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), bg);
                let text_col = if active {
                    text_color()
                } else {
                    theme::text_secondary()
                };
                // Icon
                ui.painter().text(
                    pos2(rect.left() + 12.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    icon,
                    FontId::proportional(12.0),
                    if active { active_accent() } else { theme::text_muted() },
                );
                // Label
                ui.painter().text(
                    pos2(rect.left() + 32.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &label,
                    FontId::proportional(12.0),
                    text_col,
                );
                if response.clicked() {
                    state.active_settings_tab = i;
                }
            }
        });
}

fn render_content(
    ui: &mut egui::Ui,
    active_tab: usize,
    draft: &mut storage::settings::AppSettings,
) {
    let content_height = ui.available_height();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), content_height), Sense::hover());
    ui.painter_at(rect)
        .rect_filled(rect, CornerRadius::same(0), content_bg());

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(0.0, 0.0)))
            .layout(Layout::top_down(Align::LEFT)),
        |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 24.0);
                    match active_tab {
                        0 => render_general_tab(ui, draft),
                        1 => render_editor_tab(ui, draft),
                        2 => render_data_grid_tab(ui, draft),
                        3 => render_connections_tab(ui, draft),
                        4 => render_vault_tab(ui, draft),
                        5 => render_backup_tab(ui, draft),
                        6 => render_ai_assist_tab(ui, draft),
                        7 => render_diagnostics_tab(ui, draft),
                        8 => render_language_tab(ui, draft),
                        9 => render_updates_tab(ui, draft),
                        _ => {}
                    }
                });
            });
        },
    );
}

// =============================================================================
// Tab 0 — General (wired to AppSettings)
// =============================================================================

fn render_general_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    // --- Appearance section ---
    settings_section(ui, &t("settings_sec_appearance"));

    settings_row(ui, &t("settings_row_theme"), &t("settings_desc_theme"), |ui| {
        render_appearance_combo(ui, "settings_appearance_general", draft);
    });

    settings_row(ui, &t("settings_row_accent"), &t("settings_desc_accent"), |ui| {
        let swatches = [
            ("emerald", theme::ACCENT_EMERALD),
            ("blue", theme::ACCENT_BLUE),
            ("purple", theme::ACCENT_PURPLE),
            ("yellow", theme::ACCENT_YELLOW),
            ("red", theme::ACCENT_RED),
        ];
        for (name, color) in swatches.iter() {
            let (r, resp) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::click());
            ui.painter().rect_filled(r, CornerRadius::same(10), *color);
            if draft.accent_color == *name {
                ui.painter().rect_stroke(
                    r.expand(2.0),
                    CornerRadius::same(12),
                    Stroke::new(1.5, theme::text_primary()),
                    StrokeKind::Outside,
                );
            }
            if resp.clicked() {
                draft.accent_color = name.to_string();
            }
            ui.add_space(4.0);
        }
    });

    settings_row(ui, &t("settings_row_density"), &t("settings_desc_density"), |ui| {
        let density_opts = ["compact", "default", "comfortable"];
        let density_labels = [t("settings_chip_compact"), t("settings_chip_default"), t("settings_chip_comfortable")];
        let density_label_refs: Vec<&str> = density_labels.iter().map(|s| s.as_str()).collect();
        let mut sel = density_opts.iter().position(|o| *o == draft.density).unwrap_or(1);
        settings_chips(ui, "density", &density_label_refs, &mut sel);
        draft.density = density_opts[sel].to_string();
    });

    // --- Workflow section ---
    settings_section(ui, &t("settings_sec_workflow"));

    settings_row(ui, &t("settings_row_autocommit"), &t("settings_desc_autocommit"), |ui| {
        settings_toggle(ui, "general_autocommit", &mut draft.auto_commit);
    });

    settings_row(ui, &t("settings_row_warn_dangling"), &t("settings_desc_warn_dangling"), |ui| {
        settings_toggle(ui, "general_warn_dangling", &mut draft.warn_dangling_tx);
    });

    settings_row(ui, &t("settings_row_confirm_drop"), &t("settings_desc_confirm_drop"), |ui| {
        settings_toggle(ui, "general_confirm_drop", &mut draft.confirm_destructive);
    });

    settings_row(ui, &t("settings_row_result_limit"), &t("settings_desc_result_limit"), |ui| {
        let limits = ["100", "500", "1 000", "5 000", "10 000", "50 000"];
        let limit_vals: [usize; 6] = [100, 500, 1_000, 5_000, 10_000, 50_000];
        let current_label = match draft.default_row_limit {
            100 => "100",
            500 => "500",
            1_000 => "1 000",
            5_000 => "5 000",
            10_000 => "10 000",
            50_000 => "50 000",
            _ => "1 000",
        };
        ComboBox::from_id_salt("general_row_limit")
            .width(120.0)
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for (lbl, val) in limits.iter().zip(limit_vals.iter()) {
                    ui.selectable_value(&mut draft.default_row_limit, *val, *lbl);
                }
            });
    });

    // --- Startup section ---
    settings_section(ui, &t("settings_sec_startup"));

    settings_row(ui, &t("settings_row_reopen_tabs"), &t("settings_desc_reopen_tabs"), |ui| {
        settings_toggle(ui, "general_reopen", &mut draft.reopen_tabs);
    });

    settings_row(ui, &t("settings_row_autoconnect"), &t("settings_desc_autoconnect"), |ui| {
        settings_toggle(ui, "general_autoconnect", &mut draft.auto_connect_vault);
    });
}

// =============================================================================
// Tab 1 — Editor (wired to AppSettings)
// =============================================================================

fn render_editor_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    // --- Font section ---
    settings_section(ui, &t("settings_sec_font"));

    settings_row(ui, &t("settings_row_family"), &t("settings_desc_family"), |ui| {
        let families = ["SF Mono", "JetBrains Mono", "Fira Code", "Menlo", "Monaco"];
        ComboBox::from_id_salt("editor_font_family")
            .width(160.0)
            .selected_text(&draft.font_family)
            .show_ui(ui, |ui| {
                for f in families.iter() {
                    ui.selectable_value(&mut draft.font_family, f.to_string(), *f);
                }
            });
    });

    settings_row(ui, &t("settings_row_size"), &t("settings_desc_size"), |ui| {
        let sizes = ["10", "11", "12", "13", "14", "16", "18"];
        let size_vals: [f32; 7] = [10.0, 11.0, 12.0, 13.0, 14.0, 16.0, 18.0];
        let current = format!("{}", draft.font_size as i32);
        ComboBox::from_id_salt("editor_font_size")
            .width(80.0)
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for (lbl, val) in sizes.iter().zip(size_vals.iter()) {
                    ui.selectable_value(&mut draft.font_size, *val, *lbl);
                }
            });
    });

    settings_row(ui, &t("settings_row_ligatures"), &t("settings_desc_ligatures"), |ui| {
        settings_toggle(ui, "editor_ligatures", &mut draft.font_ligatures);
    });

    // --- Editing section ---
    settings_section(ui, &t("settings_sec_editing"));

    settings_row(ui, &t("settings_row_autocomplete"), &t("settings_desc_autocomplete"), |ui| {
        settings_toggle(ui, "editor_autocomplete", &mut draft.enable_code_completion);
    });

    settings_row(ui, &t("settings_row_format_save"), &t("settings_desc_format_save"), |ui| {
        settings_toggle(ui, "editor_format_save", &mut draft.format_on_save);
    });

    settings_row(ui, &t("settings_row_tab_size"), &t("settings_desc_tab_size"), |ui| {
        let tab_opts: [usize; 3] = [2, 4, 8];
        let current_label = format!("{}", draft.tab_size);
        ComboBox::from_id_salt("editor_tab_size")
            .width(80.0)
            .selected_text(&current_label)
            .show_ui(ui, |ui| {
                for val in tab_opts.iter() {
                    ui.selectable_value(&mut draft.tab_size, *val, format!("{}", val));
                }
            });
    });

    settings_row(ui, &t("settings_row_show_ws"), &t("settings_desc_show_ws"), |ui| {
        settings_toggle(ui, "editor_show_ws", &mut draft.show_whitespace);
    });

    settings_row(ui, &t("settings_row_word_wrap"), &t("settings_desc_word_wrap"), |ui| {
        settings_toggle(ui, "editor_word_wrap", &mut draft.word_wrap);
    });

    // --- AI Inline section ---
    settings_section(ui, &t("settings_sec_ai_inline"));

    settings_row(ui, &t("settings_row_suggest_type"), &t("settings_desc_suggest_type"), |ui| {
        settings_toggle(ui, "editor_ai_type", &mut draft.ai_suggest_inline);
    });

    settings_row(ui, &t("settings_row_suggest_hold"), &t("settings_desc_suggest_hold"), |ui| {
        settings_toggle(ui, "editor_ai_hold", &mut draft.ai_suggest_on_hold);
    });
}

// =============================================================================
// Tab 2 — Data Grid (replaces old Records tab, keeps wired settings)
// =============================================================================

fn render_data_grid_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    // --- Display section ---
    settings_section(ui, &t("settings_sec_display"));

    settings_row(ui, &t("settings_row_row_height"), &t("settings_desc_row_height"), |ui| {
        let rh_opts = ["24px", "28px", "32px"];
        let rh_labels = [t("settings_chip_short"), t("settings_chip_default"), t("settings_chip_tall")];
        let rh_label_refs: Vec<&str> = rh_labels.iter().map(|s| s.as_str()).collect();
        let mut sel = rh_opts.iter().position(|o| *o == draft.grid_row_height).unwrap_or(1);
        settings_chips(ui, "grid_row_height", &rh_label_refs, &mut sel);
        draft.grid_row_height = rh_opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_show_rownum"), &t("settings_desc_show_rownum"), |ui| {
        settings_toggle(ui, "grid_show_rownum", &mut draft.show_line_numbers);
    });

    settings_row(ui, &t("settings_row_color_null"), &t("settings_desc_color_null"), |ui| {
        settings_toggle(ui, "grid_color_null", &mut draft.color_null_cells);
    });

    settings_row(ui, &t("settings_row_color_fk"), &t("settings_desc_color_fk"), |ui| {
        settings_toggle(ui, "grid_color_fk", &mut draft.color_fk_cells);
    });

    settings_row(ui, &t("settings_row_tabular_nums"), &t("settings_desc_tabular_nums"), |ui| {
        settings_toggle(ui, "grid_tabular", &mut draft.tabular_numbers);
    });

    // --- Editing section ---
    settings_section(ui, &t("settings_sec_editing"));

    settings_row(ui, &t("settings_row_edit_dblclick"), &t("settings_desc_edit_dblclick"), |ui| {
        settings_toggle(ui, "grid_edit_dbl", &mut draft.edit_on_double_click);
    });

    settings_row(ui, &t("settings_row_autocommit_cells"), &t("settings_desc_autocommit_cells"), |ui| {
        settings_toggle(ui, "grid_auto_commit", &mut draft.auto_commit_cells);
    });

    settings_row(ui, &t("settings_row_confirm_bulk"), &t("settings_desc_confirm_bulk"), |ui| {
        settings_toggle(ui, "grid_confirm_bulk", &mut draft.confirm_bulk_delete);
    });

    // --- Truncation section ---
    settings_section(ui, &t("settings_sec_truncation"));

    settings_row(ui, &t("settings_row_long_text"), &t("settings_desc_long_text"), |ui| {
        let preview_opts = ["64 chars", "128 chars", "160 chars", "256 chars", "512 chars"];
        ComboBox::from_id_salt("grid_text_preview")
            .width(120.0)
            .selected_text(&draft.long_text_preview)
            .show_ui(ui, |ui| {
                for o in preview_opts.iter() {
                    ui.selectable_value(&mut draft.long_text_preview, o.to_string(), *o);
                }
            });
    });

    settings_row(ui, &t("settings_row_json_cells"), &t("settings_desc_json_cells"), |ui| {
        let json_opts = ["Single-line", "Collapsed", "Pretty"];
        let mut sel = json_opts.iter().position(|o| *o == draft.json_cell_display).unwrap_or(0);
        settings_chips(ui, "grid_json", &json_opts, &mut sel);
        draft.json_cell_display = json_opts[sel].to_string();
    });

    // --- Existing wired settings from old Records tab ---
    settings_section(ui, &t("settings_sec_query_defaults"));

    settings_row(ui, &t("settings_row_default_limit"), &t("settings_desc_default_limit"), |ui| {
        let mut row_limit = draft.default_row_limit as i64;
        if ui
            .add(
                egui::DragValue::new(&mut row_limit)
                    .range(1..=1_000_000)
                    .speed(100)
                    .max_decimals(0),
            )
            .changed()
        {
            draft.default_row_limit = row_limit.clamp(1, 1_000_000) as usize;
        }
    });

    settings_row(ui, &t("settings_row_data_tz"), &t("settings_desc_data_tz"), |ui| {
        ComboBox::from_id_salt("settings_data_timezone_combo")
            .width(200.0)
            .selected_text(data_timezone_label(&draft.data_timezone))
            .show_ui(ui, |ui| {
                for (code, label) in data_timezone_options() {
                    ui.selectable_value(&mut draft.data_timezone, (*code).to_string(), *label);
                }
            });
    });
}

// =============================================================================
// Tab 3 — Connections (shell)
// =============================================================================

fn render_connections_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_pool"));

    settings_row(ui, &t("settings_row_min_conn"), &t("settings_desc_min_conn"), |ui| {
        let pool_opts: [usize; 4] = [1, 2, 5, 10];
        let current_label = format!("{}", draft.pool_min);
        ComboBox::from_id_salt("conn_min")
            .width(80.0)
            .selected_text(&current_label)
            .show_ui(ui, |ui| {
                for val in pool_opts.iter() {
                    ui.selectable_value(&mut draft.pool_min, *val, format!("{}", val));
                }
            });
    });

    settings_row(ui, &t("settings_row_max_conn"), &t("settings_desc_max_conn"), |ui| {
        let pool_opts: [usize; 4] = [5, 10, 20, 50];
        let current_label = format!("{}", draft.pool_max);
        ComboBox::from_id_salt("conn_max")
            .width(80.0)
            .selected_text(&current_label)
            .show_ui(ui, |ui| {
                for val in pool_opts.iter() {
                    ui.selectable_value(&mut draft.pool_max, *val, format!("{}", val));
                }
            });
    });

    settings_row(ui, &t("settings_row_idle_timeout"), &t("settings_desc_idle_timeout"), |ui| {
        let idle_opts = ["30s", "1m", "2m", "5m", "15m"];
        let idle_labels = ["30 s", "1 min", "2 min", "5 min", "15 min"];
        ComboBox::from_id_salt("conn_idle")
            .width(100.0)
            .selected_text(&draft.idle_timeout)
            .show_ui(ui, |ui| {
                for (val, lbl) in idle_opts.iter().zip(idle_labels.iter()) {
                    ui.selectable_value(&mut draft.idle_timeout, val.to_string(), *lbl);
                }
            });
    });

    settings_section(ui, &t("settings_sec_defaults"));

    settings_row(ui, &t("settings_row_ssl"), &t("settings_desc_ssl"), |ui| {
        let ssl_opts = ["disable", "prefer", "require", "verify-full"];
        let ssl_labels = ["Disable", "Prefer", "Require", "Verify-Full"];
        let mut sel = ssl_opts.iter().position(|o| *o == draft.ssl_mode).unwrap_or(2);
        settings_chips(ui, "conn_ssl", &ssl_labels, &mut sel);
        draft.ssl_mode = ssl_opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_stmt_timeout"), &t("settings_desc_stmt_timeout"), |ui| {
        let stmt_opts = ["5s", "15s", "30s", "1m", "none"];
        let stmt_labels = ["5 s", "15 s", "30 s", "1 min", "None"];
        ComboBox::from_id_salt("conn_stmt_timeout")
            .width(100.0)
            .selected_text(&draft.statement_timeout)
            .show_ui(ui, |ui| {
                for (val, lbl) in stmt_opts.iter().zip(stmt_labels.iter()) {
                    ui.selectable_value(&mut draft.statement_timeout, val.to_string(), *lbl);
                }
            });
    });

    settings_row(ui, &t("settings_row_lock_timeout"), &t("settings_desc_lock_timeout"), |ui| {
        let lock_opts = ["1s", "5s", "15s", "30s"];
        let lock_labels = ["1 s", "5 s", "15 s", "30 s"];
        ComboBox::from_id_salt("conn_lock_timeout")
            .width(100.0)
            .selected_text(&draft.lock_timeout)
            .show_ui(ui, |ui| {
                for (val, lbl) in lock_opts.iter().zip(lock_labels.iter()) {
                    ui.selectable_value(&mut draft.lock_timeout, val.to_string(), *lbl);
                }
            });
    });

    settings_section(ui, &t("settings_sec_replicas"));

    settings_row(ui, &t("settings_row_auto_route"), &t("settings_desc_auto_route"), |ui| {
        settings_toggle(ui, "conn_auto_route", &mut draft.auto_route_replicas);
    });

    settings_row(ui, &t("settings_row_show_lag"), &t("settings_desc_show_lag"), |ui| {
        settings_toggle(ui, "conn_show_lag", &mut draft.show_replica_lag);
    });
}

// =============================================================================
// Tab 4 — Vault & Security (shell)
// =============================================================================

fn render_vault_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_storage"));

    settings_row(ui, &t("settings_row_vault_loc"), &t("settings_desc_vault_loc"), |ui| {
        ComboBox::from_id_salt("vault_location")
            .width(160.0)
            .selected_text(&draft.vault_location)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut draft.vault_location, "~/Library/FerrumGrid/vault.db".to_string(), "~/Library/FerrumGrid/vault.db");
                ui.selectable_value(&mut draft.vault_location, "Encrypted file".to_string(), "Encrypted file");
                ui.selectable_value(&mut draft.vault_location, "Custom path".to_string(), "Custom path");
            });
    });

    settings_row(ui, &t("settings_row_master_key"), &t("settings_desc_master_key"), |ui| {
        let opts = ["Keychain", "Password", "Hardware"];
        let mut sel = opts.iter().position(|o| *o == draft.master_key_type).unwrap_or(0);
        settings_chips(ui, "vault_master_key", &opts, &mut sel);
        draft.master_key_type = opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_autolock"), &t("settings_desc_autolock"), |ui| {
        let opts = ["1m", "5m", "15m", "Never"];
        let labels = ["1 min", "5 min", "15 min", "Never"];
        ComboBox::from_id_salt("vault_autolock")
            .width(100.0)
            .selected_text(&draft.auto_lock_after)
            .show_ui(ui, |ui| {
                for (val, lbl) in opts.iter().zip(labels.iter()) {
                    ui.selectable_value(&mut draft.auto_lock_after, val.to_string(), *lbl);
                }
            });
    });

    settings_section(ui, &t("settings_sec_audit"));

    settings_row(ui, &t("settings_row_log_cred"), &t("settings_desc_log_cred"), |ui| {
        settings_toggle(ui, "vault_log_cred", &mut draft.log_credential_use);
    });

    settings_row(ui, &t("settings_row_redact_ss"), &t("settings_desc_redact_ss"), |ui| {
        settings_toggle(ui, "vault_redact", &mut draft.redact_screenshots);
    });

    settings_row(ui, &t("settings_row_block_clip"), &t("settings_desc_block_clip"), |ui| {
        settings_toggle(ui, "vault_block_clip", &mut draft.block_clipboard_key);
    });

    settings_section(ui, &t("settings_sec_sharing"));

    settings_row(ui, &t("settings_row_team_sync"), &t("settings_desc_team_sync"), |ui| {
        settings_toggle(ui, "vault_team_sync", &mut draft.team_vault_sync);
    });

    settings_row(ui, &t("settings_row_export_fmt"), &t("settings_desc_export_fmt"), |ui| {
        let opts = [".vault", "JSON", "CSV"];
        let mut sel = opts.iter().position(|o| *o == draft.export_format).unwrap_or(0);
        settings_chips(ui, "vault_export_fmt", &opts, &mut sel);
        draft.export_format = opts[sel].to_string();
    });
}

// =============================================================================
// Tab 5 — Backup (partially wired: backup_directory)
// =============================================================================

fn render_backup_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_auto_backup"));

    settings_row(ui, &t("settings_row_daily"), &t("settings_desc_daily"), |ui| {
        settings_toggle(ui, "backup_daily", &mut draft.daily_snapshot);
    });

    settings_row(ui, &t("settings_row_weekly"), &t("settings_desc_weekly"), |ui| {
        settings_toggle(ui, "backup_weekly", &mut draft.weekly_archive);
    });

    settings_row(ui, &t("settings_row_predeploy"), &t("settings_desc_predeploy"), |ui| {
        settings_toggle(ui, "backup_predeploy", &mut draft.pre_deploy_hook);
    });

    settings_row(ui, &t("settings_row_retention"), &t("settings_desc_retention"), |ui| {
        let opts = ["7 days", "14 days", "30 days", "90 days", "1 year"];
        ComboBox::from_id_salt("backup_retention")
            .width(100.0)
            .selected_text(&draft.backup_retention)
            .show_ui(ui, |ui| {
                for o in opts.iter() {
                    ui.selectable_value(&mut draft.backup_retention, o.to_string(), *o);
                }
            });
    });

    settings_section(ui, &t("settings_sec_storage"));

    settings_row(ui, &t("settings_row_backup_folder"), &t("settings_desc_backup_folder"), |ui| {
        let display = if draft.backup_directory.trim().is_empty() {
            "(default: ~/Documents)".to_string()
        } else {
            draft.backup_directory.clone()
        };
        ui.label(
            RichText::new(display)
                .monospace()
                .size(11.0)
                .color(theme::text_muted()),
        );
        if ui.small_button(t("settings_browse")).clicked() {
            let mut dialog = rfd::FileDialog::new();
            let initial = if draft.backup_directory.trim().is_empty() {
                std::env::home_dir()
                    .map(|h| h.join("Documents"))
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
            } else {
                std::path::PathBuf::from(&draft.backup_directory)
            };
            dialog = dialog.set_directory(initial);
            if let Some(path) = dialog.pick_folder() {
                draft.backup_directory = path.display().to_string();
            }
        }
    });

    settings_row(ui, &t("settings_row_compression"), &t("settings_desc_compression"), |ui| {
        let opts = ["none", "gzip", "zstd"];
        let mut sel = opts.iter().position(|o| *o == draft.backup_compression).unwrap_or(2);
        settings_chips(ui, "backup_compress", &opts, &mut sel);
        draft.backup_compression = opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_verify_dump"), &t("settings_desc_verify_dump"), |ui| {
        settings_toggle(ui, "backup_verify", &mut draft.verify_after_dump);
    });

    settings_section(ui, &t("settings_sec_restore"));

    settings_row(ui, &t("settings_row_restore_copy"), &t("settings_desc_restore_copy"), |ui| {
        settings_toggle(ui, "backup_restore_copy", &mut draft.always_restore_copy);
    });

    settings_row(ui, &t("settings_row_require_name"), &t("settings_desc_require_name"), |ui| {
        settings_toggle(ui, "backup_require_name", &mut draft.require_typing_name);
    });
}

// =============================================================================
// Tab 6 — AI Assist (shell)
// =============================================================================

fn render_ai_assist_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_provider"));

    settings_row(ui, &t("settings_row_backend"), &t("settings_desc_backend"), |ui| {
        let opts = ["Local", "OpenAI", "Anthropic"];
        let mut sel = opts.iter().position(|o| *o == draft.ai_backend).unwrap_or(2);
        settings_chips(ui, "ai_backend", &opts, &mut sel);
        draft.ai_backend = opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_model"), &t("settings_desc_model"), |ui| {
        ComboBox::from_id_salt("ai_model")
            .width(150.0)
            .selected_text(&draft.ai_model)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut draft.ai_model, "gpt-4o".to_string(), "GPT-4o");
                ui.selectable_value(&mut draft.ai_model, "claude-sonnet-4-5".to_string(), "Claude Sonnet");
                ui.selectable_value(&mut draft.ai_model, "claude-haiku-4-5".to_string(), "Claude Haiku");
                ui.selectable_value(&mut draft.ai_model, "local-llm".to_string(), "Local LLM");
            });
    });

    settings_row(ui, &t("settings_row_api_key"), &t("settings_desc_api_key"), |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut draft.ai_api_key)
                .password(true)
                .hint_text("sk-… / anthropic key")
                .desired_width(220.0),
        );
    });

    settings_row(ui, &t("settings_row_send_schema"), &t("settings_desc_send_schema"), |ui| {
        settings_toggle(ui, "ai_send_schema", &mut draft.ai_send_schema);
    });

    settings_row(ui, &t("settings_row_row_samples"), &t("settings_desc_row_samples"), |ui| {
        settings_toggle(ui, "ai_row_samples", &mut draft.ai_allow_row_samples);
    });

    settings_section(ui, &t("settings_sec_behavior"));

    settings_row(ui, &t("settings_row_inline_suggest"), &t("settings_desc_inline_suggest"), |ui| {
        settings_toggle(ui, "ai_inline", &mut draft.ai_suggest_inline);
    });

    settings_row(ui, &t("settings_row_explain_hover"), &t("settings_desc_explain_hover"), |ui| {
        settings_toggle(ui, "ai_explain", &mut draft.ai_explain_on_hover);
    });

    settings_row(ui, &t("settings_row_autofix"), &t("settings_desc_autofix"), |ui| {
        settings_toggle(ui, "ai_autofix", &mut draft.ai_auto_fix);
    });

    settings_row(ui, &t("settings_row_gen_test"), &t("settings_desc_gen_test"), |ui| {
        settings_toggle(ui, "ai_gen_test", &mut draft.ai_generate_test_data);
    });

    settings_section(ui, &t("settings_sec_privacy"));

    settings_row(ui, &t("settings_row_block_pii"), &t("settings_desc_block_pii"), |ui| {
        settings_toggle(ui, "ai_block_pii", &mut draft.ai_block_pii);
    });

    settings_row(ui, &t("settings_row_telemetry"), &t("settings_desc_telemetry"), |ui| {
        settings_toggle(ui, "ai_telemetry", &mut draft.ai_telemetry);
    });
}

// =============================================================================
// Tab 7 — Diagnostics (shell)
// =============================================================================

fn render_diagnostics_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_panel"));

    settings_row(ui, &t("settings_row_show_launch"), &t("settings_desc_show_launch"), |ui| {
        settings_toggle(ui, "diag_show_launch", &mut draft.diag_show_on_launch);
    });

    settings_row(ui, &t("settings_row_buf_size"), &t("settings_desc_buf_size"), |ui| {
        let buf_opts = ["256", "1,000", "2,000", "4,000", "16,000"];
        ComboBox::from_id_salt("diag_buf_size")
            .width(100.0)
            .selected_text(&draft.diag_buffer_size)
            .show_ui(ui, |ui| {
                for o in buf_opts.iter() {
                    ui.selectable_value(&mut draft.diag_buffer_size, o.to_string(), *o);
                }
            });
    });

    settings_row(ui, &t("settings_row_persist"), &t("settings_desc_persist"), |ui| {
        settings_toggle(ui, "diag_persist", &mut draft.diag_persist);
    });

    settings_section(ui, &t("settings_sec_performance"));

    settings_row(ui, &t("settings_row_slow_query"), &t("settings_desc_slow_query"), |ui| {
        let slow_opts = ["100ms", "500ms", "1s", "5s"];
        let slow_labels = ["100 ms", "500 ms", "1 s", "5 s"];
        ComboBox::from_id_salt("diag_slow_query")
            .width(100.0)
            .selected_text(&draft.slow_query_threshold)
            .show_ui(ui, |ui| {
                for (val, lbl) in slow_opts.iter().zip(slow_labels.iter()) {
                    ui.selectable_value(&mut draft.slow_query_threshold, val.to_string(), *lbl);
                }
            });
    });

    settings_row(ui, &t("settings_row_render_budget"), &t("settings_desc_render_budget"), |ui| {
        let budget_opts = ["8ms", "16ms", "32ms"];
        let budget_labels = ["8 ms", "16 ms", "32 ms"];
        ComboBox::from_id_salt("diag_render_budget")
            .width(100.0)
            .selected_text(&draft.render_budget_warn)
            .show_ui(ui, |ui| {
                for (val, lbl) in budget_opts.iter().zip(budget_labels.iter()) {
                    ui.selectable_value(&mut draft.render_budget_warn, val.to_string(), *lbl);
                }
            });
    });

    settings_row(ui, &t("settings_row_track_ctid"), &t("settings_desc_track_ctid"), |ui| {
        settings_toggle(ui, "diag_track_ctid", &mut draft.track_ctid_conflicts);
    });
}

// =============================================================================
// Tab 8 — Language & i18n (partially wired: language)
// =============================================================================

fn render_language_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_language"));

    settings_row(ui, &t("settings_row_ui_lang"), &t("settings_desc_ui_lang"), |ui| {
        render_language_combo(ui, "settings_language_general", draft);
    });

    settings_row(ui, &t("settings_row_date_fmt"), &t("settings_desc_date_fmt"), |ui| {
        let opts = ["YYYY-MM-DD", "MM/DD/YYYY", "DD.MM.YYYY"];
        let mut sel = opts.iter().position(|o| *o == draft.date_format).unwrap_or(0);
        settings_chips(ui, "lang_date_fmt", &opts, &mut sel);
        draft.date_format = opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_time_fmt"), &t("settings_desc_time_fmt"), |ui| {
        let opts = ["12-hour", "24-hour"];
        let labels = ["12h", "24h"];
        let mut sel = opts.iter().position(|o| *o == draft.time_format).unwrap_or(1);
        settings_chips(ui, "lang_time_fmt", &labels, &mut sel);
        draft.time_format = opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_num_fmt"), &t("settings_desc_num_fmt"), |ui| {
        let opts = ["1,234.56", "1.234,56", "1 234.56"];
        let mut sel = opts.iter().position(|o| *o == draft.number_format).unwrap_or(0);
        settings_chips(ui, "lang_num_fmt", &opts, &mut sel);
        draft.number_format = opts[sel].to_string();
    });

    settings_section(ui, &t("settings_sec_database"));

    settings_row(ui, &t("settings_row_encoding"), &t("settings_desc_encoding"), |ui| {
        let opts = ["UTF8", "LATIN1", "EUC_KR", "SJIS"];
        ComboBox::from_id_salt("lang_encoding")
            .width(120.0)
            .selected_text(&draft.client_encoding)
            .show_ui(ui, |ui| {
                for o in opts.iter() {
                    ui.selectable_value(&mut draft.client_encoding, o.to_string(), *o);
                }
            });
    });

    settings_row(ui, &t("settings_row_unknown_enc"), &t("settings_desc_unknown_enc"), |ui| {
        let opts = ["UTF-8 (replace)", "UTF-8 (ignore)", "Error"];
        let mut sel = opts.iter().position(|o| *o == draft.unknown_encoding).unwrap_or(0);
        settings_chips(ui, "lang_unknown_enc", &opts, &mut sel);
        draft.unknown_encoding = opts[sel].to_string();
    });
}

// =============================================================================
// Tab 9 — Updates (shell)
// =============================================================================

fn render_updates_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    settings_section(ui, &t("settings_sec_channel"));

    settings_row(ui, &t("settings_row_update_channel"), &t("settings_desc_update_channel"), |ui| {
        let channel_opts = ["Stable", "Beta", "Nightly"];
        let mut sel = channel_opts.iter().position(|o| *o == draft.update_channel).unwrap_or(0);
        settings_chips(ui, "updates_channel", &channel_opts, &mut sel);
        draft.update_channel = channel_opts[sel].to_string();
    });

    settings_row(ui, &t("settings_row_check_freq"), &t("settings_desc_check_freq"), |ui| {
        let freq_opts = ["Every launch", "Daily", "Weekly", "Never"];
        ComboBox::from_id_salt("updates_check_freq")
            .width(130.0)
            .selected_text(&draft.check_frequency)
            .show_ui(ui, |ui| {
                for o in freq_opts.iter() {
                    ui.selectable_value(&mut draft.check_frequency, o.to_string(), *o);
                }
            });
    });

    settings_row(ui, &t("settings_row_auto_install"), &t("settings_desc_auto_install"), |ui| {
        settings_toggle(ui, "updates_auto_install", &mut draft.auto_install_updates);
    });

    settings_section(ui, &t("settings_sec_status"));

    settings_row(ui, &t("settings_row_version"), "", |ui| {
        ui.label(
            RichText::new(env!("CARGO_PKG_VERSION"))
                .monospace()
                .size(12.0)
                .color(theme::text_primary()),
        );
        let (badge_rect, _) = ui.allocate_exact_size(vec2(60.0, 20.0), Sense::hover());
        ui.painter().rect_filled(
            badge_rect,
            CornerRadius::same(10),
            theme::with_alpha(theme::accent_color(), 40),
        );
        ui.painter().text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            t("settings_badge_up_to_date"),
            FontId::proportional(10.0),
            theme::accent_color(),
        );
    });
}

// =============================================================================
// Footer
// =============================================================================

fn render_footer(ui: &mut egui::Ui, close_action: &mut CloseAction) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), FOOTER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        CornerRadius { nw: 0, ne: 0, sw: 10, se: 10 },
        theme::bg_medium(),
    );
    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, theme::border_subtle()),
    );

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(16.0, 8.0)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.label(
                RichText::new(t("settings_footer_sync"))
                    .color(theme::text_muted())
                    .monospace()
                    .size(11.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(theme::primary_button(&t("settings_btn_apply"))).clicked() {
                    *close_action = CloseAction::Apply;
                }
                if ui.add(theme::secondary_button(&t("settings_btn_cancel"))).clicked() {
                    *close_action = CloseAction::Cancel;
                }
            });
        },
    );
}

// =============================================================================
// Shared helper: section header
// =============================================================================

fn settings_section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .monospace()
                .size(SECTION_LABEL_SIZE)
                .color(theme::text_muted()),
        );
    });
    // Bottom border line
    let rect = ui.available_rect_before_wrap();
    let y = rect.top() + 2.0;
    ui.painter().hline(
        rect.left()..=rect.right(),
        y,
        Stroke::new(1.0, theme::border_subtle()),
    );
    ui.add_space(8.0);
}

// =============================================================================
// Shared helper: settings row (title + description on left, control on right)
// =============================================================================

fn settings_row(
    ui: &mut egui::Ui,
    title: &str,
    desc: &str,
    control: impl FnOnce(&mut egui::Ui),
) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        // Left side: title + description
        ui.allocate_ui_with_layout(
            vec2(ui.available_width() * 0.5, 32.0),
            Layout::top_down(Align::LEFT),
            |ui| {
                ui.label(
                    RichText::new(title)
                        .size(ROW_TITLE_SIZE)
                        .color(theme::text_primary()),
                );
                if !desc.is_empty() {
                    ui.label(
                        RichText::new(desc)
                            .size(ROW_DESC_SIZE)
                            .color(theme::text_muted()),
                    );
                }
            },
        );

        // Right side: control widget — pushed to the right
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            control(ui);
        });
    });
    ui.add_space(4.0);
}

// =============================================================================
// Shared helper: toggle switch (32x18 pill)
// =============================================================================

fn settings_toggle(ui: &mut egui::Ui, id: &str, value: &mut bool) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(TOGGLE_W, TOGGLE_H), Sense::click());
    if response.clicked() {
        *value = !*value;
    }

    let anim_t = ui.ctx().animate_bool_with_time(
        egui::Id::new(id),
        *value,
        0.15,
    );

    let bg_color = if *value {
        theme::accent_color()
    } else {
        theme::bg_light()
    };
    let border_color = if *value {
        theme::accent_color()
    } else {
        theme::border_default()
    };

    ui.painter().rect_filled(
        rect,
        CornerRadius::same(9),
        bg_color,
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(9),
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );

    // Knob
    let knob_r = (TOGGLE_H - 6.0) / 2.0;
    let knob_x = rect.left() + 3.0 + knob_r + anim_t * (TOGGLE_W - 6.0 - knob_r * 2.0);
    ui.painter().circle_filled(
        pos2(knob_x, rect.center().y),
        knob_r,
        Color32::WHITE,
    );
}

// =============================================================================
// Shared helper: chips row (horizontal pill buttons)
// =============================================================================

fn settings_chips(ui: &mut egui::Ui, id: &str, options: &[&str], selected: &mut usize) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, label) in options.iter().enumerate() {
            let is_selected = *selected == i;
            let bg = if is_selected {
                theme::with_alpha(theme::accent_color(), 50)
            } else {
                theme::bg_light()
            };
            let text_col = if is_selected {
                theme::accent_color()
            } else {
                theme::text_secondary()
            };
            let border = if is_selected {
                theme::accent_color()
            } else {
                theme::border_default()
            };

            let text = RichText::new(*label).size(11.0).color(text_col);
            let btn = egui::Button::new(text)
                .fill(bg)
                .stroke(Stroke::new(1.0, border))
                .corner_radius(CornerRadius::same(CHIP_H as u8 / 2))
                .min_size(vec2(0.0, CHIP_H));
            if ui.add(btn).clicked() {
                *selected = i;
            }
        }
    });
    let _ = id; // id reserved for future persistence
}

// =============================================================================
// Legacy helpers (kept for backward compat within the existing wired tabs)
// =============================================================================

fn render_appearance_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    draft: &mut storage::settings::AppSettings,
) {
    ComboBox::from_id_salt(id)
        .width(150.0)
        .selected_text(appearance_label(&draft.appearance))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut draft.appearance,
                "system".to_string(),
                t("settings_appearance_system"),
            );
            ui.selectable_value(
                &mut draft.appearance,
                "dark".to_string(),
                t("settings_appearance_dark"),
            );
            ui.selectable_value(
                &mut draft.appearance,
                "light".to_string(),
                t("settings_appearance_light"),
            );
        });
}

fn appearance_label(appearance: &str) -> String {
    match appearance {
        "dark" => t("settings_appearance_dark"),
        "light" => t("settings_appearance_light"),
        _ => t("settings_appearance_system"),
    }
}

fn render_language_combo(
    ui: &mut egui::Ui,
    id: &'static str,
    draft: &mut storage::settings::AppSettings,
) {
    let mut selected = Language::from_code(&draft.language);
    ComboBox::from_id_salt(id)
        .width(190.0)
        .selected_text(format!("{} ({})", selected.name(), selected.code()))
        .show_ui(ui, |ui| {
            for lang in Language::all() {
                ui.selectable_value(
                    &mut selected,
                    lang,
                    format!("{} ({})", lang.name(), lang.code()),
                );
            }
        });
    draft.language = selected.code().to_string();
}

/// 커스텀 단축키 / 백업 스케줄 입력 UI (간단한 텍스트 편집).
pub fn render_custom_settings(ui: &mut egui::Ui, settings: &mut crate::storage::settings::AppSettings) {
    ui.label("Custom shortcuts (format: \"action=Cmd+K\")");
    let mut buf = settings
        .custom_shortcuts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");
    if ui.add(egui::TextEdit::multiline(&mut buf).desired_rows(4)).changed() {
        settings.custom_shortcuts.clear();
        for line in buf.lines() {
            if let Some((k, v)) = line.split_once('=') {
                settings.custom_shortcuts.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    ui.add_space(8.0);
    ui.label("Backup schedule (cron expression, empty = off)");
    ui.add(egui::TextEdit::singleline(&mut settings.backup_schedule_cron));
}
