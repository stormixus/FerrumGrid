use eframe::egui::{self, Color32, CornerRadius, Id, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

use crate::db::bridge::{DbBridge, DbCommand};
use crate::i18n::{t, tf};
use crate::state::{
    AppState, ConnectionStatus, DataSource, MainView, build_data_select_sql_with_columns,
};
use crate::types::{ColumnInfo, ConnectionId};
use crate::ui::grid::request_table_columns_for_data;
use crate::ui::theme;


const CARD_WIDTH: f32 = 300.0;
const CARD_HEADER_HEIGHT: f32 = 42.0;
const CARD_ROW_HEIGHT: f32 = 23.0;
const CARD_FOOTER_HEIGHT: f32 = 22.0;
const CARD_MAX_ROWS: usize = 8;
const CARD_GAP_X: f32 = 160.0;
const CARD_GAP_Y: f32 = 88.0;
const CARD_START_X: f32 = 44.0;
const CARD_START_Y: f32 = 44.0;

#[derive(Debug, Clone)]
pub struct TableCard {
    pub schema: String,
    pub table_name: String,
    pub table_type: String,
    pub pos: Pos2,
    pub columns: Vec<ColumnInfo>,
    pub is_dragging: bool,
}

impl TableCard {
    pub fn height(&self) -> f32 {
        let visible_rows = self.columns.len().min(CARD_MAX_ROWS);
        let footer = if self.columns.len() > CARD_MAX_ROWS {
            CARD_FOOTER_HEIGHT
        } else {
            8.0
        };
        CARD_HEADER_HEIGHT + visible_rows as f32 * CARD_ROW_HEIGHT + footer
    }

    pub fn full_id(&self) -> String {
        format!("{}.{}", self.schema, self.table_name)
    }

    pub fn column_y(&self, col_idx: usize) -> f32 {
        let visible_idx = col_idx.min(CARD_MAX_ROWS.saturating_sub(1));
        self.pos.y
            + CARD_HEADER_HEIGHT
            + visible_idx as f32 * CARD_ROW_HEIGHT
            + CARD_ROW_HEIGHT / 2.0
    }
}

pub use crate::types::ForeignKey;

#[derive(Debug, Clone, Default)]
pub struct ERDiagramState {
    pub cards: HashMap<String, TableCard>,
    pub foreign_keys: Vec<ForeignKey>,
    pub selected_schema: String,
    pub last_loaded_schema: String,
    pub search: String,
    pub selected_table: Option<String>,
    pub show_diagram: bool,
    pub pan_offset: Vec2,
    pub is_panning: bool,
    pub last_mouse_pos: Option<Pos2>,
    pub zoom: f32,
    pub last_click_time: Option<std::time::Instant>,
    pub last_clicked_id: Option<String>,
}

impl ERDiagramState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }

    pub fn layout_cards(&mut self) {
        self.layout_cards_with_width(1200.0);
    }

    pub fn layout_cards_with_width(&mut self, available_width: f32) {
        let mut ids: Vec<String> = self.cards.keys().cloned().collect();
        ids.sort_by(|a, b| {
            let degree_b = relation_degree(b, &self.foreign_keys);
            let degree_a = relation_degree(a, &self.foreign_keys);
            degree_b.cmp(&degree_a).then_with(|| a.cmp(b))
        });

        let len = ids.len().max(1);
        let cols_that_fit = ((available_width - CARD_START_X) / (CARD_WIDTH + CARD_GAP_X))
            .floor()
            .max(1.0) as usize;
        let rows = (len as f32 / cols_that_fit as f32).ceil().max(1.0) as usize;
        let mut row_heights = vec![0.0_f32; rows];

        for (idx, id) in ids.iter().enumerate() {
            let (row, _) = layout_slot(idx, cols_that_fit);
            if let Some(card) = self.cards.get_mut(id) {
                row_heights[row] = row_heights[row].max(card.height());
                card.is_dragging = false;
            }
        }

        let mut row_offsets = vec![CARD_START_Y; rows];
        let mut next_y = CARD_START_Y;
        for (idx, height) in row_heights.iter().enumerate() {
            row_offsets[idx] = next_y;
            next_y += *height + CARD_GAP_Y;
        }

        for (idx, id) in ids.iter().enumerate() {
            let (row, col) = layout_slot(idx, cols_that_fit);
            if let Some(card) = self.cards.get_mut(id) {
                card.pos = Pos2::new(
                    CARD_START_X + col as f32 * (CARD_WIDTH + CARD_GAP_X),
                    row_offsets[row],
                );
            }
        }
    }

    /// Force-directed layout: simple spring/electrical simulation to position tables based on FK relationships.
    pub fn apply_force_directed_layout(&mut self, canvas_width: f32, canvas_height: f32) {
        let mut rng_state: u64 = 0xdeadbeef;
        let mut rng = || -> f32 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 33) as u32 as f32) / (u32::MAX as f32)
        };
        for (_, card) in self.cards.iter_mut() {
            if card.pos.x == 0.0 && card.pos.y == 0.0 {
                card.pos.x = rng() * canvas_width;
                card.pos.y = rng() * canvas_height;
            }
        }
        let iterations = 200;
        let repulsion = 5000.0;
        let spring_length = 200.0;
        let spring_k = 0.05;
        let damping = 0.85;
        let mut velocities: HashMap<String, Vec2> = self.cards.keys().map(|id| (id.clone(), Vec2::ZERO)).collect();
        for _ in 0..iterations {
            let mut forces: HashMap<String, Vec2> = self.cards.keys().map(|id| (id.clone(), Vec2::ZERO)).collect();
            let ids: Vec<String> = self.cards.keys().cloned().collect();
            for i in 0..ids.len() {
                for j in (i+1)..ids.len() {
                    let id_i = &ids[i];
                    let id_j = &ids[j];
                    let pos_i = self.cards[id_i].pos;
                    let pos_j = self.cards[id_j].pos;
                    let delta = pos_i - pos_j;
                    let dist = delta.length().max(1.0);
                    let force = repulsion / (dist * dist);
                    let dir = delta / dist;
                    let f_i = forces.get_mut(id_i).unwrap();
                    *f_i = *f_i + dir * force;
                    let f_j = forces.get_mut(id_j).unwrap();
                    *f_j = *f_j - dir * force;
                }
            }
            for fk in &self.foreign_keys {
                let source_id = format!("{}.{}", fk.source_schema, fk.source_table);
                let target_id = format!("{}.{}", fk.target_schema, fk.target_table);
                if let (Some(pos_s), Some(pos_t)) = (self.cards.get(&source_id).map(|c| c.pos), self.cards.get(&target_id).map(|c| c.pos)) {
                    let delta = pos_t - pos_s;
                    let dist = delta.length();
                    let displacement = dist - spring_length;
                    let dir = if dist > 0.0 { delta / dist } else { Vec2::ZERO };
                    let force = spring_k * displacement;
                    let f_s = forces.get_mut(&source_id).unwrap();
                    *f_s = *f_s + dir * force;
                    let f_t = forces.get_mut(&target_id).unwrap();
                    *f_t = *f_t - dir * force;
                }
            }
            for id in &ids {
                let force = forces[id];
                let vel = velocities.get_mut(id).unwrap();
                *vel = *vel + force;
                *vel = *vel * damping;
                let pos = self.cards.get_mut(id).unwrap();
                pos.pos = pos.pos + *vel;
                pos.pos.x = pos.pos.x.clamp(0.0, (canvas_width - CARD_WIDTH).max(CARD_WIDTH));
                pos.pos.y = pos.pos.y.clamp(0.0, (canvas_height - pos.height()).max(pos.height()));
            }
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for (_, card) in self.cards.iter() {
            min_x = min_x.min(card.pos.x);
            min_y = min_y.min(card.pos.y);
            max_x = max_x.max(card.pos.x + CARD_WIDTH);
            max_y = max_y.max(card.pos.y + card.height());
        }
        let dx = (canvas_width - (max_x - min_x)) / 2.0 - min_x;
        let dy = (canvas_height - (max_y - min_y)) / 2.0 - min_y;
        for (_, card) in self.cards.iter_mut() {
            card.pos.x += dx;
            card.pos.y += dy;
        }
    }

    pub fn get_card(&self, schema: &str, table: &str) -> Option<&TableCard> {
        self.cards.get(&format!("{}.{}", schema, table))
    }

    pub fn get_column_index(&self, schema: &str, table: &str, column: &str) -> Option<usize> {
        self.get_card(schema, table)
            .and_then(|card| card.columns.iter().position(|c| c.name == column))
    }
}

