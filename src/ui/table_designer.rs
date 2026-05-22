use eframe::egui::{
    self, Align, Button, Color32, ComboBox, CornerRadius, Frame, Margin, RichText, ScrollArea,
    Sense, Stroke, Ui, Vec2, Window,
};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::state::AppState;
use crate::ui::theme;

const PG_TYPES: &[&str] = &[
    "INTEGER",
    "BIGINT",
    "SMALLINT",
    "SERIAL",
    "BIGSERIAL",
    "BOOLEAN",
    "TEXT",
    "VARCHAR",
    "CHAR",
    "NUMERIC",
    "DECIMAL",
    "REAL",
    "DOUBLE PRECISION",
    "DATE",
    "TIMESTAMP",
    "TIMESTAMPTZ",
    "TIME",
    "INTERVAL",
    "BYTEA",
    "UUID",
    "JSON",
    "JSONB",
    "INET",
    "CIDR",
    "ARRAY",
    "TSVECTOR",
    "TSQUERY",
    "POINT",
    "LINE",
    "LSEG",
    "BOX",
    "PATH",
    "POLYGON",
    "CIRCLE",
];

#[derive(Debug, Clone, Default)]
pub struct TableDesignerState {
    pub show: bool,
    pub schema: String,
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<IndexDef>,
    pub selected_column: Option<usize>,
    pub generated_ddl: Option<String>,
    pub show_ddl_preview: bool,
    pub editing_table: Option<(String, String)>,
    pub original_columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub length: Option<String>,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: String,
    pub is_foreign_key: bool,
    pub fk_ref_schema: String,
    pub fk_ref_table: String,
    pub fk_ref_column: String,
}

impl Default for ColumnDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            data_type: "INTEGER".to_string(),
            length: None,
            is_nullable: true,
            is_primary_key: false,
            is_unique: false,
            default_value: String::new(),
            is_foreign_key: false,
            fk_ref_schema: String::new(),
            fk_ref_table: String::new(),
            fk_ref_column: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: String,
}

impl Default for IndexDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            columns: Vec::new(),
            is_unique: false,
            index_type: "BTREE".to_string(),
        }
    }
}

pub fn render_table_designer(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.table_designer.show {
        return;
    }

    if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        if state.table_designer.show_ddl_preview {
            state.table_designer.show_ddl_preview = false;
        } else {
            state.table_designer.show = false;
        }
        return;
    }

    if !state.table_designer.columns.is_empty() && !state.table_designer.show_ddl_preview {
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            let sel = state.table_designer.selected_column.map_or(
                state.table_designer.columns.len() - 1,
                |c| c.saturating_sub(1),
            );
            state.table_designer.selected_column = Some(sel);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            let max = state.table_designer.columns.len() - 1;
            let sel = state.table_designer.selected_column.map_or(0, |c| (c + 1).min(max));
            state.table_designer.selected_column = Some(sel);
        }
    }

    let mut open = state.table_designer.show;
    let mut should_close = false;

    Window::new(if state.table_designer.editing_table.is_some() {
        "Edit Table"
    } else {
        "Create Table"
    })
    .default_size([900.0, 700.0])
    .resizable(true)
    .collapsible(false)
    .open(&mut open)
    .show(ctx, |ui| {
        render_designer_ui(ui, state, bridge, &mut should_close);
    });

    if should_close {
        open = false;
    }
    state.table_designer.show = open;
}

