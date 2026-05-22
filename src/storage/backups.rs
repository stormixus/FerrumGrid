use std::path::PathBuf;
use crate::types::BackupRecord;

fn config_dir() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine config directory");
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.to_path_buf()
}

fn backups_file() -> PathBuf {
    config_dir().join("backups.json")
}

pub fn load_backups() -> Vec<BackupRecord> {
    let path = backups_file();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_backups(backups: &[BackupRecord]) {
    let path = backups_file();
    if let Ok(data) = serde_json::to_string_pretty(backups) {
        std::fs::write(path, data).ok();
    }
}
