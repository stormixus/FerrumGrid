use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// ThemeMode — user preference (Auto follows OS)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Auto,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Auto => "Auto",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    pub fn resolve(self, ctx: &egui::Context) -> egui::Theme {
        match self {
            ThemeMode::Light => egui::Theme::Light,
            ThemeMode::Dark => egui::Theme::Dark,
            ThemeMode::Auto => ctx.system_theme().unwrap_or(egui::Theme::Light),
        }
    }
}

// ============================================================================
// Tokens — full color palette per theme
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    // Surfaces
    pub bg_app: Color32,
    pub bg_surface: Color32,
    pub bg_sidebar: Color32,
    pub bg_elev: Color32,
    pub bg_overlay: Color32,

    // Borders
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,

    // Text
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,
    pub text_inverse: Color32,

    // Accent (copper)
    pub accent: Color32,
    pub accent_hot: Color32,
    pub accent_soft: Color32,

    // Semantic
    pub success: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub info: Color32,

    // Syntax
    pub syntax_keyword: Color32,
    pub syntax_string: Color32,
    pub syntax_number: Color32,
    pub syntax_comment: Color32,

    // Object monogram chips
    pub chip_table: Color32,
    pub chip_view: Color32,
    pub chip_mat_view: Color32,
    pub chip_function: Color32,
    pub chip_index: Color32,
    pub chip_query: Color32,

    // Misc
    pub pk: Color32,
    pub null: Color32,
    pub stripe: Color32,
    pub selection_bg: Color32,
    pub selection_stroke: Color32,
    pub active_line: Color32,
}

impl Tokens {
    pub const LIGHT: Tokens = Tokens {
        bg_app:     Color32::from_rgb(0xF5, 0xF4, 0xF2),
        bg_surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        bg_sidebar: Color32::from_rgb(0xED, 0xEA, 0xE5),
        bg_elev:    Color32::from_rgb(0xFA, 0xF8, 0xF5),
        bg_overlay: Color32::from_rgba_premultiplied(0, 0, 0, 30),

        border_subtle:  Color32::from_rgb(0xE5, 0xE2, 0xDD),
        border_default: Color32::from_rgb(0xD6, 0xD2, 0xCB),
        border_strong:  Color32::from_rgb(0xA8, 0xA2, 0x9A),

        text_primary:   Color32::from_rgb(0x1C, 0x19, 0x17),
        text_secondary: Color32::from_rgb(0x57, 0x53, 0x4E),
        text_muted:     Color32::from_rgb(0xA8, 0xA2, 0x9A),
        text_disabled:  Color32::from_rgb(0xC8, 0xC4, 0xBE),
        text_inverse:   Color32::from_rgb(0xFF, 0xFF, 0xFF),

        accent:      Color32::from_rgb(0xB4, 0x53, 0x09),
        accent_hot:  Color32::from_rgb(0x9A, 0x34, 0x12),
        accent_soft: Color32::from_rgba_premultiplied(0xB4, 0x53, 0x09, 30),

        success: Color32::from_rgb(0x04, 0x78, 0x57),
        danger:  Color32::from_rgb(0xB9, 0x1C, 0x1C),
        warn:    Color32::from_rgb(0xA1, 0x62, 0x07),
        info:    Color32::from_rgb(0x03, 0x69, 0xA1),

        syntax_keyword: Color32::from_rgb(0x03, 0x69, 0xA1),
        syntax_string:  Color32::from_rgb(0x9A, 0x34, 0x12),
        syntax_number:  Color32::from_rgb(0x04, 0x78, 0x57),
        syntax_comment: Color32::from_rgb(0xA8, 0xA2, 0x9A),

        chip_table:    Color32::from_rgb(0xB4, 0x53, 0x09),
        chip_view:     Color32::from_rgb(0x03, 0x69, 0xA1),
        chip_mat_view: Color32::from_rgb(0xA1, 0x62, 0x07),
        chip_function: Color32::from_rgb(0x04, 0x78, 0x57),
        chip_index:    Color32::from_rgb(0x57, 0x53, 0x4E),
        chip_query:    Color32::from_rgb(0x57, 0x53, 0x4E),

        pk:               Color32::from_rgb(0xA1, 0x62, 0x07),
        null:             Color32::from_rgb(0xA8, 0xA2, 0x9A),
        stripe:           Color32::from_rgba_premultiplied(0, 0, 0, 6),
        selection_bg:     Color32::from_rgba_premultiplied(0xB4, 0x53, 0x09, 30),
        selection_stroke: Color32::from_rgb(0xB4, 0x53, 0x09),
        active_line:      Color32::from_rgba_premultiplied(0xB4, 0x53, 0x09, 16),
    };

