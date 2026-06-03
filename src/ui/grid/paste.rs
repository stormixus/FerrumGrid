//! Clipboard & export helpers (Plan v7 Phase 1.95c).

use crate::state::AppState;
use crate::types::{CellValue, QueryResult};

pub(crate) fn result_to_tsv(result: &QueryResult) -> String {
    let mut out = String::new();
    let headers: Vec<&str> = result.columns.iter().map(|c| c.name.as_str()).collect();
    out.push_str(&headers.join("\t"));
    out.push('\n');
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
        out.push_str(&cells.join("\t"));
        out.push('\n');
    }
    out
}

pub(crate) fn export_csv(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let task = rfd::AsyncFileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("query_result.csv")
        .save_file();

    let columns = result.columns.clone();
    let rows = result.rows.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let mut wtr = csv::Writer::from_writer(Vec::new());
                let headers: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
                let _ = wtr.write_record(&headers);
                for row in &rows {
                    let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
                    let _ = wtr.write_record(&cells);
                }
                if let Ok(data) = wtr.into_inner() {
                    let _ = handle.write(&data).await;
                }
            }
        });
    });
}

pub(crate) fn export_json(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let task = rfd::AsyncFileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("query_result.json")
        .save_file();

    let columns = result.columns.clone();
    let rows = result.rows.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let headers: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
                let json_rows: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|row| {
                        let obj: serde_json::Map<String, serde_json::Value> = headers
                            .iter()
                            .zip(row.iter())
                            .map(|(col, cell)| (col.to_string(), cell_to_json(cell)))
                            .collect();
                        serde_json::Value::Object(obj)
                    })
                    .collect();
                if let Ok(data) = serde_json::to_string_pretty(&json_rows) {
                    let _ = handle.write(data.as_bytes()).await;
                }
            }
        });
    });
}

/// 현재 결과를 `INSERT INTO … VALUES …;` 구문 문자열로 직렬화.
/// 대상 테이블명은 활성 data source 에서, 없으면 `"table_name"` placeholder.
fn build_sql_inserts(result: &QueryResult, table_name: &str) -> String {
    let col_list: String = result
        .columns
        .iter()
        .map(|c| format!("\"{}\"", c.name))
        .collect::<Vec<_>>()
        .join(", ");
    let mut out = String::new();
    for row in &result.rows {
        let vals: Vec<String> = row.iter().map(cell_to_sql_literal).collect();
        out.push_str(&format!(
            "INSERT INTO {} ({}) VALUES ({});\n",
            table_name,
            col_list,
            vals.join(", ")
        ));
    }
    out
}

fn result_table_name(state: &AppState) -> String {
    state
        .data_edit
        .source
        .as_ref()
        .map(|s| format!("\"{}\".\"{}\"", s.schema, s.table))
        .unwrap_or_else(|| "\"table_name\"".to_string())
}

pub(crate) fn export_sql_insert(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let body = build_sql_inserts(result, &result_table_name(state));

    let task = rfd::AsyncFileDialog::new()
        .add_filter("SQL", &["sql"])
        .set_file_name("query_result.sql")
        .save_file();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let _ = handle.write(body.as_bytes()).await;
            }
        });
    });
}

/// 클립보드용 INSERT 문자열 (파일 저장 없이). None = 결과 없음.
pub(crate) fn result_to_sql_insert(state: &AppState) -> Option<String> {
    let result = state.current_result.as_ref()?;
    Some(build_sql_inserts(result, &result_table_name(state)))
}