fn render_designer_ui(
    ui: &mut Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    should_close: &mut bool,
) {
    let conn = state
        .active_connection
        .and_then(|id| state.connections.get(&id));
    let schemas = conn.map(|c| c.schemas.clone()).unwrap_or_default();

    ui.horizontal(|ui| {
        ui.label("Schema:");
        ComboBox::from_id_salt("td_schema")
            .width(150.0)
            .selected_text(&state.table_designer.schema)
            .show_ui(ui, |ui| {
                for schema in &schemas {
                    if ui
                        .selectable_label(&state.table_designer.schema == schema, schema)
                        .clicked()
                    {
                        state.table_designer.schema.clone_from(schema);
                    }
                }
            });

        ui.add_space(20.0);

        ui.label("Table Name:");
        let name_edit = ui.add(
            theme::text_input(&mut state.table_designer.table_name)
                .desired_width(200.0)
                .hint_text("table_name"),
        );
        if name_edit.lost_focus() {
            state.table_designer.table_name = sanitize_identifier(&state.table_designer.table_name);
        }

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if ui.add(primary_button("Apply DDL")).clicked() {
                apply_ddl(state, bridge);
                *should_close = true;
            }

            ui.add_space(8.0);

            if ui.add(primary_button("Generate DDL")).clicked() {
                state.table_designer.generated_ddl = Some(generate_ddl(state));
                state.table_designer.show_ddl_preview = true;
            }

            ui.add_space(8.0);

            if ui.button("Cancel").clicked() {
                *should_close = true;
            }
        });
    });

    ui.separator();

    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            ui.set_min_width(ui.available_width());
            render_columns_panel(ui, state);
        });

        cols[1].vertical(|ui| {
            ui.set_min_width(ui.available_width());
            if let Some(selected) = state.table_designer.selected_column {
                render_column_detail(ui, state, selected, &schemas);
            } else {
                render_indexes_panel(ui, state);
            }
        });
    });

    if state.table_designer.show_ddl_preview {
        let mut ddl_preview_open = state.table_designer.show_ddl_preview;
        let mut should_close_ddl_preview = false;

        Window::new("Generated DDL")
            .default_size([600.0, 400.0])
            .resizable(true)
            .collapsible(false)
            .open(&mut ddl_preview_open)
            .show(ui.ctx(), |ui| {
                if let Some(ref ddl) = state.table_designer.generated_ddl {
                    ui.add_sized(
                        ui.available_size(),
                        theme::multiline_mono_text_input(&mut ddl.clone()).code_editor(),
                    );
                }
                ui.horizontal(|ui| {
                    if ui.add(primary_button("Apply")).clicked() {
                        apply_ddl(state, bridge);
                    }
                    if ui.button("Copy to Clipboard").clicked() {
                        if let Some(ref ddl) = state.table_designer.generated_ddl {
                            ui.ctx().output_mut(|o| {
                                o.commands
                                    .push(egui::output::OutputCommand::CopyText(ddl.clone()));
                            });
                        }
                    }
                    if ui.button("Close").clicked() {
                        should_close_ddl_preview = true;
                    }
                });
            });

        if should_close_ddl_preview {
            ddl_preview_open = false;
        }
        state.table_designer.show_ddl_preview = ddl_preview_open;
    }
}

fn render_columns_panel(ui: &mut Ui, state: &mut AppState) {
    Frame::new()
        .fill(theme::bg_dark())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.strong("Columns");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(small_button("+")).clicked() {
                        let col = ColumnDef::default();
                        state.table_designer.columns.push(col);
                        state.table_designer.selected_column =
                            Some(state.table_designer.columns.len() - 1);
                    }
                });
            });

            ui.add_space(8.0);

            ScrollArea::vertical()
                .id_salt("columns_list")
                .show(ui, |ui| {
                    let mut to_delete = None;
                    let mut selection_changed = false;
                    let mut new_selection = state.table_designer.selected_column;

                    for (idx, col) in state.table_designer.columns.iter().enumerate() {
                        let is_selected = state.table_designer.selected_column == Some(idx);
                        let mut frame = Frame::new()
                            .inner_margin(Margin::same(6))
                            .corner_radius(CornerRadius::same(4));

                        if is_selected {
                            frame = frame
                                .fill(theme::accent_copper_dim())
                                .stroke(Stroke::new(1.0, theme::accent_color()));
                        } else {
                            frame = frame.fill(theme::bg_medium());
                        }

                        let response = frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let null_text = if col.is_nullable { "NULL" } else { "NOT NULL" };

                                if col.is_primary_key {
                                    type_chip(ui, "PK", theme::ACCENT_YELLOW);
                                }
                                if col.is_foreign_key {
                                    type_chip(ui, "FK", theme::ACCENT_BLUE);
                                }
                                ui.label(&col.name);
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(null_text)
                                            .color(theme::text_muted())
                                            .size(10.0),
                                    );
                                    ui.label(
                                        RichText::new(&col.data_type)
                                            .color(theme::ACCENT_BLUE)
                                            .size(11.0),
                                    );
                                });
                            });
                        });

                        let response =
                            ui.interact(response.response.rect, ui.id().with(idx), Sense::click());

                        if response.clicked() {
                            new_selection = Some(idx);
                            selection_changed = true;
                        }

                        response.context_menu(|ui| {
                            if ui.button("Delete").clicked() {
                                to_delete = Some(idx);
                                ui.close_menu();
                            }
                        });
                    }

                    if let Some(idx) = to_delete {
                        state.table_designer.columns.remove(idx);
                        if state.table_designer.selected_column == Some(idx) {
                            state.table_designer.selected_column = None;
                        } else if let Some(sel) = state.table_designer.selected_column {
                            if sel > idx {
                                state.table_designer.selected_column = Some(sel - 1);
                            }
                        }
                    }

                    if selection_changed {
                        state.table_designer.selected_column = new_selection;
                    }
                });
        });
}

