use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, ComboBox, CornerRadius, FontId, Frame, Layout,
    Margin, Rect, RichText, Sense, Stroke, StrokeKind, UiBuilder,
};

use crate::i18n::{set_language, t, Language};
use crate::state::{data_timezone_label, data_timezone_options, AppState};
use crate::storage;
use crate::ui::theme;

const PREF_WIDTH: f32 = 640.0;
const PREF_HEIGHT: f32 = 440.0;
const HEADER_HEIGHT: f32 = 84.0;
const FOOTER_HEIGHT: f32 = 56.0;
const TAB_WIDTH: f32 = 84.0;
const TAB_HEIGHT: f32 = 58.0;
const LABEL_WIDTH: f32 = 210.0;
const CONTROL_GAP: f32 = 10.0;

fn window_bg() -> Color32 {
    theme::bg_medium()
}

fn header_bg() -> Color32 {
    theme::bg_dark()
}

fn content_bg() -> Color32 {
    theme::bg_light()
}

fn footer_bg() -> Color32 {
    theme::bg_dark()
}

fn text_color() -> Color32 {
    theme::text_primary()
}

fn text_soft() -> Color32 {
    theme::text_secondary()
}

fn active_accent() -> Color32 {
    theme::ACCENT_EMERALD
}

fn tab_active_bg() -> Color32 {
    theme::accent_copper_dim()
}

fn tab_hover_bg() -> Color32 {
    theme::bg_medium()
}

fn inactive_tab_text() -> Color32 {
    theme::text_muted()
}

#[derive(Clone, Copy, PartialEq)]
enum CloseAction {
    None,
    Cancel,
    Apply,
    RestoreDefaults,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum PrefIcon {
    General,
    Tabs,
    Code,
    Editor,
    Records,
    Recovery,
    Ai,
    Environment,
    Advanced,
}

#[derive(Clone, Copy)]
struct TabSpec {
    label_key: &'static str,
    icon: PrefIcon,
}

const TABS: [TabSpec; 3] = [
    TabSpec {
        label_key: "settings_tab_general",
        icon: PrefIcon::General,
    },
    TabSpec {
        label_key: "settings_tab_editor",
        icon: PrefIcon::Editor,
    },
    TabSpec {
        label_key: "settings_tab_records",
        icon: PrefIcon::Records,
    },
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
    state.active_settings_tab = state.active_settings_tab.min(TABS.len() - 1);

    let mut open = true;
    let mut close_action = CloseAction::None;

    egui::Window::new(t("settings_title"))
        .id(egui::Id::new("settings_preferences_window"))
        .open(&mut open)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_size(vec2(PREF_WIDTH, PREF_HEIGHT))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(window_bg())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(0)),
        )
        .show(ctx, |ui| {
            ui.set_min_size(vec2(PREF_WIDTH, PREF_HEIGHT));
            render_header(ui, state, &mut close_action);

            let active_tab = state.active_settings_tab;
            if let Some(draft) = state.settings_draft.as_mut() {
                render_content(ui, active_tab, draft);
            }

            render_footer(ui, &mut close_action);
        });

    preview_draft_appearance(ctx, state);

    if !open && close_action == CloseAction::None {
        close_action = CloseAction::Cancel;
    }

    match close_action {
        CloseAction::None => false,
        CloseAction::Cancel => {
            settings.dark_mode = theme::apply_appearance(ctx, &settings.appearance);
            set_language(Language::from_code(&settings.language));
            state.settings_draft = None;
            state.show_settings_dialog = false;
            false
        }
        CloseAction::RestoreDefaults => {
            let mut defaults = storage::settings::AppSettings::default();
            defaults.dark_mode = theme::apply_appearance(ctx, &defaults.appearance);
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
            next.dark_mode = theme::apply_appearance(ctx, &next.appearance);
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

    let preview_dark_mode = theme::apply_appearance(ctx, &draft.appearance);
    if draft.dark_mode != preview_dark_mode {
        draft.dark_mode = preview_dark_mode;
        ctx.request_repaint();
    }

    set_language(Language::from_code(&draft.language));
}

fn render_header(ui: &mut egui::Ui, state: &mut AppState, close_action: &mut CloseAction) {
    let (rect, _) = ui.allocate_exact_size(vec2(PREF_WIDTH, HEADER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, CornerRadius::same(10), header_bg());
    painter.rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.bottom() - 8.0), rect.right_bottom()),
        CornerRadius::same(0),
        header_bg(),
    );
    painter.hline(
        rect.x_range(),
        rect.bottom() - 1.0,
        Stroke::new(1.0, theme::border_subtle()),
    );

    render_traffic_lights(
        ui,
        &painter,
        rect.left_top() + vec2(18.0, 14.0),
        close_action,
    );
    painter.text(
        pos2(rect.left() + 84.0, rect.top() + 15.0),
        Align2::LEFT_CENTER,
        t(TABS[state.active_settings_tab].label_key),
        FontId::proportional(12.0),
        text_soft(),
    );

    let tabs_rect = Rect::from_min_max(
        pos2(rect.left() + 8.0, rect.top() + 24.0),
        pos2(rect.right() - 8.0, rect.bottom() - 2.0),
    );
    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(tabs_rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
            for (idx, tab) in TABS.iter().enumerate() {
                render_tab(ui, idx, *tab, state);
            }
        },
    );
}

