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

pub fn init() {
    let mut translations = TRANSLATIONS.write().unwrap();

    // English (default)
    let mut en = Translation::new();
    en.insert("app_title", "FerrumGrid");
    en.insert("menu_file", "File");
    en.insert("menu_about", "About FerrumGrid");
    en.insert("menu_new_connection", "New Connection");
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

    translations.insert(Language::English, en);

    // Korean
    let mut ko = Translation::new();
    ko.insert("app_title", "FerrumGrid");
    ko.insert("menu_file", "파일");
    ko.insert("menu_about", "FerrumGrid 정보");
    ko.insert("menu_new_connection", "새 연결");
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

    translations.insert(Language::Korean, ko);

    // Japanese
    let mut ja = Translation::new();
    ja.insert("app_title", "FerrumGrid");
    ja.insert("menu_file", "ファイル");
    ja.insert("menu_new_connection", "新規接続");
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
