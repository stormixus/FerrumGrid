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
    en.insert("ctx_compare_schema", "Compare Schema...");
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
    en.insert("tree_copy_table", "Copy Table (Transfer)");
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
    en.insert("objects_rows", "Rows");
    en.insert("objects_no_tables", "No tables found");
    en.insert("objects_no_tables_help", "Try a different schema or search term");
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
    en.insert("grid_add_row", "Add Row");
    en.insert("grid_delete_row", "Delete Row");
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
    en.insert("info_view_connection_title", "Connection");
    en.insert("info_view_connection_none", "Not connected");
    en.insert(
        "info_view_connection_select",
        "Select or open a connection from the tree",
    );
    en.insert("info_view_status", "Status");
    en.insert("info_view_status_connected", "Connected");
    en.insert("info_view_status_disconnected", "Disconnected");
    en.insert("info_view_status_error", "Error");
    en.insert("info_view_host", "Host");
    en.insert("info_view_port", "Port");
    en.insert("info_view_database", "Database");
    en.insert("info_view_user", "User");
    en.insert("info_view_ssl", "SSL");
    en.insert("info_view_schemas_n", "{0} schemas");
    en.insert("info_view_tables_n", "{0} tables");
    en.insert("info_view_views_n", "{0} views");
    en.insert("info_view_matviews_n", "{0} materialized views");
    en.insert("info_view_functions_n", "{0} functions");
    en.insert("info_view_roles_n", "{0} roles");
    en.insert("info_view_objects_title", "Objects");
    en.insert("info_view_schema", "Schema");
    en.insert("info_view_loading", "Loading…");
    en.insert("info_view_no_schema_filter", "All schemas");
    en.insert("info_view_query_title", "Query");
    en.insert("info_view_query_active_tab", "Active tab");
    en.insert("info_view_query_running", "Running…");
    en.insert("info_view_query_idle", "Idle");
    en.insert("info_view_query_last_rows", "Last result rows");
    en.insert("info_view_query_last_cols", "Last result columns");
    en.insert("info_view_query_explicit_tx", "Explicit transaction active");
    en.insert("info_view_query_no_tx", "No active transaction");
    en.insert("info_view_query_chars", "Editor chars");
    en.insert("info_view_query_truncated", "Result truncated");
    en.insert("info_view_query_error", "Last error");
    en.insert("info_view_backup_title", "Backup");
    en.insert("info_view_backup_format", "Format");
    en.insert("info_view_backup_running", "Running");
    en.insert("info_view_backup_idle", "Idle");
    en.insert("info_view_backup_last_error", "Last error");
    en.insert("info_view_backup_history_n", "{0} entries");
    en.insert("info_view_backup_last", "Last backup");
    en.insert("info_view_backup_no_history", "No backup history");
    en.insert("info_view_automation_title", "Automation");
    en.insert("info_view_automation_total", "Registered tasks");
    en.insert("info_view_automation_running", "Active runs");
    en.insert("info_view_automation_draft_ready", "Draft ready: {0}");
    en.insert("info_view_automation_draft_empty", "No draft");
    en.insert("info_view_automation_draft_untitled", "(untitled)");
    en.insert("info_view_model_title", "Model");
    en.insert("info_view_model_no_card", "Click a table card to inspect it");
    en.insert("info_view_model_cards_n", "{0} tables");
    en.insert("info_view_model_visible_n", "{0} visible");
    en.insert("info_view_bi_title", "Business Insights");
    en.insert("info_view_bi_no_result", "Run a Query first to drive BI");
    en.insert("info_view_bi_numeric_cols", "Numeric columns");
    en.insert("info_view_bi_text_cols", "Text columns");
    en.insert("info_view_bi_total_rows", "Total rows");
    en.insert("info_view_function_title", "Functions");
    en.insert("info_view_role_title", "Roles");
    en.insert("info_view_view_title", "Views");
    en.insert("info_view_matview_title", "Materialized Views");
    en.insert("info_view_table_title", "Tables");
    en.insert("info_view_count_in_schema", "{0} in {1}");
    en.insert(
        "info_view_open_data_hint",
        "Open a table to see column-level details",
    );
    en.insert("info_view_diagnostics_title", "Diagnostics");
    en.insert("info_view_diagnostics_pending", "Pending invalidations");
    en.insert("info_view_diagnostics_warned", "Echo timeouts");
    en.insert("transfer_title", "Transfer Tables");
    en.insert("transfer_source", "Source:");
    en.insert("transfer_target", "Target:");
    en.insert("transfer_tables_header", "Tables to transfer (dependency order):");
    en.insert("transfer_select_all", "Select All");
    en.insert("transfer_deselect_all", "Deselect All");
    en.insert("transfer_include_data", "Include data");
    en.insert("transfer_if_exists", "If exists:");
    en.insert("transfer_start", "Transfer");
    en.insert("transfer_cancel", "Cancel");
    en.insert("transfer_not_implemented", "Transfer backend not yet connected");
    en.insert("migration_title", "Schema Migration Wizard");
    en.insert("migration_step_select", "Select");
    en.insert("migration_step_diff", "Diff");
    en.insert("migration_step_sql", "SQL");
    en.insert("migration_source_conn", "Source:");
    en.insert("migration_source_schema", "Schema:");
    en.insert("migration_target_conn", "Target:");
    en.insert("migration_target_schema", "Schema:");
    en.insert("migration_compare", "Compare");
    en.insert("migration_comparing", "Comparing schemas...");
    en.insert("migration_no_diff", "No diff available");
    en.insert("migration_no_changes", "Schemas are identical — no changes needed");
    en.insert("migration_tables_added", "tables added");
    en.insert("migration_tables_modified", "tables modified");
    en.insert("migration_tables_removed", "tables removed");
    en.insert("migration_preview_sql", "Preview SQL");
    en.insert("migration_copy_sql", "Copy SQL");
    en.insert("migration_apply", "Apply to Target");
    en.insert("migration_applying", "Applying migration...");
    en.insert("migration_success", "Migration applied successfully!");
    en.insert("migration_back", "Back");
    en.insert("migration_close", "Close");
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
    ko.insert("ctx_compare_schema", "스키마 비교...");
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
    ko.insert("tree_copy_table", "테이블 복사 (전송)");
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
    ko.insert("objects_rows", "행 수");
    ko.insert("objects_no_tables", "테이블 없음");
    ko.insert("objects_no_tables_help", "다른 스키마나 검색어를 시도하세요");
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
    ko.insert("grid_add_row", "행 추가");
    ko.insert("grid_delete_row", "행 삭제");
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
    ko.insert("info_view_connection_title", "연결");
    ko.insert("info_view_connection_none", "연결되지 않음");
    ko.insert(
        "info_view_connection_select",
        "트리에서 연결을 선택하거나 열어주세요",
    );
    ko.insert("info_view_status", "상태");
    ko.insert("info_view_status_connected", "연결됨");
    ko.insert("info_view_status_disconnected", "연결 끊김");
    ko.insert("info_view_status_error", "오류");
    ko.insert("info_view_host", "호스트");
    ko.insert("info_view_port", "포트");
    ko.insert("info_view_database", "데이터베이스");
    ko.insert("info_view_user", "사용자");
    ko.insert("info_view_ssl", "SSL");
    ko.insert("info_view_schemas_n", "스키마 {0}개");
    ko.insert("info_view_tables_n", "테이블 {0}개");
    ko.insert("info_view_views_n", "뷰 {0}개");
    ko.insert("info_view_matviews_n", "구체화 뷰 {0}개");
    ko.insert("info_view_functions_n", "함수 {0}개");
    ko.insert("info_view_roles_n", "역할 {0}개");
    ko.insert("info_view_objects_title", "객체");
    ko.insert("info_view_schema", "스키마");
    ko.insert("info_view_loading", "로딩 중…");
    ko.insert("info_view_no_schema_filter", "모든 스키마");
    ko.insert("info_view_query_title", "쿼리");
    ko.insert("info_view_query_active_tab", "활성 탭");
    ko.insert("info_view_query_running", "실행 중…");
    ko.insert("info_view_query_idle", "대기");
    ko.insert("info_view_query_last_rows", "마지막 결과 행");
    ko.insert("info_view_query_last_cols", "마지막 결과 열");
    ko.insert("info_view_query_explicit_tx", "명시적 트랜잭션 활성");
    ko.insert("info_view_query_no_tx", "활성 트랜잭션 없음");
    ko.insert("info_view_query_chars", "에디터 글자수");
    ko.insert("info_view_query_truncated", "결과 잘림");
    ko.insert("info_view_query_error", "마지막 오류");
    ko.insert("info_view_backup_title", "백업");
    ko.insert("info_view_backup_format", "포맷");
    ko.insert("info_view_backup_running", "실행 중");
    ko.insert("info_view_backup_idle", "대기");
    ko.insert("info_view_backup_last_error", "마지막 오류");
    ko.insert("info_view_backup_history_n", "{0}개 항목");
    ko.insert("info_view_backup_last", "마지막 백업");
    ko.insert("info_view_backup_no_history", "백업 이력 없음");
    ko.insert("info_view_automation_title", "자동화");
    ko.insert("info_view_automation_total", "등록된 작업");
    ko.insert("info_view_automation_running", "실행 중인 작업");
    ko.insert("info_view_automation_draft_ready", "초안 준비됨: {0}");
    ko.insert("info_view_automation_draft_empty", "초안 없음");
    ko.insert("info_view_automation_draft_untitled", "(제목 없음)");
    ko.insert("info_view_model_title", "모델");
    ko.insert("info_view_model_no_card", "테이블 카드를 클릭해 상세 보기");
    ko.insert("info_view_model_cards_n", "테이블 {0}개");
    ko.insert("info_view_model_visible_n", "보이는 항목 {0}개");
    ko.insert("info_view_bi_title", "비즈니스 인사이트");
    ko.insert("info_view_bi_no_result", "BI를 보려면 먼저 쿼리를 실행하세요");
    ko.insert("info_view_bi_numeric_cols", "숫자 컬럼");
    ko.insert("info_view_bi_text_cols", "텍스트 컬럼");
    ko.insert("info_view_bi_total_rows", "전체 행");
    ko.insert("info_view_function_title", "함수");
    ko.insert("info_view_role_title", "역할");
    ko.insert("info_view_view_title", "뷰");
    ko.insert("info_view_matview_title", "구체화 뷰");
    ko.insert("info_view_table_title", "테이블");
    ko.insert("info_view_count_in_schema", "{1}에 {0}개");
    ko.insert(
        "info_view_open_data_hint",
        "테이블을 열면 컬럼 단위 정보를 볼 수 있어요",
    );
    ko.insert("info_view_diagnostics_title", "진단");
    ko.insert("info_view_diagnostics_pending", "대기 중인 무효화");
    ko.insert("info_view_diagnostics_warned", "에코 타임아웃");
    ko.insert("transfer_title", "테이블 전송");
    ko.insert("transfer_source", "소스:");
    ko.insert("transfer_target", "대상:");
    ko.insert("transfer_tables_header", "전송할 테이블 (의존성 순서):");
    ko.insert("transfer_select_all", "전체 선택");
    ko.insert("transfer_deselect_all", "전체 해제");
    ko.insert("transfer_include_data", "데이터 포함");
    ko.insert("transfer_if_exists", "이미 존재 시:");
    ko.insert("transfer_start", "전송");
    ko.insert("transfer_cancel", "취소");
    ko.insert("transfer_not_implemented", "전송 백엔드가 아직 연결되지 않음");
    ko.insert("migration_title", "스키마 마이그레이션 마법사");
    ko.insert("migration_step_select", "선택");
    ko.insert("migration_step_diff", "비교");
    ko.insert("migration_step_sql", "SQL");
    ko.insert("migration_source_conn", "소스:");
    ko.insert("migration_source_schema", "스키마:");
    ko.insert("migration_target_conn", "대상:");
    ko.insert("migration_target_schema", "스키마:");
    ko.insert("migration_compare", "비교");
    ko.insert("migration_comparing", "스키마 비교 중...");
    ko.insert("migration_no_diff", "비교 결과 없음");
    ko.insert("migration_no_changes", "스키마가 동일합니다 — 변경 불필요");
    ko.insert("migration_tables_added", "테이블 추가");
    ko.insert("migration_tables_modified", "테이블 수정");
    ko.insert("migration_tables_removed", "테이블 삭제");
    ko.insert("migration_preview_sql", "SQL 미리보기");
    ko.insert("migration_copy_sql", "SQL 복사");
    ko.insert("migration_apply", "대상에 적용");
    ko.insert("migration_applying", "마이그레이션 적용 중...");
    ko.insert("migration_success", "마이그레이션 적용 완료!");
    ko.insert("migration_back", "뒤로");
    ko.insert("migration_close", "닫기");
}

