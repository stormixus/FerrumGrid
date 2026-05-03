use eframe::egui::{
    self, Color32, CornerRadius, Id, Margin, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};
use std::collections::HashMap;

use crate::db::bridge::DbBridge;
use crate::state::{AppState, ConnectionStatus};
use crate::types::{ColumnInfo, ConnectionId};
use crate::ui::theme;

const CARD_WIDTH: f32 = 200.0;
const CARD_HEADER_HEIGHT: f32 = 32.0;
const CARD_ROW_HEIGHT: f32 = 22.0;
const PK_ICON_WIDTH: f32 = 20.0;
const FK_ICON_WIDTH: f32 = 20.0;

#[derive(Debug, Clone)]
pub struct TableCard {
    pub schema: String,
    pub table_name: String,
    pub pos: Pos2,
    pub columns: Vec<ColumnInfo>,
    pub is_dragging: bool,
}

impl TableCard {
    pub fn height(&self) -> f32 {
        CARD_HEADER_HEIGHT + self.columns.len() as f32 * CARD_ROW_HEIGHT + 8.0
    }

    pub fn full_id(&self) -> String {
        format!("{}.{}", self.schema, self.table_name)
    }

    pub fn column_y(&self, col_idx: usize) -> f32 {
        self.pos.y + CARD_HEADER_HEIGHT + col_idx as f32 * CARD_ROW_HEIGHT + CARD_ROW_HEIGHT / 2.0
    }

