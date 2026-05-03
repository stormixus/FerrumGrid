use eframe::egui::{
    self, Color32, ComboBox, CornerRadius, Margin, RichText, ScrollArea, Sense, Stroke, TextEdit,
};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::{AppState, ConnectionStatus, MainView};
use crate::types::{CellValue, ConnectionId, FunctionInfo, RoleInfo, TableInfo};
use crate::ui::{icons_svg, theme};

#[derive(Clone)]
struct TableRow {
    schema: String,
    name: String,
    table_type: String,
    column_count: Option<usize>,
    index_count: Option<usize>,
}

#[derive(Clone)]
enum ObjectAction {
    ViewData {
        conn_id: ConnectionId,
        schema: String,
        name: String,
    },
    DesignTable {
        schema: String,
        name: String,
    },
    CopySql(String),
    NewTable,
    OpenModel,
    AddAutomationQuery {
        title: String,
        sql: String,
    },
}

pub fn render_objects_view(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    render_tabs(ui, state);
    let action = render_sub_toolbar(ui, state, bridge);
    if let Some(action) = action {
        handle_action(ui, state, bridge, action);
    }
    render_objects_list(ui, state, bridge);
}

fn render_tabs(ui: &mut egui::Ui, state: &AppState) {
    let (title, subtitle, _color) = view_copy(state.active_main_view);
    let tab_frame = egui::Frame::new()
        .fill(theme::BG_SHELL)
        .inner_margin(Margin::symmetric(theme::SPACE_LG as i8, 0))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    tab_frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.set_min_height(38.0);
        ui.horizontal(|ui| {
            let (icon, icon_name) = view_icon(state.active_main_view);
            crate::ui::icon_img(ui, icon, icon_name, 16.0);
            ui.add_space(6.0);
            ui.label(
                RichText::new(title)
                    .color(theme::TEXT_PRIMARY)
                    .size(13.0)
                    .strong(),
            );
            ui.label(RichText::new(subtitle).color(theme::TEXT_MUTED).size(11.0));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(conn_id) = state.active_connection {
                    if let Some(conn) = state.connections.get(&conn_id) {
                        let status = match &conn.status {
                            ConnectionStatus::Connected { server_version } => {
                                format!("{}  PG {}", conn.config.display_name, server_version)
                            }
                            ConnectionStatus::Connecting => {
                                format!("{}  {}", conn.config.display_name, t("status_connecting"))
                            }
                            ConnectionStatus::Disconnected => {
                                format!(
                                    "{}  {}",
                                    conn.config.display_name,
                                    t("status_disconnected")
                                )
                            }
                        };
                        ui.label(RichText::new(status).color(theme::TEXT_MUTED).size(11.0));
                    }
                }
            });
        });
    });
}

fn render_sub_toolbar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let mut action = None;
    let frame = egui::Frame::new()
        .fill(theme::BG_DARK)
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE));

    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            if icon_button(ui, icons_svg::REFRESH, "objects_refresh", "Refresh").clicked() {
                refresh_current_view(state, bridge);
            }

            if matches!(state.active_main_view, MainView::Table)
                && icon_button(ui, icons_svg::PLUS, "objects_new", "New Table").clicked()
            {
                action = Some(ObjectAction::NewTable);
            }

            if matches!(state.active_main_view, MainView::Model)
                && icon_button(ui, icons_svg::MODEL, "objects_model", "Open ER Diagram").clicked()
            {
                action = Some(ObjectAction::OpenModel);
            }

            ui.add_space(theme::SPACE_MD);
            render_schema_filter(ui, state);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    TextEdit::singleline(&mut state.objects_search)
                        .desired_width(210.0)
                        .hint_text("Search")
                        .margin(Margin::symmetric(8, 2)),
                );
            });
        });
    });

    action
}

