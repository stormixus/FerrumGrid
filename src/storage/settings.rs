use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub appearance: String,
    pub dark_mode: bool,
    pub font_size: f32,
    pub default_row_limit: usize,
    pub auto_commit: bool,
    pub confirm_destructive: bool,
    pub language: String,
    pub show_objects_under_schema: bool,
    pub show_objects_under_table: bool,
    pub use_default_object_font: bool,
    pub safe_confirm_dialog: bool,
    pub ask_before_closing_queries: bool,
    pub ask_before_closing_tables: bool,
    pub show_function_wizard: bool,
    pub share_usage_data: bool,
    pub auto_check_updates: bool,
    pub include_system_profile: bool,
    pub open_new_queries_in_tabs: bool,
    pub enable_code_completion: bool,
    pub code_completion_popup: bool,
    pub show_line_numbers: bool,
    pub enable_auto_recovery: bool,
    pub ai_assistant_enabled: bool,
    pub backup_directory: String,
    pub data_timezone: String,
    /// Plan v7 Phase 1.3 — PK 부재 테이블 편집을 위한 ctid opt-in.
    ///
    /// `false` (기본): PK 화이트리스트 가드 활성, mutation 거부.
    /// `true` (opt-in): `RowKeyKind::Ctid` + `RETURNING ctid` 강제 +
    /// `affected != 1` 즉시 ROLLBACK + DiagnosticsPanel 영구 배너.
    /// VACUUM FULL 중 ctid 변동 시 mutation 실패 → 사용자 인지 가능.
    #[serde(default)]
    pub unsafe_ctid: bool,

    // --- General ---
    pub accent_color: String,
    pub density: String,
    pub warn_dangling_tx: bool,
    pub reopen_tabs: bool,
    pub auto_connect_vault: bool,

    // --- Editor ---
    pub font_family: String,
    pub font_ligatures: bool,
    pub format_on_save: bool,
    pub tab_size: usize,
    pub show_whitespace: bool,
    pub word_wrap: bool,
    pub ai_suggest_inline: bool,
    pub ai_suggest_on_hold: bool,

    // --- Data Grid ---
    pub grid_row_height: String,
    pub color_null_cells: bool,
    pub color_fk_cells: bool,
    pub tabular_numbers: bool,
    pub edit_on_double_click: bool,
    pub auto_commit_cells: bool,
    pub confirm_bulk_delete: bool,
    pub long_text_preview: String,
    pub json_cell_display: String,

    // --- Connections ---
    pub pool_min: usize,
    pub pool_max: usize,
    pub idle_timeout: String,
    pub ssl_mode: String,
    pub statement_timeout: String,
    pub lock_timeout: String,
    pub auto_route_replicas: bool,
    pub show_replica_lag: bool,

    // --- Diagnostics ---
    pub diag_show_on_launch: bool,
    pub diag_buffer_size: String,
    pub diag_persist: bool,
    pub slow_query_threshold: String,
    pub render_budget_warn: String,
    pub track_ctid_conflicts: bool,

    // --- Updates ---
    pub update_channel: String,
    pub check_frequency: String,
    pub auto_install_updates: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            appearance: "system".to_string(),
            dark_mode: true,
            font_size: 13.0,
            default_row_limit: 1000,
            auto_commit: true,
            confirm_destructive: true,
            language: "en".to_string(),
            show_objects_under_schema: true,
            show_objects_under_table: true,
            use_default_object_font: true,
            safe_confirm_dialog: true,
            ask_before_closing_queries: true,
            ask_before_closing_tables: true,
            show_function_wizard: true,
            share_usage_data: false,
            auto_check_updates: true,
            include_system_profile: false,
            open_new_queries_in_tabs: true,
            enable_code_completion: true,
            code_completion_popup: true,
            show_line_numbers: true,
            enable_auto_recovery: true,
            ai_assistant_enabled: false,
            backup_directory: String::new(),
            data_timezone: "Asia/Seoul".to_string(),
            unsafe_ctid: false,

            // General
            accent_color: "emerald".to_string(),
            density: "default".to_string(),
            warn_dangling_tx: true,
            reopen_tabs: true,
            auto_connect_vault: false,

            // Editor
            font_family: "SF Mono".to_string(),
            font_ligatures: false,
            format_on_save: true,
            tab_size: 2,
            show_whitespace: false,
            word_wrap: false,
            ai_suggest_inline: true,
            ai_suggest_on_hold: true,

            // Data Grid
            grid_row_height: "28px".to_string(),
            color_null_cells: true,
            color_fk_cells: true,
            tabular_numbers: true,
            edit_on_double_click: true,
            auto_commit_cells: false,
            confirm_bulk_delete: true,
            long_text_preview: "160 chars".to_string(),
            json_cell_display: "Single-line".to_string(),

            // Connections
            pool_min: 2,
            pool_max: 8,
            idle_timeout: "2m".to_string(),
            ssl_mode: "require".to_string(),
            statement_timeout: "1m".to_string(),
            lock_timeout: "30s".to_string(),
            auto_route_replicas: false,
            show_replica_lag: true,

            // Diagnostics
            diag_show_on_launch: true,
            diag_buffer_size: "2,000".to_string(),
            diag_persist: true,
            slow_query_threshold: "500ms".to_string(),
            render_budget_warn: "16ms".to_string(),
            track_ctid_conflicts: true,

            // Updates
            update_channel: "Stable".to_string(),
            check_frequency: "Daily".to_string(),
            auto_install_updates: true,
        }
    }
}