fn render_column_detail(ui: &mut Ui, state: &mut AppState, idx: usize, schemas: &[String]) {
    if idx >= state.table_designer.columns.len() {
        return;
    }

    let col = &mut state.table_designer.columns[idx];

    Frame::new()
        .fill(theme::bg_dark())
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.strong("Column Properties");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("×").clicked() {
                        state.table_designer.selected_column = None;
                    }
                });
            });

            ui.add_space(12.0);

            egui::Grid::new("column_props")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    let name_edit = ui.add(theme::text_input(&mut col.name).desired_width(180.0));
                    if name_edit.lost_focus() {
                        col.name = sanitize_identifier(&col.name);
                    }
                    ui.end_row();

                    ui.label("Data Type:");
                    ComboBox::from_id_salt("col_type")
                        .width(180.0)
                        .selected_text(&col.data_type)
                        .show_ui(ui, |ui| {
                            for &type_name in PG_TYPES {
                                if ui
                                    .selectable_label(col.data_type == type_name, type_name)
                                    .clicked()
                                {
                                    col.data_type = type_name.to_string();
                                }
                            }
                        });
                    ui.end_row();

                    if needs_length(&col.data_type) {
                        ui.label("Length:");
                        ui.add(
                            theme::text_input(col.length.get_or_insert_with(String::new))
                                .desired_width(80.0)
                                .hint_text("e.g., 255"),
                        );
                        ui.end_row();
                    }

                    ui.label("Default:");
                    ui.add(theme::text_input(&mut col.default_value).desired_width(180.0));
                    ui.end_row();

                    ui.label("");
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut col.is_nullable, "Nullable");
                        ui.checkbox(&mut col.is_primary_key, "Primary Key");
                        ui.checkbox(&mut col.is_unique, "Unique");
                    });
                    ui.end_row();

                    ui.label("");
                    ui.checkbox(&mut col.is_foreign_key, "Foreign Key");
                    ui.end_row();

                    if col.is_foreign_key {
                        ui.label("References:");
                        ui.horizontal(|ui| {
                            ComboBox::from_id_salt("fk_schema")
                                .width(100.0)
                                .selected_text(&col.fk_ref_schema)
                                .show_ui(ui, |ui| {
                                    for schema in schemas {
                                        if ui
                                            .selectable_label(&col.fk_ref_schema == schema, schema)
                                            .clicked()
                                        {
                                            col.fk_ref_schema.clone_from(schema);
                                        }
                                    }
                                });

                            ui.add(
                                theme::text_input(&mut col.fk_ref_table)
                                    .desired_width(100.0)
                                    .hint_text("table"),
                            );
                            ui.add(
                                theme::text_input(&mut col.fk_ref_column)
                                    .desired_width(100.0)
                                    .hint_text("column"),
                            );
                        });
                        ui.end_row();
                    }
                });
        });
}

