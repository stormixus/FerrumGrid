//! Clipboard paste decomposition (text → grid cells) (Plan v7 Phase 1.95c
//! — TODO).
//!
//! Will host: paste text 분해 (탭/줄바꿈) + cell 매핑 + bulk-edit dirty 마크.
//! Currently all logic remains in `super::mod.rs`. Target ≤600 lines.
