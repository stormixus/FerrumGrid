use std::path::PathBuf;

use crate::types::ConnectionId;

const MAX_HISTORY: usize = 1000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub query: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u128,
    pub row_count: usize,
    pub conn_id: ConnectionId,
    pub conn_name: String,
    pub truncated: bool,
}

fn history_file() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine data directory");
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir).ok();
    data_dir.join("query_history.json")
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_history(entries: &[HistoryEntry]) {
    let path = history_file();
    if let Ok(data) = serde_json::to_string_pretty(entries) {
        std::fs::write(path, data).ok();
    }
}

pub fn add_entry(entries: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    entries.push(entry);
    truncate_history(entries);
    save_history(entries);
}

fn truncate_history(entries: &mut Vec<HistoryEntry>) {
    if entries.len() > MAX_HISTORY {
        entries.drain(0..entries.len() - MAX_HISTORY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_eviction() {
        let mut entries = Vec::new();
        for i in 0..1005 {
            entries.push(HistoryEntry {
                query: format!("SELECT {i}"),
                timestamp: chrono::Utc::now(),
                duration_ms: 1,
                row_count: 1,
                conn_id: ConnectionId::new(),
                conn_name: "test".to_string(),
                truncated: false,
            });
            truncate_history(&mut entries);
        }
        assert_eq!(entries.len(), MAX_HISTORY);
        assert!(entries[0].query.contains("5"));
    }
}
