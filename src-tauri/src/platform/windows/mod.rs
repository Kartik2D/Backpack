use std::collections::HashSet;
use std::path::Path;

use crate::model::App;
use crate::store::normalize_path_key;

mod appx;
mod launch;
mod registry;
mod scan;

pub use launch::{is_trackable, launch_detached, local_path_exists, wait_for_app};
pub use scan::discover_games;

struct InstallIndex {
    steam_appids: HashSet<String>,
    epic_paths: HashSet<String>,
    aumids: HashSet<String>,
    ubisoft_ids: HashSet<String>,
}

fn build_install_index() -> InstallIndex {
    let steam_appids = scan::installed_steam_appids();
    let epic_paths = scan::scan_epic()
        .into_iter()
        .map(|d| normalize_path_key(&d.app.path))
        .collect();
    let aumids = appx::installed_aumids();
    let ubisoft_ids = registry::ubisoft_installs()
        .into_iter()
        .map(|install| install.id)
        .collect();

    InstallIndex {
        steam_appids,
        epic_paths,
        aumids,
        ubisoft_ids,
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

    if lower.starts_with("shell:appsfolder") {
        return index.aumids.contains(&normalize_path_key(path));
    }

    if let Some(rest) = lower.strip_prefix("uplay://launch/") {
        let id = rest.split('/').next().unwrap_or("");
        return index.ubisoft_ids.contains(id);
    }

    if lower.ends_with(".lnk") {
        if !Path::new(path).exists() {
            return false;
        }
        return match launch::resolve_lnk_target(path) {
            Some(target) if target.is_empty() => true,
            Some(target) => Path::new(&target).exists(),
            None => Path::new(path).exists(),
        };
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

pub fn handle_run_event(_app: &tauri::AppHandle, _event: &tauri::RunEvent) {}
