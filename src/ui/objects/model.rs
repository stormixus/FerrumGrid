//! ER model view.
//!
//! Plan v7 Phase 1.95b2 cut-over (from `super::mod.rs`).

use eframe::egui;

use crate::db::bridge::DbBridge;
use crate::state::AppState;

use super::{active_conn, selected_schema_or_public, ObjectAction};

pub(super) fn render_model_tools(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
) -> Option<ObjectAction> {
    let _ = active_conn(state)?;
    if state.er_diagram.selected_schema.is_empty() {
        state.er_diagram.selected_schema = selected_schema_or_public(state);
    }
    crate::ui::er_diagram::render_er_diagram(ui, state, bridge);
    None
}