fn insert_recent_ui_ja(ja: &mut Translation) {
    ja.insert("ctx_close_connection", "接続を閉じる");
    ja.insert("ctx_open_connection", "接続を開く");
    ja.insert("ctx_switch_connection_profile", "接続プロファイルの切り替え");
    ja.insert("ctx_no_saved_profiles", "保存済みプロファイルなし");
    ja.insert("ctx_edit_connection", "接続を編集...");
    ja.insert("ctx_new_connection", "新規接続");
    ja.insert("ctx_delete_connection", "接続を削除");
    ja.insert("ctx_duplicate_connection", "接続を複製...");
    ja.insert("ctx_new_database", "新規データベース...");
    ja.insert("ctx_new_table", "新規テーブル");
    ja.insert("ctx_new_query", "新規クエリ");
    ja.insert("ctx_console", "コンソール");
    ja.insert("ctx_execute_sql_file", "SQLファイルを実行...");
    ja.insert("ctx_open_schema", "スキーマを開く");
    ja.insert("ctx_backup_schema", "{0} をバックアップ");
    ja.insert("ctx_edit_schema", "スキーマを編集...");
    ja.insert("ctx_new_schema", "新規スキーマ...");
    ja.insert("ctx_delete_schema", "スキーマを削除");
    ja.insert("ctx_dump_sql_file", "SQLファイルをダンプ");
    ja.insert("ctx_data_dictionary", "データ辞書...");
    ja.insert(
        "ctx_reverse_database_to_model",
        "データベースをモデルにリバース...",
    );
    ja.insert("ctx_find_in_database", "データベース内を検索...");
    ja.insert("ctx_add_star", "スターを追加");
    ja.insert("ctx_color", "色:");
    ja.insert("ctx_manage_group", "グループを管理");
    ja.insert("ctx_create_group", "グループを作成...");
    ja.insert("ctx_move_to_group", "グループへ移動...");
    ja.insert("ctx_compare_schema", "スキーマを比較...");
    ja.insert("ctx_share", "共有...");
    ja.insert("ctx_refresh", "更新");
    ja.insert("ctx_close_all_connections", "すべての接続を閉じる");
    ja.insert("ctx_manage_connections", "接続を管理...");
    ja.insert("ctx_new_group", "新規グループ");

    ja.insert("tree_no_connections", "接続なし");
    ja.insert(
        "tree_create_connection",
        "スキーマを閲覧するには接続を作成してください",
    );
    ja.insert("tree_empty", "(空)");
    ja.insert("tree_tables", "テーブル");
    ja.insert("tree_views", "ビュー");
    ja.insert("tree_materialized_views", "マテリアライズドビュー");
    ja.insert("tree_functions", "関数");
    ja.insert("tree_queries", "クエリ");
    ja.insert("tree_backups", "バックアップ");
    ja.insert("tree_schema_backup", "スキーマバックアップ");
    ja.insert("tree_full_database_backup", "フルデータベースバックアップ");
    ja.insert("tree_fields", "フィールド");
    ja.insert("tree_indexes", "インデックス");
    ja.insert("tree_foreign_keys", "外部キー");
    ja.insert("tree_unique", "ユニーク");
    ja.insert("tree_rules", "ルール");
    ja.insert("tree_triggers", "トリガー");
    ja.insert("tree_edit_table", "テーブルを編集");
    ja.insert("tree_view_data_top_100", "データを表示（上位100件）");
    ja.insert("tree_copy_select", "SELECT * をコピー");
    ja.insert("tree_copy_table", "テーブルをコピー（転送）");
    ja.insert("tree_refresh_metadata", "メタデータを更新");
    ja.insert("tree_copy_signature", "シグネチャをコピー");
    ja.insert("tree_copy_rule_ddl", "ルールDDLをコピー");
    ja.insert("tree_copy_trigger_ddl", "トリガーDDLをコピー");
    ja.insert("tree_show_functions", "関数を表示");
    ja.insert("tree_show_group", "{0} を表示");
    ja.insert("tree_showing_group", "{1} の {0} を表示中");
    ja.insert("tree_showing_functions", "{0} の関数を表示中");
    ja.insert("tree_backup_schema_title", "バックアップ: {0}");
    ja.insert("tree_backup_full_title", "バックアップ: フル");
    ja.insert("tree_backup_scope_schema", "バックアップ範囲: {0} スキーマ");
    ja.insert("tree_backup_scope_full", "バックアップ範囲: フルデータベース");
    ja.insert("tree_refreshing_connections", "{0} 件の接続を更新中...");
    ja.insert("tree_explorer_refreshed", "エクスプローラーを更新しました");
    ja.insert("tree_closing_all_connections", "すべての接続を閉じています...");

    ja.insert("objects_all_schemas", "すべてのスキーマ");
    ja.insert("objects_schema", "スキーマ");
    ja.insert("objects_name", "名前");
    ja.insert("objects_type", "タイプ");
    ja.insert("objects_rows", "行数");
    ja.insert("objects_no_tables", "テーブルが見つかりません");
    ja.insert("objects_no_tables_help", "別のスキーマまたは検索語をお試しください");
    ja.insert("objects_columns", "カラム");
    ja.insert("objects_indexes", "インデックス");
    ja.insert("objects_actions", "操作");
    ja.insert("objects_search", "検索");
    ja.insert("objects_new_table", "新規テーブル");
    ja.insert("objects_open_model", "ER図を開く");
    ja.insert("objects_signature", "シグネチャ");
    ja.insert("objects_returns", "戻り値");
    ja.insert("objects_lang", "言語");
    ja.insert("objects_role", "ロール");
    ja.insert("objects_login", "ログイン");
    ja.insert("objects_privileges", "権限");
    ja.insert("objects_valid_until", "有効期限");
    ja.insert("objects_column", "カラム");
    ja.insert("objects_non_null", "NULLなし");
    ja.insert("objects_min", "最小");
    ja.insert("objects_max", "最大");
    ja.insert("objects_average", "平均");
    ja.insert("objects_no_active_connection", "アクティブな接続なし");
    ja.insert(
        "objects_no_active_connection_help",
        "PostgreSQLに接続してデータベースオブジェクトを閲覧・操作してください。",
    );
    ja.insert("objects_tables_title", "テーブル");
    ja.insert("objects_tables_subtitle", "ベーステーブルと編集可能なリレーション");
    ja.insert("objects_views_title", "ビュー");
    ja.insert("objects_views_subtitle", "クエリベースの仮想オブジェクト");
    ja.insert("objects_materialized_title", "マテリアライズドビュー");
    ja.insert("objects_materialized_subtitle", "保存されたクエリスナップショット");
    ja.insert("objects_functions_title", "関数");
    ja.insert("objects_functions_subtitle", "スキーマ別PostgreSQLルーティン");
    ja.insert("objects_users_title", "ユーザー");
    ja.insert("objects_users_subtitle", "ロールとログイン権限");
    ja.insert("objects_backup_title", "バックアップ");
    ja.insert("objects_backup_subtitle", "pg_dumpおよびリストアコマンドビルダー");
    ja.insert("objects_automation_title", "自動化");
    ja.insert("objects_automation_subtitle", "メンテナンスクエリプリセット");
    ja.insert("objects_model_title", "モデル");
    ja.insert("objects_model_subtitle", "ER図とスキーマモデリング");
    ja.insert("objects_bi_title", "BI");
    ja.insert("objects_bi_subtitle", "結果セットのクイックプロファイリング");
    ja.insert("objects_connections_title", "接続");
    ja.insert("objects_connections_subtitle", "データベース接続の設定");
    ja.insert("objects_query_title", "クエリ");
    ja.insert("objects_query_subtitle", "SQLエディター");
    ja.insert("objects_data_title", "データ");
    ja.insert("objects_data_subtitle", "テーブル行の閲覧");

    ja.insert("backup_schema", "スキーマバックアップ");
    ja.insert("backup_full_database", "フルデータベースバックアップ");
    ja.insert(
        "backup_no_folder_selected",
        "バックアップフォルダーが選択されていません",
    );
    ja.insert("backup_folder_title", "FerrumGrid バックアップフォルダー");
    ja.insert("backup_choose_folder", "フォルダーを選択");
    ja.insert("backup_open_folder", "フォルダーを開く");
    ja.insert("backup_folder_updated", "バックアップフォルダーを更新しました");
    ja.insert("backup_format", "フォーマット");
    ja.insert("backup_custom_archive", "カスタムアーカイブ (.dump)");
    ja.insert("backup_plain_sql", "プレーンSQL (.sql)");
    ja.insert("backup_running_label", "バックアップ中...");
    ja.insert("backup_running_status", "{0} をバックアップ中...");
    ja.insert("backup_run", "バックアップ実行");
    ja.insert("backup_pg_dump_running", "pg_dump 実行中");
    ja.insert("backup_tar_archive", "Tarアーカイブ");
    ja.insert("backup_recent", "最近の FerrumGrid バックアップ");
    ja.insert("backup_no_session", "このセッションにバックアップはありません");
    ja.insert("backup_files_title_count", "バックアップファイル ({0})");
    ja.insert("backup_files_title", "バックアップファイル");
    ja.insert("backup_files_refresh", "更新");
    ja.insert(
        "backup_files_set_folder",
        "ファイルを閲覧するにはバックアップフォルダーを設定してください",
    );
    ja.insert("backup_files_empty", "バックアップファイルが見つかりません");
    ja.insert("backup_files_col_name", "名前");
    ja.insert("backup_files_col_size", "サイズ");
    ja.insert("backup_files_col_created", "作成日時");
    ja.insert("backup_files_col_modified", "更新日時");
    ja.insert("backup_files_col_actions", "操作");
    ja.insert("backup_files_show", "表示");
    ja.insert("backup_files_delete_confirm", "削除しますか?");
    ja.insert("backup_files_yes", "はい");
    ja.insert("backup_files_no", "いいえ");
    ja.insert("backup_files_delete", "削除");

    ja.insert("schema_visualizer_title", "スキーマビジュアライザー");
    ja.insert(
        "schema_visualizer_desc",
        "テーブル、カラム、外部キーの関係を視覚的に探索します。",
    );
    ja.insert("schema_visualizer_open", "ビジュアライザーを開く");
    ja.insert("visualizer_schema", "スキーマ");
    ja.insert("visualizer_search_hint", "テーブルまたはカラムを検索");
    ja.insert("visualizer_reload", "再読み込み");
    ja.insert("visualizer_auto_layout", "自動レイアウト");
    ja.insert("visualizer_fit", "フィット");
    ja.insert("visualizer_zoom", "ズーム");
    ja.insert("visualizer_close_tooltip", "スキーマビジュアライザーを閉じる");
    ja.insert("visualizer_loading_columns", "カラムを読み込み中...");
    ja.insert(
        "visualizer_loading_title",
        "スキーマビジュアライザーを読み込み中...",
    );
    ja.insert(
        "visualizer_loading_subtitle",
        "テーブル、カラム、リレーションがここに自動的に表示されます。",
    );
    ja.insert("visualizer_no_matching_tables", "一致するテーブルなし");
    ja.insert(
        "visualizer_clear_search_hint",
        "スキーマ全体を表示するには検索をクリアしてください。",
    );
    ja.insert("visualizer_no_tables_title", "このスキーマにテーブルはありません");
    ja.insert(
        "visualizer_no_tables_subtitle",
        "別のスキーマを選択するか更新してください。",
    );
    ja.insert("visualizer_more_columns", "+{0} 件のカラム");
    ja.insert("visualizer_count", "{0} テーブル  |  {1} リレーション");

    ja.insert("workspace_close_tab", "タブを閉じる");
    ja.insert("workspace_new_query", "新規クエリ");
    ja.insert("grid_revert", "元に戻す");
    ja.insert("grid_edits", "{0} 件の編集");
    ja.insert("grid_pk_required", "行を更新するには主キーが必要です");
    ja.insert("grid_invalid_values", "無効な値が {0} 件あります");
    ja.insert("grid_toggle_null", "NULLを切り替え");
    ja.insert("grid_null_value", "NULL値");
    ja.insert("grid_copy_value", "値をコピー");
    ja.insert("grid_copy_sql", "SQLをコピー");
    ja.insert("grid_no_active_data_source", "アクティブなデータソースなし");
    ja.insert("grid_no_result_set", "結果セットなし");
    ja.insert("grid_column_missing", "編集したカラムが見つかりません");
    ja.insert("grid_pk_missing", "主キーカラム {0} が結果セットにありません");
    ja.insert("grid_pk_value_missing", "主キーの値が取得できません");
    ja.insert("grid_not_null", "このカラムはNULLを許可しません");
    ja.insert("grid_bool_error", "trueまたはfalseを入力してください");
    ja.insert("grid_number_error", "有効な数値を入力してください");
    ja.insert("grid_json_error", "有効なJSONを入力してください");
    ja.insert("grid_uuid_error", "有効なUUIDを入力してください");
    ja.insert(
        "grid_bytes_error",
        "16進バイトを入力してください（例: \\xDEADBEEF）",
    );
    ja.insert("grid_date_error", "日付をYYYY-MM-DD形式で入力してください");
    ja.insert(
        "grid_datetime_error",
        "日時をYYYY-MM-DD HH:MM:SS形式で入力してください",
    );
    ja.insert("grid_now", "現在");
    ja.insert("grid_pick_date", "日付を選択");
    ja.insert("grid_pick_time", "時刻を選択");
    ja.insert("grid_prev_month", "前の月");
    ja.insert("grid_next_month", "次の月");
    ja.insert("grid_hour", "時");
    ja.insert("grid_minute", "分");
    ja.insert("grid_second", "秒");
    ja.insert("grid_weekday_mon", "月");
    ja.insert("grid_weekday_tue", "火");
    ja.insert("grid_weekday_wed", "水");
    ja.insert("grid_weekday_thu", "木");
    ja.insert("grid_weekday_fri", "金");
    ja.insert("grid_weekday_sat", "土");
    ja.insert("grid_weekday_sun", "日");
    ja.insert("grid_sort_asc", "昇順で並べ替え");
    ja.insert("grid_sort_desc", "降順で並べ替え");
    ja.insert("grid_sort_remove", "並べ替えを解除");
    ja.insert("grid_sort_clear_all", "すべての並べ替えをクリア");
    ja.insert(
        "grid_sort_unsaved",
        "並べ替える前に編集を適用または元に戻してください",
    );
    ja.insert(
        "grid_page_unsaved",
        "ページを変更する前に編集を適用または元に戻してください",
    );
    ja.insert("grid_first_page", "最初のページ");
    ja.insert("grid_prev_page", "前のページ");
    ja.insert("grid_next_page", "次のページ");
    ja.insert("grid_page", "ページ");
    ja.insert("grid_page_n", "{0} ページ");
    ja.insert("grid_limit", "制限");
    ja.insert("grid_limit_n", "制限 {0}");
    ja.insert("grid_limit_error", "有効な行制限数を入力してください");
    ja.insert("grid_enum_select", "値を選択");
    ja.insert("grid_enum_error", "許可された値のいずれかを選択してください");
    ja.insert("grid_visible_range", "{0}-{1}");
    ja.insert("data_info_no_selection", "情報なし");
    ja.insert("data_info_select_cell", "行を選択してください");
    ja.insert("data_info_cell", "選択したセル");
    ja.insert("data_info_row", "選択した行");
    ja.insert("data_info_table", "選択したテーブル");
    ja.insert("data_info_row_n", "{0} 行目");
    ja.insert("data_info_col_n", "{0} 列目");
    ja.insert("data_info_columns", "カラム");
    ja.insert("data_info_columns_n", "{0} カラム");
    ja.insert("data_info_indexes_n", "{0} インデックス");
    ja.insert("data_info_relations_n", "{0} リレーション");
    ja.insert("data_info_rules_n", "{0} ルール");
    ja.insert("data_info_triggers_n", "{0} トリガー");
    ja.insert("data_info_active_filter", "適用中のフィルター");
    ja.insert("data_info_relation_out", "出力");
    ja.insert("data_info_relation_in", "入力");
    ja.insert("data_info_selected", "選択済み");
    ja.insert("data_info_nullable", "NULL許可");
    ja.insert("data_info_value", "値");
    ja.insert("data_info_original", "元の値");
    ja.insert("data_info_revert_cell", "セルを元に戻す");
    ja.insert("data_info_dirty", "このセルには未保存の変更があります");
    ja.insert("data_info_yes", "はい");
    ja.insert("data_info_no", "いいえ");
    ja.insert("data_info_read_only", "ここでは読み取り専用の値です。");
    ja.insert("data_relation_open", "関連する行を開く");
    ja.insert("data_info_read_only_pk", "主キーの値はここでは読み取り専用です。");
    ja.insert(
        "data_info_no_metadata",
        "カラムのメタデータを読み込み中のため、編集が無効になっています。",
    );
    ja.insert("transfer_title", "テーブル転送");
    ja.insert("transfer_source", "ソース:");
    ja.insert("transfer_target", "ターゲット:");
    ja.insert("transfer_tables_header", "転送するテーブル（依存順）:");
    ja.insert("transfer_select_all", "全選択");
    ja.insert("transfer_deselect_all", "全解除");
    ja.insert("transfer_include_data", "データを含む");
    ja.insert("transfer_if_exists", "既存の場合:");
    ja.insert("transfer_start", "転送");
    ja.insert("transfer_cancel", "キャンセル");
    ja.insert("transfer_not_implemented", "転送バックエンドはまだ接続されていません");
    ja.insert("migration_title", "スキーマ移行ウィザード");
    ja.insert("migration_step_select", "選択");
    ja.insert("migration_step_diff", "差分");
    ja.insert("migration_step_sql", "SQL");
    ja.insert("migration_source_conn", "ソース:");
    ja.insert("migration_source_schema", "スキーマ:");
    ja.insert("migration_target_conn", "ターゲット:");
    ja.insert("migration_target_schema", "スキーマ:");
    ja.insert("migration_compare", "比較");
    ja.insert("migration_comparing", "スキーマを比較中...");
    ja.insert("migration_no_diff", "差分結果なし");
    ja.insert("migration_no_changes", "スキーマは同一です。変更は不要です");
    ja.insert("migration_tables_added", "テーブル追加");
    ja.insert("migration_tables_modified", "テーブル変更");
    ja.insert("migration_tables_removed", "テーブル削除");
    ja.insert("migration_preview_sql", "SQLプレビュー");
    ja.insert("migration_copy_sql", "SQLコピー");
    ja.insert("migration_apply", "ターゲットに適用");
    ja.insert("migration_applying", "移行を適用中...");
    ja.insert("migration_success", "移行が正常に適用されました!");
    ja.insert("migration_back", "戻る");
    ja.insert("migration_close", "閉じる");
}