fn layout_slot(idx: usize, cols: usize) -> (usize, usize) {
    (idx / cols, idx % cols)
}

fn relation_degree(card_id: &str, foreign_keys: &[ForeignKey]) -> usize {
    foreign_keys
        .iter()
        .filter(|fk| {
            let source = format!("{}.{}", fk.source_schema, fk.source_table);
            let target = format!("{}.{}", fk.target_schema, fk.target_table);
            source == card_id || target == card_id
        })
        .count()
}

fn draw_bezier_connection(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    label: &str,
    color: Color32,
    stroke_width: f32,
    font_size: f32,
) {
    let control_offset = ((to.x - from.x) * 0.5).abs().max(50.0);

    let cp1 = Pos2::new(from.x + control_offset, from.y);
    let cp2 = Pos2::new(to.x - control_offset, to.y);

    let mut points = Vec::with_capacity(32);
    for i in 0..=32 {
        let t = i as f32 / 32.0;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;

        let x = mt3 * from.x + 3.0 * mt2 * t * cp1.x + 3.0 * mt * t2 * cp2.x + t3 * to.x;
        let y = mt3 * from.y + 3.0 * mt2 * t * cp1.y + 3.0 * mt * t2 * cp2.y + t3 * to.y;
        points.push(Pos2::new(x, y));
    }

    let mid = points[points.len() / 2];
    painter.add(egui::Shape::line(points, Stroke::new(stroke_width, color)));
    painter.circle_filled(from, 3.0, color);
    painter.circle_filled(to, 3.0, color);

    let arrow_size = 8.0;
    let angle = (to.y - from.y).atan2(to.x - from.x);
    let arrow_p1 = Pos2::new(
        to.x - arrow_size * (angle + 0.5).cos(),
        to.y - arrow_size * (angle + 0.5).sin(),
    );
    let arrow_p2 = Pos2::new(
        to.x - arrow_size * (angle - 0.5).cos(),
        to.y - arrow_size * (angle - 0.5).sin(),
    );

    painter.line_segment([to, arrow_p1], Stroke::new(stroke_width, color));
    painter.line_segment([to, arrow_p2], Stroke::new(stroke_width, color));

    if !label.is_empty() {
        let label_text = label.chars().take(24).collect::<String>();
        let label_rect =
            Rect::from_center_size(mid + Vec2::new(0.0, -12.0), Vec2::new(126.0, 20.0));
        painter.rect_filled(
            label_rect,
            CornerRadius::same(theme::RADIUS_SM),
            theme::with_alpha(theme::bg_shell(), if theme::is_dark() { 220 } else { 236 }),
        );
        painter.rect_stroke(
            label_rect,
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, theme::border_subtle()),
            egui::StrokeKind::Inside,
        );
        painter.text(
            label_rect.center(),
            egui::Align2::CENTER_CENTER,
            label_text,
            egui::FontId::monospace(font_size),
            theme::text_muted(),
        );
    }
}

