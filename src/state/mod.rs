//! Application state.
//!
//! Plan v7 Phase 1.95c1 — state.rs (1025줄) 를 폴더 구조로 변환. sub-modules
//! 는 현재 빈 placeholder. 실제 함수 cut-over 는 후속 1.95c sub-stories 에서
//! 진행 (DataEditState/EditableCell/DataSource/DataFilter → data_edit,
//! TableDesignerState 통합 검토 → designer, EditorTab + Phase 3 dangling tx
//! 상태 → query).

mod data_edit;
mod designer;
pub mod migration;
mod query;
pub mod transfer;

pub use migration::MigrationWizardState;
pub use transfer::{ClipboardTables, TransferState};

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
    pub connection_error: Option<String>,
    pub opened_schemas: HashSet<String>,
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
            opened_schemas: HashSet::new(),
            loading_tables: HashSet::new(),
            loading_columns: HashSet::new(),
            loading_indexes: HashSet::new(),
            loading_foreign_keys: HashSet::new(),
            loading_rules: HashSet::new(),
            loading_triggers: HashSet::new(),
            loading_functions: HashSet::new(),
            loading_roles: false,
            connection_error: None,
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
    #[allow(dead_code)]
    User,
    Query,
    Data,
    Backup,
    Automation,
    Model,
    BI,
}

impl MainView {
    pub const TOOLBAR_TABS: [MainView; 6] = [
        MainView::Query,
        MainView::Data,
        MainView::Model,
        MainView::BI,
        MainView::Backup,
        MainView::Automation,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreePanelTab {
    #[default]
    Schema,
    Roles,
    History,
    Snippets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InfoPanelTab {
    #[default]
    Cell,
    Row,
    Schema,
    Sql,
}

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
    #[allow(dead_code)]
    pub id: uuid::Uuid,
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
            id: uuid::Uuid::new_v4(),
            title: title.into(),
            view,
            schema_filter: schema_filter.into(),
            search: search.into(),
            data_filter: None,
        }
    }
}

/// 스니펫 저장 다이얼로그의 입력 상태.
#[derive(Debug, Clone, Default)]
pub struct SnippetSaveDialogState {
    pub name: String,
    pub tags: String,
    pub sql: String,
    pub connection_id: Option<crate::types::ConnectionId>,
}

/// AI Assist 대화 상태.
#[derive(Default, Debug)]
pub struct AiAssistState {
    pub open: bool,
    pub prompt: String,
    pub generated_sql: String,
    pub generating: bool,
    pub error: Option<String>,
    pub result_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
}

