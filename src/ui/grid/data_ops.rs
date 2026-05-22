//! Data state operation helpers — sort, pagination, dirty tracking, column
//! metadata, edit construction / validation.

use super::*;

// ---------------------------------------------------------------------------
// Sort helpers
// ---------------------------------------------------------------------------

pub(super) fn set_sort_clause(
    state: &mut AppState,
    bridge: &DbBridge,
    column: &str,
    direction: DataSortDirection,
) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    if let Some(clause) = state
        .data_edit
        .sort
        .iter_mut()
        .find(|clause| clause.column == column)
    {
        clause.direction = direction;
    } else {
        state.data_edit.sort.push(DataSortClause {
            column: column.to_string(),
            direction,
        });
    }
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

pub(super) fn remove_sort_clause(state: &mut AppState, bridge: &DbBridge, column: &str) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state
        .data_edit
        .sort
        .retain(|clause| clause.column != column);
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

pub(super) fn clear_sort_clauses(state: &mut AppState, bridge: &DbBridge) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_sort_unsaved"));
        return;
    }

    state.data_edit.sort.clear();
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

pub(crate) fn set_data_page_index(state: &mut AppState, bridge: &DbBridge, page_index: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    state.data_edit.page_index = page_index;
    state.data_edit.page_index_input = (page_index + 1).to_string();
    reload_data_source(state, bridge);
}

pub(crate) fn apply_data_page_input(state: &mut AppState, bridge: &DbBridge) -> bool {
    let raw = state.data_edit.page_index_input.trim().replace(',', "");
    match raw.parse::<usize>() {
        Ok(page) => {
            let page_index = page.max(1) - 1;
            set_data_page_index(state, bridge, page_index);
            true
        }
        Err(_) => {
            state.last_error = Some(t("grid_limit_error"));
            false
        }
    }
}

pub(crate) fn apply_data_limit_input(state: &mut AppState, bridge: &DbBridge) -> bool {
    let raw = state.data_edit.page_limit_input.trim().replace(',', "");
    match raw.parse::<usize>() {
        Ok(limit) => {
            set_data_limit(state, bridge, limit);
            true
        }
        Err(_) => {
            state.last_error = Some(t("grid_limit_error"));
            false
        }
    }
}

pub(super) fn set_data_limit(state: &mut AppState, bridge: &DbBridge, limit: usize) {
    if has_dirty_data_edits(state) {
        state.last_error = Some(t("grid_page_unsaved"));
        return;
    }

    let limit = limit.clamp(1, MAX_DATA_PAGE_LIMIT);
    state.data_edit.page_limit = limit;
    state.data_edit.page_limit_input = limit.to_string();
    state.data_edit.page_index = 0;
    state.data_edit.page_index_input = "1".to_string();
    reload_data_source(state, bridge);
}

// ---------------------------------------------------------------------------
// Data reload
// ---------------------------------------------------------------------------

pub(crate) fn reload_data_source(state: &mut AppState, bridge: &DbBridge) {
    let Some(source) = state.active_data_source() else {
        return;
    };
    let limit = normalized_data_limit(state);
    let offset = data_page_offset(state);
    state.current_result = None;
    state.current_result_truncated = false;
    state.query_running = true;
    state.last_error = None;
    state.data_edit.selected_cell = None;
    state.data_edit.editing_cell = None;
    bridge.send(DbCommand::ExecuteQuery {
        conn_id: source.conn_id,
        sql: build_data_select_sql_with_columns(
            &source,
            &state.data_edit.sort,
            limit,
            offset,
            &state.data_columns_for_source(&source),
        ),
        row_limit: Some(limit),
    });
}

pub(crate) fn normalized_data_limit(state: &AppState) -> usize {
    state.data_edit.page_limit.clamp(1, MAX_DATA_PAGE_LIMIT)
}

pub(crate) fn data_page_offset(state: &AppState) -> usize {
    state
        .data_edit
        .page_index
        .saturating_mul(normalized_data_limit(state))
}

