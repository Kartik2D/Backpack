use std::collections::HashSet;
use std::path::{Path, PathBuf};

use windows::core::HSTRING;
use windows::Management::Deployment::PackageManager;

use crate::model::App;
use crate::store::normalize_path_key;

fn package_install_dir(package: &windows::ApplicationModel::Package) -> Option<PathBuf> {
    let path = package.InstalledLocation().ok()?.Path().ok()?;
    let path = PathBuf::from(path.to_string());
    std::fs::canonicalize(&path).ok().or(Some(path))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn is_game_package(package: &windows::ApplicationModel::Package) -> bool {
    package_install_dir(package)
        .map(|dir| dir.join("MicrosoftGame.config").exists())
        .unwrap_or(false)
}

fn app_entries(package: &windows::ApplicationModel::Package) -> Vec<(String, String)> {
    let Ok(entries) = package.GetAppListEntriesAsync().and_then(|op| op.join()) else {
        return Vec::new();
    };
    let Ok(size) = entries.Size() else {
        return Vec::new();
    };

    (0..size)
        .filter_map(|index| {
            let entry = entries.GetAt(index).ok()?;
            let aumid = entry.AppUserModelId().ok()?.to_string();
            if aumid.is_empty() {
                return None;
            }
            let name = entry
                .DisplayInfo()
                .ok()
                .and_then(|display| display.DisplayName().ok())
                .map(|name| name.to_string())
                .filter(|name| !name.is_empty())
                .or_else(|| package.DisplayName().ok().map(|name| name.to_string()))
                .unwrap_or_else(|| aumid.clone());
            Some((name, aumid))
        })
        .collect()
}

fn package_entries() -> Vec<App> {
    let Ok(manager) = PackageManager::new() else {
        return Vec::new();
    };
    let packages = manager
        .FindPackagesByUserSecurityId(&HSTRING::from(""))
        .or_else(|_| manager.FindPackages());
    let Ok(packages) = packages else {
        return Vec::new();
    };

    let mut out = Vec::new();
    let Ok(iterator) = packages.First() else {
        return out;
    };

    while iterator.HasCurrent().unwrap_or(false) {
        if let Ok(package) = iterator.Current() {
            let is_framework = package.IsFramework().unwrap_or(false);
            if !is_framework && is_game_package(&package) {
                let install_dir = package_install_dir(&package).map(|path| path_string(&path));
                out.extend(app_entries(&package).into_iter().map(|(name, aumid)| {
                    App::with_name(
                        format!("shell:AppsFolder\\{aumid}"),
                        name,
                        install_dir.clone(),
                    )
                }));
            }
        }
        if !iterator.MoveNext().unwrap_or(false) {
            break;
        }
    }

    out
}

pub fn scan_gamepass() -> Vec<App> {
    package_entries()
}

pub fn installed_aumids() -> HashSet<String> {
    package_entries()
        .into_iter()
        .map(|app| normalize_path_key(&app.path))
        .collect()
}
