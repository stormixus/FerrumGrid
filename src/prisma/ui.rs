use eframe::egui::{
    self, Align, Button, Color32, CornerRadius, Frame, Margin, RichText,
    ScrollArea, Stroke, TextEdit, Window,
};

use crate::db::bridge::DbBridge;
use crate::prisma::{
    check_prisma_installed, generate_schema_file, run_prisma_cli, PrismaCommand,
};
use crate::state::{AppState, ConnectionStatus};
use crate::ui::theme;

#[derive(Debug, Clone, Default)]
pub struct PrismaUIState {
    pub show_window: bool,
    pub schema_content: String,
    pub schema_path: String,
    pub cli_output: String,
    pub selected_command: PrismaCommandType,
    pub migration_name: String,
    pub is_running: bool,
    pub prisma_installed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrismaCommandType {
    Introspect,
    MigrateDev,
    MigrateDeploy,
    MigrateStatus,
    Generate,
    Validate,
    DBPull,
    DBPush,
}

impl Default for PrismaCommandType {
    fn default() -> Self {
        PrismaCommandType::Introspect
    }
}

pub fn render_prisma_window(ctx: &egui::Context, state: &mut AppState, _bridge: &DbBridge) {
    if !state.prisma_ui.show_window {
        return;
    }

    // Check Prisma installation once - run async check synchronously
    if !state.prisma_ui.prisma_installed && !state.prisma_ui.is_running {
        // Use a simple runtime for the check
        let rt = tokio::runtime::Runtime::new().unwrap();
        state.prisma_ui.prisma_installed = rt.block_on(check_prisma_installed());
    }

    Window::new("Prisma Integration")
        .default_size([900.0, 700.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            render_prisma_ui(ui, state, _bridge);
        });
}

fn render_prisma_ui(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    if !state.prisma_ui.prisma_installed {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label(
                RichText::new("Prisma CLI not found")
                    .color(theme::ACCENT_RED)
                    .size(18.0),
            );
            ui.label("Please install Prisma CLI:");
            ui.label("npm install -g prisma");
            ui.add_space(20.0);
            if ui.button("Check Again").clicked() {
                let rt = tokio::runtime::Runtime::new().unwrap();
                state.prisma_ui.prisma_installed = rt.block_on(check_prisma_installed());
            }
        });
        return;
    }

    // Top toolbar
    ui.horizontal(|ui| {
        ui.label("Schema Path:");
        ui.add(
            TextEdit::singleline(&mut state.prisma_ui.schema_path)
                .desired_width(300.0)
                .hint_text("./prisma/schema.prisma"),
        );

        if ui.button("Browse").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Prisma Schema", &["prisma"])
                .pick_file()
            {
                state.prisma_ui.schema_path = path.to_string_lossy().to_string();
            }
        }

        ui.separator();

        if ui.button("Load").clicked() {
            load_schema_file(state);
        }

        if ui.button("Save").clicked() {
            save_schema_file(state);
        }

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if ui.button("× Close").clicked() {
                state.prisma_ui.show_window = false;
            }
        });
    });

    ui.separator();

    // Main content area
    ui.columns(2, |cols| {
        // Left panel - Schema editor
        cols[0].vertical(|ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.strong("Schema");
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(primary_button("Create from DB")).clicked() {
                        create_schema_from_db(state, bridge);
                    }
                });
            });

            ui.add_space(8.0);

            Frame::new()
                .fill(theme::BG_DARKEST)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    let available = ui.available_size();
                    ui.add_sized(
                        available,
                        TextEdit::multiline(&mut state.prisma_ui.schema_content).code_editor(),
                    );
                });
        });

        // Right panel - Commands and output
        cols[1].vertical(|ui| {
            ui.set_min_width(ui.available_width());

            ui.strong("Prisma Commands");
            ui.add_space(8.0);

            render_commands_panel(ui, state, bridge);

            ui.add_space(16.0);

            ui.strong("Output");
            ui.add_space(8.0);

            Frame::new()
                .fill(theme::BG_DARKEST)
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut state.prisma_ui.cli_output.as_str())
                                .code_editor()
                                .desired_rows(20),
                        );
                    });
                });
        });
    });
}

