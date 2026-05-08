//! Data grid view (cell rendering, selection, hit-test, edit, paste, info
//! panel, footer, tooltips).
//!
//! Plan v7 Phase 1.95c1 — grid.rs (5240줄) 를 폴더 구조로 변환. sub-modules
//! 는 현재 빈 placeholder. 실제 함수 cut-over 는 후속 1.95c sub-stories 에서
//! 진행. Phase 1.95a 의 dispatch() wire-up 도 cut-over 후 진행.

mod cell_overlay;
pub(crate) mod data_ops;
mod date_picker;
mod footer;
mod header;
mod hit_test;
mod info_panel;
mod info_row;
mod json_editor;
mod object_info;
mod pager;
pub(crate) mod paste;
mod render;
mod selection;
mod table_info;
mod table_render;
mod toolbar;
mod tooltips;

use data_ops::*;

use footer::{
    render_data_query_footer, render_grid_body_with_reserved_footer, should_show_data_query_footer,
};
pub use render::render_grid;
pub(super) use header::{render_result_header, result_toolbar_action_width};
pub(super) use table_render::{
    passive_value_pill, render_cell, render_passive_cell, render_passive_copyable_cell,
    render_table, show_cell_copy_context_menu,
};
pub(super) use toolbar::{
    metric_chip, result_toolbar_action_button, show_dark_popup_below,
};
pub use info_panel::{render_info_panel, restore_active_data_tab};
use selection::*;
use tooltips::show_dark_hover_tooltip;

use eframe::egui::Color32;

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    build_data_select_sql_with_columns, cell_edit_text_for_type,
    is_timestamp_without_timezone_type, is_timestamptz_type, timestamp_display_to_utc,
    timestamp_display_to_utc_naive, AppState, DataFilter, DataSortClause, DataSortDirection,
    DataSource, MainView, MAX_DATA_PAGE_LIMIT,
};
use crate::types::{CellValue, ColumnInfo};
use crate::ui::er_diagram::ForeignKey;
use crate::ui::theme;

const GRID_CELL_LEFT_PAD: f32 = 12.0;
const GRID_CELL_RIGHT_PAD: f32 = 8.0;

pub(crate) fn request_foreign_keys_for_schema(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: crate::types::ConnectionId,
    schema: &str,
) {
    let should_request = state.connections.get(&conn_id).is_some_and(|conn| {
        !conn.foreign_keys.contains_key(schema) && !conn.loading_foreign_keys.contains(schema)
    });
    if !should_request {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.loading_foreign_keys.insert(schema.to_string());
    }
    bridge.send(DbCommand::ListForeignKeys {
        conn_id,
        schema: schema.to_string(),
    });
}

pub(crate) fn request_table_columns_for_data(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: crate::types::ConnectionId,
    schema: &str,
    table: &str,
) {
    let key = (schema.to_string(), table.to_string());
    let should_request = state.connections.get(&conn_id).is_some_and(|conn| {
        !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key)
    });
    if !should_request {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.loading_columns.insert(key);
    }
    bridge.send(DbCommand::ListColumns {
        conn_id,
        schema: schema.to_string(),
        table: table.to_string(),
    });
}

pub(super) fn relation_for_column(state: &AppState, column_name: Option<&str>) -> Option<ForeignKey> {
    let column_name = column_name?;
    let source = state.active_data_source()?;
    state
        .connections
        .get(&source.conn_id)?
        .foreign_keys
        .get(&source.schema)?
        .iter()
        .find(|fk| {
            fk.source_schema == source.schema
                && fk.source_table == source.table
                && fk.source_column == column_name
        })
        .cloned()
}

pub(super) fn open_related_data(state: &mut AppState, bridge: &DbBridge, fk: &ForeignKey, filter: DataFilter) {
    let Some(conn_id) = state.active_data_source().map(|source| source.conn_id) else {
        return;
    };

    state.active_connection = Some(conn_id);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    state.last_error = None;

    let title = relation_tab_title(fk, &filter.display_value);
    state.open_data_workspace_view(
        title,
        fk.target_schema.clone(),
        fk.target_table.clone(),
        Some(filter.clone()),
    );
    state.begin_data_edit_with_filter(
        conn_id,
        &fk.target_schema,
        &fk.target_table,
        Some(filter.clone()),
    );
    request_table_columns_for_data(state, bridge, conn_id, &fk.target_schema, &fk.target_table);
    request_foreign_keys_for_schema(state, bridge, conn_id, &fk.target_schema);

    let source = DataSource {
        conn_id,
        schema: fk.target_schema.clone(),
        table: fk.target_table.clone(),
        filter: Some(filter),
    };
    let limit = state.data_edit.page_limit;
    let columns = state.data_columns_for_source(&source);
    bridge.send(DbCommand::ExecuteQuery {
        conn_id,
        sql: build_data_select_sql_with_columns(&source, &state.data_edit.sort, limit, 0, &columns),
        row_limit: Some(limit),
    });
}