fn render_schema_filter(ui: &mut egui::Ui, state: &mut AppState) {
    let schemas = active_conn(state)
        .and_then(|conn_id| state.connections.get(&conn_id))
        .map(|conn| conn.schemas.clone())
        .unwrap_or_default();

    if matches!(
        state.active_main_view,
        MainView::User | MainView::Backup | MainView::BI
    ) {
        return;
    }

    ComboBox::from_id_salt("objects_schema_filter")
        .width(150.0)
        .selected_text(if state.objects_schema_filter.is_empty() {
            "All Schemas"
        } else {
            &state.objects_schema_filter
        })
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(state.objects_schema_filter.is_empty(), "All Schemas")
                .clicked()
            {
                state.objects_schema_filter.clear();
            }
            for schema in schemas {
                if ui
                    .selectable_label(state.objects_schema_filter == schema, &schema)
                    .clicked()
                {
                    state.objects_schema_filter = schema;
                }
            }
        });
}

fn render_objects_list(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG_DARK))
        .show_inside(ui, |ui| {
            let action = match state.active_main_view {
                MainView::Table | MainView::View | MainView::MaterializedView => {
                    render_table_like_objects(ui, state, bridge)
                }
                MainView::Function => render_functions(ui, state, bridge),
                MainView::User => render_roles(ui, state, bridge),
                MainView::Backup => render_backup_tools(ui, state),
                MainView::Automation => render_automation_tools(ui, state),
                MainView::Model => render_model_tools(ui, state, bridge),
                MainView::BI => render_bi_tools(ui, state),
                MainView::Connection | MainView::Query => None,
            };

            if let Some(action) = action {
                handle_action(ui, state, bridge, action);
            }
        });
}

fn render_table_like_objects(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_missing_tables(state, bridge, conn_id);
    let rows = collect_table_rows(state, conn_id);
    render_count_strip(ui, rows.len(), "objects");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_table_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &["Schema", "Name", "Type", "Columns", "Indexes", "Actions"],
            );
            for row in rows {
                let row_action = render_table_row(ui, conn_id, &row);
                if row_action.is_some() {
                    action = row_action;
                }
            }
        });

    action
}

fn render_functions(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_missing_functions(state, bridge, conn_id);
    let rows = collect_functions(state, conn_id);
    render_count_strip(ui, rows.len(), "functions");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_function_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &["Schema", "Name", "Signature", "Returns", "Lang", "Actions"],
            );
            for func in rows {
                let copied = render_function_row(ui, &func);
                if let Some(sql) = copied {
                    action = Some(ObjectAction::CopySql(sql));
                }
            }
        });
    action
}

fn render_roles(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let conn_id = match active_conn(state) {
        Some(id) => id,
        None => return render_no_connection(ui),
    };

    request_roles(state, bridge, conn_id);
    let rows = collect_roles(state, conn_id);
    render_count_strip(ui, rows.len(), "roles");

    let mut action = None;
    ScrollArea::vertical()
        .id_salt("objects_role_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &["Role", "Login", "Privileges", "Valid Until", "Actions"],
            );
            for role in rows {
                if let Some(sql) = render_role_row(ui, &role) {
                    action = Some(ObjectAction::CopySql(sql));
                }
            }
        });
    action
}

fn render_backup_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    let Some(conn_id) = active_conn(state) else {
        return render_no_connection(ui);
    };
    let conn = state.connections.get(&conn_id)?;
    let cfg = &conn.config;
    let custom = format!(
        "PGPASSWORD='{}' pg_dump --host {} --port {} --username {} --format custom --file {}.dump {}",
        shell_escape(&cfg.password),
        shell_escape(&cfg.host),
        cfg.port,
        shell_escape(&cfg.username),
        shell_escape(&cfg.database),
        shell_escape(&cfg.database)
    );
    let plain = format!(
        "PGPASSWORD='{}' pg_dump --host {} --port {} --username {} --format plain --file {}.sql {}",
        shell_escape(&cfg.password),
        shell_escape(&cfg.host),
        cfg.port,
        shell_escape(&cfg.username),
        shell_escape(&cfg.database),
        shell_escape(&cfg.database)
    );

    ui.add_space(theme::SPACE_XL);
    render_utility_card(
        ui,
        "Backup Commands",
        "Copy a ready pg_dump command for the active connection.",
        &[
            ("Custom archive", &custom),
            ("Plain SQL", &plain),
            (
                "Restore custom archive",
                &format!(
                    "PGPASSWORD='{}' pg_restore --host {} --port {} --username {} --dbname {} --clean --if-exists {}.dump",
                    shell_escape(&cfg.password),
                    shell_escape(&cfg.host),
                    cfg.port,
                    shell_escape(&cfg.username),
                    shell_escape(&cfg.database),
                    shell_escape(&cfg.database)
                ),
            ),
        ],
    )
}