/// 결과를 GitHub-flavored Markdown 표로 직렬화 (클립보드용).
pub(crate) fn result_to_markdown(result: &QueryResult) -> String {
    fn esc(s: &str) -> String {
        s.replace('|', "\\|").replace('\n', "<br>")
    }
    let headers: Vec<String> = result.columns.iter().map(|c| esc(&c.name)).collect();
    let mut out = String::new();
    out.push_str("| ");
    out.push_str(&headers.join(" | "));
    out.push_str(" |\n|");
    for _ in &headers {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in &result.rows {
        let cells: Vec<String> = row.iter().map(|c| esc(&c.to_string())).collect();
        out.push_str("| ");
        out.push_str(&cells.join(" | "));
        out.push_str(" |\n");
    }
    out
}

/// 결과를 .xlsx 파일로 내보내기 (타입 보존: 숫자/불리언은 네이티브 셀 타입).
pub(crate) fn export_xlsx(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };
    let columns: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
    let rows = result.rows.clone();

    let task = rfd::AsyncFileDialog::new()
        .add_filter("Excel", &["xlsx"])
        .set_file_name("query_result.xlsx")
        .save_file();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let mut workbook = rust_xlsxwriter::Workbook::new();
                let sheet = workbook.add_worksheet();
                let header_fmt = rust_xlsxwriter::Format::new().set_bold();
                for (c, name) in columns.iter().enumerate() {
                    let _ = sheet.write_string_with_format(0, c as u16, name, &header_fmt);
                }
                for (r, row) in rows.iter().enumerate() {
                    let excel_row = (r + 1) as u32;
                    for (c, cell) in row.iter().enumerate() {
                        let _ = write_cell_xlsx(sheet, excel_row, c as u16, cell);
                    }
                }
                if let Ok(buf) = workbook.save_to_buffer() {
                    let _ = handle.write(&buf).await;
                }
            }
        });
    });
}

fn write_cell_xlsx(
    sheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    cell: &CellValue,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    match cell {
        CellValue::Null => Ok(()),
        CellValue::Bool(v) => sheet.write_boolean(row, col, *v).map(|_| ()),
        CellValue::Int(v) => sheet.write_number(row, col, *v as f64).map(|_| ()),
        CellValue::Float(v) => sheet.write_number(row, col, *v).map(|_| ()),
        other => sheet.write_string(row, col, other.to_string()).map(|_| ()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnMeta;

    fn sample() -> QueryResult {
        QueryResult {
            columns: vec![
                ColumnMeta { name: "id".into(), type_name: "int4".into() },
                ColumnMeta { name: "note".into(), type_name: "text".into() },
            ],
            rows: vec![
                vec![CellValue::Int(1), CellValue::Text("a | b".into())],
                vec![CellValue::Null, CellValue::Text("two\nlines".into())],
            ],
            execution_time_ms: 0,
        }
    }

    #[test]
    fn markdown_table_has_header_separator_and_escaped_cells() {
        let md = result_to_markdown(&sample());
        let lines: Vec<&str> = md.lines().collect();
        assert_eq!(lines[0], "| id | note |");
        assert_eq!(lines[1], "| --- | --- |");
        // pipe inside a cell is escaped so the table stays valid
        assert!(lines[2].contains("a \\| b"), "got: {}", lines[2]);
        // newline within a cell becomes <br>
        assert!(lines[3].contains("two<br>lines"), "got: {}", lines[3]);
    }

    #[test]
    fn tsv_round_trips_headers_and_rows() {
        let tsv = result_to_tsv(&sample());
        assert!(tsv.starts_with("id\tnote\n"));
        assert!(tsv.contains("1\ta | b"));
    }
}

fn cell_to_json(cell: &CellValue) -> serde_json::Value {
    match cell {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Bool(v) => serde_json::Value::Bool(*v),
        CellValue::Int(v) => serde_json::json!(*v),
        CellValue::Float(v) => serde_json::json!(*v),
        CellValue::Text(v) | CellValue::Timestamp(v) | CellValue::Unknown(v) => {
            serde_json::Value::String(v.clone())
        }
        CellValue::Json(v) => v.clone(),
        CellValue::Uuid(v) => serde_json::Value::String(v.to_string()),
        CellValue::Bytes(v) => {
            serde_json::Value::String(format!("\\x{}", v.iter().map(|b| format!("{b:02x}")).collect::<String>()))
        }
    }
}

fn cell_to_sql_literal(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => "NULL".to_string(),
        CellValue::Bool(v) => v.to_string(),
        CellValue::Int(v) => v.to_string(),
        CellValue::Float(v) => v.to_string(),
        CellValue::Text(v) | CellValue::Timestamp(v) | CellValue::Unknown(v) => {
            format!("'{}'", v.replace('\'', "''"))
        }
        CellValue::Json(v) => format!("'{}'", v.to_string().replace('\'', "''")),
        CellValue::Uuid(v) => format!("'{v}'"),
        CellValue::Bytes(v) => {
            format!("'\\x{}'", v.iter().map(|b| format!("{b:02x}")).collect::<String>())
        }
    }
}
