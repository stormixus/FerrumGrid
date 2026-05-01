//! Vector icons drawn directly via egui Painter.
//!
//! Each icon is a function that fills a Rect with a color. They scale to any
//! size, take a stroke width derived from the size, and adopt the caller's
//! color so light/dark themes work automatically. No font / image / SVG
//! dependencies.

use eframe::egui::{
    self, epaint, Align2, Color32, CornerRadius, FontId, Painter, Pos2, Rect, Response,
    RichText, Sense, Shape, Stroke, Ui, Vec2,
};
use std::f32::consts::{PI, TAU};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    // Object kinds (used in tree, palette, dialogs)
    Database,
    Schema,
    Table,
    View,
    MaterializedView,
    Function,
    Index,
    Key,
    Connection,

    // Actions
    Play,
    Stop,
    Plus,
    Close,
    Export,
    Copy,
    Refresh,
    Settings,

    // Search & command
    Search,
    Command,
    Filter,

    // Status
    ErrorMark,
    Check,
    Warning,
    NullMarker,

    // View toggles
    Form,
    Grid,

    // Security
    Lock,
    Unlock,

    // Misc
    ChevronDown,
    ChevronRight,
    Dot,
    Ellipsis,
    Clock,
    Logo,
}

// ============================================================================
// Public API
// ============================================================================

/// Draw an icon that fits inside `rect`, with the given foreground color and
/// stroke width.
pub fn draw(painter: &Painter, icon: Icon, rect: Rect, color: Color32, stroke_w: f32) {
    let stroke = Stroke::new(stroke_w, color);
    let r = rect.shrink(stroke_w * 0.5 + 0.5);
    match icon {
        Icon::Database => di_database(painter, r, stroke),
        Icon::Schema => di_schema(painter, r, stroke),
        Icon::Table => di_table(painter, r, stroke),
        Icon::View => di_view(painter, r, color, stroke),
        Icon::MaterializedView => di_mat_view(painter, r, color, stroke),
        Icon::Function => di_function(painter, r, stroke),
        Icon::Index => di_index(painter, r, stroke),
        Icon::Key => di_key(painter, r, stroke),
        Icon::Connection => di_connection(painter, r, stroke),

        Icon::Play => di_play(painter, r, color),
        Icon::Stop => di_stop(painter, r, color),
        Icon::Plus => di_plus(painter, r, stroke),
        Icon::Close => di_close(painter, r, stroke),
        Icon::Export => di_export(painter, r, stroke),
        Icon::Copy => di_copy(painter, r, stroke),
        Icon::Refresh => di_refresh(painter, r, color, stroke),
        Icon::Settings => di_settings(painter, r, stroke),

        Icon::Search => di_search(painter, r, stroke),
        Icon::Command => di_command(painter, r, stroke),
        Icon::Filter => di_filter(painter, r, stroke),

        Icon::ErrorMark => di_error(painter, r, stroke),
        Icon::Check => di_check(painter, r, stroke),
        Icon::Warning => di_warning(painter, r, color, stroke),
        Icon::NullMarker => di_null(painter, r, stroke),

        Icon::Form => di_form(painter, r, color, stroke),
        Icon::Grid => di_grid(painter, r, stroke),

        Icon::Lock => di_lock(painter, r, stroke),
        Icon::Unlock => di_unlock(painter, r, stroke),

        Icon::ChevronDown => di_chev_down(painter, r, stroke),
        Icon::ChevronRight => di_chev_right(painter, r, stroke),
        Icon::Dot => di_dot(painter, r, color),
        Icon::Ellipsis => di_ellipsis(painter, r, color),
        Icon::Clock => di_clock(painter, r, stroke),
        Icon::Logo => di_logo(painter, r, color),
    }
}

/// Allocate a square slot of `size` and draw an icon into it. Returns the response.
pub fn icon(ui: &mut Ui, kind: Icon, size: f32, color: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    draw(ui.painter(), kind, rect, color, default_stroke_w(size));
    resp
}

/// Render an icon at a precomputed Rect (caller owns layout).
pub fn icon_at(painter: &Painter, kind: Icon, rect: Rect, color: Color32) {
    let s = rect.width().min(rect.height());
    draw(painter, kind, rect, color, default_stroke_w(s));
}

