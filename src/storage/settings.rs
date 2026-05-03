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
