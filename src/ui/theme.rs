use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Visuals,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Design token palette — DataGrip-style dark + macOS-style light
// ---------------------------------------------------------------------------

static DARK_MODE: AtomicBool = AtomicBool::new(true);

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

// u8 versions for CornerRadius::same
pub const RADIUS_SM: u8 = 2;
pub const RADIUS_MD: u8 = 4;
pub const RADIUS_LG: u8 = 6;

// f32 versions kept for any place that needs them
pub const INPUT_HEIGHT: f32 = 32.0;
pub const INPUT_MARGIN_X: i8 = 10;
pub const INPUT_MARGIN_Y: i8 = 6;
pub const INPUT_BG: Color32 = Color32::from_rgb(13, 16, 21);
pub const KEYWORD_COLOR: Color32 = ACCENT_BLUE;
pub const STRING_COLOR: Color32 = Color32::from_rgb(206, 145, 120);
pub const COMMENT_COLOR: Color32 = Color32::from_rgb(98, 140, 90);
pub const NUMBER_COLOR: Color32 = Color32::from_rgb(181, 206, 168);

const LIGHT_BG_SHELL: Color32 = Color32::from_rgb(241, 243, 247);
const LIGHT_BG_DARKEST: Color32 = Color32::from_rgb(247, 248, 251);
const LIGHT_BG_DARK: Color32 = Color32::from_rgb(250, 251, 253);
const LIGHT_BG_MEDIUM: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_BG_LIGHT: Color32 = Color32::from_rgb(235, 239, 245);
const LIGHT_BG_ELEVATED: Color32 = Color32::from_rgb(255, 255, 255);
const LIGHT_BG_EDITOR: Color32 = Color32::from_rgb(253, 254, 255);
const LIGHT_INPUT_BG: Color32 = Color32::from_rgb(255, 255, 255);

const LIGHT_TEXT_PRIMARY: Color32 = Color32::from_rgb(30, 34, 42);
const LIGHT_TEXT_SECONDARY: Color32 = Color32::from_rgb(82, 90, 106);
const LIGHT_TEXT_MUTED: Color32 = Color32::from_rgb(120, 129, 146);
const LIGHT_TEXT_DISABLED: Color32 = Color32::from_rgb(168, 176, 190);

const LIGHT_BORDER_SUBTLE: Color32 = Color32::from_rgb(226, 231, 238);
const LIGHT_BORDER_DEFAULT: Color32 = Color32::from_rgb(205, 212, 224);
const LIGHT_BORDER_STRONG: Color32 = Color32::from_rgb(176, 186, 202);
const LIGHT_BORDER_GLOW: Color32 = Color32::from_rgb(213, 161, 111);

pub fn is_dark() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

pub fn bg_shell() -> Color32 {
    pick(BG_SHELL, LIGHT_BG_SHELL)
}

pub fn bg_darkest() -> Color32 {
    pick(BG_DARKEST, LIGHT_BG_DARKEST)
}

pub fn bg_dark() -> Color32 {
    pick(BG_DARK, LIGHT_BG_DARK)
}

pub fn bg_medium() -> Color32 {
    pick(BG_MEDIUM, LIGHT_BG_MEDIUM)
}

pub fn bg_light() -> Color32 {
    pick(BG_LIGHT, LIGHT_BG_LIGHT)
}

pub fn bg_elevated() -> Color32 {
    pick(BG_ELEVATED, LIGHT_BG_ELEVATED)
}

pub fn bg_editor() -> Color32 {
    pick(BG_EDITOR, LIGHT_BG_EDITOR)
}

pub fn input_bg() -> Color32 {
    pick(INPUT_BG, LIGHT_INPUT_BG)
}

pub fn text_primary() -> Color32 {
    pick(TEXT_PRIMARY, LIGHT_TEXT_PRIMARY)
}

pub fn text_secondary() -> Color32 {
    pick(TEXT_SECONDARY, LIGHT_TEXT_SECONDARY)
}

pub fn text_muted() -> Color32 {
    pick(TEXT_MUTED, LIGHT_TEXT_MUTED)
}

pub fn text_disabled() -> Color32 {
    pick(TEXT_DISABLED, LIGHT_TEXT_DISABLED)
}

pub fn border_subtle() -> Color32 {
    pick(BORDER_SUBTLE, LIGHT_BORDER_SUBTLE)
}

pub fn border_default() -> Color32 {
    pick(BORDER_DEFAULT, LIGHT_BORDER_DEFAULT)
}

pub fn border_strong() -> Color32 {
    pick(BORDER_STRONG, LIGHT_BORDER_STRONG)
}

