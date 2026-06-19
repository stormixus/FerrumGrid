//! 플러그인 API 골격.
//!
//! 향후 서드파티가 FerrumGrid 에 기능을 추가할 수 있는 최소 훅을 정의한다.
//! 현재는 내장 플러그인 등록 경로를 먼저 제공하고, 다음 마일스톤에서 동적 로딩을
//! (cdylib + libloading) 얹을 수 있는 형태로 트레이트를 고정한다.

use std::sync::Arc;

/// 플러그인이 제공할 수 있는 훅 (현재는 1종 — 메뉴 항목 등록).
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    /// 좌측 트리 패널에 표시될 메뉴 항목을 반환. None 이면 노출 안 함.
    fn tree_menu_label(&self) -> Option<String> {
        None
    }
    /// 트리 메뉴 항목이 클릭됐을 때 실행될 액션.
    fn on_tree_menu_click(&self) -> Option<String> {
        None
    }
}

/// 플러그인 레지스트리. AppState 에 Arc 로 보관.
#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut registry = Self {
            plugins: Vec::new(),
        };
        registry.register(Arc::new(BuiltinPluginStatus));
        registry
    }
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn tree_labels(&self) -> Vec<String> {
        self.plugins
            .iter()
            .filter_map(|p| p.tree_menu_label())
            .collect()
    }

    pub fn plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }
}

struct BuiltinPluginStatus;

impl Plugin for BuiltinPluginStatus {
    fn name(&self) -> &str {
        "builtin-status"
    }

    fn tree_menu_label(&self) -> Option<String> {
        Some("Plugin API ready".to_string())
    }

    fn on_tree_menu_click(&self) -> Option<String> {
        Some("Plugin API is registered and ready for extensions.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test"
        }
        fn tree_menu_label(&self) -> Option<String> {
            Some("Test Plugin".to_string())
        }
    }

    #[test]
    fn registry_collects_labels() {
        let mut reg = PluginRegistry::default();
        reg.register(Arc::new(TestPlugin));
        assert_eq!(
            reg.tree_labels(),
            vec!["Plugin API ready".to_string(), "Test Plugin".to_string()]
        );
    }
}
