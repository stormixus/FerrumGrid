//! Role / User objects view.
//!
//! Plan v7 Phase 1.95b3b cut-over (from `super::mod.rs`). Phase 2 의 Create/
//! Alter Role UI 가 본 모듈에 추가될 예정.

use eframe::egui::{self, ScrollArea};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::t;
use crate::state::AppState;
use crate::types::{ConnectionId, RoleInfo};
use crate::ui::theme;

use super::{
    active_conn, cell_label, data_row, quote_ident, render_count_strip, render_no_connection,
    table_header, type_chip, ObjectAction, ROLE_COLUMNS,
};

pub(super) fn render_roles(
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
    ScrollArea::both()
        .id_salt("objects_role_rows")
        .show(ui, |ui| {
            table_header(
                ui,
                &ROLE_COLUMNS,
                &[
                    t("objects_role"),
                    t("objects_login"),
                    t("objects_privileges"),
                    t("objects_valid_until"),
                    t("objects_actions"),
                ],
            );
            for role in rows {
                if let Some(row_action) = render_role_row(ui, &role) {
                    action = Some(row_action);
                }
            }
        });
    action
}

fn render_role_row(ui: &mut egui::Ui, role: &RoleInfo) -> Option<ObjectAction> {
    let mut action: Option<ObjectAction> = None;
    let response = data_row(ui, &ROLE_COLUMNS, |cells| {
        cells.col(|ui| cell_label(ui, &role.name, theme::text_primary(), 12.0, true));
        cells.col(|ui| {
            type_chip(
                ui,
                if role.can_login { "LOGIN" } else { "NOLOGIN" },
                if role.can_login {
                    theme::ACCENT_GREEN
                } else {
                    theme::text_muted()
                },
            );
        });
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
        let privileges = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.join(", ")
        };
        cells.col(|ui| cell_label(ui, &privileges, theme::text_secondary(), 11.0, false));
        cells.col(|ui| {
            cell_label(
                ui,
                role.valid_until.as_deref().unwrap_or("-"),
                theme::text_muted(),
                11.0,
                false,
            );
        });
        cells.col(|ui| {
            if ui.small_button("SQL").clicked() {
                action = Some(ObjectAction::CopySql(format!(
                    "ALTER ROLE {};",
                    quote_ident(&role.name)
                )));
            }
        });
    });
    if action.is_none() && response.clicked() {
        action = Some(ObjectAction::SelectRole {
            name: role.name.clone(),
        });
    }
    action
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
