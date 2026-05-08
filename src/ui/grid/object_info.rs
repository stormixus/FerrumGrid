//! Object-info renderers — per-MainView info panel content (connection,
//! table/view/matview, function, role, query, backup, automation, model, BI).
//!
//! Plan v7 — extracted from `info_panel.rs` to keep that file under 800 lines.
//! Pure rendering helpers; no behavior change.

use eframe::egui::{self, RichText};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{AppState, ConnectionStatus, DataSource};
use crate::ui::theme;

use super::info_panel::{info_section_label, value_box};
use super::info_row::render_info_empty;
use super::metric_chip;
use super::table_info::{ensure_table_info_metadata, render_info_table_overview};

pub(super) fn render_connection_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_connection_title"));
    ui.add_space(theme::SPACE_XS);

    let Some(active_id) = state.active_connection else {
        render_info_empty(ui, &t("info_view_connection_select"));
        return;
    };
    let Some(conn) = state.connections.get(&active_id) else {
        render_info_empty(ui, &t("info_view_connection_none"));
        return;
    };

    ui.label(
        RichText::new(&conn.config.display_name)
            .color(theme::text_primary())
            .strong()
            .size(16.0),
    );
    ui.label(
        RichText::new(format!(
            "{}@{}:{}/{}",
            conn.config.username, conn.config.host, conn.config.port, conn.config.database
        ))
        .color(theme::text_muted())
        .monospace()
        .size(11.0),
    );
    ui.add_space(theme::SPACE_SM);

    ui.horizontal_wrapped(|ui| {
        let (label, color) = match &conn.status {
            ConnectionStatus::Connected { .. } => {
                (t("info_view_status_connected"), theme::ACCENT_GREEN)
            }
            ConnectionStatus::Connecting => {
                (t("info_view_loading"), theme::ACCENT_YELLOW)
            }
            ConnectionStatus::Disconnected => {
                (t("info_view_status_disconnected"), theme::text_muted())
            }
        };
        metric_chip(ui, &label, color);
        if conn.config.use_tls {
            metric_chip(ui, &t("info_view_ssl"), theme::ACCENT_EMERALD);
        }
    });

    if let ConnectionStatus::Connected { server_version } = &conn.status {
        if !server_version.is_empty() {
            ui.add_space(theme::SPACE_SM);
            ui.label(
                RichText::new(server_version)
                    .color(theme::text_muted())
                    .monospace()
                    .size(10.5),
            );
        }
    }

    ui.add_space(theme::SPACE_LG);
    info_section_label(ui, &t("info_view_objects_title"));
    ui.add_space(theme::SPACE_XS);
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("info_view_schemas_n", &[&conn.schemas.len().to_string()]),
            theme::ACCENT_BLUE,
        );
        let total_tables: usize = conn.tables.values().map(|tables| tables.len()).sum();
        if total_tables > 0 {
            metric_chip(
                ui,
                &tf("info_view_tables_n", &[&total_tables.to_string()]),
                theme::ACCENT_COPPER,
            );
        }
        let total_functions: usize = conn.functions.values().map(|f| f.len()).sum();
        if total_functions > 0 {
            metric_chip(
                ui,
                &tf("info_view_functions_n", &[&total_functions.to_string()]),
                theme::ACCENT_YELLOW,
            );
        }
        if !conn.roles.is_empty() {
            metric_chip(
                ui,
                &tf("info_view_roles_n", &[&conn.roles.len().to_string()]),
                theme::ACCENT_COPPER_LIGHT,
            );
        }
    });

    if let Some(err) = &conn.connection_error {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_status_error"));
        ui.add_space(theme::SPACE_XS);
        value_box(ui, err, theme::ACCENT_RED);
    }
}

