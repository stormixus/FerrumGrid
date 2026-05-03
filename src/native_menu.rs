use crate::state::{AppState, MainView};
use crate::storage::settings::AppSettings;
use eframe::egui;

#[cfg(target_os = "macos")]
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    Menu, MenuId, MenuItem, PredefinedMenuItem, Submenu,
};

#[cfg(target_os = "macos")]
pub struct NativeMenu {
    menu_bar: Menu,
    installed: bool,
    about_id: MenuId,
    settings_id: MenuId,
    new_connection_id: MenuId,
    new_tab_id: MenuId,
    query_view_id: MenuId,
    toggle_theme_id: MenuId,
    er_diagram_id: MenuId,
    table_designer_id: MenuId,
    prisma_id: MenuId,
}

#[cfg(not(target_os = "macos"))]
pub struct NativeMenu;

impl NativeMenu {
    #[cfg(target_os = "macos")]
    pub fn install() -> Self {
        let menu_bar = Menu::new();

        let app_menu = Submenu::new("FerrumGrid", true);
        let about = MenuItem::with_id("about", crate::i18n::t("menu_about"), true, None);
        let settings = MenuItem::with_id(
            "settings",
            format!("{}...", crate::i18n::t("menu_settings")),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma)),
        );
        app_menu
            .append_items(&[
                &about,
                &PredefinedMenuItem::separator(),
                &settings,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .expect("failed to build FerrumGrid app menu");

        let new_connection = MenuItem::with_id(
            "new_connection",
            crate::i18n::t("menu_new_connection"),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN)),
        );
        let new_tab = MenuItem::with_id(
            "new_tab",
            crate::i18n::t("menu_new_tab"),
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyN,
            )),
        );
        let file_menu = Submenu::with_items(
            crate::i18n::t("menu_file"),
            true,
            &[
                &new_connection,
                &new_tab,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::close_window(None),
            ],
        )
        .expect("failed to build FerrumGrid file menu");

        let query_view = MenuItem::with_id(
            "query_view",
            crate::i18n::t("menu_query"),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyL)),
        );
        let query_menu =
            Submenu::with_items(crate::i18n::t("menu_query"), true, &[&query_view, &new_tab])
                .expect("failed to build FerrumGrid query menu");

        let toggle_theme = MenuItem::with_id(
            "toggle_theme",
            format!(
                "{} / {}",
                crate::i18n::t("menu_light_mode"),
                crate::i18n::t("menu_dark_mode")
            ),
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyD,
            )),
        );
        let er_diagram =
            MenuItem::with_id("er_diagram", crate::i18n::t("menu_er_diagram"), true, None);
        let view_menu = Submenu::with_items(
            crate::i18n::t("menu_view"),
            true,
            &[
                &toggle_theme,
                &PredefinedMenuItem::separator(),
                &er_diagram,
                &PredefinedMenuItem::fullscreen(None),
            ],
        )
        .expect("failed to build FerrumGrid view menu");

        let table_designer = MenuItem::with_id(
            "table_designer",
            crate::i18n::t("menu_table_designer"),
            true,
            None,
        );
        let prisma = MenuItem::with_id("prisma", crate::i18n::t("menu_prisma"), true, None);
        let tools_menu = Submenu::with_items(
            crate::i18n::t("menu_tools"),
            true,
            &[&table_designer, &prisma],
        )
        .expect("failed to build FerrumGrid tools menu");

        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(None),
                &PredefinedMenuItem::redo(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::cut(None),
                &PredefinedMenuItem::copy(None),
                &PredefinedMenuItem::paste(None),
                &PredefinedMenuItem::select_all(None),
            ],
        )
        .expect("failed to build FerrumGrid edit menu");

        let window_menu = Submenu::with_items(
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(None),
                &PredefinedMenuItem::bring_all_to_front(None),
            ],
        )
        .expect("failed to build FerrumGrid window menu");

        menu_bar
            .append_items(&[
                &app_menu,
                &file_menu,
                &edit_menu,
                &query_menu,
                &view_menu,
                &tools_menu,
                &window_menu,
            ])
            .expect("failed to install FerrumGrid menu items");
        window_menu.set_as_windows_menu_for_nsapp();
        menu_bar.init_for_nsapp();

        Self {
            menu_bar,
            installed: true,
            about_id: about.id().clone(),
            settings_id: settings.id().clone(),
            new_connection_id: new_connection.id().clone(),
            new_tab_id: new_tab.id().clone(),
            query_view_id: query_view.id().clone(),
            toggle_theme_id: toggle_theme.id().clone(),
            er_diagram_id: er_diagram.id().clone(),
            table_designer_id: table_designer.id().clone(),
            prisma_id: prisma.id().clone(),
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn install() -> Self {
        Self
    }

    #[cfg(target_os = "macos")]
    pub fn refresh_locale(&mut self) {
        self.menu_bar.remove_for_nsapp();
        self.installed = false;
        *self = Self::install();
    }

    #[cfg(not(target_os = "macos"))]
    pub fn refresh_locale(&mut self) {}

    #[cfg(target_os = "macos")]
    pub fn handle_events(
        &self,
        ctx: &egui::Context,
        state: &mut AppState,
        settings: &mut AppSettings,
    ) {
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            let id = event.id();
            if id == &self.about_id {
                state.show_about_dialog = true;
            } else if id == &self.settings_id {
                state.show_settings_dialog = true;
            } else if id == &self.new_connection_id {
                state.show_connection_dialog = true;
                state.connection_dialog = Default::default();
            } else if id == &self.new_tab_id {
                let n = state.editor_tabs.len() + 1;
                state
                    .editor_tabs
                    .push(crate::types::EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
                state.active_main_view = MainView::Query;
            } else if id == &self.query_view_id {
                state.active_main_view = MainView::Query;
            } else if id == &self.toggle_theme_id {
                if ctx.style().visuals.dark_mode {
                    crate::ui::theme::FerrumTheme::apply_light(ctx);
                    settings.appearance = "light".to_string();
                    settings.dark_mode = false;
                } else {
                    crate::ui::theme::FerrumTheme::apply_dark(ctx);
                    settings.appearance = "dark".to_string();
                    settings.dark_mode = true;
                }
                crate::storage::settings::save_settings(settings);
            } else if id == &self.er_diagram_id {
                state.er_diagram.show_diagram = !state.er_diagram.show_diagram;
            } else if id == &self.table_designer_id {
                crate::ui::table_designer::open_for_new_table(state);
            } else if id == &self.prisma_id {
                crate::prisma::ui::open_prisma_window(state);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn handle_events(
        &self,
        _ctx: &egui::Context,
        _state: &mut AppState,
        _settings: &mut AppSettings,
    ) {
    }
}

#[cfg(target_os = "macos")]
impl Drop for NativeMenu {
    fn drop(&mut self) {
        if self.installed {
            self.menu_bar.remove_for_nsapp();
        }
    }
}