fn insert_recent_ui_zh(zh: &mut Translation) {
    zh.insert("ctx_close_connection", "关闭连接");
    zh.insert("ctx_open_connection", "打开连接");
    zh.insert("ctx_switch_connection_profile", "切换连接配置文件");
    zh.insert("ctx_no_saved_profiles", "无已保存的配置文件");
    zh.insert("ctx_edit_connection", "编辑连接...");
    zh.insert("ctx_new_connection", "新建连接");
    zh.insert("ctx_delete_connection", "删除连接");
    zh.insert("ctx_duplicate_connection", "复制连接...");
    zh.insert("ctx_new_database", "新建数据库...");
    zh.insert("ctx_new_table", "新建表");
    zh.insert("ctx_new_query", "新建查询");
    zh.insert("ctx_console", "控制台");
    zh.insert("ctx_execute_sql_file", "执行SQL文件...");
    zh.insert("ctx_open_schema", "打开模式");
    zh.insert("ctx_backup_schema", "备份 {0}");
    zh.insert("ctx_edit_schema", "编辑模式...");
    zh.insert("ctx_new_schema", "新建模式...");
    zh.insert("ctx_delete_schema", "删除模式");
    zh.insert("ctx_dump_sql_file", "导出SQL文件");
    zh.insert("ctx_data_dictionary", "数据字典...");
    zh.insert("ctx_reverse_database_to_model", "将数据库逆向为模型...");
    zh.insert("ctx_find_in_database", "在数据库中查找...");
    zh.insert("ctx_add_star", "添加收藏");
    zh.insert("ctx_color", "颜色:");
    zh.insert("ctx_manage_group", "管理分组");
    zh.insert("ctx_create_group", "创建分组...");
    zh.insert("ctx_move_to_group", "移动到分组...");
    zh.insert("ctx_compare_schema", "比较架构...");
    zh.insert("ctx_share", "共享...");
    zh.insert("ctx_refresh", "刷新");
    zh.insert("ctx_close_all_connections", "关闭所有连接");
    zh.insert("ctx_manage_connections", "管理连接...");
    zh.insert("ctx_new_group", "新建分组");

    zh.insert("tree_no_connections", "无连接");
    zh.insert("tree_create_connection", "创建连接以浏览模式");
    zh.insert("tree_empty", "（空）");
    zh.insert("tree_tables", "表");
    zh.insert("tree_views", "视图");
    zh.insert("tree_materialized_views", "物化视图");
    zh.insert("tree_functions", "函数");
    zh.insert("tree_queries", "查询");
    zh.insert("tree_backups", "备份");
    zh.insert("tree_schema_backup", "模式备份");
    zh.insert("tree_full_database_backup", "全库备份");
    zh.insert("tree_fields", "字段");
    zh.insert("tree_indexes", "索引");
    zh.insert("tree_foreign_keys", "外键");
    zh.insert("tree_unique", "唯一");
    zh.insert("tree_rules", "规则");
    zh.insert("tree_triggers", "触发器");
    zh.insert("tree_edit_table", "编辑表");
    zh.insert("tree_view_data_top_100", "查看数据（前100条）");
    zh.insert("tree_copy_select", "复制 SELECT *");
    zh.insert("tree_copy_table", "复制表（传输）");
    zh.insert("tree_refresh_metadata", "刷新元数据");
    zh.insert("tree_copy_signature", "复制签名");
    zh.insert("tree_copy_rule_ddl", "复制规则DDL");
    zh.insert("tree_copy_trigger_ddl", "复制触发器DDL");
    zh.insert("tree_show_functions", "显示函数");
    zh.insert("tree_show_group", "显示 {0}");
    zh.insert("tree_showing_group", "正在显示 {1} 中的 {0}");
    zh.insert("tree_showing_functions", "正在显示 {0} 中的函数");
    zh.insert("tree_backup_schema_title", "备份: {0}");
    zh.insert("tree_backup_full_title", "备份: 全库");
    zh.insert("tree_backup_scope_schema", "备份范围: {0} 模式");
    zh.insert("tree_backup_scope_full", "备份范围: 全库");
    zh.insert("tree_refreshing_connections", "正在刷新 {0} 个连接...");
    zh.insert("tree_explorer_refreshed", "资源管理器已刷新");
    zh.insert("tree_closing_all_connections", "正在关闭所有连接...");

    zh.insert("objects_all_schemas", "所有模式");
    zh.insert("objects_schema", "模式");
    zh.insert("objects_name", "名称");
    zh.insert("objects_type", "类型");
    zh.insert("objects_rows", "行数");
    zh.insert("objects_no_tables", "未找到表");
    zh.insert("objects_no_tables_help", "尝试其他架构或搜索词");
    zh.insert("objects_columns", "列");
    zh.insert("objects_indexes", "索引");
    zh.insert("objects_actions", "操作");
    zh.insert("objects_search", "搜索");
    zh.insert("objects_new_table", "新建表");
    zh.insert("objects_open_model", "打开ER图");
    zh.insert("objects_signature", "签名");
    zh.insert("objects_returns", "返回值");
    zh.insert("objects_lang", "语言");
    zh.insert("objects_role", "角色");
    zh.insert("objects_login", "登录");
    zh.insert("objects_privileges", "权限");
    zh.insert("objects_valid_until", "有效期至");
    zh.insert("objects_column", "列");
    zh.insert("objects_non_null", "非空");
    zh.insert("objects_min", "最小值");
    zh.insert("objects_max", "最大值");
    zh.insert("objects_average", "平均值");
    zh.insert("objects_no_active_connection", "无活跃连接");
    zh.insert(
        "objects_no_active_connection_help",
        "连接到PostgreSQL以浏览和操作数据库对象。",
    );
    zh.insert("objects_tables_title", "表");
    zh.insert("objects_tables_subtitle", "基础表和可编辑关系");
    zh.insert("objects_views_title", "视图");
    zh.insert("objects_views_subtitle", "基于查询的虚拟对象");
    zh.insert("objects_materialized_title", "物化视图");
    zh.insert("objects_materialized_subtitle", "存储的查询快照");
    zh.insert("objects_functions_title", "函数");
    zh.insert("objects_functions_subtitle", "按模式划分的PostgreSQL例程");
    zh.insert("objects_users_title", "用户");
    zh.insert("objects_users_subtitle", "角色和登录权限");
    zh.insert("objects_backup_title", "备份");
    zh.insert("objects_backup_subtitle", "pg_dump和恢复命令生成器");
    zh.insert("objects_automation_title", "自动化");
    zh.insert("objects_automation_subtitle", "维护查询预设");
    zh.insert("objects_model_title", "模型");
    zh.insert("objects_model_subtitle", "ER图和模式建模");
    zh.insert("objects_bi_title", "BI");
    zh.insert("objects_bi_subtitle", "快速结果集分析");
    zh.insert("objects_connections_title", "连接");
    zh.insert("objects_connections_subtitle", "数据库连接配置");
    zh.insert("objects_query_title", "查询");
    zh.insert("objects_query_subtitle", "SQL编辑器");
    zh.insert("objects_data_title", "数据");
    zh.insert("objects_data_subtitle", "浏览表数据行");

    zh.insert("backup_schema", "模式备份");
    zh.insert("backup_full_database", "全库备份");
    zh.insert("backup_no_folder_selected", "未选择备份文件夹");
    zh.insert("backup_folder_title", "FerrumGrid 备份文件夹");
    zh.insert("backup_choose_folder", "选择文件夹");
    zh.insert("backup_open_folder", "打开文件夹");
    zh.insert("backup_folder_updated", "备份文件夹已更新");
    zh.insert("backup_format", "格式");
    zh.insert("backup_custom_archive", "自定义归档 (.dump)");
    zh.insert("backup_plain_sql", "纯SQL (.sql)");
    zh.insert("backup_running_label", "备份中...");
    zh.insert("backup_running_status", "正在备份 {0}...");
    zh.insert("backup_run", "运行备份");
    zh.insert("backup_pg_dump_running", "pg_dump 运行中");
    zh.insert("backup_tar_archive", "Tar归档");
    zh.insert("backup_recent", "最近的 FerrumGrid 备份");
    zh.insert("backup_no_session", "本次会话中无备份");
    zh.insert("backup_files_title_count", "备份文件 ({0})");
    zh.insert("backup_files_title", "备份文件");
    zh.insert("backup_files_refresh", "刷新");
    zh.insert("backup_files_set_folder", "请设置备份文件夹以浏览文件");
    zh.insert("backup_files_empty", "未找到备份文件");
    zh.insert("backup_files_col_name", "名称");
    zh.insert("backup_files_col_size", "大小");
    zh.insert("backup_files_col_created", "创建时间");
    zh.insert("backup_files_col_modified", "修改时间");
    zh.insert("backup_files_col_actions", "操作");
    zh.insert("backup_files_show", "显示");
    zh.insert("backup_files_delete_confirm", "确认删除?");
    zh.insert("backup_files_yes", "是");
    zh.insert("backup_files_no", "否");
    zh.insert("backup_files_delete", "删除");

    zh.insert("schema_visualizer_title", "模式可视化器");
    zh.insert("schema_visualizer_desc", "可视化探索表、列和外键关系。");
    zh.insert("schema_visualizer_open", "打开可视化器");
    zh.insert("visualizer_schema", "模式");
    zh.insert("visualizer_search_hint", "搜索表或列");
    zh.insert("visualizer_reload", "重新加载");
    zh.insert("visualizer_auto_layout", "自动布局");
    zh.insert("visualizer_fit", "适配");
    zh.insert("visualizer_zoom", "缩放");
    zh.insert("visualizer_close_tooltip", "关闭模式可视化器");
    zh.insert("visualizer_loading_columns", "正在加载列...");
    zh.insert("visualizer_loading_title", "正在加载模式可视化器...");
    zh.insert(
        "visualizer_loading_subtitle",
        "表、列和关系将自动显示在这里。",
    );
    zh.insert("visualizer_no_matching_tables", "无匹配的表");
    zh.insert("visualizer_clear_search_hint", "清除搜索框以显示完整模式。");
    zh.insert("visualizer_no_tables_title", "此模式中没有表");
    zh.insert("visualizer_no_tables_subtitle", "请选择其他模式或刷新。");
    zh.insert("visualizer_more_columns", "+{0} 列");
    zh.insert("visualizer_count", "{0} 个表  |  {1} 个关系");

    zh.insert("workspace_close_tab", "关闭标签页");
    zh.insert("workspace_new_query", "新建查询");
    zh.insert("grid_revert", "还原");
    zh.insert("grid_edits", "{0} 处编辑");
    zh.insert("grid_pk_required", "更新行需要主键");
    zh.insert("grid_invalid_values", "{0} 个无效值");
    zh.insert("grid_toggle_null", "切换NULL");
    zh.insert("grid_null_value", "NULL值");
    zh.insert("grid_copy_value", "复制值");
    zh.insert("grid_copy_sql", "复制SQL");
    zh.insert("grid_no_active_data_source", "无活跃数据源");
    zh.insert("grid_no_result_set", "无结果集");
    zh.insert("grid_column_missing", "已编辑的列不再可用");
    zh.insert("grid_pk_missing", "主键列 {0} 不在结果集中");
    zh.insert("grid_pk_value_missing", "主键值不可用");
    zh.insert("grid_not_null", "此列不允许NULL");
    zh.insert("grid_bool_error", "请输入 true 或 false");
    zh.insert("grid_number_error", "请输入有效的数字");
    zh.insert("grid_json_error", "请输入有效的JSON");
    zh.insert("grid_uuid_error", "请输入有效的UUID");
    zh.insert(
        "grid_bytes_error",
        "请输入十六进制字节，例如 \\xDEADBEEF",
    );
    zh.insert("grid_date_error", "请以YYYY-MM-DD格式输入日期");
    zh.insert(
        "grid_datetime_error",
        "请以YYYY-MM-DD HH:MM:SS格式输入日期和时间",
    );
    zh.insert("grid_now", "现在");
    zh.insert("grid_pick_date", "选择日期");
    zh.insert("grid_pick_time", "选择时间");
    zh.insert("grid_prev_month", "上个月");
    zh.insert("grid_next_month", "下个月");
    zh.insert("grid_hour", "时");
    zh.insert("grid_minute", "分");
    zh.insert("grid_second", "秒");
    zh.insert("grid_weekday_mon", "一");
    zh.insert("grid_weekday_tue", "二");
    zh.insert("grid_weekday_wed", "三");
    zh.insert("grid_weekday_thu", "四");
    zh.insert("grid_weekday_fri", "五");
    zh.insert("grid_weekday_sat", "六");
    zh.insert("grid_weekday_sun", "日");
    zh.insert("grid_sort_asc", "升序排列");
    zh.insert("grid_sort_desc", "降序排列");
    zh.insert("grid_sort_remove", "取消排序");
    zh.insert("grid_sort_clear_all", "清除所有排序");
    zh.insert("grid_sort_unsaved", "排序前请先应用或还原编辑");
    zh.insert("grid_page_unsaved", "翻页前请先应用或还原编辑");
    zh.insert("grid_first_page", "第一页");
    zh.insert("grid_prev_page", "上一页");
    zh.insert("grid_next_page", "下一页");
    zh.insert("grid_page", "页");
    zh.insert("grid_page_n", "第 {0} 页");
    zh.insert("grid_limit", "限制");
    zh.insert("grid_limit_n", "限制 {0}");
    zh.insert("grid_limit_error", "请输入有效的行数限制");
    zh.insert("grid_enum_select", "选择值");
    zh.insert("grid_enum_error", "请选择允许的值之一");
    zh.insert("grid_visible_range", "{0}-{1}");
    zh.insert("data_info_no_selection", "无信息");
    zh.insert("data_info_select_cell", "请选择一行");
    zh.insert("data_info_cell", "选中的单元格");
    zh.insert("data_info_row", "选中的行");
    zh.insert("data_info_table", "选中的表");
    zh.insert("data_info_row_n", "第 {0} 行");
    zh.insert("data_info_col_n", "第 {0} 列");
    zh.insert("data_info_columns", "列");
    zh.insert("data_info_columns_n", "{0} 列");
    zh.insert("data_info_indexes_n", "{0} 个索引");
    zh.insert("data_info_relations_n", "{0} 个关系");
    zh.insert("data_info_rules_n", "{0} 个规则");
    zh.insert("data_info_triggers_n", "{0} 个触发器");
    zh.insert("data_info_active_filter", "活跃筛选器");
    zh.insert("data_info_relation_out", "出");
    zh.insert("data_info_relation_in", "入");
    zh.insert("data_info_selected", "已选中");
    zh.insert("data_info_nullable", "可为空");
    zh.insert("data_info_value", "值");
    zh.insert("data_info_original", "原始值");
    zh.insert("data_info_revert_cell", "还原单元格");
    zh.insert("data_info_dirty", "此单元格有未保存的更改");
    zh.insert("data_info_yes", "是");
    zh.insert("data_info_no", "否");
    zh.insert("data_info_read_only", "此值在这里是只读的。");
    zh.insert("data_relation_open", "打开关联行");
    zh.insert("data_info_read_only_pk", "主键值在这里是只读的。");
    zh.insert(
        "data_info_no_metadata",
        "列元数据仍在加载中，因此编辑已禁用。",
    );
    zh.insert("transfer_title", "传输表");
    zh.insert("transfer_source", "来源:");
    zh.insert("transfer_target", "目标:");
    zh.insert("transfer_tables_header", "要传输的表（依赖顺序）:");
    zh.insert("transfer_select_all", "全选");
    zh.insert("transfer_deselect_all", "全部取消");
    zh.insert("transfer_include_data", "包含数据");
    zh.insert("transfer_if_exists", "如果已存在:");
    zh.insert("transfer_start", "传输");
    zh.insert("transfer_cancel", "取消");
    zh.insert("transfer_not_implemented", "传输后端尚未连接");
    zh.insert("migration_title", "架构迁移向导");
    zh.insert("migration_step_select", "选择");
    zh.insert("migration_step_diff", "差异");
    zh.insert("migration_step_sql", "SQL");
    zh.insert("migration_source_conn", "来源:");
    zh.insert("migration_source_schema", "架构:");
    zh.insert("migration_target_conn", "目标:");
    zh.insert("migration_target_schema", "架构:");
    zh.insert("migration_compare", "比较");
    zh.insert("migration_comparing", "正在比较架构...");
    zh.insert("migration_no_diff", "无差异结果");
    zh.insert("migration_no_changes", "架构相同，无需更改");
    zh.insert("migration_tables_added", "表已添加");
    zh.insert("migration_tables_modified", "表已修改");
    zh.insert("migration_tables_removed", "表已删除");
    zh.insert("migration_preview_sql", "预览SQL");
    zh.insert("migration_copy_sql", "复制SQL");
    zh.insert("migration_apply", "应用到目标");
    zh.insert("migration_applying", "正在应用迁移...");
    zh.insert("migration_success", "迁移应用成功!");
    zh.insert("migration_back", "返回");
    zh.insert("migration_close", "关闭");
}