pub(super) fn render_table_kind_info(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    table_type: &str,
    title: &str,
) {
    let Some(active_id) = state.active_connection else {
        info_section_label(ui, title);
        ui.add_space(theme::SPACE_XS);
        render_info_empty(ui, &t("info_view_connection_select"));
        return;
    };

    if let Some((schema, name)) = state.objects_selected_table.clone() {
        let exists = state
            .connections
            .get(&active_id)
            .and_then(|conn| conn.tables.get(&schema))
            .is_some_and(|tables| {
                tables
                    .iter()
                    .any(|t| t.name == name && t.table_type == table_type)
            });
        if exists {
            let source = DataSource {
                conn_id: active_id,
                schema,
                table: name,
                filter: None,
            };
            ensure_table_info_metadata(state, bridge, &source);
            render_info_table_overview(ui, state, &source);
            return;
        }
    }

    info_section_label(ui, title);
    ui.add_space(theme::SPACE_XS);

    let Some(conn) = state.connections.get(&active_id) else {
        render_info_empty(ui, &t("info_view_connection_none"));
        return;
    };

    let schema_filter = state.objects_schema_filter.trim();
    let schemas: Vec<&String> = if schema_filter.is_empty() {
        conn.schemas.iter().collect()
    } else {
        conn.schemas
            .iter()
            .filter(|s| s.as_str() == schema_filter)
            .collect()
    };

    let scope_label = if schema_filter.is_empty() {
        t("info_view_no_schema_filter")
    } else {
        schema_filter.to_string()
    };
    ui.label(
        RichText::new(&scope_label)
            .color(theme::text_primary())
            .strong()
            .size(14.0),
    );
    ui.label(
        RichText::new(&conn.config.display_name)
            .color(theme::text_muted())
            .monospace()
            .size(11.0),
    );
    ui.add_space(theme::SPACE_SM);

    let mut total = 0usize;
    let mut per_schema: Vec<(String, usize)> = Vec::new();
    for schema in &schemas {
        let count = conn
            .tables
            .get(*schema)
            .map(|tables| {
                tables
                    .iter()
                    .filter(|t| t.table_type == table_type)
                    .count()
            })
            .unwrap_or(0);
        total += count;
        if count > 0 {
            per_schema.push(((*schema).clone(), count));
        }
    }

    ui.horizontal_wrapped(|ui| {
        let count_label = match table_type {
            "VIEW" => "info_view_views_n",
            "MATERIALIZED VIEW" => "info_view_matviews_n",
            _ => "info_view_tables_n",
        };
        metric_chip(
            ui,
            &tf(count_label, &[&total.to_string()]),
            theme::ACCENT_COPPER,
        );
        if !state.objects_search.is_empty() {
            metric_chip(
                ui,
                &format!("filter: {}", state.objects_search),
                theme::ACCENT_BLUE,
            );
        }
    });

    if !per_schema.is_empty() && schemas.len() > 1 {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_schema"));
        ui.add_space(theme::SPACE_XS);
        for (schema, count) in per_schema.iter().take(10) {
            ui.label(
                RichText::new(tf("info_view_count_in_schema", &[&count.to_string(), schema]))
                    .color(theme::text_secondary())
                    .size(11.5),
            );
        }
    }

    ui.add_space(theme::SPACE_LG);
    ui.label(
        RichText::new(t("info_view_open_data_hint"))
            .color(theme::text_muted())
            .italics()
            .size(11.0),
    );
}

pub(super) fn render_function_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_function_title"));
    ui.add_space(theme::SPACE_XS);

    let Some(active_id) = state.active_connection else {
        render_info_empty(ui, &t("info_view_connection_select"));
        return;
    };
    let Some(conn) = state.connections.get(&active_id) else {
        render_info_empty(ui, &t("info_view_connection_none"));
        return;
    };

    if let Some((schema, name)) = state.objects_selected_function.as_ref() {
        if let Some(func) = conn
            .functions
            .get(schema)
            .and_then(|funcs| funcs.iter().find(|f| f.name == *name))
        {
            ui.label(
                RichText::new(&func.name)
                    .color(theme::text_primary())
                    .strong()
                    .size(16.0),
            );
            ui.label(
                RichText::new(format!("{}.{}", schema, name))
                    .color(theme::text_muted())
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(theme::SPACE_SM);
            ui.horizontal_wrapped(|ui| {
                metric_chip(ui, &func.kind, theme::ACCENT_COPPER);
                metric_chip(ui, &func.language, theme::ACCENT_BLUE);
                metric_chip(ui, &func.return_type, theme::ACCENT_EMERALD);
            });
            ui.add_space(theme::SPACE_LG);
            info_section_label(ui, &t("objects_signature"));
            ui.add_space(theme::SPACE_XS);
            value_box(
                ui,
                &format!("({})", func.arguments),
                theme::text_secondary(),
            );
            return;
        }
    }

    let total: usize = conn.functions.values().map(|f| f.len()).sum();
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("info_view_functions_n", &[&total.to_string()]),
            theme::ACCENT_YELLOW,
        );
    });

    if conn.functions.is_empty() && !conn.loading_functions.is_empty() {
        ui.add_space(theme::SPACE_SM);
        ui.label(
            RichText::new(t("info_view_loading"))
                .color(theme::text_muted())
                .italics()
                .size(11.0),
        );
        return;
    }

    let mut entries: Vec<(String, usize)> = conn
        .functions
        .iter()
        .map(|(schema, funcs)| (schema.clone(), funcs.len()))
        .filter(|(_, n)| *n > 0)
        .collect();
    entries.sort_by_key(|b| std::cmp::Reverse(b.1));

    if !entries.is_empty() {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_schema"));
        ui.add_space(theme::SPACE_XS);
        for (schema, count) in entries.iter().take(10) {
            ui.label(
                RichText::new(tf("info_view_count_in_schema", &[&count.to_string(), schema]))
                    .color(theme::text_secondary())
                    .size(11.5),
            );
        }
    }
}

