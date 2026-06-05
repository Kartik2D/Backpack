use std::collections::HashSet;
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

// Case-insensitive path key for duplicate detection on Windows.
fn normalize_path_key(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

fn dedupe_apps(mut apps: Vec<App>) -> Vec<App> {
    let mut seen = HashSet::new();
    apps.retain(|a| seen.insert(normalize_path_key(&a.path)));
    apps
}

#[cfg(target_os = "windows")]
struct InstallIndex {
    steam_appids: HashSet<String>,
    epic_paths: HashSet<String>,
    aumids: HashSet<String>,
    ubisoft_ids: HashSet<String>,
}

#[cfg(target_os = "windows")]
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

    let aumids = installed_aumids();

    let mut ubisoft_ids = HashSet::new();
    for item in registry_array(&scan_registry(), "ubisoft") {
        if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
            ubisoft_ids.insert(id.to_string());
        }
    }

    InstallIndex {
        steam_appids,
        epic_paths,
        aumids,
        ubisoft_ids,
    }
}

#[cfg(target_os = "windows")]
fn installed_aumids() -> HashSet<String> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
Get-StartApps | ForEach-Object { 'shell:appsfolder\' + $_.AppID.ToLower() }
"#;

    let output = match windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return HashSet::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(normalize_path_key)
        .collect()
}

