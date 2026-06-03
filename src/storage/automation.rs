//! 자동화(예약 SQL) 작업의 디스크 영속화.
//!
//! `AutomationStore` 는 런타임에 in-memory (`Arc<RwLock<…>>`) 로 유지되지만,
//! 앱 재시작 시 예약 작업이 사라지지 않도록 JSON 파일로 저장/로드한다.
//! `backups.rs` / `history.rs` 와 동일한 패턴.

use std::path::PathBuf;

use crate::automation::scheduler::ScheduledTask;

fn config_dir() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine config directory");
    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(config_dir).ok();
    config_dir.to_path_buf()
}

fn automation_file() -> PathBuf {
    config_dir().join("automation.json")
}

/// 저장된 예약 작업 로드. 파일이 없거나 파싱 실패 시 빈 목록.
pub fn load_tasks() -> Vec<ScheduledTask> {
    let path = automation_file();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// 예약 작업 전체를 디스크에 저장 (mutation 직후 호출).
pub fn save_tasks(tasks: &[ScheduledTask]) {
    let path = automation_file();
    if let Ok(data) = serde_json::to_string_pretty(tasks) {
        std::fs::write(path, data).ok();
    }
}