fn render_tab(ui: &mut egui::Ui, idx: usize, tab: TabSpec, state: &mut AppState) {
    let (rect, response) = ui.allocate_exact_size(vec2(TAB_WIDTH, TAB_HEIGHT), Sense::click());
    if response.clicked() {
        state.active_settings_tab = idx;
    }

    let active = state.active_settings_tab == idx;
    let painter = ui.painter_at(rect);
    let tab_rect = rect.shrink2(vec2(3.0, 1.0));
    if active {
        painter.rect_filled(tab_rect, CornerRadius::same(theme::RADIUS_LG), tab_active_bg());
        painter.rect_filled(
            Rect::from_min_max(
                pos2(tab_rect.left() + 5.0, tab_rect.bottom() - 2.0),
                pos2(tab_rect.right() - 5.0, tab_rect.bottom()),
            ),
            CornerRadius::same(1),
            active_accent(),
        );
    } else if response.hovered() {
        painter.rect_filled(tab_rect, CornerRadius::same(theme::RADIUS_LG), tab_hover_bg());
    }

    let color = if active {
        active_accent()
    } else {
        inactive_tab_text()
    };
    paint_pref_icon(
        &painter,
        tab.icon,
        pos2(rect.center().x, rect.top() + 18.0),
        color,
    );

    painter.text(
        pos2(rect.center().x, rect.bottom() - 9.0),
        Align2::CENTER_CENTER,
        t(tab.label_key),
        FontId::proportional(9.5),
        color,
    );
}

fn render_content(
    ui: &mut egui::Ui,
    active_tab: usize,
    draft: &mut storage::settings::AppSettings,
) {
    let content_height = PREF_HEIGHT - HEADER_HEIGHT - FOOTER_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(vec2(PREF_WIDTH, content_height), Sense::hover());
    ui.painter_at(rect)
        .rect_filled(rect, CornerRadius::same(0), content_bg());

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(0.0, 0.0)))
            .layout(Layout::top_down(Align::LEFT)),
        |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(96.0);
                ui.vertical(|ui| {
                    ui.set_width(600.0);
                    match active_tab {
                        0 => render_general_tab(ui, draft),
                        1 => render_editor_tab(ui, draft),
                        _ => render_records_tab(ui, draft),
                    }
                });
            });
        },
    );
}

fn render_general_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    form_row_sized(ui, &t("settings_appearance"), 54.0, |ui| {
        render_appearance_combo(ui, "settings_appearance_general", draft);
        ui.add_space(48.0);
        paint_theme_preview(ui);
    });
    form_row(ui, &t("settings_language"), |ui| {
        render_language_combo(ui, "settings_language_general", draft);
    });
}


