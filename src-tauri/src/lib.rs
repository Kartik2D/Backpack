mod commands;
mod metadata;
mod model;
mod platform;
mod store;
mod track;
mod window;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent,
};

use crate::commands::{
    add_apps, apply_metadata, get_apps, get_game_states, get_metadata, igdb_search, launch,
    remove_app, scan_games,
};
use crate::model::AppList;
use crate::store::load_apps;
use crate::track::GameStates;
use crate::window::{ensure_main_window, hide_on_close, show_window};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppList::default())
        .manage(GameStates::default())
        .invoke_handler(tauri::generate_handler![
            get_apps,
            get_game_states,
            add_apps,
            launch,
            scan_games,
            get_metadata,
            remove_app,
            igdb_search,
            apply_metadata
        ])
        .setup(|app| {
            *app.state::<AppList>().0.lock().unwrap() = load_apps(app.handle());
            let main_window = ensure_main_window(app.handle())?;
            hide_on_close(&main_window);

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                // Right-click opens the menu; left-click shows the window (consistent across platforms).
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_window(tray.app_handle());
                    }
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }

            tray.build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            // Keep the process alive (minimal, windowless) when the last window closes.
            RunEvent::ExitRequested { code, api, .. } if code.is_none() => {
                api.prevent_exit();
            }
            other => platform::handle_run_event(app, &other),
        });
}
