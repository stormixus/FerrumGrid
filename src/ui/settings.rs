use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, ComboBox, CornerRadius, FontId, Frame, Layout,
    Margin, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, UiBuilder,
};

use crate::i18n::{set_language, t, Language};
use crate::state::AppState;
use crate::storage;
use crate::ui::theme;

const PREF_WIDTH: f32 = 790.0;
const PREF_HEIGHT: f32 = 580.0;
const HEADER_HEIGHT: f32 = 84.0;
const FOOTER_HEIGHT: f32 = 56.0;
const TAB_WIDTH: f32 = 84.0;
const TAB_HEIGHT: f32 = 58.0;
const LABEL_WIDTH: f32 = 210.0;
const CONTROL_GAP: f32 = 10.0;

const WINDOW_BG: Color32 = Color32::from_rgb(31, 31, 31);
const HEADER_BG: Color32 = Color32::from_rgb(28, 28, 28);
const CONTENT_BG: Color32 = Color32::from_rgb(38, 38, 38);
const FIELD_BG: Color32 = Color32::from_rgb(48, 48, 48);
const FOOTER_BG: Color32 = Color32::from_rgb(29, 29, 29);
const BORDER: Color32 = Color32::from_rgb(58, 58, 58);
const TEXT: Color32 = Color32::from_rgb(224, 224, 224);
const TEXT_SOFT: Color32 = Color32::from_rgb(172, 172, 172);
const TEXT_MUTED: Color32 = Color32::from_rgb(116, 116, 116);
const ACTIVE_BLUE: Color32 = Color32::from_rgb(24, 130, 255);

#[derive(Clone, Copy, PartialEq)]
enum CloseAction {
    None,
    Cancel,
    Apply,
    RestoreDefaults,
}

#[derive(Clone, Copy)]
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

const TABS: [TabSpec; 9] = [
    TabSpec {
        label_key: "settings_tab_general",
        icon: PrefIcon::General,
    },
    TabSpec {
        label_key: "settings_tab_tabs",
        icon: PrefIcon::Tabs,
    },
    TabSpec {
        label_key: "settings_tab_code_completion",
        icon: PrefIcon::Code,
    },
    TabSpec {
        label_key: "settings_tab_editor",
        icon: PrefIcon::Editor,
    },
    TabSpec {
        label_key: "settings_tab_records",
        icon: PrefIcon::Records,
    },
    TabSpec {
        label_key: "settings_tab_auto_recovery",
        icon: PrefIcon::Recovery,
    },
    TabSpec {
        label_key: "settings_tab_ai",
        icon: PrefIcon::Ai,
    },
    TabSpec {
        label_key: "settings_tab_environment",
        icon: PrefIcon::Environment,
    },
    TabSpec {
        label_key: "settings_tab_advanced",
        icon: PrefIcon::Advanced,
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
                .fill(WINDOW_BG)
                .stroke(Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(0)),
        )
        .show(ctx, |ui| {
            ui.set_min_size(vec2(PREF_WIDTH, PREF_HEIGHT));
            render_header(ui, state);

            let active_tab = state.active_settings_tab;
            if let Some(draft) = state.settings_draft.as_mut() {
                render_content(ui, active_tab, draft);
            }

            render_footer(ui, &mut close_action);
        });

    if !open && close_action == CloseAction::None {
        close_action = CloseAction::Cancel;
    }

    match close_action {
        CloseAction::None => false,
        CloseAction::Cancel => {
            state.settings_draft = None;
            state.show_settings_dialog = false;
            false
        }
        CloseAction::RestoreDefaults => {
            state.settings_draft = Some(storage::settings::AppSettings::default());
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
            set_language(Language::from_code(&next.language));

            *settings = next;
            storage::settings::save_settings(settings);
            state.status_message = t("settings_saved");
            state.show_settings_dialog = false;

            old_language != settings.language
        }
    }
}

