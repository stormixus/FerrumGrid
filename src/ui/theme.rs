use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke,
    TextStyle, Visuals,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Design token palette — Supabase-inspired dark developer console
// ---------------------------------------------------------------------------

static DARK_MODE: AtomicBool = AtomicBool::new(true);

// Background layers — DESIGN.md hierarchy:
// Canvas (#171717) = main surface for all panels (unified look)
// Deep (#0f0f0f) = inset areas: code editor, inputs
// Raised (#1f1f1f ~ #292929) = toolbars, active tabs, cards
pub const BG_SHELL: Color32 = Color32::from_rgb(23, 23, 23);
pub const BG_DARKEST: Color32 = Color32::from_rgb(15, 15, 15);
pub const BG_DARK: Color32 = Color32::from_rgb(23, 23, 23);
pub const BG_MEDIUM: Color32 = Color32::from_rgb(31, 31, 31);
pub const BG_LIGHT: Color32 = Color32::from_rgb(41, 41, 41);
pub const BG_ELEVATED: Color32 = Color32::from_rgb(46, 46, 46);
pub const BG_EDITOR: Color32 = Color32::from_rgb(15, 15, 15);

// Text
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(250, 250, 250);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(137, 137, 137);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(77, 77, 77);

// Accent — Supabase emerald identity. Copper names remain as compatibility aliases.
pub const ACCENT_EMERALD: Color32 = Color32::from_rgb(62, 207, 142);
pub const ACCENT_EMERALD_LIGHT: Color32 = Color32::from_rgb(92, 230, 167);
pub const ACCENT_EMERALD_DIM: Color32 = Color32::from_rgb(22, 78, 55);
pub const ACCENT_COPPER: Color32 = ACCENT_EMERALD;
pub const ACCENT_COPPER_LIGHT: Color32 = ACCENT_EMERALD_LIGHT;
pub const ACCENT_COPPER_DIM: Color32 = ACCENT_EMERALD_DIM;
pub const ACCENT_TEAL: Color32 = Color32::from_rgb(0, 197, 115);

// Semantic colors
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(118, 156, 255);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(62, 207, 142);
pub const ACCENT_RED: Color32 = Color32::from_rgb(229, 72, 77);
pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(245, 204, 93);

// Borders / separators
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(36, 36, 36);
pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(46, 46, 46);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(54, 54, 54);
pub const BORDER_GLOW: Color32 = ACCENT_EMERALD;

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
pub const INPUT_HEIGHT: f32 = 28.0;
pub const INPUT_MARGIN_X: i8 = 8;
pub const INPUT_MARGIN_Y: i8 = 4;
pub const INPUT_BG: Color32 = Color32::from_rgb(15, 15, 15);
pub const BUTTON_HEIGHT: f32 = 28.0;
pub const KEYWORD_COLOR: Color32 = ACCENT_BLUE;
pub const STRING_COLOR: Color32 = Color32::from_rgb(62, 207, 142);
pub const COMMENT_COLOR: Color32 = Color32::from_rgb(137, 137, 137);
pub const NUMBER_COLOR: Color32 = Color32::from_rgb(245, 204, 93);

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
const LIGHT_BORDER_GLOW: Color32 = Color32::from_rgb(62, 207, 142);

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
    pick(ACCENT_COPPER_DIM, Color32::from_rgb(220, 250, 235))
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
            FontId::new(12.0, FontFamily::Monospace),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(12.0, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(12.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(10.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(13.5, FontFamily::Proportional),
        );

        style.spacing.item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
        style.spacing.button_padding = egui::vec2(SPACE_MD, 5.0);
        style.spacing.icon_spacing = 4.0;
        style.spacing.interact_size = egui::vec2(28.0, 28.0);
        style.spacing.menu_margin = Margin::same(SPACE_SM_I);
        style.spacing.window_margin = Margin::same(SPACE_MD_I);
        style.spacing.indent = 14.0;
        style.animation_time = 0.12;

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
        v.window_shadow = egui::Shadow::NONE;

        // Widgets — noninteractive
        v.widgets.noninteractive.bg_fill = BG_MEDIUM;
        v.widgets.noninteractive.weak_bg_fill = BG_MEDIUM;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DISABLED);
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — inactive
        v.widgets.inactive.bg_fill = BG_LIGHT;
        v.widgets.inactive.weak_bg_fill = BG_MEDIUM;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_DEFAULT);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — hovered
        v.widgets.hovered.bg_fill = BG_ELEVATED;
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(36, 36, 36);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — active (pressed)
        v.widgets.active.bg_fill = ACCENT_COPPER_DIM;
        v.widgets.active.weak_bg_fill = Color32::from_rgb(25, 64, 47);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_MD);

        // Widgets — open
        v.widgets.open.bg_fill = BG_ELEVATED;
        v.widgets.open.weak_bg_fill = BG_ELEVATED;
        v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.open.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_MD);

        v.selection.bg_fill = Color32::from_rgba_unmultiplied(62, 207, 142, 38);
        v.selection.stroke = Stroke::new(1.0, ACCENT_COPPER_LIGHT);

        v.override_text_color = Some(TEXT_PRIMARY);
        v.hyperlink_color = ACCENT_COPPER_LIGHT;
        v.interact_cursor = Some(egui::CursorIcon::PointingHand);

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
        v.window_shadow = egui::Shadow::NONE;

        v.widgets.noninteractive.bg_fill = bg_medium();
        v.widgets.noninteractive.weak_bg_fill = Color32::from_rgb(248, 250, 253);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_subtle());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_disabled());
        v.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.inactive.bg_fill = bg_medium();
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(249, 250, 252);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, border_default());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary());
        v.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.hovered.bg_fill = Color32::from_rgb(246, 248, 251);
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(241, 245, 250);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, border_glow());
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, text_primary());
        v.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.active.bg_fill = Color32::from_rgb(220, 250, 235);
        v.widgets.active.weak_bg_fill = Color32::from_rgb(230, 252, 241);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.active.fg_stroke = Stroke::new(2.0, text_primary());
        v.widgets.active.corner_radius = CornerRadius::same(RADIUS_MD);

        v.widgets.open.bg_fill = bg_elevated();
        v.widgets.open.weak_bg_fill = Color32::from_rgb(245, 248, 252);
        v.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_COPPER);
        v.widgets.open.fg_stroke = Stroke::new(1.5, text_primary());
        v.widgets.open.corner_radius = CornerRadius::same(RADIUS_MD);

        v.selection.bg_fill = Color32::from_rgba_unmultiplied(62, 207, 142, 38);
        v.selection.stroke = Stroke::new(1.0, ACCENT_COPPER);

        v.override_text_color = Some(text_primary());
        v.hyperlink_color = ACCENT_COPPER;
        v.interact_cursor = Some(egui::CursorIcon::PointingHand);

        v
    }
}

