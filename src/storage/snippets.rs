//! 저장된 SQL 스니펫(이름 + 본문)의 디스크 영속화.
//!
//! 개인용 진단 쿼리 라이브러리(lock 체크, bloat 쿼리 등)를 이름으로 보관/삽입.
//! `history.rs` 와 동일한 JSON load/save 패턴.

use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snippet {
    pub id: uuid::Uuid,
    pub name: String,
    pub body: String,
}

impl Snippet {
    pub fn new(name: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            body: body.into(),
        }
    }
}

fn snippets_file() -> PathBuf {
    let dirs = directories::ProjectDirs::from("com", "ferrumgrid", "FerrumGrid")
        .expect("failed to determine data directory");
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir).ok();
    data_dir.join("snippets.json")
}

pub fn load_snippets() -> Vec<Snippet> {
    let path = snippets_file();
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_snippets(snippets: &[Snippet]) {
    let path = snippets_file();
    if let Ok(data) = serde_json::to_string_pretty(snippets) {
        std::fs::write(path, data).ok();
    }
}
