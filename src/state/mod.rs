//! Application state.
//!
//! Plan v7 Phase 1.95c1 — state.rs (1025줄) 를 폴더 구조로 변환. sub-modules
//! 는 현재 빈 placeholder. 실제 함수 cut-over 는 후속 1.95c sub-stories 에서
//! 진행 (DataEditState/EditableCell/DataSource/DataFilter → data_edit,
//! TableDesignerState 통합 검토 → designer, EditorTab + Phase 3 dangling tx
//! 상태 → query).

mod data_edit;
mod designer;
mod query;

// Plan v7 Phase 1.95c2 — data_edit cut-over. 외부 callers 가 `crate::state::*`
// 로 접근하던 항목을 그대로 노출하기 위해 `pub use` 재출. mod.rs 내부에서는
// AppState impl 가 이 re-export 를 통해 동일 path 로 접근.
pub use data_edit::{
    build_data_select_sql_with_columns, cell_edit_text_for_type, data_filter_from_cell,
    data_filter_from_text, data_timezone_label, data_timezone_offset_seconds,
    data_timezone_options, is_timestamp_without_timezone_type, is_timestamptz_type,
    timestamp_display_to_utc, timestamp_display_to_utc_naive, DataEditState,
    DataFilter, DataSortClause, DataSortDirection, DataSource, EditableCell,
    DEFAULT_DATA_PAGE_LIMIT, MAX_DATA_PAGE_LIMIT,
};

use std::collections::{HashMap, HashSet};

use crate::connection_url::PostgresConnectionUrl;
use crate::prisma::ui::PrismaUIState;
use crate::storage::settings::AppSettings;
use crate::storage::vault::VaultSession;
use crate::types::{
    BackupFormat, BackupRecord, ColumnInfo, ConnectionConfig, ConnectionId, EditorTab,
    FunctionInfo, IndexInfo, QueryResult, RoleInfo, RuleInfo, TableInfo, TriggerInfo,
};
use crate::ui::diagnostics_panel::DiagnosticsPanel;
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
    pub data_filter: Option<DataFilter>,
}

// Plan v7 Phase 1.95c2 — DataSource / DataFilter / DataSortDirection /
// DataSortClause / EditableCell / DataEditState 와 DEFAULT_DATA_PAGE_LIMIT /
// MAX_DATA_PAGE_LIMIT 가 src/state/data_edit.rs 로 cut-over.
// 외부 callers 는 mod.rs 의 pub use re-export 를 통해 동일 path 사용.

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
            data_filter: None,
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
    pub schema_context_menu: Option<SchemaContextMenuState>,
    pub dragging_saved_connection: Option<ConnectionId>,
    pub diagnostics_panel: DiagnosticsPanel,
    /// Plan v7 Phase 3b — Query 탭 명시 BEGIN 활성 여부.
    pub explicit_tx_active: bool,
    /// Plan v7 Phase 3b — 명시 BEGIN 시작 시각 (dangling tx 경과 측정).
    pub explicit_tx_started: Option<std::time::Instant>,
    /// Plan v7 Phase 3b — 30s warn toast 이미 표시했는지.
    pub explicit_tx_warned: bool,
}

#[derive(Clone)]
pub struct SchemaContextMenuState {
    pub conn_id: ConnectionId,
    pub schema: String,
    pub pos: [f32; 2],
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
            schema_context_menu: None,
            dragging_saved_connection: None,
            diagnostics_panel: DiagnosticsPanel::default(),
            explicit_tx_active: false,
            explicit_tx_started: None,
            explicit_tx_warned: false,
        }
    }
}

impl AppState {
    pub fn begin_data_edit(&mut self, conn_id: ConnectionId, schema: &str, table: &str) {
        self.begin_data_edit_with_filter(conn_id, schema, table, None);
    }

