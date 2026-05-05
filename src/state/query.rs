//! Query workspace state (editor tabs, transaction lifecycle).
//!
//! Plan v7 Phase 1.95c1 placeholder → Phase 3b 에서 dangling tx state 가
//! `AppState.explicit_tx_*` 필드로 직접 추가됨.
//!
//! 향후 cut-over 후보:
//! - `EditorTab` (현재 `crate::types`) → 본 모듈로 이동
//! - `explicit_tx_active` / `explicit_tx_started` / `explicit_tx_warned` →
//!   `QueryTxState` struct 로 그룹화