fn render_automation_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    let schema = selected_schema_or_public(state);
    ui.add_space(theme::SPACE_XL);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Automation Presets")
                .color(theme::TEXT_PRIMARY)
                .size(14.0)
                .strong(),
        );
        ui.label(
            RichText::new("Create maintenance query tabs from the current schema.")
                .color(theme::TEXT_MUTED)
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_LG);

    let presets = [
        (
            "Vacuum Analyze",
            format!("VACUUM (VERBOSE, ANALYZE) {};", quote_ident(&schema)),
        ),
        (
            "Reindex Schema",
            format!("REINDEX SCHEMA {};", quote_ident(&schema)),
        ),
        (
            "Refresh Mat Views",
            format!(
                "DO $$\nDECLARE r record;\nBEGIN\n  FOR r IN SELECT schemaname, matviewname FROM pg_matviews WHERE schemaname = '{}'\n  LOOP\n    EXECUTE format('REFRESH MATERIALIZED VIEW %I.%I', r.schemaname, r.matviewname);\n  END LOOP;\nEND $$;",
                schema.replace('\'', "''")
            ),
        ),
    ];

    let mut action = None;
    for (title, sql) in presets {
        egui::Frame::new()
            .fill(theme::BG_MEDIUM)
            .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(theme::RADIUS_MD))
            .inner_margin(Margin::same(theme::SPACE_LG as i8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(title)
                            .color(theme::TEXT_PRIMARY)
                            .size(12.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(theme::secondary_button("Create Query")).clicked() {
                            action = Some(ObjectAction::AddAutomationQuery {
                                title: title.to_string(),
                                sql: sql.clone(),
                            });
                        }
                    });
                });
                ui.add_space(theme::SPACE_SM);
                code_line(ui, &sql);
            });
        ui.add_space(theme::SPACE_MD);
    }

    action
}

fn render_model_tools(
    ui: &mut egui::Ui,
    state: &AppState,
    _bridge: &DbBridge,
) -> Option<ObjectAction> {
    let _ = active_conn(state)?;
    ui.add_space(theme::SPACE_XL);
    let mut action = None;
    egui::Frame::new()
        .fill(theme::BG_MEDIUM)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::ui::icon_img(ui, icons_svg::MODEL, "objects_model_large", 24.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("ER Model")
                            .color(theme::TEXT_PRIMARY)
                            .size(15.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Open the relationship canvas for the selected schema.")
                            .color(theme::TEXT_MUTED)
                            .size(11.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(theme::primary_button("Open Diagram")).clicked() {
                        action = Some(ObjectAction::OpenModel);
                    }
                });
            });
        });
    action
}

