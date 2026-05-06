use std::collections::HashMap;
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    static ref CURRENT_LANG: Arc<RwLock<Language>> = Arc::new(RwLock::new(Language::English));
    static ref TRANSLATIONS: Arc<RwLock<HashMap<Language, Translation>>> = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Korean,
    Japanese,
    ChineseSimplified,
    Spanish,
    French,
    German,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Korean => "한국어",
            Language::Japanese => "日本語",
            Language::ChineseSimplified => "简体中文",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Korean => "ko",
            Language::Japanese => "ja",
            Language::ChineseSimplified => "zh-CN",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "ko" => Language::Korean,
            "ja" => Language::Japanese,
            "zh-CN" => Language::ChineseSimplified,
            "es" => Language::Spanish,
            "fr" => Language::French,
            "de" => Language::German,
            _ => Language::English,
        }
    }

    pub fn all() -> Vec<Language> {
        vec![
            Language::English,
            Language::Korean,
            Language::Japanese,
            Language::ChineseSimplified,
            Language::Spanish,
            Language::French,
            Language::German,
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Translation {
    strings: HashMap<String, String>,
}

impl Translation {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.strings.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.strings.get(key)
    }
}

fn insert_recent_ui_en(en: &mut Translation) {
    en.insert("ctx_close_connection", "Close Connection");
    en.insert("ctx_open_connection", "Open Connection");
    en.insert("ctx_switch_connection_profile", "Switch Connection Profile");
    en.insert("ctx_no_saved_profiles", "No saved profiles");
    en.insert("ctx_edit_connection", "Edit Connection...");
    en.insert("ctx_new_connection", "New Connection");
    en.insert("ctx_delete_connection", "Delete Connection");
    en.insert("ctx_duplicate_connection", "Duplicate Connection...");
    en.insert("ctx_new_database", "New Database...");
    en.insert("ctx_new_table", "New Table");
    en.insert("ctx_new_query", "New Query");
    en.insert("ctx_console", "Console");
    en.insert("ctx_execute_sql_file", "Execute SQL File...");
    en.insert("ctx_open_schema", "Open Schema");
    en.insert("ctx_backup_schema", "Back Up {0}");
    en.insert("ctx_edit_schema", "Edit Schema...");
    en.insert("ctx_new_schema", "New Schema...");
    en.insert("ctx_delete_schema", "Delete Schema");
    en.insert("ctx_dump_sql_file", "Dump SQL File");
    en.insert("ctx_data_dictionary", "Data Dictionary...");
    en.insert(
        "ctx_reverse_database_to_model",
        "Reverse Database to Model...",
    );
    en.insert("ctx_find_in_database", "Find in Database...");
    en.insert("ctx_add_star", "Add Star");
    en.insert("ctx_color", "Color:");
    en.insert("ctx_manage_group", "Manage Group");
    en.insert("ctx_create_group", "Create Group...");
    en.insert("ctx_move_to_group", "Move to Group...");
    en.insert("ctx_share", "Share...");
    en.insert("ctx_refresh", "Refresh");
    en.insert("ctx_close_all_connections", "Close All Connections");
    en.insert("ctx_manage_connections", "Manage Connections...");
    en.insert("ctx_new_group", "New Group");

    en.insert("tree_no_connections", "No connections");
    en.insert(
        "tree_create_connection",
        "Create a connection to browse schemas",
    );
    en.insert("tree_empty", "(empty)");
    en.insert("tree_tables", "Tables");
    en.insert("tree_views", "Views");
    en.insert("tree_materialized_views", "Materialized Views");
    en.insert("tree_functions", "Functions");
    en.insert("tree_queries", "Queries");
    en.insert("tree_backups", "Backups");
    en.insert("tree_schema_backup", "Schema Backup");
    en.insert("tree_full_database_backup", "Full Database Backup");
    en.insert("tree_fields", "Fields");
    en.insert("tree_indexes", "Indexes");
    en.insert("tree_foreign_keys", "Foreign Keys");
    en.insert("tree_unique", "Unique");
    en.insert("tree_rules", "Rules");
    en.insert("tree_triggers", "Triggers");
    en.insert("tree_edit_table", "Edit Table");
    en.insert("tree_view_data_top_100", "View Data (Top 100)");
    en.insert("tree_copy_select", "Copy SELECT *");
    en.insert("tree_refresh_metadata", "Refresh Metadata");
    en.insert("tree_copy_signature", "Copy Signature");
    en.insert("tree_copy_rule_ddl", "Copy Rule DDL");
    en.insert("tree_copy_trigger_ddl", "Copy Trigger DDL");
    en.insert("tree_show_functions", "Show Functions");
    en.insert("tree_show_group", "Show {0}");
    en.insert("tree_showing_group", "Showing {0} in {1}");
    en.insert("tree_showing_functions", "Showing Functions in {0}");
    en.insert("tree_backup_schema_title", "Backup: {0}");
    en.insert("tree_backup_full_title", "Backup: full");
    en.insert("tree_backup_scope_schema", "Backup scope: {0} schema");
    en.insert("tree_backup_scope_full", "Backup scope: full database");
    en.insert(
        "tree_refreshing_connections",
        "Refreshing {0} connection(s)...",
    );
    en.insert("tree_explorer_refreshed", "Explorer refreshed");
    en.insert("tree_closing_all_connections", "Closing all connections...");

    en.insert("objects_all_schemas", "All Schemas");
    en.insert("objects_schema", "Schema");
    en.insert("objects_name", "Name");
    en.insert("objects_type", "Type");
    en.insert("objects_columns", "Columns");
    en.insert("objects_indexes", "Indexes");
    en.insert("objects_actions", "Actions");
    en.insert("objects_search", "Search");
    en.insert("objects_new_table", "New Table");
    en.insert("objects_open_model", "Open ER Diagram");
    en.insert("objects_signature", "Signature");
    en.insert("objects_returns", "Returns");
    en.insert("objects_lang", "Lang");
    en.insert("objects_role", "Role");
    en.insert("objects_login", "Login");
    en.insert("objects_privileges", "Privileges");
    en.insert("objects_valid_until", "Valid Until");
    en.insert("objects_column", "Column");
    en.insert("objects_non_null", "Non-null");
    en.insert("objects_min", "Min");
    en.insert("objects_max", "Max");
    en.insert("objects_average", "Average");
    en.insert("objects_no_active_connection", "No active connection");
    en.insert(
        "objects_no_active_connection_help",
        "Connect to PostgreSQL to browse and operate on database objects.",
    );
    en.insert("objects_tables_title", "Tables");
    en.insert(
        "objects_tables_subtitle",
        "Base tables and editable relations",
    );
    en.insert("objects_views_title", "Views");
    en.insert("objects_views_subtitle", "Virtual query-backed objects");
    en.insert("objects_materialized_title", "Materialized Views");
    en.insert("objects_materialized_subtitle", "Stored query snapshots");
    en.insert("objects_functions_title", "Functions");
    en.insert(
        "objects_functions_subtitle",
        "PostgreSQL routines by schema",
    );
    en.insert("objects_users_title", "Users");
    en.insert("objects_users_subtitle", "Roles and login permissions");
    en.insert("objects_backup_title", "Backup");
    en.insert(
        "objects_backup_subtitle",
        "pg_dump and restore command builder",
    );
    en.insert("objects_automation_title", "Automation");
    en.insert("objects_automation_subtitle", "Maintenance query presets");
    en.insert("objects_model_title", "Model");
    en.insert("objects_model_subtitle", "ER diagram and schema modeling");
    en.insert("objects_bi_title", "BI");
    en.insert("objects_bi_subtitle", "Quick result-set profiling");
    en.insert("objects_connections_title", "Connections");
    en.insert("objects_connections_subtitle", "Database connection setup");
    en.insert("objects_query_title", "Query");
    en.insert("objects_query_subtitle", "SQL editor");
    en.insert("objects_data_title", "Data");
    en.insert("objects_data_subtitle", "Browse table rows");

    en.insert("backup_schema", "Schema Backup");
    en.insert("backup_full_database", "Full Database Backup");
    en.insert("backup_no_folder_selected", "No backup folder selected");
    en.insert("backup_folder_title", "FerrumGrid Backup Folder");
    en.insert("backup_choose_folder", "Choose Folder");
    en.insert("backup_open_folder", "Open Folder");
    en.insert("backup_folder_updated", "Backup folder updated");
    en.insert("backup_format", "Format");
    en.insert("backup_custom_archive", "Custom archive (.dump)");
    en.insert("backup_plain_sql", "Plain SQL (.sql)");
    en.insert("backup_running_label", "Backing Up...");
    en.insert("backup_running_status", "Backing up {0}...");
    en.insert("backup_run", "Run Backup");
    en.insert("backup_pg_dump_running", "pg_dump is running");
    en.insert("backup_tar_archive", "Tar archive");
    en.insert("backup_recent", "Recent FerrumGrid Backups");
    en.insert("backup_no_session", "No backups in this session");
    en.insert("backup_files_title_count", "Backup Files ({0})");
    en.insert("backup_files_title", "Backup Files");
    en.insert("backup_files_refresh", "Refresh");
    en.insert("backup_files_set_folder", "Set backup folder to browse files");
    en.insert("backup_files_empty", "No backup files found");
    en.insert("backup_files_col_name", "Name");
    en.insert("backup_files_col_size", "Size");
    en.insert("backup_files_col_created", "Created");
    en.insert("backup_files_col_modified", "Modified");
    en.insert("backup_files_col_actions", "Actions");
    en.insert("backup_files_show", "Show");
    en.insert("backup_files_delete_confirm", "Delete?");
    en.insert("backup_files_yes", "Yes");
    en.insert("backup_files_no", "No");
    en.insert("backup_files_delete", "Delete");

    en.insert("schema_visualizer_title", "Schema Visualizer");
    en.insert(
        "schema_visualizer_desc",
        "Explore tables, columns, and foreign-key relationships.",
    );
    en.insert("schema_visualizer_open", "Open Visualizer");
    en.insert("visualizer_schema", "Schema");
    en.insert("visualizer_search_hint", "Search tables or columns");
    en.insert("visualizer_reload", "Reload");
    en.insert("visualizer_auto_layout", "Auto Layout");
    en.insert("visualizer_fit", "Fit");
    en.insert("visualizer_zoom", "Zoom");
    en.insert("visualizer_close_tooltip", "Close Schema Visualizer");
    en.insert("visualizer_loading_columns", "Loading columns...");
    en.insert("visualizer_loading_title", "Loading schema visualizer...");
    en.insert(
        "visualizer_loading_subtitle",
        "Tables, columns, and relationships will appear here automatically.",
    );
    en.insert("visualizer_no_matching_tables", "No matching tables");
    en.insert(
        "visualizer_clear_search_hint",
        "Clear the search box to show the full schema.",
    );
    en.insert("visualizer_no_tables_title", "No tables in this schema");
    en.insert(
        "visualizer_no_tables_subtitle",
        "Select another schema or refresh.",
    );
    en.insert("visualizer_more_columns", "+{0} more columns");
    en.insert("visualizer_count", "{0} tables  |  {1} relations");

    en.insert("workspace_close_tab", "Close Tab");
    en.insert("workspace_new_query", "New Query");
    en.insert("grid_revert", "Revert");
    en.insert("grid_edits", "{0} edits");
    en.insert("grid_pk_required", "Primary key required to update rows");
    en.insert("grid_invalid_values", "{0} invalid value(s)");
    en.insert("grid_toggle_null", "Toggle NULL");
    en.insert("grid_null_value", "NULL value");
    en.insert("grid_copy_value", "Copy Value");
    en.insert("grid_copy_sql", "Copy SQL");
    en.insert("grid_no_active_data_source", "No active data source");
    en.insert("grid_no_result_set", "No result set");
    en.insert(
        "grid_column_missing",
        "Edited column is no longer available",
    );
    en.insert(
        "grid_pk_missing",
        "Primary key column {0} is not in the result set",
    );
    en.insert(
        "grid_pk_value_missing",
        "Primary key value is not available",
    );
    en.insert("grid_not_null", "This column does not allow NULL");
    en.insert("grid_bool_error", "Use true or false");
    en.insert("grid_number_error", "Enter a valid number");
    en.insert("grid_json_error", "Enter valid JSON");
    en.insert("grid_uuid_error", "Enter a valid UUID");
    en.insert(
        "grid_bytes_error",
        "Enter hex bytes, for example \\xDEADBEEF",
    );
    en.insert("grid_date_error", "Enter a date as YYYY-MM-DD");
    en.insert(
        "grid_datetime_error",
        "Enter date and time as YYYY-MM-DD HH:MM:SS",
    );
    en.insert("grid_now", "Now");
    en.insert("grid_pick_date", "Pick Date");
    en.insert("grid_pick_time", "Pick Time");
    en.insert("grid_prev_month", "Previous Month");
    en.insert("grid_next_month", "Next Month");
    en.insert("grid_hour", "Hour");
    en.insert("grid_minute", "Min");
    en.insert("grid_second", "Sec");
    en.insert("grid_weekday_mon", "M");
    en.insert("grid_weekday_tue", "T");
    en.insert("grid_weekday_wed", "W");
    en.insert("grid_weekday_thu", "T");
    en.insert("grid_weekday_fri", "F");
    en.insert("grid_weekday_sat", "S");
    en.insert("grid_weekday_sun", "S");
    en.insert("grid_sort_asc", "Sort Ascending");
    en.insert("grid_sort_desc", "Sort Descending");
    en.insert("grid_sort_remove", "Remove Sort");
    en.insert("grid_sort_clear_all", "Clear All Sorts");
    en.insert("grid_sort_unsaved", "Apply or revert edits before sorting");
    en.insert(
        "grid_page_unsaved",
        "Apply or revert edits before changing pages",
    );
    en.insert("grid_first_page", "First Page");
    en.insert("grid_prev_page", "Previous Page");
    en.insert("grid_next_page", "Next Page");
    en.insert("grid_page", "Page");
    en.insert("grid_page_n", "Page {0}");
    en.insert("grid_limit", "Limit");
    en.insert("grid_limit_n", "Limit {0}");
    en.insert("grid_limit_error", "Enter a valid row limit");
    en.insert("grid_enum_select", "Select value");
    en.insert("grid_enum_error", "Select one of the allowed values");
    en.insert("grid_visible_range", "{0}-{1}");
    en.insert("data_info_no_selection", "No Info");
    en.insert("data_info_select_cell", "Select a row");
    en.insert("data_info_cell", "Selected Cell");
    en.insert("data_info_row", "Selected Row");
    en.insert("data_info_table", "Selected Table");
    en.insert("data_info_row_n", "Row {0}");
    en.insert("data_info_col_n", "Col {0}");
    en.insert("data_info_columns", "Columns");
    en.insert("data_info_columns_n", "{0} columns");
    en.insert("data_info_indexes_n", "{0} indexes");
    en.insert("data_info_relations_n", "{0} relations");
    en.insert("data_info_rules_n", "{0} rules");
    en.insert("data_info_triggers_n", "{0} triggers");
    en.insert("data_info_active_filter", "Active Filter");
    en.insert("data_info_relation_out", "out");
    en.insert("data_info_relation_in", "in");
    en.insert("data_info_selected", "Selected");
    en.insert("data_info_nullable", "Nullable");
    en.insert("data_info_value", "Value");
    en.insert("data_info_original", "Original");
    en.insert("data_info_revert_cell", "Revert Cell");
    en.insert("data_info_dirty", "This cell has unsaved changes");
    en.insert("data_info_yes", "Yes");
    en.insert("data_info_no", "No");
    en.insert("data_info_read_only", "This value is read-only here.");
    en.insert("data_relation_open", "Open Related Row");
    en.insert(
        "data_info_read_only_pk",
        "Primary key values are read-only here.",
    );
    en.insert(
        "data_info_no_metadata",
        "Column metadata is still loading, so editing is disabled.",
    );
}

fn insert_recent_ui_ko(ko: &mut Translation) {
    ko.insert("ctx_close_connection", "연결 닫기");
    ko.insert("ctx_open_connection", "연결 열기");
    ko.insert("ctx_switch_connection_profile", "연결 프로필 전환");
    ko.insert("ctx_no_saved_profiles", "저장된 프로필 없음");
    ko.insert("ctx_edit_connection", "연결 편집...");
    ko.insert("ctx_new_connection", "새 연결");
    ko.insert("ctx_delete_connection", "연결 삭제");
    ko.insert("ctx_duplicate_connection", "연결 복제...");
    ko.insert("ctx_new_database", "새 데이터베이스...");
    ko.insert("ctx_new_table", "새 테이블");
    ko.insert("ctx_new_query", "새 쿼리");
    ko.insert("ctx_console", "콘솔");
    ko.insert("ctx_execute_sql_file", "SQL 파일 실행...");
    ko.insert("ctx_open_schema", "스키마 열기");
    ko.insert("ctx_backup_schema", "{0} 백업");
    ko.insert("ctx_edit_schema", "스키마 편집...");
    ko.insert("ctx_new_schema", "새 스키마...");
    ko.insert("ctx_delete_schema", "스키마 삭제");
    ko.insert("ctx_dump_sql_file", "SQL 파일 덤프");
    ko.insert("ctx_data_dictionary", "데이터 사전...");
    ko.insert(
        "ctx_reverse_database_to_model",
        "데이터베이스를 모델로 리버스...",
    );
    ko.insert("ctx_find_in_database", "데이터베이스에서 찾기...");
    ko.insert("ctx_add_star", "즐겨찾기 추가");
    ko.insert("ctx_color", "색상:");
    ko.insert("ctx_manage_group", "그룹 관리");
    ko.insert("ctx_create_group", "그룹 만들기...");
    ko.insert("ctx_move_to_group", "그룹으로 이동...");
    ko.insert("ctx_share", "공유...");
    ko.insert("ctx_refresh", "새로고침");
    ko.insert("ctx_close_all_connections", "모든 연결 닫기");
    ko.insert("ctx_manage_connections", "연결 관리...");
    ko.insert("ctx_new_group", "새 그룹");

    ko.insert("tree_no_connections", "연결 없음");
    ko.insert(
        "tree_create_connection",
        "스키마를 탐색하려면 연결을 생성하세요",
    );
    ko.insert("tree_empty", "(비어 있음)");
    ko.insert("tree_tables", "테이블");
    ko.insert("tree_views", "뷰");
    ko.insert("tree_materialized_views", "구체화된 뷰");
    ko.insert("tree_functions", "함수");
    ko.insert("tree_queries", "쿼리");
    ko.insert("tree_backups", "백업");
    ko.insert("tree_schema_backup", "스키마 백업");
    ko.insert("tree_full_database_backup", "전체 데이터베이스 백업");
    ko.insert("tree_fields", "필드");
    ko.insert("tree_indexes", "인덱스");
    ko.insert("tree_foreign_keys", "외래키");
    ko.insert("tree_unique", "유니크");
    ko.insert("tree_rules", "룰");
    ko.insert("tree_triggers", "트리거");
    ko.insert("tree_edit_table", "테이블 편집");
    ko.insert("tree_view_data_top_100", "데이터 보기 (상위 100)");
    ko.insert("tree_copy_select", "SELECT * 복사");
    ko.insert("tree_refresh_metadata", "메타데이터 새로고침");
    ko.insert("tree_copy_signature", "시그니처 복사");
    ko.insert("tree_copy_rule_ddl", "룰 DDL 복사");
    ko.insert("tree_copy_trigger_ddl", "트리거 DDL 복사");
    ko.insert("tree_show_functions", "함수 보기");
    ko.insert("tree_show_group", "{0} 보기");
    ko.insert("tree_showing_group", "{1}의 {0} 표시 중");
    ko.insert("tree_showing_functions", "{0}의 함수 표시 중");
    ko.insert("tree_backup_schema_title", "백업: {0}");
    ko.insert("tree_backup_full_title", "백업: 전체");
    ko.insert("tree_backup_scope_schema", "백업 범위: {0} 스키마");
    ko.insert("tree_backup_scope_full", "백업 범위: 전체 데이터베이스");
    ko.insert("tree_refreshing_connections", "{0}개 연결 새로고침 중...");
    ko.insert("tree_explorer_refreshed", "탐색기를 새로고침했습니다");
    ko.insert("tree_closing_all_connections", "모든 연결을 닫는 중...");

    ko.insert("objects_all_schemas", "모든 스키마");
    ko.insert("objects_schema", "스키마");
    ko.insert("objects_name", "이름");
    ko.insert("objects_type", "타입");
    ko.insert("objects_columns", "컬럼");
    ko.insert("objects_indexes", "인덱스");
    ko.insert("objects_actions", "작업");
    ko.insert("objects_search", "검색");
    ko.insert("objects_new_table", "새 테이블");
    ko.insert("objects_open_model", "ER 다이어그램 열기");
    ko.insert("objects_signature", "시그니처");
    ko.insert("objects_returns", "반환값");
    ko.insert("objects_lang", "언어");
    ko.insert("objects_role", "역할");
    ko.insert("objects_login", "로그인");
    ko.insert("objects_privileges", "권한");
    ko.insert("objects_valid_until", "유효 기간");
    ko.insert("objects_column", "컬럼");
    ko.insert("objects_non_null", "Null 아님");
    ko.insert("objects_min", "최소");
    ko.insert("objects_max", "최대");
    ko.insert("objects_average", "평균");
    ko.insert("objects_no_active_connection", "활성 연결 없음");
    ko.insert(
        "objects_no_active_connection_help",
        "PostgreSQL에 연결하면 데이터베이스 오브젝트를 탐색하고 조작할 수 있습니다.",
    );
    ko.insert("objects_tables_title", "테이블");
    ko.insert("objects_tables_subtitle", "기본 테이블과 편집 가능한 관계");
    ko.insert("objects_views_title", "뷰");
    ko.insert("objects_views_subtitle", "쿼리 기반 가상 오브젝트");
    ko.insert("objects_materialized_title", "구체화된 뷰");
    ko.insert("objects_materialized_subtitle", "저장된 쿼리 스냅샷");
    ko.insert("objects_functions_title", "함수");
    ko.insert("objects_functions_subtitle", "스키마별 PostgreSQL 루틴");
    ko.insert("objects_users_title", "사용자");
    ko.insert("objects_users_subtitle", "역할 및 로그인 권한");
    ko.insert("objects_backup_title", "백업");
    ko.insert("objects_backup_subtitle", "pg_dump 및 복원 명령 빌더");
    ko.insert("objects_automation_title", "자동화");
    ko.insert("objects_automation_subtitle", "유지보수 쿼리 프리셋");
    ko.insert("objects_model_title", "모델");
    ko.insert("objects_model_subtitle", "ER 다이어그램 및 스키마 모델링");
    ko.insert("objects_bi_title", "BI");
    ko.insert("objects_bi_subtitle", "결과셋 빠른 프로파일링");
    ko.insert("objects_connections_title", "연결");
    ko.insert("objects_connections_subtitle", "데이터베이스 연결 설정");
    ko.insert("objects_query_title", "쿼리");
    ko.insert("objects_query_subtitle", "SQL 에디터");
    ko.insert("objects_data_title", "데이터");
    ko.insert("objects_data_subtitle", "테이블 행 탐색");

    ko.insert("backup_schema", "스키마 백업");
    ko.insert("backup_full_database", "전체 데이터베이스 백업");
    ko.insert("backup_no_folder_selected", "선택된 백업 폴더 없음");
    ko.insert("backup_folder_title", "FerrumGrid 백업 폴더");
    ko.insert("backup_choose_folder", "폴더 선택");
    ko.insert("backup_open_folder", "폴더 열기");
    ko.insert("backup_folder_updated", "백업 폴더가 업데이트되었습니다");
    ko.insert("backup_format", "형식");
    ko.insert("backup_custom_archive", "커스텀 아카이브 (.dump)");
    ko.insert("backup_plain_sql", "Plain SQL (.sql)");
    ko.insert("backup_running_label", "백업 중...");
    ko.insert("backup_running_status", "{0} 백업 중...");
    ko.insert("backup_run", "백업 실행");
    ko.insert("backup_pg_dump_running", "pg_dump 실행 중");
    ko.insert("backup_tar_archive", "Tar 아카이브");
    ko.insert("backup_recent", "최근 FerrumGrid 백업");
    ko.insert("backup_no_session", "이번 세션에 백업 없음");
    ko.insert("backup_files_title_count", "백업 파일 ({0})");
    ko.insert("backup_files_title", "백업 파일");
    ko.insert("backup_files_refresh", "새로고침");
    ko.insert("backup_files_set_folder", "파일을 보려면 백업 폴더를 설정하세요");
    ko.insert("backup_files_empty", "백업 파일이 없습니다");
    ko.insert("backup_files_col_name", "이름");
    ko.insert("backup_files_col_size", "크기");
    ko.insert("backup_files_col_created", "생성일");
    ko.insert("backup_files_col_modified", "수정일");
    ko.insert("backup_files_col_actions", "작업");
    ko.insert("backup_files_show", "보기");
    ko.insert("backup_files_delete_confirm", "삭제?");
    ko.insert("backup_files_yes", "예");
    ko.insert("backup_files_no", "아니오");
    ko.insert("backup_files_delete", "삭제");

    ko.insert("schema_visualizer_title", "스키마 비주얼라이저");
    ko.insert(
        "schema_visualizer_desc",
        "테이블, 컬럼, 외래키 관계를 시각적으로 탐색합니다.",
    );
    ko.insert("schema_visualizer_open", "비주얼라이저 열기");
    ko.insert("visualizer_schema", "스키마");
    ko.insert("visualizer_search_hint", "테이블 또는 컬럼 검색");
    ko.insert("visualizer_reload", "다시 불러오기");
    ko.insert("visualizer_auto_layout", "자동 배치");
    ko.insert("visualizer_fit", "맞춤");
    ko.insert("visualizer_zoom", "확대/축소");
    ko.insert("visualizer_close_tooltip", "스키마 비주얼라이저 닫기");
    ko.insert("visualizer_loading_columns", "컬럼 불러오는 중...");
    ko.insert(
        "visualizer_loading_title",
        "스키마 비주얼라이저 불러오는 중...",
    );
    ko.insert(
        "visualizer_loading_subtitle",
        "테이블, 컬럼, 관계가 자동으로 여기에 표시됩니다.",
    );
    ko.insert("visualizer_no_matching_tables", "일치하는 테이블 없음");
    ko.insert(
        "visualizer_clear_search_hint",
        "전체 스키마를 보려면 검색어를 지우세요.",
    );
    ko.insert("visualizer_no_tables_title", "이 스키마에 테이블이 없습니다");
    ko.insert(
        "visualizer_no_tables_subtitle",
        "다른 스키마를 선택하거나 새로고침하세요.",
    );
    ko.insert("visualizer_more_columns", "+{0}개 컬럼 더");
    ko.insert("visualizer_count", "{0}개 테이블  |  {1}개 관계");

    ko.insert("workspace_close_tab", "탭 닫기");
    ko.insert("workspace_new_query", "새 쿼리");
    ko.insert("grid_revert", "되돌리기");
    ko.insert("grid_edits", "{0}개 수정");
    ko.insert("grid_pk_required", "행 업데이트에는 기본키가 필요합니다");
    ko.insert("grid_invalid_values", "잘못된 값 {0}개");
    ko.insert("grid_toggle_null", "NULL 전환");
    ko.insert("grid_null_value", "NULL 값");
    ko.insert("grid_copy_value", "값 복사");
    ko.insert("grid_copy_sql", "SQL 복사");
    ko.insert("grid_no_active_data_source", "활성 데이터 소스 없음");
    ko.insert("grid_no_result_set", "결과 없음");
    ko.insert(
        "grid_column_missing",
        "수정한 컬럼을 더 이상 찾을 수 없습니다",
    );
    ko.insert("grid_pk_missing", "기본키 컬럼 {0}이 결과셋에 없습니다");
    ko.insert("grid_pk_value_missing", "기본키 값을 찾을 수 없습니다");
    ko.insert("grid_not_null", "이 컬럼은 NULL을 허용하지 않습니다");
    ko.insert("grid_bool_error", "true 또는 false를 입력하세요");
    ko.insert("grid_number_error", "올바른 숫자를 입력하세요");
    ko.insert("grid_json_error", "올바른 JSON을 입력하세요");
    ko.insert("grid_uuid_error", "올바른 UUID를 입력하세요");
    ko.insert(
        "grid_bytes_error",
        "16진수 바이트를 입력하세요. 예: \\xDEADBEEF",
    );
    ko.insert("grid_date_error", "날짜를 YYYY-MM-DD 형식으로 입력하세요");
    ko.insert(
        "grid_datetime_error",
        "날짜와 시간을 YYYY-MM-DD HH:MM:SS 형식으로 입력하세요",
    );
    ko.insert("grid_now", "지금");
    ko.insert("grid_pick_date", "날짜 선택");
    ko.insert("grid_pick_time", "시간 선택");
    ko.insert("grid_prev_month", "이전 달");
    ko.insert("grid_next_month", "다음 달");
    ko.insert("grid_hour", "시");
    ko.insert("grid_minute", "분");
    ko.insert("grid_second", "초");
    ko.insert("grid_weekday_mon", "월");
    ko.insert("grid_weekday_tue", "화");
    ko.insert("grid_weekday_wed", "수");
    ko.insert("grid_weekday_thu", "목");
    ko.insert("grid_weekday_fri", "금");
    ko.insert("grid_weekday_sat", "토");
    ko.insert("grid_weekday_sun", "일");
    ko.insert("grid_sort_asc", "오름차순 정렬");
    ko.insert("grid_sort_desc", "내림차순 정렬");
    ko.insert("grid_sort_remove", "정렬 해제");
    ko.insert("grid_sort_clear_all", "전체 정렬 해제");
    ko.insert(
        "grid_sort_unsaved",
        "정렬하기 전에 수정사항을 적용하거나 되돌리세요",
    );
    ko.insert(
        "grid_page_unsaved",
        "페이지를 바꾸기 전에 수정사항을 적용하거나 되돌리세요",
    );
    ko.insert("grid_first_page", "첫 페이지");
    ko.insert("grid_prev_page", "이전 페이지");
    ko.insert("grid_next_page", "다음 페이지");
    ko.insert("grid_page", "페이지");
    ko.insert("grid_page_n", "{0}페이지");
    ko.insert("grid_limit", "제한");
    ko.insert("grid_limit_n", "제한 {0}");
    ko.insert("grid_limit_error", "올바른 행 제한 숫자를 입력하세요");
    ko.insert("grid_enum_select", "값 선택");
    ko.insert("grid_enum_error", "허용된 값 중 하나를 선택하세요");
    ko.insert("grid_visible_range", "{0}-{1}");
    ko.insert("data_info_no_selection", "정보 없음");
    ko.insert("data_info_select_cell", "행을 선택하세요");
    ko.insert("data_info_cell", "선택한 셀");
    ko.insert("data_info_row", "선택한 행");
    ko.insert("data_info_table", "선택한 테이블");
    ko.insert("data_info_row_n", "{0}행");
    ko.insert("data_info_col_n", "{0}열");
    ko.insert("data_info_columns", "컬럼");
    ko.insert("data_info_columns_n", "{0}개 컬럼");
    ko.insert("data_info_indexes_n", "{0}개 인덱스");
    ko.insert("data_info_relations_n", "{0}개 관계");
    ko.insert("data_info_rules_n", "{0}개 룰");
    ko.insert("data_info_triggers_n", "{0}개 트리거");
    ko.insert("data_info_active_filter", "적용된 필터");
    ko.insert("data_info_relation_out", "나감");
    ko.insert("data_info_relation_in", "들어옴");
    ko.insert("data_info_selected", "선택됨");
    ko.insert("data_info_nullable", "NULL 허용");
    ko.insert("data_info_value", "값");
    ko.insert("data_info_original", "원본");
    ko.insert("data_info_revert_cell", "셀 되돌리기");
    ko.insert(
        "data_info_dirty",
        "이 셀에 저장되지 않은 변경사항이 있습니다",
    );
    ko.insert("data_info_yes", "예");
    ko.insert("data_info_no", "아니오");
    ko.insert("data_info_read_only", "여기서는 읽기 전용 값입니다.");
    ko.insert("data_relation_open", "관련 행 열기");
    ko.insert(
        "data_info_read_only_pk",
        "기본키 값은 여기서 읽기 전용입니다.",
    );
    ko.insert(
        "data_info_no_metadata",
        "컬럼 메타데이터를 불러오는 중이라 편집이 비활성화됩니다.",
    );
}

pub fn init() {
    let mut translations = TRANSLATIONS.write().unwrap();

    // English (default)
    let mut en = Translation::new();
    en.insert("app_title", "FerrumGrid");
    en.insert("menu_file", "File");
    en.insert("menu_about", "About FerrumGrid");
    en.insert("menu_new_connection", "New Connection");
    en.insert("menu_close_window", "Close Window");
    en.insert("menu_show_main_window", "Show Main Window");
    en.insert("menu_quit", "Quit");
    en.insert("menu_query", "Query");
    en.insert("menu_execute", "Execute");
    en.insert("menu_new_tab", "New Tab");
    en.insert("menu_view", "View");
    en.insert("menu_tools", "Tools");
    en.insert("menu_light_mode", "Light Mode");
    en.insert("menu_dark_mode", "Dark Mode");
    en.insert("menu_er_diagram", "ER Diagram");
    en.insert("menu_table_designer", "Table Designer");
    en.insert("menu_prisma", "Prisma Integration");
    en.insert("menu_language", "Language");
    en.insert("menu_settings", "Settings");
    en.insert("explorer_title", "Explorer");
    en.insert("explorer_new", "New");
    en.insert("status_connected", "Connected");
    en.insert("status_disconnected", "Disconnected");
    en.insert("status_connecting", "Connecting...");
    en.insert("connection_dialog_title", "New Connection");
    en.insert("connection_details", "Connection Details");
    en.insert("connection_saved", "Saved Connections");
    en.insert("connection_name", "Name");
    en.insert("connection_host", "Host");
    en.insert("connection_port", "Port");
    en.insert("connection_database", "Database");
    en.insert("connection_username", "Username");
    en.insert("connection_password", "Password");
    en.insert("connection_use_tls", "Use TLS");
    en.insert("connection_encrypted", "Encrypted");
    en.insert("connection_unencrypted", "Unencrypted");
    en.insert("connection_ssh_tunnel", "SSH Tunnel");
    en.insert("connection_coming_soon", "Coming soon");
    en.insert("connection_test", "Test Connection");
    en.insert("connection_testing", "Testing connection...");
    en.insert("connection_connect", "Connect");
    en.insert("connection_save", "Save");
    en.insert("connection_cancel", "Cancel");
    en.insert(
        "connection_clipboard_title",
        "PostgreSQL URL detected in clipboard",
    );
    en.insert(
        "connection_clipboard_message",
        "Apply it to this new connection?",
    );
    en.insert("connection_clipboard_apply", "Apply");
    en.insert("connection_clipboard_ignore", "Ignore");
    en.insert("connection_clipboard_password_present", "Password included");
    en.insert("connection_clipboard_password_empty", "No password");
    en.insert("vault_title", "FerrumGrid Vault");
    en.insert("vault_setup_title", "Set up Personal Vault");
    en.insert("vault_unlock_title", "Unlock Personal Vault");
    en.insert("vault_unlocked_title", "Personal Vault Unlocked");
    en.insert(
        "vault_subtitle",
        "Connections and credentials are encrypted before they touch disk.",
    );
    en.insert("vault_name", "Vault");
    en.insert("vault_master_password", "Master Password");
    en.insert("vault_confirm_password", "Confirm Password");
    en.insert("vault_create_button", "Create Vault");
    en.insert("vault_unlock_button", "Unlock");
    en.insert("vault_show_password", "Show");
    en.insert("vault_hide_password", "Hide");
    en.insert(
        "vault_legacy_connections_will_migrate",
        "legacy connections will be encrypted into this vault.",
    );
    en.insert("vault_error_short_password", "Use at least 8 characters.");
    en.insert("vault_error_password_mismatch", "Passwords do not match.");
    en.insert("vault_error_locked", "Unlock the Personal Vault first.");
    en.insert("vault_unlocked_status", "Personal Vault unlocked");
    en.insert("query_editor_placeholder", "Enter your SQL query here...");
    en.insert("query_execute", "Execute");
    en.insert("query_cancel", "Cancel");
    en.insert("result_rows", "rows");
    en.insert("result_execution_time", "Execution time");
    en.insert("result_truncated", "Results truncated");
    en.insert("button_refresh", "Refresh");
    en.insert("button_delete", "Delete");
    en.insert("button_edit", "Edit");
    en.insert("button_add", "Add");
    en.insert("button_save", "Save");
    en.insert("button_cancel", "Cancel");
    en.insert("button_ok", "OK");
    en.insert("button_close", "Close");
    en.insert("button_apply", "Apply");
    en.insert("button_generate", "Generate");
    en.insert("er_diagram_title", "ER Diagram");
    en.insert("er_schema_select", "Select Schema");
    en.insert("er_load_schema", "Load Schema");
    en.insert("er_auto_layout", "Auto Layout");
    en.insert("er_clear", "Clear");
    en.insert("table_designer_title", "Table Designer");
    en.insert("table_designer_new", "Create Table");
    en.insert("table_designer_edit", "Edit Table");
    en.insert("table_name", "Table Name");
    en.insert("table_schema", "Schema");
    en.insert("column_name", "Column Name");
    en.insert("column_type", "Data Type");
    en.insert("column_nullable", "Nullable");
    en.insert("column_primary_key", "Primary Key");
    en.insert("column_unique", "Unique");
    en.insert("column_default", "Default");
    en.insert("column_foreign_key", "Foreign Key");
    en.insert("index_name", "Index Name");
    en.insert("index_columns", "Columns");
    en.insert("ddl_preview", "DDL Preview");
    en.insert("ddl_copy", "Copy to Clipboard");
    en.insert("prisma_title", "Prisma Integration");
    en.insert("prisma_schema_path", "Schema Path");
    en.insert("prisma_browse", "Browse");
    en.insert("prisma_load", "Load");
    en.insert("prisma_save", "Save");
    en.insert("prisma_create_from_db", "Create from DB");
    en.insert("prisma_introspect", "Introspect");
    en.insert("prisma_migrate_dev", "Migrate Dev");
    en.insert("prisma_migrate_deploy", "Migrate Deploy");
    en.insert("prisma_generate", "Generate Client");
    en.insert("prisma_validate", "Validate");
    en.insert("prisma_not_installed", "Prisma CLI not found");
    en.insert(
        "prisma_install_help",
        "Please install Prisma CLI: npm install -g prisma",
    );
    en.insert("error", "Error");
    en.insert("warning", "Warning");
    en.insert("success", "Success");
    en.insert("info", "Info");
    en.insert("confirm", "Confirm");
    en.insert("loading", "Loading...");
    en.insert("no_data", "No data");
    en.insert("no_connection", "No connection");
    en.insert("select_connection", "Select a connection");

    // Toolbar
    en.insert("toolbar_connection", "Connection");
    en.insert("toolbar_table", "Table");
    en.insert("toolbar_view", "View");
    en.insert("toolbar_materialized_view", "Materialized View");
    en.insert("toolbar_function", "Function");
    en.insert("toolbar_user", "User");
    en.insert("toolbar_others", "Others");
    en.insert("toolbar_query", "Query");
    en.insert("toolbar_backup", "Backup");
    en.insert("toolbar_automation", "Automation");
    en.insert("toolbar_model", "Model");
    en.insert("toolbar_bi", "BI");
    en.insert("view_toggle_navigator", "Toggle Navigator Pane");
    en.insert("view_toggle_results", "Toggle Results Pane");
    en.insert("view_toggle_info", "Toggle Info Pane");
    en.insert("settings_title", "Settings");
    en.insert("settings_general", "General");
    en.insert("settings_language", "Language");
    en.insert("settings_appearance", "Appearance");
    en.insert("settings_dark_mode", "Dark Mode");
    en.insert("settings_database", "Database");
    en.insert("settings_default_row_limit", "Default Row Limit");
    en.insert("settings_data_timezone", "Data Time Zone");
    en.insert("settings_auto_commit", "Auto Commit");
    en.insert(
        "settings_confirm_destructive",
        "Confirm Destructive Actions",
    );
    en.insert("settings_saved", "Settings saved");
    en.insert("settings_restore_defaults", "Restore Defaults");
    en.insert("settings_tab_general", "General");
    en.insert("settings_tab_tabs", "Tabs");
    en.insert("settings_tab_code_completion", "Code Completion");
    en.insert("settings_tab_editor", "Editor");
    en.insert("settings_tab_records", "Records");
    en.insert("settings_tab_auto_recovery", "Auto Recovery");
    en.insert("settings_tab_ai", "AI");
    en.insert("settings_tab_environment", "Environment");
    en.insert("settings_tab_advanced", "Advanced");
    en.insert("settings_appearance_system", "System Default");
    en.insert("settings_appearance_dark", "Dark");
    en.insert("settings_appearance_light", "Light");
    en.insert("settings_main_window", "Main Window");
    en.insert(
        "settings_show_schema_objects",
        "Show objects under schema in navigation pane",
    );
    en.insert(
        "settings_show_table_objects",
        "Show objects under table in navigation pane",
    );
    en.insert("settings_object_list_font", "Object List Font");
    en.insert("settings_font", "Font");
    en.insert("settings_use_default_font", "Use default font");
    en.insert("settings_confirm_dialog", "Confirm Dialog");
    en.insert("settings_safe_confirm_dialog", "Use safe confirm dialog");
    en.insert(
        "settings_ask_close_queries",
        "Ask to save new queries/profiles before closing",
    );
    en.insert(
        "settings_ask_close_tables",
        "Ask to save new table profiles before closing",
    );
    en.insert("settings_database_items", "Database Items");
    en.insert("settings_show_function_wizard", "Show function wizard");
    en.insert("settings_usage_data", "Usage Data");
    en.insert("settings_share_usage_data", "Share Usage Data");
    en.insert(
        "settings_usage_data_help",
        "Help us improve FerrumGrid by automatically sending usage data.",
    );
    en.insert("settings_update", "Update");
    en.insert(
        "settings_auto_check_updates",
        "Automatically check for updates",
    );
    en.insert(
        "settings_include_system_profile",
        "Includes anonymous system profile",
    );
    en.insert("settings_open_queries_in_tabs", "Open new queries in tabs");
    en.insert("settings_enable_code_completion", "Enable code completion");
    en.insert("settings_completion_popup", "Show suggestions while typing");
    en.insert("settings_show_line_numbers", "Show line numbers");
    en.insert("settings_enable_auto_recovery", "Enable auto recovery");
    en.insert("settings_ai_assistant", "Enable AI assistant");
    en.insert(
        "settings_placeholder_hint",
        "More controls for this section will land here.",
    );
    en.insert("about_version", "Version");
    en.insert("about_edition", "Developer Preview");
    en.insert("about_engine", "PostgreSQL Workbench");
    en.insert("about_author", "FerrumGrid Studio");
    insert_recent_ui_en(&mut en);

    translations.insert(Language::English, en);

    // Korean
    let mut ko = Translation::new();
    ko.insert("app_title", "FerrumGrid");
    ko.insert("menu_file", "파일");
    ko.insert("menu_about", "FerrumGrid 정보");
    ko.insert("menu_new_connection", "새 연결");
    ko.insert("menu_close_window", "창 닫기");
    ko.insert("menu_show_main_window", "메인 창 보기");
    ko.insert("menu_quit", "종료");
    ko.insert("menu_query", "쿼리");
    ko.insert("menu_execute", "실행");
    ko.insert("menu_new_tab", "새 탭");
    ko.insert("menu_view", "보기");
    ko.insert("menu_tools", "도구");
    ko.insert("menu_light_mode", "라이트 모드");
    ko.insert("menu_dark_mode", "다크 모드");
    ko.insert("menu_er_diagram", "ER 다이어그램");
    ko.insert("menu_table_designer", "테이블 디자이너");
    ko.insert("menu_prisma", "Prisma 연동");
    ko.insert("menu_language", "언어");
    ko.insert("menu_settings", "설정");
    ko.insert("explorer_title", "탐색기");
    ko.insert("explorer_new", "새로");
    ko.insert("status_connected", "연결됨");
    ko.insert("status_disconnected", "연결 해제");
    ko.insert("status_connecting", "연결 중...");
    ko.insert("connection_dialog_title", "새 연결");
    ko.insert("connection_details", "연결 정보");
    ko.insert("connection_saved", "저장된 연결");
    ko.insert("connection_name", "이름");
    ko.insert("connection_host", "호스트");
    ko.insert("connection_port", "포트");
    ko.insert("connection_database", "데이터베이스");
    ko.insert("connection_username", "사용자명");
    ko.insert("connection_password", "비밀번호");
    ko.insert("connection_use_tls", "TLS 사용");
    ko.insert("connection_encrypted", "암호화됨");
    ko.insert("connection_unencrypted", "암호화 안 됨");
    ko.insert("connection_ssh_tunnel", "SSH 터널");
    ko.insert("connection_coming_soon", "곧 지원 예정");
    ko.insert("connection_test", "연결 테스트");
    ko.insert("connection_testing", "연결 테스트 중...");
    ko.insert("connection_connect", "연결");
    ko.insert("connection_save", "저장");
    ko.insert("connection_cancel", "취소");
    ko.insert(
        "connection_clipboard_title",
        "클립보드에서 PostgreSQL URL을 찾음",
    );
    ko.insert("connection_clipboard_message", "새 연결에 적용할까요?");
    ko.insert("connection_clipboard_apply", "적용");
    ko.insert("connection_clipboard_ignore", "무시");
    ko.insert("connection_clipboard_password_present", "비밀번호 포함");
    ko.insert("connection_clipboard_password_empty", "비밀번호 없음");
    ko.insert("vault_title", "FerrumGrid Vault");
    ko.insert("vault_setup_title", "Personal Vault 설정");
    ko.insert("vault_unlock_title", "Personal Vault 잠금 해제");
    ko.insert("vault_unlocked_title", "Personal Vault 열림");
    ko.insert(
        "vault_subtitle",
        "연결 정보와 자격 증명은 디스크에 닿기 전에 암호화됩니다.",
    );
    ko.insert("vault_name", "Vault");
    ko.insert("vault_master_password", "마스터 비밀번호");
    ko.insert("vault_confirm_password", "비밀번호 확인");
    ko.insert("vault_create_button", "Vault 생성");
    ko.insert("vault_unlock_button", "잠금 해제");
    ko.insert("vault_show_password", "보기");
    ko.insert("vault_hide_password", "숨김");
    ko.insert(
        "vault_legacy_connections_will_migrate",
        "개의 기존 연결을 이 Vault로 암호화합니다.",
    );
    ko.insert("vault_error_short_password", "8자 이상 입력하세요.");
    ko.insert(
        "vault_error_password_mismatch",
        "비밀번호가 일치하지 않습니다.",
    );
    ko.insert(
        "vault_error_locked",
        "먼저 Personal Vault 잠금을 해제하세요.",
    );
    ko.insert("vault_unlocked_status", "Personal Vault 잠금 해제됨");
    ko.insert("query_editor_placeholder", "SQL 쿼리를 입력하세요...");
    ko.insert("query_execute", "실행");
    ko.insert("query_cancel", "취소");
    ko.insert("result_rows", "행");
    ko.insert("result_execution_time", "실행 시간");
    ko.insert("result_truncated", "결과가 잘렸습니다");
    ko.insert("button_refresh", "새로고침");
    ko.insert("button_delete", "삭제");
    ko.insert("button_edit", "편집");
    ko.insert("button_add", "추가");
    ko.insert("button_save", "저장");
    ko.insert("button_cancel", "취소");
    ko.insert("button_ok", "확인");
    ko.insert("button_close", "닫기");
    ko.insert("button_apply", "적용");
    ko.insert("button_generate", "생성");
    ko.insert("er_diagram_title", "ER 다이어그램");
    ko.insert("er_schema_select", "스키마 선택");
    ko.insert("er_load_schema", "스키마 로드");
    ko.insert("er_auto_layout", "자동 배치");
    ko.insert("er_clear", "지우기");
    ko.insert("table_designer_title", "테이블 디자이너");
    ko.insert("table_designer_new", "테이블 생성");
    ko.insert("table_designer_edit", "테이블 편집");
    ko.insert("table_name", "테이블명");
    ko.insert("table_schema", "스키마");
    ko.insert("column_name", "컬럼명");
    ko.insert("column_type", "데이터 타입");
    ko.insert("column_nullable", "NULL 허용");
    ko.insert("column_primary_key", "기본키");
    ko.insert("column_unique", "유니크");
    ko.insert("column_default", "기본값");
    ko.insert("column_foreign_key", "외래키");
    ko.insert("index_name", "인덱스명");
    ko.insert("index_columns", "컬럼");
    ko.insert("ddl_preview", "DDL 미리보기");
    ko.insert("ddl_copy", "클립보드에 복사");
    ko.insert("prisma_title", "Prisma 연동");
    ko.insert("prisma_schema_path", "스키마 경로");
    ko.insert("prisma_browse", "찾아보기");
    ko.insert("prisma_load", "불러오기");
    ko.insert("prisma_save", "저장");
    ko.insert("prisma_create_from_db", "DB에서 생성");
    ko.insert("prisma_introspect", "인트로스펙트");
    ko.insert("prisma_migrate_dev", "개발 마이그레이션");
    ko.insert("prisma_migrate_deploy", "배포 마이그레이션");
    ko.insert("prisma_generate", "클라이언트 생성");
    ko.insert("prisma_validate", "검증");
    ko.insert("prisma_not_installed", "Prisma CLI를 찾을 수 없습니다");
    ko.insert(
        "prisma_install_help",
        "Prisma CLI를 설치하세요: npm install -g prisma",
    );
    ko.insert("error", "오류");
    ko.insert("warning", "경고");
    ko.insert("success", "성공");
    ko.insert("info", "정보");
    ko.insert("confirm", "확인");
    ko.insert("loading", "로딩 중...");
    ko.insert("no_data", "데이터 없음");
    ko.insert("no_connection", "연결 없음");
    ko.insert("select_connection", "연결을 선택하세요");

    // Toolbar
    ko.insert("toolbar_connection", "연결");
    ko.insert("toolbar_table", "테이블");
    ko.insert("toolbar_view", "뷰");
    ko.insert("toolbar_materialized_view", "구체화된 뷰");
    ko.insert("toolbar_function", "함수");
    ko.insert("toolbar_user", "사용자");
    ko.insert("toolbar_others", "기타");
    ko.insert("toolbar_query", "쿼리");
    ko.insert("toolbar_backup", "백업");
    ko.insert("toolbar_automation", "자동화");
    ko.insert("toolbar_model", "모델");
    ko.insert("toolbar_bi", "BI");
    ko.insert("view_toggle_navigator", "탐색기 패널 보이기/숨기기");
    ko.insert("view_toggle_results", "결과 패널 보이기/숨기기");
    ko.insert("view_toggle_info", "정보 패널 보이기/숨기기");
    ko.insert("settings_title", "설정");
    ko.insert("settings_general", "일반");
    ko.insert("settings_language", "언어");
    ko.insert("settings_appearance", "화면");
    ko.insert("settings_dark_mode", "다크 모드");
    ko.insert("settings_database", "데이터베이스");
    ko.insert("settings_default_row_limit", "기본 행 제한");
    ko.insert("settings_data_timezone", "데이터 시간대");
    ko.insert("settings_auto_commit", "자동 커밋");
    ko.insert("settings_confirm_destructive", "삭제/변경 작업 확인");
    ko.insert("settings_saved", "설정 저장됨");
    ko.insert("settings_restore_defaults", "기본값 복원");
    ko.insert("settings_tab_general", "일반");
    ko.insert("settings_tab_tabs", "탭");
    ko.insert("settings_tab_code_completion", "코드 완성");
    ko.insert("settings_tab_editor", "에디터");
    ko.insert("settings_tab_records", "레코드");
    ko.insert("settings_tab_auto_recovery", "자동 복구");
    ko.insert("settings_tab_ai", "AI");
    ko.insert("settings_tab_environment", "환경");
    ko.insert("settings_tab_advanced", "고급");
    ko.insert("settings_appearance_system", "시스템 기본값");
    ko.insert("settings_appearance_dark", "다크");
    ko.insert("settings_appearance_light", "라이트");
    ko.insert("settings_main_window", "메인 윈도우");
    ko.insert(
        "settings_show_schema_objects",
        "탐색 패널의 스키마 아래에 오브젝트 표시",
    );
    ko.insert(
        "settings_show_table_objects",
        "탐색 패널의 테이블 아래에 오브젝트 표시",
    );
    ko.insert("settings_object_list_font", "오브젝트 목록 폰트");
    ko.insert("settings_font", "폰트");
    ko.insert("settings_use_default_font", "기본 폰트 사용");
    ko.insert("settings_confirm_dialog", "확인 대화상자");
    ko.insert("settings_safe_confirm_dialog", "안전 확인 대화상자 사용");
    ko.insert(
        "settings_ask_close_queries",
        "닫기 전에 새 쿼리/프로필 저장 여부 묻기",
    );
    ko.insert(
        "settings_ask_close_tables",
        "닫기 전에 새 테이블 프로필 저장 여부 묻기",
    );
    ko.insert("settings_database_items", "데이터베이스 항목");
    ko.insert("settings_show_function_wizard", "함수 마법사 표시");
    ko.insert("settings_usage_data", "사용 데이터");
    ko.insert("settings_share_usage_data", "사용 데이터 공유");
    ko.insert(
        "settings_usage_data_help",
        "FerrumGrid 개선을 위해 사용 데이터를 자동으로 보냅니다.",
    );
    ko.insert("settings_update", "업데이트");
    ko.insert("settings_auto_check_updates", "업데이트 자동 확인");
    ko.insert("settings_include_system_profile", "익명 시스템 프로필 포함");
    ko.insert("settings_open_queries_in_tabs", "새 쿼리를 탭으로 열기");
    ko.insert("settings_enable_code_completion", "코드 완성 사용");
    ko.insert("settings_completion_popup", "입력 중 추천 표시");
    ko.insert("settings_show_line_numbers", "줄 번호 표시");
    ko.insert("settings_enable_auto_recovery", "자동 복구 사용");
    ko.insert("settings_ai_assistant", "AI 어시스턴트 사용");
    ko.insert(
        "settings_placeholder_hint",
        "이 섹션의 추가 제어 항목이 여기에 들어갑니다.",
    );
    ko.insert("about_version", "버전");
    ko.insert("about_edition", "개발자 프리뷰");
    ko.insert("about_engine", "PostgreSQL 워크벤치");
    ko.insert("about_author", "FerrumGrid Studio");
    insert_recent_ui_ko(&mut ko);

    translations.insert(Language::Korean, ko);

    // Japanese
    let mut ja = Translation::new();
    ja.insert("app_title", "FerrumGrid");
    ja.insert("menu_file", "ファイル");
    ja.insert("menu_new_connection", "新規接続");
    ja.insert("menu_close_window", "ウィンドウを閉じる");
    ja.insert("menu_show_main_window", "メインウィンドウを表示");
    ja.insert("menu_quit", "終了");
    ja.insert("menu_query", "クエリ");
    ja.insert("menu_execute", "実行");
    ja.insert("menu_new_tab", "新規タブ");
    ja.insert("menu_view", "表示");
    ja.insert("menu_light_mode", "ライトモード");
    ja.insert("menu_dark_mode", "ダークモード");
    ja.insert("menu_er_diagram", "ER図");
    ja.insert("menu_table_designer", "テーブルデザイナー");
    ja.insert("menu_prisma", "Prisma連携");
    ja.insert("menu_language", "言語");
    ja.insert("explorer_title", "エクスプローラー");
    ja.insert("explorer_new", "新規");
    ja.insert("status_connected", "接続済み");
    ja.insert("status_disconnected", "未接続");
    ja.insert("status_connecting", "接続中...");
    ja.insert("connection_dialog_title", "新規接続");
    ja.insert("connection_name", "名前");
    ja.insert("connection_host", "ホスト");
    ja.insert("connection_port", "ポート");
    ja.insert("connection_database", "データベース");
    ja.insert("connection_username", "ユーザー名");
    ja.insert("connection_password", "パスワード");
    ja.insert("connection_use_tls", "TLSを使用");
    ja.insert("connection_test", "接続テスト");
    ja.insert("connection_save", "保存");
    ja.insert("connection_cancel", "キャンセル");
    ja.insert("query_editor_placeholder", "SQLクエリを入力してください...");
    ja.insert("query_execute", "実行");
    ja.insert("query_cancel", "キャンセル");
    ja.insert("result_rows", "行");
    ja.insert("result_execution_time", "実行時間");
    ja.insert("result_truncated", "結果が切り捨てられました");
    ja.insert("button_refresh", "更新");
    ja.insert("button_delete", "削除");
    ja.insert("button_edit", "編集");
    ja.insert("button_add", "追加");
    ja.insert("button_save", "保存");
    ja.insert("button_cancel", "キャンセル");
    ja.insert("button_ok", "OK");
    ja.insert("button_close", "閉じる");
    ja.insert("button_apply", "適用");
    ja.insert("button_generate", "生成");
    ja.insert("er_diagram_title", "ER図");
    ja.insert("er_schema_select", "スキーマ選択");
    ja.insert("er_load_schema", "スキーマ読込");
    ja.insert("er_auto_layout", "自動レイアウト");
    ja.insert("er_clear", "クリア");
    ja.insert("table_designer_title", "テーブルデザイナー");
    ja.insert("table_designer_new", "テーブル作成");
    ja.insert("table_designer_edit", "テーブル編集");
    ja.insert("table_name", "テーブル名");
    ja.insert("table_schema", "スキーマ");
    ja.insert("column_name", "カラム名");
    ja.insert("column_type", "データ型");
    ja.insert("column_nullable", "NULL許可");
    ja.insert("column_primary_key", "主キー");
    ja.insert("column_unique", "一意");
    ja.insert("column_default", "デフォルト値");
    ja.insert("column_foreign_key", "外部キー");
    ja.insert("index_name", "インデックス名");
    ja.insert("index_columns", "カラム");
    ja.insert("ddl_preview", "DDLプレビュー");
    ja.insert("ddl_copy", "クリップボードにコピー");
    ja.insert("prisma_title", "Prisma連携");
    ja.insert("prisma_schema_path", "スキーマパス");
    ja.insert("prisma_browse", "参照");
    ja.insert("prisma_load", "読込");
    ja.insert("prisma_save", "保存");
    ja.insert("prisma_create_from_db", "DBから生成");
    ja.insert("prisma_introspect", "イントロスペクト");
    ja.insert("prisma_migrate_dev", "開発マイグレーション");
    ja.insert("prisma_migrate_deploy", "デプロイマイグレーション");
    ja.insert("prisma_generate", "クライアント生成");
    ja.insert("prisma_validate", "検証");
    ja.insert("prisma_not_installed", "Prisma CLIが見つかりません");
    ja.insert(
        "prisma_install_help",
        "Prisma CLIをインストールしてください: npm install -g prisma",
    );
    ja.insert("error", "エラー");
    ja.insert("warning", "警告");
    ja.insert("success", "成功");
    ja.insert("info", "情報");
    ja.insert("confirm", "確認");
    ja.insert("loading", "読込中...");
    ja.insert("no_data", "データなし");
    ja.insert("no_connection", "接続なし");
    ja.insert("select_connection", "接続を選択してください");
    translations.insert(Language::Japanese, ja);

    // Chinese (Simplified)
    let mut zh = Translation::new();
    zh.insert("app_title", "FerrumGrid");
    zh.insert("menu_file", "文件");
    zh.insert("menu_new_connection", "新建连接");
    zh.insert("menu_close_window", "关闭窗口");
    zh.insert("menu_show_main_window", "显示主窗口");
    zh.insert("menu_quit", "退出");
    zh.insert("menu_query", "查询");
    zh.insert("menu_execute", "执行");
    zh.insert("menu_new_tab", "新建标签页");
    zh.insert("menu_view", "视图");
    zh.insert("menu_light_mode", "浅色模式");
    zh.insert("menu_dark_mode", "深色模式");
    zh.insert("menu_er_diagram", "ER图");
    zh.insert("menu_table_designer", "表设计器");
    zh.insert("menu_prisma", "Prisma集成");
    zh.insert("menu_language", "语言");
    zh.insert("explorer_title", "资源管理器");
    zh.insert("explorer_new", "新建");
    zh.insert("status_connected", "已连接");
    zh.insert("status_disconnected", "未连接");
    zh.insert("status_connecting", "连接中...");
    zh.insert("connection_dialog_title", "新建连接");
    zh.insert("connection_name", "名称");
    zh.insert("connection_host", "主机");
    zh.insert("connection_port", "端口");
    zh.insert("connection_database", "数据库");
    zh.insert("connection_username", "用户名");
    zh.insert("connection_password", "密码");
    zh.insert("connection_use_tls", "使用TLS");
    zh.insert("connection_test", "测试连接");
    zh.insert("connection_save", "保存");
    zh.insert("connection_cancel", "取消");
    zh.insert("query_editor_placeholder", "在此输入SQL查询...");
    zh.insert("query_execute", "执行");
    zh.insert("query_cancel", "取消");
    zh.insert("result_rows", "行");
    zh.insert("result_execution_time", "执行时间");
    zh.insert("result_truncated", "结果已截断");
    zh.insert("button_refresh", "刷新");
    zh.insert("button_delete", "删除");
    zh.insert("button_edit", "编辑");
    zh.insert("button_add", "添加");
    zh.insert("button_save", "保存");
    zh.insert("button_cancel", "取消");
    zh.insert("button_ok", "确定");
    zh.insert("button_close", "关闭");
    zh.insert("button_apply", "应用");
    zh.insert("button_generate", "生成");
    zh.insert("er_diagram_title", "ER图");
    zh.insert("er_schema_select", "选择模式");
    zh.insert("er_load_schema", "加载模式");
    zh.insert("er_auto_layout", "自动布局");
    zh.insert("er_clear", "清除");
    zh.insert("table_designer_title", "表设计器");
    zh.insert("table_designer_new", "创建表");
    zh.insert("table_designer_edit", "编辑表");
    zh.insert("table_name", "表名");
    zh.insert("table_schema", "模式");
    zh.insert("column_name", "列名");
    zh.insert("column_type", "数据类型");
    zh.insert("column_nullable", "可为空");
    zh.insert("column_primary_key", "主键");
    zh.insert("column_unique", "唯一");
    zh.insert("column_default", "默认值");
    zh.insert("column_foreign_key", "外键");
    zh.insert("index_name", "索引名");
    zh.insert("index_columns", "列");
    zh.insert("ddl_preview", "DDL预览");
    zh.insert("ddl_copy", "复制到剪贴板");
    zh.insert("prisma_title", "Prisma集成");
    zh.insert("prisma_schema_path", "模式路径");
    zh.insert("prisma_browse", "浏览");
    zh.insert("prisma_load", "加载");
    zh.insert("prisma_save", "保存");
    zh.insert("prisma_create_from_db", "从数据库生成");
    zh.insert("prisma_introspect", "内省");
    zh.insert("prisma_migrate_dev", "开发迁移");
    zh.insert("prisma_migrate_deploy", "部署迁移");
    zh.insert("prisma_generate", "生成客户端");
    zh.insert("prisma_validate", "验证");
    zh.insert("prisma_not_installed", "未找到Prisma CLI");
    zh.insert(
        "prisma_install_help",
        "请安装Prisma CLI: npm install -g prisma",
    );
    zh.insert("error", "错误");
    zh.insert("warning", "警告");
    zh.insert("success", "成功");
    zh.insert("info", "信息");
    zh.insert("confirm", "确认");
    zh.insert("loading", "加载中...");
    zh.insert("no_data", "无数据");
    zh.insert("no_connection", "无连接");
    zh.insert("select_connection", "请选择连接");
    translations.insert(Language::ChineseSimplified, zh);

    // Spanish
    let mut es = Translation::new();
    es.insert("app_title", "FerrumGrid");
    es.insert("menu_file", "Archivo");
    es.insert("menu_new_connection", "Nueva Conexión");
    es.insert("menu_close_window", "Cerrar Ventana");
    es.insert("menu_show_main_window", "Mostrar Ventana Principal");
    es.insert("menu_quit", "Salir");
    es.insert("menu_query", "Consulta");
    es.insert("menu_execute", "Ejecutar");
    es.insert("menu_new_tab", "Nueva Pestaña");
    es.insert("menu_view", "Ver");
    es.insert("menu_light_mode", "Modo Claro");
    es.insert("menu_dark_mode", "Modo Oscuro");
    es.insert("menu_er_diagram", "Diagrama ER");
    es.insert("menu_table_designer", "Diseñador de Tablas");
    es.insert("menu_prisma", "Integración Prisma");
    es.insert("menu_language", "Idioma");
    es.insert("explorer_title", "Explorador");
    es.insert("explorer_new", "Nuevo");
    es.insert("status_connected", "Conectado");
    es.insert("status_disconnected", "Desconectado");
    es.insert("status_connecting", "Conectando...");
    translations.insert(Language::Spanish, es);

    // French
    let mut fr = Translation::new();
    fr.insert("app_title", "FerrumGrid");
    fr.insert("menu_file", "Fichier");
    fr.insert("menu_new_connection", "Nouvelle Connexion");
    fr.insert("menu_close_window", "Fermer la fenêtre");
    fr.insert("menu_show_main_window", "Afficher la fenêtre principale");
    fr.insert("menu_quit", "Quitter");
    fr.insert("menu_query", "Requête");
    fr.insert("menu_execute", "Exécuter");
    fr.insert("menu_new_tab", "Nouvel Onglet");
    fr.insert("menu_view", "Affichage");
    fr.insert("menu_light_mode", "Mode Clair");
    fr.insert("menu_dark_mode", "Mode Sombre");
    fr.insert("menu_er_diagram", "Diagramme ER");
    fr.insert("menu_table_designer", "Concepteur de Tables");
    fr.insert("menu_prisma", "Intégration Prisma");
    fr.insert("menu_language", "Langue");
    fr.insert("explorer_title", "Explorateur");
    fr.insert("explorer_new", "Nouveau");
    fr.insert("status_connected", "Connecté");
    fr.insert("status_disconnected", "Déconnecté");
    fr.insert("status_connecting", "Connexion en cours...");
    translations.insert(Language::French, fr);

    // German
    let mut de = Translation::new();
    de.insert("app_title", "FerrumGrid");
    de.insert("menu_file", "Datei");
    de.insert("menu_new_connection", "Neue Verbindung");
    de.insert("menu_close_window", "Fenster schließen");
    de.insert("menu_show_main_window", "Hauptfenster anzeigen");
    de.insert("menu_quit", "Beenden");
    de.insert("menu_query", "Abfrage");
    de.insert("menu_execute", "Ausführen");
    de.insert("menu_new_tab", "Neuer Tab");
    de.insert("menu_view", "Ansicht");
    de.insert("menu_light_mode", "Heller Modus");
    de.insert("menu_dark_mode", "Dunkler Modus");
    de.insert("menu_er_diagram", "ER-Diagramm");
    de.insert("menu_table_designer", "Tabellen-Designer");
    de.insert("menu_prisma", "Prisma-Integration");
    de.insert("menu_language", "Sprache");
    de.insert("explorer_title", "Explorer");
    de.insert("explorer_new", "Neu");
    de.insert("status_connected", "Verbunden");
    de.insert("status_disconnected", "Getrennt");
    de.insert("status_connecting", "Verbinden...");
    translations.insert(Language::German, de);
}

pub fn set_language(lang: Language) {
    let mut current = CURRENT_LANG.write().unwrap();
    *current = lang;
}

pub fn get_language() -> Language {
    *CURRENT_LANG.read().unwrap()
}

pub fn t(key: &str) -> String {
    let lang = get_language();
    let translations = TRANSLATIONS.read().unwrap();

    translations
        .get(&lang)
        .and_then(|t| t.get(key).cloned())
        .or_else(|| {
            translations
                .get(&Language::English)
                .and_then(|t| t.get(key).cloned())
        })
        .unwrap_or_else(|| key.to_string())
}

/// Format a translation with arguments
#[allow(dead_code)]
pub fn tf(key: &str, args: &[&str]) -> String {
    let template = t(key);
    let mut result = template;

    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("{{{}}}", i);
        result = result.replace(&placeholder, arg);
    }

    result
}

/// Initialize the i18n system with a saved language preference
pub fn init_with_saved(saved_lang: Option<&str>) {
    init();

    if let Some(code) = saved_lang {
        set_language(Language::from_code(code));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation() {
        init();

        set_language(Language::English);
        assert_eq!(t("menu_file"), "File");

        set_language(Language::Korean);
        assert_eq!(t("menu_file"), "파일");

        // Fallback to English for unknown keys
        assert_eq!(t("unknown_key"), "unknown_key");
    }

    #[test]
    fn test_format() {
        init();
        let result = tf("Found {0} rows", &["42"]);
        assert_eq!(result, "Found 42 rows");
    }
}