fn render_header(ui: &mut egui::Ui, state: &mut AppState) {
    let (rect, _) = ui.allocate_exact_size(vec2(PREF_WIDTH, HEADER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, CornerRadius::same(10), HEADER_BG);
    painter.rect_filled(
        Rect::from_min_max(pos2(rect.left(), rect.bottom() - 8.0), rect.right_bottom()),
        CornerRadius::same(0),
        HEADER_BG,
    );
    painter.hline(
        rect.x_range(),
        rect.bottom() - 1.0,
        Stroke::new(1.0, Color32::from_rgb(34, 34, 34)),
    );

    paint_traffic_lights(&painter, rect.left_top() + vec2(18.0, 14.0));
    painter.text(
        pos2(rect.left() + 84.0, rect.top() + 15.0),
        Align2::LEFT_CENTER,
        t(TABS[state.active_settings_tab].label_key),
        FontId::proportional(12.0),
        TEXT_SOFT,
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
        painter.rect_filled(
            tab_rect,
            CornerRadius::same(5),
            Color32::from_rgb(34, 38, 42),
        );
        painter.rect_filled(
            Rect::from_min_max(
                pos2(tab_rect.left() + 5.0, tab_rect.bottom() - 2.0),
                pos2(tab_rect.right() - 5.0, tab_rect.bottom()),
            ),
            CornerRadius::same(1),
            ACTIVE_BLUE,
        );
    } else if response.hovered() {
        painter.rect_filled(
            tab_rect,
            CornerRadius::same(5),
            Color32::from_rgb(33, 33, 33),
        );
    }

    let color = if active {
        ACTIVE_BLUE
    } else {
        Color32::from_rgb(154, 154, 154)
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
        .rect_filled(rect, CornerRadius::same(0), CONTENT_BG);

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
                        1 => render_tabs_tab(ui, draft),
                        2 => render_code_completion_tab(ui, draft),
                        3 => render_editor_tab(ui, draft),
                        4 => render_records_tab(ui, draft),
                        5 => render_auto_recovery_tab(ui, draft),
                        6 => render_ai_tab(ui, draft),
                        7 => render_environment_tab(ui, draft),
                        _ => render_advanced_tab(ui, draft),
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

    checkbox_row(
        ui,
        &t("settings_main_window"),
        &mut draft.show_objects_under_schema,
        &t("settings_show_schema_objects"),
    );
    checkbox_subrow(
        ui,
        &mut draft.show_objects_under_table,
        &t("settings_show_table_objects"),
    );

    font_panel(ui, draft);

    checkbox_row(
        ui,
        &t("settings_confirm_dialog"),
        &mut draft.safe_confirm_dialog,
        &t("settings_safe_confirm_dialog"),
    );
    checkbox_subrow(
        ui,
        &mut draft.ask_before_closing_queries,
        &t("settings_ask_close_queries"),
    );
    checkbox_subrow(
        ui,
        &mut draft.ask_before_closing_tables,
        &t("settings_ask_close_tables"),
    );

    checkbox_row(
        ui,
        &t("settings_database_items"),
        &mut draft.show_function_wizard,
        &t("settings_show_function_wizard"),
    );

    usage_data_row(ui, draft);

    checkbox_row(
        ui,
        &t("settings_update"),
        &mut draft.auto_check_updates,
        &t("settings_auto_check_updates"),
    );
    checkbox_subrow(
        ui,
        &mut draft.include_system_profile,
        &t("settings_include_system_profile"),
    );
}

fn render_tabs_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_tabs"));
    checkbox_row(
        ui,
        &t("settings_tab_tabs"),
        &mut draft.open_new_queries_in_tabs,
        &t("settings_open_queries_in_tabs"),
    );
    checkbox_subrow(
        ui,
        &mut draft.ask_before_closing_queries,
        &t("settings_ask_close_queries"),
    );
    hint(ui);
}

fn render_code_completion_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_code_completion"));
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
    hint(ui);
}

fn render_editor_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_editor"));
    checkbox_row(
        ui,
        &t("settings_tab_editor"),
        &mut draft.show_line_numbers,
        &t("settings_show_line_numbers"),
    );
    form_row(ui, &t("settings_font"), |ui| {
        ui.add(
            egui::Slider::new(&mut draft.font_size, 9.0..=24.0)
                .show_value(true)
                .suffix(" pt"),
        );
    });
    hint(ui);
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
    checkbox_row(
        ui,
        &t("settings_database"),
        &mut draft.auto_commit,
        &t("settings_auto_commit"),
    );
    hint(ui);
}

fn render_auto_recovery_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_auto_recovery"));
    checkbox_row(
        ui,
        &t("settings_tab_auto_recovery"),
        &mut draft.enable_auto_recovery,
        &t("settings_enable_auto_recovery"),
    );
    checkbox_subrow(
        ui,
        &mut draft.ask_before_closing_tables,
        &t("settings_ask_close_tables"),
    );
    hint(ui);
}

fn render_ai_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_ai"));
    checkbox_row(
        ui,
        &t("settings_tab_ai"),
        &mut draft.ai_assistant_enabled,
        &t("settings_ai_assistant"),
    );
    ui.add_space(8.0);
    form_row(ui, "Provider", |ui| {
        ui.add_enabled(false, egui::Button::new("Local / Custom"));
    });
    hint(ui);
}

fn render_environment_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_environment"));
    form_row(ui, &t("settings_language"), |ui| {
        render_language_combo(ui, "settings_language_environment", draft);
    });
    form_row(ui, &t("settings_appearance"), |ui| {
        render_appearance_combo(ui, "settings_appearance_environment", draft);
    });
    hint(ui);
}

fn render_advanced_tab(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    section_heading(ui, t("settings_tab_advanced"));
    checkbox_row(
        ui,
        &t("settings_confirm_dialog"),
        &mut draft.confirm_destructive,
        &t("settings_confirm_destructive"),
    );
    checkbox_subrow(
        ui,
        &mut draft.safe_confirm_dialog,
        &t("settings_safe_confirm_dialog"),
    );
    hint(ui);
}

