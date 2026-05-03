use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Visuals,
};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Design token palette — DataGrip-style dark, copper/amber accent for Ferrum
// ---------------------------------------------------------------------------

// Background layers (darkest -> lightest)
pub const BG_SHELL: Color32 = Color32::from_rgb(8, 10, 13);
pub const BG_DARKEST: Color32 = Color32::from_rgb(12, 14, 18);
pub const BG_DARK: Color32 = Color32::from_rgb(18, 21, 27);
pub const BG_MEDIUM: Color32 = Color32::from_rgb(26, 31, 40);
pub const BG_LIGHT: Color32 = Color32::from_rgb(38, 45, 57);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(48, 58, 72);
pub const BG_EDITOR: Color32 = Color32::from_rgb(10, 12, 16);

// Text
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(220, 223, 228);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 165, 178);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 106, 122);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(68, 72, 84);

// Accent — copper/amber identity
pub const ACCENT_COPPER: Color32 = Color32::from_rgb(204, 120, 50);
pub const ACCENT_COPPER_LIGHT: Color32 = Color32::from_rgb(230, 152, 80);
pub const ACCENT_COPPER_DIM: Color32 = Color32::from_rgb(130, 76, 32);
pub const ACCENT_TEAL: Color32 = Color32::from_rgb(52, 190, 171);

// Semantic colors
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(86, 156, 214);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(78, 190, 100);
pub const ACCENT_RED: Color32 = Color32::from_rgb(210, 70, 70);
pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(220, 190, 80);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(204, 120, 50);

// Borders / separators
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(38, 41, 50);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(55, 59, 72);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(80, 86, 104);
pub const BORDER_GLOW: Color32 = Color32::from_rgb(96, 78, 61);

// ---------------------------------------------------------------------------
// Spacing scale — stored as f32 for use in add_space / vec2 calls.
// For Margin/CornerRadius (which need integer types) use the _I8/_U8 variants.
// ---------------------------------------------------------------------------
pub const SPACE_XS: f32 = 2.0;
pub const SPACE_SM: f32 = 4.0;
pub const SPACE_MD: f32 = 8.0;
pub const SPACE_LG: f32 = 12.0;
pub const SPACE_XL: f32 = 16.0;
pub const SPACE_XXL: f32 = 24.0;

// i8 versions for Margin::same / Margin::symmetric
pub const SPACE_XS_I: i8 = 2;
pub const SPACE_SM_I: i8 = 4;
pub const SPACE_MD_I: i8 = 8;
pub const SPACE_LG_I: i8 = 12;
pub const SPACE_XL_I: i8 = 16;
pub const SPACE_XXL_I: i8 = 24;

// u8 versions for CornerRadius::same
pub const RADIUS_SM: u8 = 2;
pub const RADIUS_MD: u8 = 4;
pub const RADIUS_LG: u8 = 6;

// f32 versions kept for any place that needs them
pub const RADIUS_SM_F: f32 = 2.0;
pub const RADIUS_MD_F: f32 = 4.0;
pub const RADIUS_LG_F: f32 = 6.0;

// ---------------------------------------------------------------------------
// Legacy aliases kept so editor.rs / grid.rs compile without changes
// ---------------------------------------------------------------------------
pub const NULL_COLOR: Color32 = TEXT_MUTED;
pub const PK_COLOR: Color32 = ACCENT_YELLOW;
pub const ERROR_COLOR: Color32 = ACCENT_RED;
pub const SUCCESS_COLOR: Color32 = ACCENT_GREEN;
pub const KEYWORD_COLOR: Color32 = ACCENT_BLUE;
pub const STRING_COLOR: Color32 = Color32::from_rgb(206, 145, 120);
pub const COMMENT_COLOR: Color32 = Color32::from_rgb(98, 140, 90);
pub const NUMBER_COLOR: Color32 = Color32::from_rgb(181, 206, 168);
pub const TABLE_STRIPE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 5);

// ---------------------------------------------------------------------------
// FerrumTheme
// ---------------------------------------------------------------------------

pub struct FerrumTheme;

impl FerrumTheme {
    pub fn apply_dark(ctx: &egui::Context) {
        Self::configure_text_styles(ctx);
        ctx.set_visuals(Self::dark_visuals());
    }

    pub fn apply_light(ctx: &egui::Context) {
        Self::configure_text_styles(ctx);
        ctx.set_visuals(egui::Visuals::light());
    }

    fn configure_text_styles(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(11.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(15.0, FontFamily::Proportional),
        );

        style.spacing.item_spacing = egui::vec2(SPACE_MD, SPACE_SM);
        style.spacing.button_padding = egui::vec2(SPACE_LG, SPACE_SM);
        style.spacing.interact_size = egui::vec2(30.0, 26.0);
        style.spacing.menu_margin = Margin::same(SPACE_SM_I);
        style.spacing.window_margin = Margin::same(SPACE_MD_I);
        style.spacing.indent = 16.0;

        ctx.set_style(style);
    }