    pub fn pk_column_idx(&self) -> Option<usize> {
        self.columns.iter().position(|c| c.is_primary_key)
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
        let mut x = 50.0;
        let mut y = 50.0;
        let max_height = 600.0;

        for card in self.cards.values_mut() {
            card.pos = Pos2::new(x, y);
            y += card.height() + 40.0;
            if y > max_height {
                y = 50.0;
                x += CARD_WIDTH + 60.0;
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
        painter.text(
            mid + Vec2::new(0.0, -8.0),
            egui::Align2::CENTER_BOTTOM,
            label,
            egui::FontId::monospace(9.0),
            theme::TEXT_MUTED,
        );
    }
}

fn draw_card(
    _ctx: &egui::Context,
    ui: &mut egui::Ui,
    canvas_rect: Rect,
    pan_offset: Vec2,
    zoom: f32,
    card: &mut TableCard,
    drag_interaction: bool,
) -> Option<Vec2> {
    let card_id = Id::new("er_card_").with(&card.full_id());
    let screen_pos = world_to_screen(card.pos, canvas_rect, pan_offset, zoom);
    let screen_rect = Rect::from_min_size(
        screen_pos,
        Vec2::new(CARD_WIDTH * zoom, card.height() * zoom),
    );
    let response = ui.interact(screen_rect, card_id, Sense::click_and_drag());

    if drag_interaction {
        if response.drag_started() {
            card.is_dragging = true;
        }
        if response.dragged() {
            return Some(response.drag_delta() / zoom);
        }
        if response.drag_stopped() {
            card.is_dragging = false;
        }
    }

    let header_rect = Rect::from_min_size(
        screen_pos,
        Vec2::new(CARD_WIDTH * zoom, CARD_HEADER_HEIGHT * zoom),
    );
    let body_rect = Rect::from_min_size(
        Pos2::new(screen_pos.x, screen_pos.y + CARD_HEADER_HEIGHT * zoom),
        Vec2::new(
            CARD_WIDTH * zoom,
            (card.height() - CARD_HEADER_HEIGHT) * zoom,
        ),
    );

    let header_color = theme::BG_LIGHT;
    let body_color = theme::BG_DARK;
    let border_color = theme::BORDER_DEFAULT;

    let header_frame = egui::Frame::new()
        .fill(header_color)
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .stroke(Stroke::new(1.0, border_color));

    header_frame.show(ui, |ui| {
        ui.set_clip_rect(header_rect);
        let text = RichText::new(&card.table_name)
            .color(theme::ACCENT_COPPER)
            .strong()
            .size((12.0 * zoom).clamp(9.5, 15.0));
        ui.put(
            header_rect.shrink2(Vec2::new(8.0, 6.0)),
            egui::Label::new(text).truncate(),
        );
        if card.pk_column_idx().is_some() {
            ui.painter().text(
                header_rect.right_center() - Vec2::new(8.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                "PK",
                egui::FontId::monospace((8.0 * zoom).clamp(7.0, 10.0)),
                theme::ACCENT_YELLOW,
            );
        }
    });

    let body_frame = egui::Frame::new()
        .fill(body_color)
        .corner_radius(CornerRadius::same(theme::RADIUS_SM))
        .stroke(Stroke::new(1.0, border_color))
        .inner_margin(Margin::same(4));

    body_frame.show(ui, |ui| {
        ui.set_clip_rect(body_rect);
        for (idx, col) in card.columns.iter().enumerate() {
            let y = screen_pos.y
                + CARD_HEADER_HEIGHT * zoom
                + idx as f32 * CARD_ROW_HEIGHT * zoom
                + 2.0 * zoom;
            let row_rect = Rect::from_min_size(
                Pos2::new(screen_pos.x + 4.0 * zoom, y),
                Vec2::new(CARD_WIDTH * zoom - 8.0 * zoom, CARD_ROW_HEIGHT * zoom),
            );

            let mut icon_text = String::new();
            let mut icon_color = theme::TEXT_MUTED;

            if col.is_primary_key {
                icon_text = "PK".to_string();
                icon_color = theme::ACCENT_YELLOW;
            }

            let is_fk = card.columns.iter().any(|c| {
                c.name != col.name
                    && c.name.ends_with("_id")
                    && col.name == c.name.replace("_id", "")
            });

            if is_fk && icon_text.is_empty() {
                icon_text = "FK".to_string();
                icon_color = theme::ACCENT_BLUE;
            }

            ui.painter().text(
                Pos2::new(row_rect.left() + 4.0, row_rect.center().y),
                egui::Align2::LEFT_CENTER,
                &icon_text,
                egui::FontId::monospace((8.0 * zoom).clamp(7.0, 10.0)),
                icon_color,
            );

            ui.painter().text(
                Pos2::new(
                    row_rect.left() + PK_ICON_WIDTH.max(FK_ICON_WIDTH) * zoom + 4.0 * zoom,
                    row_rect.center().y,
                ),
                egui::Align2::LEFT_CENTER,
                &col.name,
                egui::FontId::proportional((11.0 * zoom).clamp(8.5, 13.0)),
                theme::TEXT_PRIMARY,
            );

            ui.painter().text(
                Pos2::new(row_rect.right() - 4.0 * zoom, row_rect.center().y),
                egui::Align2::RIGHT_CENTER,
                &col.data_type,
                egui::FontId::proportional((10.0 * zoom).clamp(8.0, 12.0)),
                theme::TEXT_MUTED,
            );
        }
    });

    ui.painter().rect_stroke(
        screen_rect.expand(1.0),
        CornerRadius::same(theme::RADIUS_SM),
        Stroke::new(2.0, border_color),
        egui::StrokeKind::Inside,
    );

    None
}

pub fn render_er_diagram(ctx: &egui::Context, state: &mut AppState, bridge: &DbBridge) {
    if !state.er_diagram.show_diagram {
        return;
    }

    let active_conn = match state.active_connection {
        Some(id) => id,
        None => return,
    };

    let conn = match state.connections.get(&active_conn) {
        Some(c) if matches!(c.status, ConnectionStatus::Connected { .. }) => c,
        _ => return,
    };

    let schema = state.er_diagram.selected_schema.clone();
    if schema.is_empty() && !conn.schemas.is_empty() {
        state.er_diagram.selected_schema = conn.schemas[0].clone();
    }

    egui::Window::new("ER Diagram")
        .default_size([800.0, 600.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            render_toolbar(ui, state, bridge, active_conn);

            ui.separator();

            let available = ui.available_size();
            let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());

            let bg_color = theme::BG_DARKEST;
            painter.rect_filled(response.rect, CornerRadius::same(0), bg_color);

            let mouse_pos = ctx.input(|i| i.pointer.hover_pos());

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

            let mut any_dragging = false;
            let mut drag_delta = Vec2::ZERO;

            for card in state.er_diagram.cards.values_mut() {
                if card.is_dragging {
                    any_dragging = true;
                }
            }

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

            draw_foreign_keys(&painter, &state.er_diagram, response.rect);

            for card in state.er_diagram.cards.values_mut() {
                if let Some(delta) = draw_card(
                    ctx,
                    ui,
                    response.rect,
                    state.er_diagram.pan_offset,
                    state.er_diagram.zoom,
                    card,
                    true,
                ) {
                    card.pos += delta;
                }
            }
        });
}

fn draw_foreign_keys(painter: &egui::Painter, er_state: &ERDiagramState, canvas_rect: Rect) {
    for fk in &er_state.foreign_keys {
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

        draw_bezier_connection(painter, from, to, &fk.name, theme::ACCENT_BLUE, 1.5);
    }
}

fn render_toolbar(
    ui: &mut egui::Ui,
    state: &mut AppState,
    bridge: &DbBridge,
    conn_id: ConnectionId,
) {
    ui.horizontal(|ui| {
        let conn = state.connections.get(&conn_id).unwrap();

        egui::ComboBox::from_id_salt("er_schema_select")
            .selected_text(&state.er_diagram.selected_schema)
            .width(150.0)
            .show_ui(ui, |ui| {
                for schema in &conn.schemas {
                    if ui
                        .selectable_label(state.er_diagram.selected_schema == *schema, schema)
                        .clicked()
                    {
                        state.er_diagram.selected_schema = schema.clone();
                    }
                }
            });

        ui.add_space(8.0);

        if ui.button("Load Schema").clicked() {
            load_schema_tables(state, bridge, conn_id);
        }

        ui.add_space(8.0);

        if ui.button("Auto Layout").clicked() {
            state.er_diagram.layout_cards();
            state.er_diagram.pan_offset = Vec2::ZERO;
        }

        ui.add_space(8.0);

        if ui.button("-").clicked() {
            state.er_diagram.zoom = (state.er_diagram.zoom * 0.9).clamp(0.5, 1.8);
        }
        ui.add(
            egui::Slider::new(&mut state.er_diagram.zoom, 0.5..=1.8)
                .show_value(false)
                .text("Zoom"),
        );
        if ui.button("+").clicked() {
            state.er_diagram.zoom = (state.er_diagram.zoom * 1.1).clamp(0.5, 1.8);
        }

        ui.add_space(8.0);

        if ui.button("Clear").clicked() {
            state.er_diagram.cards.clear();
            state.er_diagram.foreign_keys.clear();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!(
                    "{} tables, {} relations",
                    state.er_diagram.cards.len(),
                    state.er_diagram.foreign_keys.len()
                ))
                .color(theme::TEXT_MUTED)
                .size(11.0),
            );
        });
    });
}