pub struct AppState {
    pub connections: HashMap<ConnectionId, ConnectionState>,
    pub active_connection: Option<ConnectionId>,
    pub active_main_view: MainView,
    pub workspace_tabs: Vec<WorkspaceTab>,
    pub active_workspace_tab: usize,
    pub editor_tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub saved_snippets: Vec<crate::storage::snippets::SnippetEntry>,
    pub snippet_save_dialog: Option<SnippetSaveDialogState>,
    pub ai_assist: AiAssistState,
    /// 활성 에디터의 커서 char 위치 (매 프레임 TextEdit 출력에서 갱신).
    /// 커서 위치 문장 실행(run-statement)에 사용.
    pub editor_cursor_char: Option<usize>,
    /// 활성 에디터의 선택 영역 (정렬된 char 인덱스). None = 선택 없음.
    /// 선택 영역 실행(run-selection)에 사용.
    pub editor_selection: Option<(usize, usize)>,
    /// 에디터 찾기/바꾸기 바 표시 여부 (⌘F 로 토글).
    pub find_open: bool,
    /// 찾기 검색어 (대소문자 무시 부분 일치).
    pub find_query: String,
    /// 바꾸기 문자열.
    pub find_replace: String,
    /// 현재 포커스된 매치 인덱스 (next/prev 순환).
    pub find_match_idx: usize,
    /// EXPLAIN 플랜 트리 (파싱 결과). 창에 표시.
    pub explain_plan: Option<crate::db::explain::PlanNode>,
    pub show_explain_window: bool,
    /// AI 플랜 해석 결과(조언) + 백그라운드 작업 슬롯.
    pub explain_advice: Option<String>,
    pub explain_advice_job: std::sync::Arc<std::sync::Mutex<crate::ai::AiJob>>,
    /// AI text-to-SQL 프롬프트 바 + 백그라운드 작업 슬롯.
    pub ai_prompt_open: bool,
    pub ai_prompt_input: String,
    pub ai_job: std::sync::Arc<std::sync::Mutex<crate::ai::AiJob>>,
    /// AI 결과를 활성 탭에 append 대신 replace (에러 auto-fix 시).
    pub ai_replace_active_tab: bool,
    /// 카탈로그(시퀀스/enum/익스텐션) 브라우저 창 상태.
    pub show_catalog_window: bool,
    pub catalog_needs_fetch: bool,
    pub catalog: Option<crate::db::catalog::CatalogObjects>,
    /// CREATE EXTENSION 입력 + DROP 확인 대기 키 ("ext:name" / "seq:s.n" / "type:s.n").
    pub catalog_new_extension: String,
    pub catalog_new_sequence: String,
    pub catalog_confirm_drop: Option<String>,
    /// 테이블 COMMENT 편집 버퍼 + 현재 편집 대상 (schema, table) — 대상 변경 시 재시드.
    pub comment_edit_buffer: String,
    pub comment_edit_for: Option<(String, String)>,
    /// 컬럼 COMMENT 편집: 선택 컬럼명 + 버퍼 + 재시드 키 (schema, table, column).
    pub column_comment_selected: String,
    pub column_comment_buffer: String,
    pub column_comment_for: Option<(String, String, String)>,
    /// 권한(GRANT/REVOKE) 브라우저 창 상태 + 폼.
    pub show_privileges_window: bool,
    pub privileges_needs_fetch: bool,
    pub grants: Vec<crate::db::privileges::GrantRow>,
    pub grant_form_target: String,
    pub grant_form_grantee: String,
    pub grant_form_privilege: String,
    /// GRANT 대상 객체 종류: "table" / "sequence" / "functions".
    pub grant_form_object: String,
    /// DBA 세션 모니터 창 상태.
    pub show_sessions_window: bool,
    pub sessions_needs_fetch: bool,
    pub sessions: Vec<crate::db::sessions::SessionRow>,
    /// terminate 확인 대기 중인 pid (2-step 확인).
    pub sessions_confirm_terminate: Option<i32>,
    /// 프로덕션 파괴적 문장 — typed 확인 대기 중인 SQL.
    pub pending_prod_confirm: Option<String>,
    /// 프로덕션 확인 입력 ("production" 일치 시 실행 허용).
    pub prod_confirm_input: String,
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
    pub show_data_filter: bool,
    pub tree_panel_tab: TreePanelTab,
    pub tree_search: String,
    pub info_panel_tab: InfoPanelTab,
    pub show_command_palette: bool,
    pub command_palette_search: String,
    pub command_palette_selected: usize,
    pub active_settings_tab: usize,
    pub settings_draft: Option<AppSettings>,
    pub objects_schema_filter: String,
    pub objects_search: String,
    /// 객체 리스트(Tables/Views/MaterializedViews/Functions/Roles)에서 단일 클릭으로
    /// 선택된 항목 — info 패널이 이 선택을 사용해 컬럼/메타를 표시.
    pub objects_selected_table: Option<(String, String)>,
    pub objects_selected_function: Option<(String, String)>,
    pub objects_selected_role: Option<String>,
    pub connection_dialog: ConnectionDialogState,
    pub saved_connections: Vec<ConnectionConfig>,
    pub vault: VaultUiState,
    pub default_row_limit: usize,
    pub status_message: String,
    pub er_diagram: ERDiagramState,
    pub table_designer: std::sync::Arc<std::sync::Mutex<TableDesignerState>>,
    pub prisma_ui: std::sync::Arc<std::sync::Mutex<PrismaUIState>>,
    pub backup_format: BackupFormat,
    pub backup_running: bool,
    pub backup_last_error: Option<String>,
    pub backup_history: Vec<BackupRecord>,
    pub data_timezone: String,
    pub schema_context_menu: Option<SchemaContextMenuState>,
    pub dragging_saved_connection: Option<ConnectionId>,
    pub diagnostics_panel: DiagnosticsPanel,
    /// Plan v7 Phase 4b3/4b4 — 등록된 자동화 작업 registry. UI thread + scheduler
    /// runner 가 공유하므로 Arc<RwLock<>> wrap.
    pub automation: std::sync::Arc<std::sync::RwLock<crate::automation::scheduler::AutomationStore>>,
    /// Automation Create form 입력 draft (다음 등록 전 임시 저장).
    pub automation_draft: AutomationDraft,
    /// Plan v7 Phase 3b — Query 탭 명시 BEGIN 활성 여부.
    pub explicit_tx_active: bool,
    /// Plan v7 Phase 3b — 명시 BEGIN 시작 시각 (dangling tx 경과 측정).
    pub explicit_tx_started: Option<std::time::Instant>,
    /// Plan v7 Phase 3b — 30s warn toast 이미 표시했는지.
    pub explicit_tx_warned: bool,
    /// US-J1 — Drop CASCADE 미리보기 다이얼로그 상태. None 이면 미표시.
    pub drop_dialog: Option<DropDialogState>,
    /// US-M1 — InvalidateTable Pre 가 도착했지만 매칭 Post/SchemaChange 가 아직
    /// 안 온 oid → 시작 Instant. Post 시 remove. update() 가 5s/30s 임계 초과
    /// 항목을 EchoTimeout / CacheStale 채널로 push.
    pub pending_invalidations: HashMap<u32, std::time::Instant>,
    /// US-M2 — pending_invalidations 의 oid 중 EchoTimeout 을 이미 push 한 oid
    /// 집합. 동일 oid 에 대한 5s timeout 중복 push 방지. Post 시 함께 remove.
    pub echo_warned: HashSet<u32>,
    pub query_history: Vec<crate::storage::history::HistoryEntry>,
    pub show_history_panel: bool,
    pub show_monitoring_window: bool,
    pub show_session_monitor: bool,
    pub show_schema_diff_window: bool,
    pub schema_diff_rows: Vec<String>,
    pub session_monitor_rows: Vec<crate::ui::session_monitor::SessionRow>,
    pub diag_slow_query_ms: u64,
    /// 히스토리 패널 검색 필터(대소문자 무시 부분 일치). 메모리 내 필터링.
    pub history_search: String,
    /// 저장된 SQL 스니펫 라이브러리 (이름으로 삽입).
    pub snippets: Vec<crate::storage::snippets::SnippetEntry>,
    pub plugins: crate::plugin::PluginRegistry,
    /// 새 스니펫 저장 시 입력하는 이름 버퍼.
    pub snippet_draft_name: String,
    pub transfer: TransferState,
    pub clipboard_tables: Option<ClipboardTables>,
    pub migration_wizard: MigrationWizardState,
    pub show_backup_wizard: bool,
    pub backup_wizard_state: Option<std::sync::Arc<std::sync::Mutex<BackupWizardState>>>,
    pub restore_confirm_dialog: Option<RestoreConfirmState>,
}