    fn dark_visuals() -> Visuals {
        let mut v = Visuals::dark();

        v.panel_fill = BG_DARK;
        v.window_fill = BG_MEDIUM;
        v.faint_bg_color = BG_LIGHT;
        v.extreme_bg_color = BG_SHELL;
        v.code_bg_color = BG_EDITOR;

        v.window_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        v.window_corner_radius = CornerRadius::same(RADIUS_MD);
        v.window_shadow = egui::Shadow {
            offset: [0, 4],
            blur: 20,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };

        // Widgets — noninteractive
        v.widgets.noninteractive.bg_fill = BG_MEDIUM;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_SM);

        // Widgets — inactive
        v.widgets.inactive.bg_fill = BG_LIGHT;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_SM);

        // Widgets — hovered
        v.widgets.hovered.bg_fill = Color32::from_rgb(52, 61, 76);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_GLOW);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_SM);

        // Widgets — active (pressed)
        v.widgets.active.bg_fill = ACCENT_COPPER_DIM;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_SM);

        // Widgets — open
        v.widgets.open.bg_fill = BG_ELEVATED;
        v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.open.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_SM);

        v.selection.bg_fill = Color32::from_rgba_premultiplied(204, 120, 50, 55);
        v.selection.stroke = Stroke::new(1.0, ACCENT_COPPER);

        v.override_text_color = Some(TEXT_PRIMARY);
        v.hyperlink_color = ACCENT_COPPER_LIGHT;

        v
    }
}

// ---------------------------------------------------------------------------
// Backward-compat entry point
// ---------------------------------------------------------------------------
pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    install_apple_system_fonts(&mut fonts);
    install_cjk_font_fallbacks(&mut fonts);
    ctx.set_fonts(fonts);

    FerrumTheme::apply_dark(ctx);
}

pub fn apply_appearance(ctx: &egui::Context, appearance: &str) -> bool {
    let use_dark = match appearance {
        "light" => false,
        "dark" => true,
        _ => !matches!(ctx.system_theme(), Some(egui::Theme::Light)),
    };

    if use_dark {
        FerrumTheme::apply_dark(ctx);
    } else {
        FerrumTheme::apply_light(ctx);
    }

    use_dark
}

fn install_apple_system_fonts(fonts: &mut FontDefinitions) {
    install_font(
        fonts,
        "ferrum_sf_pro",
        "/System/Library/Fonts/SFNS.ttf",
        &[FontFamily::Proportional],
        FontPlacement::Front,
    );
    install_font(
        fonts,
        "ferrum_sf_mono",
        "/System/Library/Fonts/SFNSMono.ttf",
        &[FontFamily::Monospace],
        FontPlacement::Front,
    );
}

fn install_cjk_font_fallbacks(fonts: &mut FontDefinitions) {
    let candidates = [
        (
            "ferrum_apple_gothic",
            "/System/Library/Fonts/Supplemental/AppleGothic.ttf",
        ),
        (
            "ferrum_arial_unicode",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ),
    ];

    for (name, path) in candidates {
        install_font(
            fonts,
            name,
            path,
            &[FontFamily::Proportional, FontFamily::Monospace],
            FontPlacement::Back,
        );
    }
}

enum FontPlacement {
    Front,
    Back,
}

fn install_font(
    fonts: &mut FontDefinitions,
    name: &str,
    path: &str,
    families: &[FontFamily],
    placement: FontPlacement,
) {
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };

    fonts
        .font_data
        .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));

    for family in families {
        let family_fonts = fonts.families.entry(family.clone()).or_default();
        family_fonts.retain(|font_name| font_name != name);
        match placement {
            FontPlacement::Front => family_fonts.insert(0, name.to_owned()),
            FontPlacement::Back => family_fonts.push(name.to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Button helpers
// ---------------------------------------------------------------------------

pub fn primary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
        .fill(ACCENT_COPPER)
        .stroke(Stroke::new(1.0, ACCENT_COPPER_LIGHT))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn secondary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_PRIMARY))
        .fill(BG_LIGHT)
        .stroke(Stroke::new(1.0, BORDER_STRONG))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn danger_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
        .fill(Color32::from_rgb(140, 40, 40))
        .stroke(Stroke::new(1.0, ACCENT_RED))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn ghost_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, BORDER_DEFAULT))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn type_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(FontId::monospace(11.0))
            .color(TEXT_MUTED),
    );
}

pub fn conn_status_color(connected: bool, connecting: bool) -> Color32 {
    if connecting {
        ACCENT_YELLOW
    } else if connected {
        ACCENT_GREEN
    } else {
        ACCENT_RED
    }
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha)
}
