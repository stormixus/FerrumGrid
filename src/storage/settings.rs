use std::path::PathBuf;

use crate::ui::theme::ThemeMode;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_row_limit")]
    pub default_row_limit: usize,
    #[serde(default = "default_true")]
    pub auto_commit: bool,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    #[serde(default = "default_true")]
    pub sidebar_visible: bool,
    #[serde(default = "default_true")]
    pub result_panel_visible: bool,
}

fn default_font_size() -> f32 {
    13.0
}
fn default_row_limit() -> usize {
    1000
}
fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Auto,
            font_size: default_font_size(),
            default_row_limit: default_row_limit(),
            auto_commit: true,
            confirm_destructive: true,
            sidebar_visible: true,
            result_panel_visible: true,
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
        Ok(data) => toml::from_str(&data).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

pub fn save_settings(settings: &AppSettings) {
    let path = settings_file();
    if let Ok(data) = toml::to_string_pretty(settings) {
        std::fs::write(path, data).ok();
    }
}