fn render_bi_tools(ui: &mut egui::Ui, state: &AppState) -> Option<ObjectAction> {
    ui.add_space(theme::SPACE_LG);
    let Some(result) = state.current_result.as_ref() else {
        empty_state(
            ui,
            "No result set",
            "Run a query first, then BI will summarize rows and numeric columns here.",
        );
        return None;
    };

    render_count_strip(
        ui,
        result.rows.len(),
        &format!("rows, {} columns", result.columns.len()),
    );
    ui.add_space(theme::SPACE_MD);

    ScrollArea::vertical().id_salt("bi_summary").show(ui, |ui| {
        table_header(ui, &["Column", "Type", "Non-null", "Min", "Max", "Average"]);
        for (idx, column) in result.columns.iter().enumerate() {
            let mut count = 0usize;
            let mut values = Vec::new();
            for row in &result.rows {
                if let Some(cell) = row.get(idx) {
                    match cell {
                        CellValue::Int(v) => {
                            count += 1;
                            values.push(*v as f64);
                        }
                        CellValue::Float(v) => {
                            count += 1;
                            values.push(*v);
                        }
                        CellValue::Null => {}
                        _ => count += 1,
                    }
                }
            }

            let (min, max, avg) = if values.is_empty() {
                ("-".to_string(), "-".to_string(), "-".to_string())
            } else {
                let min = values.iter().copied().fold(f64::INFINITY, f64::min);
                let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                (format_number(min), format_number(max), format_number(avg))
            };

            data_row(ui, |ui| {
                ui.label(
                    RichText::new(&column.name)
                        .color(theme::TEXT_PRIMARY)
                        .size(12.0),
                );
                type_chip(ui, &column.type_name, theme::ACCENT_BLUE);
                ui.label(RichText::new(count.to_string()).color(theme::TEXT_SECONDARY));
                ui.label(RichText::new(min).color(theme::TEXT_MUTED));
                ui.label(RichText::new(max).color(theme::TEXT_MUTED));
                ui.label(RichText::new(avg).color(theme::TEXT_MUTED));
            });
        }
    });

    None
}

fn render_table_row(
    ui: &mut egui::Ui,
    conn_id: ConnectionId,
    row: &TableRow,
) -> Option<ObjectAction> {
    let mut action = None;
    data_row(ui, |ui| {
        ui.label(
            RichText::new(&row.schema)
                .color(theme::TEXT_MUTED)
                .size(12.0),
        );
        ui.label(
            RichText::new(&row.name)
                .color(theme::TEXT_PRIMARY)
                .size(12.0)
                .strong(),
        );
        type_chip(ui, &row.table_type, table_type_color(&row.table_type));
        ui.label(
            RichText::new(
                row.column_count
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            )
            .color(theme::TEXT_SECONDARY)
            .size(12.0),
        );
        ui.label(
            RichText::new(
                row.index_count
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            )
            .color(theme::TEXT_SECONDARY)
            .size(12.0),
        );
        ui.horizontal(|ui| {
            if ui.small_button("Data").clicked() {
                action = Some(ObjectAction::ViewData {
                    conn_id,
                    schema: row.schema.clone(),
                    name: row.name.clone(),
                });
            }
            if row.table_type != "VIEW" && ui.small_button("Design").clicked() {
                action = Some(ObjectAction::DesignTable {
                    schema: row.schema.clone(),
                    name: row.name.clone(),
                });
            }
            if ui.small_button("SQL").clicked() {
                action = Some(ObjectAction::CopySql(format!(
                    "SELECT * FROM {}.{};",
                    quote_ident(&row.schema),
                    quote_ident(&row.name)
                )));
            }
        });
    });
    action
}

fn render_function_row(ui: &mut egui::Ui, func: &FunctionInfo) -> Option<String> {
    let mut copied = None;
    data_row(ui, |ui| {
        ui.label(
            RichText::new(&func.schema)
                .color(theme::TEXT_MUTED)
                .size(12.0),
        );
        ui.label(
            RichText::new(&func.name)
                .color(theme::TEXT_PRIMARY)
                .size(12.0)
                .strong(),
        );
        ui.label(
            RichText::new(format!("({})", func.arguments))
                .color(theme::TEXT_SECONDARY)
                .size(11.0),
        );
        type_chip(ui, &func.return_type, theme::ACCENT_TEAL);
        ui.label(
            RichText::new(&func.language)
                .color(theme::TEXT_MUTED)
                .size(11.0),
        );
        ui.horizontal(|ui| {
            type_chip(ui, &func.kind, theme::ACCENT_COPPER);
            if ui.small_button("SQL").clicked() {
                copied = Some(format!(
                    "SELECT * FROM {}.{}({});",
                    quote_ident(&func.schema),
                    quote_ident(&func.name),
                    argument_placeholders(&func.arguments)
                ));
            }
        });
    });
    copied
}