fn draw_canvas_grid(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let spacing = (34.0 * zoom).clamp(18.0, 54.0);
    let color = theme::with_alpha(
        theme::border_subtle(),
        if theme::is_dark() { 64 } else { 96 },
    );

    let mut x = rect.left() + pan.x.rem_euclid(spacing);
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(0.6, color),
        );
        x += spacing;
    }

    let mut y = rect.top() + pan.y.rem_euclid(spacing);
    while y < rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(0.6, color),
        );
        y += spacing;
    }
}

fn paint_canvas_message(painter: &egui::Painter, rect: Rect, title: &str, subtitle: &str) {
    painter.text(
        rect.center() + Vec2::new(0.0, -12.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(15.0),
        theme::text_secondary(),
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 14.0),
        egui::Align2::CENTER_CENTER,
        subtitle,
        egui::FontId::proportional(11.0),
        theme::text_muted(),
    );
}

fn visible_card_ids(er_state: &ERDiagramState) -> HashSet<String> {
    let needle = er_state.search.trim().to_lowercase();
    if needle.is_empty() {
        return er_state.cards.keys().cloned().collect();
    }

    er_state
        .cards
        .iter()
        .filter_map(|(id, card)| {
            let table_match = card.table_name.to_lowercase().contains(&needle)
                || card.schema.to_lowercase().contains(&needle);
            let column_match = card.columns.iter().any(|col| {
                col.name.to_lowercase().contains(&needle)
                    || col.data_type.to_lowercase().contains(&needle)
            });
            (table_match || column_match).then(|| id.clone())
        })
        .collect()
}

struct CardDrawParams {
    canvas_rect: Rect,
    pan_offset: Vec2,
    zoom: f32,
    selected: bool,
    drag_interaction: bool,
}

#[derive(Default)]
struct CardInteraction {
    drag_delta: Option<Vec2>,
    clicked: bool,
}

