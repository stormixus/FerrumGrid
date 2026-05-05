//! Cell editing, selection, date/time pickers, and relation-jump UI.
//!
//! Extracted from `super::mod.rs` — hosts render_editable_cell,
//! render_readonly_data_cell, date/time editors, cell overlay widgets,
//! relation jump buttons, and supporting helpers.

use chrono::{Datelike, Timelike};
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::t;
use crate::state::{
    data_filter_from_cell, data_filter_from_text, data_timezone_offset_seconds, AppState,
};
use crate::types::{CellValue, ColumnMeta};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

use super::info_panel::{dark_select_control, editable_cell_display_text};
use super::tooltips::show_dark_hover_tooltip;
use super::{
    data_column_info, edit_kind, has_table_column_metadata, open_related_data, parse_bool,
    relation_for_column, render_cell, render_passive_cell, render_passive_copyable_cell,
    passive_value_pill, request_foreign_keys_for_schema, show_cell_copy_context_menu,
    show_dark_popup_below, validate_edit_value, EditKind, GRID_CELL_RIGHT_PAD,
};

pub(super) fn render_editable_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    row_idx: usize,
    col_idx: usize,
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
) {
    if !has_table_column_metadata(state) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            None,
        );
        return;
    }

    if !state.data_edit.cells.contains_key(&(row_idx, col_idx)) {
        state.data_edit.cells.insert(
            (row_idx, col_idx),
            crate::state::EditableCell::from_cell_for_type(
                fallback_cell,
                column.map(|col| col.type_name.as_str()).unwrap_or_default(),
                &state.data_timezone,
            ),
        );
    }

    let column_info = column.and_then(|col| data_column_info(state, &col.name).cloned());
    if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
        render_readonly_data_cell(
            ui,
            state,
            bridge,
            (row_idx, col_idx),
            fallback_cell,
            column,
            Some(&t("data_info_read_only_pk")),
        );
        return;
    }

    let cell_key = (row_idx, col_idx);
    let type_name = column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .or_else(|| column.map(|col| col.type_name.clone()))
        .unwrap_or_default();
    let nullable = column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();

    let Some(snapshot) = state.data_edit.cells.get(&cell_key).cloned() else {
        render_cell(ui, fallback_cell);
        return;
    };

    let dirty = snapshot.is_dirty();
    let error = validate_edit_value(&snapshot, &type_name, nullable, &enum_values);
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();
    if selected {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_TEAL, 34),
        );
    }
    if dirty {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_COPPER, 30),
        );
    } else if error.is_some() {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_RED, 28),
        );
    }
    if selected {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(1.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::ACCENT_TEAL),
            egui::StrokeKind::Inside,
        );
    }

    let is_editing = state.data_edit.editing_cell == Some(cell_key);
    if !is_editing {
        let relation_target = if !snapshot.is_null && error.is_none() {
            relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
                data_filter_from_text(fk.target_column.clone(), type_name.clone(), &snapshot.value)
                    .map(|filter| (fk, filter))
            })
        } else {
            None
        };
        let response = ui.interact(
            rect,
            ui.make_persistent_id(("data_cell", row_idx, col_idx)),
            egui::Sense::click(),
        );
        let copy_text = editable_cell_display_text(&snapshot);
        show_cell_copy_context_menu(&response, &copy_text);
        let content_width = relation_content_width(rect, relation_target.is_some());
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, rect.height()),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                render_editable_display_cell(ui, &snapshot, fallback_cell, &type_name);
                if dirty {
                    ui.add_space(2.0);
                    ui.painter().circle_filled(
                        ui.cursor().left_center() + egui::vec2(4.0, 0.0),
                        2.0,
                        theme::ACCENT_COPPER,
                    );
                    ui.add_space(8.0);
                }
            },
        );
        let relation_clicked = if let Some((fk, filter)) = relation_target {
            let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
            if clicked {
                open_related_data(state, bridge, &fk, filter);
            }
            clicked
        } else {
            false
        };

        if response.clicked() && !relation_clicked {
            select_data_cell(state, row_idx, col_idx, true);
        }
        if let Some(error) = error {
            show_dark_hover_tooltip(ui, response.id.with("error"), &response, &error);
        }
        return;
    }

    let mut close_editor = false;
    let Some(edit) = state.data_edit.cells.get_mut(&cell_key) else {
        render_cell(ui, fallback_cell);
        return;
    };

    let editor_rect = cell_overlay_editor_rect(rect, nullable);
    if nullable && cell_overlay_null_toggle(ui, rect, cell_key, edit.is_null).clicked() {
        edit.is_null = !edit.is_null;
    }

    if edit.is_null {
        render_cell_overlay_value(ui, editor_rect, "NULL", theme::text_muted());
    } else if !enum_values.is_empty() {
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(editor_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.set_clip_rect(editor_rect);
                close_editor |= render_enum_editor(ui, edit, row_idx, col_idx, &enum_values);
            },
        );
    } else {
        match edit_kind(&type_name, fallback_cell) {
            EditKind::Bool => {
                close_editor |= render_cell_bool_overlay(ui, editor_rect, edit);
            }
            EditKind::Date => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::DateTime => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Number | EditKind::Json | EditKind::Uuid | EditKind::Bytes => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, true, error.as_deref());
            }
            EditKind::Text => {
                close_editor |=
                    render_cell_text_overlay(ui, editor_rect, edit, false, error.as_deref());
            }
        }
    }

    if close_editor || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.data_edit.editing_cell = None;
    }
}