    pub fn begin_data_edit_with_filter(
        &mut self,
        conn_id: ConnectionId,
        schema: &str,
        table: &str,
        filter: Option<DataFilter>,
    ) {
        let source = DataSource {
            conn_id,
            schema: schema.to_string(),
            table: table.to_string(),
            filter,
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
                page_index_input: "1".to_string(),
                selected_cell: None,
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
            filter: tab.data_filter.clone(),
        };
        let sort = if self.data_edit.source.as_ref() == Some(&source) {
            self.data_edit.sort.clone()
        } else {
            Vec::new()
        };
        let (page_limit, page_limit_input, page_index, page_index_input) =
            if self.data_edit.source.as_ref() == Some(&source) {
                (
                    self.data_edit.page_limit,
                    self.data_edit.page_limit_input.clone(),
                    self.data_edit.page_index,
                    self.data_edit.page_index_input.clone(),
                )
            } else {
                (
                    DEFAULT_DATA_PAGE_LIMIT,
                    DEFAULT_DATA_PAGE_LIMIT.to_string(),
                    0,
                    "1".to_string(),
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
            page_index_input,
            selected_cell: None,
            editing_cell: None,
        };
    }

    pub fn active_data_source(&self) -> Option<DataSource> {
        let tab = self.workspace_tabs.get(self.active_workspace_tab)?;
        if tab.view != MainView::Data {
            return None;
        }
        if let Some(source) = self.data_edit.source.as_ref() {
            if source.schema == tab.schema_filter
                && source.table == tab.search
                && source.filter == tab.data_filter
            {
                return Some(source.clone());
            }
        }
        Some(DataSource {
            conn_id: self.active_connection?,
            schema: tab.schema_filter.clone(),
            table: tab.search.clone(),
            filter: tab.data_filter.clone(),
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
        self.open_workspace_view_with_filter(view, title, schema_filter, search, None);
    }

    pub fn open_data_workspace_view(
        &mut self,
        title: impl Into<String>,
        schema_filter: impl Into<String>,
        search: impl Into<String>,
        data_filter: Option<DataFilter>,
    ) {
        self.open_workspace_view_with_filter(
            MainView::Data,
            title,
            schema_filter,
            search,
            data_filter,
        );
    }

    fn open_workspace_view_with_filter(
        &mut self,
        view: MainView,
        title: impl Into<String>,
        schema_filter: impl Into<String>,
        search: impl Into<String>,
        data_filter: Option<DataFilter>,
    ) {
        let title = title.into();
        let schema_filter = schema_filter.into();
        let search = search.into();

        if let Some(index) = self.workspace_tabs.iter().position(|tab| {
            tab.view == view
                && tab.schema_filter == schema_filter
                && tab.search == search
                && tab.data_filter == data_filter
        }) {
            self.workspace_tabs[index].title = title;
            self.active_workspace_tab = index;
        } else {
            let mut tab = WorkspaceTab::new(view, title, schema_filter, search);
            tab.data_filter = data_filter;
            self.workspace_tabs.push(tab);
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
        if tab.view == MainView::Model && !tab.schema_filter.is_empty() {
            self.er_diagram.selected_schema = tab.schema_filter.clone();
            self.er_diagram.show_diagram = true;
        }
    }
}

// Plan v7 Phase 1.95c2 — build_data_select_sql_with_columns / data_filter_*
// / cell_to_sql_literal / text_to_sql_literal / cell_edit_text /
// cell_edit_text_for_type / timestamp_display_to_utc(_naive) / is_timestamp*
// / data_timezone_options/label/offset_seconds / parse_offset_seconds /
// parse_utc_datetime / parse_display_datetime / hex_encode / quote_ident /
// quote_literal / tests mod 모두 src/state/data_edit.rs 로 cut-over.

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
    pub clipboard_import_enabled: bool,
    pub last_clipboard_scan: Option<std::time::Instant>,
    pub last_clipboard_text: Option<String>,
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
            clipboard_import_enabled: true,
            last_clipboard_scan: None,
            last_clipboard_text: None,
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
            clipboard_import_enabled: false,
            last_clipboard_scan: None,
            last_clipboard_text: None,
            pending_clipboard_import: None,
        }
    }
}
