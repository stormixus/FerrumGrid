use std::fmt;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::rand_core::{OsRng, RngCore};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use zeroize::{Zeroize, Zeroizing};

use crate::types::{ConnectionConfig, ConnectionId};

pub const VAULT_KIND: &str = "ferrumgrid.personal_vault";
const VAULT_VERSION: u32 = 1;
const DEFAULT_VAULT_NAME: &str = "Personal";
const KDF_ALGORITHM: &str = "argon2id";
const AEAD_ALGORITHM: &str = "chacha20poly1305";
const MEMORY_KIB: u32 = 64 * 1024;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 1;
const KEY_LEN: usize = 32;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

pub type DeviceKey = [u8; KEY_LEN];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VaultFile {
    pub version: u32,
    pub kind: String,
    pub vaults: Vec<VaultRecord>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VaultRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub kdf: KdfConfig,
    pub wrapped_key: EncryptedPayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_unlock: Option<EncryptedPayload>,
    #[serde(default)]
    pub connections: Vec<EncryptedItem>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KdfConfig {
    pub algorithm: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub salt: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedPayload {
    pub algorithm: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedItem {
    pub id: ConnectionId,
    pub kind: String,
    pub payload: EncryptedPayload,
}

#[derive(Clone)]
pub struct VaultSession {
    pub vault_id: uuid::Uuid,
    pub name: String,
    key: [u8; KEY_LEN],
}

impl fmt::Debug for VaultSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VaultSession")
            .field("vault_id", &self.vault_id)
            .field("name", &self.name)
            .field("key", &"<redacted>")
            .finish()
    }
}

impl Drop for VaultSession {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultError {
    EmptyPassword,
    MissingVault,
    UnsupportedFormat(String),
    InvalidEncoding,
    CryptoFailed,
    Json(String),
    Io(String),
    MissingDeviceUnlock,
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPassword => write!(f, "vault password is empty"),
            Self::MissingVault => write!(f, "personal vault was not found"),
            Self::UnsupportedFormat(reason) => write!(f, "unsupported vault format: {reason}"),
            Self::InvalidEncoding => write!(f, "vault data is not encoded correctly"),
            Self::CryptoFailed => write!(f, "vault password is incorrect or data is corrupted"),
            Self::Json(err) => write!(f, "failed to read vault JSON: {err}"),
            Self::Io(err) => write!(f, "failed to access vault storage: {err}"),
            Self::MissingDeviceUnlock => write!(f, "trusted-device unlock is not configured"),
        }
    }
}

impl std::error::Error for VaultError {}

pub fn create_file(
    master_password: &str,
    connections: &[ConnectionConfig],
) -> Result<(VaultFile, VaultSession), VaultError> {
    if master_password.is_empty() {
        return Err(VaultError::EmptyPassword);
    }

    let vault_id = uuid::Uuid::new_v4();
    let vault_key = random_key();
    let salt = random_salt();
    let kdf = KdfConfig {
        algorithm: KDF_ALGORITHM.to_string(),
        memory_kib: MEMORY_KIB,
        iterations: ITERATIONS,
        parallelism: PARALLELISM,
        salt: encode_hex(&salt),
    };

    let master_key = Zeroizing::new(derive_master_key(master_password, &kdf)?);
    let wrapped_key = seal(&master_key, &vault_key, b"ferrumgrid:vault-key:v1")?;

    let session = VaultSession {
        vault_id,
        name: DEFAULT_VAULT_NAME.to_string(),
        key: vault_key,
    };

    let record = VaultRecord {
        id: vault_id,
        name: DEFAULT_VAULT_NAME.to_string(),
        kdf,
        wrapped_key,
        device_unlock: None,
        connections: encrypt_connections(&session, connections)?,
    };

    Ok((
        VaultFile {
            version: VAULT_VERSION,
            kind: VAULT_KIND.to_string(),
            vaults: vec![record],
        },
        session,
    ))
}

pub fn unlock_file(
    file: &VaultFile,
    master_password: &str,
) -> Result<(Vec<ConnectionConfig>, VaultSession), VaultError> {
    if master_password.is_empty() {
        return Err(VaultError::EmptyPassword);
    }
    validate_file(file)?;

    let vault = file.vaults.first().ok_or(VaultError::MissingVault)?;
    let master_key = Zeroizing::new(derive_master_key(master_password, &vault.kdf)?);
    let vault_key_bytes = Zeroizing::new(open(
        &master_key,
        &vault.wrapped_key,
        b"ferrumgrid:vault-key:v1",
    )?);
    let vault_key: [u8; KEY_LEN] = vault_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| VaultError::CryptoFailed)?;

    let session = VaultSession {
        vault_id: vault.id,
        name: vault.name.clone(),
        key: vault_key,
    };
    let connections = decrypt_connections(&session, &vault.connections)?;

    Ok((connections, session))
}

