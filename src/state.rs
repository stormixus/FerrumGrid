use std::collections::{HashMap, HashSet};

use chrono::{TimeZone, Utc};

use crate::connection_url::PostgresConnectionUrl;
use crate::prisma::ui::PrismaUIState;
use crate::storage::settings::AppSettings;
use crate::storage::vault::VaultSession;
use crate::types::{
    BackupFormat, BackupRecord, CellValue, ColumnInfo, ConnectionConfig, ConnectionId, EditorTab,
    FunctionInfo, IndexInfo, QueryResult, RoleInfo, RuleInfo, TableInfo, TriggerInfo,
};
use crate::ui::er_diagram::ERDiagramState;
use crate::ui::table_designer::TableDesignerState;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected { server_version: String },
}

#[derive(Debug)]
pub struct ConnectionState {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub databases: Vec<String>,
    pub schemas: Vec<String>,
    pub tables: HashMap<String, Vec<TableInfo>>,
    pub columns: HashMap<(String, String), Vec<ColumnInfo>>,
    pub indexes: HashMap<(String, String), Vec<IndexInfo>>,
    pub foreign_keys: HashMap<String, Vec<crate::ui::er_diagram::ForeignKey>>,
    pub rules: HashMap<(String, String), Vec<RuleInfo>>,
    pub triggers: HashMap<(String, String), Vec<TriggerInfo>>,
    pub functions: HashMap<String, Vec<FunctionInfo>>,
    pub roles: Vec<RoleInfo>,
    pub loading_databases: bool,
    pub loading_schemas: bool,
    pub loading_tables: HashSet<String>,
    pub loading_columns: HashSet<(String, String)>,
    pub loading_indexes: HashSet<(String, String)>,
    pub loading_foreign_keys: HashSet<String>,
    pub loading_rules: HashSet<(String, String)>,
    pub loading_triggers: HashSet<(String, String)>,
    pub loading_functions: HashSet<String>,
    pub loading_roles: bool,
}

impl ConnectionState {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            databases: Vec::new(),
            schemas: Vec::new(),
            tables: HashMap::new(),
            columns: HashMap::new(),
            indexes: HashMap::new(),
            foreign_keys: HashMap::new(),
            rules: HashMap::new(),
            triggers: HashMap::new(),
            functions: HashMap::new(),
            roles: Vec::new(),
            loading_databases: false,
            loading_schemas: false,
            loading_tables: HashSet::new(),
            loading_columns: HashSet::new(),
            loading_indexes: HashSet::new(),
            loading_foreign_keys: HashSet::new(),
            loading_rules: HashSet::new(),
            loading_triggers: HashSet::new(),
            loading_functions: HashSet::new(),
            loading_roles: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    Connection,
    Table,
    View,
    MaterializedView,
    Function,
    User,
    Query,
    Data,
    Backup,
    Automation,
    Model,
    BI,
}

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
    pub title: String,
    pub view: MainView,
    pub schema_filter: String,
    pub search: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource {
    pub conn_id: ConnectionId,
    pub schema: String,
    pub table: String,
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
            editing_cell: None,
        }
    }
}

impl WorkspaceTab {
    pub fn new(
        view: MainView,
        title: impl Into<String>,
        schema_filter: impl Into<String>,
        search: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            view,
            schema_filter: schema_filter.into(),
            search: search.into(),
        }
    }
}

pub struct AppState {
    pub connections: HashMap<ConnectionId, ConnectionState>,
    pub active_connection: Option<ConnectionId>,
    pub active_main_view: MainView,
    pub workspace_tabs: Vec<WorkspaceTab>,
    pub active_workspace_tab: usize,
    pub editor_tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub current_result: Option<QueryResult>,
    pub current_result_truncated: bool,
    pub data_edit: DataEditState,
    pub query_running: bool,
    pub last_error: Option<String>,
    pub show_connection_dialog: bool,
    pub show_about_dialog: bool,
    pub show_settings_dialog: bool,
    pub show_tree_panel: bool,
    pub show_result_panel: bool,
    pub show_info_panel: bool,
    pub active_settings_tab: usize,
    pub settings_draft: Option<AppSettings>,
    pub objects_schema_filter: String,
    pub objects_search: String,
    pub connection_dialog: ConnectionDialogState,
    pub saved_connections: Vec<ConnectionConfig>,
    pub vault: VaultUiState,
    pub default_row_limit: usize,
    pub status_message: String,
    pub er_diagram: ERDiagramState,
    pub table_designer: TableDesignerState,
    pub prisma_ui: PrismaUIState,
    pub backup_format: BackupFormat,
    pub backup_running: bool,
    pub backup_last_error: Option<String>,
    pub backup_history: Vec<BackupRecord>,
    pub data_timezone: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            active_connection: None,
            active_main_view: MainView::Connection,
            workspace_tabs: vec![WorkspaceTab::new(
                MainView::Connection,
                "Connection",
                "",
                "",
            )],
            active_workspace_tab: 0,
            editor_tabs: vec![EditorTab::new("Query 1")],
            active_tab: 0,
            current_result: None,
            current_result_truncated: false,
            data_edit: DataEditState::default(),
            query_running: false,
            last_error: None,
            show_connection_dialog: true,
            show_about_dialog: false,
            show_settings_dialog: false,
            show_tree_panel: true,
            show_result_panel: true,
            show_info_panel: true,
            active_settings_tab: 0,
            settings_draft: None,
            objects_schema_filter: String::new(),
            objects_search: String::new(),
            connection_dialog: ConnectionDialogState::default(),
            saved_connections: Vec::new(),
            vault: VaultUiState::setup_required(Vec::new()),
            default_row_limit: 1000,
            status_message: "Disconnected".to_string(),
            er_diagram: ERDiagramState::new(),
            table_designer: TableDesignerState::default(),
            prisma_ui: PrismaUIState::default(),
            backup_format: BackupFormat::Custom,
            backup_running: false,
            backup_last_error: None,
            backup_history: Vec::new(),
            data_timezone: "Asia/Seoul".to_string(),
        }
    }
}

