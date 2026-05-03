use eframe::egui::{Color32, IconData};

const SIZE: u32 = 256;

#[derive(Clone, Copy)]
enum IconVariant {
    Dark,
    Light,
}

pub fn icon_for_dark_mode(is_dark: bool) -> IconData {
    render_icon(if is_dark {
        IconVariant::Dark
    } else {
        IconVariant::Light
    })
}

pub fn startup_dark_mode(appearance: &str) -> bool {
    match appearance {
        "light" => false,
        "dark" => true,
        _ => system_prefers_dark().unwrap_or(true),
    }
}

#[cfg(target_os = "macos")]
fn system_prefers_dark() -> Option<bool> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .ok()?;

    if !output.status.success() {
        return Some(false);
    }

    Some(
        String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains("dark"),
    )
}

#[cfg(not(target_os = "macos"))]
fn system_prefers_dark() -> Option<bool> {
    None
}

fn render_icon(variant: IconVariant) -> IconData {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let aa = 1.5 / SIZE as f32;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let nx = (x as f32 + 0.5) / SIZE as f32;
            let ny = (y as f32 + 0.5) / SIZE as f32;
            let outer = rounded_rect_sdf(nx, ny, 0.5, 0.5, 0.468, 0.468, 0.18);
            let icon_alpha = coverage_from_sdf(outer, aa);

            let mut pixel = Pixel::transparent();
            let bg = background_color(nx, ny, variant);
            pixel.over(bg, icon_alpha);

            let grid_alpha = grid_coverage(nx, ny) * icon_alpha;
            if grid_alpha > 0.0 {
                pixel.over(grid_color(variant), grid_alpha);
            }

            let border_alpha = border_coverage(outer, aa) * icon_alpha;
            if border_alpha > 0.0 {
                pixel.over(border_color(variant), border_alpha);
            }

            let shadow = letter_coverage(nx - 0.016, ny - 0.02) * icon_alpha;
            if shadow > 0.0 {
                pixel.over(Color::new(0.0, 0.0, 0.0), shadow * shadow_alpha(variant));
            }

            let letters = letter_coverage(nx, ny) * icon_alpha;
            if letters > 0.0 {
                pixel.over(letter_gradient(nx, ny, variant), letters);
            }

            let shine = letter_highlight(nx, ny) * letters;
            if shine > 0.0 {
                pixel.over(Color::new(1.0, 1.0, 1.0), shine);
            }

            rgba.extend_from_slice(&pixel.to_rgba8());
        }
    }

    IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

fn background_color(x: f32, y: f32, variant: IconVariant) -> Color {
    let t = (x * 0.58 + y * 0.42).clamp(0.0, 1.0);
    let (a, b, glow) = match variant {
        IconVariant::Dark => (
            Color::from_rgb(7, 11, 18),
            Color::from_rgb(18, 31, 49),
            Color::from_rgb(33, 72, 92),
        ),
        IconVariant::Light => (
            Color::from_rgb(249, 252, 255),
            Color::from_rgb(225, 234, 247),
            Color::from_rgb(205, 230, 237),
        ),
    };

    let mut color = a.lerp(b, t);
    let dx = x - 0.18;
    let dy = y - 0.12;
    let glow_alpha = (1.0 - ((dx * dx + dy * dy).sqrt() / 0.64)).clamp(0.0, 1.0) * 0.28;
    color = color.lerp(glow, glow_alpha);
    color
}

fn grid_color(variant: IconVariant) -> Color {
    match variant {
        IconVariant::Dark => Color::from_rgb(105, 157, 183),
        IconVariant::Light => Color::from_rgb(101, 128, 158),
    }
}

fn border_color(variant: IconVariant) -> Color {
    match variant {
        IconVariant::Dark => Color::from_rgb(83, 111, 137),
        IconVariant::Light => Color::from_rgb(169, 184, 204),
    }
}

fn shadow_alpha(variant: IconVariant) -> f32 {
    match variant {
        IconVariant::Dark => 0.38,
        IconVariant::Light => 0.16,
    }
}

fn letter_gradient(x: f32, y: f32, variant: IconVariant) -> Color {
    let t = ((x * 0.82 + y * 0.32) - 0.16).clamp(0.0, 1.0);
    let (copper, teal, blue) = match variant {
        IconVariant::Dark => (
            Color::from_rgb(255, 143, 84),
            Color::from_rgb(22, 216, 194),
            Color::from_rgb(104, 134, 255),
        ),
        IconVariant::Light => (
            Color::from_rgb(210, 98, 42),
            Color::from_rgb(0, 154, 142),
            Color::from_rgb(54, 98, 222),
        ),
    };

    if t < 0.52 {
        copper.lerp(teal, t / 0.52)
    } else {
        teal.lerp(blue, (t - 0.52) / 0.48)
    }
}

