//! Info-panel row rendering — header / empty / row summary / row fields / single field / editor control.
//!
//! Plan v7 US-G3 — extracted from `info_panel.rs`.

use eframe::egui::{self, CornerRadius, Margin, RichText, Stroke};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{data_filter_from_text, AppState, DataFilter};
use crate::types::{CellValue, ColumnInfo, ColumnMeta};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

use super::data_ops::{edit_kind, has_table_column_metadata, EditKind};
use super::date_picker::render_date_editor;
use super::info_panel::{
    editable_cell_display_text, info_icon_action_button, info_section_label, info_toggle_control,
    render_info_enum_editor, revert_data_cell, tiny_badge, value_box, SelectedRowContext,
};
use super::json_editor::render_info_json_editor;
use super::{
    metric_chip, open_related_data, parse_bool, relation_for_column, validate_edit_value,
};

pub(super) fn render_info_header(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(theme::bg_shell())
        .inner_margin(Margin::same(theme::SPACE_LG_I))
        .stroke(Stroke::new(1.0, theme::border_subtle()))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, crate::ui::icons_svg::INFO, "info", 14.0);
                ui.add_space(4.0);
                ui.label(
                    RichText::new(t("info"))
                        .color(theme::text_primary())
                        .strong(),
                );
            });
        });
}

pub(super) fn render_info_empty(ui: &mut egui::Ui, message: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(92.0);
        crate::ui::icon_img_tinted(
            ui,
            crate::ui::icons_svg::TABLE,
            "data_info_empty",
            28.0,
            theme::text_disabled(),
        );
        ui.add_space(theme::SPACE_SM);
        ui.label(RichText::new(message).color(theme::text_muted()).size(12.0));
    });
}

pub(super) fn render_info_row_summary(ui: &mut egui::Ui, context: &SelectedRowContext) {
    info_section_label(ui, &t("data_info_row"));
    ui.add_space(theme::SPACE_XS);
    ui.label(
        RichText::new(tf("data_info_row_n", &[&(context.row_idx + 1).to_string()]))
            .color(theme::text_primary())
            .strong()
            .size(16.0),
    );
    ui.label(
        RichText::new(&context.source_label)
            .color(theme::text_muted())
            .monospace()
            .size(11.0),
    );
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("data_info_row_n", &[&(context.row_idx + 1).to_string()]),
            theme::ACCENT_TEAL,
        );
        metric_chip(
            ui,
            &tf("data_info_columns_n", &[&context.columns.len().to_string()]),
            theme::ACCENT_BLUE,
        );
    });
}

pub(super) fn render_info_row_fields(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    context: &SelectedRowContext,
) {
    info_section_label(ui, &t("data_info_columns"));
    ui.add_space(theme::SPACE_SM);

    for (col_idx, column) in context.columns.iter().enumerate() {
        let fallback_cell = context
            .fallback_row
            .get(col_idx)
            .cloned()
            .unwrap_or(CellValue::Null);
        let column_info = context.column_infos.get(col_idx).cloned().flatten();
        let field = RowFieldContext {
            row_idx: context.row_idx,
            col_idx,
            selected: context.selected_col_idx == col_idx,
            column: column.clone(),
            column_info,
            fallback_cell,
        };
        render_info_row_field(ui, state, bridge, field);
        ui.add_space(theme::SPACE_SM);
    }
}

pub(super) struct RowFieldContext {
    row_idx: usize,
    col_idx: usize,
    selected: bool,
    column: ColumnMeta,
    column_info: Option<ColumnInfo>,
    fallback_cell: CellValue,
}