impl AppState {
    pub fn begin_data_edit(&mut self, conn_id: ConnectionId, schema: &str, table: &str) {
        let source = DataSource {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
        };
        if self.data_edit.source.as_ref() != Some(&source) {
            self.data_edit = DataEditState {
                source: Some(source),
                cells: HashMap::new(),
                sort: Vec::new(),
                applying: false,
                page_limit: DEFAULT_DATA_PAGE_LIMIT,
                page_limit_input: DEFAULT_DATA_PAGE_LIMIT.to_string(),
                page_index: 0,
                editing_cell: None,
            };
        }
    }

    pub fn reset_data_edits_for_current_result(&mut self, conn_id: ConnectionId) {
        let Some(tab) = self.workspace_tabs.get(self.active_workspace_tab) else {
            return;
        };
        if tab.view != MainView::Data {
            return;
        }
        let Some(result) = self.current_result.as_ref() else {
            return;
        };
        let source = DataSource {
            conn_id,
            schema: tab.schema_filter.clone(),
            table: tab.search.clone(),
        };
        let sort = if self.data_edit.source.as_ref() == Some(&source) {
            self.data_edit.sort.clone()
        } else {
            Vec::new()
        };
        let (page_limit, page_limit_input, page_index) =
            if self.data_edit.source.as_ref() == Some(&source) {
                (
                    self.data_edit.page_limit,
                    self.data_edit.page_limit_input.clone(),
                    self.data_edit.page_index,
                )
            } else {
                (
                    DEFAULT_DATA_PAGE_LIMIT,
                    DEFAULT_DATA_PAGE_LIMIT.to_string(),
                    0,
                )
            };
        let mut cells = HashMap::new();
        for (row_idx, row) in result.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let type_name = result
                    .columns
                    .get(col_idx)
                    .map(|col| col.type_name.as_str())
                    .unwrap_or_default();
                cells.insert(
                    (row_idx, col_idx),
                    EditableCell::from_cell_for_type(cell, type_name, &self.data_timezone),
                );
            }
        }
        self.data_edit = DataEditState {
            source: Some(source),
            cells,
            sort,
            applying: false,
            page_limit,
            page_limit_input,
            page_index,
            editing_cell: None,
        };
    }

    pub fn active_data_source(&self) -> Option<DataSource> {
        let tab = self.workspace_tabs.get(self.active_workspace_tab)?;
        if tab.view != MainView::Data {
            return None;
        }
        if let Some(source) = self.data_edit.source.as_ref() {
            if source.schema == tab.schema_filter && source.table == tab.search {
                return Some(source.clone());
            }
        }
        Some(DataSource {
            conn_id: self.active_connection?,
            schema: tab.schema_filter.clone(),
            table: tab.search.clone(),
        })
    }

    pub fn data_columns_for_source(&self, source: &DataSource) -> Vec<ColumnInfo> {
        self.connections
            .get(&source.conn_id)
            .and_then(|conn| {
                conn.columns
                    .get(&(source.schema.clone(), source.table.clone()))
            })
            .cloned()
            .unwrap_or_default()
    }

    pub fn open_workspace_main_view(&mut self, view: MainView) {
        self.open_workspace_view(view, main_view_title(view), "", "");
    }

    pub fn open_workspace_view(
        &mut self,
        view: MainView,
        title: impl Into<String>,
        schema_filter: impl Into<String>,
        search: impl Into<String>,
    ) {
        let title = title.into();
        let schema_filter = schema_filter.into();
        let search = search.into();

        if let Some(index) = self.workspace_tabs.iter().position(|tab| {
            tab.view == view && tab.schema_filter == schema_filter && tab.search == search
        }) {
            self.workspace_tabs[index].title = title;
            self.active_workspace_tab = index;
        } else {
            self.workspace_tabs
                .push(WorkspaceTab::new(view, title, schema_filter, search));
            self.active_workspace_tab = self.workspace_tabs.len() - 1;
        }

        self.apply_active_workspace_tab();
    }

    pub fn activate_workspace_tab(&mut self, index: usize) {
        if index < self.workspace_tabs.len() {
            self.active_workspace_tab = index;
            self.apply_active_workspace_tab();
        }
    }

    pub fn close_workspace_tab(&mut self, index: usize) {
        if self.workspace_tabs.is_empty() || index >= self.workspace_tabs.len() {
            return;
        }

        self.workspace_tabs.remove(index);
        if self.workspace_tabs.is_empty() {
            self.workspace_tabs
                .push(WorkspaceTab::new(MainView::Query, "Query", "", ""));
        }
        self.active_workspace_tab = self
            .active_workspace_tab
            .min(self.workspace_tabs.len().saturating_sub(1));
        self.apply_active_workspace_tab();
    }

    fn apply_active_workspace_tab(&mut self) {
        let Some(tab) = self.workspace_tabs.get(self.active_workspace_tab) else {
            return;
        };

        self.active_main_view = tab.view;
        self.objects_schema_filter = tab.schema_filter.clone();
        self.objects_search = tab.search.clone();
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
        "SELECT {} FROM {}.{}{} LIMIT {}{}",
        select_list,
        quote_ident(&source.schema),
        quote_ident(&source.table),
        order_by,
        fetch_limit,
        offset_clause
    )
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
    if !is_timestamptz_type(type_name) {
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

pub fn is_timestamptz_type(type_name: &str) -> bool {
    matches!(
        type_name.to_ascii_lowercase().as_str(),
        "timestamptz" | "timestamp with time zone"
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

pub fn main_view_title(view: MainView) -> &'static str {
    match view {
        MainView::Connection => "Connection",
        MainView::Table => "Tables",
        MainView::View => "Views",
        MainView::MaterializedView => "Materialized Views",
        MainView::Function => "Functions",
        MainView::User => "Users",
        MainView::Query => "Query",
        MainView::Data => "Data",
        MainView::Backup => "Backup",
        MainView::Automation => "Automation",
        MainView::Model => "Model",
        MainView::BI => "BI",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VaultStatus {
    SetupRequired,
    Locked,
    Unlocked,
}

pub struct VaultUiState {
    pub status: VaultStatus,
    pub name: String,
    pub master_password: String,
    pub confirm_password: String,
    pub show_password: bool,
    pub error: Option<String>,
    pub legacy_connections: Vec<ConnectionConfig>,
    pub session: Option<VaultSession>,
}

impl VaultUiState {
    pub fn setup_required(legacy_connections: Vec<ConnectionConfig>) -> Self {
        Self {
            status: VaultStatus::SetupRequired,
            name: "Personal".to_string(),
            master_password: String::new(),
            confirm_password: String::new(),
            show_password: false,
            error: None,
            legacy_connections,
            session: None,
        }
    }

    pub fn locked(name: String) -> Self {
        Self {
            status: VaultStatus::Locked,
            name,
            master_password: String::new(),
            confirm_password: String::new(),
            show_password: false,
            error: None,
            legacy_connections: Vec::new(),
            session: None,
        }
    }

    pub fn unlocked(session: VaultSession) -> Self {
        Self {
            status: VaultStatus::Unlocked,
            name: session.name.clone(),
            master_password: String::new(),
            confirm_password: String::new(),
            show_password: false,
            error: None,
            legacy_connections: Vec::new(),
            session: Some(session),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.status == VaultStatus::Unlocked && self.session.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDialogState {
    pub draft_id: ConnectionId,
    pub display_name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub show_password: bool,
    pub use_tls: bool,
    pub testing: bool,
    pub test_result: Option<Result<String, String>>,
    pub editing_id: Option<ConnectionId>,
    pub clipboard_import_checked: bool,
    pub pending_clipboard_import: Option<PostgresConnectionUrl>,
}

impl Default for ConnectionDialogState {
    fn default() -> Self {
        Self {
            draft_id: ConnectionId::new(),
            display_name: String::new(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            show_password: false,
            use_tls: false,
            testing: false,
            test_result: None,
            editing_id: None,
            clipboard_import_checked: false,
            pending_clipboard_import: None,
        }
    }
}

impl ConnectionDialogState {
    pub fn to_config(&self) -> ConnectionConfig {
        ConnectionConfig {
            id: self.editing_id.unwrap_or(self.draft_id),
            display_name: if self.display_name.is_empty() {
                format!(
                    "{}@{}:{}/{}",
                    self.username, self.host, self.port, self.database
                )
            } else {
                self.display_name.clone()
            },
            host: self.host.clone(),
            port: self.port.parse().unwrap_or(5432),
            database: self.database.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            use_tls: self.use_tls,
            color_tag: None,
            ssh_tunnel: None,
        }
    }

    pub fn from_config(config: &ConnectionConfig) -> Self {
        Self {
            draft_id: config.id,
            display_name: config.display_name.clone(),
            host: config.host.clone(),
            port: config.port.to_string(),
            database: config.database.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            show_password: false,
            use_tls: config.use_tls,
            testing: false,
            test_result: None,
            editing_id: Some(config.id),
            clipboard_import_checked: true,
            pending_clipboard_import: None,
        }
    }
}
