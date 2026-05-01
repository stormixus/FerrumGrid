mod app;
mod db;
mod state;
mod storage;
mod types;
mod ui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ferrumgrid=info".parse().unwrap()),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FerrumGrid")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FerrumGrid",
        options,
        Box::new(|cc| Ok(Box::new(app::FerrumGridApp::new(cc)))),
    )
}
