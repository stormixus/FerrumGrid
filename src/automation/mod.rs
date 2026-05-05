//! Automation — scheduled SQL execution.
//!
//! Plan v7 Phase 4b — 즉시 실행 + 예약 (tokio::spawn + interval) infrastructure.
//! 본 모듈은 backend types + scheduling 로직만 호스트. UI 진입점은
//! `src/ui/objects/automation.rs`, runtime scheduler 는 Phase 4b3 의
//! `runner.rs` (별도 sub-iteration).

pub mod runner;
pub mod scheduler;
