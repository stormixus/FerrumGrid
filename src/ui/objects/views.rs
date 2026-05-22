//! View / Materialized View display helpers.
//!
//! Plan v7 Phase 1.95b3 cut-over (from `super::mod.rs`). Phase 2 의 Create/
//! Replace View UI 가 추가되면 본 모듈에 함께 위치한다 (`render_view_create_form`
//! 등). 현재는 view-list helper 3종만 호스트.

use eframe::egui::Color32;

use crate::i18n::t;
use crate::state::MainView;
use crate::ui::{icons_svg, theme};

/// MainView → (title, subtitle, accent color) 매핑. tab 헤더 / breadcrumb 에서
/// 사용.
pub(super) fn view_copy(view: MainView) -> (String, String, Color32) {
    match view {
        MainView::Table => (
            t("objects_tables_title"),
            t("objects_tables_subtitle"),
            theme::accent_color(),
        ),
        MainView::View => (
            t("objects_views_title"),
            t("objects_views_subtitle"),
            theme::ACCENT_BLUE,
        ),
        MainView::MaterializedView => (
            t("objects_materialized_title"),
            t("objects_materialized_subtitle"),
            theme::accent_color(),
        ),
        MainView::Function => (
            t("objects_functions_title"),
            t("objects_functions_subtitle"),
            theme::ACCENT_YELLOW,
        ),
        MainView::User => (
            t("objects_users_title"),
            t("objects_users_subtitle"),
            theme::accent_color_light(),
        ),
        MainView::Backup => (
            t("objects_backup_title"),
            t("objects_backup_subtitle"),
            theme::text_muted(),
        ),
        MainView::Automation => (
            t("objects_automation_title"),
            t("objects_automation_subtitle"),
            theme::accent_color(),
        ),
        MainView::Model => (
            t("objects_model_title"),
            t("objects_model_subtitle"),
            theme::accent_color(),
        ),
        MainView::BI => (
            t("objects_bi_title"),
            t("objects_bi_subtitle"),
            theme::ACCENT_RED,
        ),
        MainView::Connection => (
            t("objects_connections_title"),
            t("objects_connections_subtitle"),
            theme::accent_color(),
        ),
        MainView::Query => (
            t("objects_query_title"),
            t("objects_query_subtitle"),
            theme::ACCENT_BLUE,
        ),
        MainView::Data => (
            t("objects_data_title"),
            t("objects_data_subtitle"),
            theme::accent_color(),
        ),
    }
}

/// MainView → (icon SVG, icon i18n key) 매핑.
pub(super) fn view_icon(view: MainView) -> (&'static str, &'static str) {
    match view {
        MainView::Table => (icons_svg::TABLE, "objects_title_table"),
        MainView::View => (icons_svg::VIEW, "objects_title_view"),
        MainView::MaterializedView => (icons_svg::MATERIALIZED_VIEW, "objects_title_materialized"),
        MainView::Function => (icons_svg::FUNCTION, "objects_title_function"),
        MainView::User => (icons_svg::USER, "objects_title_user"),
        MainView::Backup => (icons_svg::BACKUP, "objects_title_backup"),
        MainView::Automation => (icons_svg::AUTOMATION, "objects_title_automation"),
        MainView::Model => (icons_svg::MODEL, "objects_title_model"),
        MainView::BI => (icons_svg::BI, "objects_title_bi"),
        MainView::Connection => (icons_svg::CONNECTION, "objects_title_connection"),
        MainView::Query => (icons_svg::QUERY, "objects_title_query"),
        MainView::Data => (icons_svg::TABLE, "objects_title_data"),
    }
}

/// PostgreSQL table_type 문자열 → accent color 매핑 (table row 의 type chip).
pub(super) fn table_type_color(table_type: &str) -> Color32 {
    match table_type {
        "VIEW" => theme::ACCENT_BLUE,
        "MATERIALIZED VIEW" => theme::accent_color(),
        _ => theme::accent_color(),
    }
}