/// Inline icon followed by a small gap and label text on a single line.
/// Useful for status pills and menu items.
pub fn label(
    ui: &mut Ui,
    kind: Icon,
    text: &str,
    color: Color32,
    icon_size: f32,
    text_size: f32,
) -> Response {
    let resp = ui
        .horizontal(|ui| {
            icon(ui, kind, icon_size, color);
            ui.add_space(4.0);
            ui.label(RichText::new(text).color(color).size(text_size))
        })
        .response;
    resp
}

fn default_stroke_w(size: f32) -> f32 {
    (size / 12.0).clamp(1.0, 1.8)
}

// ============================================================================
// Geometry helpers
// ============================================================================

fn arc_points(c: Pos2, rx: f32, ry: f32, start: f32, end: f32, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let a = start + t * (end - start);
            Pos2::new(c.x + rx * a.cos(), c.y + ry * a.sin())
        })
        .collect()
}

fn poly_filled(p: &Painter, points: Vec<Pos2>, color: Color32) {
    p.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

// ============================================================================
// Individual icons
// ============================================================================

fn di_database(p: &Painter, r: Rect, stroke: Stroke) {
    let cx = r.center().x;
    let rx = r.width() * 0.45;
    let ry = r.height() * 0.13;
    let top_y = r.top() + ry;
    let bot_y = r.bottom() - ry;

    p.add(Shape::line(
        arc_points(Pos2::new(cx, top_y), rx, ry, 0.0, TAU, 32),
        stroke,
    ));
    p.line_segment(
        [Pos2::new(cx - rx, top_y), Pos2::new(cx - rx, bot_y)],
        stroke,
    );
    p.line_segment(
        [Pos2::new(cx + rx, top_y), Pos2::new(cx + rx, bot_y)],
        stroke,
    );
    p.add(Shape::line(
        arc_points(Pos2::new(cx, bot_y), rx, ry, 0.0, PI, 16),
        stroke,
    ));
    let mid = top_y + (bot_y - top_y) * 0.45;
    p.add(Shape::line(
        arc_points(Pos2::new(cx, mid), rx, ry, 0.0, PI, 16),
        stroke,
    ));
}

fn di_schema(p: &Painter, r: Rect, stroke: Stroke) {
    let body_top = r.top() + r.height() * 0.22;
    let body = Rect::from_min_max(
        Pos2::new(r.left(), body_top),
        Pos2::new(r.right(), r.bottom()),
    );
    p.rect_stroke(body, CornerRadius::same(2), stroke, epaint::StrokeKind::Inside);
    let tab_w = r.width() * 0.45;
    let tab = Rect::from_min_max(
        Pos2::new(r.left(), r.top()),
        Pos2::new(r.left() + tab_w, body_top + 0.5),
    );
    p.rect_stroke(
        tab,
        CornerRadius {
            nw: 2,
            ne: 2,
            sw: 0,
            se: 0,
        },
        stroke,
        epaint::StrokeKind::Inside,
    );
}

fn di_table(p: &Painter, r: Rect, stroke: Stroke) {
    p.rect_stroke(
        r,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
    // Top header row (slightly thicker via accent)
    let header_y = r.top() + r.height() * 0.32;
    p.line_segment(
        [Pos2::new(r.left(), header_y), Pos2::new(r.right(), header_y)],
        stroke,
    );
    // Body row dividers
    let row_y = r.top() + r.height() * 0.66;
    p.line_segment(
        [Pos2::new(r.left(), row_y), Pos2::new(r.right(), row_y)],
        stroke,
    );
    // Vertical column divider
    let xv = r.left() + r.width() * 0.45;
    p.line_segment([Pos2::new(xv, header_y), Pos2::new(xv, r.bottom())], stroke);
}

fn di_view(p: &Painter, r: Rect, _color: Color32, stroke: Stroke) {
    // Eye-style: outer almond + inner pupil
    let cy = r.center().y;
    let half_w = r.width() * 0.5;
    let half_h = r.height() * 0.32;
    // Top arc (curving down)
    let center_top = Pos2::new(r.center().x, cy + half_h * 1.6);
    p.add(Shape::line(
        arc_points(
            center_top,
            half_w,
            half_h * 1.6,
            PI + 0.45,
            TAU - 0.45,
            18,
        ),
        stroke,
    ));
    // Bottom arc (curving up)
    let center_bot = Pos2::new(r.center().x, cy - half_h * 1.6);
    p.add(Shape::line(
        arc_points(center_bot, half_w, half_h * 1.6, 0.45, PI - 0.45, 18),
        stroke,
    ));
    // Pupil
    p.circle_stroke(Pos2::new(r.center().x, cy), r.height() * 0.13, stroke);
}

fn di_mat_view(p: &Painter, r: Rect, color: Color32, stroke: Stroke) {
    // Filled rounded rect (faint) + outline + 2 row dividers
    let bg = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 36);
    p.rect_filled(r, CornerRadius::same(2), bg);
    p.rect_stroke(
        r,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
    for i in 1..3 {
        let y = r.top() + r.height() * i as f32 / 3.0;
        p.line_segment(
            [
                Pos2::new(r.left() + 2.0, y),
                Pos2::new(r.right() - 2.0, y),
            ],
            stroke,
        );
    }
}

fn di_function(p: &Painter, r: Rect, stroke: Stroke) {
    // Curly braces { }
    let cy = r.center().y;
    let h = r.height();
    let w = r.width();
    let lx = r.left() + w * 0.18;
    let lx2 = lx + w * 0.13;
    p.add(Shape::line(
        vec![
            Pos2::new(lx2, r.top()),
            Pos2::new(lx, r.top() + h * 0.25),
            Pos2::new(lx, cy - h * 0.05),
            Pos2::new(lx - w * 0.07, cy),
            Pos2::new(lx, cy + h * 0.05),
            Pos2::new(lx, r.bottom() - h * 0.25),
            Pos2::new(lx2, r.bottom()),
        ],
        stroke,
    ));
    let rx = r.right() - w * 0.18;
    let rx2 = rx - w * 0.13;
    p.add(Shape::line(
        vec![
            Pos2::new(rx2, r.top()),
            Pos2::new(rx, r.top() + h * 0.25),
            Pos2::new(rx, cy - h * 0.05),
            Pos2::new(rx + w * 0.07, cy),
            Pos2::new(rx, cy + h * 0.05),
            Pos2::new(rx, r.bottom() - h * 0.25),
            Pos2::new(rx2, r.bottom()),
        ],
        stroke,
    ));
}

fn di_index(p: &Painter, r: Rect, stroke: Stroke) {
    // Three stacked horizontal bars, decreasing width, with a small dot at left
    let bars = 3;
    let step = r.height() / 4.0;
    for i in 0..bars {
        let y = r.top() + step + i as f32 * step;
        let pad = i as f32 * r.width() * 0.08;
        p.line_segment(
            [
                Pos2::new(r.left() + pad + 4.0, y),
                Pos2::new(r.right() - pad, y),
            ],
            stroke,
        );
        p.circle_filled(Pos2::new(r.left() + pad + 1.5, y), stroke.width, stroke.color);
    }
}

fn di_key(p: &Painter, r: Rect, stroke: Stroke) {
    let bow_r = r.height() * 0.28;
    let bow_c = Pos2::new(r.left() + bow_r + 2.0, r.center().y);
    p.circle_stroke(bow_c, bow_r, stroke);
    p.circle_filled(bow_c, bow_r * 0.35, stroke.color);
    // Shaft
    p.line_segment(
        [
            Pos2::new(bow_c.x + bow_r, r.center().y),
            Pos2::new(r.right(), r.center().y),
        ],
        stroke,
    );
    // Notches
    let n1 = Pos2::new(r.right() - r.width() * 0.05, r.center().y);
    let n2 = Pos2::new(r.right() - r.width() * 0.22, r.center().y);
    p.line_segment(
        [n1, Pos2::new(n1.x, n1.y + r.height() * 0.18)],
        stroke,
    );
    p.line_segment(
        [n2, Pos2::new(n2.x, n2.y + r.height() * 0.12)],
        stroke,
    );
}

fn di_connection(p: &Painter, r: Rect, stroke: Stroke) {
    // Two interlocked rounded oblongs (chain link)
    let h = r.height() * 0.42;
    let w = r.width() * 0.46;
    let cy = r.center().y;
    let l = Rect::from_center_size(
        Pos2::new(r.left() + w / 2.0 + 1.0, cy),
        Vec2::new(w, h),
    );
    let r2 = Rect::from_center_size(
        Pos2::new(r.right() - w / 2.0 - 1.0, cy),
        Vec2::new(w, h),
    );
    p.rect_stroke(
        l,
        CornerRadius::same((h * 0.5) as u8),
        stroke,
        epaint::StrokeKind::Inside,
    );
    p.rect_stroke(
        r2,
        CornerRadius::same((h * 0.5) as u8),
        stroke,
        epaint::StrokeKind::Inside,
    );
    // Connecting bar (slightly thicker)
    p.line_segment(
        [
            Pos2::new(l.right() - 2.0, cy),
            Pos2::new(r2.left() + 2.0, cy),
        ],
        Stroke::new(stroke.width * 1.4, stroke.color),
    );
}

fn di_play(p: &Painter, r: Rect, color: Color32) {
    let inset_x = r.width() * 0.18;
    let inset_y = r.height() * 0.06;
    let pts = vec![
        Pos2::new(r.left() + inset_x, r.top() + inset_y),
        Pos2::new(r.right() - inset_x * 0.4, r.center().y),
        Pos2::new(r.left() + inset_x, r.bottom() - inset_y),
    ];
    poly_filled(p, pts, color);
}

fn di_stop(p: &Painter, r: Rect, color: Color32) {
    let inner = r.shrink(r.width() * 0.18);
    p.rect_filled(inner, CornerRadius::same(1), color);
}

fn di_plus(p: &Painter, r: Rect, stroke: Stroke) {
    let m = r.center();
    let s = Stroke::new(stroke.width * 1.3, stroke.color);
    let h = r.height() * 0.35;
    let w = r.width() * 0.35;
    p.line_segment([Pos2::new(m.x - w, m.y), Pos2::new(m.x + w, m.y)], s);
    p.line_segment([Pos2::new(m.x, m.y - h), Pos2::new(m.x, m.y + h)], s);
}

fn di_close(p: &Painter, r: Rect, stroke: Stroke) {
    let m = r.shrink(r.width() * 0.22);
    let s = Stroke::new(stroke.width * 1.2, stroke.color);
    p.line_segment([m.left_top(), m.right_bottom()], s);
    p.line_segment([m.left_bottom(), m.right_top()], s);
}

fn di_export(p: &Painter, r: Rect, stroke: Stroke) {
    let cx = r.center().x;
    let tray = Rect::from_min_max(
        Pos2::new(r.left(), r.bottom() - r.height() * 0.32),
        Pos2::new(r.right(), r.bottom()),
    );
    p.rect_stroke(
        tray,
        CornerRadius::same(1),
        stroke,
        epaint::StrokeKind::Inside,
    );
    p.line_segment(
        [
            Pos2::new(cx, r.top() + r.height() * 0.05),
            Pos2::new(cx, r.bottom() - r.height() * 0.42),
        ],
        Stroke::new(stroke.width * 1.2, stroke.color),
    );
    let head_y = r.top() + r.height() * 0.05;
    let hw = r.width() * 0.18;
    let s = Stroke::new(stroke.width * 1.2, stroke.color);
    p.line_segment(
        [Pos2::new(cx, head_y), Pos2::new(cx - hw, head_y + hw)],
        s,
    );
    p.line_segment(
        [Pos2::new(cx, head_y), Pos2::new(cx + hw, head_y + hw)],
        s,
    );
}

fn di_copy(p: &Painter, r: Rect, stroke: Stroke) {
    let off = r.width() * 0.2;
    let r1 = Rect::from_min_max(
        Pos2::new(r.left(), r.top() + off),
        Pos2::new(r.right() - off, r.bottom()),
    );
    let r2 = Rect::from_min_max(
        Pos2::new(r.left() + off, r.top()),
        Pos2::new(r.right(), r.bottom() - off),
    );
    p.rect_stroke(
        r1,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
    p.rect_stroke(
        r2,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
}

fn di_refresh(p: &Painter, r: Rect, color: Color32, stroke: Stroke) {
    let center = r.center();
    let radius = r.width() * 0.4;
    p.add(Shape::line(
        arc_points(center, radius, radius, 0.4, TAU - 0.2, 32),
        stroke,
    ));
    let a: f32 = 0.4;
    let tip = Pos2::new(center.x + radius * a.cos(), center.y + radius * a.sin());
    let aw = r.width() * 0.16;
    let dir = Vec2::new(a.cos(), a.sin());
    let perp = Vec2::new(-dir.y, dir.x);
    let p1 = tip - dir * aw + perp * aw * 0.7;
    let p2 = tip - dir * aw - perp * aw * 0.7;
    poly_filled(p, vec![tip, p1, p2], color);
}

fn di_settings(p: &Painter, r: Rect, stroke: Stroke) {
    // Gear approximation: 8 small rectangles around a ring + center hole.
    let m = r.center();
    let outer_r = r.width() * 0.42;
    let inner_r = r.width() * 0.28;
    p.circle_stroke(m, outer_r * 0.85, stroke);
    p.circle_stroke(m, r.width() * 0.14, stroke);
    let teeth = 8;
    for i in 0..teeth {
        let a = (i as f32 + 0.5) / teeth as f32 * TAU;
        let p0 = Pos2::new(m.x + outer_r * 0.78 * a.cos(), m.y + outer_r * 0.78 * a.sin());
        let p1 = Pos2::new(m.x + outer_r * a.cos(), m.y + outer_r * a.sin());
        p.line_segment([p0, p1], Stroke::new(stroke.width * 1.4, stroke.color));
    }
    let _ = inner_r;
}

fn di_search(p: &Painter, r: Rect, stroke: Stroke) {
    let lens_r = r.width() * 0.32;
    let lens_c = Pos2::new(r.left() + lens_r + 1.5, r.top() + lens_r + 1.5);
    p.circle_stroke(lens_c, lens_r, Stroke::new(stroke.width * 1.1, stroke.color));
    let h0 = Pos2::new(
        lens_c.x + lens_r * 0.7071,
        lens_c.y + lens_r * 0.7071,
    );
    let h1 = Pos2::new(r.right() - 1.0, r.bottom() - 1.0);
    p.line_segment([h0, h1], Stroke::new(stroke.width * 1.4, stroke.color));
}

fn di_command(p: &Painter, r: Rect, stroke: Stroke) {
    let m = r.center();
    let half = r.width() * 0.18;
    let inner = Rect::from_center_size(m, Vec2::splat(half * 2.0));
    p.rect_stroke(
        inner,
        CornerRadius::same(1),
        stroke,
        epaint::StrokeKind::Inside,
    );
    let loop_r = r.width() * 0.11;
    let off = half + loop_r + 1.0;
    for c in [
        Pos2::new(m.x - off, m.y - off),
        Pos2::new(m.x + off, m.y - off),
        Pos2::new(m.x - off, m.y + off),
        Pos2::new(m.x + off, m.y + off),
    ] {
        p.circle_stroke(c, loop_r, stroke);
    }
}

fn di_filter(p: &Painter, r: Rect, stroke: Stroke) {
    let cx = r.center().x;
    let half_top = r.width() * 0.42;
    let half_mid = r.width() * 0.13;
    let top_y = r.top() + r.height() * 0.05;
    let mid_y = r.top() + r.height() * 0.55;
    let bot_y = r.bottom() - r.height() * 0.05;
    p.add(Shape::line(
        vec![
            Pos2::new(cx - half_top, top_y),
            Pos2::new(cx + half_top, top_y),
            Pos2::new(cx + half_mid, mid_y),
            Pos2::new(cx + half_mid, bot_y),
            Pos2::new(cx - half_mid, bot_y),
            Pos2::new(cx - half_mid, mid_y),
            Pos2::new(cx - half_top, top_y),
        ],
        stroke,
    ));
}

fn di_error(p: &Painter, r: Rect, stroke: Stroke) {
    p.circle_stroke(r.center(), r.width() * 0.45, stroke);
    let m = r.shrink(r.width() * 0.32);
    let s = Stroke::new(stroke.width * 1.2, stroke.color);
    p.line_segment([m.left_top(), m.right_bottom()], s);
    p.line_segment([m.left_bottom(), m.right_top()], s);
}

fn di_check(p: &Painter, r: Rect, stroke: Stroke) {
    let s = Stroke::new(stroke.width * 1.5, stroke.color);
    p.add(Shape::line(
        vec![
            Pos2::new(r.left() + r.width() * 0.15, r.center().y + r.height() * 0.05),
            Pos2::new(
                r.left() + r.width() * 0.42,
                r.bottom() - r.height() * 0.15,
            ),
            Pos2::new(r.right() - r.width() * 0.05, r.top() + r.height() * 0.18),
        ],
        s,
    ));
}

fn di_warning(p: &Painter, r: Rect, color: Color32, stroke: Stroke) {
    let m = r.center();
    let pts = vec![
        Pos2::new(m.x, r.top() + r.height() * 0.05),
        Pos2::new(r.right() - 1.0, r.bottom() - 1.0),
        Pos2::new(r.left() + 1.0, r.bottom() - 1.0),
        Pos2::new(m.x, r.top() + r.height() * 0.05),
    ];
    p.add(Shape::line(pts, stroke));
    p.line_segment(
        [
            Pos2::new(m.x, r.top() + r.height() * 0.34),
            Pos2::new(m.x, r.top() + r.height() * 0.66),
        ],
        Stroke::new(stroke.width * 1.3, color),
    );
    p.circle_filled(
        Pos2::new(m.x, r.bottom() - r.height() * 0.18),
        stroke.width * 1.0,
        color,
    );
}

fn di_null(p: &Painter, r: Rect, stroke: Stroke) {
    p.circle_stroke(r.center(), r.width() * 0.4, stroke);
    p.line_segment(
        [
            Pos2::new(r.left() + r.width() * 0.18, r.bottom() - r.height() * 0.18),
            Pos2::new(r.right() - r.width() * 0.18, r.top() + r.height() * 0.18),
        ],
        stroke,
    );
}

fn di_form(p: &Painter, r: Rect, color: Color32, stroke: Stroke) {
    for i in 0..3 {
        let y = r.top() + r.height() * (0.22 + 0.28 * i as f32);
        let mid_x = r.left() + r.width() * 0.34;
        p.line_segment(
            [Pos2::new(r.left(), y), Pos2::new(mid_x, y)],
            Stroke::new(stroke.width * 1.6, color),
        );
        p.line_segment(
            [Pos2::new(mid_x + 3.0, y), Pos2::new(r.right(), y)],
            stroke,
        );
    }
}

fn di_grid(p: &Painter, r: Rect, stroke: Stroke) {
    let m = r.center();
    let cells = [
        Rect::from_min_max(r.left_top(), m),
        Rect::from_min_max(Pos2::new(m.x, r.top()), Pos2::new(r.right(), m.y)),
        Rect::from_min_max(Pos2::new(r.left(), m.y), Pos2::new(m.x, r.bottom())),
        Rect::from_min_max(m, r.right_bottom()),
    ];
    for c in cells {
        p.rect_stroke(
            c.shrink(0.8),
            CornerRadius::same(1),
            stroke,
            epaint::StrokeKind::Inside,
        );
    }
}

fn di_lock(p: &Painter, r: Rect, stroke: Stroke) {
    let body = Rect::from_min_max(
        Pos2::new(r.left() + r.width() * 0.1, r.top() + r.height() * 0.45),
        Pos2::new(r.right() - r.width() * 0.1, r.bottom()),
    );
    p.rect_stroke(
        body,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
    let shackle_c = Pos2::new(r.center().x, r.top() + r.height() * 0.45);
    p.add(Shape::line(
        arc_points(
            shackle_c,
            r.width() * 0.27,
            r.height() * 0.32,
            PI,
            TAU,
            16,
        ),
        stroke,
    ));
    p.circle_filled(
        Pos2::new(r.center().x, r.top() + r.height() * 0.7),
        stroke.width * 1.3,
        stroke.color,
    );
}

fn di_unlock(p: &Painter, r: Rect, stroke: Stroke) {
    let body = Rect::from_min_max(
        Pos2::new(r.left() + r.width() * 0.1, r.top() + r.height() * 0.45),
        Pos2::new(r.right() - r.width() * 0.1, r.bottom()),
    );
    p.rect_stroke(
        body,
        CornerRadius::same(2),
        stroke,
        epaint::StrokeKind::Inside,
    );
    let shackle_c = Pos2::new(r.left() + r.width() * 0.36, r.top() + r.height() * 0.45);
    p.add(Shape::line(
        arc_points(
            shackle_c,
            r.width() * 0.27,
            r.height() * 0.32,
            PI,
            TAU + 0.3,
            16,
        ),
        stroke,
    ));
}

fn di_chev_down(p: &Painter, r: Rect, stroke: Stroke) {
    let s = Stroke::new(stroke.width * 1.4, stroke.color);
    p.add(Shape::line(
        vec![
            Pos2::new(r.left() + r.width() * 0.22, r.top() + r.height() * 0.36),
            Pos2::new(r.center().x, r.bottom() - r.height() * 0.28),
            Pos2::new(r.right() - r.width() * 0.22, r.top() + r.height() * 0.36),
        ],
        s,
    ));
}

fn di_chev_right(p: &Painter, r: Rect, stroke: Stroke) {
    let s = Stroke::new(stroke.width * 1.4, stroke.color);
    p.add(Shape::line(
        vec![
            Pos2::new(r.left() + r.width() * 0.36, r.top() + r.height() * 0.22),
            Pos2::new(r.right() - r.width() * 0.28, r.center().y),
            Pos2::new(r.left() + r.width() * 0.36, r.bottom() - r.height() * 0.22),
        ],
        s,
    ));
}

fn di_dot(p: &Painter, r: Rect, color: Color32) {
    p.circle_filled(r.center(), r.width() * 0.18, color);
}

fn di_ellipsis(p: &Painter, r: Rect, color: Color32) {
    let m = r.center();
    let dr = r.width() * 0.08;
    p.circle_filled(Pos2::new(m.x - r.width() * 0.3, m.y), dr, color);
    p.circle_filled(m, dr, color);
    p.circle_filled(Pos2::new(m.x + r.width() * 0.3, m.y), dr, color);
}

fn di_clock(p: &Painter, r: Rect, stroke: Stroke) {
    p.circle_stroke(r.center(), r.width() * 0.45, stroke);
    p.line_segment(
        [
            r.center(),
            Pos2::new(r.center().x, r.top() + r.height() * 0.18),
        ],
        Stroke::new(stroke.width * 1.2, stroke.color),
    );
    p.line_segment(
        [
            r.center(),
            Pos2::new(r.right() - r.width() * 0.22, r.center().y),
        ],
        stroke,
    );
}

fn di_logo(p: &Painter, r: Rect, color: Color32) {
    // "Forge" mark — solid copper square with a diagonal cleave (highlight)
    let s = r.shrink(0.5);
    p.rect_filled(s, CornerRadius::same(2), color);
    let highlight = Color32::from_rgba_premultiplied(255, 255, 255, 50);
    p.add(Shape::convex_polygon(
        vec![
            Pos2::new(s.right(), s.top()),
            Pos2::new(s.right(), s.center().y),
            Pos2::new(s.center().x, s.top()),
        ],
        highlight,
        Stroke::NONE,
    ));
    let shadow = Color32::from_rgba_premultiplied(0, 0, 0, 50);
    p.add(Shape::convex_polygon(
        vec![
            Pos2::new(s.left(), s.bottom()),
            Pos2::new(s.left(), s.center().y),
            Pos2::new(s.center().x, s.bottom()),
        ],
        shadow,
        Stroke::NONE,
    ));
}

// ============================================================================
// Object-kind monogram chips (kept from previous design — these are letters
// in colored chips, not to be confused with vector icons above).
// ============================================================================

pub const MONO_TABLE: &str = "T";
pub const MONO_VIEW: &str = "V";
pub const MONO_MAT_VIEW: &str = "M";
pub const MONO_FUNCTION: &str = "λ";
pub const MONO_INDEX: &str = "=";
pub const MONO_QUERY: &str = "Q";
pub const MONO_KEY: &str = "K";

pub fn paint_chip(
    painter: &Painter,
    rect: Rect,
    ch: &str,
    fg: Color32,
    bg: Color32,
) {
    painter.rect_filled(rect, CornerRadius::same(2), bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        ch,
        FontId::new(rect.height() * 0.62, egui::FontFamily::Proportional),
        fg,
    );
}

pub fn chip_bg(color: Color32) -> Color32 {
    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 36)
}
