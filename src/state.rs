use std::collections::{HashMap, HashSet};

use crate::types::{
    ColumnInfo, ConnectionConfig, ConnectionId, EditorTab, IndexInfo, QueryResult,
    TableInfo,
};
use crate::ui::theme::ThemeMode;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected { server_version: String },
}

#[derive(Debug)]
pub struct ConnectionState {
    pub id: ConnectionId,
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub schemas: Vec<String>,
    pub tables: HashMap<String, Vec<TableInfo>>,
    pub columns: HashMap<(String, String), Vec<ColumnInfo>>,
    pub indexes: HashMap<(String, String), Vec<IndexInfo>>,
    pub expanded_nodes: HashSet<String>,
    pub loading_schemas: bool,
    pub loading_tables: HashSet<String>,
    pub loading_columns: HashSet<(String, String)>,
}

impl ConnectionState {
    pub fn new(config: ConnectionConfig) -> Self {
        let id = config.id;
        Self {
            id,
            config,
            status: ConnectionStatus::Disconnected,
            schemas: Vec::new(),
            tables: HashMap::new(),
            columns: HashMap::new(),
            indexes: HashMap::new(),
            expanded_nodes: HashSet::new(),
            loading_schemas: false,
            loading_tables: HashSet::new(),
            loading_columns: HashSet::new(),
        }
    }
}

// ============================================================================
// Object filter — drives the sidebar's object toolbar
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFilter {
    All,
    Tables,
    Views,
    Functions,
    Queries,
    History,
}

impl Default for ObjectFilter {
    fn default() -> Self {
        Self::All
    }
}

impl ObjectFilter {
    pub fn label(self) -> &'static str {
        match self {
            ObjectFilter::All => "All",
            ObjectFilter::Tables => "Tables",
            ObjectFilter::Views => "Views",
            ObjectFilter::Functions => "Functions",
            ObjectFilter::Queries => "Queries",
            ObjectFilter::History => "History",
        }
    }
}

// ============================================================================
// Command palette state
// ============================================================================

#[derive(Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected_index: usize,
    pub focus_requested: bool,
}

impl CommandPaletteState {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected_index = 0;
        self.focus_requested = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
    }
}

// ============================================================================
// AppState
// ============================================================================

pub struct AppState {
    pub connections: HashMap<ConnectionId, ConnectionState>,
    pub active_connection: Option<ConnectionId>,
    pub editor_tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub current_result: Option<QueryResult>,
    pub current_result_truncated: bool,
    pub query_running: bool,
    pub last_error: Option<String>,
    pub show_connection_dialog: bool,
    pub connection_dialog: ConnectionDialogState,
    pub saved_connections: Vec<ConnectionConfig>,
    pub default_row_limit: usize,
    pub status_message: String,

    // New (Forge design)
    pub theme_mode: ThemeMode,
    pub object_filter: ObjectFilter,
    pub sidebar_search: String,
    pub sidebar_visible: bool,
    pub result_panel_visible: bool,
    pub command_palette: CommandPaletteState,
    pub result_view_form: bool,   // Form vs Grid toggle for results
    pub result_filter: String,    // inline row filter
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            active_connection: None,
            editor_tabs: vec![EditorTab::new("Query 1")],
            active_tab: 0,
            current_result: None,
            current_result_truncated: false,
            query_running: false,
            last_error: None,
            show_connection_dialog: true,
            connection_dialog: ConnectionDialogState::default(),
            saved_connections: Vec::new(),
            default_row_limit: 1000,
            status_message: "Disconnected".to_string(),

            theme_mode: ThemeMode::default(),
            object_filter: ObjectFilter::All,
            sidebar_search: String::new(),
            sidebar_visible: true,
            result_panel_visible: true,
            command_palette: CommandPaletteState::default(),
            result_view_form: false,
            result_filter: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDialogState {
    pub display_name: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub testing: bool,
    pub test_result: Option<Result<String, String>>,
    pub editing_id: Option<ConnectionId>,
}

impl Default for ConnectionDialogState {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            use_tls: false,
            testing: false,
            test_result: None,
            editing_id: None,
        }
    }
}

impl ConnectionDialogState {
    pub fn to_config(&self) -> ConnectionConfig {
        ConnectionConfig {
            id: self.editing_id.unwrap_or_else(ConnectionId::new),
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
            display_name: config.display_name.clone(),
            host: config.host.clone(),
            port: config.port.to_string(),
            database: config.database.clone(),
            username: config.username.clone(),
            password: config.password.clone(),
            use_tls: config.use_tls,
            testing: false,
            test_result: None,
            editing_id: Some(config.id),
        }
    }
}