#[cfg(target_os = "windows")]
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
        return match resolve_lnk_target(path) {
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

#[cfg(target_os = "windows")]
fn prune_missing(apps: Vec<App>) -> Vec<App> {
    let index = build_install_index();
    apps.into_iter()
        .filter(|a| app_still_exists(&a.path, &index))
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn prune_missing(apps: Vec<App>) -> Vec<App> {
    apps.into_iter()
        .filter(|a| Path::new(&a.path).exists())
        .collect()
}

fn clean_apps(apps: Vec<App>) -> Vec<App> {
    dedupe_apps(prune_missing(apps))
}

// Existence check for manually dropped local paths (.exe, .app, .lnk).
fn local_path_exists(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    #[cfg(target_os = "windows")]
    if path.to_lowercase().ends_with(".lnk") {
        return match resolve_lnk_target(path) {
            Some(target) if target.is_empty() => true,
            Some(target) => Path::new(&target).exists(),
            None => true,
        };
    }
    true
}

#[derive(Deserialize)]
struct IgdbToken {
    access_token: String,
}

#[derive(Deserialize)]
struct IgdbCover {
    image_id: Option<String>,
}

#[derive(Deserialize)]
struct IgdbGame {
    id: Option<i64>,
    name: Option<String>,
    summary: Option<String>,
    cover: Option<IgdbCover>,
}

#[derive(Clone, Serialize)]
struct IgdbSearchResult {
    id: i64,
    name: String,
    image: String,
    description: String,
}

struct IgdbClient {
    client_id: String,
    access_token: String,
    http: reqwest::blocking::Client,
}

impl IgdbClient {
    fn from_env() -> Option<Self> {
        let client_id = std::env::var("IGDB_CLIENT_ID")
            .or_else(|_| std::env::var("TWITCH_CLIENT_ID"))
            .ok()?;
        let http = reqwest::blocking::Client::new();
        let access_token = std::env::var("IGDB_ACCESS_TOKEN")
            .or_else(|_| std::env::var("TWITCH_ACCESS_TOKEN"))
            .ok()
            .or_else(|| {
                let client_secret = std::env::var("IGDB_CLIENT_SECRET")
                    .or_else(|_| std::env::var("TWITCH_CLIENT_SECRET"))
                    .ok()?;
                http.post("https://id.twitch.tv/oauth2/token")
                    .query(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("grant_type", "client_credentials"),
                    ])
                    .send()
                    .ok()?
                    .error_for_status()
                    .ok()?
                    .json::<IgdbToken>()
                    .ok()
                    .map(|t| t.access_token)
            })?;

        Some(Self {
            client_id,
            access_token,
            http,
        })
    }

    fn lookup_game(&self, name: &str) -> Option<AppMetadata> {
        self.search_games(name).into_iter().next().map(|result| {
            let image = if result.image.is_empty() {
                None
            } else {
                Some(result.image.replace("t_cover_small", "t_cover_big"))
            };
            AppMetadata {
                name: Some(result.name),
                image,
                description: (!result.description.is_empty()).then_some(result.description),
            }
        })
    }

    fn search_games(&self, name: &str) -> Vec<IgdbSearchResult> {
        let search = igdb_search_name(name);
        if search.is_empty() {
            return Vec::new();
        }
        let query = format!(
            "search \"{}\"; fields name,summary,cover.image_id; limit 12;",
            escape_igdb_string(&search)
        );
        self
            .http
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .body(query)
            .send()
            .ok()
            .and_then(|r| r.error_for_status().ok())
            .and_then(|r| r.json::<Vec<IgdbGame>>().ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|game| {
                let id = game.id?;
                let name = game.name?;
                Some(IgdbSearchResult {
                    id,
                    name,
                    image: game
                        .cover
                        .and_then(|cover| cover.image_id)
                        .map(|id| igdb_cover_url(&id, "cover_small"))
                        .unwrap_or_default(),
                    description: game.summary.unwrap_or_default(),
                })
            })
            .collect()
    }
}

fn search_igdb(query: &str) -> Vec<IgdbSearchResult> {
    IgdbClient::from_env()
        .map(|igdb| igdb.search_games(query))
        .unwrap_or_default()
}

fn igdb_cover_url(image_id: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/t_{size}/{image_id}.jpg")
}

fn apply_metadata_to_app(app: &mut App, metadata: AppMetadata) {
    if let Some(name) = metadata.name.filter(|s| !s.is_empty()) {
        app.name = name;
    }
    if let Some(image) = metadata.image.filter(|s| !s.is_empty()) {
        app.image = image.replace("t_cover_small", "t_cover_big");
    }
    if let Some(description) = metadata.description.filter(|s| !s.is_empty()) {
        app.description = description;
    }
}

struct AppMetadata {
    name: Option<String>,
    image: Option<String>,
    description: Option<String>,
}

fn escape_igdb_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn igdb_search_name(name: &str) -> String {
    name.replace(['_', '-'], " ")
        .replace('™', "")
        .replace('®', "")
        .trim()
        .to_string()
}

fn enrich_with_igdb(apps: Vec<App>) -> Vec<App> {
    let Some(igdb) = IgdbClient::from_env() else {
        return apps;
    };

    apps.into_iter()
        .map(|mut app| {
            if let Some(metadata) = igdb.lookup_game(&app.name) {
                if let Some(name) = metadata.name.filter(|s| !s.is_empty()) {
                    app.name = name;
                }
                if let Some(image) = metadata.image.filter(|s| !s.is_empty()) {
                    app.image = image;
                }
                if let Some(description) = metadata.description.filter(|s| !s.is_empty()) {
                    app.description = description;
                }
            }
            app
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn windows_command(program: &str) -> Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
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

#[cfg(target_os = "windows")]
fn wait_for_app(path: &str) {
    // The path is passed via an env var (not $args): with `-Command`, trailing
    // arguments are appended to the command string rather than exposed as $args.
    // Start-Process resolves both .exe and .lnk shortcuts; -Wait blocks until exit.
    let _ = windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Start-Process -FilePath $env:BACKPACK_LAUNCH_PATH -Wait",
        ])
        .env("BACKPACK_LAUNCH_PATH", path)
        .status();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn wait_for_app(path: &str) {
    // On Linux the dropped path is expected to be the executable itself.
    if let Ok(mut child) = Command::new(path).spawn() {
        let _ = child.wait();
    }
}

// Fire-and-forget launch used when we can't track the app's lifetime.
#[cfg(target_os = "windows")]
fn launch_detached(path: &str) {
    // Packaged apps (shell:AppsFolder\<AUMID>) activate most reliably through
    // explorer.exe; protocols and regular files go through Start-Process.
    if path.to_lowercase().starts_with("shell:") {
        let _ = windows_command("explorer.exe").arg(path).spawn();
        return;
    }
    let _ = windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Start-Process -FilePath $env:BACKPACK_LAUNCH_PATH",
        ])
        .env("BACKPACK_LAUNCH_PATH", path)
        .spawn();
}

// Resolve a .lnk shortcut's TargetPath. Returns Some("") for Store/UWP
// shortcuts, which point at an AppUserModelID via a shell ID list and have no
// file target.
#[cfg(target_os = "windows")]
fn resolve_lnk_target(path: &str) -> Option<String> {
    let script = r#"
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($env:BACKPACK_LNK_PATH)
$shortcut.TargetPath
"#;

    let output = windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("BACKPACK_LNK_PATH", path)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Whether we can reliably wait for the app to exit. UWP/Store/Game Pass apps
// run inside a container; their launcher returns immediately, so `-Wait` is
// meaningless and the window would reappear instantly. In that case we'd rather
// leave the window open than close it and never get a reopen signal.
#[cfg(target_os = "windows")]
fn is_trackable(path: &str) -> bool {
    let lower = path.to_lowercase();
    // Packaged apps (WindowsApps / AppsFolder AUMIDs) and protocol launches
    // (e.g. steam://rungameid/...) hand off to another process and exit, so we
    // can't wait on them.
    if lower.contains("windowsapps") || lower.contains("shell:appsfolder") || lower.contains("://") {
        return false;
    }
    if lower.ends_with(".lnk") {
        match resolve_lnk_target(path) {
            // Empty target => Store/UWP shortcut (AUMID, no file). Target under
            // WindowsApps => packaged app. Neither is waitable.
            Some(target) => {
                let t = target.to_lowercase();
                if target.is_empty() || t.contains("windowsapps") {
                    return false;
                }
            }
            // Couldn't resolve: assume not trackable so we don't close the window.
            None => return false,
        }
    }
    true
}

// Extract single-line `"key" "value"` entries from a Valve VDF/ACF file.
#[cfg(target_os = "windows")]
fn vdf_values(content: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\"");
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(&needle) else {
            continue;
        };
        // The remainder looks like:  \t\t"value"
        if let Some(start) = rest.find('"') {
            if let Some(end) = rest[start + 1..].find('"') {
                // VDF escapes backslashes as \\.
                out.push(rest[start + 1..start + 1 + end].replace("\\\\", "\\"));
            }
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn steam_path() -> Option<PathBuf> {
    let output = windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-ItemProperty 'HKCU:\\Software\\Valve\\Steam' -ErrorAction SilentlyContinue).SteamPath",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn scan_gamepass() -> Vec<App> {
    // Enumerate installed Store packages, keep only those that are Xbox games
    // (identified by a MicrosoftGame.config in their real install dir), and emit
    // their launch AUMID. IGDB fills image/description later.
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$starts = @{}
Get-StartApps | ForEach-Object { $starts[$_.AppID] = $_.Name }
$out = @()
foreach ($p in Get-AppxPackage) {
    if ($p.IsFramework -or $p.NonRemovable -or $p.SignatureKind -ne 'Store') { continue }
    $loc = $p.InstallLocation
    if (-not $loc) { continue }
    $item = Get-Item $loc
    $real = if ($item.LinkType -eq 'Junction') { @($item.Target)[0] } else { $loc }
    if (-not (Test-Path (Join-Path $real 'MicrosoftGame.config'))) { continue }
    $appId = @((Get-AppxPackageManifest $p).Package.Applications.Application)[0].Id
    if (-not $appId) { continue }
    $aumid = $p.PackageFamilyName + '!' + $appId
    $name = $starts[$aumid]
    if (-not $name) { $name = $p.Name }
    $out += [pscustomobject]@{ name = $name; path = ('shell:AppsFolder\' + $aumid) }
}
if ($out.Count -eq 0) { '[]' } else { ConvertTo-Json -Compress -InputObject @($out) }
"#;

    let output = match windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = match serde_json::from_str(stdout.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    // A single result may deserialize as an object rather than an array.
    let items = match value {
        serde_json::Value::Array(a) => a,
        other @ serde_json::Value::Object(_) => vec![other],
        _ => Vec::new(),
    };

    items
        .into_iter()
        .filter_map(|item| {
            let path = item.get("path")?.as_str()?.to_string();
            if path.is_empty() {
                return None;
            }
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(App {
                path,
                name,
                image: String::new(),
                description: String::new(),
            })
        })
        .collect()
}

// A discovered game before the IGDB metadata pass.
#[cfg(target_os = "windows")]
struct Discovered {
    app: App,
}

// Walk a directory (bounded depth and file budget) collecting executables.
#[cfg(target_os = "windows")]
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
#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn folder_name(dir: &str) -> String {
    Path::new(dir)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(dir)
        .to_string()
}

// Epic Games: JSON manifests under ProgramData. Launch via the
// com.epicgames.launcher:// protocol.
#[cfg(target_os = "windows")]
fn scan_epic() -> Vec<Discovered> {
    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
    let dir = Path::new(&program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");
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

// Read registry-based launchers (GOG, Ubisoft, EA, Battle.net) in one shot.
#[cfg(target_os = "windows")]
fn scan_registry() -> serde_json::Value {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
function Subkeys($path) { if (Test-Path $path) { Get-ChildItem $path } }
$result = [ordered]@{}

$gog = @()
foreach ($k in Subkeys 'HKLM:\SOFTWARE\WOW6432Node\GOG.com\Games') {
    $p = Get-ItemProperty $k.PSPath
    if ($p.exe) { $gog += [pscustomobject]@{ name = $p.gameName; exe = $p.exe } }
}
$result.gog = $gog

$ubisoft = @()
foreach ($k in Subkeys 'HKLM:\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs') {
    $p = Get-ItemProperty $k.PSPath
    if ($p.InstallDir) { $ubisoft += [pscustomobject]@{ id = $k.PSChildName; dir = $p.InstallDir } }
}
$result.ubisoft = $ubisoft

$ea = @()
foreach ($base in 'HKLM:\SOFTWARE\WOW6432Node\EA Games', 'HKLM:\SOFTWARE\WOW6432Node\Origin Games') {
    foreach ($k in Subkeys $base) {
        $p = Get-ItemProperty $k.PSPath
        $dir = $p.'Install Dir'
        if (-not $dir) { $dir = $p.InstallDir }
        $name = $p.DisplayName
        if (-not $name) { $name = $k.PSChildName }
        if ($dir) { $ea += [pscustomobject]@{ name = $name; dir = $dir } }
    }
}
$result.ea = $ea

$blizzard = @()
foreach ($base in 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall', 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall') {
    foreach ($k in Subkeys $base) {
        $p = Get-ItemProperty $k.PSPath
        if ($p.Publisher -eq 'Blizzard Entertainment' -and $p.DisplayName -and $p.InstallLocation -and $p.DisplayName -ne 'Battle.net') {
            $blizzard += [pscustomobject]@{ name = $p.DisplayName; dir = $p.InstallLocation; icon = $p.DisplayIcon }
        }
    }
}
$result.blizzard = $blizzard

ConvertTo-Json -Compress -Depth 5 -InputObject $result
"#;

    let output = match windows_command("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return serde_json::Value::Null,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or(serde_json::Value::Null)
}

#[cfg(target_os = "windows")]
fn registry_array<'a>(reg: &'a serde_json::Value, key: &str) -> Vec<&'a serde_json::Value> {
    match reg.get(key) {
        Some(serde_json::Value::Array(a)) => a.iter().collect(),
        Some(other @ serde_json::Value::Object(_)) => vec![other],
        _ => Vec::new(),
    }
}

// GOG: DRM-free, launch the executable directly (lifetime is trackable).
#[cfg(target_os = "windows")]
fn scan_gog(reg: &serde_json::Value) -> Vec<Discovered> {
    registry_array(reg, "gog")
        .into_iter()
        .filter_map(|item| {
            let exe = item.get("exe")?.as_str()?.to_string();
            if exe.is_empty() {
                return None;
            }
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| file_stem(&exe));
            Some(Discovered {
                app: App {
                    path: exe.clone(),
                    name,
                    image: String::new(),
                    description: String::new(),
                },
            })
        })
        .collect()
}

// Ubisoft Connect: launch via uplay:// using the install id; icon from the
// game's main exe.
#[cfg(target_os = "windows")]
fn scan_ubisoft(reg: &serde_json::Value) -> Vec<Discovered> {
    registry_array(reg, "ubisoft")
        .into_iter()
        .filter_map(|item| {
            let id = item.get("id")?.as_str()?.to_string();
            let dir = item.get("dir")?.as_str()?.to_string();
            Some(Discovered {
                app: App {
                    path: format!("uplay://launch/{id}/0"),
                    name: folder_name(&dir),
                    image: String::new(),
                    description: String::new(),
                },
            })
        })
        .collect()
}

// EA app / Origin: registry gives the install dir; launch the main exe directly.
#[cfg(target_os = "windows")]
fn scan_ea(reg: &serde_json::Value) -> Vec<Discovered> {
    registry_array(reg, "ea")
        .into_iter()
        .filter_map(|item| {
            let dir = item.get("dir")?.as_str()?.to_string();
            let exe = find_game_exe(Path::new(&dir))?.to_string_lossy().into_owned();
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| folder_name(&dir));
            Some(Discovered {
                app: App {
                    path: exe.clone(),
                    name,
                    image: String::new(),
                    description: String::new(),
                },
            })
        })
        .collect()
}

// Battle.net (Blizzard): detected via uninstall entries; launch the main exe.
#[cfg(target_os = "windows")]
fn scan_blizzard(reg: &serde_json::Value) -> Vec<Discovered> {
    registry_array(reg, "blizzard")
        .into_iter()
        .filter_map(|item| {
            let dir = item.get("dir")?.as_str()?.to_string();
            let exe = find_game_exe(Path::new(&dir))?.to_string_lossy().into_owned();
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Some(Discovered {
                app: App {
                    path: exe.clone(),
                    name,
                    image: String::new(),
                    description: String::new(),
                },
            })
        })
        .collect()
}

// itch.io: installs live under %APPDATA%\itch\apps\<slug>; launch the exe.
#[cfg(target_os = "windows")]
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
            app: App {
                path: exe.clone(),
                name: folder_name(&folder.to_string_lossy()),
                image: String::new(),
                description: String::new(),
            },
        });
    }
    out
}