/// US-J1 / US-L1 — Drop 다이얼로그의 active 상태.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropTargetKind {
    Table,
    View,
    MaterializedView,
}

impl DropTargetKind {
    /// US-L1 — `table_type` (information_schema 의 'BASE TABLE' / 'VIEW' /
    /// 'MATERIALIZED VIEW') 를 enum 으로 변환. 기본 fallback 은 Table.
    pub fn from_table_type(table_type: &str) -> Self {
        match table_type {
            "VIEW" => DropTargetKind::View,
            "MATERIALIZED VIEW" => DropTargetKind::MaterializedView,
            _ => DropTargetKind::Table,
        }
    }

    /// US-L1 — DROP SQL 의 object 키워드 ('TABLE' / 'VIEW' / 'MATERIALIZED VIEW').
    pub fn drop_keyword(&self) -> &'static str {
        match self {
            DropTargetKind::Table => "TABLE",
            DropTargetKind::View => "VIEW",
            DropTargetKind::MaterializedView => "MATERIALIZED VIEW",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DropDialogState {
    pub conn_id: ConnectionId,
    pub schema: String,
    pub table: String,
    /// US-L1 — DROP SQL 의 object 키워드 분기.
    pub kind: DropTargetKind,
    /// 종속 객체 표시 (최대 50, truncated 플래그 별도).
    pub dependents: Vec<String>,
    pub truncated: bool,
    /// dependents 조회 진행 중 여부.
    pub loading: bool,
    /// dependents 조회 실패 메시지 (있으면 UI에 노출).
    pub fetch_error: Option<String>,
    /// 대상 oid 가 무효(information_schema 권한 부족)하여 dependents 조회를 생략했는지.
    pub oid_unavailable: bool,
    /// 사용자가 'Drop CASCADE' 버튼을 click 한 후 confirmation 진행 중 여부.
    /// (현재는 dialog 가 즉시 닫히므로 미사용 — 후속 multi-step UX 시 활용)
    #[allow(dead_code)]
    pub confirming: bool,
}

impl DropDialogState {
    pub fn new(
        conn_id: ConnectionId,
        schema: impl Into<String>,
        table: impl Into<String>,
        kind: DropTargetKind,
    ) -> Self {
        Self {
            conn_id,
            schema: schema.into(),
            table: table.into(),
            kind,
            dependents: Vec::new(),
            truncated: false,
            loading: true,
            fetch_error: None,
            oid_unavailable: false,
            confirming: false,
        }
    }
}

/// Automation Create form 의 임시 입력 값. 등록 후 reset_for_create() 로 초기화.
#[derive(Debug, Default, Clone)]
pub struct AutomationDraft {
    pub title: String,
    pub sql: String,
    /// Interval 모드의 초 단위 주기. 0 이면 Once 로 해석 (즉시 1회).
    pub interval_secs: u64,
}

impl AutomationDraft {
    pub fn reset(&mut self) {
        self.title.clear();
        self.sql.clear();
        self.interval_secs = 0;
    }
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
            saved_snippets: Vec::new(),
            snippet_save_dialog: None,
            ai_assist: AiAssistState::default(),
            editor_cursor_char: None,
            editor_selection: None,
            find_open: false,
            find_query: String::new(),
            find_replace: String::new(),
            find_match_idx: 0,
            explain_plan: None,
            show_explain_window: false,
            explain_advice: None,
            explain_advice_job: std::sync::Arc::new(std::sync::Mutex::new(
                crate::ai::AiJob::default(),
            )),
            ai_prompt_open: false,
            ai_prompt_input: String::new(),
            ai_job: std::sync::Arc::new(std::sync::Mutex::new(crate::ai::AiJob::default())),
            ai_replace_active_tab: false,
            show_catalog_window: false,
            catalog_needs_fetch: false,
            catalog: None,
            catalog_new_extension: String::new(),
            catalog_new_sequence: String::new(),
            catalog_confirm_drop: None,
            comment_edit_buffer: String::new(),
            comment_edit_for: None,
            column_comment_selected: String::new(),
            column_comment_buffer: String::new(),
            column_comment_for: None,
            show_privileges_window: false,
            privileges_needs_fetch: false,
            grants: Vec::new(),
            grant_form_target: String::new(),
            grant_form_grantee: String::new(),
            grant_form_privilege: "SELECT".to_string(),
            grant_form_object: "table".to_string(),
            show_sessions_window: false,
            sessions_needs_fetch: false,
            sessions: Vec::new(),
            sessions_confirm_terminate: None,
            pending_prod_confirm: None,
            prod_confirm_input: String::new(),
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
            show_data_filter: false,
            tree_panel_tab: TreePanelTab::default(),
            tree_search: String::new(),
            info_panel_tab: InfoPanelTab::default(),
            show_command_palette: false,
            command_palette_search: String::new(),
            command_palette_selected: 0,
            active_settings_tab: 0,
            settings_draft: None,
            objects_schema_filter: String::new(),
            objects_search: String::new(),
            objects_selected_table: None,
            objects_selected_function: None,
            objects_selected_role: None,
            connection_dialog: ConnectionDialogState::default(),
            saved_connections: Vec::new(),
            vault: VaultUiState::setup_required(Vec::new()),
            default_row_limit: 1000,
            status_message: "Disconnected".to_string(),
            er_diagram: ERDiagramState::new(),
            table_designer: std::sync::Arc::new(std::sync::Mutex::new(TableDesignerState::default())),
            prisma_ui: std::sync::Arc::new(std::sync::Mutex::new(PrismaUIState::default())),
            backup_format: BackupFormat::Custom,
            backup_running: false,
            backup_last_error: None,
            backup_history: Vec::new(),
            data_timezone: "Asia/Seoul".to_string(),
            schema_context_menu: None,
            dragging_saved_connection: None,
            diagnostics_panel: DiagnosticsPanel::default(),
            automation: std::sync::Arc::new(std::sync::RwLock::new(
                crate::automation::scheduler::AutomationStore::from_tasks(
                    crate::storage::automation::load_tasks(),
                ),
            )),
            automation_draft: AutomationDraft::default(),
            explicit_tx_active: false,
            explicit_tx_started: None,
            explicit_tx_warned: false,
            drop_dialog: None,
            pending_invalidations: HashMap::new(),
            echo_warned: HashSet::new(),
            query_history: Vec::new(),
            show_history_panel: false,
            show_monitoring_window: false,
            show_session_monitor: false,
            session_monitor_rows: Vec::new(),
            show_schema_diff_window: false,
            schema_diff_rows: Vec::<String>::new(),
            diag_slow_query_ms: 500,
            history_search: String::new(),
            snippets: crate::storage::snippets::load_snippets(),
            plugins: crate::plugin::PluginRegistry::default(),
            snippet_draft_name: String::new(),
            transfer: TransferState::default(),
            clipboard_tables: None,
            migration_wizard: MigrationWizardState::default(),
            show_backup_wizard: false,
            backup_wizard_state: None,
            restore_confirm_dialog: None,
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
                selection_range: None,
                drag_anchor: None,
                editing_cell: None,
                pending_deletes: HashSet::new(),
                inserted_rows: HashSet::new(),
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
            selection_range: None,
            drag_anchor: None,
            editing_cell: None,
            pending_deletes: HashSet::new(),
            inserted_rows: HashSet::new(),
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

        let view = tab.view;
        let schema_filter = tab.schema_filter.clone();
        let search = tab.search.clone();

        // 다른 탭에서 남은 결과/에러가 새 탭의 결과 패널에 노출되지 않도록 정리.
        // Data 탭은 grid::restore_active_data_tab가 다시 채워준다.
        if view != MainView::Data {
            self.current_result = None;
            self.current_result_truncated = false;
            self.last_error = None;
        }

        self.active_main_view = view;
        self.objects_schema_filter = schema_filter.clone();
        self.objects_search = search;
        if view == MainView::Model && !schema_filter.is_empty() {
            self.er_diagram.selected_schema = schema_filter;
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
    /// libpq sslmode: require / verify-ca / verify-full.
    pub sslmode: String,
    /// CA root / client cert / client key 파일 경로 (빈 문자열 = 미설정).
    pub ssl_root_cert: String,
    pub ssl_client_cert: String,
    pub ssl_client_key: String,
    /// 선택적 폴더/그룹명 (dev/staging/prod 등). 빈 문자열 = 미분류.
    pub group: String,
    /// 읽기 전용 / 프로덕션 가드레일.
    pub read_only: bool,
    pub is_production: bool,
    /// 인증 방식 ("password" / "rds-iam") + AWS 리전.
    pub auth_mode: String,
    pub aws_region: String,
    /// SSH 터널 설정 (system ssh 포워딩).
    pub ssh_enabled: bool,
    pub ssh_host: String,
    pub ssh_port: String,
    pub ssh_user: String,
    pub ssh_key: String,
    /// 연결 URL/DSN 빠른 입력 필드 (폼과 양방향 변환용 transient 버퍼).
    pub url_input: String,
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
            sslmode: "require".to_string(),
            ssl_root_cert: String::new(),
            ssl_client_cert: String::new(),
            ssl_client_key: String::new(),
            group: String::new(),
            read_only: false,
            is_production: false,
            auth_mode: "password".to_string(),
            aws_region: String::new(),
            ssh_enabled: false,
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            ssh_user: String::new(),
            ssh_key: String::new(),
            url_input: String::new(),
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

/// 빈 문자열은 None, 그 외는 trim 후 Some(path).
fn opt_path(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
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
            sslmode: self.sslmode.clone(),
            ssl_root_cert: opt_path(&self.ssl_root_cert),
            ssl_client_cert: opt_path(&self.ssl_client_cert),
            ssl_client_key: opt_path(&self.ssl_client_key),
            read_only: self.read_only,
            is_production: self.is_production,
            auth_mode: self.auth_mode.clone(),
            aws_region: opt_path(&self.aws_region),
            ssh_tunnel: if self.ssh_enabled && !self.ssh_host.trim().is_empty() {
                Some(crate::types::SshTunnelConfig {
                    host: self.ssh_host.trim().to_string(),
                    port: self.ssh_port.trim().parse().unwrap_or(22),
                    username: self.ssh_user.trim().to_string(),
                    key_path: opt_path(&self.ssh_key),
                })
            } else {
                None
            },
            color_tag: None,
            group: {
                let g = self.group.trim();
                if g.is_empty() {
                    None
                } else {
                    Some(g.to_string())
                }
            },
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
            sslmode: if config.sslmode.is_empty() {
                "require".to_string()
            } else {
                config.sslmode.clone()
            },
            ssl_root_cert: config.ssl_root_cert.clone().unwrap_or_default(),
            ssl_client_cert: config.ssl_client_cert.clone().unwrap_or_default(),
            ssl_client_key: config.ssl_client_key.clone().unwrap_or_default(),
            read_only: config.read_only,
            is_production: config.is_production,
            auth_mode: if config.auth_mode.is_empty() {
                "password".to_string()
            } else {
                config.auth_mode.clone()
            },
            aws_region: config.aws_region.clone().unwrap_or_default(),
            ssh_enabled: config.ssh_tunnel.is_some(),
            ssh_host: config.ssh_tunnel.as_ref().map(|t| t.host.clone()).unwrap_or_default(),
            ssh_port: config
                .ssh_tunnel
                .as_ref()
                .map(|t| t.port.to_string())
                .unwrap_or_else(|| "22".to_string()),
            ssh_user: config
                .ssh_tunnel
                .as_ref()
                .map(|t| t.username.clone())
                .unwrap_or_default(),
            ssh_key: config
                .ssh_tunnel
                .as_ref()
                .and_then(|t| t.key_path.clone())
                .unwrap_or_default(),
            group: config.group.clone().unwrap_or_default(),
            url_input: crate::connection_url::PostgresConnectionUrl {
                host: config.host.clone(),
                port: config.port,
                database: config.database.clone(),
                username: config.username.clone(),
                password: config.password.clone(),
                use_tls: config.use_tls,
            }
            .to_url(),
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

#[derive(Debug, Clone)]
pub struct BackupWizardState {
    pub step: usize, // 0: Scope, 1: Format, 2: Run & Progress
    pub schema_scope: Option<String>,
    pub format: BackupFormat,
    pub running: bool,
    pub progress: f32,
    pub current_table: String,
    pub completed: bool,
    pub error: Option<String>,
    pub closed: bool,
}

#[derive(Debug, Clone)]
pub struct RestoreConfirmState {
    pub record: BackupRecord,
    pub running: bool,
    pub progress: f32,
    pub completed: bool,
    pub error: Option<String>,
}


pub fn spawn_new_window() {
    let _ = std::process::Command::new(std::env::current_exe().unwrap_or_else(|_| "ferrumgrid".into()))
        .spawn();
}