fn render_commands_panel(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let commands = [
        (PrismaCommandType::Introspect, "Introspect", "Pull DB schema into Prisma schema"),
        (PrismaCommandType::MigrateDev, "Migrate Dev", "Create and apply migration"),
        (PrismaCommandType::MigrateDeploy, "Migrate Deploy", "Deploy pending migrations"),
        (PrismaCommandType::MigrateStatus, "Migrate Status", "Check migration status"),
        (PrismaCommandType::Generate, "Generate", "Generate Prisma Client"),
        (PrismaCommandType::Validate, "Validate", "Validate schema"),
        (PrismaCommandType::DBPull, "DB Pull", "Introspect without updating schema"),
        (PrismaCommandType::DBPush, "DB Push", "Push schema to database"),
    ];

    for (cmd_type, label, tooltip) in &commands {
        ui.horizontal(|ui| {
            let selected = state.prisma_ui.selected_command == *cmd_type;

            let btn = Button::new(*label)
                .fill(if selected {
                    theme::ACCENT_COPPER
                } else {
                    theme::BG_LIGHT
                })
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                .corner_radius(CornerRadius::same(4));

            if ui.add(btn).on_hover_text(*tooltip).clicked() {
                state.prisma_ui.selected_command = *cmd_type;
            }

            // Additional inputs for specific commands
            match cmd_type {
                PrismaCommandType::MigrateDev => {
                    ui.add(
                        TextEdit::singleline(&mut state.prisma_ui.migration_name)
                            .desired_width(150.0)
                            .hint_text("migration_name"),
                    );
                }
                _ => {}
            }

            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if state.prisma_ui.is_running {
                    ui.spinner();
                } else if ui.button("Run").clicked() {
                    run_selected_command(state, bridge);
                }
            });
        });
        ui.add_space(4.0);
    }
}

fn run_selected_command(state: &mut AppState, _bridge: &DbBridge) {
    let schema_path = state.prisma_ui.schema_path.clone();

    if schema_path.is_empty() {
        state.prisma_ui.cli_output = "Error: Please specify schema path".to_string();
        return;
    }

    let cmd = match state.prisma_ui.selected_command {
        PrismaCommandType::Introspect => PrismaCommand::Introspect {
            schema_path,
            output_path: None,
        },
        PrismaCommandType::MigrateDev => PrismaCommand::Migrate {
            schema_path,
            name: state
                .prisma_ui
                .migration_name
                .clone()
                .replace(' ', "_")
                .to_lowercase(),
            create_only: false,
        },
        PrismaCommandType::MigrateDeploy => PrismaCommand::MigrateDeploy { schema_path },
        PrismaCommandType::MigrateStatus => PrismaCommand::MigrateStatus { schema_path },
        PrismaCommandType::Generate => PrismaCommand::Generate { schema_path },
        PrismaCommandType::Validate => PrismaCommand::Validate { schema_path },
        PrismaCommandType::DBPull => PrismaCommand::DBPull { schema_path },
        PrismaCommandType::DBPush => PrismaCommand::DBPush {
            schema_path,
            force: false,
        },
    };

    state.prisma_ui.is_running = true;
    state.prisma_ui.cli_output = "Running...".to_string();

    // Run async in background
    let ctx = _bridge.clone();
    tokio::spawn(async move {
        match run_prisma_cli(cmd).await {
            Ok(result) => {
                let output = format!(
                    "Exit Code: {:?}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                    result.exit_code, result.stdout, result.stderr
                );
                // Update UI (would need proper channel for thread safety)
            }
            Err(e) => {
                // Update UI with error
            }
        }
    });
}

fn load_schema_file(state: &mut AppState) {
    let path = &state.prisma_ui.schema_path;
    if path.is_empty() {
        state.prisma_ui.cli_output = "Error: Please specify schema path".to_string();
        return;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => {
            state.prisma_ui.schema_content = content;
            state.prisma_ui.cli_output = format!("Loaded schema from: {}", path);
        }
        Err(e) => {
            state.prisma_ui.cli_output = format!("Error loading file: {}", e);
        }
    }
}