fn insert_recent_ui_es(es: &mut Translation) {
    es.insert("ctx_close_connection", "Cerrar conexión");
    es.insert("ctx_open_connection", "Abrir conexión");
    es.insert("ctx_switch_connection_profile", "Cambiar perfil de conexión");
    es.insert("ctx_no_saved_profiles", "Sin perfiles guardados");
    es.insert("ctx_edit_connection", "Editar conexión...");
    es.insert("ctx_new_connection", "Nueva conexión");
    es.insert("ctx_delete_connection", "Eliminar conexión");
    es.insert("ctx_duplicate_connection", "Duplicar conexión...");
    es.insert("ctx_new_database", "Nueva base de datos...");
    es.insert("ctx_new_table", "Nueva tabla");
    es.insert("ctx_new_query", "Nueva consulta");
    es.insert("ctx_console", "Consola");
    es.insert("ctx_execute_sql_file", "Ejecutar archivo SQL...");
    es.insert("ctx_open_schema", "Abrir esquema");
    es.insert("ctx_backup_schema", "Hacer copia de seguridad de {0}");
    es.insert("ctx_edit_schema", "Editar esquema...");
    es.insert("ctx_new_schema", "Nuevo esquema...");
    es.insert("ctx_delete_schema", "Eliminar esquema");
    es.insert("ctx_dump_sql_file", "Exportar archivo SQL");
    es.insert("ctx_data_dictionary", "Diccionario de datos...");
    es.insert(
        "ctx_reverse_database_to_model",
        "Revertir base de datos a modelo...",
    );
    es.insert("ctx_find_in_database", "Buscar en la base de datos...");
    es.insert("ctx_add_star", "Agregar favorito");
    es.insert("ctx_color", "Color:");
    es.insert("ctx_manage_group", "Administrar grupo");
    es.insert("ctx_create_group", "Crear grupo...");
    es.insert("ctx_move_to_group", "Mover a grupo...");
    es.insert("ctx_compare_schema", "Comparar esquema...");
    es.insert("ctx_share", "Compartir...");
    es.insert("ctx_refresh", "Actualizar");
    es.insert("ctx_close_all_connections", "Cerrar todas las conexiones");
    es.insert("ctx_manage_connections", "Administrar conexiones...");
    es.insert("ctx_new_group", "Nuevo grupo");

    es.insert("tree_no_connections", "Sin conexiones");
    es.insert(
        "tree_create_connection",
        "Cree una conexión para explorar esquemas",
    );
    es.insert("tree_empty", "(vacío)");
    es.insert("tree_tables", "Tablas");
    es.insert("tree_views", "Vistas");
    es.insert("tree_materialized_views", "Vistas materializadas");
    es.insert("tree_functions", "Funciones");
    es.insert("tree_queries", "Consultas");
    es.insert("tree_backups", "Copias de seguridad");
    es.insert("tree_schema_backup", "Copia de seguridad del esquema");
    es.insert(
        "tree_full_database_backup",
        "Copia de seguridad completa de la base de datos",
    );
    es.insert("tree_fields", "Campos");
    es.insert("tree_indexes", "Índices");
    es.insert("tree_foreign_keys", "Claves foráneas");
    es.insert("tree_unique", "Único");
    es.insert("tree_rules", "Reglas");
    es.insert("tree_triggers", "Disparadores");
    es.insert("tree_edit_table", "Editar tabla");
    es.insert("tree_view_data_top_100", "Ver datos (top 100)");
    es.insert("tree_copy_select", "Copiar SELECT *");
    es.insert("tree_copy_table", "Copiar tabla (transferir)");
    es.insert("tree_refresh_metadata", "Actualizar metadatos");
    es.insert("tree_copy_signature", "Copiar firma");
    es.insert("tree_copy_rule_ddl", "Copiar DDL de regla");
    es.insert("tree_copy_trigger_ddl", "Copiar DDL de disparador");
    es.insert("tree_show_functions", "Mostrar funciones");
    es.insert("tree_show_group", "Mostrar {0}");
    es.insert("tree_showing_group", "Mostrando {0} en {1}");
    es.insert("tree_showing_functions", "Mostrando funciones en {0}");
    es.insert("tree_backup_schema_title", "Copia de seguridad: {0}");
    es.insert("tree_backup_full_title", "Copia de seguridad: completa");
    es.insert(
        "tree_backup_scope_schema",
        "Alcance de la copia de seguridad: esquema {0}",
    );
    es.insert(
        "tree_backup_scope_full",
        "Alcance de la copia de seguridad: base de datos completa",
    );
    es.insert("tree_refreshing_connections", "Actualizando {0} conexión(es)...");
    es.insert("tree_explorer_refreshed", "Explorador actualizado");
    es.insert("tree_closing_all_connections", "Cerrando todas las conexiones...");

    es.insert("objects_all_schemas", "Todos los esquemas");
    es.insert("objects_schema", "Esquema");
    es.insert("objects_name", "Nombre");
    es.insert("objects_type", "Tipo");
    es.insert("objects_rows", "Filas");
    es.insert("objects_no_tables", "No se encontraron tablas");
    es.insert("objects_no_tables_help", "Intente otro esquema o término de búsqueda");
    es.insert("objects_columns", "Columnas");
    es.insert("objects_indexes", "Índices");
    es.insert("objects_actions", "Acciones");
    es.insert("objects_search", "Buscar");
    es.insert("objects_new_table", "Nueva tabla");
    es.insert("objects_open_model", "Abrir diagrama ER");
    es.insert("objects_signature", "Firma");
    es.insert("objects_returns", "Devuelve");
    es.insert("objects_lang", "Idioma");
    es.insert("objects_role", "Rol");
    es.insert("objects_login", "Inicio de sesión");
    es.insert("objects_privileges", "Privilegios");
    es.insert("objects_valid_until", "Válido hasta");
    es.insert("objects_column", "Columna");
    es.insert("objects_non_null", "No nulo");
    es.insert("objects_min", "Mínimo");
    es.insert("objects_max", "Máximo");
    es.insert("objects_average", "Promedio");
    es.insert("objects_no_active_connection", "Sin conexión activa");
    es.insert(
        "objects_no_active_connection_help",
        "Conéctese a PostgreSQL para explorar y operar objetos de base de datos.",
    );
    es.insert("objects_tables_title", "Tablas");
    es.insert("objects_tables_subtitle", "Tablas base y relaciones editables");
    es.insert("objects_views_title", "Vistas");
    es.insert("objects_views_subtitle", "Objetos virtuales basados en consultas");
    es.insert("objects_materialized_title", "Vistas materializadas");
    es.insert(
        "objects_materialized_subtitle",
        "Instantáneas de consultas almacenadas",
    );
    es.insert("objects_functions_title", "Funciones");
    es.insert(
        "objects_functions_subtitle",
        "Rutinas de PostgreSQL por esquema",
    );
    es.insert("objects_users_title", "Usuarios");
    es.insert(
        "objects_users_subtitle",
        "Roles y permisos de inicio de sesión",
    );
    es.insert("objects_backup_title", "Copia de seguridad");
    es.insert(
        "objects_backup_subtitle",
        "Constructor de comandos pg_dump y restauración",
    );
    es.insert("objects_automation_title", "Automatización");
    es.insert(
        "objects_automation_subtitle",
        "Presets de consultas de mantenimiento",
    );
    es.insert("objects_model_title", "Modelo");
    es.insert(
        "objects_model_subtitle",
        "Diagrama ER y modelado de esquemas",
    );
    es.insert("objects_bi_title", "BI");
    es.insert(
        "objects_bi_subtitle",
        "Perfilado rápido de conjuntos de resultados",
    );
    es.insert("objects_connections_title", "Conexiones");
    es.insert(
        "objects_connections_subtitle",
        "Configuración de conexión de base de datos",
    );
    es.insert("objects_query_title", "Consulta");
    es.insert("objects_query_subtitle", "Editor SQL");
    es.insert("objects_data_title", "Datos");
    es.insert("objects_data_subtitle", "Explorar filas de tabla");

    es.insert("backup_schema", "Copia de seguridad del esquema");
    es.insert(
        "backup_full_database",
        "Copia de seguridad completa de la base de datos",
    );
    es.insert(
        "backup_no_folder_selected",
        "No se ha seleccionado carpeta de copia de seguridad",
    );
    es.insert(
        "backup_folder_title",
        "Carpeta de copia de seguridad de FerrumGrid",
    );
    es.insert("backup_choose_folder", "Elegir carpeta");
    es.insert("backup_open_folder", "Abrir carpeta");
    es.insert("backup_folder_updated", "Carpeta de copia de seguridad actualizada");
    es.insert("backup_format", "Formato");
    es.insert("backup_custom_archive", "Archivo personalizado (.dump)");
    es.insert("backup_plain_sql", "SQL plano (.sql)");
    es.insert("backup_running_label", "Realizando copia de seguridad...");
    es.insert(
        "backup_running_status",
        "Realizando copia de seguridad de {0}...",
    );
    es.insert("backup_run", "Ejecutar copia de seguridad");
    es.insert("backup_pg_dump_running", "pg_dump en ejecución");
    es.insert("backup_tar_archive", "Archivo Tar");
    es.insert(
        "backup_recent",
        "Copias de seguridad recientes de FerrumGrid",
    );
    es.insert("backup_no_session", "Sin copias de seguridad en esta sesión");
    es.insert("backup_files_title_count", "Archivos de copia de seguridad ({0})");
    es.insert("backup_files_title", "Archivos de copia de seguridad");
    es.insert("backup_files_refresh", "Actualizar");
    es.insert(
        "backup_files_set_folder",
        "Establezca la carpeta de copia de seguridad para explorar archivos",
    );
    es.insert(
        "backup_files_empty",
        "No se encontraron archivos de copia de seguridad",
    );
    es.insert("backup_files_col_name", "Nombre");
    es.insert("backup_files_col_size", "Tamaño");
    es.insert("backup_files_col_created", "Creado");
    es.insert("backup_files_col_modified", "Modificado");
    es.insert("backup_files_col_actions", "Acciones");
    es.insert("backup_files_show", "Mostrar");
    es.insert("backup_files_delete_confirm", "¿Eliminar?");
    es.insert("backup_files_yes", "Sí");
    es.insert("backup_files_no", "No");
    es.insert("backup_files_delete", "Eliminar");

    es.insert("schema_visualizer_title", "Visualizador de esquemas");
    es.insert(
        "schema_visualizer_desc",
        "Explore tablas, columnas y relaciones de clave foránea.",
    );
    es.insert("schema_visualizer_open", "Abrir visualizador");
    es.insert("visualizer_schema", "Esquema");
    es.insert("visualizer_search_hint", "Buscar tablas o columnas");
    es.insert("visualizer_reload", "Recargar");
    es.insert("visualizer_auto_layout", "Diseño automático");
    es.insert("visualizer_fit", "Ajustar");
    es.insert("visualizer_zoom", "Zoom");
    es.insert("visualizer_close_tooltip", "Cerrar visualizador de esquemas");
    es.insert("visualizer_loading_columns", "Cargando columnas...");
    es.insert(
        "visualizer_loading_title",
        "Cargando visualizador de esquemas...",
    );
    es.insert(
        "visualizer_loading_subtitle",
        "Las tablas, columnas y relaciones aparecerán aquí automáticamente.",
    );
    es.insert("visualizer_no_matching_tables", "Sin tablas coincidentes");
    es.insert(
        "visualizer_clear_search_hint",
        "Limpie el cuadro de búsqueda para mostrar el esquema completo.",
    );
    es.insert("visualizer_no_tables_title", "No hay tablas en este esquema");
    es.insert(
        "visualizer_no_tables_subtitle",
        "Seleccione otro esquema o actualice.",
    );
    es.insert("visualizer_more_columns", "+{0} columnas más");
    es.insert("visualizer_count", "{0} tablas  |  {1} relaciones");

    es.insert("workspace_close_tab", "Cerrar pestaña");
    es.insert("workspace_new_query", "Nueva consulta");
    es.insert("grid_revert", "Revertir");
    es.insert("grid_edits", "{0} ediciones");
    es.insert(
        "grid_pk_required",
        "Se requiere clave primaria para actualizar filas",
    );
    es.insert("grid_invalid_values", "{0} valor(es) inválido(s)");
    es.insert("grid_toggle_null", "Alternar NULL");
    es.insert("grid_null_value", "Valor NULL");
    es.insert("grid_copy_value", "Copiar valor");
    es.insert("grid_copy_sql", "Copiar SQL");
    es.insert("grid_no_active_data_source", "Sin fuente de datos activa");
    es.insert("grid_no_result_set", "Sin conjunto de resultados");
    es.insert("grid_column_missing", "La columna editada ya no está disponible");
    es.insert(
        "grid_pk_missing",
        "La columna de clave primaria {0} no está en el conjunto de resultados",
    );
    es.insert(
        "grid_pk_value_missing",
        "El valor de clave primaria no está disponible",
    );
    es.insert("grid_not_null", "Esta columna no permite NULL");
    es.insert("grid_bool_error", "Use true o false");
    es.insert("grid_number_error", "Ingrese un número válido");
    es.insert("grid_json_error", "Ingrese JSON válido");
    es.insert("grid_uuid_error", "Ingrese un UUID válido");
    es.insert(
        "grid_bytes_error",
        "Ingrese bytes hexadecimales, por ejemplo \\xDEADBEEF",
    );
    es.insert("grid_date_error", "Ingrese una fecha como YYYY-MM-DD");
    es.insert(
        "grid_datetime_error",
        "Ingrese fecha y hora como YYYY-MM-DD HH:MM:SS",
    );
    es.insert("grid_now", "Ahora");
    es.insert("grid_pick_date", "Elegir fecha");
    es.insert("grid_pick_time", "Elegir hora");
    es.insert("grid_prev_month", "Mes anterior");
    es.insert("grid_next_month", "Mes siguiente");
    es.insert("grid_hour", "Hora");
    es.insert("grid_minute", "Min");
    es.insert("grid_second", "Seg");
    es.insert("grid_weekday_mon", "L");
    es.insert("grid_weekday_tue", "M");
    es.insert("grid_weekday_wed", "X");
    es.insert("grid_weekday_thu", "J");
    es.insert("grid_weekday_fri", "V");
    es.insert("grid_weekday_sat", "S");
    es.insert("grid_weekday_sun", "D");
    es.insert("grid_sort_asc", "Ordenar ascendente");
    es.insert("grid_sort_desc", "Ordenar descendente");
    es.insert("grid_sort_remove", "Quitar ordenación");
    es.insert("grid_sort_clear_all", "Limpiar todas las ordenaciones");
    es.insert(
        "grid_sort_unsaved",
        "Aplique o revierta las ediciones antes de ordenar",
    );
    es.insert(
        "grid_page_unsaved",
        "Aplique o revierta las ediciones antes de cambiar de página",
    );
    es.insert("grid_first_page", "Primera página");
    es.insert("grid_prev_page", "Página anterior");
    es.insert("grid_next_page", "Página siguiente");
    es.insert("grid_page", "Página");
    es.insert("grid_page_n", "Página {0}");
    es.insert("grid_limit", "Límite");
    es.insert("grid_limit_n", "Límite {0}");
    es.insert("grid_limit_error", "Ingrese un límite de filas válido");
    es.insert("grid_enum_select", "Seleccionar valor");
    es.insert("grid_enum_error", "Seleccione uno de los valores permitidos");
    es.insert("grid_visible_range", "{0}-{1}");
    es.insert("data_info_no_selection", "Sin información");
    es.insert("data_info_select_cell", "Seleccione una fila");
    es.insert("data_info_cell", "Celda seleccionada");
    es.insert("data_info_row", "Fila seleccionada");
    es.insert("data_info_table", "Tabla seleccionada");
    es.insert("data_info_row_n", "Fila {0}");
    es.insert("data_info_col_n", "Col {0}");
    es.insert("data_info_columns", "Columnas");
    es.insert("data_info_columns_n", "{0} columnas");
    es.insert("data_info_indexes_n", "{0} índices");
    es.insert("data_info_relations_n", "{0} relaciones");
    es.insert("data_info_rules_n", "{0} reglas");
    es.insert("data_info_triggers_n", "{0} disparadores");
    es.insert("data_info_active_filter", "Filtro activo");
    es.insert("data_info_relation_out", "saliente");
    es.insert("data_info_relation_in", "entrante");
    es.insert("data_info_selected", "Seleccionado");
    es.insert("data_info_nullable", "Nullable");
    es.insert("data_info_value", "Valor");
    es.insert("data_info_original", "Original");
    es.insert("data_info_revert_cell", "Revertir celda");
    es.insert("data_info_dirty", "Esta celda tiene cambios sin guardar");
    es.insert("data_info_yes", "Sí");
    es.insert("data_info_no", "No");
    es.insert("data_info_read_only", "Este valor es de solo lectura aquí.");
    es.insert("data_relation_open", "Abrir fila relacionada");
    es.insert(
        "data_info_read_only_pk",
        "Los valores de clave primaria son de solo lectura aquí.",
    );
    es.insert(
        "data_info_no_metadata",
        "Los metadatos de columna aún se están cargando, por lo que la edición está deshabilitada.",
    );
    es.insert("transfer_title", "Transferir tablas");
    es.insert("transfer_source", "Origen:");
    es.insert("transfer_target", "Destino:");
    es.insert("transfer_tables_header", "Tablas a transferir (orden de dependencia):");
    es.insert("transfer_select_all", "Seleccionar todo");
    es.insert("transfer_deselect_all", "Deseleccionar todo");
    es.insert("transfer_include_data", "Incluir datos");
    es.insert("transfer_if_exists", "Si existe:");
    es.insert("transfer_start", "Transferir");
    es.insert("transfer_cancel", "Cancelar");
    es.insert("transfer_not_implemented", "Backend de transferencia aún no conectado");
    es.insert("migration_title", "Asistente de migración de esquema");
    es.insert("migration_step_select", "Seleccionar");
    es.insert("migration_step_diff", "Diferencias");
    es.insert("migration_step_sql", "SQL");
    es.insert("migration_source_conn", "Origen:");
    es.insert("migration_source_schema", "Esquema:");
    es.insert("migration_target_conn", "Destino:");
    es.insert("migration_target_schema", "Esquema:");
    es.insert("migration_compare", "Comparar");
    es.insert("migration_comparing", "Comparando esquemas...");
    es.insert("migration_no_diff", "Sin resultados de diferencia");
    es.insert("migration_no_changes", "Los esquemas son idénticos, no se necesitan cambios");
    es.insert("migration_tables_added", "tablas añadidas");
    es.insert("migration_tables_modified", "tablas modificadas");
    es.insert("migration_tables_removed", "tablas eliminadas");
    es.insert("migration_preview_sql", "Vista previa SQL");
    es.insert("migration_copy_sql", "Copiar SQL");
    es.insert("migration_apply", "Aplicar al destino");
    es.insert("migration_applying", "Aplicando migración...");
    es.insert("migration_success", "¡Migración aplicada correctamente!");
    es.insert("migration_back", "Atrás");
    es.insert("migration_close", "Cerrar");
}