fn render_footer(ui: &mut egui::Ui, close_action: &mut CloseAction) {
    let (rect, _) = ui.allocate_exact_size(vec2(PREF_WIDTH, FOOTER_HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, CornerRadius::same(0), FOOTER_BG);
    painter.hline(
        rect.x_range(),
        rect.top(),
        Stroke::new(1.0, Color32::from_rgb(25, 25, 25)),
    );

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(14.0, 9.0)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new(t("settings_restore_defaults")).color(TEXT))
                        .fill(Color32::from_rgb(68, 68, 68))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(72, 72, 72)))
                        .corner_radius(CornerRadius::same(5)),
                )
                .clicked()
            {
                *close_action = CloseAction::RestoreDefaults;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new(t("button_ok")).color(Color32::WHITE))
                            .fill(ACTIVE_BLUE)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(55, 154, 255)))
                            .corner_radius(CornerRadius::same(5)),
                    )
                    .clicked()
                {
                    *close_action = CloseAction::Apply;
                }
                if ui
                    .add(
                        egui::Button::new(RichText::new(t("button_cancel")).color(TEXT))
                            .fill(Color32::from_rgb(68, 68, 68))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(72, 72, 72)))
                            .corner_radius(CornerRadius::same(5)),
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
        ui.checkbox(value, RichText::new(text).color(TEXT).size(13.0));
    });
    ui.add_space(5.0);
}

fn checkbox_subrow(ui: &mut egui::Ui, value: &mut bool, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + CONTROL_GAP + 22.0);
        ui.checkbox(value, RichText::new(text).color(TEXT).size(13.0));
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
            ui.label(RichText::new(label).color(TEXT).size(13.0).strong());
        },
    );
}

fn section_heading(ui: &mut egui::Ui, title: String) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).color(TEXT).size(15.0).strong());
    ui.add_space(16.0);
}

fn hint(ui: &mut egui::Ui) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + CONTROL_GAP);
        ui.label(
            RichText::new(t("settings_placeholder_hint"))
                .color(TEXT_MUTED)
                .size(12.0),
        );
    });
}

fn font_panel(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    ui.horizontal(|ui| {
        label_cell(ui, &t("settings_object_list_font"));
        ui.add_space(CONTROL_GAP);
        Frame::new()
            .fill(FIELD_BG)
            .corner_radius(CornerRadius::same(9))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.set_width(294.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(t("settings_font"))
                            .color(TEXT)
                            .size(13.0)
                            .strong(),
                    );
                    let mut font_name = ".AppleSystemUIFont 12.0".to_string();
                    ui.add_enabled(
                        !draft.use_default_object_font,
                        TextEdit::singleline(&mut font_name).desired_width(180.0),
                    );
                    ui.add_enabled(!draft.use_default_object_font, egui::Button::new("..."));
                });
                ui.horizontal(|ui| {
                    ui.add_space(38.0);
                    ui.checkbox(
                        &mut draft.use_default_object_font,
                        RichText::new(t("settings_use_default_font"))
                            .color(TEXT)
                            .size(13.0),
                    );
                });
            });
    });
    ui.add_space(8.0);
}

fn usage_data_row(ui: &mut egui::Ui, draft: &mut storage::settings::AppSettings) {
    ui.horizontal(|ui| {
        label_cell(ui, &t("settings_usage_data"));
        ui.add_space(CONTROL_GAP);
        ui.checkbox(
            &mut draft.share_usage_data,
            RichText::new(t("settings_share_usage_data"))
                .color(TEXT)
                .size(13.0),
        );
        ui.add_enabled(false, egui::Button::new(t("settings_usage_data")));
    });
    ui.horizontal(|ui| {
        ui.add_space(LABEL_WIDTH + CONTROL_GAP + 22.0);
        ui.label(
            RichText::new(t("settings_usage_data_help"))
                .color(TEXT_SOFT)
                .size(11.0),
        );
        ui.label(
            RichText::new("Learn More...")
                .color(Color32::from_rgb(82, 171, 255))
                .size(11.0),
        );
    });
    ui.add_space(8.0);
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
    painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(30, 38, 43));
    painter.rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, Color32::from_rgb(93, 101, 105)),
        StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_size(rect.left_top() + vec2(4.0, 5.0), vec2(20.0, 40.0)),
        CornerRadius::same(1),
        Color32::from_rgb(36, 46, 51),
    );
    for i in 0..5 {
        let y = rect.top() + 9.0 + i as f32 * 7.0;
        painter.rect_filled(
            Rect::from_min_size(pos2(rect.left() + 8.0, y), vec2(12.0, 2.0)),
            CornerRadius::same(1),
            if i % 2 == 0 {
                ACTIVE_BLUE
            } else {
                Color32::from_rgb(89, 188, 99)
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

fn paint_traffic_lights(painter: &egui::Painter, origin: egui::Pos2) {
    let colors = [
        Color32::from_rgb(255, 95, 87),
        Color32::from_rgb(255, 189, 46),
        Color32::from_rgb(40, 200, 64),
    ];
    for (idx, color) in colors.iter().enumerate() {
        painter.circle_filled(origin + vec2(idx as f32 * 20.0, 0.0), 6.4, *color);
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
            painter.rect_filled(front, CornerRadius::same(2), HEADER_BG);
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
