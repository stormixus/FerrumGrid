//! JSON tree editor — renders editable JSON value as collapsible nodes.
//!
//! Plan v7 US-F3 — extracted from `info_panel.rs`. Hosts
//! `render_info_json_editor` + recursive node rendering helpers.

use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

use crate::ui::theme;

use super::info_panel::{info_toggle_control, tiny_badge};

pub(super) fn render_info_json_editor(ui: &mut egui::Ui, edit: &mut crate::state::EditableCell) {
    let source = edit.value.trim();
    let mut value = if source.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        match serde_json::from_str::<serde_json::Value>(source) {
            Ok(value) => value,
            Err(error) => {
                ui.label(
                    RichText::new(error.to_string())
                        .color(theme::ACCENT_RED)
                        .size(11.0),
                );
                ui.add_space(theme::SPACE_XS);
                ui.add(
                    theme::multiline_mono_text_input(&mut edit.value)
                        .desired_width(ui.available_width())
                        .desired_rows(4)
                        .code_editor(),
                );
                return;
            }
        }
    };

    let mut changed = false;
    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_MD_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            changed |= render_json_value_node(ui, "JSON", &mut value, 0, "$");
        });

    if changed {
        if let Ok(next) = serde_json::to_string(&value) {
            edit.value = next;
        }
    }
}

fn render_json_value_node(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut serde_json::Value,
    depth: usize,
    path: &str,
) -> bool {
    match value {
        serde_json::Value::Object(object) => {
            render_json_branch_header(
                ui,
                label,
                "object",
                &object.len().to_string(),
                depth,
                theme::ACCENT_BLUE,
            );
            if object.is_empty() {
                render_json_empty_line(ui, "{}", depth + 1);
                return false;
            }

            let mut changed = false;
            let keys = object.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                if let Some(child) = object.get_mut(&key) {
                    let child_path = json_child_path(path, &key);
                    changed |= render_json_value_node(ui, &key, child, depth + 1, &child_path);
                }
            }
            changed
        }
        serde_json::Value::Array(items) => {
            render_json_branch_header(
                ui,
                label,
                "array",
                &items.len().to_string(),
                depth,
                theme::accent_color(),
            );
            if items.is_empty() {
                render_json_empty_line(ui, "[]", depth + 1);
                return false;
            }

            let mut changed = false;
            for (idx, item) in items.iter_mut().enumerate() {
                let child_path = format!("{path}[{idx}]");
                changed |=
                    render_json_value_node(ui, &format!("[{idx}]"), item, depth + 1, &child_path);
            }
            changed
        }
        _ => render_json_scalar_node(ui, label, value, depth, path),
    }
}

fn render_json_branch_header(
    ui: &mut egui::Ui,
    label: &str,
    kind: &str,
    count: &str,
    depth: usize,
    color: Color32,
) {
    ui.add_space(theme::SPACE_XS);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(label)
                .color(theme::text_primary())
                .strong()
                .size(11.5),
        );
        tiny_badge(ui, kind, color);
        tiny_badge(ui, count, theme::text_muted());
    });
}

fn render_json_scalar_node(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut serde_json::Value,
    depth: usize,
    path: &str,
) -> bool {
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(label)
                .color(theme::text_primary())
                .strong()
                .size(11.5),
        );
        tiny_badge(ui, json_value_kind(value), json_value_color(value));
    });

    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_space(json_depth_indent(depth + 1));
        changed |= render_json_scalar_control(ui, value, path);
    });
    changed
}

fn render_json_scalar_control(
    ui: &mut egui::Ui,
    value: &mut serde_json::Value,
    path: &str,
) -> bool {
    match value {
        serde_json::Value::String(text) => ui
            .add(
                theme::text_input(text)
                    .id_salt(("json_string", path.to_owned()))
                    .desired_width(ui.available_width()),
            )
            .changed(),
        serde_json::Value::Number(number) => {
            let buffer_id = ui.make_persistent_id(("json_number_buffer", path.to_owned()));
            let canonical = number.to_string();
            let mut text = ui
                .data_mut(|data| data.get_temp::<String>(buffer_id))
                .unwrap_or(canonical);
            let response = ui.add(
                theme::mono_text_input(&mut text)
                    .id_salt(("json_number", path.to_owned()))
                    .desired_width(ui.available_width()),
            );
            if response.changed() {
                ui.data_mut(|data| data.insert_temp(buffer_id, text.clone()));
            } else if !response.has_focus() {
                ui.data_mut(|data| {
                    data.remove_temp::<String>(buffer_id);
                });
            }
            if response.changed() {
                if let Ok(serde_json::Value::Number(parsed)) =
                    serde_json::from_str::<serde_json::Value>(&text)
                {
                    *value = serde_json::Value::Number(parsed);
                    return true;
                }
            }
            false
        }
        serde_json::Value::Bool(flag) => {
            let mut checked = *flag;
            let response = info_toggle_control(ui, &mut checked, &flag.to_string(), true);
            if response.changed() {
                *flag = checked;
                true
            } else {
                false
            }
        }
        serde_json::Value::Null => {
            let buffer_id = ui.make_persistent_id(("json_null_buffer", path.to_owned()));
            let mut text = ui
                .data_mut(|data| data.get_temp::<String>(buffer_id))
                .unwrap_or_else(|| "null".to_string());
            let response = ui.add(
                theme::mono_text_input(&mut text)
                    .id_salt(("json_null", path.to_owned()))
                    .desired_width(ui.available_width()),
            );
            if response.changed() {
                ui.data_mut(|data| data.insert_temp(buffer_id, text.clone()));
            } else if !response.has_focus() {
                ui.data_mut(|data| {
                    data.remove_temp::<String>(buffer_id);
                });
            }
            if response.changed() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    *value = parsed;
                    return true;
                }
            }
            false
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => false,
    }
}

fn json_child_path(parent: &str, key: &str) -> String {
    format!("{parent}.{}", key.replace('\\', "\\\\").replace('.', "\\."))
}

fn render_json_empty_line(ui: &mut egui::Ui, text: &str, depth: usize) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space(json_depth_indent(depth));
        ui.label(
            RichText::new(text)
                .color(theme::text_muted())
                .monospace()
                .size(11.0),
        );
    });
}

fn json_value_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn json_value_color(value: &serde_json::Value) -> Color32 {
    match value {
        serde_json::Value::Null => theme::text_muted(),
        serde_json::Value::Bool(_) => theme::ACCENT_YELLOW,
        serde_json::Value::Number(_) => theme::accent_color_light(),
        serde_json::Value::String(_) => theme::ACCENT_GREEN,
        serde_json::Value::Array(_) => theme::accent_color(),
        serde_json::Value::Object(_) => theme::ACCENT_BLUE,
    }
}

fn json_depth_indent(depth: usize) -> f32 {
    (depth as f32 * 14.0).min(84.0)
}

