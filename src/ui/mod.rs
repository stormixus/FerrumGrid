use eframe::egui;

pub mod about;
pub mod dialogs;
pub mod editor;
pub mod er_diagram;
pub mod grid;
pub mod icons;
pub mod icons_svg;
pub mod objects;
pub mod panels;
pub mod settings;
pub mod table_designer;
pub mod theme;
pub mod tree_browser;

/// Helper to render an SVG icon as a small image inline.
pub fn icon_img(ui: &mut egui::Ui, svg_content: &str, name: &str, size: f32) {
    let uri = format!("bytes://{}.svg", name);
    ui.ctx()
        .include_bytes(uri.clone(), svg_content.as_bytes().to_vec());
    ui.add(egui::Image::new(uri).fit_to_exact_size(egui::vec2(size, size)));
}

/// Helper to render an SVG icon with a label.
pub fn icon_label(
    ui: &mut egui::Ui,
    svg_content: &str,
    name: &str,
    label: impl Into<egui::WidgetText>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        icon_img(ui, svg_content, name, 14.0);
        ui.label(label);
    });
}
