//! BI (Business Intelligence) backend — column statistics, group-by, pivot
//! reshape helpers.
//!
//! Plan v7 Phase 4c — UI 진입점은 `src/ui/objects/bi.rs`. 본 모듈은 pure
//! function helpers 만 호스트 (testable).

pub mod aggregate;
pub mod card;