fn cell_overlay_editor_rect(cell_rect: egui::Rect, nullable: bool) -> egui::Rect {
    let left_pad = if nullable { 38.0 } else { 0.0 };
    let left = cell_rect.left() + left_pad;
    let right = (cell_rect.right() - GRID_CELL_RIGHT_PAD).max(left + 28.0);
    egui::Rect::from_min_max(
        egui::pos2(left, cell_rect.center().y - 12.0),
        egui::pos2(right, cell_rect.center().y + 12.0),
    )
}

fn cell_overlay_null_toggle(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    checked: bool,
) -> egui::Response {
    let rect = egui::Rect::from_center_size(
        egui::pos2(cell_rect.left() + 17.0, cell_rect.center().y),
        egui::vec2(32.0, 18.0),
    );
    let response = ui.interact(
        rect,
        ui.make_persistent_id(("cell_null_toggle", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let fill = if checked {
        theme::with_alpha(theme::ACCENT_TEAL, if hovered { 52 } else { 34 })
    } else if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if checked {
        theme::ACCENT_TEAL
    } else {
        theme::border_default()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, stroke),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "NULL",
        egui::FontId::proportional(9.5),
        if checked {
            theme::ACCENT_TEAL
        } else {
            theme::text_muted()
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("grid_toggle_null"),
    );
    response
}

fn render_cell_text_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
    monospace: bool,
    error: Option<&str>,
) -> bool {
    let response = ui.put(
        rect,
        cell_overlay_text_input(&mut edit.value, monospace).desired_width(rect.width()),
    );
    response.request_focus();
    if let Some(error) = error {
        show_dark_hover_tooltip(ui, response.id.with("error"), &response, error);
    }
    enter_pressed(ui)
}

fn cell_overlay_text_input(text: &mut String, monospace: bool) -> egui::TextEdit<'_> {
    let input = egui::TextEdit::singleline(text)
        .background_color(theme::bg_darkest())
        .text_color(theme::text_primary())
        .margin(Margin::symmetric(7, 2))
        .min_size(egui::vec2(0.0, 24.0))
        .vertical_align(egui::Align::Center);
    if monospace {
        input.font(egui::TextStyle::Monospace)
    } else {
        input.font(egui::TextStyle::Body)
    }
}

fn render_cell_bool_overlay(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    edit: &mut crate::state::EditableCell,
) -> bool {
    let mut checked = parse_bool(&edit.value).unwrap_or(false);
    let response = ui.put(rect, egui::Checkbox::new(&mut checked, ""));
    if response.changed() {
        edit.value = checked.to_string();
    }
    response.lost_focus() && enter_pressed(ui)
}

fn render_cell_overlay_value(ui: &mut egui::Ui, rect: egui::Rect, text: &str, color: Color32) {
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::bg_darkest(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::monospace(12.0),
        color,
    );
}

pub(super) fn render_readonly_data_cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    cell_key: (usize, usize),
    fallback_cell: &CellValue,
    column: Option<&ColumnMeta>,
    tooltip: Option<&str>,
) {
    let (row_idx, col_idx) = cell_key;
    let selected = state.data_edit.selected_cell == Some(cell_key);
    let rect = ui.available_rect_before_wrap();

    if selected {
        ui.painter().rect_filled(
            rect.shrink2(egui::vec2(0.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_TEAL, 30),
        );
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(1.0, 2.0)),
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::ACCENT_TEAL),
            egui::StrokeKind::Inside,
        );
    }

    let response = ui.interact(
        rect,
        ui.make_persistent_id(("data_cell_readonly", row_idx, col_idx)),
        egui::Sense::click(),
    );
    let copy_text = fallback_cell.to_string();
    show_cell_copy_context_menu(&response, &copy_text);
    let relation_target =
        relation_for_column(state, column.map(|col| col.name.as_str())).and_then(|fk| {
            let type_name = column
                .map(|col| col.type_name.clone())
                .unwrap_or_else(|| fk.source_column.clone());
            data_filter_from_cell(fk.target_column.clone(), type_name, fallback_cell)
                .map(|filter| (fk, filter))
        });
    let content_width = relation_content_width(rect, relation_target.is_some());
    ui.allocate_ui_with_layout(
        egui::vec2(content_width, rect.height()),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            render_passive_cell(ui, fallback_cell);
        },
    );

    let relation_clicked = if let Some((fk, filter)) = relation_target {
        let clicked = render_relation_jump_button(ui, rect, cell_key, selected);
        if clicked {
            open_related_data(state, bridge, &fk, filter);
        }
        clicked
    } else {
        false
    };

    if response.clicked() && !relation_clicked {
        select_data_cell(state, row_idx, col_idx, false);
    }
    if let Some(tooltip) = tooltip {
        show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    }
}

