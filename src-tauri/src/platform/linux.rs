use std::path::Path;
use std::process::Command;

use crate::model::App;

pub fn discover_games() -> Vec<App> {
    Vec::new()
}

pub fn prune_missing(apps: Vec<App>) -> Vec<App> {
    apps.into_iter()
        .filter(|a| Path::new(&a.path).exists())
        .collect()
}

pub fn local_path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn is_trackable(_path: &str) -> bool {
    true
}

pub fn wait_for_app(path: &str) {
    // On Linux the dropped path is expected to be the executable itself.
    if let Ok(mut child) = Command::new(path).spawn() {
        let _ = child.wait();
    }
}

pub fn launch_detached(path: &str) {
    let _ = Command::new(path).spawn();
}

pub fn handle_run_event(_app: &tauri::AppHandle, _event: &tauri::RunEvent) {}
