use eframe::egui::{self, Color32, CornerRadius, Id, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

use crate::db::bridge::DbBridge;
use crate::i18n::{t, tf};
use crate::state::{AppState, ConnectionStatus};
use crate::types::{ColumnInfo, ConnectionId};
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

#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub name: String,
    pub source_schema: String,
    pub source_table: String,
    pub source_column: String,
    pub target_schema: String,
    pub target_table: String,
    pub target_column: String,
}

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
}

impl ERDiagramState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }

    pub fn layout_cards(&mut self) {
        let mut ids: Vec<String> = self.cards.keys().cloned().collect();
        ids.sort_by(|a, b| {
            let degree_b = relation_degree(b, &self.foreign_keys);
            let degree_a = relation_degree(a, &self.foreign_keys);
            degree_b.cmp(&degree_a).then_with(|| a.cmp(b))
        });

        let len = ids.len().max(1);
        let rows = if len <= 4 { 1 } else { 2 };
        let mut row_heights = vec![0.0_f32; rows];

        for (idx, id) in ids.iter().enumerate() {
            let (row, _) = layout_slot(idx, rows);
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
            let (row, col) = layout_slot(idx, rows);
            if let Some(card) = self.cards.get_mut(id) {
                card.pos = Pos2::new(
                    CARD_START_X + col as f32 * (CARD_WIDTH + CARD_GAP_X),
                    row_offsets[row],
                );
            }
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

fn layout_slot(idx: usize, rows: usize) -> (usize, usize) {
    if rows <= 1 {
        (0, idx)
    } else {
        (idx % rows, idx / rows)
    }
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
            egui::FontId::monospace(9.0),
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

    let painter = ui.painter();
    let hovered = response.hovered();
    let border_color = if params.selected {
        theme::ACCENT_TEAL
    } else if hovered {
        theme::ACCENT_COPPER_LIGHT
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
                painter,
                Pos2::new(text_x, row_rect.center().y),
                "PK",
                theme::ACCENT_YELLOW,
                zoom,
            );
            text_x += 24.0 * zoom;
        }
        if is_fk {
            paint_pill(
                painter,
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
            format!("+{hidden} more columns"),
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
        "MATERIALIZED VIEW" => theme::ACCENT_TEAL,
        _ => theme::ACCENT_COPPER,
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

    let mut clicked_table = None;
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
            clicked_table = Some(card_id);
        }
    }

    if let Some(card_id) = clicked_table {
        state.er_diagram.selected_table = Some(card_id);
    } else if response.clicked() {
        state.er_diagram.selected_table = None;
    }

    if state.er_diagram.cards.is_empty() {
        paint_canvas_message(
            &painter,
            response.rect,
            &t("visualizer_loading_title"),
            &t("visualizer_loading_subtitle"),
        );
    } else if visible_ids.is_empty() {
        paint_canvas_message(
            &painter,
            response.rect,
            &t("visualizer_no_matching_tables"),
            &t("visualizer_clear_search_hint"),
        );
    }
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
        let (color, stroke_width, label) = if selected_relation {
            (theme::ACCENT_TEAL, 2.1, fk.name.as_str())
        } else if selected_id.is_some() {
            (theme::with_alpha(theme::ACCENT_TEAL, 36), 0.8, "")
        } else if dense {
            (theme::with_alpha(theme::ACCENT_TEAL, 82), 0.95, "")
        } else {
            (
                theme::with_alpha(theme::ACCENT_TEAL, 170),
                1.35,
                fk.name.as_str(),
            )
        };

        draw_bezier_connection(painter, from, to, label, color, stroke_width);
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
            state.er_diagram.layout_cards();
            state.er_diagram.pan_offset = Vec2::ZERO;
        }

        ui.add_space(8.0);

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
