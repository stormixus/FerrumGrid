//! EXPLAIN 플랜 트리 뷰어 (floating window).
//!
//! `state.explain_plan` (파싱된 `PlanNode` 트리) 을 들여쓰기된 노드 트리로 렌더.
//! 각 노드: 노드 타입, 대상 relation, 비용(Total Cost), 예상 행 수(Plan Rows),
//! ANALYZE 시 실제 행 수(Actual Rows). 비용이 큰 노드는 강조.

use eframe::egui::{self, RichText};

use crate::db::explain::PlanNode;
use crate::i18n::t;
use crate::state::AppState;
use crate::storage::settings::AppSettings;
use crate::ui::theme;

pub fn render_explain_window(ctx: &egui::Context, state: &mut AppState, settings: &AppSettings) {
    if !state.show_explain_window {
        return;
    }
    let Some(plan) = state.explain_plan.clone() else {
        return;
    };

    // AI 조언 작업 결과 수거.
    if let Some(res) = state.explain_advice_job.lock().ok().and_then(|mut g| g.result.take()) {
        match res {
            Ok(advice) => state.explain_advice = Some(advice),
            Err(e) => state.explain_advice = Some(format!("AI error: {e}")),
        }
    }
    let advice_running = state.explain_advice_job.lock().map(|g| g.running).unwrap_or(false);

    // 트리 전체에서 최대 비용 — heat 색상 기준.
    let max_cost = max_total_cost(&plan).unwrap_or(0.0);

    let mut open = true;
    let mut interpret = false;
    egui::Window::new(t("explain_window_title"))
        .open(&mut open)
        .default_size([560.0, 460.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(t("explain_window_hint"))
                        .color(theme::text_muted())
                        .size(11.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(!advice_running, theme::secondary_button(&t("explain_interpret")))
                        .clicked()
                    {
                        interpret = true;
                    }
                    if advice_running {
                        ui.spinner();
                    }
                });
            });
            ui.separator();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    render_node(ui, &plan, 0, max_cost);
                    if let Some(advice) = &state.explain_advice {
                        ui.add_space(theme::SPACE_MD);
                        ui.separator();
                        ui.label(
                            RichText::new(t("explain_advice_title"))
                                .color(theme::ACCENT_PURPLE)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(theme::SPACE_XS);
                        ui.label(RichText::new(advice).size(11.5).color(theme::text_secondary()));
                    }
                });
        });

    if interpret {
        start_interpret_job(ctx, state, settings, &plan);
    }
    if !open {
        state.show_explain_window = false;
    }
}

/// 플랜 트리를 들여쓰기 텍스트로 직렬화 (AI 입력용).
fn plan_to_text(node: &PlanNode, depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
    out.push_str(&node.node_type);
    if let Some(rel) = &node.relation {
        out.push_str(&format!(" on {rel}"));
    }
    if let Some(c) = node.total_cost {
        out.push_str(&format!(" cost={c:.1}"));
    }
    if let Some(r) = node.plan_rows {
        out.push_str(&format!(" rows={r:.0}"));
    }
    if let Some(a) = node.actual_rows {
        out.push_str(&format!(" actual={a:.0}"));
    }
    out.push('\n');
    for child in &node.children {
        plan_to_text(child, depth + 1, out);
    }
}

fn start_interpret_job(
    ctx: &egui::Context,
    state: &AppState,
    settings: &AppSettings,
    plan: &PlanNode,
) {
    let job = state.explain_advice_job.clone();
    {
        let mut g = job.lock().expect("advice job lock");
        if g.running {
            return;
        }
        g.running = true;
        g.result = None;
    }
    let mut plan_text = String::new();
    plan_to_text(plan, 0, &mut plan_text);
    let ctx = ctx.clone();
    let settings_clone = settings.clone();
    std::thread::spawn(move || {
        let res = crate::ai::interpret_plan(&plan_text, &settings_clone);
        if let Ok(mut g) = job.lock() {
            g.running = false;
            g.result = Some(res);
        }
        ctx.request_repaint();
    });
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

/// EXPLAIN 텍스트에서 Seq Scan 을 찾고 추천 인덱스 DDL을 반환.
pub fn recommend_indexes_from_plan(plan_text: &str) -> Vec<String> {
    let mut recs = Vec::new();
    let mut current_table: Option<String> = None;
    for line in plan_text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("Seq Scan on ") {
            if let Some(name) = rest.split_whitespace().next() {
                current_table = Some(name.split('.').last().unwrap_or(name).to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("Index Scan using ") {
            if let Some(idx) = rest.split_whitespace().next() {
                recs.push(format!("-- detected index: {idx}"));
            }
        }
        if let Some(table) = current_table.as_ref() {
            if line.contains("rows=") && line.contains("Seq Scan") {
                recs.push(format!(
                    "CREATE INDEX ON {} (id);  -- approximate recommendation",
                    table
                ));
                current_table = None;
            }
        }
    }
    recs.dedup();
    recs
}
