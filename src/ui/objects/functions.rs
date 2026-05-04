//! Function objects view.
//!
//! Plan v7 Phase 1.95b3b cut-over (from `super::mod.rs`). Phase 2 의 Create
//! Function template + 즉시 실행 UI 가 본 모듈에 추가될 예정.

use eframe::egui::{self, ScrollArea};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::AppState;
use crate::types::{ConnectionId, FunctionInfo};
use crate::ui::theme;

use super::{
    active_conn, cell_label, data_row, quote_ident, render_count_strip, render_no_connection,
    selected_schemas, table_header, type_chip, ObjectAction, FUNCTION_COLUMNS,
};

pub(super) fn render_functions(
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
                &FUNCTION_COLUMNS,
                &[
                    t("objects_schema"),
                    t("objects_name"),
                    t("objects_signature"),
                    t("objects_returns"),
                    t("objects_lang"),
                    t("objects_actions"),
                ],
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

fn render_function_row(ui: &mut egui::Ui, func: &FunctionInfo) -> Option<String> {
    let mut copied = None;
    data_row(ui, &FUNCTION_COLUMNS, |cells| {
        cells.col(|ui| cell_label(ui, &func.schema, theme::text_muted(), 12.0, false));
        cells.col(|ui| cell_label(ui, &func.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| {
            cell_label(
                ui,
                &format!("({})", func.arguments),
                theme::text_secondary(),
                11.0,
                false,
            );
        });
        cells.col(|ui| type_chip(ui, &func.return_type, theme::ACCENT_TEAL));
        cells.col(|ui| cell_label(ui, &func.language, theme::text_muted(), 11.0, false));
        cells.col(|ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = theme::SPACE_MD;
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
    });
    copied
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

/// Function 호출 SQL 생성 시 인자 placeholder (`$1, $2, ...`) 를 포맷.
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