fn draw_card(
    ui: &mut egui::Ui,
    card: &mut TableCard,
    foreign_keys: &[ForeignKey],
    params: CardDrawParams,
) -> CardInteraction {
    let card_id = Id::new("er_card_").with(card.full_id());
    let screen_pos = world_to_screen(card.pos, params.canvas_rect, params.pan_offset, params.zoom);
    let screen_rect = Rect::from_min_size(
        screen_pos,
        Vec2::new(CARD_WIDTH * params.zoom, card.height() * params.zoom),
    );
    let zoom = params.zoom;
    let selected = params.selected;
    let response = ui.interact(screen_rect, card_id, Sense::click_and_drag());
    let mut interaction = CardInteraction {
        clicked: response.clicked(),
        ..Default::default()
    };

    if params.drag_interaction {
        if response.drag_started() {
            card.is_dragging = true;
        }
        if response.dragged() {
            interaction.drag_delta = Some(response.drag_delta() / params.zoom);
        }
        if response.drag_stopped() {
            card.is_dragging = false;
        }
    }

    let painter = ui.painter().with_clip_rect(params.canvas_rect);
    let hovered = response.hovered();
    let border_color = if params.selected {
        theme::accent_color()
    } else if hovered {
        theme::accent_color_light()
    } else {
        theme::border_default()
    };
    let radius = CornerRadius::same(theme::RADIUS_LG);

    let shadow_rect = screen_rect.translate(Vec2::new(0.0, 5.0 * params.zoom));
    painter.rect_filled(
        shadow_rect,
        radius,
        Color32::from_black_alpha(if theme::is_dark() { 76 } else { 28 }),
    );
    painter.rect_filled(screen_rect, radius, theme::bg_medium());
    painter.rect_stroke(
        screen_rect,
        radius,
        Stroke::new(if selected { 2.0 } else { 1.0 }, border_color),
        egui::StrokeKind::Inside,
    );

    let header_rect = Rect::from_min_size(
        screen_pos,
        Vec2::new(CARD_WIDTH * zoom, CARD_HEADER_HEIGHT * zoom),
    );
    painter.rect_filled(
        header_rect,
        CornerRadius {
            nw: theme::RADIUS_LG,
            ne: theme::RADIUS_LG,
            sw: 0,
            se: 0,
        },
        theme::bg_light(),
    );
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(header_rect.left(), header_rect.bottom() - 1.0),
            Vec2::new(header_rect.width(), 1.0),
        ),
        CornerRadius::ZERO,
        theme::border_subtle(),
    );

    let accent = table_type_color(&card.table_type);
    painter.circle_filled(
        Pos2::new(header_rect.left() + 15.0 * zoom, header_rect.center().y),
        4.0 * zoom,
        accent,
    );
    painter.text(
        Pos2::new(
            header_rect.left() + 28.0 * zoom,
            header_rect.center().y - 6.0 * zoom,
        ),
        egui::Align2::LEFT_CENTER,
        &card.table_name,
        egui::FontId::proportional((13.0 * zoom).clamp(10.0, 15.0)),
        theme::text_primary(),
    );
    painter.text(
        Pos2::new(
            header_rect.left() + 28.0 * zoom,
            header_rect.center().y + 9.0 * zoom,
        ),
        egui::Align2::LEFT_CENTER,
        table_type_label(&card.table_type),
        egui::FontId::proportional((9.5 * zoom).clamp(8.0, 11.0)),
        theme::text_muted(),
    );

    let visible_columns = card.columns.len().min(CARD_MAX_ROWS);
    if visible_columns == 0 {
        painter.text(
            Pos2::new(screen_rect.center().x, header_rect.bottom() + 34.0 * zoom),
            egui::Align2::CENTER_CENTER,
            t("visualizer_loading_columns"),
            egui::FontId::proportional((11.0 * zoom).clamp(9.0, 12.0)),
            theme::text_disabled(),
        );
    }

    for (idx, col) in card.columns.iter().take(CARD_MAX_ROWS).enumerate() {
        let row_top = header_rect.bottom() + idx as f32 * CARD_ROW_HEIGHT * zoom;
        let row_rect = Rect::from_min_size(
            Pos2::new(screen_rect.left() + 8.0 * zoom, row_top),
            Vec2::new(screen_rect.width() - 16.0 * zoom, CARD_ROW_HEIGHT * zoom),
        );

        if idx % 2 == 1 {
            painter.rect_filled(
                row_rect.expand2(Vec2::new(4.0 * zoom, 0.0)),
                CornerRadius::same(theme::RADIUS_SM),
                theme::with_alpha(theme::bg_dark(), if theme::is_dark() { 70 } else { 130 }),
            );
        }

        let is_fk = is_foreign_key_column(card, &col.name, foreign_keys);
        let mut text_x = row_rect.left() + 4.0 * zoom;
        if col.is_primary_key {
            paint_pill(
                &painter,
                Pos2::new(text_x, row_rect.center().y),
                "PK",
                theme::ACCENT_YELLOW,
                zoom,
            );
            text_x += 24.0 * zoom;
        }
        if is_fk {
            paint_pill(
                &painter,
                Pos2::new(text_x, row_rect.center().y),
                "FK",
                theme::ACCENT_BLUE,
                zoom,
            );
            text_x += 24.0 * zoom;
        }

        let column_clip = Rect::from_min_max(
            row_rect.left_top(),
            Pos2::new(row_rect.right() - 102.0 * zoom, row_rect.bottom()),
        );
        painter.with_clip_rect(column_clip).text(
            Pos2::new(text_x, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &col.name,
            egui::FontId::proportional((11.5 * zoom).clamp(8.5, 13.0)),
            if col.is_primary_key {
                theme::ACCENT_YELLOW
            } else {
                theme::text_primary()
            },
        );

        let type_text = format_column_type(col);
        let type_clip = Rect::from_min_max(
            Pos2::new(row_rect.right() - 112.0 * zoom, row_rect.top()),
            row_rect.right_bottom(),
        );
        painter.with_clip_rect(type_clip).text(
            Pos2::new(row_rect.right() - 4.0 * zoom, row_rect.center().y),
            egui::Align2::RIGHT_CENTER,
            type_text,
            egui::FontId::monospace((10.0 * zoom).clamp(8.0, 11.5)),
            theme::text_muted(),
        );
    }

    if card.columns.len() > CARD_MAX_ROWS {
        let hidden = card.columns.len() - CARD_MAX_ROWS;
        let footer_y =
            header_rect.bottom() + visible_columns as f32 * CARD_ROW_HEIGHT * zoom + 11.0 * zoom;
        painter.text(
            Pos2::new(screen_rect.center().x, footer_y),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tf("visualizer_more_columns", &[&hidden.to_string()]),
            egui::FontId::proportional((10.5 * zoom).clamp(8.0, 11.5)),
            theme::text_muted(),
        );
    }

    interaction
}

fn paint_pill(painter: &egui::Painter, left_center: Pos2, label: &str, color: Color32, zoom: f32) {
    let size = Vec2::new(20.0 * zoom, 13.0 * zoom);
    let rect = Rect::from_min_size(Pos2::new(left_center.x, left_center.y - size.y / 2.0), size);
    painter.rect_filled(
        rect,
        CornerRadius::same(theme::RADIUS_SM),
        theme::with_alpha(color, if theme::is_dark() { 34 } else { 52 }),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace((8.0 * zoom).clamp(7.0, 9.0)),
        color,
    );
}