impl AppSettings {
    pub fn normalize(&mut self) {
        if !matches!(self.appearance.as_str(), "system" | "dark" | "light") {
            self.appearance = if self.dark_mode { "dark" } else { "light" }.to_string();
        }
        self.default_row_limit = self.default_row_limit.clamp(1, 1_000_000);
        self.font_size = self.font_size.clamp(9.0, 24.0);
        if self.data_timezone.trim().is_empty() {
            self.data_timezone = "Asia/Seoul".to_string();
        }
        // New fields
        if !matches!(self.density.as_str(), "compact" | "default" | "comfortable") {
            self.density = "default".to_string();
        }
        self.tab_size = self.tab_size.clamp(1, 8);
        self.pool_min = self.pool_min.clamp(1, 50);
        self.pool_max = self.pool_max.clamp(self.pool_min, 100);
    }
}

fn settings_file() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine config directory");
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.join("settings.toml")
}

pub fn load_settings() -> AppSettings {
    let path = settings_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => {
            let had_appearance = data.contains("appearance");
            let mut settings: AppSettings = toml::from_str(&data).unwrap_or_default();
            if !had_appearance {
                settings.appearance = if settings.dark_mode { "dark" } else { "light" }.to_string();
            }
            settings.normalize();
            settings
        }
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(settings: &AppSettings) {
    let path = settings_file();
    if let Ok(data) = toml::to_string_pretty(settings) {
        std::fs::write(path, data).ok();
    }
}

#[cfg(test)]
mod settings_serde_tests {
    use super::*;

    #[test]
    fn unsafe_ctid_defaults_false_when_field_absent_in_toml() {
        // Plan v7 Phase 1.3 — backward-compat: 기존 settings.toml 에 unsafe_ctid
        // 키 없으면 #[serde(default)] 가 false 로 deserialize.
        let toml_without_field = r#"
            appearance = "system"
            dark_mode = true
            font_size = 13.0
            default_row_limit = 1000
            auto_commit = true
            confirm_destructive = true
            language = "en"
            show_objects_under_schema = true
            show_objects_under_table = true
            use_default_object_font = true
            safe_confirm_dialog = true
            ask_before_closing_queries = true
            ask_before_closing_tables = true
            show_function_wizard = true
            share_usage_data = false
            auto_check_updates = true
            include_system_profile = false
            open_new_queries_in_tabs = true
            enable_code_completion = true
            code_completion_popup = true
            show_line_numbers = true
            enable_auto_recovery = true
            ai_assistant_enabled = false
            backup_directory = ""
            data_timezone = "Asia/Seoul"
        "#;
        let parsed: AppSettings =
            toml::from_str(toml_without_field).expect("legacy toml must parse");
        assert!(!parsed.unsafe_ctid, "absent field defaults to false");
    }

    #[test]
    fn unsafe_ctid_round_trips_when_set_true() {
        let original = AppSettings {
            unsafe_ctid: true,
            ..AppSettings::default()
        };
        let serialized = toml::to_string(&original).expect("serialize");
        let restored: AppSettings = toml::from_str(&serialized).expect("deserialize");
        assert!(restored.unsafe_ctid, "true 가 round-trip 후에도 보존");
    }

    #[test]
    fn backup_directory_round_trips() {
        // US-J3 — backup_directory 가 settings.toml 직렬화/역직렬화 round-trip 보존
        let original = AppSettings {
            backup_directory: "/Users/test/Documents/Backups".to_string(),
            ..AppSettings::default()
        };
        let serialized = toml::to_string(&original).expect("serialize");
        let restored: AppSettings = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(restored.backup_directory, "/Users/test/Documents/Backups");
    }
}
