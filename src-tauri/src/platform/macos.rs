use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::{App, Discovered};
use crate::store::{dedupe_apps, normalize_path_key};
use crate::window::show_window;

use super::{folder_name, vdf_values};

struct InstallIndex {
    steam_appids: HashSet<String>,
    epic_paths: HashSet<String>,
}

fn build_install_index() -> InstallIndex {
    let mut steam_appids = HashSet::new();
    if let Some(steam) = steam_path() {
        let library_file = steam.join("steamapps").join("libraryfolders.vdf");
        if let Ok(content) = std::fs::read_to_string(&library_file) {
            let mut libraries = vec![steam];
            libraries.extend(vdf_values(&content, "path").into_iter().map(PathBuf::from));
            for library in libraries {
                let steamapps = library.join("steamapps");
                let Ok(entries) = std::fs::read_dir(&steamapps) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let Some(name) = name.to_str() else {
                        continue;
                    };
                    if let Some(appid) = name
                        .strip_prefix("appmanifest_")
                        .and_then(|s| s.strip_suffix(".acf"))
                    {
                        steam_appids.insert(appid.to_string());
                    }
                }
            }
        }
    }

    let epic_paths = scan_epic()
        .into_iter()
        .map(|d| normalize_path_key(&d.app.path))
        .collect();

    InstallIndex {
        steam_appids,
        epic_paths,
    }
}

fn app_still_exists(path: &str, index: &InstallIndex) -> bool {
    let lower = path.to_lowercase();

    if let Some(appid) = lower.strip_prefix("steam://rungameid/") {
        let appid = appid.trim_end_matches('/');
        return index.steam_appids.contains(appid);
    }

    if lower.starts_with("com.epicgames.launcher://") {
        return index.epic_paths.contains(&normalize_path_key(path));
    }

    // Unknown custom protocols are kept to avoid false removals.
    if lower.contains("://") {
        return true;
    }

    Path::new(path).exists()
}

pub fn prune_missing(apps: Vec<App>) -> Vec<App> {
    let index = build_install_index();
    apps.into_iter()
        .filter(|a| app_still_exists(&a.path, &index))
        .collect()
}

pub fn local_path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn wait_for_app(path: &str) {
    // `.app` bundles aren't directly executable; `open -W` launches and waits.
    let _ = Command::new("open").arg("-W").arg(path).status();
}

pub fn launch_detached(path: &str) {
    let _ = Command::new("open").arg(path).spawn();
}

pub fn is_trackable(path: &str) -> bool {
    !path.to_lowercase().contains("://")
}

pub fn handle_run_event(app: &tauri::AppHandle, event: &tauri::RunEvent) {
    if let tauri::RunEvent::Reopen { .. } = event {
        show_window(app);
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn steam_path() -> Option<PathBuf> {
    home_dir().map(|home| {
        home.join("Library")
            .join("Application Support")
            .join("Steam")
    })
}

fn scan_steam() -> Vec<App> {
    let Some(steam) = steam_path() else {
        return Vec::new();
    };

    let library_file = steam.join("steamapps").join("libraryfolders.vdf");
    let Ok(content) = std::fs::read_to_string(&library_file) else {
        return Vec::new();
    };

    // Always include the base install; libraryfolders.vdf covers extra drives.
    let mut libraries = vec![steam.clone()];
    libraries.extend(vdf_values(&content, "path").into_iter().map(PathBuf::from));

    let mut games = Vec::new();
    let mut seen = HashSet::new();
    for library in libraries {
        let steamapps = library.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("appmanifest_") && n.ends_with(".acf"))
                .unwrap_or(false);
            if !is_manifest {
                continue;
            }
            let Ok(acf) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Some(appid) = vdf_values(&acf, "appid").into_iter().next() else {
                continue;
            };
            // Skip Steamworks redistributables and other non-game tools.
            if appid == "228980" || !seen.insert(appid.clone()) {
                continue;
            }
            let name = vdf_values(&acf, "name")
                .into_iter()
                .next()
                .unwrap_or_else(|| format!("Steam app {appid}"));
            games.push(App {
                path: format!("steam://rungameid/{appid}"),
                name,
                image: String::new(),
                description: String::new(),
            });
        }
    }
    games
}