fn is_foreign_key_column(card: &TableCard, column: &str, foreign_keys: &[ForeignKey]) -> bool {
    foreign_keys.iter().any(|fk| {
        fk.source_schema == card.schema
            && fk.source_table == card.table_name
            && fk.source_column == column
    })
}

fn format_column_type(col: &ColumnInfo) -> String {
    if col.is_nullable {
        format!("{}?", col.data_type)
    } else {
        col.data_type.clone()
    }
}

fn table_type_label(table_type: &str) -> &'static str {
    match table_type {
        "VIEW" => "view",
        "MATERIALIZED VIEW" => "materialized view",
        _ => "table",
    }
}

fn table_type_color(table_type: &str) -> Color32 {
    match table_type {
        "VIEW" => theme::ACCENT_BLUE,
        "MATERIALIZED VIEW" => theme::accent_color_light(),
        _ => theme::accent_color(),
    }
}

pub fn render_er_diagram(ui: &mut egui::Ui, state: &mut AppState, bridge: &DbBridge) {
    state.er_diagram.show_diagram = true;
    let active_conn = match state.active_connection {
        Some(id) => id,
        None => return,
    };

    let should_request_schemas = state.connections.get(&active_conn).is_some_and(|conn| {
        matches!(conn.status, ConnectionStatus::Connected { .. })
            && conn.schemas.is_empty()
            && !conn.loading_schemas
    });
    if should_request_schemas {
        if let Some(conn) = state.connections.get_mut(&active_conn) {
            conn.loading_schemas = true;
        }
        bridge.send(crate::db::bridge::DbCommand::ListSchemas {
            conn_id: active_conn,
        });
    }

    let conn = match state.connections.get(&active_conn) {
        Some(c) if matches!(c.status, ConnectionStatus::Connected { .. }) => c,
        _ => return,
    };

    let schema = state.er_diagram.selected_schema.clone();
    if schema.is_empty() && !conn.schemas.is_empty() {
        state.er_diagram.selected_schema = conn.schemas[0].clone();
    }

    // 진단용 로딩 플래그 capture — sync 가 mutate 하기 전 (post-sync 에 사용).
    // tables/foreign_keys 둘 중 하나라도 in-flight 이면 "loading" 상태.
    let schema_for_loading = state.er_diagram.selected_schema.clone();
    let is_schema_loading = !schema_for_loading.is_empty()
        && state.connections.get(&active_conn).is_some_and(|c| {
            c.loading_tables.contains(&schema_for_loading)
                || c.loading_foreign_keys.contains(&schema_for_loading)
        });

    render_toolbar(ui, state, bridge, active_conn);
    sync_schema_visualizer(state, bridge, active_conn);

    ui.separator();

    let available = ui.available_size().max(Vec2::new(640.0, 360.0));
    let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());

    let bg_color = theme::bg_darkest();
    painter.rect_filled(response.rect, CornerRadius::same(0), bg_color);
    draw_canvas_grid(
        &painter,
        response.rect,
        state.er_diagram.pan_offset,
        state.er_diagram.zoom,
    );

    let mouse_pos = ui.ctx().input(|i| i.pointer.hover_pos());

    if response.dragged() && state.er_diagram.is_panning {
        if let (Some(current), Some(last)) = (mouse_pos, state.er_diagram.last_mouse_pos) {
            state.er_diagram.pan_offset += current - last;
        }
    }

    if response.drag_stopped() {
        state.er_diagram.is_panning = false;
    }

    if response.drag_started() && !state.er_diagram.cards.values().any(|c| c.is_dragging) {
        state.er_diagram.is_panning = true;
    }

    state.er_diagram.last_mouse_pos = mouse_pos;

    let any_dragging = state.er_diagram.cards.values().any(|card| card.is_dragging);
    let mut drag_delta = Vec2::ZERO;

    if !any_dragging {
        ui.input(|i| {
            for (key, delta) in [
                (egui::Key::ArrowUp, Vec2::new(0.0, -20.0)),
                (egui::Key::ArrowDown, Vec2::new(0.0, 20.0)),
                (egui::Key::ArrowLeft, Vec2::new(-20.0, 0.0)),
                (egui::Key::ArrowRight, Vec2::new(20.0, 0.0)),
            ] {
                if i.key_pressed(key) {
                    drag_delta += delta;
                }
            }
        });

        if drag_delta != Vec2::ZERO {
            for card in state.er_diagram.cards.values_mut() {
                card.pos += drag_delta;
            }
        }
    }

    let visible_ids = visible_card_ids(&state.er_diagram);
    let foreign_keys = state.er_diagram.foreign_keys.clone();
    draw_foreign_keys(&painter, &state.er_diagram, response.rect, &visible_ids);

    let mut card_clicked = false;
    let mut card_ids: Vec<String> = state.er_diagram.cards.keys().cloned().collect();
    card_ids.sort();
    for card_id in card_ids {
        if !visible_ids.contains(&card_id) {
            continue;
        }
        let selected = state.er_diagram.selected_table.as_deref() == Some(card_id.as_str());
        let Some(card) = state.er_diagram.cards.get_mut(&card_id) else {
            continue;
        };
        let schema = card.schema.clone();
        let table_name = card.table_name.clone();
        let interaction = draw_card(
            ui,
            card,
            &foreign_keys,
            CardDrawParams {
                canvas_rect: response.rect,
                pan_offset: state.er_diagram.pan_offset,
                zoom: state.er_diagram.zoom,
                selected,
                drag_interaction: true,
            },
        );
        if let Some(delta) = interaction.drag_delta {
            card.pos += delta;
        }
        if interaction.clicked {
            card_clicked = true;
            let now = std::time::Instant::now();
            let is_double_click = state
                .er_diagram
                .last_click_time
                .is_some_and(|last_time| {
                    state.er_diagram.last_clicked_id.as_deref() == Some(card_id.as_str())
                        && now.duration_since(last_time) < std::time::Duration::from_millis(300)
                });
            if is_double_click {
                state.er_diagram.last_click_time = None;
                state.er_diagram.last_clicked_id = None;
                open_table_data_from_er(state, bridge, active_conn, &schema, &table_name);
            } else {
                state.er_diagram.last_click_time = Some(now);
                state.er_diagram.last_clicked_id = Some(card_id.clone());
                state.er_diagram.selected_table = Some(card_id);
            }
        }
    }

    if !card_clicked && response.clicked() {
        state.er_diagram.selected_table = None;
    }

    if !visible_ids.is_empty() {
        let mut sorted_visible: Vec<&String> = visible_ids.iter().collect();
        sorted_visible.sort();

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.er_diagram.selected_table = None;
        }

        let tab_forward = ui.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.shift);
        let tab_backward = ui.input(|i| i.key_pressed(egui::Key::Tab) && i.modifiers.shift);
        if tab_forward || tab_backward {
            let current_idx = state
                .er_diagram
                .selected_table
                .as_ref()
                .and_then(|sel| sorted_visible.iter().position(|id| *id == sel));
            let next_idx = match current_idx {
                Some(idx) => {
                    if tab_forward {
                        (idx + 1) % sorted_visible.len()
                    } else {
                        (idx + sorted_visible.len() - 1) % sorted_visible.len()
                    }
                }
                None => 0,
            };
            state.er_diagram.selected_table = Some(sorted_visible[next_idx].clone());
        }
    }

    if state.er_diagram.cards.is_empty() {
        if is_schema_loading || schema_for_loading.is_empty() {
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(response.rect));
            child_ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new(t("visualizer_loading_title"))
                            .color(theme::text_secondary())
                            .size(15.0),
                    );
                    ui.label(
                        RichText::new(t("visualizer_loading_subtitle"))
                            .color(theme::text_muted())
                            .size(11.0),
                    );
                });
            });
        } else {
            paint_canvas_message(
                &painter,
                response.rect,
                &t("visualizer_no_tables_title"),
                &t("visualizer_no_tables_subtitle"),
            );
        }
    } else if visible_ids.is_empty() {
        paint_canvas_message(
            &painter,
            response.rect,
            &t("visualizer_no_matching_tables"),
            &t("visualizer_clear_search_hint"),
        );
    }
}