// Amazon Games: default library; each game has a fuel.json describing its exe.
#[cfg(target_os = "windows")]
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
            app: App {
                path: exe.clone(),
                name: folder_name(&folder.to_string_lossy()),
                image: String::new(),
                description: String::new(),
            },
        });
    }
    out
}

#[cfg(target_os = "windows")]
fn discover_games() -> Vec<App> {
    let mut pending: Vec<Discovered> = Vec::new();
    pending.extend(scan_steam().into_iter().map(|app| Discovered { app }));
    pending.extend(scan_gamepass().into_iter().map(|app| Discovered { app }));

    pending.extend(scan_epic());
    let reg = scan_registry();
    pending.extend(scan_gog(&reg));
    pending.extend(scan_ubisoft(&reg));
    pending.extend(scan_ea(&reg));
    pending.extend(scan_blizzard(&reg));
    pending.extend(scan_itch());
    pending.extend(scan_amazon());

    dedupe_apps(pending.into_iter().map(|d| d.app).collect())
}

#[cfg(not(target_os = "windows"))]
fn discover_games() -> Vec<App> {
    Vec::new()
}

// Discover installed games across all supported launchers, drop entries the user
// uninstalled outside Backpack, dedupe, merge new finds, persist, and return.
fn scan_and_merge(app: &tauri::AppHandle) -> Vec<App> {
    let discovered = discover_games();

    let state = app.state::<AppList>();
    let mut list = state.0.lock().unwrap();
    *list = clean_apps(list.clone());

    let mut seen: HashSet<String> = list.iter().map(|a| normalize_path_key(&a.path)).collect();
    let mut additions = Vec::new();
    for game in discovered {
        if seen.insert(normalize_path_key(&game.path)) {
            additions.push(game);
        }
    }

    list.extend(enrich_with_igdb(additions));
    let result = dedupe_apps(list.clone());
    *list = result.clone();
    drop(list);
    save_apps(app, &result);
    result
}

