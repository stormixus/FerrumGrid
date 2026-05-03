use std::{fmt, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConnectionId(pub uuid::Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionConfig {
    pub id: ConnectionId,
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(default)]
    pub password: String,
    pub use_tls: bool,
    pub color_tag: Option<String>,
    pub ssh_tunnel: Option<SshTunnelConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupFormat {
    Custom,
    Plain,
}

impl BackupFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Custom => "Custom archive",
            Self::Plain => "Plain SQL",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Custom => "dump",
            Self::Plain => "sql",
        }
    }

    pub fn pg_dump_format(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Plain => "plain",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupRequest {
    pub conn_id: ConnectionId,
    pub config: ConnectionConfig,
    pub output_dir: PathBuf,
    pub schema: Option<String>,
    pub format: BackupFormat,
}

#[derive(Debug, Clone)]
pub struct BackupRecord {
    pub conn_id: ConnectionId,
    pub connection_name: String,
    pub database: String,
    pub schema: Option<String>,
    pub format: BackupFormat,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub duration_ms: u128,
    pub completed_at: String,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            id: ConnectionId::new(),
            display_name: String::new(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            use_tls: false,
            color_tag: None,
            ssh_tunnel: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<CellValue>>,
    pub execution_time_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ColumnMeta {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Json(serde_json::Value),
    Timestamp(String),
    Uuid(uuid::Uuid),
    Bytes(Vec<u8>),
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct DataCellEdit {
    pub schema: String,
    pub table: String,
    pub column: String,
    pub column_type: String,
    pub pk: Vec<DataKeyValue>,
    pub value: DataEditValue,
}

#[derive(Debug, Clone)]
pub struct DataKeyValue {
    pub column: String,
    pub column_type: String,
    pub value: CellValue,
}

#[derive(Debug, Clone)]
pub enum DataEditValue {
    Null,
    Text(String),
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Null => write!(f, "NULL"),
            CellValue::Bool(v) => write!(f, "{v}"),
            CellValue::Int(v) => write!(f, "{v}"),
            CellValue::Float(v) => write!(f, "{v}"),
            CellValue::Text(v) => write!(f, "{v}"),
            CellValue::Json(v) => write!(f, "{v}"),
            CellValue::Timestamp(v) => write!(f, "{v}"),
            CellValue::Uuid(v) => write!(f, "{v}"),
            CellValue::Bytes(v) => write!(f, "\\x{}", hex_encode(v)),
            CellValue::Unknown(v) => write!(f, "{v}"),
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub table_type: String,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub enum_values: Vec<String>,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
}

#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub name: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct TriggerInfo {
    pub name: String,
    pub definition: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub schema: String,
    pub name: String,
    pub arguments: String,
    pub return_type: String,
    pub kind: String,
    pub language: String,
}

#[derive(Debug, Clone)]
pub struct RoleInfo {
    pub name: String,
    pub can_login: bool,
    pub is_superuser: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub can_replicate: bool,
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub connection_id: Option<ConnectionId>,
}

impl EditorTab {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: title.into(),
            content: String::new(),
            connection_id: None,
        }
    }
}