pub(super) fn render_info_row_field(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    field: RowFieldContext,
) {
    let cell_key = (field.row_idx, field.col_idx);
    let type_name = field
        .column_info
        .as_ref()
        .map(|info| info.data_type.clone())
        .unwrap_or_else(|| field.column.type_name.clone());
    let nullable = field
        .column_info
        .as_ref()
        .map(|info| info.is_nullable)
        .unwrap_or(true);
    let enum_values = field
        .column_info
        .as_ref()
        .map(|info| info.enum_values.clone())
        .unwrap_or_default();
    let is_primary_key = field
        .column_info
        .as_ref()
        .is_some_and(|info| info.is_primary_key);
    let can_edit = has_table_column_metadata(state) && !is_primary_key;
    let data_timezone = state.data_timezone.clone();

    let stroke_color = if field.selected {
        theme::ACCENT_TEAL
    } else {
        theme::border_subtle()
    };

    egui::Frame::new()
        .fill(theme::bg_darkest())
        .inner_margin(Margin::same(theme::SPACE_LG_I))
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&field.column.name)
                        .color(theme::text_primary())
                        .strong()
                        .size(12.5),
                );
                ui.label(
                    RichText::new(&type_name)
                        .color(theme::text_muted())
                        .monospace()
                        .size(10.5),
                );
                if is_primary_key {
                    tiny_badge(ui, "PK", theme::ACCENT_YELLOW);
                }
                if field.selected {
                    tiny_badge(ui, &t("data_info_selected"), theme::ACCENT_TEAL);
                }
            });

            if let Some(default_value) = field
                .column_info
                .as_ref()
                .and_then(|info| info.default_value.as_ref())
            {
                ui.add_space(theme::SPACE_XS);
                ui.label(
                    RichText::new(format!("{}: {}", t("column_default"), default_value))
                        .color(theme::text_muted())
                        .monospace()
                        .size(10.0),
                );
            }

            ui.add_space(theme::SPACE_SM);

            if can_edit {
                if let Some(edit) = state.data_edit.cells.get_mut(&cell_key) {
                    if nullable {
                        info_toggle_control(ui, &mut edit.is_null, &t("grid_toggle_null"), true);
                        ui.add_space(theme::SPACE_XS);
                    }

                    if edit.is_null {
                        value_box(ui, "NULL", theme::text_muted());
                    } else if !enum_values.is_empty() {
                        render_info_enum_editor(
                            ui,
                            edit,
                            field.row_idx,
                            field.col_idx,
                            &enum_values,
                        );
                    } else {
                        render_info_editor_control(
                            ui,
                            edit,
                            &type_name,
                            &field.fallback_cell,
                            &data_timezone,
                        );
                    }
                }
            } else {
                value_box(ui, &field.fallback_cell.to_string(), theme::text_primary());
                ui.add_space(theme::SPACE_XS);
                ui.label(
                    RichText::new(if is_primary_key {
                        t("data_info_read_only_pk")
                    } else if !has_table_column_metadata(state) {
                        t("data_info_no_metadata")
                    } else {
                        t("data_info_read_only")
                    })
                    .color(theme::text_muted())
                    .size(10.5),
                );
            }

            let snapshot = state.data_edit.cells.get(&cell_key).cloned();
            if let Some(snapshot) = snapshot {
                if let Some(error) =
                    validate_edit_value(&snapshot, &type_name, nullable, &enum_values)
                {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(RichText::new(error).color(theme::ACCENT_RED).size(11.0));
                }

                ui.add_space(theme::SPACE_SM);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(theme::SPACE_SM, theme::SPACE_SM);
                    if info_icon_action_button(
                        ui,
                        crate::ui::icons_svg::COPY,
                        "info_copy_field",
                        &t("grid_copy_value"),
                        true,
                        theme::ACCENT_BLUE,
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(editable_cell_display_text(&snapshot));
                    }

                    if can_edit
                        && info_icon_action_button(
                            ui,
                            crate::ui::icons_svg::REFRESH,
                            "info_revert_cell",
                            &t("data_info_revert_cell"),
                            snapshot.is_dirty(),
                            theme::text_muted(),
                        )
                        .clicked()
                    {
                        revert_data_cell(state, cell_key, &type_name);
                    }

                    if let Some((fk, filter)) =
                        relation_filter_for_snapshot(state, &field.column, &type_name, &snapshot)
                    {
                        let relation_resp = info_icon_action_button(
                            ui,
                            crate::ui::icons_svg::CHEVRON_RIGHT,
                            "info_relation_jump",
                            &t("data_relation_open"),
                            true,
                            theme::ACCENT_TEAL,
                        );
                        if relation_resp.clicked() {
                            open_related_data(state, bridge, &fk, filter);
                        }
                    }
                });

                if snapshot.is_dirty() {
                    ui.add_space(theme::SPACE_XS);
                    ui.label(
                        RichText::new(t("data_info_dirty"))
                            .color(theme::ACCENT_COPPER_LIGHT)
                            .size(10.5),
                    );
                }
            }
        });
}

pub(super) fn relation_filter_for_snapshot(
    state: &AppState,
    column: &ColumnMeta,
    type_name: &str,
    snapshot: &crate::state::EditableCell,
) -> Option<(ForeignKey, DataFilter)> {
    if snapshot.is_null {
        return None;
    }
    relation_for_column(state, Some(&column.name)).and_then(|fk| {
        data_filter_from_text(
            fk.target_column.clone(),
            type_name.to_string(),
            &snapshot.value,
        )
        .map(|filter| (fk, filter))
    })
}

pub(super) fn render_info_editor_control(
    ui: &mut egui::Ui,
    edit: &mut crate::state::EditableCell,
    type_name: &str,
    fallback_cell: &CellValue,
    data_timezone: &str,
) {
    match edit_kind(type_name, fallback_cell) {
        EditKind::Bool => {
            let mut checked = parse_bool(&edit.value).unwrap_or(false);
            if info_toggle_control(ui, &mut checked, "true", true).changed() {
                edit.value = checked.to_string();
            }
        }
        EditKind::Date => {
            render_date_editor(ui, edit, false, data_timezone, None);
        }
        EditKind::DateTime => {
            render_date_editor(ui, edit, true, data_timezone, None);
        }
        EditKind::Json => {
            render_info_json_editor(ui, edit);
        }
        EditKind::Text => {
            ui.add(
                theme::multiline_text_input(&mut edit.value)
                    .desired_width(ui.available_width())
                    .desired_rows(2),
            );
        }
        EditKind::Number | EditKind::Uuid | EditKind::Bytes => {
            ui.add(theme::mono_text_input(&mut edit.value).desired_width(ui.available_width()));
        }
    }
}