fn open_table_data_from_er(
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
    schema: &str,
    table_name: &str,
) {
    state.active_connection = Some(conn_id);
    state.current_result = None;
    state.current_result_truncated = false;
    state.begin_data_edit(conn_id, schema, table_name);
    request_table_columns_for_data(state, bridge, conn_id, schema, table_name);
    let source = DataSource {
        conn_id,
        schema: schema.to_string(),
        table: table_name.to_string(),
        filter: None,
    };
    let limit = state.data_edit.page_limit;
    let columns = state.data_columns_for_source(&source);
    let sql =
        build_data_select_sql_with_columns(&source, &state.data_edit.sort, limit, 0, &columns);
    bridge.send(DbCommand::ExecuteQuery {
        conn_id,
        sql,
        row_limit: Some(limit),
    });
    if let Some(conn) = state.connections.get(&conn_id) {
        if matches!(conn.status, ConnectionStatus::Connected { .. }) {
            state.query_running = true;
        }
    }
    state.open_workspace_view(
        MainView::Data,
        format!("{schema}.{table_name}"),
        schema,
        table_name,
    );
}

fn draw_foreign_keys(
    painter: &egui::Painter,
    er_state: &ERDiagramState,
    canvas_rect: Rect,
    visible_ids: &HashSet<String>,
) {
    let visible_fk_count = er_state
        .foreign_keys
        .iter()
        .filter(|fk| {
            visible_ids.contains(&format!("{}.{}", fk.source_schema, fk.source_table))
                && visible_ids.contains(&format!("{}.{}", fk.target_schema, fk.target_table))
        })
        .count();
    let dense = visible_fk_count > 12 || visible_ids.len() > 8;
    let selected_id = er_state.selected_table.as_deref();

    for fk in &er_state.foreign_keys {
        let source_id = format!("{}.{}", fk.source_schema, fk.source_table);
        let target_id = format!("{}.{}", fk.target_schema, fk.target_table);
        if !visible_ids.contains(&source_id) || !visible_ids.contains(&target_id) {
            continue;
        }

        let source_card = match er_state.get_card(&fk.source_schema, &fk.source_table) {
            Some(c) => c,
            None => continue,
        };
        let target_card = match er_state.get_card(&fk.target_schema, &fk.target_table) {
            Some(c) => c,
            None => continue,
        };

        let source_col_idx =
            match er_state.get_column_index(&fk.source_schema, &fk.source_table, &fk.source_column)
            {
                Some(idx) => idx,
                None => continue,
            };
        let target_col_idx =
            match er_state.get_column_index(&fk.target_schema, &fk.target_table, &fk.target_column)
            {
                Some(idx) => idx,
                None => continue,
            };

        let source_y = source_card.column_y(source_col_idx);
        let target_y = target_card.column_y(target_col_idx);

        let from = world_to_screen(
            Pos2::new(source_card.pos.x + CARD_WIDTH, source_y),
            canvas_rect,
            er_state.pan_offset,
            er_state.zoom,
        );
        let to = world_to_screen(
            Pos2::new(target_card.pos.x, target_y),
            canvas_rect,
            er_state.pan_offset,
            er_state.zoom,
        );

        let selected_relation = selected_id.is_some_and(|id| id == source_id || id == target_id);
        let (color, stroke_width, label, font_size) = if selected_relation {
            (theme::accent_color(), 2.1, fk.name.as_str(), 9.5)
        } else if selected_id.is_some() {
            (theme::with_alpha(theme::accent_color(), 36), 0.8, "", 9.5)
        } else if dense {
            (theme::with_alpha(theme::accent_color(), 82), 0.95, fk.name.as_str(), 8.5)
        } else {
            (
                theme::with_alpha(theme::accent_color(), 170),
                1.35,
                fk.name.as_str(),
                9.5,
            )
        };

        draw_bezier_connection(painter, from, to, label, color, stroke_width, font_size);
    }
}

