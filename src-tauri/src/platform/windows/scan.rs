use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::model::{App, Discovered};
use crate::store::dedupe_apps;

use super::appx;
use super::registry;
use crate::platform::{file_stem, folder_name, vdf_values};

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn installed_steam_appids() -> HashSet<String> {
    let mut steam_appids = HashSet::new();
    if let Some(steam) = registry::steam_path() {
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
    steam_appids
}

fn scan_steam() -> Vec<App> {
    let Some(steam) = registry::steam_path() else {
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
            let install_dir = vdf_values(&acf, "installdir")
                .into_iter()
                .next()
                .map(|dir| path_string(&steamapps.join("common").join(dir)));
            games.push(App::with_name(
                format!("steam://rungameid/{appid}"),
                name,
                install_dir,
            ));
        }
    }
    games
}

// Walk a directory (bounded depth and file budget) collecting executables.
fn collect_exes(dir: &Path, depth: u32, budget: &mut u32, out: &mut Vec<(PathBuf, u64)>) {
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
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => collect_exes(&path, depth - 1, budget, out),
            Ok(ft) if ft.is_file() => {
                let is_exe = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("exe"))
                    .unwrap_or(false);
                if is_exe {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push((path, size));
                }
            }
            _ => {}
        }
    }
}

// Best-effort guess of a game's main executable: the largest .exe that doesn't
// look like an installer/redistributable/helper.
fn find_game_exe(dir: &Path) -> Option<PathBuf> {
    const SKIP: [&str; 18] = [
        "unins",
        "setup",
        "vcredist",
        "vc_redist",
        "dxsetup",
        "dotnet",
        "redist",
        "crashpad",
        "crashreport",
        "crashhandler",
        "easyanticheat",
        "battleye",
        "be_service",
        "touchup",
        "cleanup",
        "prereq",
        "oalinst",
        "dxwebsetup",
    ];

    let mut exes = Vec::new();
    let mut budget = 4000u32;
    collect_exes(dir, 3, &mut budget, &mut exes);
    exes.into_iter()
        .filter(|(p, _)| {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            !SKIP.iter().any(|s| name.contains(s))
        })
        .max_by_key(|(_, size)| *size)
        .map(|(p, _)| p)
}

// Epic Games: JSON manifests under ProgramData. Launch via the
// com.epicgames.launcher:// protocol.
fn epic_manifest_dir() -> PathBuf {
    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
    Path::new(&program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests")
}

pub fn scan_epic() -> Vec<Discovered> {
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
        let install_dir = get("InstallLocation");
        out.push(Discovered {
            app: App::with_name(
                format!(
                    "com.epicgames.launcher://apps/{namespace}%3A{item_id}%3A{app_name}?action=launch&silent=true"
                ),
                name,
                (!install_dir.is_empty()).then_some(install_dir),
            ),
        });
    }
    out
}

// GOG: DRM-free, launch the executable directly (lifetime is trackable).
fn scan_gog() -> Vec<Discovered> {
    registry::gog_installs()
        .into_iter()
        .filter_map(|item| {
            if item.exe.is_empty() {
                return None;
            }
            let name = (!item.name.is_empty())
                .then_some(item.name)
                .unwrap_or_else(|| file_stem(&item.exe));
            let install_dir = Path::new(&item.exe).parent().map(path_string);
            Some(Discovered {
                app: App::with_name(item.exe, name, install_dir),
            })
        })
        .collect()
}

// Ubisoft Connect: launch via uplay:// using the install id.
fn scan_ubisoft() -> Vec<Discovered> {
    registry::ubisoft_installs()
        .into_iter()
        .map(|item| Discovered {
            app: App::with_name(
                format!("uplay://launch/{}/0", item.id),
                folder_name(&item.dir),
                Some(item.dir),
            ),
        })
        .collect()
}

// EA app / Origin: registry gives the install dir; launch the main exe directly.
fn scan_ea() -> Vec<Discovered> {
    registry::ea_installs()
        .into_iter()
        .filter_map(|item| {
            let exe = find_game_exe(Path::new(&item.dir))?
                .to_string_lossy()
                .into_owned();
            let name = if item.name.is_empty() {
                folder_name(&item.dir)
            } else {
                item.name
            };
            Some(Discovered {
                app: App::with_name(exe, name, Some(item.dir)),
            })
        })
        .collect()
}

// Battle.net (Blizzard): detected via uninstall entries; launch the main exe.
fn scan_blizzard() -> Vec<Discovered> {
    registry::blizzard_installs()
        .into_iter()
        .filter_map(|item| {
            let exe = find_game_exe(Path::new(&item.dir))?
                .to_string_lossy()
                .into_owned();
            Some(Discovered {
                app: App::with_name(exe, item.name, Some(item.dir)),
            })
        })
        .collect()
}

// itch.io: installs live under %APPDATA%\itch\apps\<slug>; launch the exe.
fn scan_itch() -> Vec<Discovered> {
    let Ok(appdata) = std::env::var("APPDATA") else {
        return Vec::new();
    };
    let dir = Path::new(&appdata).join("itch").join("apps");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let folder = entry.path();
        if !folder.is_dir() {
            continue;
        }
        let Some(exe) = find_game_exe(&folder).map(|p| p.to_string_lossy().into_owned()) else {
            continue;
        };
        out.push(Discovered {
            app: App::with_name(
                exe,
                folder_name(&folder.to_string_lossy()),
                Some(path_string(&folder)),
            ),
        });
    }
    out
}

// Amazon Games: default library; each game has a fuel.json describing its exe.
fn scan_amazon() -> Vec<Discovered> {
    let library = PathBuf::from("C:\\Amazon Games\\Library");
    let Ok(entries) = std::fs::read_dir(&library) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let folder = entry.path();
        if !folder.is_dir() {
            continue;
        }
        // fuel.json names the launch command; fall back to a heuristic exe.
        let exe = std::fs::read_to_string(folder.join("fuel.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| {
                v.get("Main")
                    .and_then(|m| m.get("Command"))
                    .and_then(|c| c.as_str())
                    .map(|c| folder.join(c))
            })
            .filter(|p| p.exists())
            .or_else(|| find_game_exe(&folder));

        let Some(exe) = exe.map(|p| p.to_string_lossy().into_owned()) else {
            continue;
        };
        out.push(Discovered {
            app: App::with_name(
                exe,
                folder_name(&folder.to_string_lossy()),
                Some(path_string(&folder)),
            ),
        });
    }
    out
}

pub fn discover_games() -> Vec<App> {
    let mut pending: Vec<Discovered> = Vec::new();
    pending.extend(scan_steam().into_iter().map(|app| Discovered { app }));
    pending.extend(
        appx::scan_gamepass()
            .into_iter()
            .map(|app| Discovered { app }),
    );
    pending.extend(scan_epic());
    pending.extend(scan_gog());
    pending.extend(scan_ubisoft());
    pending.extend(scan_ea());
    pending.extend(scan_blizzard());
    pending.extend(scan_itch());
    pending.extend(scan_amazon());

    dedupe_apps(pending.into_iter().map(|d| d.app).collect())
}