fn select_data_cell(state: &mut AppState, row_idx: usize, col_idx: usize, editable: bool) {
    let cell_key = (row_idx, col_idx);
    state.data_edit.selected_cell = Some(cell_key);
    state.data_edit.editing_cell = editable.then_some(cell_key);
    state.show_info_panel = true;
}

pub(super) fn ensure_foreign_keys_for_active_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    request_foreign_keys_for_schema(state, bridge, source.conn_id, &source.schema);
}

fn relation_content_width(cell_rect: egui::Rect, has_relation: bool) -> f32 {
    if has_relation {
        (cell_rect.width() - 34.0).max(0.0)
    } else {
        cell_rect.width()
    }
}

fn render_relation_jump_button(
    ui: &mut egui::Ui,
    cell_rect: egui::Rect,
    cell_key: (usize, usize),
    selected: bool,
) -> bool {
    let button_rect = egui::Rect::from_min_max(
        egui::pos2(cell_rect.right() - 30.0, cell_rect.center().y - 12.0),
        egui::pos2(cell_rect.right() - 6.0, cell_rect.center().y + 12.0),
    );
    let response = ui.interact(
        button_rect,
        ui.make_persistent_id(("relation_jump", cell_key.0, cell_key.1)),
        egui::Sense::click(),
    );
    let hovered = response.hovered();
    let emphasized = hovered || selected;
    let fill = if emphasized {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::ACCENT_TEAL)
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(button_rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        button_rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let icon_color = if emphasized {
        theme::ACCENT_TEAL
    } else {
        theme::with_alpha(theme::ACCENT_TEAL, 190)
    };
    let center = button_rect.center();
    let tip = egui::pos2(center.x + 3.0, center.y);
    ui.painter().line_segment(
        [egui::pos2(center.x - 3.5, center.y - 5.5), tip],
        Stroke::new(2.2, icon_color),
    );
    ui.painter().line_segment(
        [tip, egui::pos2(center.x - 3.5, center.y + 5.5)],
        Stroke::new(2.2, icon_color),
    );

    if hovered {
        set_pointing_cursor_on_hover(ui, &response, true);
    }
    show_dark_hover_tooltip(
        ui,
        response.id.with("tooltip"),
        &response,
        &t("data_relation_open"),
    );

    response.clicked()
}

pub(super) fn relation_tab_title(fk: &ForeignKey, value: &str) -> String {
    format!(
        "{}.{} · {}",
        fk.target_schema,
        fk.target_table,
        compact_relation_value(value)
    )
}

fn compact_relation_value(value: &str) -> String {
    const MAX_CHARS: usize = 22;
    if value.chars().count() <= MAX_CHARS {
        value.to_string()
    } else {
        let mut compact = value.chars().take(MAX_CHARS).collect::<String>();
        compact.push_str("...");
        compact
    }
}

fn render_editable_display_cell(
    ui: &mut egui::Ui,
    edit: &crate::state::EditableCell,
    fallback_cell: &CellValue,
    type_name: &str,
) {
    if edit.is_null {
        passive_value_pill(ui, "NULL", theme::text_muted());
        return;
    }

    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let value = parse_bool(&edit.value).unwrap_or(false);
            let (text, color) = if value {
                ("true", theme::ACCENT_GREEN)
            } else {
                ("false", theme::ACCENT_RED)
            };
            passive_value_pill(ui, text, color);
        }
        EditKind::Number => {
            render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT)
        }
        EditKind::Json => render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_TEAL),
        EditKind::Date | EditKind::DateTime => {
            render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_BLUE)
        }
        EditKind::Uuid => render_passive_copyable_cell(ui, &edit.value, theme::ACCENT_COPPER_LIGHT),
        EditKind::Bytes => render_passive_copyable_cell(ui, &edit.value, theme::text_muted()),
        EditKind::Text => render_passive_copyable_cell(ui, &edit.value, theme::text_primary()),
    }
}

