use eframe::egui::{
    self, Align, Button, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, Window,
};
use std::sync::{Arc, Mutex};

use crate::db::bridge::DbBridge;
use crate::prisma::{
    append_model_to_schema, check_prisma_installed, generate_migration, generate_schema_file,
    get_prisma_version, run_prisma_cli, sync_db_to_schema, sync_schema_to_db, PrismaCommand,
    PrismaSchema,
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
    pub prisma_version: Option<String>,
    pub pending_output: Option<Arc<Mutex<Option<String>>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PrismaCommandType {
    #[default]
    Introspect,
    MigrateDev,
    MigrateDeploy,
    MigrateStatus,
    Generate,
    Validate,
    Format,
    DBPull,
    DBPush,
}

pub fn render_prisma_window(ctx: &egui::Context, state: &mut AppState, _bridge: &DbBridge) {
    if !state.prisma_ui.show_window {
        return;
    }

    if state.prisma_ui.schema_path.is_empty() {
        state.prisma_ui.schema_path = "./prisma/schema.prisma".to_string();
    }

    poll_prisma_output(ctx, state);

    // Check Prisma installation once - run async check synchronously
    if !state.prisma_ui.prisma_installed && !state.prisma_ui.is_running {
        // Use a simple runtime for the check
        let rt = tokio::runtime::Runtime::new().unwrap();
        state.prisma_ui.prisma_installed = rt.block_on(check_prisma_installed());
        state.prisma_ui.prisma_version = rt.block_on(get_prisma_version());
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
                state.prisma_ui.prisma_version = rt.block_on(get_prisma_version());
            }
        });
        return;
    }

    // Top toolbar
    ui.horizontal(|ui| {
        ui.label("Schema Path:");
        ui.add(
            theme::text_input(&mut state.prisma_ui.schema_path)
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

        if ui.button("Create File").clicked() {
            create_schema_file(state);
        }

        if ui.button("Save").clicked() {
            save_schema_file(state);
        }

        if ui.button("Append Table Model").clicked() {
            append_table_model(state);
        }

        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            if ui.button("× Close").clicked() {
                state.prisma_ui.show_window = false;
            } else if let Some(version) = &state.prisma_ui.prisma_version {
                ui.label(RichText::new(version).color(theme::text_muted()).size(10.0));
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
                    if ui.add(theme::primary_button("Create from DB")).clicked() {
                        create_schema_from_db(state, bridge);
                    }
                    if ui.add(theme::primary_button("Preview SQL")).clicked() {
                        preview_prisma_sql(state);
                    }
                    if ui.add(theme::primary_button("Apply SQL")).clicked() {
                        apply_prisma_schema(state, bridge);
                    }
                });
            });

            ui.add_space(8.0);

            Frame::new()
                .fill(theme::bg_darkest())
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    let available = ui.available_size();
                    ui.add_sized(
                        available,
                        theme::multiline_mono_text_input(&mut state.prisma_ui.schema_content)
                            .code_editor(),
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
                .fill(theme::bg_darkest())
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        let mut output = state.prisma_ui.cli_output.clone();
                        ui.add(
                            theme::multiline_mono_text_input(&mut output)
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
        (
            PrismaCommandType::Introspect,
            "Introspect",
            "Pull DB schema into Prisma schema",
        ),
        (
            PrismaCommandType::MigrateDev,
            "Migrate Dev",
            "Create and apply migration",
        ),
        (
            PrismaCommandType::MigrateDeploy,
            "Migrate Deploy",
            "Deploy pending migrations",
        ),
        (
            PrismaCommandType::MigrateStatus,
            "Migrate Status",
            "Check migration status",
        ),
        (
            PrismaCommandType::Generate,
            "Generate",
            "Generate Prisma Client",
        ),
        (PrismaCommandType::Validate, "Validate", "Validate schema"),
        (PrismaCommandType::Format, "Format", "Format schema file"),
        (
            PrismaCommandType::DBPull,
            "DB Pull",
            "Introspect without updating schema",
        ),
        (
            PrismaCommandType::DBPush,
            "DB Push",
            "Push schema to database",
        ),
    ];

    for (cmd_type, label, tooltip) in &commands {
        ui.horizontal(|ui| {
            let selected = state.prisma_ui.selected_command == *cmd_type;

            let btn = Button::new(*label)
                .fill(if selected {
                    theme::ACCENT_COPPER
                } else {
                    theme::bg_light()
                })
                .stroke(Stroke::new(1.0, theme::border_default()))
                .corner_radius(CornerRadius::same(4));

            if ui.add(btn).on_hover_text(*tooltip).clicked() {
                state.prisma_ui.selected_command = *cmd_type;
            }

            // Additional inputs for specific commands
            if cmd_type == &PrismaCommandType::MigrateDev {
                ui.add(
                    theme::text_input(&mut state.prisma_ui.migration_name)
                        .desired_width(150.0)
                        .hint_text("migration_name"),
                );
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
        PrismaCommandType::Format => PrismaCommand::Format { schema_path },
        PrismaCommandType::DBPull => PrismaCommand::DBPull { schema_path },
        PrismaCommandType::DBPush => PrismaCommand::DBPush {
            schema_path,
            force: false,
        },
    };

    state.prisma_ui.is_running = true;
    state.prisma_ui.cli_output = "Running...".to_string();
    let pending_output = Arc::new(Mutex::new(None));
    state.prisma_ui.pending_output = Some(pending_output.clone());

    std::thread::spawn(move || {
        let output = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to start Prisma runtime: {e}"))
            .and_then(|rt| {
                rt.block_on(async move {
                    match run_prisma_cli(cmd).await {
                        Ok(result) => Ok(format!(
                            "Success: {}\nExit Code: {:?}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                            result.success, result.exit_code, result.stdout, result.stderr
                        )),
                        Err(e) => Err(format!("Prisma command failed: {e}")),
                    }
                })
            });

        if let Ok(mut slot) = pending_output.lock() {
            *slot = Some(match output {
                Ok(output) => output,
                Err(error) => error,
            });
        }
    });
}

fn poll_prisma_output(ctx: &egui::Context, state: &mut AppState) {
    let Some(slot) = state.prisma_ui.pending_output.clone() else {
        return;
    };

    let output = slot.lock().ok().and_then(|mut guard| guard.take());
    if let Some(output) = output {
        state.prisma_ui.cli_output = output;
        state.prisma_ui.is_running = false;
        state.prisma_ui.pending_output = None;
    } else {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn create_schema_file(state: &mut AppState) {
    let path = state.prisma_ui.schema_path.clone();
    if path.is_empty() {
        state.prisma_ui.cli_output = "Error: Please specify schema path".to_string();
        return;
    }

    let conn_string = state
        .active_connection
        .and_then(|id| state.connections.get(&id))
        .map(|conn| {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                conn.config.username,
                conn.config.password,
                conn.config.host,
                conn.config.port,
                conn.config.database
            )
        })
        .unwrap_or_default();

    match generate_schema_file("postgresql", &conn_string, &path) {
        Ok(schema) => {
            state.prisma_ui.schema_content = schema;
            state.prisma_ui.cli_output = format!("Created schema file: {path}");
        }
        Err(e) => state.prisma_ui.cli_output = e,
    }
}

fn append_table_model(state: &mut AppState) {
    let path = state.prisma_ui.schema_path.clone();
    let Some(ddl) = state.table_designer.generated_ddl.clone() else {
        state.prisma_ui.cli_output = "Error: No generated table DDL to append".to_string();
        return;
    };

    match append_model_to_schema(&path, &ddl) {
        Ok(()) => {
            load_schema_file(state);
            state.prisma_ui.cli_output = "Appended generated table model".to_string();
        }
        Err(e) => state.prisma_ui.cli_output = e,
    }
}

fn preview_prisma_sql(state: &mut AppState) {
    match PrismaSchema::parse(&state.prisma_ui.schema_content) {
        Ok(schema) => {
            let name = if state.prisma_ui.migration_name.trim().is_empty() {
                "preview".to_string()
            } else {
                state.prisma_ui.migration_name.trim().replace(' ', "_")
            };
            let sql = schema.to_sql();
            state.prisma_ui.cli_output = format!(
                "{}\n\n-- Direct SQL preview\n{}",
                generate_migration(None, &schema, &name),
                sql
            );
        }
        Err(e) => {
            state.prisma_ui.cli_output = e;
        }
    }
}

fn apply_prisma_schema(state: &mut AppState, bridge: &DbBridge) {
    let conn_id = match state.active_connection {
        Some(id) => id,
        None => {
            state.prisma_ui.cli_output = "Error: No active connection".to_string();
            return;
        }
    };

    if !state
        .connections
        .get(&conn_id)
        .is_some_and(|conn| matches!(conn.status, ConnectionStatus::Connected { .. }))
    {
        state.prisma_ui.cli_output = "Error: Not connected".to_string();
        return;
    }

    match PrismaSchema::parse(&state.prisma_ui.schema_content) {
        Ok(schema) => {
            let result = sync_schema_to_db(&schema, conn_id, bridge);
            state.prisma_ui.cli_output = format!(
                "Success: {}\n{}\n\n{}",
                result.success,
                result.message,
                result.sql_statements.join("\n")
            );
            state.query_running = true;
        }
        Err(e) => {
            state.prisma_ui.cli_output = e;
        }
    }
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

fn create_schema_from_db(state: &mut AppState, _bridge: &DbBridge) {
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

    let schema_name = if state.objects_schema_filter.is_empty() {
        conn.schemas
            .iter()
            .find(|schema| schema.as_str() == "public")
            .cloned()
            .or_else(|| conn.schemas.first().cloned())
            .unwrap_or_default()
    } else {
        state.objects_schema_filter.clone()
    };

    let fallback_schema = conn
        .tables
        .get(&schema_name)
        .map(|tables| PrismaSchema::from_db_schema(&schema_name, tables, &conn.columns));

    match sync_db_to_schema(state, &schema_name, conn_id)
        .or_else(|_| fallback_schema.ok_or_else(|| "No tables found".to_string()))
    {
        Ok(schema) => {
            state.prisma_ui.schema_content = format!(
                "// Generated by FerrumGrid from schema '{}'\n\n{}",
                schema_name,
                schema.to_prisma_schema()
            );
            state.prisma_ui.cli_output = format!("Generated schema from database '{schema_name}'");
        }
        Err(e) => {
            state.prisma_ui.cli_output = e;
        }
    }
}

#[allow(dead_code)] // Only invoked from the macOS native menu (cfg-gated).
pub fn open_prisma_window(state: &mut AppState) {
    state.prisma_ui.show_window = true;
    if state.prisma_ui.schema_path.is_empty() {
        state.prisma_ui.schema_path = "./prisma/schema.prisma".to_string();
    }
    // Check Prisma installation
    let rt = tokio::runtime::Runtime::new().unwrap();
    state.prisma_ui.prisma_installed = rt.block_on(check_prisma_installed());
    state.prisma_ui.prisma_version = rt.block_on(get_prisma_version());
}