fn insert_recent_ui_fr(fr: &mut Translation) {
    fr.insert("ctx_close_connection", "Fermer la connexion");
    fr.insert("ctx_open_connection", "Ouvrir la connexion");
    fr.insert(
        "ctx_switch_connection_profile",
        "Changer de profil de connexion",
    );
    fr.insert("ctx_no_saved_profiles", "Aucun profil enregistré");
    fr.insert("ctx_edit_connection", "Modifier la connexion...");
    fr.insert("ctx_new_connection", "Nouvelle connexion");
    fr.insert("ctx_delete_connection", "Supprimer la connexion");
    fr.insert("ctx_duplicate_connection", "Dupliquer la connexion...");
    fr.insert("ctx_new_database", "Nouvelle base de données...");
    fr.insert("ctx_new_table", "Nouvelle table");
    fr.insert("ctx_new_query", "Nouvelle requête");
    fr.insert("ctx_console", "Console");
    fr.insert("ctx_execute_sql_file", "Exécuter un fichier SQL...");
    fr.insert("ctx_open_schema", "Ouvrir le schéma");
    fr.insert("ctx_backup_schema", "Sauvegarder {0}");
    fr.insert("ctx_edit_schema", "Modifier le schéma...");
    fr.insert("ctx_new_schema", "Nouveau schéma...");
    fr.insert("ctx_delete_schema", "Supprimer le schéma");
    fr.insert("ctx_dump_sql_file", "Exporter le fichier SQL");
    fr.insert("ctx_data_dictionary", "Dictionnaire de données...");
    fr.insert(
        "ctx_reverse_database_to_model",
        "Rétro-ingénierie de la base de données vers le modèle...",
    );
    fr.insert(
        "ctx_find_in_database",
        "Rechercher dans la base de données...",
    );
    fr.insert("ctx_add_star", "Ajouter aux favoris");
    fr.insert("ctx_color", "Couleur :");
    fr.insert("ctx_manage_group", "Gérer le groupe");
    fr.insert("ctx_create_group", "Créer un groupe...");
    fr.insert("ctx_move_to_group", "Déplacer vers le groupe...");
    fr.insert("ctx_compare_schema", "Comparer le schéma...");
    fr.insert("ctx_share", "Partager...");
    fr.insert("ctx_refresh", "Actualiser");
    fr.insert("ctx_close_all_connections", "Fermer toutes les connexions");
    fr.insert("ctx_manage_connections", "Gérer les connexions...");
    fr.insert("ctx_new_group", "Nouveau groupe");

    fr.insert("tree_no_connections", "Aucune connexion");
    fr.insert(
        "tree_create_connection",
        "Créez une connexion pour parcourir les schémas",
    );
    fr.insert("tree_empty", "(vide)");
    fr.insert("tree_tables", "Tables");
    fr.insert("tree_views", "Vues");
    fr.insert("tree_materialized_views", "Vues matérialisées");
    fr.insert("tree_functions", "Fonctions");
    fr.insert("tree_queries", "Requêtes");
    fr.insert("tree_backups", "Sauvegardes");
    fr.insert("tree_schema_backup", "Sauvegarde du schéma");
    fr.insert(
        "tree_full_database_backup",
        "Sauvegarde complète de la base de données",
    );
    fr.insert("tree_fields", "Champs");
    fr.insert("tree_indexes", "Index");
    fr.insert("tree_foreign_keys", "Clés étrangères");
    fr.insert("tree_unique", "Unique");
    fr.insert("tree_rules", "Règles");
    fr.insert("tree_triggers", "Déclencheurs");
    fr.insert("tree_edit_table", "Modifier la table");
    fr.insert("tree_view_data_top_100", "Voir les données (100 premières)");
    fr.insert("tree_copy_select", "Copier SELECT *");
    fr.insert("tree_copy_table", "Copier la table (transfert)");
    fr.insert("tree_refresh_metadata", "Actualiser les métadonnées");
    fr.insert("tree_copy_signature", "Copier la signature");
    fr.insert("tree_copy_rule_ddl", "Copier le DDL de règle");
    fr.insert("tree_copy_trigger_ddl", "Copier le DDL de déclencheur");
    fr.insert("tree_show_functions", "Afficher les fonctions");
    fr.insert("tree_show_group", "Afficher {0}");
    fr.insert("tree_showing_group", "Affichage de {0} dans {1}");
    fr.insert("tree_showing_functions", "Affichage des fonctions dans {0}");
    fr.insert("tree_backup_schema_title", "Sauvegarde : {0}");
    fr.insert("tree_backup_full_title", "Sauvegarde : complète");
    fr.insert(
        "tree_backup_scope_schema",
        "Portée de la sauvegarde : schéma {0}",
    );
    fr.insert(
        "tree_backup_scope_full",
        "Portée de la sauvegarde : base de données complète",
    );
    fr.insert(
        "tree_refreshing_connections",
        "Actualisation de {0} connexion(s)...",
    );
    fr.insert("tree_explorer_refreshed", "Explorateur actualisé");
    fr.insert(
        "tree_closing_all_connections",
        "Fermeture de toutes les connexions...",
    );

    fr.insert("objects_all_schemas", "Tous les schémas");
    fr.insert("objects_schema", "Schéma");
    fr.insert("objects_name", "Nom");
    fr.insert("objects_type", "Type");
    fr.insert("objects_rows", "Lignes");
    fr.insert("objects_no_tables", "Aucune table trouvée");
    fr.insert("objects_no_tables_help", "Essayez un autre schéma ou terme de recherche");
    fr.insert("objects_columns", "Colonnes");
    fr.insert("objects_indexes", "Index");
    fr.insert("objects_actions", "Actions");
    fr.insert("objects_search", "Rechercher");
    fr.insert("objects_new_table", "Nouvelle table");
    fr.insert("objects_open_model", "Ouvrir le diagramme ER");
    fr.insert("objects_signature", "Signature");
    fr.insert("objects_returns", "Retourne");
    fr.insert("objects_lang", "Langage");
    fr.insert("objects_role", "Rôle");
    fr.insert("objects_login", "Connexion");
    fr.insert("objects_privileges", "Privilèges");
    fr.insert("objects_valid_until", "Valide jusqu'au");
    fr.insert("objects_column", "Colonne");
    fr.insert("objects_non_null", "Non nul");
    fr.insert("objects_min", "Minimum");
    fr.insert("objects_max", "Maximum");
    fr.insert("objects_average", "Moyenne");
    fr.insert("objects_no_active_connection", "Aucune connexion active");
    fr.insert(
        "objects_no_active_connection_help",
        "Connectez-vous à PostgreSQL pour parcourir et gérer les objets de base de données.",
    );
    fr.insert("objects_tables_title", "Tables");
    fr.insert(
        "objects_tables_subtitle",
        "Tables de base et relations modifiables",
    );
    fr.insert("objects_views_title", "Vues");
    fr.insert(
        "objects_views_subtitle",
        "Objets virtuels basés sur des requêtes",
    );
    fr.insert("objects_materialized_title", "Vues matérialisées");
    fr.insert(
        "objects_materialized_subtitle",
        "Instantanés de requêtes stockés",
    );
    fr.insert("objects_functions_title", "Fonctions");
    fr.insert(
        "objects_functions_subtitle",
        "Routines PostgreSQL par schéma",
    );
    fr.insert("objects_users_title", "Utilisateurs");
    fr.insert(
        "objects_users_subtitle",
        "Rôles et permissions de connexion",
    );
    fr.insert("objects_backup_title", "Sauvegarde");
    fr.insert(
        "objects_backup_subtitle",
        "Générateur de commandes pg_dump et restauration",
    );
    fr.insert("objects_automation_title", "Automatisation");
    fr.insert(
        "objects_automation_subtitle",
        "Préréglages de requêtes de maintenance",
    );
    fr.insert("objects_model_title", "Modèle");
    fr.insert(
        "objects_model_subtitle",
        "Diagramme ER et modélisation de schéma",
    );
    fr.insert("objects_bi_title", "BI");
    fr.insert(
        "objects_bi_subtitle",
        "Profilage rapide des jeux de résultats",
    );
    fr.insert("objects_connections_title", "Connexions");
    fr.insert(
        "objects_connections_subtitle",
        "Configuration de la connexion à la base de données",
    );
    fr.insert("objects_query_title", "Requête");
    fr.insert("objects_query_subtitle", "Éditeur SQL");
    fr.insert("objects_data_title", "Données");
    fr.insert("objects_data_subtitle", "Parcourir les lignes de la table");

    fr.insert("backup_schema", "Sauvegarde du schéma");
    fr.insert(
        "backup_full_database",
        "Sauvegarde complète de la base de données",
    );
    fr.insert(
        "backup_no_folder_selected",
        "Aucun dossier de sauvegarde sélectionné",
    );
    fr.insert("backup_folder_title", "Dossier de sauvegarde FerrumGrid");
    fr.insert("backup_choose_folder", "Choisir un dossier");
    fr.insert("backup_open_folder", "Ouvrir le dossier");
    fr.insert("backup_folder_updated", "Dossier de sauvegarde mis à jour");
    fr.insert("backup_format", "Format");
    fr.insert("backup_custom_archive", "Archive personnalisée (.dump)");
    fr.insert("backup_plain_sql", "SQL simple (.sql)");
    fr.insert("backup_running_label", "Sauvegarde en cours...");
    fr.insert("backup_running_status", "Sauvegarde de {0} en cours...");
    fr.insert("backup_run", "Lancer la sauvegarde");
    fr.insert("backup_pg_dump_running", "pg_dump en cours d'exécution");
    fr.insert("backup_tar_archive", "Archive Tar");
    fr.insert("backup_recent", "Sauvegardes récentes de FerrumGrid");
    fr.insert("backup_no_session", "Aucune sauvegarde dans cette session");
    fr.insert("backup_files_title_count", "Fichiers de sauvegarde ({0})");
    fr.insert("backup_files_title", "Fichiers de sauvegarde");
    fr.insert("backup_files_refresh", "Actualiser");
    fr.insert(
        "backup_files_set_folder",
        "Définissez le dossier de sauvegarde pour parcourir les fichiers",
    );
    fr.insert("backup_files_empty", "Aucun fichier de sauvegarde trouvé");
    fr.insert("backup_files_col_name", "Nom");
    fr.insert("backup_files_col_size", "Taille");
    fr.insert("backup_files_col_created", "Créé");
    fr.insert("backup_files_col_modified", "Modifié");
    fr.insert("backup_files_col_actions", "Actions");
    fr.insert("backup_files_show", "Afficher");
    fr.insert("backup_files_delete_confirm", "Supprimer ?");
    fr.insert("backup_files_yes", "Oui");
    fr.insert("backup_files_no", "Non");
    fr.insert("backup_files_delete", "Supprimer");

    fr.insert("schema_visualizer_title", "Visualiseur de schéma");
    fr.insert(
        "schema_visualizer_desc",
        "Explorez les tables, colonnes et relations de clés étrangères.",
    );
    fr.insert("schema_visualizer_open", "Ouvrir le visualiseur");
    fr.insert("visualizer_schema", "Schéma");
    fr.insert("visualizer_search_hint", "Rechercher des tables ou des colonnes");
    fr.insert("visualizer_reload", "Recharger");
    fr.insert("visualizer_auto_layout", "Disposition automatique");
    fr.insert("visualizer_fit", "Ajuster");
    fr.insert("visualizer_zoom", "Zoom");
    fr.insert("visualizer_close_tooltip", "Fermer le visualiseur de schéma");
    fr.insert("visualizer_loading_columns", "Chargement des colonnes...");
    fr.insert(
        "visualizer_loading_title",
        "Chargement du visualiseur de schéma...",
    );
    fr.insert(
        "visualizer_loading_subtitle",
        "Les tables, colonnes et relations apparaîtront ici automatiquement.",
    );
    fr.insert("visualizer_no_matching_tables", "Aucune table correspondante");
    fr.insert(
        "visualizer_clear_search_hint",
        "Effacez la recherche pour afficher le schéma complet.",
    );
    fr.insert("visualizer_no_tables_title", "Aucune table dans ce schéma");
    fr.insert(
        "visualizer_no_tables_subtitle",
        "Sélectionnez un autre schéma ou actualisez.",
    );
    fr.insert("visualizer_more_columns", "+{0} colonnes supplémentaires");
    fr.insert("visualizer_count", "{0} tables  |  {1} relations");

    fr.insert("workspace_close_tab", "Fermer l'onglet");
    fr.insert("workspace_new_query", "Nouvelle requête");
    fr.insert("grid_revert", "Rétablir");
    fr.insert("grid_edits", "{0} modification(s)");
    fr.insert(
        "grid_pk_required",
        "Clé primaire requise pour mettre à jour les lignes",
    );
    fr.insert("grid_invalid_values", "{0} valeur(s) invalide(s)");
    fr.insert("grid_toggle_null", "Basculer NULL");
    fr.insert("grid_null_value", "Valeur NULL");
    fr.insert("grid_copy_value", "Copier la valeur");
    fr.insert("grid_copy_sql", "Copier SQL");
    fr.insert("grid_no_active_data_source", "Aucune source de données active");
    fr.insert("grid_no_result_set", "Aucun jeu de résultats");
    fr.insert(
        "grid_column_missing",
        "La colonne modifiée n'est plus disponible",
    );
    fr.insert(
        "grid_pk_missing",
        "La colonne de clé primaire {0} n'est pas dans le jeu de résultats",
    );
    fr.insert(
        "grid_pk_value_missing",
        "La valeur de clé primaire n'est pas disponible",
    );
    fr.insert("grid_not_null", "Cette colonne n'autorise pas NULL");
    fr.insert("grid_bool_error", "Utilisez true ou false");
    fr.insert("grid_number_error", "Entrez un nombre valide");
    fr.insert("grid_json_error", "Entrez du JSON valide");
    fr.insert("grid_uuid_error", "Entrez un UUID valide");
    fr.insert(
        "grid_bytes_error",
        "Entrez des octets hexadécimaux, par exemple \\xDEADBEEF",
    );
    fr.insert("grid_date_error", "Entrez une date au format YYYY-MM-DD");
    fr.insert(
        "grid_datetime_error",
        "Entrez la date et l'heure au format YYYY-MM-DD HH:MM:SS",
    );
    fr.insert("grid_now", "Maintenant");
    fr.insert("grid_pick_date", "Choisir une date");
    fr.insert("grid_pick_time", "Choisir une heure");
    fr.insert("grid_prev_month", "Mois précédent");
    fr.insert("grid_next_month", "Mois suivant");
    fr.insert("grid_hour", "Heure");
    fr.insert("grid_minute", "Min");
    fr.insert("grid_second", "Sec");
    fr.insert("grid_weekday_mon", "L");
    fr.insert("grid_weekday_tue", "M");
    fr.insert("grid_weekday_wed", "M");
    fr.insert("grid_weekday_thu", "J");
    fr.insert("grid_weekday_fri", "V");
    fr.insert("grid_weekday_sat", "S");
    fr.insert("grid_weekday_sun", "D");
    fr.insert("grid_sort_asc", "Trier par ordre croissant");
    fr.insert("grid_sort_desc", "Trier par ordre décroissant");
    fr.insert("grid_sort_remove", "Supprimer le tri");
    fr.insert("grid_sort_clear_all", "Effacer tous les tris");
    fr.insert(
        "grid_sort_unsaved",
        "Appliquez ou rétablissez les modifications avant de trier",
    );
    fr.insert(
        "grid_page_unsaved",
        "Appliquez ou rétablissez les modifications avant de changer de page",
    );
    fr.insert("grid_first_page", "Première page");
    fr.insert("grid_prev_page", "Page précédente");
    fr.insert("grid_next_page", "Page suivante");
    fr.insert("grid_page", "Page");
    fr.insert("grid_page_n", "Page {0}");
    fr.insert("grid_limit", "Limite");
    fr.insert("grid_limit_n", "Limite {0}");
    fr.insert("grid_limit_error", "Entrez une limite de lignes valide");
    fr.insert("grid_enum_select", "Sélectionner une valeur");
    fr.insert("grid_enum_error", "Sélectionnez l'une des valeurs autorisées");
    fr.insert("grid_visible_range", "{0}-{1}");
    fr.insert("data_info_no_selection", "Aucune information");
    fr.insert("data_info_select_cell", "Sélectionnez une ligne");
    fr.insert("data_info_cell", "Cellule sélectionnée");
    fr.insert("data_info_row", "Ligne sélectionnée");
    fr.insert("data_info_table", "Table sélectionnée");
    fr.insert("data_info_row_n", "Ligne {0}");
    fr.insert("data_info_col_n", "Col {0}");
    fr.insert("data_info_columns", "Colonnes");
    fr.insert("data_info_columns_n", "{0} colonnes");
    fr.insert("data_info_indexes_n", "{0} index");
    fr.insert("data_info_relations_n", "{0} relations");
    fr.insert("data_info_rules_n", "{0} règles");
    fr.insert("data_info_triggers_n", "{0} déclencheurs");
    fr.insert("data_info_active_filter", "Filtre actif");
    fr.insert("data_info_relation_out", "sortante");
    fr.insert("data_info_relation_in", "entrante");
    fr.insert("data_info_selected", "Sélectionné");
    fr.insert("data_info_nullable", "Nullable");
    fr.insert("data_info_value", "Valeur");
    fr.insert("data_info_original", "Original");
    fr.insert("data_info_revert_cell", "Rétablir la cellule");
    fr.insert("data_info_dirty", "Cette cellule a des modifications non enregistrées");
    fr.insert("data_info_yes", "Oui");
    fr.insert("data_info_no", "Non");
    fr.insert("data_info_read_only", "Cette valeur est en lecture seule ici.");
    fr.insert("data_relation_open", "Ouvrir la ligne liée");
    fr.insert(
        "data_info_read_only_pk",
        "Les valeurs de clé primaire sont en lecture seule ici.",
    );
    fr.insert(
        "data_info_no_metadata",
        "Les métadonnées de colonne sont encore en cours de chargement, la modification est donc désactivée.",
    );
    fr.insert("transfer_title", "Transférer les tables");
    fr.insert("transfer_source", "Source :");
    fr.insert("transfer_target", "Cible :");
    fr.insert("transfer_tables_header", "Tables à transférer (ordre de dépendance) :");
    fr.insert("transfer_select_all", "Tout sélectionner");
    fr.insert("transfer_deselect_all", "Tout désélectionner");
    fr.insert("transfer_include_data", "Inclure les données");
    fr.insert("transfer_if_exists", "Si existe :");
    fr.insert("transfer_start", "Transférer");
    fr.insert("transfer_cancel", "Annuler");
    fr.insert("transfer_not_implemented", "Backend de transfert pas encore connecté");
    fr.insert("migration_title", "Assistant de migration de schéma");
    fr.insert("migration_step_select", "Sélection");
    fr.insert("migration_step_diff", "Différences");
    fr.insert("migration_step_sql", "SQL");
    fr.insert("migration_source_conn", "Source :");
    fr.insert("migration_source_schema", "Schéma :");
    fr.insert("migration_target_conn", "Cible :");
    fr.insert("migration_target_schema", "Schéma :");
    fr.insert("migration_compare", "Comparer");
    fr.insert("migration_comparing", "Comparaison des schémas...");
    fr.insert("migration_no_diff", "Aucun résultat de différence");
    fr.insert("migration_no_changes", "Les schémas sont identiques, aucun changement nécessaire");
    fr.insert("migration_tables_added", "tables ajoutées");
    fr.insert("migration_tables_modified", "tables modifiées");
    fr.insert("migration_tables_removed", "tables supprimées");
    fr.insert("migration_preview_sql", "Aperçu SQL");
    fr.insert("migration_copy_sql", "Copier SQL");
    fr.insert("migration_apply", "Appliquer à la cible");
    fr.insert("migration_applying", "Application de la migration...");
    fr.insert("migration_success", "Migration appliquée avec succès !");
    fr.insert("migration_back", "Retour");
    fr.insert("migration_close", "Fermer");
}

