//! Data edit state — cell-level edits, dirty tracking, in-memory representation
//! of pending INSERT/UPDATE/DELETE staging.
//!
//! Plan v7 Phase 1.95c2 cut-over (from `super::mod.rs`). Hosts:
//! - `DataSource` / `DataFilter` / `DataSortDirection` / `DataSortClause`
//! - `EditableCell` / `DataEditState` (in-memory representation)
//! - `DEFAULT_DATA_PAGE_LIMIT` / `MAX_DATA_PAGE_LIMIT`
//! - SQL building / cell formatting / timezone helpers
//!
//! `AppState` impl methods that reference these types remain in `super::mod.rs`
//! and access this module via the parent's `pub use data_edit::*` re-exports
//! (backward-compat for external `crate::state::*` callers).

use std::collections::HashMap;

use chrono::{TimeZone, Utc};

use crate::types::{CellValue, ColumnInfo, ConnectionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource {
    pub conn_id: ConnectionId,
    pub schema: String,
    pub table: String,
    pub filter: Option<DataFilter>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataFilter {
    pub column: String,
    pub column_type: String,
    pub display_value: String,
    pub sql_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSortClause {
    pub column: String,
    pub direction: DataSortDirection,
}

pub const DEFAULT_DATA_PAGE_LIMIT: usize = 100;
pub const MAX_DATA_PAGE_LIMIT: usize = 1_000_000;

#[derive(Debug, Clone)]
pub struct EditableCell {
    pub original: CellValue,
    pub original_text: String,
    pub value: String,
    pub is_null: bool,
}

impl EditableCell {
    pub fn from_cell_for_type(cell: &CellValue, type_name: &str, timezone: &str) -> Self {
        let value = cell_edit_text_for_type(cell, type_name, timezone);
        Self {
            original: cell.clone(),
            original_text: value.clone(),
            value,
            is_null: matches!(cell, CellValue::Null),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.is_null != matches!(self.original, CellValue::Null)
            || (!self.is_null && self.value != self.original_text)
    }
}

#[derive(Debug, Clone)]
pub struct DataEditState {
    pub source: Option<DataSource>,
    pub cells: HashMap<(usize, usize), EditableCell>,
    pub sort: Vec<DataSortClause>,
    pub applying: bool,
    pub page_limit: usize,
    pub page_limit_input: String,
    pub page_index: usize,
    pub page_index_input: String,
    pub selected_cell: Option<(usize, usize)>,
    pub editing_cell: Option<(usize, usize)>,
}

impl Default for DataEditState {
    fn default() -> Self {
        Self {
            source: None,
            cells: HashMap::new(),
            sort: Vec::new(),
            applying: false,
            page_limit: DEFAULT_DATA_PAGE_LIMIT,
            page_limit_input: DEFAULT_DATA_PAGE_LIMIT.to_string(),
            page_index: 0,
            page_index_input: "1".to_string(),
            selected_cell: None,
            editing_cell: None,
        }
    }
}

pub fn build_data_select_sql_with_columns(
    source: &DataSource,
    sort: &[DataSortClause],
    limit: usize,
    offset: usize,
    columns: &[ColumnInfo],
) -> String {
    let select_list = if columns.is_empty() {
        "*".to_string()
    } else {
        columns
            .iter()
            .map(|column| {
                if column.enum_values.is_empty() {
                    quote_ident(&column.name)
                } else {
                    format!(
                        "{}::text AS {}",
                        quote_ident(&column.name),
                        quote_ident(&column.name)
                    )
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let where_clause = source
        .filter
        .as_ref()
        .map(|filter| {
            format!(
                " WHERE {} = {}",
                quote_ident(&filter.column),
                filter.sql_value
            )
        })
        .unwrap_or_default();
    let order_by = if sort.is_empty() {
        String::new()
    } else {
        format!(
            " ORDER BY {}",
            sort.iter()
                .map(|clause| {
                    let direction = match clause.direction {
                        DataSortDirection::Asc => "ASC",
                        DataSortDirection::Desc => "DESC",
                    };
                    format!("{} {}", quote_ident(&clause.column), direction)
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let limit = limit.clamp(1, MAX_DATA_PAGE_LIMIT);
    let fetch_limit = limit.saturating_add(1);
    let offset_clause = if offset > 0 {
        format!(" OFFSET {}", offset)
    } else {
        String::new()
    };
    format!(
        "SELECT {} FROM {}.{}{}{} LIMIT {}{}",
        select_list,
        quote_ident(&source.schema),
        quote_ident(&source.table),
        where_clause,
        order_by,
        fetch_limit,
        offset_clause
    )
}

pub fn data_filter_from_cell(
    column: impl Into<String>,
    column_type: impl Into<String>,
    cell: &CellValue,
) -> Option<DataFilter> {
    if matches!(cell, CellValue::Null) {
        return None;
    }

    let column = column.into();
    let column_type = column_type.into();
    let display_value = cell.to_string();
    let sql_value = cell_to_sql_literal(cell, &column_type);
    Some(DataFilter {
        column,
        column_type,
        display_value,
        sql_value,
    })
}

pub fn data_filter_from_text(
    column: impl Into<String>,
    column_type: impl Into<String>,
    value: &str,
) -> Option<DataFilter> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        return None;
    }

    let column = column.into();
    let column_type = column_type.into();
    let sql_value = text_to_sql_literal(value, &column_type);
    Some(DataFilter {
        column,
        column_type,
        display_value: value.to_string(),
        sql_value,
    })
}

fn cell_to_sql_literal(cell: &CellValue, type_name: &str) -> String {
    match cell {
        CellValue::Null => "NULL".to_string(),
        CellValue::Bool(value) => value.to_string(),
        CellValue::Int(value) => value.to_string(),
        CellValue::Float(value) => value.to_string(),
        CellValue::Text(value) | CellValue::Timestamp(value) | CellValue::Unknown(value) => {
            text_to_sql_literal(value, type_name)
        }
        CellValue::Json(value) => quote_literal(&value.to_string()),
        CellValue::Uuid(value) => quote_literal(&value.to_string()),
        CellValue::Bytes(value) => quote_literal(&format!("\\x{}", hex_encode(value))),
    }
}

fn text_to_sql_literal(value: &str, type_name: &str) -> String {
    let value = value.trim();
    match type_name.to_ascii_lowercase().as_str() {
        "bool" | "boolean" => value.to_ascii_lowercase(),
        "smallint" | "integer" | "bigint" | "int2" | "int4" | "int8" | "real"
        | "double precision" | "float4" | "float8" | "numeric" | "decimal" => value.to_string(),
        _ => quote_literal(value),
    }
}

fn cell_edit_text(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Text(v) | CellValue::Timestamp(v) | CellValue::Unknown(v) => v.clone(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Json(v) => v.to_string(),
        CellValue::Uuid(v) => v.to_string(),
        CellValue::Bytes(v) => format!("\\x{}", hex_encode(v)),
    }
}

pub fn cell_edit_text_for_type(cell: &CellValue, type_name: &str, timezone: &str) -> String {
    let raw = cell_edit_text(cell);
    if !is_timestamp_type(type_name) {
        return raw;
    }

    parse_utc_datetime(&raw)
        .and_then(|utc| {
            data_timezone_offset_seconds(timezone).and_then(|seconds| {
                chrono::FixedOffset::east_opt(seconds).map(|offset| {
                    utc.with_timezone(&offset)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
            })
        })
        .unwrap_or(raw)
}

pub fn timestamp_display_to_utc(value: &str, timezone: &str) -> Option<String> {
    let offset = chrono::FixedOffset::east_opt(data_timezone_offset_seconds(timezone)?)?;
    let naive = parse_display_datetime(value)?;
    let local = offset.from_local_datetime(&naive).single()?;
    Some(
        local
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S%:z")
            .to_string(),
    )
}

pub fn timestamp_display_to_utc_naive(value: &str, timezone: &str) -> Option<String> {
    let offset = chrono::FixedOffset::east_opt(data_timezone_offset_seconds(timezone)?)?;
    let naive = parse_display_datetime(value)?;
    let local = offset.from_local_datetime(&naive).single()?;
    Some(
        local
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    )
}

pub fn is_timestamp_type(type_name: &str) -> bool {
    is_timestamptz_type(type_name) || is_timestamp_without_timezone_type(type_name)
}

pub fn is_timestamptz_type(type_name: &str) -> bool {
    matches!(
        type_name.to_ascii_lowercase().as_str(),
        "timestamptz" | "timestamp with time zone"
    )
}

pub fn is_timestamp_without_timezone_type(type_name: &str) -> bool {
    matches!(
        type_name.to_ascii_lowercase().as_str(),
        "timestamp" | "timestamp without time zone"
    )
}

pub fn data_timezone_options() -> &'static [(&'static str, &'static str)] {
    &[
        ("Asia/Seoul", "Asia/Seoul (KST, UTC+09:00)"),
        ("UTC", "UTC / Greenwich"),
        ("local", "System Local Time"),
        ("+09:00", "UTC+09:00"),
        ("+00:00", "UTC+00:00"),
    ]
}

pub fn data_timezone_label(value: &str) -> String {
    data_timezone_options()
        .iter()
        .find(|(code, _)| *code == value)
        .map(|(_, label)| (*label).to_string())
        .unwrap_or_else(|| value.to_string())
}

pub fn data_timezone_offset_seconds(value: &str) -> Option<i32> {
    match value.trim() {
        "Asia/Seoul" => Some(9 * 3600),
        "UTC" | "Etc/UTC" | "GMT" | "+00:00" | "-00:00" => Some(0),
        "local" => Some(chrono::Local::now().offset().local_minus_utc()),
        other => parse_offset_seconds(other),
    }
}

fn parse_offset_seconds(value: &str) -> Option<i32> {
    let sign = match value.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &value[1..];
    let (hours, minutes) = rest.split_once(':')?;
    let hours = hours.parse::<i32>().ok()?;
    let minutes = minutes.parse::<i32>().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

fn parse_utc_datetime(value: &str) -> Option<chrono::DateTime<Utc>> {
    let trimmed = value.trim();
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Some(datetime.with_timezone(&Utc));
    }
    if let Ok(datetime) = chrono::DateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S %z") {
        return Some(datetime.with_timezone(&Utc));
    }
    let without_utc = trimmed
        .strip_suffix(" UTC")
        .or_else(|| trimmed.strip_suffix("Z"))
        .unwrap_or(trimmed);
    parse_display_datetime(without_utc).map(|naive| Utc.from_utc_datetime(&naive))
}

fn parse_display_datetime(value: &str) -> Option<chrono::NaiveDateTime> {
    let value = value.trim();
    [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
    ]
    .iter()
    .find_map(|format| chrono::NaiveDateTime::parse_from_str(value, format).ok())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn quote_literal(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_timestamp_without_timezone_in_configured_timezone() {
        let cell = CellValue::Timestamp("2022-12-24 18:10:00".to_string());

        assert_eq!(
            cell_edit_text_for_type(&cell, "timestamp without time zone", "Asia/Seoul"),
            "2022-12-25 03:10:00"
        );
    }

    #[test]
    fn converts_display_timestamp_without_timezone_back_to_utc_naive() {
        assert_eq!(
            timestamp_display_to_utc_naive("2022-12-25 03:10:00", "Asia/Seoul").as_deref(),
            Some("2022-12-24 18:10:00")
        );
    }

    #[test]
    fn converts_display_timestamptz_back_to_utc_offset() {
        assert_eq!(
            timestamp_display_to_utc("2022-12-25 03:10:00", "Asia/Seoul").as_deref(),
            Some("2022-12-24 18:10:00+00:00")
        );
    }
}