// ---------------------------------------------------------------------------
// Backward-compat entry point
// ---------------------------------------------------------------------------
pub fn configure_fonts(ctx: &egui::Context, language: &str) {
    let mut fonts = FontDefinitions::default();
    install_apple_system_fonts(&mut fonts);
    install_locale_ui_fonts(&mut fonts, language);
    install_cjk_font_fallbacks(&mut fonts);
    ctx.set_fonts(fonts);
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

fn install_locale_ui_fonts(fonts: &mut FontDefinitions, language: &str) {
    match language {
        "ko" => install_font(
            fonts,
            "ferrum_apple_sd_gothic_neo",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            &[FontFamily::Proportional],
            FontPlacement::Front,
        ),
        "zh-CN" => install_font(
            fonts,
            "ferrum_st_heiti",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            &[FontFamily::Proportional],
            FontPlacement::Front,
        ),
        _ => {}
    }
}

fn install_cjk_font_fallbacks(fonts: &mut FontDefinitions) {
    let candidates = [
        (
            "ferrum_apple_sd_gothic_neo",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
        ),
        (
            "ferrum_hiragino_sans_gb",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ),
        (
            "ferrum_st_heiti",
            "/System/Library/Fonts/STHeiti Medium.ttc",
        ),
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
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE).size(12.5))
        .fill(BG_DARKEST)
        .stroke(Stroke::new(1.0, ACCENT_EMERALD))
        .corner_radius(CornerRadius::same(255))
        .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn secondary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(text_secondary()).size(12.5))
        .fill(bg_medium())
        .stroke(Stroke::new(1.0, border_default()))
        .corner_radius(CornerRadius::same(RADIUS_MD))
        .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn ghost_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(text_secondary()).size(12.5))
        .fill(bg_darkest())
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(RADIUS_MD))
        .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn primary_icon_button(
    icon: egui::Image<'static>,
    text: impl Into<String>,
) -> egui::Button<'static> {
    egui::Button::image_and_text(
        icon,
        egui::RichText::new(text.into())
            .color(Color32::WHITE)
            .size(12.5),
    )
    .fill(BG_DARKEST)
    .stroke(Stroke::new(1.0, ACCENT_EMERALD))
    .corner_radius(CornerRadius::same(255))
    .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn secondary_icon_button(
    icon: egui::Image<'static>,
    text: impl Into<String>,
) -> egui::Button<'static> {
    egui::Button::image_and_text(
        icon,
        egui::RichText::new(text.into())
            .color(text_secondary())
            .size(12.5),
    )
    .fill(bg_medium())
    .stroke(Stroke::new(1.0, border_default()))
    .corner_radius(CornerRadius::same(RADIUS_MD))
    .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn ghost_icon_button(
    icon: egui::Image<'static>,
    text: impl Into<String>,
) -> egui::Button<'static> {
    egui::Button::image_and_text(
        icon,
        egui::RichText::new(text.into())
            .color(text_secondary())
            .size(12.5),
    )
    .fill(bg_darkest())
    .stroke(Stroke::new(1.0, border_subtle()))
    .corner_radius(CornerRadius::same(RADIUS_MD))
    .min_size(egui::vec2(0.0, BUTTON_HEIGHT))
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn text_input(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .background_color(input_bg())
        .text_color(text_primary())
        .margin(Margin::symmetric(INPUT_MARGIN_X, INPUT_MARGIN_Y))
        .min_size(egui::vec2(0.0, INPUT_HEIGHT))
        .vertical_align(egui::Align::Center)
}

pub fn mono_text_input(text: &mut String) -> egui::TextEdit<'_> {
    text_input(text).font(TextStyle::Monospace)
}

pub fn multiline_text_input(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::multiline(text)
        .background_color(input_bg())
        .text_color(text_primary())
        .margin(Margin::symmetric(INPUT_MARGIN_X, INPUT_MARGIN_Y))
}

pub fn multiline_mono_text_input(text: &mut String) -> egui::TextEdit<'_> {
    multiline_text_input(text).font(TextStyle::Monospace)
}

pub fn password_input(text: &mut String, reveal: bool) -> egui::TextEdit<'_> {
    text_input(text).password(!reveal)
}
