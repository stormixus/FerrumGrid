use eframe::egui::IconData;

const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app-icon.png");
const APP_ICON_LIGHT_PNG: &[u8] = include_bytes!("../assets/app-icon-light.png");

pub fn icon_for_dark_mode(is_dark: bool) -> IconData {
    #[cfg(target_os = "macos")]
    update_macos_dock_icon(is_dark);

    if is_dark {
        load_png_icon(APP_ICON_PNG)
    } else {
        load_png_icon(APP_ICON_LIGHT_PNG)
    }
}

#[cfg(target_os = "macos")]
pub fn update_macos_dock_icon(is_dark: bool) {
    use objc2::runtime::AnyObject;

    let png_bytes = if is_dark {
        APP_ICON_PNG
    } else {
        APP_ICON_LIGHT_PNG
    };

    unsafe {
        let app: *mut AnyObject = objc2::msg_send![objc2::class!(NSApplication), sharedApplication];
        if app.is_null() {
            return;
        }

        let ns_data: *mut AnyObject = objc2::msg_send![
            objc2::class!(NSData),
            dataWithBytes: png_bytes.as_ptr() as *const std::ffi::c_void,
            length: png_bytes.len()
        ];
        if ns_data.is_null() {
            return;
        }

        let ns_image: *mut AnyObject = objc2::msg_send![objc2::class!(NSImage), alloc];
        if ns_image.is_null() {
            return;
        }
        let ns_image: *mut AnyObject = objc2::msg_send![ns_image, initWithData: ns_data];
        if ns_image.is_null() {
            return;
        }

        let _: () = objc2::msg_send![app, setApplicationIconImage: ns_image];
        let _: () = objc2::msg_send![ns_image, release];
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
