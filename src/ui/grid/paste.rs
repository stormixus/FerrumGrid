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

pub(crate) fn export_sql_insert(state: &AppState) {
    let result = match &state.current_result {
        Some(r) => r,
        None => return,
    };

    let table_name = state
        .data_edit
        .source
        .as_ref()
        .map(|s| format!("\"{}\".\"{}\"", s.schema, s.table))
        .unwrap_or_else(|| "\"table_name\"".to_string());

    let task = rfd::AsyncFileDialog::new()
        .add_filter("SQL", &["sql"])
        .set_file_name("query_result.sql")
        .save_file();

    let columns = result.columns.clone();
    let rows = result.rows.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Some(handle) = task.await {
                let col_list: String = columns
                    .iter()
                    .map(|c| format!("\"{}\"", c.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut out = String::new();
                for row in &rows {
                    let vals: Vec<String> = row.iter().map(cell_to_sql_literal).collect();
                    out.push_str(&format!(
                        "INSERT INTO {} ({}) VALUES ({});\n",
                        table_name,
                        col_list,
                        vals.join(", ")
                    ));
                }
                let _ = handle.write(out.as_bytes()).await;
            }
        });
    });
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