    pub const DARK: Tokens = Tokens {
        bg_app:     Color32::from_rgb(0x0F, 0x11, 0x15),
        bg_surface: Color32::from_rgb(0x18, 0x1B, 0x21),
        bg_sidebar: Color32::from_rgb(0x13, 0x16, 0x1B),
        bg_elev:    Color32::from_rgb(0x1F, 0x23, 0x2C),
        bg_overlay: Color32::from_rgba_premultiplied(0, 0, 0, 140),

        border_subtle:  Color32::from_rgb(0x23, 0x27, 0x2F),
        border_default: Color32::from_rgb(0x3A, 0x3F, 0x4A),
        border_strong:  Color32::from_rgb(0x5A, 0x61, 0x72),

        text_primary:   Color32::from_rgb(0xE5, 0xE7, 0xEB),
        text_secondary: Color32::from_rgb(0xA1, 0xA6, 0xB0),
        text_muted:     Color32::from_rgb(0x6B, 0x72, 0x80),
        text_disabled:  Color32::from_rgb(0x44, 0x48, 0x54),
        text_inverse:   Color32::from_rgb(0x0F, 0x11, 0x15),

        accent:      Color32::from_rgb(0xE8, 0x94, 0x56),
        accent_hot:  Color32::from_rgb(0xFB, 0xBF, 0x77),
        accent_soft: Color32::from_rgba_premultiplied(0xE8, 0x94, 0x56, 40),

        success: Color32::from_rgb(0x34, 0xD3, 0x99),
        danger:  Color32::from_rgb(0xF8, 0x71, 0x71),
        warn:    Color32::from_rgb(0xFB, 0xBF, 0x24),
        info:    Color32::from_rgb(0x60, 0xA5, 0xFA),

        syntax_keyword: Color32::from_rgb(0x60, 0xA5, 0xFA),
        syntax_string:  Color32::from_rgb(0xFB, 0xBF, 0x77),
        syntax_number:  Color32::from_rgb(0xB5, 0xCE, 0xA8),
        syntax_comment: Color32::from_rgb(0x6B, 0x72, 0x80),

        chip_table:    Color32::from_rgb(0xE8, 0x94, 0x56),
        chip_view:     Color32::from_rgb(0x60, 0xA5, 0xFA),
        chip_mat_view: Color32::from_rgb(0xFB, 0xBF, 0x24),
        chip_function: Color32::from_rgb(0x34, 0xD3, 0x99),
        chip_index:    Color32::from_rgb(0xA1, 0xA6, 0xB0),
        chip_query:    Color32::from_rgb(0xA1, 0xA6, 0xB0),

        pk:               Color32::from_rgb(0xFB, 0xBF, 0x24),
        null:             Color32::from_rgb(0x6B, 0x72, 0x80),
        stripe:           Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 8),
        selection_bg:     Color32::from_rgba_premultiplied(0xE8, 0x94, 0x56, 55),
        selection_stroke: Color32::from_rgb(0xE8, 0x94, 0x56),
        active_line:      Color32::from_rgba_premultiplied(0xE8, 0x94, 0x56, 18),
    };

    pub const fn for_theme(theme: egui::Theme) -> Self {
        match theme {
            egui::Theme::Light => Self::LIGHT,
            egui::Theme::Dark => Self::DARK,
        }
    }

    pub fn current(ctx: &egui::Context) -> Self {
        Self::for_theme(ctx.theme())
    }
}

// ============================================================================
// Spacing / radius scale
// ============================================================================

pub const SPACE_XS: f32 = 2.0;
pub const SPACE_SM: f32 = 4.0;
pub const SPACE_MD: f32 = 8.0;
pub const SPACE_LG: f32 = 12.0;
pub const SPACE_XL: f32 = 16.0;
pub const SPACE_XXL: f32 = 24.0;

pub const SPACE_XS_I: i8 = 2;
pub const SPACE_SM_I: i8 = 4;
pub const SPACE_MD_I: i8 = 8;
pub const SPACE_LG_I: i8 = 12;
pub const SPACE_XL_I: i8 = 16;
pub const SPACE_XXL_I: i8 = 24;

pub const RADIUS_SM: u8 = 3;
pub const RADIUS_MD: u8 = 5;
pub const RADIUS_LG: u8 = 8;

pub const RADIUS_SM_F: f32 = 3.0;
pub const RADIUS_MD_F: f32 = 5.0;
pub const RADIUS_LG_F: f32 = 8.0;

// ============================================================================
// FerrumTheme — apply visuals
// ============================================================================

pub struct FerrumTheme;