pub fn unlock_file_with_device_key(
    file: &VaultFile,
    device_key: &DeviceKey,
) -> Result<(Vec<ConnectionConfig>, VaultSession), VaultError> {
    validate_file(file)?;

    let vault = file.vaults.first().ok_or(VaultError::MissingVault)?;
    let payload = vault
        .device_unlock
        .as_ref()
        .ok_or(VaultError::MissingDeviceUnlock)?;
    let vault_key_bytes = Zeroizing::new(open(
        device_key,
        payload,
        device_unlock_aad(vault.id).as_bytes(),
    )?);
    let vault_key: [u8; KEY_LEN] = vault_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| VaultError::CryptoFailed)?;

    let session = VaultSession {
        vault_id: vault.id,
        name: vault.name.clone(),
        key: vault_key,
    };
    let connections = decrypt_connections(&session, &vault.connections)?;

    Ok((connections, session))
}

pub fn enable_device_unlock(
    file: &VaultFile,
    session: &VaultSession,
    device_key: &DeviceKey,
) -> Result<VaultFile, VaultError> {
    validate_file(file)?;

    let mut next = file.clone();
    let vault = next
        .vaults
        .iter_mut()
        .find(|vault| vault.id == session.vault_id)
        .ok_or(VaultError::MissingVault)?;

    vault.device_unlock = Some(seal(
        device_key,
        &session.key,
        device_unlock_aad(session.vault_id).as_bytes(),
    )?);

    Ok(next)
}

pub fn save_file(
    file: &VaultFile,
    session: &VaultSession,
    connections: &[ConnectionConfig],
) -> Result<VaultFile, VaultError> {
    validate_file(file)?;

    let mut next = file.clone();
    let vault = next
        .vaults
        .iter_mut()
        .find(|vault| vault.id == session.vault_id)
        .ok_or(VaultError::MissingVault)?;

    vault.connections = encrypt_connections(session, connections)?;
    Ok(next)
}

pub fn vault_name(file: &VaultFile) -> Option<String> {
    file.vaults.first().map(|vault| vault.name.clone())
}

fn validate_file(file: &VaultFile) -> Result<(), VaultError> {
    if file.kind != VAULT_KIND {
        return Err(VaultError::UnsupportedFormat(file.kind.clone()));
    }
    if file.version != VAULT_VERSION {
        return Err(VaultError::UnsupportedFormat(format!(
            "version {}",
            file.version
        )));
    }
    Ok(())
}

fn encrypt_connections(
    session: &VaultSession,
    connections: &[ConnectionConfig],
) -> Result<Vec<EncryptedItem>, VaultError> {
    connections
        .iter()
        .map(|config| {
            let plaintext =
                serde_json::to_vec(config).map_err(|err| VaultError::Json(err.to_string()))?;
            Ok(EncryptedItem {
                id: config.id,
                kind: "postgres_connection".to_string(),
                payload: seal(
                    &session.key,
                    &plaintext,
                    connection_aad(config.id).as_bytes(),
                )?,
            })
        })
        .collect()
}

fn decrypt_connections(
    session: &VaultSession,
    items: &[EncryptedItem],
) -> Result<Vec<ConnectionConfig>, VaultError> {
    let mut connections = Vec::with_capacity(items.len());

    for item in items {
        if item.kind != "postgres_connection" {
            continue;
        }

        let plaintext = open(
            &session.key,
            &item.payload,
            connection_aad(item.id).as_bytes(),
        )?;
        let config = serde_json::from_slice::<ConnectionConfig>(&plaintext)
            .map_err(|err| VaultError::Json(err.to_string()))?;
        connections.push(config);
    }

    Ok(connections)
}