fn render_toolbar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
) {
    ui.horizontal(|ui| {
        let schemas = state
            .connections
            .get(&conn_id)
            .map(|conn| conn.schemas.clone())
            .unwrap_or_default();

        ui.label(
            RichText::new(t("visualizer_schema"))
                .color(theme::text_muted())
                .size(11.0)
                .strong(),
        );
        egui::ComboBox::from_id_salt("er_schema_select")
            .selected_text(&state.er_diagram.selected_schema)
            .width(170.0)
            .show_ui(ui, |ui| {
                for schema in &schemas {
                    if ui
                        .selectable_label(state.er_diagram.selected_schema == *schema, schema)
                        .clicked()
                    {
                        state.er_diagram.selected_schema = schema.clone();
                        if let Some(tab) = state
                            .workspace_tabs
                            .get_mut(state.active_workspace_tab)
                            .filter(|tab| tab.view == crate::state::MainView::Model)
                        {
                            tab.title = format!("Model: {schema}");
                            tab.schema_filter = schema.clone();
                        }
                    }
                }
            });

        ui.add_space(8.0);

        ui.add(
            theme::text_input(&mut state.er_diagram.search)
                .desired_width(220.0)
                .hint_text(t("visualizer_search_hint")),
        );

        ui.add_space(8.0);

        if ui
            .add(theme::secondary_button(&t("visualizer_reload")))
            .clicked()
        {
            reload_schema_visualizer(state, bridge, conn_id);
        }

        ui.add_space(8.0);

        if ui
            .add(theme::secondary_button(&t("visualizer_auto_layout")))
            .clicked()
        {
            state.er_diagram.layout_cards_with_width(ui.available_width());
            state.er_diagram.pan_offset = Vec2::ZERO;
        }

        if ui
            .add(theme::secondary_button(&t("visualizer_force_layout")))
            .clicked()
        {
            let canvas_size = ui.available_size();
            state.er_diagram.apply_force_directed_layout(canvas_size.x, canvas_size.y);
        }

        ui.add_space(8.0);

        // Handle scroll wheel zoom
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_speed = 0.003;
            state.er_diagram.zoom = (state.er_diagram.zoom + scroll_delta * zoom_speed).clamp(0.5, 1.8);
        }

        if ui.add(theme::ghost_button("-")).clicked() {
            state.er_diagram.zoom = (state.er_diagram.zoom * 0.9).clamp(0.5, 1.8);
        }
        ui.add(
            egui::Slider::new(&mut state.er_diagram.zoom, 0.5..=1.8)
                .show_value(false)
                .text(t("visualizer_zoom")),
        );
        if ui.add(theme::ghost_button("+")).clicked() {
            state.er_diagram.zoom = (state.er_diagram.zoom * 1.1).clamp(0.5, 1.8);
        }

        ui.add_space(8.0);

        if ui.add(theme::ghost_button(&t("visualizer_fit"))).clicked() {
            state.er_diagram.zoom = 0.9;
            state.er_diagram.pan_offset = Vec2::new(12.0, 12.0);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(tf(
                    "visualizer_count",
                    &[
                        &state.er_diagram.cards.len().to_string(),
                        &state.er_diagram.foreign_keys.len().to_string(),
                    ],
                ))
                .color(theme::text_muted())
                .size(11.0),
            );
        });
    });
}