fn render_role_row(ui: &mut egui::Ui, role: &RoleInfo) -> Option<String> {
    let mut copied = None;
    data_row(ui, |ui| {
        ui.label(
            RichText::new(&role.name)
                .color(theme::TEXT_PRIMARY)
                .size(12.0)
                .strong(),
        );
        type_chip(
            ui,
            if role.can_login { "LOGIN" } else { "NOLOGIN" },
            if role.can_login {
                theme::ACCENT_GREEN
            } else {
                theme::TEXT_MUTED
            },
        );
        let mut flags = Vec::new();
        if role.is_superuser {
            flags.push("SUPERUSER");
        }
        if role.can_create_db {
            flags.push("CREATEDB");
        }
        if role.can_create_role {
            flags.push("CREATEROLE");
        }
        if role.can_replicate {
            flags.push("REPLICATION");
        }
        ui.label(
            RichText::new(if flags.is_empty() {
                "-".to_string()
            } else {
                flags.join(", ")
            })
            .color(theme::TEXT_SECONDARY)
            .size(11.0),
        );
        ui.label(
            RichText::new(role.valid_until.as_deref().unwrap_or("-"))
                .color(theme::TEXT_MUTED)
                .size(11.0),
        );
        if ui.small_button("SQL").clicked() {
            copied = Some(format!("ALTER ROLE {};", quote_ident(&role.name)));
        }
    });
    copied
}

fn collect_table_rows(state: &AppState, conn_id: ConnectionId) -> Vec<TableRow> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let schemas = selected_schemas(state);
    let search = state.objects_search.to_lowercase();
    let mut rows = Vec::new();

    for schema in schemas {
        if let Some(tables) = conn.tables.get(&schema) {
            for table in tables {
                if !matches_table_kind(state.active_main_view, table) {
                    continue;
                }
                if !search.is_empty()
                    && !table.name.to_lowercase().contains(&search)
                    && !schema.to_lowercase().contains(&search)
                    && !table.table_type.to_lowercase().contains(&search)
                {
                    continue;
                }

                let key = (schema.clone(), table.name.clone());
                rows.push(TableRow {
                    schema: schema.clone(),
                    name: table.name.clone(),
                    table_type: table.table_type.clone(),
                    column_count: conn.columns.get(&key).map(Vec::len),
                    index_count: conn.indexes.get(&key).map(Vec::len),
                });
            }
        }
    }

    rows.sort_by(|a, b| (&a.schema, &a.name).cmp(&(&b.schema, &b.name)));
    rows
}

fn collect_functions(state: &AppState, conn_id: ConnectionId) -> Vec<FunctionInfo> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let schemas = selected_schemas(state);
    let search = state.objects_search.to_lowercase();
    let mut rows = Vec::new();

    for schema in schemas {
        if let Some(functions) = conn.functions.get(&schema) {
            for func in functions {
                if !search.is_empty()
                    && !func.name.to_lowercase().contains(&search)
                    && !func.arguments.to_lowercase().contains(&search)
                    && !func.return_type.to_lowercase().contains(&search)
                {
                    continue;
                }
                rows.push(func.clone());
            }
        }
    }

    rows.sort_by(|a, b| {
        (&a.schema, &a.name, &a.arguments).cmp(&(&b.schema, &b.name, &b.arguments))
    });
    rows
}

fn collect_roles(state: &AppState, conn_id: ConnectionId) -> Vec<RoleInfo> {
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };
    let search = state.objects_search.to_lowercase();
    conn.roles
        .iter()
        .filter(|role| search.is_empty() || role.name.to_lowercase().contains(&search))
        .cloned()
        .collect()
}

fn request_missing_tables(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schemas = selected_schemas(state);
    let mut to_load = Vec::new();
    if let Some(conn) = state.connections.get(&conn_id) {
        for schema in &schemas {
            if !conn.tables.contains_key(schema) && !conn.loading_tables.contains(schema) {
                to_load.push(schema.clone());
            }
        }
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for schema in &to_load {
            conn.loading_tables.insert(schema.clone());
        }
    }

    for schema in to_load {
        bridge.send(DbCommand::ListTables { conn_id, schema });
    }
}