fn epic_manifest_dir() -> PathBuf {
    PathBuf::from("/Users/Shared")
        .join("Epic Games")
        .join("Launcher")
        .join("Data")
        .join("Manifests")
}

fn scan_epic() -> Vec<Discovered> {
    let dir = epic_manifest_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let is_item = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("item"))
            .unwrap_or(false);
        if !is_item {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let get = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();

        let is_app = v
            .get("bIsApplication")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let is_game = v
            .get("AppCategories")
            .and_then(|x| x.as_array())
            .map(|a| a.iter().any(|c| c.as_str() == Some("games")))
            .unwrap_or(false);
        if !is_app || !is_game {
            continue;
        }

        let namespace = get("CatalogNamespace");
        let item_id = get("CatalogItemId");
        let app_name = get("AppName");
        if namespace.is_empty() || item_id.is_empty() || app_name.is_empty() {
            continue;
        }
        let name = get("DisplayName");
        out.push(Discovered {
            app: App {
                path: format!(
                    "com.epicgames.launcher://apps/{namespace}%3A{item_id}%3A{app_name}?action=launch&silent=true"
                ),
                name,
                image: String::new(),
                description: String::new(),
            },
        });
    }
    out
}

fn plist_raw_value(info_plist: &Path, key: &str) -> Option<String> {
    let output = Command::new("plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(info_plist)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn app_bundle_name(path: &Path) -> String {
    let info = path.join("Contents").join("Info.plist");
    plist_raw_value(&info, "CFBundleDisplayName")
        .or_else(|| plist_raw_value(&info, "CFBundleName"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        })
}

fn is_game_app_bundle(path: &Path) -> bool {
    let info = path.join("Contents").join("Info.plist");
    plist_raw_value(&info, "LSApplicationCategoryType").as_deref()
        == Some("public.app-category.games")
}

fn collect_app_bundles(dir: &Path, depth: u32, budget: &mut u32, out: &mut Vec<PathBuf>) {
    if depth == 0 || *budget == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *budget == 0 {
            return;
        }
        *budget -= 1;
        let path = entry.path();
        let is_app = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("app"))
            .unwrap_or(false);
        if is_app {
            out.push(path);
            continue;
        }
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            collect_app_bundles(&path, depth - 1, budget, out);
        }
    }
}

fn app_to_discovered(path: PathBuf) -> Discovered {
    let name = app_bundle_name(&path);
    Discovered {
        app: App {
            path: path.to_string_lossy().into_owned(),
            name,
            image: String::new(),
            description: String::new(),
        },
    }
}

// itch.io: macOS installs live under ~/Library/Application Support/itch/apps.
fn scan_itch() -> Vec<Discovered> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };
    let dir = home
        .join("Library")
        .join("Application Support")
        .join("itch")
        .join("apps");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let folder = entry.path();
        if !folder.is_dir() {
            continue;
        }
        let mut bundles = Vec::new();
        let mut budget = 800u32;
        collect_app_bundles(&folder, 4, &mut budget, &mut bundles);
        out.extend(bundles.into_iter().map(app_to_discovered));
    }
    out
}

fn scan_macos_applications() -> Vec<Discovered> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(home) = home_dir() {
        roots.push(home.join("Applications"));
    }

    let mut out = Vec::new();
    for root in roots {
        let mut bundles = Vec::new();
        let mut budget = 1200u32;
        collect_app_bundles(&root, 2, &mut budget, &mut bundles);
        out.extend(
            bundles
                .into_iter()
                .filter(|path| is_game_app_bundle(path))
                .map(app_to_discovered),
        );
    }
    out
}

pub fn discover_games() -> Vec<App> {
    let mut pending: Vec<Discovered> = Vec::new();
    pending.extend(scan_steam().into_iter().map(|app| Discovered { app }));
    pending.extend(scan_epic());
    pending.extend(scan_itch());
    pending.extend(scan_macos_applications());

    dedupe_apps(pending.into_iter().map(|d| d.app).collect())
}