fn render_enum_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    row_idx: usize,
    col_idx: usize,
    enum_values: &[String],
) -> bool {
    let selected = if edit.value.trim().is_empty() {
        t("grid_enum_select")
    } else {
        edit.value.clone()
    };
    if let Some(value) = dark_select_control(
        ui,
        ("enum_cell", row_idx, col_idx),
        &selected,
        enum_values,
        ui.available_width().max(96.0),
    ) {
        edit.value = value;
        true
    } else {
        false
    }
}

pub(super) fn render_date_editor(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    include_time: bool,
    timezone: &str,
    error: Option<&str>,
) -> bool {
    let (mut date, mut time) = split_datetime_value(&edit.value);
    let mut close_editor = false;
    let mut changed = false;
    let mut use_now = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SPACE_SM;
        let date_response = ui.add(
            theme::mono_text_input(&mut date)
                .desired_width(108.0)
                .hint_text("YYYY-MM-DD"),
        );
        changed |= date_response.changed();
        close_editor |= date_response.lost_focus() && enter_pressed(ui);
        if let Some(error) = error {
            show_dark_hover_tooltip(ui, date_response.id.with("error"), &date_response, error);
        }
        changed |= render_date_picker_button(ui, &mut date);

        if include_time {
            let time_response = ui.add(
                theme::mono_text_input(&mut time)
                    .desired_width(92.0)
                    .hint_text("HH:MM:SS"),
            );
            changed |= time_response.changed();
            close_editor |= time_response.lost_focus() && enter_pressed(ui);
            if let Some(error) = error {
                show_dark_hover_tooltip(ui, time_response.id.with("error"), &time_response, error);
            }
            changed |= render_time_picker_button(ui, &mut time);
        }

        use_now = inline_dark_text_button(ui, &t("grid_now")).clicked();
    });

    if changed {
        edit.value = compose_datetime_edit_value(&date, &time, include_time);
    }

    if use_now {
        let now_utc = chrono::Utc::now();
        edit.value = if include_time {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d %H:%M:%S").to_string())
        } else {
            data_timezone_offset_seconds(timezone)
                .and_then(chrono::FixedOffset::east_opt)
                .map(|offset| {
                    now_utc
                        .with_timezone(&offset)
                        .format("%Y-%m-%d")
                        .to_string()
                })
                .unwrap_or_else(|| now_utc.format("%Y-%m-%d").to_string())
        };
        close_editor = true;
    }

    close_editor
}