pub(super) fn render_role_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_role_title"));
    ui.add_space(theme::SPACE_XS);

    let Some(active_id) = state.active_connection else {
        render_info_empty(ui, &t("info_view_connection_select"));
        return;
    };
    let Some(conn) = state.connections.get(&active_id) else {
        render_info_empty(ui, &t("info_view_connection_none"));
        return;
    };

    if conn.loading_roles && conn.roles.is_empty() {
        ui.label(
            RichText::new(t("info_view_loading"))
                .color(theme::text_muted())
                .italics()
                .size(11.0),
        );
        return;
    }

    if let Some(name) = state.objects_selected_role.as_ref() {
        if let Some(role) = conn.roles.iter().find(|r| r.name == *name) {
            ui.label(
                RichText::new(&role.name)
                    .color(theme::text_primary())
                    .strong()
                    .size(16.0),
            );
            ui.add_space(theme::SPACE_SM);
            ui.horizontal_wrapped(|ui| {
                metric_chip(
                    ui,
                    if role.can_login { "LOGIN" } else { "NOLOGIN" },
                    if role.can_login {
                        theme::ACCENT_GREEN
                    } else {
                        theme::text_muted()
                    },
                );
                if role.is_superuser {
                    metric_chip(ui, "SUPERUSER", theme::ACCENT_RED);
                }
                if role.can_create_db {
                    metric_chip(ui, "CREATEDB", theme::ACCENT_BLUE);
                }
                if role.can_create_role {
                    metric_chip(ui, "CREATEROLE", theme::ACCENT_COPPER);
                }
                if role.can_replicate {
                    metric_chip(ui, "REPLICATION", theme::ACCENT_EMERALD);
                }
            });
            if let Some(valid_until) = &role.valid_until {
                ui.add_space(theme::SPACE_SM);
                ui.label(
                    RichText::new(format!("valid until: {}", valid_until))
                        .color(theme::text_muted())
                        .monospace()
                        .size(11.0),
                );
            }
            return;
        }
    }

    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("info_view_roles_n", &[&conn.roles.len().to_string()]),
            theme::ACCENT_COPPER_LIGHT,
        );
    });

    if !conn.roles.is_empty() {
        ui.add_space(theme::SPACE_LG);
        for role in conn.roles.iter().take(12) {
            ui.label(
                RichText::new(&role.name)
                    .color(theme::text_secondary())
                    .monospace()
                    .size(11.5),
            );
        }
        if conn.roles.len() > 12 {
            ui.add_space(theme::SPACE_XS);
            ui.label(
                RichText::new(format!("+ {}", conn.roles.len() - 12))
                    .color(theme::text_muted())
                    .size(11.0),
            );
        }
    }
}