fn insert_recent_ui_de(de: &mut Translation) {
    de.insert("ctx_close_connection", "Verbindung schließen");
    de.insert("ctx_open_connection", "Verbindung öffnen");
    de.insert("ctx_switch_connection_profile", "Verbindungsprofil wechseln");
    de.insert("ctx_no_saved_profiles", "Keine gespeicherten Profile");
    de.insert("ctx_edit_connection", "Verbindung bearbeiten...");
    de.insert("ctx_new_connection", "Neue Verbindung");
    de.insert("ctx_delete_connection", "Verbindung löschen");
    de.insert("ctx_duplicate_connection", "Verbindung duplizieren...");
    de.insert("ctx_new_database", "Neue Datenbank...");
    de.insert("ctx_new_table", "Neue Tabelle");
    de.insert("ctx_new_query", "Neue Abfrage");
    de.insert("ctx_console", "Konsole");
    de.insert("ctx_execute_sql_file", "SQL-Datei ausführen...");
    de.insert("ctx_open_schema", "Schema öffnen");
    de.insert("ctx_backup_schema", "{0} sichern");
    de.insert("ctx_edit_schema", "Schema bearbeiten...");
    de.insert("ctx_new_schema", "Neues Schema...");
    de.insert("ctx_delete_schema", "Schema löschen");
    de.insert("ctx_dump_sql_file", "SQL-Datei exportieren");
    de.insert("ctx_data_dictionary", "Datenwörterbuch...");
    de.insert(
        "ctx_reverse_database_to_model",
        "Datenbank in Modell umkehren...",
    );
    de.insert("ctx_find_in_database", "In Datenbank suchen...");
    de.insert("ctx_add_star", "Favorit hinzufügen");
    de.insert("ctx_color", "Farbe:");
    de.insert("ctx_manage_group", "Gruppe verwalten");
    de.insert("ctx_create_group", "Gruppe erstellen...");
    de.insert("ctx_move_to_group", "In Gruppe verschieben...");
    de.insert("ctx_compare_schema", "Schema vergleichen...");
    de.insert("ctx_share", "Teilen...");
    de.insert("ctx_refresh", "Aktualisieren");
    de.insert("ctx_close_all_connections", "Alle Verbindungen schließen");
    de.insert("ctx_manage_connections", "Verbindungen verwalten...");
    de.insert("ctx_new_group", "Neue Gruppe");

    de.insert("tree_no_connections", "Keine Verbindungen");
    de.insert(
        "tree_create_connection",
        "Erstellen Sie eine Verbindung, um Schemata zu durchsuchen",
    );
    de.insert("tree_empty", "(leer)");
    de.insert("tree_tables", "Tabellen");
    de.insert("tree_views", "Ansichten");
    de.insert("tree_materialized_views", "Materialisierte Ansichten");
    de.insert("tree_functions", "Funktionen");
    de.insert("tree_queries", "Abfragen");
    de.insert("tree_backups", "Sicherungen");
    de.insert("tree_schema_backup", "Schema-Sicherung");
    de.insert("tree_full_database_backup", "Vollständige Datenbanksicherung");
    de.insert("tree_fields", "Felder");
    de.insert("tree_indexes", "Indizes");
    de.insert("tree_foreign_keys", "Fremdschlüssel");
    de.insert("tree_unique", "Eindeutig");
    de.insert("tree_rules", "Regeln");
    de.insert("tree_triggers", "Trigger");
    de.insert("tree_edit_table", "Tabelle bearbeiten");
    de.insert("tree_view_data_top_100", "Daten anzeigen (Top 100)");
    de.insert("tree_copy_select", "SELECT * kopieren");
    de.insert("tree_copy_table", "Tabelle kopieren (Transfer)");
    de.insert("tree_refresh_metadata", "Metadaten aktualisieren");
    de.insert("tree_copy_signature", "Signatur kopieren");
    de.insert("tree_copy_rule_ddl", "Regel-DDL kopieren");
    de.insert("tree_copy_trigger_ddl", "Trigger-DDL kopieren");
    de.insert("tree_show_functions", "Funktionen anzeigen");
    de.insert("tree_show_group", "{0} anzeigen");
    de.insert("tree_showing_group", "{0} in {1} wird angezeigt");
    de.insert("tree_showing_functions", "Funktionen in {0} werden angezeigt");
    de.insert("tree_backup_schema_title", "Sicherung: {0}");
    de.insert("tree_backup_full_title", "Sicherung: vollständig");
    de.insert("tree_backup_scope_schema", "Sicherungsbereich: {0}-Schema");
    de.insert(
        "tree_backup_scope_full",
        "Sicherungsbereich: vollständige Datenbank",
    );
    de.insert(
        "tree_refreshing_connections",
        "{0} Verbindung(en) werden aktualisiert...",
    );
    de.insert("tree_explorer_refreshed", "Explorer aktualisiert");
    de.insert(
        "tree_closing_all_connections",
        "Alle Verbindungen werden geschlossen...",
    );

    de.insert("objects_all_schemas", "Alle Schemata");
    de.insert("objects_schema", "Schema");
    de.insert("objects_name", "Name");
    de.insert("objects_type", "Typ");
    de.insert("objects_rows", "Zeilen");
    de.insert("objects_no_tables", "Keine Tabellen gefunden");
    de.insert("objects_no_tables_help", "Versuchen Sie ein anderes Schema oder einen anderen Suchbegriff");
    de.insert("objects_columns", "Spalten");
    de.insert("objects_indexes", "Indizes");
    de.insert("objects_actions", "Aktionen");
    de.insert("objects_search", "Suchen");
    de.insert("objects_new_table", "Neue Tabelle");
    de.insert("objects_open_model", "ER-Diagramm öffnen");
    de.insert("objects_signature", "Signatur");
    de.insert("objects_returns", "Rückgabe");
    de.insert("objects_lang", "Sprache");
    de.insert("objects_role", "Rolle");
    de.insert("objects_login", "Anmeldung");
    de.insert("objects_privileges", "Berechtigungen");
    de.insert("objects_valid_until", "Gültig bis");
    de.insert("objects_column", "Spalte");
    de.insert("objects_non_null", "Nicht null");
    de.insert("objects_min", "Minimum");
    de.insert("objects_max", "Maximum");
    de.insert("objects_average", "Durchschnitt");
    de.insert("objects_no_active_connection", "Keine aktive Verbindung");
    de.insert(
        "objects_no_active_connection_help",
        "Verbinden Sie sich mit PostgreSQL, um Datenbankobjekte zu durchsuchen und zu verwalten.",
    );
    de.insert("objects_tables_title", "Tabellen");
    de.insert(
        "objects_tables_subtitle",
        "Basistabellen und bearbeitbare Relationen",
    );
    de.insert("objects_views_title", "Ansichten");
    de.insert(
        "objects_views_subtitle",
        "Abfragebasierte virtuelle Objekte",
    );
    de.insert("objects_materialized_title", "Materialisierte Ansichten");
    de.insert(
        "objects_materialized_subtitle",
        "Gespeicherte Abfrage-Snapshots",
    );
    de.insert("objects_functions_title", "Funktionen");
    de.insert(
        "objects_functions_subtitle",
        "PostgreSQL-Routinen nach Schema",
    );
    de.insert("objects_users_title", "Benutzer");
    de.insert(
        "objects_users_subtitle",
        "Rollen und Anmeldeberechtigungen",
    );
    de.insert("objects_backup_title", "Sicherung");
    de.insert(
        "objects_backup_subtitle",
        "pg_dump- und Wiederherstellungsbefehl-Generator",
    );
    de.insert("objects_automation_title", "Automatisierung");
    de.insert(
        "objects_automation_subtitle",
        "Wartungsabfrage-Voreinstellungen",
    );
    de.insert("objects_model_title", "Modell");
    de.insert(
        "objects_model_subtitle",
        "ER-Diagramm und Schema-Modellierung",
    );
    de.insert("objects_bi_title", "BI");
    de.insert(
        "objects_bi_subtitle",
        "Schnelles Ergebnismenge-Profiling",
    );
    de.insert("objects_connections_title", "Verbindungen");
    de.insert(
        "objects_connections_subtitle",
        "Datenbankverbindungs-Einrichtung",
    );
    de.insert("objects_query_title", "Abfrage");
    de.insert("objects_query_subtitle", "SQL-Editor");
    de.insert("objects_data_title", "Daten");
    de.insert("objects_data_subtitle", "Tabellenzeilen durchsuchen");

    de.insert("backup_schema", "Schema-Sicherung");
    de.insert("backup_full_database", "Vollständige Datenbanksicherung");
    de.insert("backup_no_folder_selected", "Kein Sicherungsordner ausgewählt");
    de.insert("backup_folder_title", "FerrumGrid-Sicherungsordner");
    de.insert("backup_choose_folder", "Ordner wählen");
    de.insert("backup_open_folder", "Ordner öffnen");
    de.insert("backup_folder_updated", "Sicherungsordner aktualisiert");
    de.insert("backup_format", "Format");
    de.insert("backup_custom_archive", "Benutzerdefiniertes Archiv (.dump)");
    de.insert("backup_plain_sql", "Einfaches SQL (.sql)");
    de.insert("backup_running_label", "Sicherung läuft...");
    de.insert("backup_running_status", "{0} wird gesichert...");
    de.insert("backup_run", "Sicherung starten");
    de.insert("backup_pg_dump_running", "pg_dump wird ausgeführt");
    de.insert("backup_tar_archive", "Tar-Archiv");
    de.insert("backup_recent", "Aktuelle FerrumGrid-Sicherungen");
    de.insert("backup_no_session", "Keine Sicherungen in dieser Sitzung");
    de.insert("backup_files_title_count", "Sicherungsdateien ({0})");
    de.insert("backup_files_title", "Sicherungsdateien");
    de.insert("backup_files_refresh", "Aktualisieren");
    de.insert(
        "backup_files_set_folder",
        "Legen Sie den Sicherungsordner fest, um Dateien zu durchsuchen",
    );
    de.insert("backup_files_empty", "Keine Sicherungsdateien gefunden");
    de.insert("backup_files_col_name", "Name");
    de.insert("backup_files_col_size", "Größe");
    de.insert("backup_files_col_created", "Erstellt");
    de.insert("backup_files_col_modified", "Geändert");
    de.insert("backup_files_col_actions", "Aktionen");
    de.insert("backup_files_show", "Anzeigen");
    de.insert("backup_files_delete_confirm", "Löschen?");
    de.insert("backup_files_yes", "Ja");
    de.insert("backup_files_no", "Nein");
    de.insert("backup_files_delete", "Löschen");

    de.insert("schema_visualizer_title", "Schema-Visualisierer");
    de.insert(
        "schema_visualizer_desc",
        "Erkunden Sie Tabellen, Spalten und Fremdschlüssel-Beziehungen.",
    );
    de.insert("schema_visualizer_open", "Visualisierer öffnen");
    de.insert("visualizer_schema", "Schema");
    de.insert("visualizer_search_hint", "Tabellen oder Spalten suchen");
    de.insert("visualizer_reload", "Neu laden");
    de.insert("visualizer_auto_layout", "Automatisches Layout");
    de.insert("visualizer_fit", "Anpassen");
    de.insert("visualizer_zoom", "Zoom");
    de.insert("visualizer_close_tooltip", "Schema-Visualisierer schließen");
    de.insert("visualizer_loading_columns", "Spalten werden geladen...");
    de.insert(
        "visualizer_loading_title",
        "Schema-Visualisierer wird geladen...",
    );
    de.insert(
        "visualizer_loading_subtitle",
        "Tabellen, Spalten und Beziehungen werden hier automatisch angezeigt.",
    );
    de.insert("visualizer_no_matching_tables", "Keine passenden Tabellen");
    de.insert(
        "visualizer_clear_search_hint",
        "Löschen Sie das Suchfeld, um das vollständige Schema anzuzeigen.",
    );
    de.insert("visualizer_no_tables_title", "Keine Tabellen in diesem Schema");
    de.insert(
        "visualizer_no_tables_subtitle",
        "Wählen Sie ein anderes Schema oder aktualisieren Sie.",
    );
    de.insert("visualizer_more_columns", "+{0} weitere Spalten");
    de.insert("visualizer_count", "{0} Tabellen  |  {1} Relationen");

    de.insert("workspace_close_tab", "Tab schließen");
    de.insert("workspace_new_query", "Neue Abfrage");
    de.insert("grid_revert", "Zurücksetzen");
    de.insert("grid_edits", "{0} Änderung(en)");
    de.insert(
        "grid_pk_required",
        "Primärschlüssel erforderlich, um Zeilen zu aktualisieren",
    );
    de.insert("grid_invalid_values", "{0} ungültiger Wert(e)");
    de.insert("grid_toggle_null", "NULL umschalten");
    de.insert("grid_null_value", "NULL-Wert");
    de.insert("grid_copy_value", "Wert kopieren");
    de.insert("grid_copy_sql", "SQL kopieren");
    de.insert("grid_no_active_data_source", "Keine aktive Datenquelle");
    de.insert("grid_no_result_set", "Kein Ergebnissatz");
    de.insert(
        "grid_column_missing",
        "Bearbeitete Spalte ist nicht mehr verfügbar",
    );
    de.insert(
        "grid_pk_missing",
        "Primärschlüsselspalte {0} ist nicht im Ergebnissatz",
    );
    de.insert(
        "grid_pk_value_missing",
        "Primärschlüsselwert ist nicht verfügbar",
    );
    de.insert("grid_not_null", "Diese Spalte erlaubt kein NULL");
    de.insert("grid_bool_error", "Verwenden Sie true oder false");
    de.insert("grid_number_error", "Geben Sie eine gültige Zahl ein");
    de.insert("grid_json_error", "Geben Sie gültiges JSON ein");
    de.insert("grid_uuid_error", "Geben Sie eine gültige UUID ein");
    de.insert(
        "grid_bytes_error",
        "Geben Sie Hex-Bytes ein, z.B. \\xDEADBEEF",
    );
    de.insert("grid_date_error", "Geben Sie ein Datum als YYYY-MM-DD ein");
    de.insert(
        "grid_datetime_error",
        "Geben Sie Datum und Uhrzeit als YYYY-MM-DD HH:MM:SS ein",
    );
    de.insert("grid_now", "Jetzt");
    de.insert("grid_pick_date", "Datum wählen");
    de.insert("grid_pick_time", "Uhrzeit wählen");
    de.insert("grid_prev_month", "Vorheriger Monat");
    de.insert("grid_next_month", "Nächster Monat");
    de.insert("grid_hour", "Std");
    de.insert("grid_minute", "Min");
    de.insert("grid_second", "Sek");
    de.insert("grid_weekday_mon", "M");
    de.insert("grid_weekday_tue", "D");
    de.insert("grid_weekday_wed", "M");
    de.insert("grid_weekday_thu", "D");
    de.insert("grid_weekday_fri", "F");
    de.insert("grid_weekday_sat", "S");
    de.insert("grid_weekday_sun", "S");
    de.insert("grid_sort_asc", "Aufsteigend sortieren");
    de.insert("grid_sort_desc", "Absteigend sortieren");
    de.insert("grid_sort_remove", "Sortierung entfernen");
    de.insert("grid_sort_clear_all", "Alle Sortierungen löschen");
    de.insert(
        "grid_sort_unsaved",
        "Wenden Sie Änderungen an oder setzen Sie sie zurück, bevor Sie sortieren",
    );
    de.insert(
        "grid_page_unsaved",
        "Wenden Sie Änderungen an oder setzen Sie sie zurück, bevor Sie die Seite wechseln",
    );
    de.insert("grid_first_page", "Erste Seite");
    de.insert("grid_prev_page", "Vorherige Seite");
    de.insert("grid_next_page", "Nächste Seite");
    de.insert("grid_page", "Seite");
    de.insert("grid_page_n", "Seite {0}");
    de.insert("grid_limit", "Limit");
    de.insert("grid_limit_n", "Limit {0}");
    de.insert("grid_limit_error", "Geben Sie ein gültiges Zeilenlimit ein");
    de.insert("grid_enum_select", "Wert auswählen");
    de.insert("grid_enum_error", "Wählen Sie einen der zulässigen Werte aus");
    de.insert("grid_visible_range", "{0}-{1}");
    de.insert("data_info_no_selection", "Keine Information");
    de.insert("data_info_select_cell", "Wählen Sie eine Zeile aus");
    de.insert("data_info_cell", "Ausgewählte Zelle");
    de.insert("data_info_row", "Ausgewählte Zeile");
    de.insert("data_info_table", "Ausgewählte Tabelle");
    de.insert("data_info_row_n", "Zeile {0}");
    de.insert("data_info_col_n", "Sp. {0}");
    de.insert("data_info_columns", "Spalten");
    de.insert("data_info_columns_n", "{0} Spalten");
    de.insert("data_info_indexes_n", "{0} Indizes");
    de.insert("data_info_relations_n", "{0} Relationen");
    de.insert("data_info_rules_n", "{0} Regeln");
    de.insert("data_info_triggers_n", "{0} Trigger");
    de.insert("data_info_active_filter", "Aktiver Filter");
    de.insert("data_info_relation_out", "ausgehend");
    de.insert("data_info_relation_in", "eingehend");
    de.insert("data_info_selected", "Ausgewählt");
    de.insert("data_info_nullable", "Nullable");
    de.insert("data_info_value", "Wert");
    de.insert("data_info_original", "Original");
    de.insert("data_info_revert_cell", "Zelle zurücksetzen");
    de.insert(
        "data_info_dirty",
        "Diese Zelle hat nicht gespeicherte Änderungen",
    );
    de.insert("data_info_yes", "Ja");
    de.insert("data_info_no", "Nein");
    de.insert("data_info_read_only", "Dieser Wert ist hier schreibgeschützt.");
    de.insert("data_relation_open", "Verknüpfte Zeile öffnen");
    de.insert(
        "data_info_read_only_pk",
        "Primärschlüsselwerte sind hier schreibgeschützt.",
    );
    de.insert(
        "data_info_no_metadata",
        "Spaltenmetadaten werden noch geladen, daher ist die Bearbeitung deaktiviert.",
    );
    de.insert("transfer_title", "Tabellen übertragen");
    de.insert("transfer_source", "Quelle:");
    de.insert("transfer_target", "Ziel:");
    de.insert("transfer_tables_header", "Zu übertragende Tabellen (Abhängigkeitsreihenfolge):");
    de.insert("transfer_select_all", "Alle auswählen");
    de.insert("transfer_deselect_all", "Alle abwählen");
    de.insert("transfer_include_data", "Daten einschließen");
    de.insert("transfer_if_exists", "Falls vorhanden:");
    de.insert("transfer_start", "Übertragen");
    de.insert("transfer_cancel", "Abbrechen");
    de.insert("transfer_not_implemented", "Transfer-Backend noch nicht verbunden");
    de.insert("migration_title", "Schema-Migrationsassistent");
    de.insert("migration_step_select", "Auswahl");
    de.insert("migration_step_diff", "Unterschiede");
    de.insert("migration_step_sql", "SQL");
    de.insert("migration_source_conn", "Quelle:");
    de.insert("migration_source_schema", "Schema:");
    de.insert("migration_target_conn", "Ziel:");
    de.insert("migration_target_schema", "Schema:");
    de.insert("migration_compare", "Vergleichen");
    de.insert("migration_comparing", "Schemas werden verglichen...");
    de.insert("migration_no_diff", "Kein Vergleichsergebnis");
    de.insert("migration_no_changes", "Schemas sind identisch, keine Änderungen nötig");
    de.insert("migration_tables_added", "Tabellen hinzugefügt");
    de.insert("migration_tables_modified", "Tabellen geändert");
    de.insert("migration_tables_removed", "Tabellen entfernt");
    de.insert("migration_preview_sql", "SQL-Vorschau");
    de.insert("migration_copy_sql", "SQL kopieren");
    de.insert("migration_apply", "Auf Ziel anwenden");
    de.insert("migration_applying", "Migration wird angewendet...");
    de.insert("migration_success", "Migration erfolgreich angewendet!");
    de.insert("migration_back", "Zurück");
    de.insert("migration_close", "Schließen");
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
    en.insert("button_data", "View Data");
    en.insert("button_design", "Design");
    en.insert("button_sql", "Copy SQL");
    en.insert("button_drop", "Drop");
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
    ko.insert("button_data", "데이터 보기");
    ko.insert("button_design", "디자인");
    ko.insert("button_sql", "SQL 복사");
    ko.insert("button_drop", "삭제");
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
    insert_recent_ui_ja(&mut ja);
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
    insert_recent_ui_zh(&mut zh);
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
    insert_recent_ui_es(&mut es);
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
    insert_recent_ui_fr(&mut fr);
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
    insert_recent_ui_de(&mut de);
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