fn compose_datetime_edit_value(date: &str, time: &str, include_time: bool) -> String {
    if include_time {
        format!("{} {}", date.trim(), time.trim())
            .trim()
            .to_string()
    } else {
        date.trim().to_string()
    }
}

fn render_date_picker_button(ui: &mut egui::Ui, date: &mut String) -> bool {
    let selected = parse_picker_date(date).unwrap_or_else(default_picker_date);
    let response = picker_icon_button(
        ui,
        crate::ui::icons_svg::CALENDAR,
        "date_picker_icon",
        &t("grid_pick_date"),
    );
    let popup_id = response.id.with("date_picker");
    if response.clicked() {
        let opening = !ui.memory(|memory| memory.is_popup_open(popup_id));
        ui.memory_mut(|memory| {
            if opening {
                memory.data.insert_temp(
                    popup_id.with("visible_month"),
                    (selected.year(), selected.month()),
                );
            }
            memory.toggle_popup(popup_id);
        });
    }

    let mut changed = false;
    show_dark_popup_below(ui, popup_id, &response, 238.0, theme::SPACE_MD_I, |ui| {
        changed |= render_date_picker_calendar(ui, popup_id, selected, date);
    });
    changed
}

fn render_date_picker_calendar(
    ui: &mut egui::Ui,
    popup_id: egui::Id,
    selected: chrono::NaiveDate,
    date: &mut String,
) -> bool {
    let visible_id = popup_id.with("visible_month");
    let (mut year, mut month) = ui
        .memory(|memory| memory.data.get_temp::<(i32, u32)>(visible_id))
        .unwrap_or((selected.year(), selected.month()));
    let mut changed = false;

    ui.horizontal(|ui| {
        if picker_nav_button(ui, "<", &t("grid_prev_month")).clicked() {
            (year, month) = shifted_year_month(year, month, -1);
            ui.memory_mut(|memory| memory.data.insert_temp(visible_id, (year, month)));
        }
        ui.add_space(theme::SPACE_XS);
        ui.label(
            RichText::new(format!("{year:04}-{month:02}"))
                .color(theme::text_primary())
                .monospace()
                .strong()
                .size(13.0),
        );
        ui.add_space(theme::SPACE_XS);
        if picker_nav_button(ui, ">", &t("grid_next_month")).clicked() {
            (year, month) = shifted_year_month(year, month, 1);
            ui.memory_mut(|memory| memory.data.insert_temp(visible_id, (year, month)));
        }
    });

    ui.add_space(theme::SPACE_SM);
    let labels = date_picker_weekday_labels();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for label in labels {
            date_picker_label_cell(ui, &label);
        }
    });

    let Some(first_day) = chrono::NaiveDate::from_ymd_opt(year, month, 1) else {
        return false;
    };
    let leading = first_day.weekday().num_days_from_monday() as i32;
    let days = days_in_month(year, month) as i32;
    let today = chrono::Utc::now().date_naive();
    let mut day = 1;

    for week in 0..6 {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for weekday in 0..7 {
                let cell_index = week * 7 + weekday;
                if cell_index < leading || day > days {
                    ui.allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::hover());
                    continue;
                }

                let candidate =
                    chrono::NaiveDate::from_ymd_opt(year, month, day as u32).unwrap_or(selected);
                if date_picker_day_cell(ui, day as u32, candidate == selected, candidate == today)
                    .clicked()
                {
                    *date = candidate.format("%Y-%m-%d").to_string();
                    changed = true;
                    ui.memory_mut(|memory| memory.close_popup());
                }
                day += 1;
            }
        });
        if day > days {
            break;
        }
    }

    changed
}

