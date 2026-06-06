use std::collections::HashSet;
use std::path::Path;

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::metadata::{
    apply_metadata_to_app, enrich_missing, enrich_new, refresh_all, search_igdb,
};
use crate::model::{App, AppList, AppMetadata, IgdbSearchResult};
use crate::platform;
use crate::store::{dedupe_apps, local_path_exists, normalize_path_key, save_apps};
use crate::track::{self, GameState, GameStates, TrackTarget};

// Progress payload emitted to the UI so a single toast can report what the
// long-running scan/metadata flow is currently doing.
#[derive(Clone, Serialize)]
struct ScanProgress {
    message: String,
}

fn report_progress(app: &tauri::AppHandle, message: &str) {
    let _ = app.emit(
        "scan-progress",
        ScanProgress {
            message: message.to_string(),
        },
    );
}

// Result of a scan / metadata refresh, returned to the UI so the success toast
// can report what happened: games newly added plus IGDB requests made and items
// (fields) downloaded.
#[derive(Clone, Serialize)]
pub struct ScanReport {
    apps: Vec<App>,
    added: usize,
    requests: usize,
    items: usize,
}

// Discover installed games across all supported launchers, drop entries the user
// uninstalled outside Backpack, dedupe, merge new finds, persist, and return.
fn scan_and_merge(app: &tauri::AppHandle) -> Result<ScanReport, String> {
    let report = |message: &str| report_progress(app, message);

    report("Checking installed launchers…");
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
    let added = additions.len();

    let merged = {
        let mut list = state.0.lock().unwrap();
        list.extend(additions);
        dedupe_apps(list.clone())
    };
    report(&format!("Found {} games. Fetching metadata…", merged.len()));
    let (result, stats) = enrich_missing(merged, &report)?;
    report("Saving library…");
    *state.0.lock().unwrap() = result.clone();
    save_apps(app, &result);
    Ok(ScanReport {
        apps: result,
        added,
        requests: stats.requests,
        items: stats.items,
    })
}

fn refresh_all_metadata(app: &tauri::AppHandle) -> Result<ScanReport, String> {
    let report = |message: &str| report_progress(app, message);

    let state = app.state::<AppList>();
    let list = state.0.lock().unwrap().clone();
    report("Refreshing metadata…");
    let (result, stats) = refresh_all(dedupe_apps(platform::prune_missing(list)), &report)?;
    report("Saving library…");
    *state.0.lock().unwrap() = result.clone();
    save_apps(app, &result);
    Ok(ScanReport {
        apps: result,
        added: 0,
        requests: stats.requests,
        items: stats.items,
    })
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
    key_art: String,
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
                key_art: Some(key_art),
                description: Some(description),
            },
        );
    }
    let result = list.clone();
    drop(list);
    save_apps(app, &result);
    result
}

fn install_dir_for_local_path(path: &str) -> Option<String> {
    let lower = path.to_lowercase();
    if lower.contains("://") || lower.starts_with("shell:") {
        return None;
    }

    let path = Path::new(path);
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("app"))
        .unwrap_or(false)
    {
        return Some(path.to_string_lossy().into_owned());
    }

    path.parent()
        .map(|parent| parent.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn scan_games(app: tauri::AppHandle) -> Result<ScanReport, String> {
    // Scanning touches launchers, registries and the filesystem; keep it off the
    // main thread so the UI stays responsive.
    tauri::async_runtime::spawn_blocking(move || scan_and_merge(&app))
        .await
        .map_err(|err| format!("Scan task failed: {err}"))?
}

#[tauri::command]
pub async fn get_metadata(app: tauri::AppHandle) -> Result<ScanReport, String> {
    tauri::async_runtime::spawn_blocking(move || refresh_all_metadata(&app))
        .await
        .map_err(|err| format!("Metadata task failed: {err}"))?
}

#[tauri::command]
pub async fn igdb_search(query: String) -> Result<Vec<IgdbSearchResult>, String> {
    tauri::async_runtime::spawn_blocking(move || search_igdb(&query))
        .await
        .map_err(|err| format!("IGDB search task failed: {err}"))?
}

#[tauri::command]
pub fn get_apps(list: tauri::State<AppList>) -> Vec<App> {
    list.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_game_states(states: tauri::State<GameStates>) -> Vec<GameState> {
    track::snapshot(&states)
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
    key_art: String,
    description: String,
    app: tauri::AppHandle,
) -> Vec<App> {
    apply_selected_metadata(&path, name, image, key_art, description, &app)
}

#[tauri::command]
pub fn add_apps(paths: Vec<String>, app: tauri::AppHandle) -> Result<Vec<App>, String> {
    let state = app.state::<AppList>();
    let existing = state.0.lock().unwrap().clone();
    let mut seen: HashSet<String> = existing
        .iter()
        .map(|a| normalize_path_key(&platform::normalize_launch_path(&a.path)))
        .collect();
    let mut additions = Vec::new();

    for path in paths {
        if !local_path_exists(&path) {
            continue;
        }
        let canonical_path = platform::normalize_launch_path(&path);
        let key = normalize_path_key(&canonical_path);
        if seen.contains(&key) {
            continue;
        }
        seen.insert(key);
        additions.push(App::with_name(
            canonical_path,
            platform::file_stem(&path),
            install_dir_for_local_path(&path),
        ));
    }

    let (additions, _stats) = enrich_new(additions, &|_: &str| {})?;
    let mut list = state.0.lock().unwrap();
    list.extend(additions);
    let result = dedupe_apps(list.clone());
    *list = result.clone();
    drop(list);
    save_apps(&app, &result);
    Ok(result)
}

#[tauri::command]
pub fn launch(path: String, window: tauri::WebviewWindow, app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let path = platform::normalize_launch_path(&path);
        let game = {
            let key = normalize_path_key(&path);
            let found = {
                let state = app.state::<AppList>();
                let list = state.0.lock().unwrap();
                list.iter()
                    .find(|item| {
                        normalize_path_key(&platform::normalize_launch_path(&item.path)) == key
                            || normalize_path_key(&item.path) == key
                    })
                    .cloned()
            };
            found.unwrap_or_else(|| {
                App::with_name(
                    path.clone(),
                    platform::file_stem(&path),
                    install_dir_for_local_path(&path),
                )
            })
        };
        let target = TrackTarget::from_app(&game, &path);

        track::emit_launching(&app, path.clone());
        platform::launch_detached(&path);
        track::spawn(app, window, path, target);
    });
}