fn render_indexes_panel(ui: &mut Ui, state: &mut AppState) {
    let column_names: Vec<String> = state
        .table_designer
        .columns
        .iter()
        .filter(|col| !col.name.is_empty())
        .map(|col| col.name.clone())
        .collect();

    Frame::new()
        .fill(theme::bg_dark())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.strong("Indexes");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(small_button("+")).clicked() {
                        let idx = IndexDef::default();
                        state.table_designer.indexes.push(idx);
                    }
                });
            });

            ui.add_space(8.0);

            ScrollArea::vertical()
                .id_salt("indexes_list")
                .show(ui, |ui| {
                    let mut to_delete = None;

                    for (idx, index) in state.table_designer.indexes.iter_mut().enumerate() {
                        Frame::new()
                            .fill(theme::bg_medium())
                            .inner_margin(Margin::same(6))
                            .corner_radius(CornerRadius::same(4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.add(
                                        theme::text_input(&mut index.name)
                                            .desired_width(130.0)
                                            .hint_text("idx_name"),
                                    );
                                    ui.checkbox(&mut index.is_unique, "Unique");
                                    ComboBox::from_id_salt(format!("idx_type_{idx}"))
                                        .width(96.0)
                                        .selected_text(&index.index_type)
                                        .show_ui(ui, |ui| {
                                            for index_type in
                                                ["BTREE", "HASH", "GIN", "GIST", "BRIN"]
                                            {
                                                if ui
                                                    .selectable_label(
                                                        index.index_type == index_type,
                                                        index_type,
                                                    )
                                                    .clicked()
                                                {
                                                    index.index_type = index_type.to_string();
                                                }
                                            }
                                        });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| {
                                            if ui.button("Delete").clicked() {
                                                to_delete = Some(idx);
                                            }
                                        },
                                    );
                                });

                                ui.add_space(4.0);
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new("Columns:")
                                            .color(theme::text_muted())
                                            .size(10.0),
                                    );
                                    for column in &column_names {
                                        let mut selected = index.columns.contains(column);
                                        if ui.checkbox(&mut selected, column).changed() {
                                            if selected {
                                                index.columns.push(column.clone());
                                            } else {
                                                index.columns.retain(|c| c != column);
                                            }
                                        }
                                    }
                                });
                            });
                    }

                    if let Some(idx) = to_delete {
                        state.table_designer.indexes.remove(idx);
                    }
                });
        });
}

fn needs_length(data_type: &str) -> bool {
    matches!(
        data_type.to_uppercase().as_str(),
        "VARCHAR" | "CHAR" | "NUMERIC" | "DECIMAL" | "VARBIT"
    )
}

fn sanitize_identifier(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "")
}