fn render_editor_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_editor"));
    form_row(ui, &t("settings_font"), |ui| {
        ui.add(
            egui::Slider::new(&mut draft.font_size, 9.0..=24.0)
                .show_value(true)
                .suffix(" pt"),
        );
    });
    checkbox_row(
        ui,
        &t("settings_tab_code_completion"),
        &mut draft.enable_code_completion,
        &t("settings_enable_code_completion"),
    );
    checkbox_subrow(
        ui,
        &mut draft.code_completion_popup,
        &t("settings_completion_popup"),
    );
}

fn render_records_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_records"));
    form_row(ui, &t("settings_default_row_limit"), |ui| {
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
    form_row(ui, &t("settings_data_timezone"), |ui| {
        ComboBox::from_id_salt("settings_data_timezone_combo")
            .width(260.0)
            .selected_text(data_timezone_label(&draft.data_timezone))
            .show_ui(ui, |ui| {
                for (code, label) in data_timezone_options() {
                    ui.selectable_value(&mut draft.data_timezone, (*code).to_string(), *label);
                }
            });
    });
    checkbox_row(
        ui,
        &t("settings_database"),
        &mut draft.auto_commit,
        &t("settings_auto_commit"),
    );
    // US-J3 — Backup directory picker.
    form_row(ui, "Backup directory", |ui| {
        let display = if draft.backup_directory.trim().is_empty() {
            "(default: ~/Documents)".to_string()
        } else {
            draft.backup_directory.clone()
        };
        ui.label(
            egui::RichText::new(display)
                .monospace()
                .size(11.0)
                .color(theme::text_muted()),
        );
        if ui.small_button("Browse…").clicked() {
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
    hint(ui);
}


fn render_footer(ui: &mut egui::Ui, close_action: &mut CloseAction) {
    let (rect, _) = ui.allocate_exact_size(vec2(PREF_WIDTH, FOOTER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(0), footer_bg());
    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, theme::border_subtle()),
    );

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(14.0, 9.0)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(t("settings_restore_defaults")).color(text_color()),
                    )
                    .fill(theme::bg_light())
                    .stroke(Stroke::new(1.0, theme::border_default()))
                    .corner_radius(CornerRadius::same(theme::RADIUS_MD)),
                )
                .clicked()
            {
                *close_action = CloseAction::RestoreDefaults;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new(t("button_ok")).color(Color32::WHITE))
                            .fill(theme::BG_DARKEST)
                            .stroke(Stroke::new(1.0, active_accent()))
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD)),
                    )
                    .clicked()
                {
                    *close_action = CloseAction::Apply;
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new(t("button_cancel")).color(text_color()))
                            .fill(theme::bg_light())
                            .stroke(Stroke::new(1.0, theme::border_default()))
                            .corner_radius(CornerRadius::same(theme::RADIUS_MD)),
                    )
                    .clicked()
                {
                    *close_action = CloseAction::Cancel;
                }
            });
        },
    );
}

fn form_row(ui: &mut egui::Ui, label: &str, add_control: impl FnOnce(&mut egui::Ui)) {
    form_row_sized(ui, label, 25.0, add_control);
}

fn form_row_sized(
    ui: &mut egui::Ui,
    label: &str,
    height: f32,
    add_control: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        label_cell_sized(ui, label, height);
        ui.add_space(CONTROL_GAP);
        ui.allocate_ui_with_layout(
            vec2(360.0, height),
            Layout::left_to_right(Align::Center),
            add_control,
        );
    });
    ui.add_space(4.0);
}

fn checkbox_row(ui: &mut egui::Ui, label: &str, value: &mut bool, text: &str) {
    ui.horizontal(|ui| {
        label_cell(ui, label);
        ui.add_space(CONTROL_GAP);
        ui.checkbox(value, RichText::new(text).color(text_color()).size(12.0));
    });
    ui.add_space(5.0);
}

fn checkbox_subrow(ui: &mut egui::Ui, value: &mut bool, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + CONTROL_GAP + 22.0);
        ui.checkbox(value, RichText::new(text).color(text_color()).size(12.0));
    });
    ui.add_space(5.0);
}

fn label_cell(ui: &mut egui::Ui, label: &str) {
    label_cell_sized(ui, label, 22.0);
}

