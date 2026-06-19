use eframe::egui::{self, Margin, RichText, Stroke, TextEdit};

use crate::ai::{collect_schema_context, generate_sql_with_tables};
use crate::i18n::t;
use crate::state::AppState;
use crate::storage::settings::AppSettings;
use crate::ui::theme;

pub fn poll_ai_assist(state: &mut AppState) {
    let Some(rx) = state.ai_assist.result_rx.as_ref() else {
        return;
    };
    match rx.try_recv() {
        Ok(Ok(sql)) => {
            state.ai_assist.generating = false;
            state.ai_assist.generated_sql = sql;
            state.ai_assist.error = None;
            state.ai_assist.result_rx = None;
        }
        Ok(Err(err)) => {
            state.ai_assist.generating = false;
            state.ai_assist.error = Some(err);
            state.ai_assist.result_rx = None;
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            state.ai_assist.generating = false;
            state.ai_assist.error = Some(t("ai_assist_failed"));
            state.ai_assist.result_rx = None;
        }
    }
}

pub fn render_ai_assist_dialog(ctx: &egui::Context, state: &mut AppState, settings: &AppSettings) {
    if !state.ai_assist.open {
        return;
    }

    let mut open = true;
    egui::Window::new(t("ai_assist_title"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(520.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_XL as i8)),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("ai_assist_prompt_label"))
                    .color(theme::text_secondary())
                    .size(12.0),
            );
            ui.add_space(theme::SPACE_SM);
            ui.add(
                TextEdit::multiline(&mut state.ai_assist.prompt)
                    .hint_text(t("ai_assist_prompt_hint"))
                    .desired_rows(4)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::monospace(12.0)),
            );

            ui.add_space(theme::SPACE_MD);

            if state.ai_assist.generating {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new(t("ai_assist_generating"))
                            .color(theme::text_muted())
                            .size(12.0),
                    );
                });
            }

            if let Some(err) = state.ai_assist.error.as_ref() {
                ui.label(
                    RichText::new(err)
                        .color(theme::accent_red_soft())
                        .size(12.0),
                );
            }

            if !state.ai_assist.generated_sql.is_empty() {
                ui.label(
                    RichText::new(t("ai_assist_preview"))
                        .color(theme::text_secondary())
                        .size(12.0)
                        .strong(),
                );
                ui.add(
                    TextEdit::multiline(&mut state.ai_assist.generated_sql)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::monospace(12.0)),
                );
                ui.label(
                    RichText::new(t("ai_assist_trust_note"))
                        .color(theme::text_muted())
                        .size(11.0)
                        .italics(),
                );
            }

            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                let can_generate = !state.ai_assist.generating && settings.ai_assistant_enabled;
                if ui
                    .add_enabled(
                        can_generate,
                        theme::primary_button(&t("ai_assist_generate")),
                    )
                    .clicked()
                {
                    start_generation(state, settings);
                }

                let can_insert = !state.ai_assist.generated_sql.trim().is_empty();
                if ui
                    .add_enabled(can_insert, theme::secondary_button(&t("ai_assist_insert")))
                    .clicked()
                {
                    insert_into_editor(state);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t("button_cancel")).clicked() {
                        state.ai_assist.open = false;
                    }
                });
            });

            if !settings.ai_assistant_enabled {
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(t("ai_assist_disabled"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
            }
        });

    if !open {
        state.ai_assist.open = false;
    }
}

fn start_generation(state: &mut AppState, settings: &AppSettings) {
    let prompt = state.ai_assist.prompt.clone();
    let schema = state
        .active_connection
        .and_then(|id| state.connections.get(&id))
        .map(collect_schema_context)
        .unwrap_or_default();
    let settings = settings.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    state.ai_assist.generating = true;
    state.ai_assist.error = None;
    state.ai_assist.generated_sql.clear();
    state.ai_assist.result_rx = Some(rx);

    std::thread::spawn(move || {
        let result = generate_sql_with_tables(&prompt, &schema, &settings);
        let _ = tx.send(result);
    });
}

fn insert_into_editor(state: &mut AppState) {
    let sql = state.ai_assist.generated_sql.trim();
    if sql.is_empty() {
        return;
    }
    if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
        if !tab.content.is_empty() && !tab.content.ends_with('\n') {
            tab.content.push('\n');
        }
        if !tab.content.is_empty() {
            tab.content.push('\n');
        }
        tab.content.push_str(sql);
        if !sql.ends_with(';') {
            tab.content.push(';');
        }
        tab.content.push('\n');
    }
    state.ai_assist.open = false;
    state.status_message = t("ai_assist_inserted");
}

pub fn open_ai_assist(state: &mut AppState) {
    state.ai_assist.open = true;
    state.ai_assist.error = None;
    if state.ai_assist.prompt.is_empty() {
        state.ai_assist.generated_sql.clear();
    }
}