fn derive_master_key(password: &str, kdf: &KdfConfig) -> Result<[u8; KEY_LEN], VaultError> {
    if kdf.algorithm != KDF_ALGORITHM {
        return Err(VaultError::UnsupportedFormat(kdf.algorithm.clone()));
    }

    let salt = decode_hex(&kdf.salt)?;
    let params = Params::new(
        kdf.memory_kib,
        kdf.iterations,
        kdf.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|_| VaultError::UnsupportedFormat("invalid Argon2 parameters".to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|_| VaultError::CryptoFailed)?;
    Ok(key)
}

fn seal(
    key_bytes: &[u8; KEY_LEN],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<EncryptedPayload, VaultError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    let nonce_bytes = random_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| VaultError::CryptoFailed)?;

    Ok(EncryptedPayload {
        algorithm: AEAD_ALGORITHM.to_string(),
        nonce: encode_hex(&nonce_bytes),
        ciphertext: encode_hex(&ciphertext),
    })
}

fn open(
    key_bytes: &[u8; KEY_LEN],
    payload: &EncryptedPayload,
    aad: &[u8],
) -> Result<Vec<u8>, VaultError> {
    if payload.algorithm != AEAD_ALGORITHM {
        return Err(VaultError::UnsupportedFormat(payload.algorithm.clone()));
    }

    let nonce_bytes = decode_hex(&payload.nonce)?;
    let nonce_array: [u8; NONCE_LEN] = nonce_bytes
        .as_slice()
        .try_into()
        .map_err(|_| VaultError::InvalidEncoding)?;
    let ciphertext = decode_hex(&payload.ciphertext)?;

    let cipher = ChaCha20Poly1305::new(Key::from_slice(key_bytes));
    cipher
        .decrypt(
            Nonce::from_slice(&nonce_array),
            Payload {
                msg: &ciphertext,
                aad,
            },
        )
        .map_err(|_| VaultError::CryptoFailed)
}

fn connection_aad(id: ConnectionId) -> String {
    format!("ferrumgrid:connection:v1:{id}")
}

fn device_unlock_aad(id: uuid::Uuid) -> String {
    format!("ferrumgrid:device-unlock:v1:{id}")
}

pub fn generate_device_key() -> DeviceKey {
    random_key()
}

fn random_key() -> [u8; KEY_LEN] {
    let mut bytes = [0u8; KEY_LEN];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

fn random_salt() -> [u8; SALT_LEN] {
    let mut bytes = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

fn random_nonce() -> [u8; NONCE_LEN] {
    let mut bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, VaultError> {
    if !value.len().is_multiple_of(2) {
        return Err(VaultError::InvalidEncoding);
    }

    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_value(pair[0])?;
            let low = hex_value(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_value(byte: u8) -> Result<u8, VaultError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(VaultError::InvalidEncoding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_round_trips_connections() {
        let mut config = ConnectionConfig::default();
        config.display_name = "Local Postgres".to_string();
        config.password = "secret".to_string();

        let (file, _session) = create_file("correct horse battery staple", &[config.clone()])
            .expect("vault creation should succeed");
        let (connections, unlocked) =
            unlock_file(&file, "correct horse battery staple").expect("vault should unlock");

        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].password, "secret");

        let mut updated = connections;
        updated[0].password = "new secret".to_string();
        let next = save_file(&file, &unlocked, &updated).expect("save should encrypt");
        let (again, _) =
            unlock_file(&next, "correct horse battery staple").expect("updated vault should open");
        assert_eq!(again[0].password, "new secret");
    }

    #[test]
    fn vault_rejects_wrong_password() {
        let config = ConnectionConfig::default();
        let (file, _) = create_file("right password", &[config]).expect("vault should create");

        let result = unlock_file(&file, "wrong password");

        assert!(matches!(result, Err(VaultError::CryptoFailed)));
    }

    #[test]
    fn trusted_device_unlock_opens_vault_without_master_password() {
        let mut config = ConnectionConfig::default();
        config.password = "secret".to_string();
        let (file, session) =
            create_file("master password", &[config]).expect("vault should create");
        let device_key = generate_device_key();
        let trusted_file =
            enable_device_unlock(&file, &session, &device_key).expect("device unlock should save");

        let (connections, _) = unlock_file_with_device_key(&trusted_file, &device_key)
            .expect("device key should unlock");

        assert_eq!(connections[0].password, "secret");
    }

    #[test]
    fn trusted_device_unlock_rejects_wrong_device_key() {
        let config = ConnectionConfig::default();
        let (file, session) =
            create_file("master password", &[config]).expect("vault should create");
        let device_key = generate_device_key();
        let trusted_file =
            enable_device_unlock(&file, &session, &device_key).expect("device unlock should save");

        let result = unlock_file_with_device_key(&trusted_file, &generate_device_key());

        assert!(matches!(result, Err(VaultError::CryptoFailed)));
    }
}
