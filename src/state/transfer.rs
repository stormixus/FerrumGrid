use crate::types::ConnectionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TransferTableStatus {
    Pending,
    InProgress,
    Done,
    Error,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IfExists {
    Drop,
    Skip,
    Truncate,
}

#[derive(Debug, Clone)]
pub struct TransferOptions {
    pub include_data: bool,
    pub if_exists: IfExists,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            include_data: true,
            if_exists: IfExists::Drop,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TransferTableEntry {
    pub schema: String,
    pub name: String,
    pub selected: bool,
    pub row_count: Option<u64>,
    pub dependencies: Vec<String>,
    pub status: TransferTableStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TransferProgress {
    pub current_table: String,
    pub current_table_index: usize,
    pub total_tables: usize,
    pub rows_transferred: u64,
    pub rows_total: Option<u64>,
    pub bytes_transferred: u64,
}

#[derive(Debug, Clone)]
pub struct TransferResult {
    pub tables_success: usize,
    pub tables_failed: usize,
    pub tables_skipped: usize,
    pub total_rows: u64,
    pub duration_ms: u128,
    pub errors: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub source_config: crate::types::ConnectionConfig,
    pub target_config: crate::types::ConnectionConfig,
    pub source_schema: String,
    pub target_schema: String,
    pub tables: Vec<String>,
    pub options: TransferOptions,
}

#[derive(Debug, Clone)]
pub struct ClipboardTables {
    pub conn_id: ConnectionId,
    pub schema: String,
    pub tables: Vec<String>,
}

#[allow(dead_code)]
pub struct TransferState {
    pub show: bool,
    pub source_conn: Option<ConnectionId>,
    pub target_conn: Option<ConnectionId>,
    pub source_schema: String,
    pub target_schema: String,
    pub tables: Vec<TransferTableEntry>,
    pub options: TransferOptions,
    pub progress: Option<TransferProgress>,
    pub result: Option<TransferResult>,
    pub loading_deps: bool,
    pub error: Option<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for TransferState {
    fn default() -> Self {
        Self {
            show: false,
            source_conn: None,
            target_conn: None,
            source_schema: String::new(),
            target_schema: String::new(),
            tables: Vec::new(),
            options: TransferOptions::default(),
            progress: None,
            result: None,
            loading_deps: false,
            error: None,
        }
    }
}

impl TransferState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn is_transferring(&self) -> bool {
        self.progress.is_some() && self.result.is_none()
    }

    #[allow(dead_code)]
    pub fn selected_tables(&self) -> Vec<&TransferTableEntry> {
        self.tables.iter().filter(|t| t.selected).collect()
    }

    pub fn open_from_clipboard(
        &mut self,
        clipboard: &ClipboardTables,
        target_conn: ConnectionId,
        target_schema: String,
    ) {
        self.reset();
        self.show = true;
        self.source_conn = Some(clipboard.conn_id);
        self.source_schema = clipboard.schema.clone();
        self.target_conn = Some(target_conn);
        self.target_schema = target_schema;
        self.tables = clipboard
            .tables
            .iter()
            .map(|name| TransferTableEntry {
                schema: clipboard.schema.clone(),
                name: name.clone(),
                selected: true,
                row_count: None,
                dependencies: Vec::new(),
                status: TransferTableStatus::Pending,
                error: None,
            })
            .collect();
    }
}