impl FerrumTheme {
    /// Register both Light and Dark visuals once at startup.
    pub fn init(ctx: &egui::Context) {
        Self::configure_text_styles(ctx);
        ctx.set_visuals_of(egui::Theme::Light, Self::visuals_for(Tokens::LIGHT, false));
        ctx.set_visuals_of(egui::Theme::Dark, Self::visuals_for(Tokens::DARK, true));
    }

    /// Resolve user preference and switch the active theme accordingly. Cheap; safe per-frame.
    pub fn apply_mode(ctx: &egui::Context, mode: ThemeMode) {
        let target = mode.resolve(ctx);
        if ctx.theme() != target {
            ctx.set_theme(target);
        }
    }

    fn configure_text_styles(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(15.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );

        style.spacing.item_spacing = egui::vec2(SPACE_MD, SPACE_SM);
        style.spacing.button_padding = egui::vec2(SPACE_LG, SPACE_SM);
        style.spacing.menu_margin = Margin::same(SPACE_SM_I);
        style.spacing.window_margin = Margin::same(SPACE_MD_I);
        style.spacing.indent = 16.0;
        style.spacing.scroll.bar_width = 10.0;

        ctx.set_style(style);
    }

    fn visuals_for(t: Tokens, dark: bool) -> Visuals {
        let mut v = if dark { Visuals::dark() } else { Visuals::light() };

        v.panel_fill = t.bg_app;
        v.window_fill = t.bg_surface;
        v.faint_bg_color = t.bg_elev;
        v.extreme_bg_color = if dark { t.bg_sidebar } else { t.bg_app };
        v.code_bg_color = t.bg_elev;

        v.window_stroke = Stroke::new(1.0, t.border_default);
        v.window_corner_radius = CornerRadius::same(RADIUS_LG);
        v.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: if dark {
                Color32::from_black_alpha(180)
            } else {
                Color32::from_black_alpha(40)
            },
        };

        // Noninteractive
        v.widgets.noninteractive.bg_fill = t.bg_surface;
        v.widgets.noninteractive.weak_bg_fill = t.bg_surface;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border_subtle);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_secondary);
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_SM);

        // Inactive (default state)
        v.widgets.inactive.bg_fill = t.bg_elev;
        v.widgets.inactive.weak_bg_fill = t.bg_elev;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border_default);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text_primary);
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_SM);

        // Hovered
        v.widgets.hovered.bg_fill = t.bg_elev;
        v.widgets.hovered.weak_bg_fill = t.bg_elev;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.accent);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, t.text_primary);
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_SM);

        // Active (pressed)
        v.widgets.active.bg_fill = t.accent;
        v.widgets.active.weak_bg_fill = t.accent;
        v.widgets.active.bg_stroke = Stroke::new(1.0, t.accent_hot);
        v.widgets.active.fg_stroke = Stroke::new(2.0, t.text_inverse);
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_SM);

        // Open (popup, dropdown)
        v.widgets.open.bg_fill = t.bg_elev;
        v.widgets.open.weak_bg_fill = t.bg_elev;
        v.widgets.open.bg_stroke = Stroke::new(1.0, t.accent);
        v.widgets.open.fg_stroke = Stroke::new(1.5, t.text_primary);
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_SM);

        v.selection.bg_fill = t.selection_bg;
        v.selection.stroke = Stroke::new(1.0, t.selection_stroke);

        v.override_text_color = Some(t.text_primary);
        v.hyperlink_color = t.accent;

        v
    }
}

/// Backward-compat entry point used by main.rs / tests.
pub fn configure_fonts(ctx: &egui::Context) {
    FerrumTheme::init(ctx);
}

// ============================================================================
// Button helpers (theme-aware)
// ============================================================================

pub fn primary_button<'a>(t: Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(t.text_inverse)
            .strong(),
    )
    .fill(t.accent)
    .stroke(Stroke::new(1.0, t.accent_hot))
    .corner_radius(CornerRadius::same(RADIUS_SM))
}

pub fn secondary_button<'a>(t: Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(egui::RichText::new(text).color(t.text_primary))
        .fill(t.bg_elev)
        .stroke(Stroke::new(1.0, t.border_default))
        .corner_radius(CornerRadius::same(RADIUS_SM))
}

pub fn ghost_button<'a>(t: Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(egui::RichText::new(text).color(t.text_secondary))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(RADIUS_SM))
}

pub fn danger_button<'a>(t: Tokens, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(t.danger)
    .stroke(Stroke::new(1.0, t.danger))
    .corner_radius(CornerRadius::same(RADIUS_SM))
}

pub fn conn_status_color(t: Tokens, connected: bool, connecting: bool) -> Color32 {
    if connecting {
        t.warn
    } else if connected {
        t.success
    } else {
        t.danger
    }
}

