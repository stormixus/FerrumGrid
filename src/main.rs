mod app;
mod app_icon;
mod connection_url;
mod db;
mod i18n;
mod korean_keyboard;
mod native_menu;
mod prisma;
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

    #[cfg(target_os = "macos")]
    use winit::platform::macos::EventLoopBuilderExtMacOS;

    let startup_settings = storage::settings::load_settings();
    let startup_icon =
        app_icon::icon_for_dark_mode(app_icon::startup_dark_mode(&startup_settings.appearance));

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FerrumGrid")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(startup_icon),
        #[cfg(target_os = "macos")]
        event_loop_builder: Some(Box::new(|builder| {
            builder.with_default_menu(false);
        })),
        ..Default::default()
    };

    eframe::run_native(
        "FerrumGrid",
        options,
        Box::new(|cc| Ok(Box::new(app::FerrumGridApp::new(cc)))),
    )
}
