use eframe::egui::IconData;

const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app-icon.png");
const APP_ICON_LIGHT_PNG: &[u8] = include_bytes!("../assets/app-icon-light.png");

pub fn icon_for_dark_mode(is_dark: bool) -> IconData {
    if is_dark {
        load_png_icon(APP_ICON_PNG)
    } else {
        load_png_icon(APP_ICON_LIGHT_PNG)
    }
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

fn load_png_icon(bytes: &[u8]) -> IconData {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("embedded app-icon PNG must parse")
        .into_rgba8();
    let width = img.width();
    let height = img.height();
    IconData {
        rgba: img.into_raw(),
        width,
        height,
    }
}
