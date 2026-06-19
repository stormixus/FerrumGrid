//! Drop confirmation dialog — table/view DROP CASCADE 전 dependents 미리보기.
//!
//! US-J1 / Plan v7 Phase 2d. `state.drop_dialog: Option<DropDialogState>` 가
//! Some 일 때 표시되는 modal egui::Window.

use eframe::egui::{self, RichText};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::AppState;
use crate::ui::theme;

/// `state.drop_dialog` 가 Some 일 때 modal 다이얼로그 렌더. Cancel 또는 Drop CASCADE
/// 클릭 시 state.drop_dialog 를 None 으로 비우고 Drop 시 ApplyDdlWithInvalidation
/// 발사.
pub fn render_drop_dialog(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    let dialog = match state.drop_dialog.as_ref() {
        Some(d) => d.clone(),
        None => return,
    };
    let mut close = false;
    let mut confirm = false;

    egui::Window::new(format!("Drop table: {}.{}", dialog.schema, dialog.table))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.set_max_width(560.0);

            ui.label(
                RichText::new(t("drop_irreversible"))
                    .color(theme::ACCENT_RED)
                    .strong()
                    .size(13.0),
            );
            ui.add_space(theme::SPACE_MD);

            ui.label(
                RichText::new(t("drop_dependents_title"))
                    .color(theme::text_primary())
                    .strong()
                    .size(12.0),
            );
            if dialog.loading {
                ui.label(
                    RichText::new(t("drop_dependents_loading"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
            } else if dialog.oid_unavailable {
                ui.label(
                    RichText::new(t("drop_dependents_oid_unavailable"))
                        .color(theme::ACCENT_YELLOW)
                        .size(11.0),
                );
            } else if let Some(err) = &dialog.fetch_error {
                ui.label(
                    RichText::new(tf("drop_dependents_fetch_failed", &[err.as_str()]))
                        .color(theme::ACCENT_RED)
                        .size(11.0),
                );
            } else if dialog.dependents.is_empty() {
                ui.label(
                    RichText::new(t("drop_dependents_none"))
                        .color(theme::ACCENT_GREEN)
                        .size(11.0),
                );
            } else {
                ui.label(
                    RichText::new(if dialog.truncated {
                        tf(
                            "drop_dependents_truncated",
                            &[
                                &crate::db::dependencies::MAX_DISPLAY.to_string(),
                                &crate::db::dependencies::MAX_DISPLAY.to_string(),
                            ],
                        )
                    } else {
                        tf(
                            "drop_dependents_count",
                            &[&dialog.dependents.len().to_string()],
                        )
                    })
                    .color(theme::text_muted())
                    .size(11.0),
                );
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        for dep in &dialog.dependents {
                            ui.label(RichText::new(dep).monospace().size(11.0));
                        }
                    });
            }

            ui.add_space(theme::SPACE_LG);

            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui
                            .add(theme::primary_button(&t("drop_cascade_confirm")))
                            .clicked()
                        {
                            confirm = true;
                        }
                        ui.add_space(theme::SPACE_SM);
                        if ui.add(theme::secondary_button(&t("settings_btn_cancel"))).clicked() {
                            close = true;
                        }
                    },
                );
            });
        });

    if confirm {
        let sql = build_drop_sql(&dialog.kind, &dialog.schema, &dialog.table);
        bridge.send(DbCommand::ApplyDdlWithInvalidation {
            conn_id: dialog.conn_id,
            sql,
            table_oid: None,
            schema_to_refresh: Some(dialog.schema.clone()),
        });
        state.status_message = format!("Dropped {}.{}", dialog.schema, dialog.table);
        state.drop_dialog = None;
    } else if close {
        state.drop_dialog = None;
    }
}

