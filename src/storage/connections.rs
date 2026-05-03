use std::path::{Path, PathBuf};

use crate::storage::vault::{self, VaultError, VaultFile, VaultSession};
use crate::types::ConnectionConfig;

fn config_dir() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine config directory");
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.to_path_buf()
}

fn connections_file() -> PathBuf {
    config_dir().join("connections.json")
}

fn device_key_file() -> PathBuf {
    config_dir().join("device.key")
}

#[derive(Debug, Clone)]
pub enum ConnectionStorageState {
    Empty,
    Legacy(Vec<ConnectionConfig>),
    VaultUnlocked {
        connections: Vec<ConnectionConfig>,
        session: VaultSession,
    },
    VaultLocked {
        name: String,
    },
    Corrupt(String),
}

pub fn load_storage_state() -> ConnectionStorageState {
    let path = connections_file();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return ConnectionStorageState::Empty;
    };

    if let Ok(file) = serde_json::from_str::<VaultFile>(&data) {
        if file.kind == vault::VAULT_KIND {
            if let Ok(Some((connections, session))) = auto_unlock_file(&file) {
                return ConnectionStorageState::VaultUnlocked {
                    connections,
                    session,
                };
            }
            return ConnectionStorageState::VaultLocked {
                name: vault::vault_name(&file).unwrap_or_else(|| "Personal".to_string()),
            };
        }
    }

    parse_legacy_storage_state(&data)
}

pub fn setup_vault(
    master_password: &str,
    connections: &[ConnectionConfig],
) -> Result<VaultSession, VaultError> {
    let (file, session) = vault::create_file(master_password, connections)?;
    let device_key = ensure_device_key()?;
    let file = vault::enable_device_unlock(&file, &session, &device_key)?;
    write_vault_file(&file)?;
    Ok(session)
}

pub fn unlock_vault(
    master_password: &str,
) -> Result<(Vec<ConnectionConfig>, VaultSession), VaultError> {
    let file = read_vault_file()?;
    let (connections, session) = vault::unlock_file(&file, master_password)?;
    let device_key = ensure_device_key()?;
    let file = vault::enable_device_unlock(&file, &session, &device_key)?;
    write_vault_file(&file)?;
    Ok((connections, session))
}

pub fn save_connections(
    connections: &[ConnectionConfig],
    session: &VaultSession,
) -> Result<(), VaultError> {
    let file = read_vault_file()?;
    let next = vault::save_file(&file, session, connections)?;
    let device_key = ensure_device_key()?;
    let next = vault::enable_device_unlock(&next, session, &device_key)?;
    write_vault_file(&next)
}

#[cfg(test)]
fn parse_storage_state(data: &str) -> ConnectionStorageState {
    if data.trim().is_empty() {
        return ConnectionStorageState::Empty;
    }

    if let Ok(file) = serde_json::from_str::<VaultFile>(data) {
        if file.kind == vault::VAULT_KIND {
            return ConnectionStorageState::VaultLocked {
                name: vault::vault_name(&file).unwrap_or_else(|| "Personal".to_string()),
            };
        }
    }

    parse_legacy_storage_state(data)
}

fn parse_legacy_storage_state(data: &str) -> ConnectionStorageState {
    match serde_json::from_str::<Vec<ConnectionConfig>>(data) {
        Ok(connections) => ConnectionStorageState::Legacy(connections),
        Err(err) => ConnectionStorageState::Corrupt(err.to_string()),
    }
}

fn auto_unlock_file(
    file: &VaultFile,
) -> Result<Option<(Vec<ConnectionConfig>, VaultSession)>, VaultError> {
    let Some(device_key) = read_device_key()? else {
        return Ok(None);
    };

    match vault::unlock_file_with_device_key(file, &device_key) {
        Ok(unlocked) => Ok(Some(unlocked)),
        Err(_) => Ok(None),
    }
}

fn read_vault_file() -> Result<VaultFile, VaultError> {
    let path = connections_file();
    let data = std::fs::read_to_string(&path).map_err(|err| VaultError::Io(err.to_string()))?;
    serde_json::from_str::<VaultFile>(&data).map_err(|err| VaultError::Json(err.to_string()))
}

fn write_vault_file(file: &VaultFile) -> Result<(), VaultError> {
    let path = connections_file();
    let data =
        serde_json::to_string_pretty(file).map_err(|err| VaultError::Json(err.to_string()))?;
    std::fs::write(&path, data).map_err(|err| VaultError::Io(err.to_string()))?;
    set_private_file_permissions(&path);
    Ok(())
}

fn ensure_device_key() -> Result<vault::DeviceKey, VaultError> {
    if let Some(key) = read_device_key()? {
        return Ok(key);
    }

    let key = vault::generate_device_key();
    let path = device_key_file();
    std::fs::write(&path, key).map_err(|err| VaultError::Io(err.to_string()))?;
    set_private_file_permissions(&path);
    Ok(key)
}

fn read_device_key() -> Result<Option<vault::DeviceKey>, VaultError> {
    let path = device_key_file();
    let data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(VaultError::Io(err.to_string())),
    };

    if data.len() != std::mem::size_of::<vault::DeviceKey>() {
        return Ok(None);
    }

    let key = data
        .as_slice()
        .try_into()
        .map_err(|_| VaultError::InvalidEncoding)?;
    Ok(Some(key))
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = std::fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        std::fs::set_permissions(path, permissions).ok();
    }
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_legacy_connection_array() {
        let config = ConnectionConfig::default();
        let data = serde_json::to_string(&vec![config]).expect("legacy should serialize");

        let state = parse_storage_state(&data);

        assert!(matches!(state, ConnectionStorageState::Legacy(items) if items.len() == 1));
    }

    #[test]
    fn detects_vault_file() {
        let (file, _) = vault::create_file("password", &[]).expect("vault should create");
        let data = serde_json::to_string(&file).expect("vault should serialize");

        let state = parse_storage_state(&data);

        assert!(
            matches!(state, ConnectionStorageState::VaultLocked { name } if name == "Personal")
        );
    }
}