fn save_schema_file(state: &mut AppState) {
    let path = &state.prisma_ui.schema_path;
    if path.is_empty() {
        state.prisma_ui.cli_output = "Error: Please specify schema path".to_string();
        return;
    }

    match std::fs::write(path, &state.prisma_ui.schema_content) {
        Ok(()) => {
            state.prisma_ui.cli_output = format!("Saved schema to: {}", path);
        }
        Err(e) => {
            state.prisma_ui.cli_output = format!("Error saving file: {}", e);
        }
    }
}

fn create_schema_from_db(state: &mut AppState, bridge: &DbBridge) {
    let conn_id = match state.active_connection {
        Some(id) => id,
        None => {
            state.prisma_ui.cli_output = "Error: No active connection".to_string();
            return;
        }
    };

    let conn = match state.connections.get(&conn_id) {
        Some(c) if matches!(c.status, ConnectionStatus::Connected { .. }) => c,
        _ => {
            state.prisma_ui.cli_output = "Error: Not connected".to_string();
            return;
        }
    };

    let schema_name = conn.schemas.first().cloned().unwrap_or_default();

    // Build schema manually from DB metadata
    let mut schema_lines = vec![
        "// Generated by FerrumGrid".to_string(),
        "".to_string(),
        "generator client {".to_string(),
        "  provider = \"prisma-client-js\"".to_string(),
        "}".to_string(),
        "".to_string(),
        "datasource db {".to_string(),
        "  provider = \"postgresql\"".to_string(),
        "  url      = env(\"DATABASE_URL\")".to_string(),
        "}".to_string(),
        "".to_string(),
    ];

    for table in conn.tables.get(&schema_name).cloned().unwrap_or_default() {
        let key = (schema_name.clone(), table.name.clone());
        let columns = conn.columns.get(&key).cloned().unwrap_or_default();

        schema_lines.push(format!("model {} {{", table.name));

        for col in &columns {
            let prisma_type = db_type_to_prisma(&col.data_type, col.is_nullable);
            let attrs = if col.is_primary_key {
                " @id"
            } else {
                ""
            };
            schema_lines.push(format!("  {} {}{}", col.name, prisma_type, attrs));
        }

        schema_lines.push("}".to_string());
        schema_lines.push("".to_string());
    }

    state.prisma_ui.schema_content = schema_lines.join("\n");
    state.prisma_ui.cli_output =
        format!("Generated schema from database '{}'", schema_name);
}

fn db_type_to_prisma(db_type: &str, is_nullable: bool) -> String {
    let base = match db_type.to_lowercase().as_str() {
        "character varying" | "varchar" | "text" | "char" => "String",
        "integer" | "int" | "int4" | "serial" => "Int",
        "bigint" | "int8" | "bigserial" => "BigInt",
        "numeric" | "decimal" => "Decimal",
        "real" | "float4" => "Float",
        "double precision" | "float8" => "Float",
        "boolean" | "bool" => "Boolean",
        "timestamp" | "timestamptz" | "timestamp without time zone" | "timestamp with time zone" => {
            "DateTime"
        }
        "date" => "DateTime",
        "time" => "DateTime",
        "json" | "jsonb" => "Json",
        "bytea" => "Bytes",
        "uuid" => "String",
        _ => "String",
    };

    if is_nullable {
        format!("{}?", base)
    } else {
        base.to_string()
    }
}

fn primary_button(text: &str) -> Button<'_> {
    Button::new(RichText::new(text).color(Color32::WHITE))
        .fill(theme::ACCENT_COPPER)
        .stroke(Stroke::new(1.0, theme::ACCENT_COPPER_LIGHT))
        .corner_radius(CornerRadius::same(4))
}

pub fn open_prisma_window(state: &mut AppState) {
    state.prisma_ui.show_window = true;
    // Check Prisma installation
    let rt = tokio::runtime::Runtime::new().unwrap();
    state.prisma_ui.prisma_installed = rt.block_on(check_prisma_installed());
}