fn label_cell_sized(ui: &mut egui::Ui, label: &str, height: f32) {
    ui.allocate_ui_with_layout(
        vec2(LABEL_WIDTH, height),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.label(RichText::new(label).color(text_color()).size(12.0).strong());
        },
    );
}

fn section_heading(ui: &mut egui::Ui, title: String) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).color(text_color()).size(12.0).strong());
    ui.add_space(16.0);
}

fn hint(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + CONTROL_GAP);
        ui.label(
            RichText::new(t("settings_placeholder_hint"))
                .color(theme::text_muted())
                .size(12.0),
        );
    });
}


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

fn paint_theme_preview(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(78.0, 50.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(2), theme::bg_darkest());
    painter.rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, theme::border_strong()),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_size(rect.left_top() + vec2(4.0, 5.0), vec2(20.0, 40.0)),
        CornerRadius::same(1),
        theme::bg_dark(),
    );
    for i in 0..5 {
        let y = rect.top() + 9.0 + i as f32 * 7.0;
        painter.rect_filled(
            Rect::from_min_size(pos2(rect.left() + 8.0, y), vec2(12.0, 2.0)),
            CornerRadius::same(1),
            if i % 2 == 0 {
                active_accent()
            } else {
                theme::ACCENT_GREEN
            },
        );
    }
    let bars = [
        (32.0, 24.0, Color32::from_rgb(246, 147, 64)),
        (41.0, 32.0, Color32::from_rgb(82, 171, 255)),
        (50.0, 18.0, Color32::from_rgb(238, 207, 76)),
        (59.0, 28.0, Color32::from_rgb(93, 202, 112)),
        (68.0, 12.0, Color32::from_rgb(255, 96, 96)),
    ];
    for (x, h, color) in bars {
        painter.rect_filled(
            Rect::from_min_size(pos2(rect.left() + x, rect.bottom() - 7.0 - h), vec2(5.0, h)),
            CornerRadius::same(1),
            color,
        );
    }
}

fn render_traffic_lights(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    origin: egui::Pos2,
    close_action: &mut CloseAction,
) {
    let disabled_fill = if theme::is_dark() {
        Color32::from_rgb(74, 74, 74)
    } else {
        Color32::from_rgb(212, 212, 212)
    };
    let disabled_stroke = if theme::is_dark() {
        Color32::from_rgb(86, 86, 86)
    } else {
        Color32::from_rgb(194, 194, 194)
    };

    for idx in 0..3 {
        let center = origin + vec2(idx as f32 * 20.0, 0.0);
        let hit_rect = Rect::from_center_size(center, vec2(18.0, 18.0));

        if idx == 0 {
            let response = ui.interact(
                hit_rect,
                egui::Id::new("settings_close_traffic_light"),
                Sense::click(),
            );
            let fill = if response.hovered() {
                Color32::from_rgb(255, 112, 105)
            } else {
                Color32::from_rgb(255, 95, 87)
            };
            painter.circle_filled(center, 6.4, fill);
            painter.circle_stroke(
                center,
                6.4,
                Stroke::new(0.8, Color32::from_rgb(220, 55, 50)),
            );
            if response.clicked() {
                *close_action = CloseAction::Cancel;
            }
        } else {
            painter.circle_filled(center, 6.4, disabled_fill);
            painter.circle_stroke(center, 6.4, Stroke::new(0.8, disabled_stroke));
        }
    }
}