/// Plan v7 / US-I1 — Paste/Delete StateOp 가 미등록 셀에서도 동작하도록 시드.
/// `current_result.rows[row][col]` 에서 CellValue 를 읽고 columns[col].type_name 으로
/// EditableCell 등록. 이미 등록된 셀은 noop. row/col out-of-bounds 시 noop.
///
/// **Apply/Revert invariant 보장** — original 스냅샷은 result row 의 실제 DB 값
/// 으로 시드되므로 dirty 추적과 revert 가 정확히 동작.
pub(crate) fn ensure_data_edit_cell_from_result(
    state: &mut AppState,
    row_idx: usize,
    col_idx: usize,
) {
    if state.data_edit.cells.contains_key(&(row_idx, col_idx)) {
        return;
    }
    let Some(result) = state.current_result.as_ref() else {
        return;
    };
    let Some(row) = result.rows.get(row_idx) else {
        return;
    };
    let Some(cell) = row.get(col_idx) else {
        return;
    };
    debug_assert_eq!(
        result.columns.len(),
        row.len(),
        "QueryResult invariant: row width must match columns count"
    );
    let type_name = result
        .columns
        .get(col_idx)
        .map(|c| c.type_name.as_str())
        .unwrap_or("")
        .to_string();
    let cell_clone = cell.clone();
    let timezone = state.data_timezone.clone();
    state.data_edit.cells.insert(
        (row_idx, col_idx),
        crate::state::EditableCell::from_cell_for_type(&cell_clone, &type_name, &timezone),
    );
}

// ---------------------------------------------------------------------------
// Dirty tracking
// ---------------------------------------------------------------------------

pub(super) fn has_dirty_data_edits(state: &AppState) -> bool {
    !state.data_edit.pending_deletes.is_empty()
        || !state.data_edit.inserted_rows.is_empty()
        || state.data_edit.cells.values().any(|cell| cell.is_dirty())
}

#[derive(Clone, Copy)]
pub(super) enum EditKind {
    Bool,
    Number,
    Json,
    Date,
    DateTime,
    Uuid,
    Bytes,
    Text,
}

pub(crate) struct DataEditSummary {
    pub(crate) conn_id: crate::types::ConnectionId,
    pub(crate) dirty_count: usize,
    pub(crate) can_apply: bool,
    pub(crate) blocked_reason: Option<String>,
    pub(crate) color: Color32,
}

pub(crate) fn data_edit_summary(state: &AppState) -> Option<DataEditSummary> {
    if state.active_main_view != MainView::Data {
        return None;
    }

    let source = state.active_data_source()?;
    let cell_dirty_count = state
        .data_edit
        .cells
        .values()
        .filter(|cell| cell.is_dirty())
        .count();
    let dirty_count = cell_dirty_count
        + state.data_edit.pending_deletes.len()
        + state.data_edit.inserted_rows.len();
    if dirty_count == 0 {
        return None;
    }

    let pk_columns = primary_key_columns(state);
    let invalid_count = count_invalid_edits(state);
    // Plan v7 §10 / Phase 1.2 — PK 컬럼 타입 화이트리스트 가드.
    // 비허용 타입 (json/jsonb, array, range, composite 등) 인 PK 가 하나라도 있으면
    // mutation UI hard-disable + 배너 (silent 데이터 손상 차단).
    let unsupported_pk = pk_columns
        .iter()
        .find(|col| !crate::db::row_key::is_pk_type_allowed(&col.data_type));
    let blocked_reason = if pk_columns.is_empty() {
        Some(t("grid_pk_required"))
    } else if let Some(col) = unsupported_pk {
        Some(format!(
            "PK column '{}' has unsupported type '{}' for safe row identity \
             — editing disabled to prevent data loss.",
            col.name, col.data_type
        ))
    } else if invalid_count > 0 {
        Some(tf("grid_invalid_values", &[&invalid_count.to_string()]))
    } else {
        None
    };
    let can_apply = blocked_reason.is_none() && !state.data_edit.applying;
    let color = if blocked_reason.is_some() {
        theme::ACCENT_YELLOW
    } else {
        theme::accent_color()
    };

    Some(DataEditSummary {
        conn_id: source.conn_id,
        dirty_count,
        can_apply,
        blocked_reason,
        color,
    })
}

