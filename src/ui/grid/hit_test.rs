//! Grid hit-test (pixel coords → CellKey) (Plan v7 Phase 1.95c — TODO).
//!
//! Will host: pointer 좌표 → row/col 변환 + bounds clamp 헬퍼. Currently all
//! logic remains in `super::mod.rs`. Target ≤400 lines.
