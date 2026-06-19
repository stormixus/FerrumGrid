use eframe::egui::{self, Margin, RichText, Stroke, TextEdit};

use crate::i18n::t;
use crate::state::{AppState, SnippetSaveDialogState};
use crate::ui::theme;

pub fn open_snippet_save_dialog(
    state: &mut AppState,
    sql: impl Into<String>,
    connection_id: Option<crate::types::ConnectionId>,
) {
    let sql = sql.into();
    if sql.trim().is_empty() {
        return;
    }
    let default_name = format!("Snippet {}", state.snippets.len() + 1);
    state.snippet_save_dialog = Some(SnippetSaveDialogState {
        name: default_name,
        tags: String::new(),
        sql,
        connection_id,
    });
}

pub fn render_snippet_save_dialog(ctx: &egui::Context, state: &mut AppState) {
    let Some(mut dialog) = state.snippet_save_dialog.clone() else {
        return;
    };

    let mut open = true;
    let mut save = false;
    let mut cancel = false;

    egui::Window::new(t("snippet_save_title"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(420.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::bg_medium())
                .stroke(Stroke::new(1.0, theme::border_default()))
                .inner_margin(Margin::same(theme::SPACE_XL as i8)),
        )
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("snippet_save_name"))
                    .color(theme::text_muted())
                    .size(12.0),
            );
            ui.add(
                TextEdit::singleline(&mut dialog.name)
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(13.0)),
            );
            ui.add_space(theme::SPACE_MD);
            ui.label(
                RichText::new(t("snippet_save_tags"))
                    .color(theme::text_muted())
                    .size(12.0),
            );
            ui.add(
                TextEdit::singleline(&mut dialog.tags)
                    .hint_text(t("snippet_save_tags_hint"))
                    .desired_width(f32::INFINITY)
                    .font(egui::FontId::proportional(12.0)),
            );
            ui.add_space(theme::SPACE_LG);
            ui.horizontal(|ui| {
                if ui.add(theme::primary_button(&t("button_save"))).clicked() {
                    save = true;
                }
                if ui.button(t("button_cancel")).clicked() {
                    cancel = true;
                }
            });
        });

    state.snippet_save_dialog = Some(dialog);

    if !open || cancel {
        state.snippet_save_dialog = None;
    } else if save {
        if let Some(dialog) = state.snippet_save_dialog.take() {
            let name = dialog.name.trim();
            if name.is_empty() {
                state.snippet_save_dialog = Some(dialog);
                return;
            }
            let tags: Vec<String> = dialog
                .tags
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let mut entry = crate::storage::snippets::SnippetEntry::new(name, dialog.sql);
            entry.tags = tags;
            entry.connection_id = dialog.connection_id;
            state.snippets.push(entry);
            crate::storage::snippets::save_snippets(&state.snippets);
            state.status_message = t("snippet_saved");
            state.tree_panel_tab = crate::state::TreePanelTab::Snippets;
            state.show_tree_panel = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_name_counts_existing_snippets_panel_entries() {
        let mut state = AppState::default();
        state
            .snippets
            .push(crate::storage::snippets::SnippetEntry::new(
                "one",
                "select 1;",
            ));
        state
            .snippets
            .push(crate::storage::snippets::SnippetEntry::new(
                "two",
                "select 2;",
            ));

        open_snippet_save_dialog(&mut state, "select 3;", None);

        assert_eq!(
            state
                .snippet_save_dialog
                .as_ref()
                .map(|dialog| dialog.name.as_str()),
            Some("Snippet 3")
        );
    }
}