pub(super) fn render_query_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_query_title"));
    ui.add_space(theme::SPACE_XS);

    let active_tab = state.editor_tabs.get(state.active_tab);
    let title = active_tab
        .map(|tab| tab.title.clone())
        .unwrap_or_else(|| t("info_view_query_idle"));
    ui.label(
        RichText::new(&title)
            .color(theme::text_primary())
            .strong()
            .size(14.0),
    );

    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        if state.query_running {
            metric_chip(ui, &t("info_view_query_running"), theme::ACCENT_YELLOW);
        } else {
            metric_chip(ui, &t("info_view_query_idle"), theme::text_muted());
        }
        if state.explicit_tx_active {
            metric_chip(ui, &t("info_view_query_explicit_tx"), theme::ACCENT_RED);
        }
        if let Some(tab) = active_tab {
            metric_chip(
                ui,
                &tf("info_view_query_chars", &[&tab.content.len().to_string()]),
                theme::ACCENT_BLUE,
            );
        }
    });

    if let Some(result) = &state.current_result {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("data_info_table"));
        ui.add_space(theme::SPACE_XS);
        ui.horizontal_wrapped(|ui| {
            metric_chip(
                ui,
                &tf("info_view_query_last_rows", &[&result.rows.len().to_string()]),
                theme::ACCENT_EMERALD,
            );
            metric_chip(
                ui,
                &tf(
                    "info_view_query_last_cols",
                    &[&result.columns.len().to_string()],
                ),
                theme::ACCENT_BLUE,
            );
            metric_chip(
                ui,
                &format!("{} ms", result.execution_time_ms),
                theme::ACCENT_GREEN,
            );
            if state.current_result_truncated {
                metric_chip(ui, &t("info_view_query_truncated"), theme::ACCENT_YELLOW);
            }
        });
    }

    if let Some(err) = &state.last_error {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_query_error"));
        ui.add_space(theme::SPACE_XS);
        value_box(ui, err, theme::ACCENT_RED);
    }
}

pub(super) fn render_backup_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_backup_title"));
    ui.add_space(theme::SPACE_XS);

    ui.label(
        RichText::new(state.backup_format.label())
            .color(theme::text_primary())
            .strong()
            .size(14.0),
    );
    ui.add_space(theme::SPACE_SM);
    ui.horizontal_wrapped(|ui| {
        if state.backup_running {
            metric_chip(ui, &t("info_view_backup_running"), theme::ACCENT_YELLOW);
        } else {
            metric_chip(ui, &t("info_view_backup_idle"), theme::text_muted());
        }
        metric_chip(
            ui,
            &tf(
                "info_view_backup_history_n",
                &[&state.backup_history.len().to_string()],
            ),
            theme::ACCENT_BLUE,
        );
    });

    if let Some(err) = &state.backup_last_error {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_backup_last_error"));
        ui.add_space(theme::SPACE_XS);
        value_box(ui, err, theme::ACCENT_RED);
    }

    ui.add_space(theme::SPACE_LG);
    info_section_label(ui, &t("info_view_backup_last"));
    ui.add_space(theme::SPACE_XS);
    if let Some(record) = state.backup_history.last() {
        ui.label(
            RichText::new(&record.connection_name)
                .color(theme::text_primary())
                .strong()
                .size(12.0),
        );
        ui.label(
            RichText::new(&record.database)
                .color(theme::text_muted())
                .monospace()
                .size(11.0),
        );
        ui.label(
            RichText::new(&record.completed_at)
                .color(theme::text_muted())
                .size(11.0),
        );
        ui.label(
            RichText::new(format!(
                "{} · {} ms",
                format_size(record.size_bytes),
                record.duration_ms
            ))
            .color(theme::text_secondary())
            .monospace()
            .size(11.0),
        );
    } else {
        ui.label(
            RichText::new(t("info_view_backup_no_history"))
                .color(theme::text_muted())
                .italics()
                .size(11.0),
        );
    }
}

pub(super) fn render_automation_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_automation_title"));
    ui.add_space(theme::SPACE_XS);

    let total = state
        .automation
        .read()
        .map(|store| store.len())
        .unwrap_or(0);
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &format!("{}: {}", t("info_view_automation_total"), total),
            theme::ACCENT_EMERALD,
        );
    });

    ui.add_space(theme::SPACE_LG);
    if state.automation_draft.title.trim().is_empty()
        && state.automation_draft.sql.trim().is_empty()
    {
        ui.label(
            RichText::new(t("info_view_automation_draft_empty"))
                .color(theme::text_muted())
                .italics()
                .size(11.0),
        );
    } else {
        let label = if state.automation_draft.title.is_empty() {
            t("info_view_automation_draft_untitled")
        } else {
            state.automation_draft.title.clone()
        };
        ui.label(
            RichText::new(tf("info_view_automation_draft_ready", &[&label]))
                .color(theme::ACCENT_EMERALD)
                .strong()
                .size(12.0),
        );
        if !state.automation_draft.sql.is_empty() {
            let preview: String = state
                .automation_draft
                .sql
                .chars()
                .take(120)
                .collect::<String>();
            ui.add_space(theme::SPACE_XS);
            value_box(ui, &preview, theme::text_secondary());
        }
    }
}