fn request_missing_functions(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schemas = selected_schemas(state);
    let mut to_load = Vec::new();
    if let Some(conn) = state.connections.get(&conn_id) {
        for schema in &schemas {
            if !conn.functions.contains_key(schema) && !conn.loading_functions.contains(schema) {
                to_load.push(schema.clone());
            }
        }
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        for schema in &to_load {
            conn.loading_functions.insert(schema.clone());
        }
    }

    for schema in to_load {
        bridge.send(DbCommand::ListFunctions { conn_id, schema });
    }
}

fn request_roles(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let should_load = state
        .connections
        .get(&conn_id)
        .is_some_and(|conn| conn.roles.is_empty() && !conn.loading_roles);
    if should_load {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_roles = true;
        }
        bridge.send(DbCommand::ListRoles { conn_id });
    }
}

fn refresh_current_view(state: &mut AppState, bridge: &DbBridge) {
    let Some(conn_id) = active_conn(state) else {
        return;
    };
    let schemas = selected_schemas(state);

    match state.active_main_view {
        MainView::Table
        | MainView::View
        | MainView::MaterializedView
        | MainView::Model
        | MainView::BI => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                for schema in &schemas {
                    conn.tables.remove(schema);
                    conn.loading_tables.insert(schema.clone());
                }
            }
            for schema in schemas {
                bridge.send(DbCommand::ListTables { conn_id, schema });
            }
        }
        MainView::Function => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                for schema in &schemas {
                    conn.functions.remove(schema);
                    conn.loading_functions.insert(schema.clone());
                }
            }
            for schema in schemas {
                bridge.send(DbCommand::ListFunctions { conn_id, schema });
            }
        }
        MainView::User => {
            if let Some(conn) = state.connections.get_mut(&conn_id) {
                conn.roles.clear();
                conn.loading_roles = true;
            }
            bridge.send(DbCommand::ListRoles { conn_id });
        }
        _ => {}
    }
}

fn handle_action(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge, action: ObjectAction) {
    match action {
        ObjectAction::ViewData {
            conn_id,
            schema,
            name,
        } => {
            bridge.send(DbCommand::ExecuteQuery {
                conn_id,
                sql: format!(
                    "SELECT * FROM {}.{} LIMIT 100",
                    quote_ident(&schema),
                    quote_ident(&name)
                ),
                row_limit: Some(100),
            });
            state.query_running = true;
            state.active_main_view = MainView::Query;
        }
        ObjectAction::DesignTable { schema, name } => {
            crate::ui::table_designer::open_for_existing_table(state, &schema, &name, bridge);
        }
        ObjectAction::CopySql(sql) => {
            ui.ctx().copy_text(sql);
        }
        ObjectAction::NewTable => {
            if state.objects_schema_filter.is_empty() {
                crate::ui::table_designer::open_for_new_table(state);
            } else {
                let schema = state.objects_schema_filter.clone();
                crate::ui::table_designer::open_for_new_table_with_schema(state, &schema);
            }
        }
        ObjectAction::OpenModel => {
            if state.er_diagram.selected_schema.is_empty() {
                state.er_diagram.selected_schema = selected_schema_or_public(state);
            }
            state.er_diagram.show_diagram = true;
        }
        ObjectAction::AddAutomationQuery { title, sql } => {
            state.editor_tabs.push(crate::types::EditorTab::new(title));
            state.active_tab = state.editor_tabs.len() - 1;
            if let Some(tab) = state.editor_tabs.get_mut(state.active_tab) {
                tab.content = sql;
                tab.connection_id = state.active_connection;
            }
            state.active_main_view = MainView::Query;
        }
    }
}

fn render_utility_card(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    commands: &[(&str, &str)],
) -> Option<ObjectAction> {
    let mut action = None;
    egui::Frame::new()
        .fill(theme::BG_MEDIUM)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(theme::RADIUS_MD))
        .inner_margin(Margin::same(theme::SPACE_XL as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .color(theme::TEXT_PRIMARY)
                    .size(15.0)
                    .strong(),
            );
            ui.label(RichText::new(subtitle).color(theme::TEXT_MUTED).size(11.0));
            ui.add_space(theme::SPACE_LG);
            for (label, command) in commands {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(*label)
                            .color(theme::TEXT_SECONDARY)
                            .size(12.0)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Copy").clicked() {
                            action = Some(ObjectAction::CopySql((*command).to_string()));
                        }
                    });
                });
                code_line(ui, command);
                ui.add_space(theme::SPACE_MD);
            }
        });
    action
}

