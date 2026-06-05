use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};

#[derive(Clone, Serialize, Deserialize)]
struct App {
    path: String,
    name: String,
    image: String,
    description: String,
}

#[derive(Default)]
struct AppList(Mutex<Vec<App>>);

fn apps_file(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("apps.json"))
}

fn load_apps(app: &tauri::AppHandle) -> Vec<App> {
    apps_file(app)
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_apps(app: &tauri::AppHandle, list: &[App]) {
    if let Some(file) = apps_file(app) {
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(list) {
            let _ = std::fs::write(file, json);
        }
    }
}

// Extract the app's own icon as a base64 PNG data URI so it can be stored and
// rendered offline. macOS only for now; other platforms get no icon.
#[cfg(target_os = "macos")]
fn app_icon(path: &str) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let resources = Path::new(path).join("Contents/Resources");
    // Prefer the icon named in Info.plist, else fall back to the first .icns found.
    let icns = std::process::Command::new("defaults")
        .arg("read")
        .arg(Path::new(path).join("Contents/Info"))
        .arg("CFBundleIconFile")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .map(|name| {
            let p = resources.join(&name);
            if p.extension().is_some() {
                p
            } else {
                resources.join(format!("{name}.icns"))
            }
        })
        .filter(|p| p.exists())
        .or_else(|| {
            std::fs::read_dir(&resources).ok()?.flatten().find_map(|e| {
                let p = e.path();
                (p.extension()?.eq_ignore_ascii_case("icns")).then_some(p)
            })
        })?;

    let png = std::env::temp_dir().join("backpack_icon.png");
    Command::new("sips")
        .args(["-s", "format", "png"])
        .arg(&icns)
        .arg("--out")
        .arg(&png)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let bytes = std::fs::read(&png).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(not(target_os = "macos"))]
fn app_icon(_path: &str) -> Option<String> {
    None
}

fn file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn show_window(app: &tauri::AppHandle) {
    let window = match app.get_webview_window("main") {
        Some(w) => Some(w),
        None => WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
            .title("Backpack")
            .inner_size(800.0, 600.0)
            .min_inner_size(360.0, 300.0)
            .theme(Some(tauri::Theme::Dark))
            .focused(true)
            .build()
            .ok(),
    };
    if let Some(w) = window {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

// Launch the target and block until it exits.
#[cfg(target_os = "macos")]
fn wait_for_app(path: &str) {
    // `.app` bundles aren't directly executable; `open -W` launches and waits.
    let _ = Command::new("open").arg("-W").arg(path).status();
}

#[cfg(not(target_os = "macos"))]
fn wait_for_app(path: &str) {
    // On Windows/Linux the dropped path is the executable itself.
    if let Ok(mut child) = Command::new(path).spawn() {
        let _ = child.wait();
    }
}

#[tauri::command]
fn get_apps(list: tauri::State<AppList>) -> Vec<App> {
    list.0.lock().unwrap().clone()
}

#[tauri::command]
fn add_apps(paths: Vec<String>, app: tauri::AppHandle) -> Vec<App> {
    let state = app.state::<AppList>();
    for path in paths {
        let image = app_icon(&path).unwrap_or_default();
        state.0.lock().unwrap().push(App {
            name: file_stem(&path),
            path,
            image,
            description: String::new(),
        });
    }
    let list = state.0.lock().unwrap().clone();
    save_apps(&app, &list);
    list
}

#[tauri::command]
fn launch(path: String, window: tauri::WebviewWindow, app: tauri::AppHandle) {
    let _ = window.close();
    std::thread::spawn(move || {
        wait_for_app(&path);
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || show_window(&handle));
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppList::default())
        .invoke_handler(tauri::generate_handler![get_apps, add_apps, launch])
        .setup(|app| {
            *app.state::<AppList>().0.lock().unwrap() = load_apps(app.handle());

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
            #[cfg(target_os = "macos")]
            RunEvent::Reopen { .. } => show_window(app),
            _ => {}
        });
}