fn render_time_picker_button(ui: &mut egui::Ui, time: &mut String) -> bool {
    let response = picker_icon_button(
        ui,
        crate::ui::icons_svg::CLOCK,
        "time_picker_icon",
        &t("grid_pick_time"),
    );
    let popup_id = response.id.with("time_picker");
    if response.clicked() {
        ui.memory_mut(|memory| memory.toggle_popup(popup_id));
    }

    let mut changed = false;
    show_dark_popup_below(ui, popup_id, &response, 196.0, theme::SPACE_MD_I, |ui| {
        changed |= render_time_picker(ui, time);
    });
    changed
}

fn render_time_picker(ui: &mut egui::Ui, time: &mut String) -> bool {
    let mut parsed = parse_picker_time(time).unwrap_or_else(default_picker_time);
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme::SPACE_MD;
        changed |= render_time_unit_picker(ui, &t("grid_hour"), &mut parsed.0, 23);
        changed |= render_time_unit_picker(ui, &t("grid_minute"), &mut parsed.1, 59);
        changed |= render_time_unit_picker(ui, &t("grid_second"), &mut parsed.2, 59);
    });

    if changed {
        *time = format!("{:02}:{:02}:{:02}", parsed.0, parsed.1, parsed.2);
    }
    changed
}

fn render_time_unit_picker(ui: &mut egui::Ui, label: &str, value: &mut u32, max: u32) -> bool {
    let mut changed = false;
    ui.allocate_ui_with_layout(
        egui::vec2(48.0, 118.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.label(
                RichText::new(label)
                    .color(theme::text_muted())
                    .strong()
                    .size(10.0),
            );
            if picker_step_button(ui, "+").clicked() {
                *value = if *value >= max { 0 } else { *value + 1 };
                changed = true;
            }
            let value_response = time_value_cell(ui, *value);
            if value_response.hovered() {
                let scroll_y = ui.input(|input| input.smooth_scroll_delta.y);
                if scroll_y > 4.0 {
                    *value = if *value >= max { 0 } else { *value + 1 };
                    changed = true;
                } else if scroll_y < -4.0 {
                    *value = if *value == 0 { max } else { *value - 1 };
                    changed = true;
                }
            }
            if picker_step_button(ui, "-").clicked() {
                *value = if *value == 0 { max } else { *value - 1 };
                changed = true;
            }
        },
    );
    changed
}

fn picker_icon_button(
    ui: &mut egui::Ui,
    icon_svg: &str,
    icon_name: &str,
    tooltip: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(32.0, 32.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 170))
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(14.0, 14.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(icon_rect)
            .layout(egui::Layout::centered_and_justified(
                egui::Direction::LeftToRight,
            )),
        |ui| {
            ui.set_clip_rect(icon_rect);
            ui.add(crate::ui::icon_image_tinted(
                ui,
                icon_svg,
                icon_name,
                14.0,
                theme::ACCENT_BLUE,
            ));
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(ui, response.id.with("picker_tooltip"), &response, tooltip);
    response
}

fn picker_nav_button(ui: &mut egui::Ui, label: &str, tooltip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 26.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_darkest()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(13.0),
        theme::text_secondary(),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    show_dark_hover_tooltip(ui, response.id.with("tooltip"), &response, tooltip);
    response
}

fn date_picker_label_cell(ui: &mut egui::Ui, label: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 20.0), egui::Sense::hover());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        theme::text_muted(),
    );
}