fn render_no_connection(ui: &mut egui::Ui) -> Option<ObjectAction> {
    empty_state(
        ui,
        "No active connection",
        "Connect to PostgreSQL to browse and operate on database objects.",
    );
    None
}

fn empty_state(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);
        crate::ui::icon_img(ui, icons_svg::DATABASE, "objects_empty", 32.0);
        ui.add_space(theme::SPACE_MD);
        ui.label(RichText::new(title).color(theme::TEXT_MUTED).size(16.0));
        ui.label(
            RichText::new(subtitle)
                .color(theme::TEXT_DISABLED)
                .size(11.0),
        );
    });
}

fn table_header(ui: &mut egui::Ui, headers: &[&str]) {
    egui::Frame::new()
        .fill(theme::BG_SHELL)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ))
        .show(ui, |ui| {
            ui.columns(headers.len(), |cols| {
                for (col, header) in cols.iter_mut().zip(headers) {
                    col.label(
                        RichText::new(*header)
                            .color(theme::TEXT_MUTED)
                            .size(10.5)
                            .strong(),
                    );
                }
            });
        });
}

fn data_row(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    let frame = egui::Frame::new()
        .fill(theme::BG_DARK)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
        .inner_margin(Margin::symmetric(
            theme::SPACE_LG as i8,
            theme::SPACE_SM as i8,
        ));
    let response = frame
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_LG;
                content(ui);
            });
        })
        .response;

    if response.hovered() {
        ui.painter().rect_filled(
            response.rect,
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::ACCENT_TEAL, 10),
        );
    }
}

fn render_count_strip(ui: &mut egui::Ui, count: usize, label: &str) {
    ui.horizontal(|ui| {
        ui.add_space(theme::SPACE_LG);
        ui.label(
            RichText::new(format!("{count} {label}"))
                .color(theme::TEXT_MUTED)
                .size(11.0),
        );
    });
    ui.add_space(theme::SPACE_SM);
}

fn code_line(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(theme::BG_DARKEST)
        .stroke(Stroke::new(1.0, theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .inner_margin(Margin::same(theme::SPACE_MD as i8))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .font(egui::FontId::monospace(11.0))
                    .color(theme::TEXT_SECONDARY),
            );
        });
}

fn type_chip(ui: &mut egui::Ui, label: &str, color: Color32) {
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), egui::FontId::monospace(10.5), color);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(galley.rect.width() + 12.0, 19.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        theme::with_alpha(color, 26),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(10.5),
        color,
    );
}

fn icon_button(ui: &mut egui::Ui, svg: &str, name: &str, tooltip: &'static str) -> egui::Response {
    let response = ui
        .add_sized(
            egui::vec2(28.0, 26.0),
            egui::Button::new("")
                .fill(theme::BG_LIGHT)
                .stroke(Stroke::new(1.0, theme::BORDER_DEFAULT))
                .corner_radius(CornerRadius::same(theme::RADIUS_MD)),
        )
        .on_hover_text(tooltip);

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect), |ui| {
        crate::ui::icon_img(ui, svg, name, 13.0);
    });

    response
}

fn active_conn(state: &AppState) -> Option<ConnectionId> {
    let conn_id = state.active_connection?;
    let conn = state.connections.get(&conn_id)?;
    matches!(conn.status, ConnectionStatus::Connected { .. }).then_some(conn_id)
}

fn selected_schemas(state: &AppState) -> Vec<String> {
    let Some(conn_id) = state.active_connection else {
        return Vec::new();
    };
    let Some(conn) = state.connections.get(&conn_id) else {
        return Vec::new();
    };

    if state.objects_schema_filter.is_empty() {
        conn.schemas.clone()
    } else {
        vec![state.objects_schema_filter.clone()]
    }
}

