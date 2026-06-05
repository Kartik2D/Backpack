use std::collections::HashSet;
use std::path::PathBuf;

use tauri::Manager;

use crate::model::App;
use crate::platform;

fn apps_file(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("apps.json"))
}

pub fn load_apps(app: &tauri::AppHandle) -> Vec<App> {
    let mut apps: Vec<App> = apps_file(app)
        .and_then(|f| std::fs::read_to_string(f).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    for item in &mut apps {
        if item.original_name.is_empty() {
            let stem = platform::file_stem(&item.path);
            item.original_name = if stem.is_empty() {
                item.name.clone()
            } else {
                stem
            };
        }
    }
    apps
}

pub fn save_apps(app: &tauri::AppHandle, list: &[App]) {
    if let Some(file) = apps_file(app) {
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(list) {
            let _ = std::fs::write(file, json);
        }
    }
}

// Case-insensitive path key for duplicate detection on Windows.
pub fn normalize_path_key(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

pub fn dedupe_apps(mut apps: Vec<App>) -> Vec<App> {
    let mut seen = HashSet::new();
    apps.retain(|a| seen.insert(normalize_path_key(&a.path)));
    apps
}

pub fn clean_apps(apps: Vec<App>) -> Vec<App> {
    dedupe_apps(platform::prune_missing(apps))
}

// Existence check for manually dropped local paths (.exe, .app, .lnk).
pub fn local_path_exists(path: &str) -> bool {
    platform::local_path_exists(path)
}