/// US-L1 — DropTargetKind + schema/name → DROP CASCADE SQL.
pub(crate) fn build_drop_sql(
    kind: &crate::state::DropTargetKind,
    schema: &str,
    name: &str,
) -> String {
    format!(
        "DROP {} \"{}\".\"{}\" CASCADE",
        kind.drop_keyword(),
        schema.replace('"', "\"\""),
        name.replace('"', "\"\""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{DropDialogState, DropTargetKind};
    use crate::types::ConnectionId;

    #[test]
    fn drop_dialog_state_new_initializes_loading_true() {
        let cid = ConnectionId(uuid::Uuid::new_v4());
        let d = DropDialogState::new(cid, "public", "users", DropTargetKind::Table);
        assert_eq!(d.schema, "public");
        assert_eq!(d.table, "users");
        assert_eq!(d.kind, DropTargetKind::Table);
        assert!(d.loading);
        assert!(!d.confirming);
        assert!(d.dependents.is_empty());
        assert!(!d.truncated);
        assert!(!d.oid_unavailable);
        assert!(d.fetch_error.is_none());
    }

    #[test]
    fn drop_dialog_state_open_lifecycle_transitions() {
        let cid = ConnectionId(uuid::Uuid::new_v4());
        let mut d = DropDialogState::new(cid, "public", "orders", DropTargetKind::Table);
        assert!(d.loading);
        d.dependents.push("public.order_items".to_string());
        d.loading = false;
        assert!(!d.loading);
        assert_eq!(d.dependents.len(), 1);
        d.confirming = true;
        assert!(d.confirming);
    }

    #[test]
    fn drop_dialog_state_truncated_flag_independent_of_loading() {
        let cid = ConnectionId(uuid::Uuid::new_v4());
        let mut d = DropDialogState::new(cid, "public", "logs", DropTargetKind::Table);
        d.loading = false;
        d.truncated = true;
        for i in 0..50 {
            d.dependents.push(format!("public.log_part_{i}"));
        }
        assert_eq!(d.dependents.len(), 50);
        assert!(d.truncated);
    }

    #[test]
    fn drop_dialog_state_close_resets_to_none_via_appstate() {
        let cid = ConnectionId(uuid::Uuid::new_v4());
        let mut state = AppState {
            drop_dialog: Some(DropDialogState::new(
                cid,
                "public",
                "users",
                DropTargetKind::Table,
            )),
            ..AppState::default()
        };
        assert!(state.drop_dialog.is_some());
        state.drop_dialog = None;
        assert!(state.drop_dialog.is_none());
    }

    #[test]
    fn build_drop_sql_table_uses_drop_table_keyword() {
        let sql = build_drop_sql(&DropTargetKind::Table, "public", "users");
        assert_eq!(sql, "DROP TABLE \"public\".\"users\" CASCADE");
    }

    #[test]
    fn build_drop_sql_view_uses_drop_view_keyword() {
        let sql = build_drop_sql(&DropTargetKind::View, "analytics", "summary_v");
        assert_eq!(sql, "DROP VIEW \"analytics\".\"summary_v\" CASCADE");
    }

    #[test]
    fn build_drop_sql_materialized_view_uses_drop_materialized_view_keyword() {
        let sql =
            build_drop_sql(&DropTargetKind::MaterializedView, "warehouse", "daily_agg");
        assert_eq!(
            sql,
            "DROP MATERIALIZED VIEW \"warehouse\".\"daily_agg\" CASCADE"
        );
    }

    #[test]
    fn build_drop_sql_escapes_identifier_double_quotes() {
        let sql = build_drop_sql(&DropTargetKind::Table, "we\"ird", "ta\"ble");
        assert_eq!(sql, "DROP TABLE \"we\"\"ird\".\"ta\"\"ble\" CASCADE");
    }

    #[test]
    fn drop_target_kind_from_table_type_classifies_correctly() {
        assert_eq!(
            DropTargetKind::from_table_type("BASE TABLE"),
            DropTargetKind::Table
        );
        assert_eq!(
            DropTargetKind::from_table_type("VIEW"),
            DropTargetKind::View
        );
        assert_eq!(
            DropTargetKind::from_table_type("MATERIALIZED VIEW"),
            DropTargetKind::MaterializedView
        );
        assert_eq!(
            DropTargetKind::from_table_type("FOREIGN TABLE"),
            DropTargetKind::Table
        );
    }
}
