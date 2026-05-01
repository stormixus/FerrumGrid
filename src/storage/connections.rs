use std::path::PathBuf;

use crate::types::ConnectionConfig;

fn connections_file() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine config directory");
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.join("connections.json")
}

pub fn load_connections() -> Vec<ConnectionConfig> {
    let path = connections_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_connections(connections: &[ConnectionConfig]) {
    let path = connections_file();
    if let Ok(data) = serde_json::to_string_pretty(connections) {
        std::fs::write(path, data).ok();
    }
}

pub fn store_password(conn_id: &crate::types::ConnectionId, password: &str) {
    let service = format!("ferrumgrid:{}", conn_id.0);
    if let Ok(entry) = keyring::Entry::new(&service, "password") {
        entry.set_password(password).ok();
    }
}

pub fn load_password(conn_id: &crate::types::ConnectionId) -> Option<String> {
    let service = format!("ferrumgrid:{}", conn_id.0);
    if let Ok(entry) = keyring::Entry::new(&service, "password") {
        entry.get_password().ok()
    } else {
        None
    }
}

pub fn delete_password(conn_id: &crate::types::ConnectionId) {
    let service = format!("ferrumgrid:{}", conn_id.0);
    if let Ok(entry) = keyring::Entry::new(&service, "password") {
        entry.delete_credential().ok();
    }
}
