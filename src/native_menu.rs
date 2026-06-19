use crate::state::AppState;
#[cfg(target_os = "macos")]
use crate::state::MainView;
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
    edit_undo_id: MenuId,
    edit_redo_id: MenuId,
    edit_cut_id: MenuId,
    edit_copy_id: MenuId,
    edit_paste_id: MenuId,
    edit_select_all_id: MenuId,
    new_connection_id: MenuId,
    new_tab_id: MenuId,
    close_window_id: MenuId,
    show_main_window_id: MenuId,
    quit_id: MenuId,
    query_view_id: MenuId,
    toggle_theme_id: MenuId,
    er_diagram_id: MenuId,
    table_designer_id: MenuId,
    prisma_id: MenuId,
    monitoring_id: MenuId,
    session_monitor_id: MenuId,
    schema_diff_id: MenuId,
    new_window_id: MenuId,
}

#[cfg(not(target_os = "macos"))]
pub struct NativeMenu;

#[derive(Default)]
pub struct NativeMenuActions {
    pub hide_main_window: bool,
    pub show_main_window: bool,
    pub quit_requested: bool,
}

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
        let quit = MenuItem::with_id(
            "quit",
            crate::i18n::t("menu_quit"),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyQ)),
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
                &quit,
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
        let close_window = MenuItem::with_id(
            "close_window",
            crate::i18n::t("menu_close_window"),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyW)),
        );
        let file_menu = Submenu::with_items(
            crate::i18n::t("menu_file"),
            true,
            &[
                &new_connection,
                &new_tab,
                &PredefinedMenuItem::separator(),
                &close_window,
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

        let edit_undo = MenuItem::with_id(
            "edit_undo",
            "Undo",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyZ)),
        );
        let edit_redo = MenuItem::with_id(
            "edit_redo",
            "Redo",
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyZ,
            )),
        );
        let edit_cut = MenuItem::with_id(
            "edit_cut",
            "Cut",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyX)),
        );
        let edit_copy = MenuItem::with_id(
            "edit_copy",
            "Copy",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyC)),
        );
        let edit_paste = MenuItem::with_id(
            "edit_paste",
            "Paste",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyV)),
        );
        let edit_select_all = MenuItem::with_id(
            "edit_select_all",
            "Select All",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyA)),
        );
        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &edit_undo,
                &edit_redo,
                &PredefinedMenuItem::separator(),
                &edit_cut,
                &edit_copy,
                &edit_paste,
                &edit_select_all,
            ],
        )
        .expect("failed to build FerrumGrid edit menu");

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

        let monitoring =
            MenuItem::with_id("monitoring", crate::i18n::t("menu_monitoring"), true, None);

        let table_designer = MenuItem::with_id(
            "table_designer",
            crate::i18n::t("menu_table_designer"),
            true,
            None,
        );
        let prisma = MenuItem::with_id("prisma", crate::i18n::t("menu_prisma"), true, None);
        let session_monitor = MenuItem::with_id(
            "session_monitor",
            crate::i18n::t("menu_session_monitor"),
            true,
            None,
        );
        let schema_diff = MenuItem::with_id(
            "schema_diff",
            crate::i18n::t("menu_schema_diff"),
            true,
            None,
        );
        let new_window = MenuItem::with_id(
            "new_window",
            crate::i18n::t("menu_new_window"),
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyN,
            )),
        );
        let tools_menu = Submenu::with_items(
            crate::i18n::t("menu_tools"),
            true,
            &[
                &table_designer,
                &prisma,
                &monitoring,
                &session_monitor,
                &schema_diff,
                &new_window,
            ],
        )
        .expect("failed to build FerrumGrid tools menu");

        let show_main_window = MenuItem::with_id(
            "show_main_window",
            crate::i18n::t("menu_show_main_window"),
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Digit0)),
        );
        let window_menu = Submenu::with_items(
            "Window",
            true,
            &[
                &show_main_window,
                &PredefinedMenuItem::separator(),
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
            edit_undo_id: edit_undo.id().clone(),
            edit_redo_id: edit_redo.id().clone(),
            edit_cut_id: edit_cut.id().clone(),
            edit_copy_id: edit_copy.id().clone(),
            edit_paste_id: edit_paste.id().clone(),
            edit_select_all_id: edit_select_all.id().clone(),
            new_connection_id: new_connection.id().clone(),
            new_tab_id: new_tab.id().clone(),
            close_window_id: close_window.id().clone(),
            show_main_window_id: show_main_window.id().clone(),
            quit_id: quit.id().clone(),
            query_view_id: query_view.id().clone(),
            toggle_theme_id: toggle_theme.id().clone(),
            er_diagram_id: er_diagram.id().clone(),
            table_designer_id: table_designer.id().clone(),
            prisma_id: prisma.id().clone(),
            monitoring_id: monitoring.id().clone(),
            session_monitor_id: session_monitor.id().clone(),
            schema_diff_id: schema_diff.id().clone(),
            new_window_id: new_window.id().clone(),
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
    ) -> NativeMenuActions {
        let mut actions = NativeMenuActions::default();
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            let id = event.id();
            if id == &self.about_id {
                state.show_about_dialog = true;
            } else if id == &self.settings_id {
                state.show_settings_dialog = true;
            } else if id == &self.edit_undo_id {
                push_key_event(ctx, egui::Key::Z, egui::Modifiers::COMMAND);
            } else if id == &self.edit_redo_id {
                push_key_event(
                    ctx,
                    egui::Key::Z,
                    egui::Modifiers::SHIFT | egui::Modifiers::COMMAND,
                );
            } else if id == &self.edit_cut_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::RequestCut);
            } else if id == &self.edit_copy_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::RequestCopy);
            } else if id == &self.edit_paste_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
            } else if id == &self.edit_select_all_id {
                push_key_event(ctx, egui::Key::A, egui::Modifiers::COMMAND);
            } else if id == &self.new_connection_id {
                state.show_connection_dialog = true;
                state.connection_dialog = Default::default();
            } else if id == &self.new_tab_id {
                let n = state.editor_tabs.len() + 1;
                state
                    .editor_tabs
                    .push(crate::types::EditorTab::new(format!("Query {n}")));
                state.active_tab = state.editor_tabs.len() - 1;
                state.open_workspace_main_view(MainView::Query);
            } else if id == &self.close_window_id {
                actions.hide_main_window = true;
            } else if id == &self.show_main_window_id {
                actions.show_main_window = true;
            } else if id == &self.quit_id {
                actions.quit_requested = true;
            } else if id == &self.query_view_id {
                state.open_workspace_main_view(MainView::Query);
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
                state.open_workspace_main_view(MainView::Model);
                state.er_diagram.show_diagram = true;
            } else if id == &self.table_designer_id {
                crate::ui::table_designer::open_for_new_table(state);
            } else if id == &self.prisma_id {
                crate::prisma::ui::open_prisma_window(state);
            } else if id == &self.monitoring_id {
                state.show_monitoring_window = true;
            } else if id == &self.session_monitor_id {
                state.show_session_monitor = true;
                state.sessions_needs_fetch = true;
            } else if id == &self.schema_diff_id {
                state.show_schema_diff_window = true;
            } else if id == &self.new_window_id {
                crate::state::spawn_new_window();
            }
        }
        actions
    }

    #[cfg(not(target_os = "macos"))]
    pub fn handle_events(
        &self,
        _ctx: &egui::Context,
        _state: &mut AppState,
        _settings: &mut AppSettings,
    ) -> NativeMenuActions {
        NativeMenuActions::default()
    }
}

#[cfg(target_os = "macos")]
fn push_key_event(ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) {
    ctx.input_mut(|input| {
        input.events.push(egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        });
    });
}

#[cfg(target_os = "macos")]
impl Drop for NativeMenu {
    fn drop(&mut self) {
        if self.installed {
            self.menu_bar.remove_for_nsapp();
        }
    }
}
