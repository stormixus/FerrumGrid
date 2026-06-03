//! EXPLAIN 플랜 트리 뷰어 (floating window).
//!
//! `state.explain_plan` (파싱된 `PlanNode` 트리) 을 들여쓰기된 노드 트리로 렌더.
//! 각 노드: 노드 타입, 대상 relation, 비용(Total Cost), 예상 행 수(Plan Rows),
//! ANALYZE 시 실제 행 수(Actual Rows). 비용이 큰 노드는 강조.

use eframe::egui::{self, RichText};

use crate::db::explain::PlanNode;
use crate::i18n::t;
use crate::state::AppState;
use crate::ui::theme;

pub fn render_explain_window(ctx: &egui::Context, state: &mut AppState) {
    if !state.show_explain_window {
        return;
    }
    let Some(plan) = state.explain_plan.clone() else {
        return;
    };

    // 트리 전체에서 최대 비용 — heat 색상 기준.
    let max_cost = max_total_cost(&plan).unwrap_or(0.0);

    let mut open = true;
    egui::Window::new(t("explain_window_title"))
        .open(&mut open)
        .default_size([520.0, 420.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(t("explain_window_hint"))
                    .color(theme::text_muted())
                    .size(11.0),
            );
            ui.separator();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_node(ui, &plan, 0, max_cost);
                });
        });

    if !open {
        state.show_explain_window = false;
    }
}

fn max_total_cost(node: &PlanNode) -> Option<f64> {
    let mut max = node.total_cost;
    for child in &node.children {
        if let Some(c) = max_total_cost(child) {
            max = Some(max.map_or(c, |m: f64| m.max(c)));
        }
    }
    max
}

fn render_node(ui: &mut egui::Ui, node: &PlanNode, depth: usize, max_cost: f64) {
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 16.0);

        // 비용 비율로 heat 색상 (가장 비싼 노드 = 빨강).
        let heat = node
            .total_cost
            .filter(|_| max_cost > 0.0)
            .map(|c| (c / max_cost) as f32)
            .unwrap_or(0.0);
        let node_color = if heat > 0.66 {
            theme::ACCENT_RED
        } else if heat > 0.33 {
            theme::ACCENT_YELLOW
        } else {
            theme::accent_color()
        };

        ui.label(
            RichText::new(&node.node_type)
                .color(node_color)
                .strong()
                .size(12.0),
        );
        if let Some(rel) = &node.relation {
            ui.label(
                RichText::new(format!("on {rel}"))
                    .color(theme::text_secondary())
                    .monospace()
                    .size(11.0),
            );
        }
        let mut meta: Vec<String> = Vec::new();
        if let Some(cost) = node.total_cost {
            meta.push(format!("cost {cost:.1}"));
        }
        if let Some(rows) = node.plan_rows {
            meta.push(format!("rows≈{rows:.0}"));
        }
        if let Some(actual) = node.actual_rows {
            meta.push(format!("actual {actual:.0}"));
        }
        if !meta.is_empty() {
            ui.label(
                RichText::new(meta.join(" · "))
                    .color(theme::text_muted())
                    .size(10.5),
            );
        }
    });
    for child in &node.children {
        render_node(ui, child, depth + 1, max_cost);
    }
}