fn world_to_screen(pos: Pos2, canvas_rect: Rect, pan_offset: Vec2, zoom: f32) -> Pos2 {
    canvas_rect.min + pan_offset + pos.to_vec2() * zoom
}

fn sync_schema_visualizer(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schema = state.er_diagram.selected_schema.clone();
    if schema.is_empty() {
        return;
    }

    let schema_changed = state.er_diagram.last_loaded_schema != schema;
    if schema_changed {
        state.er_diagram.cards.clear();
        state.er_diagram.foreign_keys.clear();
        state.er_diagram.selected_table = None;
        state.er_diagram.last_loaded_schema = schema.clone();
    }

    let mut should_request_tables = false;
    let mut should_request_foreign_keys = false;
    let mut missing_columns = Vec::new();
    let mut tables = Vec::new();

    if let Some(conn) = state.connections.get(&conn_id) {
        if let Some(schema_tables) = conn.tables.get(&schema) {
            tables = schema_tables.clone();
            for table in &tables {
                let key = (schema.clone(), table.name.clone());
                if !conn.columns.contains_key(&key) && !conn.loading_columns.contains(&key) {
                    missing_columns.push(table.name.clone());
                }
            }

            should_request_foreign_keys = !conn.foreign_keys.contains_key(&schema)
                && !conn.loading_foreign_keys.contains(&schema);
        } else if !conn.loading_tables.contains(&schema) {
            should_request_tables = true;
        }
    }

    if should_request_tables {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_tables.insert(schema.clone());
        }
        bridge.send(crate::db::bridge::DbCommand::ListTables {
            conn_id,
            schema: schema.clone(),
        });
    }

    for table in missing_columns {
        let key = (schema.clone(), table.clone());
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_columns.insert(key);
        }
        bridge.send(crate::db::bridge::DbCommand::ListColumns {
            conn_id,
            schema: schema.clone(),
            table,
        });
    }

    if should_request_foreign_keys {
        if let Some(conn) = state.connections.get_mut(&conn_id) {
            conn.loading_foreign_keys.insert(schema.clone());
        }
        bridge.send(crate::db::bridge::DbCommand::ListForeignKeys {
            conn_id,
            schema: schema.clone(),
        });
    }

    let Some(conn) = state.connections.get(&conn_id) else {
        return;
    };
    let mut relations_just_loaded = false;
    if let Some(fks) = conn.foreign_keys.get(&schema) {
        relations_just_loaded = state.er_diagram.foreign_keys.is_empty() && !fks.is_empty();
        state.er_diagram.foreign_keys = fks.clone();
    }

    let previous_keys: HashSet<String> = state.er_diagram.cards.keys().cloned().collect();
    let mut next_cards = HashMap::new();

    for table in tables {
        let full_id = format!("{}.{}", schema, table.name);
        let columns = conn
            .columns
            .get(&(schema.clone(), table.name.clone()))
            .cloned()
            .unwrap_or_default();

        let pos = state
            .er_diagram
            .cards
            .get(&full_id)
            .map(|card| card.pos)
            .unwrap_or(Pos2::new(CARD_START_X, CARD_START_Y));

        let card = TableCard {
            schema: schema.clone(),
            table_name: table.name.clone(),
            table_type: table.table_type.clone(),
            pos,
            columns,
            is_dragging: false,
        };
        state.er_diagram.cards.insert(card.full_id(), card);
        next_cards.insert(full_id, ());
    }

    state
        .er_diagram
        .cards
        .retain(|key, _| next_cards.contains_key(key));

    let next_keys: HashSet<String> = state.er_diagram.cards.keys().cloned().collect();
    if schema_changed || previous_keys != next_keys || relations_just_loaded {
        state.er_diagram.layout_cards();
        state.er_diagram.pan_offset = Vec2::ZERO;
    }
}

fn reload_schema_visualizer(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schema = state.er_diagram.selected_schema.clone();
    if schema.is_empty() {
        return;
    }

    if let Some(conn) = state.connections.get_mut(&conn_id) {
        conn.tables.remove(&schema);
        conn.foreign_keys.remove(&schema);
        conn.loading_tables.insert(schema.clone());
        conn.loading_foreign_keys.insert(schema.clone());

        let keys: Vec<(String, String)> = conn
            .columns
            .keys()
            .filter(|(column_schema, _)| column_schema == &schema)
            .cloned()
            .collect();
        for key in keys {
            conn.columns.remove(&key);
            conn.loading_columns.remove(&key);
        }
    }

    state.er_diagram.cards.clear();
    state.er_diagram.foreign_keys.clear();
    state.er_diagram.last_loaded_schema.clear();
    bridge.send(crate::db::bridge::DbCommand::ListTables {
        conn_id,
        schema: schema.clone(),
    });
    bridge.send(crate::db::bridge::DbCommand::ListForeignKeys { conn_id, schema });
}

pub fn handle_fk_response(state: &mut AppState, schema: &str, fks: &[ForeignKey]) {
    if state.er_diagram.selected_schema == schema {
        state.er_diagram.foreign_keys = fks.to_vec();
    }
}