fn date_picker_day_cell(
    ui: &mut egui::Ui,
    day: u32,
    selected: bool,
    today: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        theme::with_alpha(theme::ACCENT_TEAL, 54)
    } else if hovered {
        theme::bg_light()
    } else {
        Color32::TRANSPARENT
    };
    if fill != Color32::TRANSPARENT {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    }
    let stroke = if selected {
        Some(Stroke::new(1.0, theme::ACCENT_TEAL))
    } else if today {
        Some(Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 150)))
    } else {
        None
    };
    if let Some(stroke) = stroke {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(theme::RADIUS_MD),
            stroke,
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        day.to_string(),
        egui::FontId::monospace(12.0),
        if selected {
            theme::ACCENT_TEAL
        } else {
            theme::text_secondary()
        },
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn picker_step_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(44.0, 22.0), egui::Sense::click());
    let fill = if response.hovered() {
        theme::bg_light()
    } else {
        theme::bg_darkest()
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(1.0, theme::border_default()),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(13.0),
        theme::text_secondary(),
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

fn time_value_cell(ui: &mut egui::Ui, value: u32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(44.0, 30.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        theme::input_bg(),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        Stroke::new(
            1.0,
            if response.hovered() {
                theme::ACCENT_BLUE
            } else {
                theme::border_default()
            },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{value:02}"),
        egui::FontId::monospace(14.0),
        theme::text_primary(),
    );
    response
}

fn parse_picker_date(date: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()
}

fn default_picker_date() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

fn parse_picker_time(time: &str) -> Option<(u32, u32, u32)> {
    let trimmed = time.trim();
    let cleaned = trimmed
        .split(['+', '-', '.', 'Z'])
        .next()
        .unwrap_or(trimmed)
        .trim();
    let parsed = chrono::NaiveTime::parse_from_str(cleaned, "%H:%M:%S")
        .or_else(|_| chrono::NaiveTime::parse_from_str(cleaned, "%H:%M"))
        .ok()?;
    Some((parsed.hour(), parsed.minute(), parsed.second()))
}

fn default_picker_time() -> (u32, u32, u32) {
    let now = chrono::Utc::now().time();
    (now.hour(), now.minute(), now.second())
}

fn shifted_year_month(year: i32, month: u32, delta_months: i32) -> (i32, u32) {
    let month_index = year * 12 + month as i32 - 1 + delta_months;
    (
        month_index.div_euclid(12),
        (month_index.rem_euclid(12) + 1) as u32,
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = shifted_year_month(year, month, 1);
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .unwrap_or(31)
}

fn date_picker_weekday_labels() -> [String; 7] {
    [
        t("grid_weekday_mon"),
        t("grid_weekday_tue"),
        t("grid_weekday_wed"),
        t("grid_weekday_thu"),
        t("grid_weekday_fri"),
        t("grid_weekday_sat"),
        t("grid_weekday_sun"),
    ]
}

fn inline_dark_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let font = egui::FontId::proportional(11.5);
    let text_color = theme::text_secondary();
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), text_color)
        .rect
        .width();
    let width = (text_width + 20.0).max(46.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());
    let hovered = response.hovered();
    let fill = if hovered {
        theme::bg_light()
    } else {
        theme::bg_medium()
    };
    let stroke = if hovered {
        Stroke::new(1.0, theme::with_alpha(theme::ACCENT_TEAL, 150))
    } else {
        Stroke::new(1.0, theme::border_default())
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(theme::RADIUS_MD), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_MD),
        stroke,
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );
    set_pointing_cursor_on_hover(ui, &response, true);
    response
}

pub(super) fn set_pointing_cursor_on_hover(ui: &mut egui::Ui, response: &egui::Response, enabled: bool) {
    if enabled && response.hovered() {
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
    }
}

fn split_datetime_value(value: &str) -> (String, String) {
    let value = value.trim();
    if value.is_empty() {
        return ("".to_string(), "".to_string());
    }

    if let Some((date, time)) = value.split_once(' ') {
        return (date.to_string(), time.to_string());
    }
    if let Some((date, time)) = value.split_once('T') {
        return (date.to_string(), time.trim_end_matches('Z').to_string());
    }
    (value.to_string(), "00:00:00".to_string())
}

pub(super) fn enter_pressed(ui: &egui::Ui) -> bool {
    ui.input(|i| i.key_pressed(egui::Key::Enter))
}