pub fn border_glow() -> Color32 {
    pick(BORDER_GLOW, LIGHT_BORDER_GLOW)
}

pub fn accent_copper_dim() -> Color32 {
    pick(ACCENT_COPPER_DIM, Color32::from_rgb(248, 228, 208))
}

fn pick(dark: Color32, light: Color32) -> Color32 {
    if is_dark() {
        dark
    } else {
        light
    }
}

// ---------------------------------------------------------------------------
// FerrumTheme
// ---------------------------------------------------------------------------

pub struct FerrumTheme;

impl FerrumTheme {
    pub fn apply_dark(ctx: &egui::Context) {
        DARK_MODE.store(true, Ordering::Relaxed);
        Self::configure_text_styles(ctx);
        ctx.set_visuals(Self::dark_visuals());
    }

    pub fn apply_light(ctx: &egui::Context) {
        DARK_MODE.store(false, Ordering::Relaxed);
        Self::configure_text_styles(ctx);
        ctx.set_visuals(Self::light_visuals());
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
        style.spacing.interact_size = egui::vec2(32.0, 30.0);
        style.spacing.menu_margin = Margin::same(SPACE_SM_I);
        style.spacing.window_margin = Margin::same(SPACE_MD_I);
        style.spacing.indent = 16.0;
        style.animation_time = 0.16;

        ctx.set_style(style);
    }

    fn dark_visuals() -> Visuals {
        let mut v = Visuals::dark();

        v.panel_fill = BG_DARK;
        v.window_fill = BG_MEDIUM;
        v.faint_bg_color = BG_LIGHT;
        v.extreme_bg_color = INPUT_BG;
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
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — inactive
        v.widgets.inactive.bg_fill = BG_LIGHT;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — hovered
        v.widgets.hovered.bg_fill = Color32::from_rgb(52, 61, 76);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_GLOW);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — active (pressed)
        v.widgets.active.bg_fill = ACCENT_COPPER_DIM;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — open
        v.widgets.open.bg_fill = BG_ELEVATED;
        v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.open.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_MD);

        v.selection.bg_fill = Color32::from_rgba_unmultiplied(204, 120, 50, 55);
        v.selection.stroke = Stroke::new(1.0, ACCENT_COPPER_LIGHT);

        v.override_text_color = Some(TEXT_PRIMARY);
        v.hyperlink_color = ACCENT_COPPER_LIGHT;

        v
    }

    fn light_visuals() -> Visuals {
        let mut v = Visuals::light();

        v.panel_fill = bg_dark();
        v.window_fill = bg_medium();
        v.faint_bg_color = bg_light();
        v.extreme_bg_color = input_bg();
        v.code_bg_color = bg_editor();

        v.window_stroke = Stroke::new(1.0, border_default());
        v.window_corner_radius = CornerRadius::same(RADIUS_MD);
        v.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(28),
        };

        v.widgets.noninteractive.bg_fill = bg_medium();
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_subtle());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary());
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.inactive.bg_fill = bg_medium();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, border_default());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary());
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.hovered.bg_fill = Color32::from_rgb(246, 248, 251);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, border_glow());
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, text_primary());
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.active.bg_fill = Color32::from_rgb(244, 224, 205);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.active.fg_stroke = Stroke::new(2.0, text_primary());
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.open.bg_fill = bg_elevated();
        v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.open.fg_stroke = Stroke::new(1.5, text_primary());
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_MD);

        v.selection.bg_fill = Color32::from_rgba_unmultiplied(204, 120, 50, 42);
        v.selection.stroke = Stroke::new(1.0, ACCENT_COPPER);

        v.override_text_color = Some(text_primary());
        v.hyperlink_color = ACCENT_COPPER;

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
    egui::Button::new(egui::RichText::new(text).color(text_primary()))
        .fill(bg_light())
        .stroke(Stroke::new(1.0, border_strong()))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn ghost_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(text_secondary()))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, border_default()))
        .corner_radius(CornerRadius::same(RADIUS_MD))
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn text_input(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .margin(Margin::symmetric(INPUT_MARGIN_X, INPUT_MARGIN_Y))
        .min_size(egui::vec2(0.0, INPUT_HEIGHT))
        .vertical_align(egui::Align::Center)
}

pub fn mono_text_input(text: &mut String) -> egui::TextEdit<'_> {
    text_input(text).font(TextStyle::Monospace)
}

pub fn password_input(text: &mut String, reveal: bool) -> egui::TextEdit<'_> {
    text_input(text).password(!reveal)
}
