use std::collections::HashSet;

use tauri::Manager;

use crate::metadata::{apply_metadata_to_app, enrich_new, refresh_all, search_igdb};
use crate::model::{App, AppList, AppMetadata, IgdbSearchResult, MetadataCache};
use crate::platform;
use crate::store::{dedupe_apps, local_path_exists, normalize_path_key, save_apps};
use crate::window::show_window;

// Discover installed games across all supported launchers, drop entries the user
// uninstalled outside Backpack, dedupe, merge new finds, persist, and return.
fn scan_and_merge(app: &tauri::AppHandle) -> Vec<App> {
    let discovered = platform::discover_games();

    let state = app.state::<AppList>();
    let additions = {
        let mut list = state.0.lock().unwrap();
        *list = crate::store::clean_apps(list.clone());

        let mut seen: HashSet<String> = list.iter().map(|a| normalize_path_key(&a.path)).collect();
        let mut additions = Vec::new();
        for game in discovered {
            if seen.insert(normalize_path_key(&game.path)) {
                additions.push(game);
            }
        }
        additions
    };

    let cache = app.state::<MetadataCache>();
    let additions = enrich_new(additions, &cache);
    let mut list = state.0.lock().unwrap();
    list.extend(additions);
    let result = dedupe_apps(list.clone());
    *list = result.clone();
    drop(list);
    save_apps(app, &result);
    result
}

fn refresh_all_metadata(app: &tauri::AppHandle) -> Vec<App> {
    let state = app.state::<AppList>();
    let list = state.0.lock().unwrap().clone();
    let cache = app.state::<MetadataCache>();
    let result = refresh_all(dedupe_apps(platform::prune_missing(list)), &cache);
    *state.0.lock().unwrap() = result.clone();
    save_apps(app, &result);
    result
}

fn remove_app_from_list(path: &str, app: &tauri::AppHandle) -> Vec<App> {
    let key = normalize_path_key(path);
    let state = app.state::<AppList>();
    let mut list = state.0.lock().unwrap();
    list.retain(|item| normalize_path_key(&item.path) != key);
    let result = list.clone();
    drop(list);
    save_apps(app, &result);
    result
}

fn apply_selected_metadata(
    path: &str,
    name: String,
    image: String,
    description: String,
    app: &tauri::AppHandle,
) -> Vec<App> {
    let key = normalize_path_key(path);
    let state = app.state::<AppList>();
    let mut list = state.0.lock().unwrap();
    if let Some(item) = list
        .iter_mut()
        .find(|item| normalize_path_key(&item.path) == key)
    {
        apply_metadata_to_app(
            item,
            AppMetadata {
                name: Some(name),
                image: Some(image),
                description: Some(description),
            },
        );
    }
    let result = list.clone();
    drop(list);
    save_apps(app, &result);
    result
}

#[tauri::command]
pub async fn scan_games(app: tauri::AppHandle) -> Vec<App> {
    // Scanning touches launchers, registries and the filesystem; keep it off the
    // main thread so the UI stays responsive.
    tauri::async_runtime::spawn_blocking(move || scan_and_merge(&app))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn get_metadata(app: tauri::AppHandle) -> Vec<App> {
    tauri::async_runtime::spawn_blocking(move || refresh_all_metadata(&app))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn igdb_search(query: String) -> Vec<IgdbSearchResult> {
    tauri::async_runtime::spawn_blocking(move || search_igdb(&query))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_apps(list: tauri::State<AppList>) -> Vec<App> {
    list.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn remove_app(path: String, app: tauri::AppHandle) -> Vec<App> {
    remove_app_from_list(&path, &app)
}

#[tauri::command]
pub fn apply_metadata(
    path: String,
    name: String,
    image: String,
    description: String,
    app: tauri::AppHandle,
) -> Vec<App> {
    apply_selected_metadata(&path, name, image, description, &app)
}

#[tauri::command]
pub fn add_apps(paths: Vec<String>, app: tauri::AppHandle) -> Vec<App> {
    let state = app.state::<AppList>();
    let existing = state.0.lock().unwrap().clone();
    let mut seen: HashSet<String> = existing
        .iter()
        .map(|a| normalize_path_key(&a.path))
        .collect();
    let mut additions = Vec::new();

    for path in paths {
        let key = normalize_path_key(&path);
        if seen.contains(&key) || !local_path_exists(&path) {
            continue;
        }
        seen.insert(key);
        additions.push(App {
            name: platform::file_stem(&path),
            path,
            image: String::new(),
            description: String::new(),
        });
    }

    let cache = app.state::<MetadataCache>();
    let additions = enrich_new(additions, &cache);
    let mut list = state.0.lock().unwrap();
    list.extend(additions);
    let result = dedupe_apps(list.clone());
    *list = result.clone();
    drop(list);
    save_apps(&app, &result);
    result
}

#[tauri::command]
pub fn launch(path: String, window: tauri::WebviewWindow, app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // Protocol and packaged launches hand off to a launcher process, so we
        // can't wait for the game itself to exit.
        if !platform::is_trackable(&path) {
            platform::launch_detached(&path);
            return;
        }

        let win = window.clone();
        let _ = app.run_on_main_thread(move || {
            let _ = win.close();
        });
        platform::wait_for_app(&path);
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || show_window(&handle));
    });
}