pub(super) fn render_model_info(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    let Some(active_id) = state.active_connection else {
        info_section_label(ui, &t("info_view_model_title"));
        ui.add_space(theme::SPACE_XS);
        render_info_empty(ui, &t("info_view_connection_select"));
        return;
    };

    if let Some(card_id) = state.er_diagram.selected_table.clone() {
        if let Some((schema, table)) = card_id.split_once('.') {
            let source = DataSource {
                conn_id: active_id,
                schema: schema.to_string(),
                table: table.to_string(),
                filter: None,
            };
            ensure_table_info_metadata(state, bridge, &source);
            render_info_table_overview(ui, state, &source);
            return;
        }
    }

    info_section_label(ui, &t("info_view_model_title"));
    ui.add_space(theme::SPACE_XS);
    let schema_label = if state.er_diagram.selected_schema.is_empty() {
        t("info_view_no_schema_filter")
    } else {
        state.er_diagram.selected_schema.clone()
    };
    ui.label(
        RichText::new(&schema_label)
            .color(theme::text_primary())
            .strong()
            .size(14.0),
    );
    ui.add_space(theme::SPACE_SM);
    let card_count = state.er_diagram.cards.len();
    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("info_view_model_cards_n", &[&card_count.to_string()]),
            theme::ACCENT_GREEN,
        );
        let fk_count = state.er_diagram.foreign_keys.len();
        if fk_count > 0 {
            metric_chip(
                ui,
                &tf("data_info_relations_n", &[&fk_count.to_string()]),
                theme::ACCENT_BLUE,
            );
        }
    });

    ui.add_space(theme::SPACE_LG);
    ui.label(
        RichText::new(t("info_view_model_no_card"))
            .color(theme::text_muted())
            .italics()
            .size(11.0),
    );
}

pub(super) fn render_bi_info(ui: &mut egui::Ui, state: &AppState) {
    info_section_label(ui, &t("info_view_bi_title"));
    ui.add_space(theme::SPACE_XS);

    let Some(result) = &state.current_result else {
        render_info_empty(ui, &t("info_view_bi_no_result"));
        return;
    };

    let mut numeric_cols: Vec<&str> = Vec::new();
    let mut text_cols: Vec<&str> = Vec::new();
    for col in &result.columns {
        let lower = col.type_name.to_lowercase();
        if matches!(
            lower.as_str(),
            "integer"
                | "bigint"
                | "smallint"
                | "numeric"
                | "real"
                | "double precision"
                | "double"
                | "float"
                | "money"
        ) || lower.starts_with("numeric")
            || lower.starts_with("int")
            || lower.starts_with("float")
        {
            numeric_cols.push(col.name.as_str());
        } else if matches!(lower.as_str(), "text" | "varchar" | "character varying" | "char")
            || lower.starts_with("varchar")
            || lower.starts_with("char")
            || lower == "text"
        {
            text_cols.push(col.name.as_str());
        }
    }

    ui.horizontal_wrapped(|ui| {
        metric_chip(
            ui,
            &tf("info_view_bi_total_rows", &[&result.rows.len().to_string()]),
            theme::ACCENT_EMERALD,
        );
        metric_chip(
            ui,
            &tf("info_view_bi_numeric_cols", &[&numeric_cols.len().to_string()]),
            theme::ACCENT_BLUE,
        );
        metric_chip(
            ui,
            &tf("info_view_bi_text_cols", &[&text_cols.len().to_string()]),
            theme::ACCENT_COPPER,
        );
    });

    if !numeric_cols.is_empty() {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_bi_numeric_cols"));
        ui.add_space(theme::SPACE_XS);
        for name in numeric_cols.iter().take(8) {
            ui.label(
                RichText::new(*name)
                    .color(theme::text_secondary())
                    .monospace()
                    .size(11.5),
            );
        }
    }
    if !text_cols.is_empty() {
        ui.add_space(theme::SPACE_LG);
        info_section_label(ui, &t("info_view_bi_text_cols"));
        ui.add_space(theme::SPACE_XS);
        for name in text_cols.iter().take(8) {
            ui.label(
                RichText::new(*name)
                    .color(theme::text_secondary())
                    .monospace()
                    .size(11.5),
            );
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