fn selected_schema_or_public(state: &AppState) -> String {
    if !state.objects_schema_filter.is_empty() {
        return state.objects_schema_filter.clone();
    }
    state
        .active_connection
        .and_then(|id| state.connections.get(&id))
        .and_then(|conn| {
            conn.schemas
                .iter()
                .find(|schema| *schema == "public")
                .cloned()
        })
        .or_else(|| {
            state
                .active_connection
                .and_then(|id| state.connections.get(&id))
                .and_then(|conn| conn.schemas.first().cloned())
        })
        .unwrap_or_else(|| "public".to_string())
}

fn matches_table_kind(view: MainView, table: &TableInfo) -> bool {
    match view {
        MainView::View => table.table_type == "VIEW",
        MainView::MaterializedView => table.table_type == "MATERIALIZED VIEW",
        MainView::Table => table.table_type != "VIEW" && table.table_type != "MATERIALIZED VIEW",
        _ => true,
    }
}

fn view_copy(view: MainView) -> (&'static str, &'static str, Color32) {
    match view {
        MainView::Table => (
            "Tables",
            "Base tables and editable relations",
            theme::ACCENT_COPPER,
        ),
        MainView::View => ("Views", "Virtual query-backed objects", theme::ACCENT_BLUE),
        MainView::MaterializedView => (
            "Materialized Views",
            "Stored query snapshots",
            theme::ACCENT_TEAL,
        ),
        MainView::Function => (
            "Functions",
            "PostgreSQL routines by schema",
            theme::ACCENT_YELLOW,
        ),
        MainView::User => (
            "Users",
            "Roles and login permissions",
            theme::ACCENT_COPPER_LIGHT,
        ),
        MainView::Backup => (
            "Backup",
            "pg_dump and restore command builder",
            theme::TEXT_MUTED,
        ),
        MainView::Automation => (
            "Automation",
            "Maintenance query presets",
            theme::ACCENT_TEAL,
        ),
        MainView::Model => (
            "Model",
            "ER diagram and schema modeling",
            theme::ACCENT_GREEN,
        ),
        MainView::BI => ("BI", "Quick result-set profiling", theme::ACCENT_RED),
        MainView::Connection => (
            "Connections",
            "Database connection setup",
            theme::ACCENT_GREEN,
        ),
        MainView::Query => ("Query", "SQL editor", theme::ACCENT_BLUE),
    }
}

fn view_icon(view: MainView) -> (&'static str, &'static str) {
    match view {
        MainView::Table => (icons_svg::TABLE, "objects_title_table"),
        MainView::View => (icons_svg::VIEW, "objects_title_view"),
        MainView::MaterializedView => (icons_svg::MATERIALIZED_VIEW, "objects_title_materialized"),
        MainView::Function => (icons_svg::FUNCTION, "objects_title_function"),
        MainView::User => (icons_svg::USER, "objects_title_user"),
        MainView::Backup => (icons_svg::BACKUP, "objects_title_backup"),
        MainView::Automation => (icons_svg::AUTOMATION, "objects_title_automation"),
        MainView::Model => (icons_svg::MODEL, "objects_title_model"),
        MainView::BI => (icons_svg::BI, "objects_title_bi"),
        MainView::Connection => (icons_svg::CONNECTION, "objects_title_connection"),
        MainView::Query => (icons_svg::QUERY, "objects_title_query"),
    }
}

fn table_type_color(table_type: &str) -> Color32 {
    match table_type {
        "VIEW" => theme::ACCENT_BLUE,
        "MATERIALIZED VIEW" => theme::ACCENT_TEAL,
        _ => theme::ACCENT_COPPER,
    }
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

fn argument_placeholders(args: &str) -> String {
    if args.trim().is_empty() {
        return String::new();
    }
    args.split(',')
        .enumerate()
        .map(|(idx, arg)| {
            let name = arg.split_whitespace().next().unwrap_or("arg");
            format!("/* {}: ${} */", name, idx + 1)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_number(value: f64) -> String {
    if value.abs() >= 1000.0 || value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.3}")
    }
}