// ---------------------------------------------------------------------------
// Column helpers
// ---------------------------------------------------------------------------

pub(super) fn data_column_info<'a>(state: &'a AppState, column_name: &str) -> Option<&'a ColumnInfo> {
    let source = state.active_data_source()?;
    state
        .connections
        .get(&source.conn_id)?
        .columns
        .get(&(source.schema, source.table))?
        .iter()
        .find(|col| col.name == column_name)
}

pub(super) fn has_table_column_metadata(state: &AppState) -> bool {
    let Some(source) = state.active_data_source() else {
        return false;
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .is_some()
}

pub(super) fn table_columns(state: &AppState) -> Vec<ColumnInfo> {
    let Some(source) = state.active_data_source() else {
        return Vec::new();
    };
    state
        .connections
        .get(&source.conn_id)
        .and_then(|conn| conn.columns.get(&(source.schema, source.table)))
        .cloned()
        .unwrap_or_default()
}

fn primary_key_columns(state: &AppState) -> Vec<ColumnInfo> {
    table_columns(state)
        .into_iter()
        .filter(|col| col.is_primary_key)
        .collect()
}

// ---------------------------------------------------------------------------
// Edit operations
// ---------------------------------------------------------------------------

pub(crate) fn build_data_edits(state: &AppState) -> Result<Vec<crate::db::edits::RowEditOp>, String> {
    use crate::db::edits::{ColumnAssignment, EditValue, PkColumn, RowEditOp};
    let source = state
        .active_data_source()
        .ok_or_else(|| t("grid_no_active_data_source"))?;
    let result = state
        .current_result
        .as_ref()
        .ok_or_else(|| t("grid_no_result_set"))?;
    let table_columns = table_columns(state);
    let pk_columns: Vec<ColumnInfo> = table_columns
        .iter()
        .filter(|col| col.is_primary_key)
        .cloned()
        .collect();
    if pk_columns.is_empty() {
        return Err(t("grid_pk_required"));
    }

    let mut edits = Vec::new();

    // INSERT ops for inserted rows
    for &row_idx in &state.data_edit.inserted_rows {
        if state.data_edit.pending_deletes.contains(&row_idx) {
            continue;
        }
        let mut columns = Vec::new();
        for (col_idx, col_meta) in result.columns.iter().enumerate() {
            let col_info = table_columns.iter().find(|info| info.name == col_meta.name);
            if col_info.is_some_and(|info| info.default_value.is_some() || info.is_primary_key) {
                if let Some(cell) = state.data_edit.cells.get(&(row_idx, col_idx)) {
                    if !cell.is_dirty() {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            let column_type = col_info
                .map(|info| info.data_type.clone())
                .unwrap_or_else(|| col_meta.type_name.clone());
            let value = if let Some(cell) = state.data_edit.cells.get(&(row_idx, col_idx)) {
                if cell.is_null {
                    EditValue::Null
                } else {
                    EditValue::Text(cell.value.clone())
                }
            } else {
                EditValue::Null
            };
            columns.push(ColumnAssignment {
                column: col_meta.name.clone(),
                column_type,
                value,
            });
        }
        let returning_pk: Vec<String> = pk_columns.iter().map(|c| c.name.clone()).collect();
        edits.push(RowEditOp::Insert {
            tmp_id: uuid::Uuid::new_v4(),
            schema: source.schema.clone(),
            table: source.table.clone(),
            columns,
            returning_pk,
        });
    }

    // UPDATE ops for existing dirty cells (skip inserted/deleted rows)
    for ((row_idx, col_idx), cell) in &state.data_edit.cells {
        if !cell.is_dirty() {
            continue;
        }
        if state.data_edit.inserted_rows.contains(row_idx) {
            continue;
        }
        if state.data_edit.pending_deletes.contains(row_idx) {
            continue;
        }
        let column = result
            .columns
            .get(*col_idx)
            .ok_or_else(|| t("grid_column_missing"))?;
        let column_info = table_columns
            .iter()
            .find(|info| info.name == column.name)
            .cloned();
        if column_info.as_ref().is_some_and(|info| info.is_primary_key) {
            continue;
        }
        let column_type = column_info
            .as_ref()
            .map(|info| info.data_type.clone())
            .unwrap_or_else(|| column.type_name.clone());
        let nullable = column_info
            .as_ref()
            .map(|info| info.is_nullable)
            .unwrap_or(true);
        let enum_values = column_info
            .as_ref()
            .map(|info| info.enum_values.as_slice())
            .unwrap_or(&[]);
        if let Some(error) = validate_edit_value(cell, &column_type, nullable, enum_values) {
            return Err(error);
        }

        let mut pk = Vec::new();
        for pk_col in &pk_columns {
            let pk_idx = result
                .columns
                .iter()
                .position(|col| col.name == pk_col.name)
                .ok_or_else(|| tf("grid_pk_missing", &[&pk_col.name]))?;
            let original = state
                .data_edit
                .cells
                .get(&(*row_idx, pk_idx))
                .map(|cell| cell.original.clone())
                .or_else(|| {
                    result
                        .rows
                        .get(*row_idx)
                        .and_then(|row| row.get(pk_idx))
                        .cloned()
                })
                .ok_or_else(|| t("grid_pk_value_missing"))?;
            pk.push(PkColumn {
                column: pk_col.name.clone(),
                column_type: pk_col.data_type.clone(),
                value: original,
            });
        }

        let value = if cell.is_null {
            EditValue::Null
        } else if is_timestamptz_type(&column_type) {
            EditValue::Text(
                timestamp_display_to_utc(&cell.value, &state.data_timezone)
                    .unwrap_or_else(|| cell.value.clone()),
            )
        } else if is_timestamp_without_timezone_type(&column_type) {
            EditValue::Text(
                timestamp_display_to_utc_naive(&cell.value, &state.data_timezone)
                    .unwrap_or_else(|| cell.value.clone()),
            )
        } else {
            EditValue::Text(cell.value.clone())
        };

        edits.push(RowEditOp::Update {
            schema: source.schema.clone(),
            table: source.table.clone(),
            column: ColumnAssignment {
                column: column.name.clone(),
                column_type,
                value,
            },
            pk,
        });
    }

    // DELETE ops for pending deletes (skip inserted rows — those are just discarded)
    for &row_idx in &state.data_edit.pending_deletes {
        if state.data_edit.inserted_rows.contains(&row_idx) {
            continue;
        }
        let Some(row_data) = result.rows.get(row_idx) else {
            continue;
        };
        let mut pk = Vec::new();
        for pk_col in &pk_columns {
            let pk_idx = result
                .columns
                .iter()
                .position(|col| col.name == pk_col.name)
                .ok_or_else(|| tf("grid_pk_missing", &[&pk_col.name]))?;
            let value = row_data
                .get(pk_idx)
                .cloned()
                .ok_or_else(|| t("grid_pk_value_missing"))?;
            pk.push(PkColumn {
                column: pk_col.name.clone(),
                column_type: pk_col.data_type.clone(),
                value,
            });
        }
        edits.push(RowEditOp::Delete {
            schema: source.schema.clone(),
            table: source.table.clone(),
            pk,
        });
    }

    Ok(edits)
}

fn count_invalid_edits(state: &AppState) -> usize {
    let Some(result) = state.current_result.as_ref() else {
        return 0;
    };
    state
        .data_edit
        .cells
        .iter()
        .filter(|((_, col_idx), cell)| {
            if !cell.is_dirty() {
                return false;
            }
            let Some(column) = result.columns.get(*col_idx) else {
                return true;
            };
            let info = data_column_info(state, &column.name);
            let type_name = info
                .map(|info| info.data_type.as_str())
                .unwrap_or(column.type_name.as_str());
            let nullable = info.map(|info| info.is_nullable).unwrap_or(true);
            let enum_values = info.map(|info| info.enum_values.as_slice()).unwrap_or(&[]);
            validate_edit_value(cell, type_name, nullable, enum_values).is_some()
        })
        .count()
}

pub(crate) fn revert_data_edits(state: &mut AppState) {
    // Remove inserted rows from result
    if !state.data_edit.inserted_rows.is_empty() {
        let mut to_remove: Vec<usize> = state.data_edit.inserted_rows.iter().copied().collect();
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        if let Some(result) = state.current_result.as_mut() {
            for idx in &to_remove {
                if *idx < result.rows.len() {
                    result.rows.remove(*idx);
                }
            }
        }
        state.data_edit.cells.retain(|(row, _), _| !state.data_edit.inserted_rows.contains(row));
        state.data_edit.inserted_rows.clear();
    }

    state.data_edit.pending_deletes.clear();

    let column_types = state
        .current_result
        .as_ref()
        .map(|result| {
            result
                .columns
                .iter()
                .map(|column| column.type_name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for ((_, col_idx), cell) in state.data_edit.cells.iter_mut() {
        let type_name = column_types
            .get(*col_idx)
            .map(String::as_str)
            .unwrap_or_default();
        cell.value = cell_edit_text_for_type(&cell.original, type_name, &state.data_timezone);
        cell.original_text = cell.value.clone();
        cell.is_null = matches!(cell.original, CellValue::Null);
    }
    state.data_edit.editing_cell = None;
}

// ---------------------------------------------------------------------------
// Edit kind / validation
// ---------------------------------------------------------------------------

pub(super) fn validate_edit_value(
    cell: &crate::state::EditableCell,
    type_name: &str,
    nullable: bool,
    enum_values: &[String],
) -> Option<String> {
    if cell.is_null {
        return (!nullable).then(|| t("grid_not_null"));
    }

    if !enum_values.is_empty() && !enum_values.iter().any(|value| value == cell.value.trim()) {
        return Some(t("grid_enum_error"));
    }

    match edit_kind(type_name, &cell.original) {
        EditKind::Bool => parse_bool(&cell.value)
            .is_none()
            .then(|| t("grid_bool_error")),
        EditKind::Number => cell
            .value
            .trim()
            .parse::<f64>()
            .is_err()
            .then(|| t("grid_number_error")),
        EditKind::Json => serde_json::from_str::<serde_json::Value>(&cell.value)
            .is_err()
            .then(|| t("grid_json_error")),
        EditKind::Date => (!is_valid_date(&cell.value)).then(|| t("grid_date_error")),
        EditKind::DateTime => (!is_valid_datetime(&cell.value)).then(|| t("grid_datetime_error")),
        EditKind::Uuid => uuid::Uuid::parse_str(cell.value.trim())
            .is_err()
            .then(|| t("grid_uuid_error")),
        EditKind::Bytes => {
            let value = cell
                .value
                .trim()
                .strip_prefix("\\x")
                .unwrap_or(cell.value.trim());
            (!value.chars().all(|ch| ch.is_ascii_hexdigit()) || !value.len().is_multiple_of(2))
                .then(|| t("grid_bytes_error"))
        }
        EditKind::Text => None,
    }
}

pub(super) fn edit_kind(type_name: &str, cell: &CellValue) -> EditKind {
    let lower = type_name.to_ascii_lowercase();
    if matches!(cell, CellValue::Bool(_)) || matches!(lower.as_str(), "bool" | "boolean") {
        EditKind::Bool
    } else if matches!(cell, CellValue::Int(_) | CellValue::Float(_))
        || matches!(
            lower.as_str(),
            "smallint"
                | "integer"
                | "bigint"
                | "int2"
                | "int4"
                | "int8"
                | "real"
                | "double precision"
                | "float4"
                | "float8"
                | "numeric"
                | "decimal"
        )
    {
        EditKind::Number
    } else if matches!(cell, CellValue::Json(_)) || matches!(lower.as_str(), "json" | "jsonb") {
        EditKind::Json
    } else if lower == "date" {
        EditKind::Date
    } else if matches!(cell, CellValue::Timestamp(_))
        || matches!(
            lower.as_str(),
            "timestamp"
                | "timestamptz"
                | "timestamp without time zone"
                | "timestamp with time zone"
        )
    {
        EditKind::DateTime
    } else if matches!(cell, CellValue::Uuid(_)) || lower == "uuid" {
        EditKind::Uuid
    } else if matches!(cell, CellValue::Bytes(_)) || lower == "bytea" {
        EditKind::Bytes
    } else {
        EditKind::Text
    }
}

fn is_valid_date(value: &str) -> bool {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").is_ok()
}

fn is_valid_datetime(value: &str) -> bool {
    let value = value.trim();
    if chrono::DateTime::parse_from_rfc3339(value).is_ok() {
        return true;
    }

    [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
    ]
    .iter()
    .any(|format| chrono::NaiveDateTime::parse_from_str(value, format).is_ok())
}

pub(super) fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "y" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CellValue, ColumnMeta, QueryResult};

    fn make_state(rows: usize, cols: usize) -> AppState {
        let mut state = AppState::default();
        let columns = (0..cols)
            .map(|i| ColumnMeta {
                name: format!("c{i}"),
                type_name: if i == 0 { "int4".to_string() } else { "text".to_string() },
            })
            .collect();
        let result_rows = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| {
                        if c == 0 {
                            CellValue::Int(r as i64 + 100)
                        } else {
                            CellValue::Text(format!("v{r}_{c}"))
                        }
                    })
                    .collect()
            })
            .collect();
        state.current_result = Some(QueryResult {
            columns,
            rows: result_rows,
            execution_time_ms: 0,
        });
        state
    }

    #[test]
    fn ensure_from_result_registers_new_cell_with_correct_type() {
        let mut state = make_state(3, 2);
        ensure_data_edit_cell_from_result(&mut state, 1, 0);
        let cell = state.data_edit.cells.get(&(1, 0)).expect("registered");
        // original should be the result row's CellValue (Int(101))
        assert!(matches!(cell.original, CellValue::Int(101)));
    }

    #[test]
    fn ensure_from_result_is_noop_for_already_registered_cell() {
        let mut state = make_state(3, 2);
        // Pre-register with a different value
        state.data_edit.cells.insert(
            (1, 0),
            crate::state::EditableCell {
                original: CellValue::Int(999),
                original_text: "999".to_string(),
                value: "edited".to_string(),
                is_null: false,
            },
        );
        ensure_data_edit_cell_from_result(&mut state, 1, 0);
        let cell = state.data_edit.cells.get(&(1, 0)).unwrap();
        assert!(matches!(cell.original, CellValue::Int(999)));
        assert_eq!(cell.value, "edited");
    }

    #[test]
    fn ensure_from_result_out_of_bounds_is_noop() {
        let mut state = make_state(2, 2);
        ensure_data_edit_cell_from_result(&mut state, 99, 0);
        ensure_data_edit_cell_from_result(&mut state, 0, 99);
        assert!(state.data_edit.cells.is_empty());
    }

    #[test]
    fn ensure_from_result_no_current_result_is_noop() {
        let mut state = AppState::default();
        ensure_data_edit_cell_from_result(&mut state, 0, 0);
        assert!(state.data_edit.cells.is_empty());
    }
}