fn grid_coverage(x: f32, y: f32) -> f32 {
    let spacing = 28.0 / SIZE as f32;
    let line = 0.62 / SIZE as f32;
    let offset = 15.0 / SIZE as f32;
    let dx = repeating_distance(x - offset, spacing);
    let dy = repeating_distance(y - offset, spacing);
    let vertical = (1.0 - dx / line).clamp(0.0, 1.0);
    let horizontal = (1.0 - dy / line).clamp(0.0, 1.0);
    let lines = vertical.max(horizontal) * 0.11;
    let intersections = (vertical * horizontal).powf(0.45) * 0.16;
    (lines + intersections).clamp(0.0, 0.18)
}

fn border_coverage(sdf: f32, aa: f32) -> f32 {
    let stroke = 1.2 / SIZE as f32;
    let d = sdf.abs();
    (1.0 - ((d - stroke).max(0.0) / aa)).clamp(0.0, 1.0) * 0.55
}

fn letter_coverage(x: f32, y: f32) -> f32 {
    let f = union_many(&[
        rounded_rect_coverage(x, y, 0.275, 0.5, 0.07, 0.31, 0.025),
        rounded_rect_coverage(x, y, 0.43, 0.265, 0.205, 0.065, 0.026),
        rounded_rect_coverage(x, y, 0.395, 0.462, 0.17, 0.056, 0.024),
    ]);

    let outer = ellipse_coverage(x, y, 0.655, 0.545, 0.205, 0.19);
    let inner = ellipse_coverage(x, y, 0.655, 0.545, 0.097, 0.09);
    let notch = rounded_rect_coverage(x, y, 0.81, 0.405, 0.095, 0.07, 0.03);
    let bowl = (outer * (1.0 - inner) * (1.0 - notch * 0.92)).clamp(0.0, 1.0);

    let descender = rounded_rect_coverage(x, y, 0.805, 0.655, 0.056, 0.215, 0.03);
    let hook = rounded_rect_coverage(x, y, 0.69, 0.825, 0.16, 0.055, 0.034);
    let hook_cut = rounded_rect_coverage(x, y, 0.62, 0.762, 0.09, 0.043, 0.025);
    let g = union_many(&[bowl, descender, hook * (1.0 - hook_cut * 0.82)]);

    union_many(&[f, g])
}

fn letter_highlight(x: f32, y: f32) -> f32 {
    let diagonal = (0.46 - (x * 0.32 + y * 0.68)).clamp(0.0, 1.0);
    diagonal * 0.16
}

fn rounded_rect_coverage(x: f32, y: f32, cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let sdf = rounded_rect_sdf(x, y, cx, cy, hx, hy, r);
    coverage_from_sdf(sdf, 1.2 / SIZE as f32)
}

fn ellipse_coverage(x: f32, y: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let dx = (x - cx) / rx;
    let dy = (y - cy) / ry;
    let sdf = ((dx * dx + dy * dy).sqrt() - 1.0) * rx.min(ry);
    coverage_from_sdf(sdf, 1.2 / SIZE as f32)
}

fn rounded_rect_sdf(x: f32, y: f32, cx: f32, cy: f32, hx: f32, hy: f32, r: f32) -> f32 {
    let qx = (x - cx).abs() - hx + r;
    let qy = (y - cy).abs() - hy + r;
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let inside = qx.max(qy).min(0.0);
    outside + inside - r
}

fn coverage_from_sdf(sdf: f32, aa: f32) -> f32 {
    (0.5 - sdf / aa).clamp(0.0, 1.0)
}

fn repeating_distance(value: f32, spacing: f32) -> f32 {
    let v = value.rem_euclid(spacing);
    v.min(spacing - v)
}

fn union_many(values: &[f32]) -> f32 {
    values
        .iter()
        .fold(0.0, |acc, value| 1.0 - (1.0 - acc) * (1.0 - value))
        .clamp(0.0, 1.0)
}

#[derive(Clone, Copy)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
        }
    }
}

#[derive(Clone, Copy)]
struct Pixel {
    color: Color,
    alpha: f32,
}

impl Pixel {
    fn transparent() -> Self {
        Self {
            color: Color::new(0.0, 0.0, 0.0),
            alpha: 0.0,
        }
    }

    fn over(&mut self, color: Color, alpha: f32) {
        let alpha = alpha.clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return;
        }

        let out_alpha = alpha + self.alpha * (1.0 - alpha);
        if out_alpha <= f32::EPSILON {
            return;
        }

        self.color = Color {
            r: (color.r * alpha + self.color.r * self.alpha * (1.0 - alpha)) / out_alpha,
            g: (color.g * alpha + self.color.g * self.alpha * (1.0 - alpha)) / out_alpha,
            b: (color.b * alpha + self.color.b * self.alpha * (1.0 - alpha)) / out_alpha,
        };
        self.alpha = out_alpha;
    }

    fn to_rgba8(self) -> [u8; 4] {
        [
            channel(self.color.r),
            channel(self.color.g),
            channel(self.color.b),
            channel(self.alpha),
        ]
    }
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl From<Color32> for Color {
    fn from(value: Color32) -> Self {
        Self::from_rgb(value.r(), value.g(), value.b())
    }
}