// ============================================================================
// Monogram chip — small colored pill with a single character
// ============================================================================

pub fn paint_monogram(
    ui: &mut egui::Ui,
    ch: &str,
    fg: Color32,
    bg: Color32,
    size: f32,
) -> egui::Rect {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(RADIUS_SM), bg);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        ch,
        FontId::new(size * 0.62, FontFamily::Proportional),
        fg,
    );
    rect
}

/// Translucent variant with an accent fill 14% opacity.
pub fn monogram_bg(color: Color32) -> Color32 {
    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 36)
}

// ============================================================================
// Icon-prefixed button widgets (vector icon + label, theme-aware)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Render a button with a leading vector icon and a label. Returns the response.
pub fn icon_button(
    ui: &mut egui::Ui,
    kind: BtnKind,
    icon: crate::ui::icons::Icon,
    label: &str,
    t: Tokens,
    enabled: bool,
) -> egui::Response {
    icon_button_sized(ui, kind, icon, label, t, enabled, 13.0, 28.0)
}

/// Compact (smaller) variant.
pub fn icon_button_sm(
    ui: &mut egui::Ui,
    kind: BtnKind,
    icon: crate::ui::icons::Icon,
    label: &str,
    t: Tokens,
    enabled: bool,
) -> egui::Response {
    icon_button_sized(ui, kind, icon, label, t, enabled, 11.0, 22.0)
}

fn icon_button_sized(
    ui: &mut egui::Ui,
    kind: BtnKind,
    icon: crate::ui::icons::Icon,
    label: &str,
    t: Tokens,
    enabled: bool,
    text_size: f32,
    height: f32,
) -> egui::Response {
    let pad_x = if height >= 28.0 { 12.0 } else { 8.0 };
    let icon_size = height - 12.0;
    let gap = 6.0;

    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(text_size),
        Color32::WHITE,
    );
    let total_w = pad_x + icon_size + gap + galley.rect.width() + pad_x;
    let (rect, resp_raw) =
        ui.allocate_exact_size(egui::vec2(total_w, height), egui::Sense::click());

    let resp = if enabled {
        resp_raw
    } else {
        // Mark as disabled — don't react to hover/click visually
        resp_raw
    };

    let (bg, fg, border) = button_colors(kind, t, enabled, resp.hovered(), resp.is_pointer_button_down_on());

    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS_SM), bg);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(RADIUS_SM),
        egui::Stroke::new(1.0, border),
        egui::epaint::StrokeKind::Inside,
    );

    let icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + pad_x, rect.center().y - icon_size / 2.0),
        egui::vec2(icon_size, icon_size),
    );
    crate::ui::icons::icon_at(ui.painter(), icon, icon_rect, fg);

    ui.painter().text(
        egui::pos2(rect.left() + pad_x + icon_size + gap, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(text_size),
        fg,
    );

    resp
}

/// Icon-only button with no label.
pub fn icon_only_button(
    ui: &mut egui::Ui,
    icon: crate::ui::icons::Icon,
    t: Tokens,
    color: Color32,
    size: f32,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(size + 8.0, size + 8.0), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(RADIUS_SM), t.bg_elev);
    }
    let icon_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(size, size));
    let fg = if resp.hovered() { t.text_primary } else { color };
    crate::ui::icons::icon_at(ui.painter(), icon, icon_rect, fg);
    resp
}

fn button_colors(
    kind: BtnKind,
    t: Tokens,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> (Color32, Color32, Color32) {
    if !enabled {
        return (t.bg_elev, t.text_disabled, t.border_subtle);
    }
    match kind {
        BtnKind::Primary => {
            if pressed {
                (t.accent_hot, t.text_inverse, t.accent_hot)
            } else if hovered {
                (t.accent_hot, t.text_inverse, t.accent)
            } else {
                (t.accent, t.text_inverse, t.accent_hot)
            }
        }
        BtnKind::Secondary => {
            if pressed || hovered {
                (t.bg_elev, t.text_primary, t.accent)
            } else {
                (t.bg_elev, t.text_primary, t.border_default)
            }
        }
        BtnKind::Ghost => {
            if pressed || hovered {
                (t.bg_elev, t.text_primary, t.border_subtle)
            } else {
                (Color32::TRANSPARENT, t.text_secondary, Color32::TRANSPARENT)
            }
        }
        BtnKind::Danger => {
            if pressed {
                (t.danger, Color32::WHITE, t.danger)
            } else if hovered {
                (t.danger, Color32::WHITE, t.danger)
            } else {
                (
                    Color32::from_rgba_premultiplied(t.danger.r(), t.danger.g(), t.danger.b(), 32),
                    t.danger,
                    t.danger,
                )
            }
        }
    }
}
