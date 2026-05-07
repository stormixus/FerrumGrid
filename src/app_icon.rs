use eframe::egui::IconData;
use resvg::usvg::TreeParsing;
use resvg::{tiny_skia, usvg};

const SIZE: u32 = 256;
const APP_ICON_DARK_SVG: &[u8] = include_bytes!("../assets/app-icon-dark.svg");
const APP_ICON_LIGHT_SVG: &[u8] = include_bytes!("../assets/app-icon-light.svg");

pub fn icon_for_dark_mode(is_dark: bool) -> IconData {
    render_icon(if is_dark {
        APP_ICON_DARK_SVG
    } else {
        APP_ICON_LIGHT_SVG
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

fn render_icon(svg: &[u8]) -> IconData {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())
        .expect("embedded app-icon SVG must parse");
    let svg_size = tree.size.to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE).expect("non-zero icon size");
    let transform = tiny_skia::Transform::from_scale(
        SIZE as f32 / svg_size.width() as f32,
        SIZE as f32 / svg_size.height() as f32,
    );
    resvg::Tree::from_usvg(&tree).render(transform, &mut pixmap.as_mut());

    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for pixel in pixmap.pixels() {
        let demul = pixel.demultiply();
        rgba.extend_from_slice(&[demul.red(), demul.green(), demul.blue(), demul.alpha()]);
    }

    IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}
