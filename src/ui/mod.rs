use eframe::egui;

pub mod about;
pub mod diagnostics_panel;
pub mod dialogs;
pub mod editor;
pub mod er_diagram;
pub mod grid;
pub mod grid_dispatch;
pub mod icons_svg;
pub mod objects;
pub mod panels;
pub mod settings;
pub mod table_designer;
pub mod theme;
pub mod tree_browser;
pub mod vault;

/// Helper to render an SVG icon as a small image inline.
pub fn icon_img(ui: &mut egui::Ui, svg_content: &str, name: &str, size: f32) {
    ui.add(icon_image(ui, svg_content, name, size));
}

pub fn icon_image(ui: &egui::Ui, svg_content: &str, name: &str, size: f32) -> egui::Image<'static> {
    let uri = format!("bytes://{}.svg", name);
    ui.ctx()
        .include_bytes(uri.clone(), svg_content.as_bytes().to_vec());
    egui::Image::new(uri).fit_to_exact_size(egui::vec2(size, size))
}

/// Render an SVG that uses `currentColor` with an explicit egui theme color.
pub fn icon_img_tinted(
    ui: &mut egui::Ui,
    svg_content: &str,
    name: &str,
    size: f32,
    color: egui::Color32,
) {
    ui.add(icon_image_tinted(ui, svg_content, name, size, color));
}

pub fn icon_image_tinted(
    ui: &egui::Ui,
    svg_content: &str,
    name: &str,
    size: f32,
    color: egui::Color32,
) -> egui::Image<'static> {
    let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
    let svg = svg_content.replace("currentColor", &hex);
    let uri = format!(
        "bytes://{}_{:02x}{:02x}{:02x}.svg",
        name,
        color.r(),
        color.g(),
        color.b()
    );
    ui.ctx().include_bytes(uri.clone(), svg.into_bytes());
    egui::Image::new(uri).fit_to_exact_size(egui::vec2(size, size))
}