fn world_to_screen(pos: Pos2, canvas_rect: Rect, pan_offset: Vec2, zoom: f32) -> Pos2 {
    canvas_rect.min + pan_offset + pos.to_vec2() * zoom
}

fn load_schema_tables(state: &mut AppState, bridge: &DbBridge, conn_id: ConnectionId) {
    let schema = state.er_diagram.selected_schema.clone();
    if schema.is_empty() {
        return;
    }

    let conn = state.connections.get(&conn_id).unwrap();
    let tables = conn.tables.get(&schema).cloned().unwrap_or_default();

    state.er_diagram.cards.clear();
    state.er_diagram.foreign_keys.clear();

    for (idx, table) in tables.iter().enumerate() {
        let columns = conn
            .columns
            .get(&(schema.clone(), table.name.clone()))
            .cloned()
            .unwrap_or_default();

        let card = TableCard {
            schema: schema.clone(),
            table_name: table.name.clone(),
            pos: Pos2::new(
                50.0 + (idx % 4) as f32 * 260.0,
                50.0 + (idx / 4) as f32 * 200.0,
            ),
            columns,
            is_dragging: false,
        };
        state.er_diagram.cards.insert(card.full_id(), card);
    }

    bridge.send(crate::db::bridge::DbCommand::ListForeignKeys {
        conn_id,
        schema: schema.clone(),
    });
}

pub fn handle_fk_response(state: &mut AppState, schema: &str, fks: &[ForeignKey]) {
    if state.er_diagram.selected_schema == schema {
        state.er_diagram.foreign_keys = fks.to_vec();
    }
}