fn refresh_all_metadata(app: &tauri::AppHandle) -> Vec<App> {
    let state = app.state::<AppList>();
    let list = state.0.lock().unwrap().clone();
    let result = enrich_with_igdb(dedupe_apps(prune_missing(list)));
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
async fn scan_games(app: tauri::AppHandle) -> Vec<App> {
    // Scanning shells out to PowerShell and the filesystem; keep it off the
    // main thread so the UI stays responsive.
    tauri::async_runtime::spawn_blocking(move || scan_and_merge(&app))
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn get_metadata(app: tauri::AppHandle) -> Vec<App> {
    tauri::async_runtime::spawn_blocking(move || refresh_all_metadata(&app))
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn igdb_search(query: String) -> Vec<IgdbSearchResult> {
    tauri::async_runtime::spawn_blocking(move || search_igdb(&query))
        .await
        .unwrap_or_default()
}

#[tauri::command]
fn get_apps(list: tauri::State<AppList>) -> Vec<App> {
    list.0.lock().unwrap().clone()
}

#[tauri::command]
fn remove_app(path: String, app: tauri::AppHandle) -> Vec<App> {
    remove_app_from_list(&path, &app)
}

#[tauri::command]
fn apply_metadata(
    path: String,
    name: String,
    image: String,
    description: String,
    app: tauri::AppHandle,
) -> Vec<App> {
    apply_selected_metadata(&path, name, image, description, &app)
}

#[tauri::command]
fn add_apps(paths: Vec<String>, app: tauri::AppHandle) -> Vec<App> {
    let state = app.state::<AppList>();
    let existing = state.0.lock().unwrap().clone();
    let mut seen: HashSet<String> = existing.iter().map(|a| normalize_path_key(&a.path)).collect();
    let mut additions = Vec::new();

    for path in paths {
        let key = normalize_path_key(&path);
        if seen.contains(&key) || !local_path_exists(&path) {
            continue;
        }
        seen.insert(key);
        additions.push(App {
            name: file_stem(&path),
            path,
            image: String::new(),
            description: String::new(),
        });
    }

    let additions = enrich_with_igdb(additions);
    let mut list = state.0.lock().unwrap();
    list.extend(additions);
    let result = dedupe_apps(list.clone());
    *list = result.clone();
    drop(list);
    save_apps(&app, &result);
    result
}

#[tauri::command]
fn launch(path: String, window: tauri::WebviewWindow, app: tauri::AppHandle) {
    std::thread::spawn(move || {
        // On Windows, UWP/Store/Game Pass apps can't have their lifetime
        // tracked. Launch them but leave the window open, since we'd never
        // receive a reliable "app exited" signal to reopen it.
        #[cfg(target_os = "windows")]
        if !is_trackable(&path) {
            launch_detached(&path);
            return;
        }

        let win = window.clone();
        let _ = app.run_on_main_thread(move || {
            let _ = win.close();
        });
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
        .invoke_handler(tauri::generate_handler![
            get_apps,
            add_apps,
            launch,
            scan_games,
            get_metadata,
            remove_app,
            igdb_search,
            apply_metadata
        ])
        .setup(|app| {
            let loaded = load_apps(app.handle());
            let original_len = loaded.len();
            let cleaned = clean_apps(loaded);
            *app.state::<AppList>().0.lock().unwrap() = cleaned.clone();
            if cleaned.len() != original_len {
                save_apps(app.handle(), &cleaned);
            }

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
        .run(|_app, event| {
            match event {
                // Keep the process alive (minimal, windowless) when the last window closes.
                RunEvent::ExitRequested { code, api, .. } if code.is_none() => {
                    api.prevent_exit();
                }
                #[cfg(target_os = "macos")]
                RunEvent::Reopen { .. } => show_window(_app),
                _ => {}
            }
        });
}