fn generate_ddl(state: &AppState) -> String {
    let td = &state.table_designer;

    if td.table_name.is_empty() || td.schema.is_empty() {
        return "-- Please specify schema and table name".to_string();
    }

    let table_ref = format!(
        "{}.{}",
        escape_identifier(&td.schema),
        escape_identifier(&td.table_name)
    );

    let mut ddl = if td.editing_table.is_some() {
        generate_alter_ddl(td, &table_ref)
    } else {
        generate_create_ddl(td, &table_ref)
    };

    for idx in &td.indexes {
        if idx.name.is_empty() || idx.columns.is_empty() {
            continue;
        }

        let unique_str = if idx.is_unique { "UNIQUE " } else { "" };
        ddl.push_str(&format!(
            "\nCREATE {unique_str}INDEX IF NOT EXISTS {} ON {} USING {} ({});",
            escape_identifier(&idx.name),
            table_ref,
            idx.index_type.to_lowercase(),
            idx.columns
                .iter()
                .map(|c| escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some((old_schema, old_table)) = &td.editing_table {
        if old_schema != &td.schema || old_table != &td.table_name {
            ddl.push_str(&format!(
                "\n-- To rename table:\n-- ALTER TABLE {}.{} RENAME TO {}.{};",
                escape_identifier(old_schema),
                escape_identifier(old_table),
                escape_identifier(&td.schema),
                escape_identifier(&td.table_name)
            ));
        }
    }

    ddl
}

fn generate_create_ddl(td: &TableDesignerState, table_ref: &str) -> String {
    let mut defs: Vec<String> = Vec::new();
    let mut pk_columns: Vec<&str> = Vec::new();

    for col in &td.columns {
        if col.name.is_empty() {
            continue;
        }
        if col.is_primary_key {
            pk_columns.push(&col.name);
        }
        defs.push(format!("    {}", column_definition_sql(col, true)));
    }

    if !pk_columns.is_empty() {
        defs.push(format!(
            "    CONSTRAINT {}_pk PRIMARY KEY ({})",
            escape_identifier(&td.table_name),
            pk_columns
                .iter()
                .map(|c| escape_identifier(c))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    for col in &td.columns {
        if col.is_foreign_key && !col.fk_ref_table.is_empty() && !col.fk_ref_column.is_empty() {
            defs.push(format!(
                "    CONSTRAINT {}_{}_fk FOREIGN KEY ({}) REFERENCES {}.{}({})",
                escape_identifier(&td.table_name),
                escape_identifier(&col.name),
                escape_identifier(&col.name),
                escape_identifier(&col.fk_ref_schema),
                escape_identifier(&col.fk_ref_table),
                escape_identifier(&col.fk_ref_column)
            ));
        }
    }

    if defs.is_empty() {
        return "-- Add at least one column before generating DDL".to_string();
    }

    format!(
        "CREATE TABLE IF NOT EXISTS {table_ref} (\n{}\n);\n",
        defs.join(",\n")
    )
}

fn generate_alter_ddl(td: &TableDesignerState, table_ref: &str) -> String {
    let mut statements = Vec::new();

    let orig_by_name: std::collections::HashMap<&str, &ColumnDef> = td
        .original_columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let current_names: std::collections::HashSet<&str> =
        td.columns.iter().map(|c| c.name.as_str()).collect();

    // Dropped columns
    for orig in &td.original_columns {
        if !orig.name.is_empty() && !current_names.contains(orig.name.as_str()) {
            statements.push(format!(
                "ALTER TABLE {table_ref} DROP COLUMN {};",
                escape_identifier(&orig.name)
            ));
        }
    }

    for col in &td.columns {
        if col.name.is_empty() {
            continue;
        }
        let ident = escape_identifier(&col.name);

        match orig_by_name.get(col.name.as_str()) {
            None => {
                // New column
                statements.push(format!(
                    "ALTER TABLE {table_ref} ADD COLUMN {};",
                    column_definition_sql(col, false)
                ));
            }
            Some(orig) => {
                if *orig == col {
                    continue;
                }
                // Type change
                if orig.data_type != col.data_type || orig.length != col.length {
                    statements.push(format!(
                        "ALTER TABLE {table_ref} ALTER COLUMN {} TYPE {} USING {}::{};",
                        ident,
                        col_type_sql(col),
                        ident,
                        col_type_sql(col)
                    ));
                }
                // Nullability change
                if orig.is_nullable != col.is_nullable {
                    statements.push(format!(
                        "ALTER TABLE {table_ref} ALTER COLUMN {} {};",
                        ident,
                        if col.is_nullable { "DROP NOT NULL" } else { "SET NOT NULL" }
                    ));
                }
                // Default change
                if orig.default_value != col.default_value {
                    if col.default_value.is_empty() {
                        statements.push(format!(
                            "ALTER TABLE {table_ref} ALTER COLUMN {} DROP DEFAULT;",
                            ident
                        ));
                    } else {
                        statements.push(format!(
                            "ALTER TABLE {table_ref} ALTER COLUMN {} SET DEFAULT {};",
                            ident, col.default_value
                        ));
                    }
                }
            }
        }
    }

    if statements.is_empty() {
        "-- No changes detected".to_string()
    } else {
        statements.join("\n") + "\n"
    }
}

fn column_definition_sql(col: &ColumnDef, include_inline_unique: bool) -> String {
    let mut sql = format!("{} {}", escape_identifier(&col.name), col_type_sql(col));

    if !col.is_nullable {
        sql.push_str(" NOT NULL");
    }
    if !col.default_value.is_empty() {
        sql.push_str(" DEFAULT ");
        sql.push_str(&col.default_value);
    }
    if include_inline_unique && col.is_unique && !col.is_primary_key {
        sql.push_str(" UNIQUE");
    }

    sql
}

fn col_type_sql(col: &ColumnDef) -> String {
    if let Some(ref len) = col.length {
        if !len.trim().is_empty() {
            return format!("{}({})", col.data_type, len.trim());
        }
    }
    col.data_type.clone()
}

fn escape_identifier(name: &str) -> String {
    if name.contains('"') || name.contains(' ') || name.starts_with(|c: char| c.is_numeric()) {
        format!("\"{}\"", name.replace('"', "\"\""))
    } else {
        name.to_string()
    }
}

fn primary_button(text: &str) -> Button<'_> {
    theme::primary_button(text)
}

fn small_button(text: &str) -> Button<'_> {
    Button::new(text)
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .min_size(Vec2::new(28.0, 28.0))
}

fn type_chip(ui: &mut Ui, label: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(25.0, 17.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        theme::with_alpha(color, 28),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(9.5),
        color,
    );
}

fn apply_ddl(state: &mut AppState, bridge: &DbBridge) {
    let conn_id = match state.active_connection {
        Some(id) => id,
        None => {
            state.last_error = Some("No active connection".to_string());
            return;
        }
    };

    let ddl = state
        .table_designer
        .generated_ddl
        .clone()
        .unwrap_or_else(|| generate_ddl(state));

    if ddl.trim_start().starts_with("--") {
        state.table_designer.generated_ddl = Some(ddl.clone());
        state.table_designer.show_ddl_preview = true;
        state.last_error = Some(ddl.trim_start_matches('-').trim().to_string());
        return;
    }

    let schema = state.table_designer.schema.clone();
    state.table_designer.generated_ddl = Some(ddl.clone());
    state.status_message = format!(
        "Applying DDL to {schema}.{}",
        state.table_designer.table_name
    );
    state.query_running = true;

    // Plan v7 Phase 2b — 2-step NOTIFY DDL (pre_drop → 1s ack → DDL → post_drop).
    // table_oid 는 editing_table 의 경우 향후 metadata 에서 회수 가능 (현재는 None).
    // 자동 ListTables refresh 는 connection_task 가 수행.
    bridge.send(DbCommand::ApplyDdlWithInvalidation {
        conn_id,
        sql: ddl,
        table_oid: None,
        schema_to_refresh: Some(schema),
    });
}

pub fn open_for_new_table(state: &mut AppState) {
    state.table_designer = TableDesignerState {
        show: true,
        schema: state
            .active_connection
            .and_then(|id| state.connections.get(&id))
            .and_then(|c| c.schemas.first().cloned())
            .unwrap_or_default(),
        table_name: String::new(),
        columns: vec![ColumnDef::default()],
        indexes: Vec::new(),
        selected_column: Some(0),
        generated_ddl: None,
        show_ddl_preview: false,
        editing_table: None,
        original_columns: Vec::new(),
    };
}

pub fn open_for_new_table_with_schema(state: &mut AppState, schema: &str) {
    state.table_designer = TableDesignerState {
        show: true,
        schema: schema.to_string(),
        table_name: String::new(),
        columns: vec![ColumnDef::default()],
        indexes: Vec::new(),
        selected_column: Some(0),
        generated_ddl: None,
        show_ddl_preview: false,
        editing_table: None,
        original_columns: Vec::new(),
    };
}

pub fn open_for_existing_table(state: &mut AppState, schema: &str, table: &str, bridge: &DbBridge) {
    let conn_id = match state.active_connection {
        Some(id) => id,
        None => return,
    };

    let conn = match state.connections.get(&conn_id) {
        Some(c) => c,
        None => return,
    };

    let columns = conn
        .columns
        .get(&(schema.to_string(), table.to_string()))
        .cloned()
        .unwrap_or_default();

    let column_defs: Vec<ColumnDef> = columns
        .iter()
        .map(|c| ColumnDef {
            name: c.name.clone(),
            data_type: c.data_type.clone(),
            length: None,
            is_nullable: c.is_nullable,
            is_primary_key: c.is_primary_key,
            is_unique: false,
            default_value: c.default_value.clone().unwrap_or_default(),
            is_foreign_key: false,
            fk_ref_schema: String::new(),
            fk_ref_table: String::new(),
            fk_ref_column: String::new(),
        })
        .collect();

    let original_columns = column_defs.clone();
    state.table_designer = TableDesignerState {
        show: true,
        schema: schema.to_string(),
        table_name: table.to_string(),
        columns: column_defs,
        indexes: Vec::new(),
        selected_column: None,
        generated_ddl: None,
        show_ddl_preview: false,
        editing_table: Some((schema.to_string(), table.to_string())),
        original_columns,
    };

    bridge.send(crate::db::bridge::DbCommand::ListForeignKeys {
        conn_id,
        schema: schema.to_string(),
    });
}

#[cfg(test)]
mod alter_ddl_tests {
    use super::*;

    fn col(name: &str, dtype: &str, nullable: bool, default: &str) -> ColumnDef {
        ColumnDef {
            name: name.to_string(),
            data_type: dtype.to_string(),
            length: None,
            is_nullable: nullable,
            is_primary_key: false,
            is_unique: false,
            default_value: default.to_string(),
            is_foreign_key: false,
            fk_ref_schema: String::new(),
            fk_ref_table: String::new(),
            fk_ref_column: String::new(),
        }
    }

    fn designer(original: Vec<ColumnDef>, current: Vec<ColumnDef>) -> TableDesignerState {
        TableDesignerState {
            schema: "public".to_string(),
            table_name: "users".to_string(),
            original_columns: original,
            columns: current,
            editing_table: Some(("public".to_string(), "users".to_string())),
            ..Default::default()
        }
    }

    #[test]
    fn no_changes_produces_no_statements() {
        let cols = vec![col("id", "INTEGER", false, ""), col("name", "TEXT", true, "")];
        let td = designer(cols.clone(), cols);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert_eq!(ddl, "-- No changes detected");
    }

    #[test]
    fn detects_added_column() {
        let orig = vec![col("id", "INTEGER", false, "")];
        let current = vec![col("id", "INTEGER", false, ""), col("email", "TEXT", true, "")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("ADD COLUMN email TEXT"));
    }

    #[test]
    fn detects_dropped_column() {
        let orig = vec![col("id", "INTEGER", false, ""), col("old_col", "TEXT", true, "")];
        let current = vec![col("id", "INTEGER", false, "")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("DROP COLUMN old_col"));
    }

    #[test]
    fn detects_type_change() {
        let orig = vec![col("age", "INTEGER", true, "")];
        let current = vec![col("age", "BIGINT", true, "")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("ALTER COLUMN age TYPE BIGINT"));
    }

    #[test]
    fn detects_nullability_change() {
        let orig = vec![col("name", "TEXT", true, "")];
        let current = vec![col("name", "TEXT", false, "")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("SET NOT NULL"));
    }

    #[test]
    fn detects_default_change() {
        let orig = vec![col("status", "TEXT", true, "")];
        let current = vec![col("status", "TEXT", true, "'active'")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("SET DEFAULT 'active'"));
    }

    #[test]
    fn detects_default_removal() {
        let orig = vec![col("status", "TEXT", true, "'active'")];
        let current = vec![col("status", "TEXT", true, "")];
        let td = designer(orig, current);
        let ddl = generate_alter_ddl(&td, "public.users");
        assert!(ddl.contains("DROP DEFAULT"));
    }
}

pub fn apply_fk_info(state: &mut AppState, foreign_keys: &[crate::ui::er_diagram::ForeignKey]) {
    for col in &mut state.table_designer.columns {
        for fk in foreign_keys {
            if fk.source_table == state.table_designer.table_name
                && fk.source_schema == state.table_designer.schema
                && fk.source_column == col.name
            {
                col.is_foreign_key = true;
                col.fk_ref_schema.clone_from(&fk.target_schema);
                col.fk_ref_table.clone_from(&fk.target_table);
                col.fk_ref_column.clone_from(&fk.target_column);
            }
        }
    }
}