fn paint_pref_icon(painter: &egui::Painter, icon: PrefIcon, center: egui::Pos2, color: Color32) {
    let stroke = Stroke::new(1.9, color);
    match icon {
        PrefIcon::General => {
            painter.circle_stroke(center, 8.8, stroke);
            painter.circle_stroke(center, 2.6, stroke);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = vec2(angle.cos(), angle.sin());
                painter.line_segment([center + dir * 11.0, center + dir * 14.0], stroke);
            }
        }
        PrefIcon::Tabs => {
            let back = Rect::from_center_size(center + vec2(3.5, 2.0), vec2(20.0, 12.0));
            let front = Rect::from_center_size(center + vec2(-2.5, -2.0), vec2(20.0, 12.0));
            painter.rect_stroke(back, CornerRadius::same(2), stroke, StrokeKind::Inside);
            painter.rect_filled(front, CornerRadius::same(2), header_bg());
            painter.rect_stroke(front, CornerRadius::same(2), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    pos2(front.left() + 4.0, front.top() + 4.0),
                    pos2(front.left() + 10.0, front.top() + 4.0),
                ],
                stroke,
            );
        }
        PrefIcon::Code => {
            let top = center.y - 8.0;
            for x in [center.x - 9.0, center.x + 2.0] {
                painter.rect_stroke(
                    Rect::from_min_size(pos2(x, top), vec2(12.0, 5.0)),
                    CornerRadius::same(1),
                    stroke,
                    StrokeKind::Inside,
                );
            }
            painter.rect_stroke(
                Rect::from_min_size(pos2(center.x - 5.0, center.y + 1.0), vec2(18.0, 5.0)),
                CornerRadius::same(1),
                stroke,
                StrokeKind::Inside,
            );
            painter.line_segment(
                [center + vec2(-2.0, -3.0), center + vec2(-2.0, 1.0)],
                stroke,
            );
            painter.line_segment([center + vec2(8.0, -3.0), center + vec2(8.0, 1.0)], stroke);
        }
        PrefIcon::Editor => {
            let page = Rect::from_center_size(center, vec2(18.0, 18.0));
            painter.rect_stroke(page, CornerRadius::same(2), stroke, StrokeKind::Inside);
            for idx in 0..3 {
                let y = page.top() + 5.0 + idx as f32 * 4.0;
                painter.line_segment(
                    [pos2(page.left() + 4.0, y), pos2(page.right() - 5.0, y)],
                    stroke,
                );
            }
            painter.line_segment([center + vec2(7.0, 8.0), center + vec2(13.0, 14.0)], stroke);
        }
        PrefIcon::Records => {
            let table = Rect::from_center_size(center, vec2(20.0, 18.0));
            painter.rect_stroke(table, CornerRadius::same(1), stroke, StrokeKind::Inside);
            for idx in 1..3 {
                let y = table.top() + idx as f32 * 6.0;
                painter.line_segment([pos2(table.left(), y), pos2(table.right(), y)], stroke);
            }
            for idx in 1..3 {
                let x = table.left() + idx as f32 * 6.6;
                painter.line_segment([pos2(x, table.top()), pos2(x, table.bottom())], stroke);
            }
        }
        PrefIcon::Recovery => {
            painter.circle_stroke(center, 9.0, stroke);
            painter.line_segment(
                [center + vec2(-1.0, -9.0), center + vec2(-7.0, -9.0)],
                stroke,
            );
            painter.line_segment(
                [center + vec2(-7.0, -9.0), center + vec2(-7.0, -3.0)],
                stroke,
            );
            painter.line_segment([center, center + vec2(0.0, -6.0)], stroke);
            painter.line_segment([center, center + vec2(5.0, 2.0)], stroke);
        }
        PrefIcon::Ai => {
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "AI",
                FontId::proportional(24.0),
                color,
            );
        }
        PrefIcon::Environment => {
            let badge = Rect::from_center_size(center, vec2(27.0, 18.0));
            painter.rect_stroke(badge, CornerRadius::same(2), stroke, StrokeKind::Inside);
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "ENV",
                FontId::proportional(8.0),
                color,
            );
        }
        PrefIcon::Advanced => {
            painter.circle_stroke(center + vec2(-4.0, -3.0), 6.0, stroke);
            painter.circle_stroke(center + vec2(-4.0, -3.0), 1.8, stroke);
            painter.line_segment([center + vec2(3.0, 6.0), center + vec2(12.0, 15.0)], stroke);
            painter.line_segment([center + vec2(8.0, 6.0), center + vec2(13.0, 1.0)], stroke);
            painter.line_segment([center + vec2(9.8, 3.2), center + vec2(14.2, 7.6)], stroke);
        }
    }
}
