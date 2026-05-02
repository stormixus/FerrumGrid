//! Native macOS menu bar (top-screen system menu) via `muda`.
//!
//! On macOS we install an NSMenu attached to NSApplication so users get the
//! standard global menu bar (FerrumGrid · File · Edit · View · Query · Window
//! · Help). Items dispatch back into the app via `MenuAction` events that the
//! `update()` loop drains each frame.
//!
//! On non-macOS platforms this module is a no-op (Phase 2: integrate with
//! winit's HWND on Windows / GTK menu on Linux).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewConnection,
    NewQueryTab,
    CloseTab,
    RunQuery,
    StopQuery,
    OpenCommandPalette,
    ToggleSidebar,
    ToggleResultPanel,
    ThemeAuto,
    ThemeLight,
    ThemeDark,
    About,
}

#[cfg(target_os = "macos")]
mod imp {
    use super::MenuAction;
    use muda::{
        accelerator::{Accelerator, Code, Modifiers},
        AboutMetadata, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu,
    };

    /// Build the menu and attach it to NSApplication. Must be called after
    /// NSApp is up — i.e. inside `eframe::App::new()` or later. The returned
    /// Menu is leaked so it lives for the entire process.
    pub fn install() {
        let menu = Menu::new();

        // ----- App menu (FerrumGrid) -----
        let app_menu = Submenu::new("FerrumGrid", true);
        let about_meta = AboutMetadata {
            name: Some("FerrumGrid".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            short_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            comments: Some("PostgreSQL client".to_string()),
            ..Default::default()
        };
        app_menu
            .append_items(&[
                &PredefinedMenuItem::about(Some("About FerrumGrid"), Some(about_meta)),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .ok();

        // ----- File -----
        let file_menu = Submenu::new("File", true);
        file_menu
            .append_items(&[
                &MenuItem::with_id(
                    MenuId::new("new_connection"),
                    "New Connection\u{2026}",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN)),
                ),
                &MenuItem::with_id(
                    MenuId::new("new_query_tab"),
                    "New Query Tab",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyT)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    MenuId::new("close_tab"),
                    "Close Tab",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyW)),
                ),
            ])
            .ok();

        // ----- Edit (system-provided text editing) -----
        let edit_menu = Submenu::new("Edit", true);
        edit_menu
            .append_items(&[
                &PredefinedMenuItem::undo(None),
                &PredefinedMenuItem::redo(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::cut(None),
                &PredefinedMenuItem::copy(None),
                &PredefinedMenuItem::paste(None),
                &PredefinedMenuItem::select_all(None),
            ])
            .ok();

        // ----- View -----
        let view_menu = Submenu::new("View", true);
        view_menu
            .append_items(&[
                &MenuItem::with_id(
                    MenuId::new("toggle_sidebar"),
                    "Toggle Sidebar",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyB)),
                ),
                &MenuItem::with_id(
                    MenuId::new("toggle_result_panel"),
                    "Toggle Result Panel",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyJ)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(MenuId::new("theme_auto"), "Appearance: Auto", true, None),
                &MenuItem::with_id(MenuId::new("theme_light"), "Appearance: Light", true, None),
                &MenuItem::with_id(MenuId::new("theme_dark"), "Appearance: Dark", true, None),
            ])
            .ok();

        // ----- Query -----
        let query_menu = Submenu::new("Query", true);
        query_menu
            .append_items(&[
                &MenuItem::with_id(
                    MenuId::new("run_query"),
                    "Run",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::Enter)),
                ),
                &MenuItem::with_id(
                    MenuId::new("stop_query"),
                    "Stop",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::Period)),
                ),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id(
                    MenuId::new("open_palette"),
                    "Command Palette\u{2026}",
                    true,
                    Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyK)),
                ),
            ])
            .ok();

        // ----- Window -----
        let window_menu = Submenu::new("Window", true);
        window_menu
            .append_items(&[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::bring_all_to_front(None),
            ])
            .ok();

        // ----- Help -----
        let help_menu = Submenu::new("Help", true);

        menu.append_items(&[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &query_menu,
            &window_menu,
            &help_menu,
        ])
        .ok();

        menu.init_for_nsapp();
        window_menu.set_as_windows_menu_for_nsapp();
        help_menu.set_as_help_menu_for_nsapp();

        // Leak so menu and submenus stay alive for the process lifetime.
        std::mem::forget(menu);
        std::mem::forget(app_menu);
        std::mem::forget(file_menu);
        std::mem::forget(edit_menu);
        std::mem::forget(view_menu);
        std::mem::forget(query_menu);
        std::mem::forget(window_menu);
        std::mem::forget(help_menu);
    }

    pub fn drain() -> Vec<MenuAction> {
        let mut out = Vec::new();
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id_str: &str = event.id.as_ref();
            let action = match id_str {
                "new_connection" => MenuAction::NewConnection,
                "new_query_tab" => MenuAction::NewQueryTab,
                "close_tab" => MenuAction::CloseTab,
                "run_query" => MenuAction::RunQuery,
                "stop_query" => MenuAction::StopQuery,
                "open_palette" => MenuAction::OpenCommandPalette,
                "toggle_sidebar" => MenuAction::ToggleSidebar,
                "toggle_result_panel" => MenuAction::ToggleResultPanel,
                "theme_auto" => MenuAction::ThemeAuto,
                "theme_light" => MenuAction::ThemeLight,
                "theme_dark" => MenuAction::ThemeDark,
                "about" => MenuAction::About,
                _ => continue,
            };
            out.push(action);
        }
        out
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::MenuAction;

    pub fn install() {}

    pub fn drain() -> Vec<MenuAction> {
        Vec::new()
    }
}

pub use imp::{drain, install};
