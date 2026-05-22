//! Date / time picker — calendar + clock UI for editable date/timestamp cells.
//!
//! Plan v7 US-F3 — extracted from `selection.rs`. Hosts the public
//! `render_date_editor` entry point + all calendar/clock sub-widgets.

use chrono::{Datelike, Timelike};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};

use crate::i18n::t;
use crate::state::data_timezone_offset_seconds;
use crate::ui::theme;

use super::selection::{enter_pressed, set_pointing_cursor_on_hover};
use super::tooltips::show_dark_hover_tooltip;
use super::show_dark_popup_below;

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
                .size(12.0),
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
        theme::with_alpha(theme::accent_color(), 54)
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
        Some(Stroke::new(1.0, theme::accent_color()))
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
            theme::accent_color()
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
        Stroke::new(1.0, theme::with_alpha(theme::accent_color(), 150))
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

