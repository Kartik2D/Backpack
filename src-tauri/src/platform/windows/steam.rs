use std::path::PathBuf;
use std::process::Command;

use super::{launch, registry};

fn steam_exe() -> Option<PathBuf> {
    registry::steam_path()
        .map(|path| path.join("steam.exe"))
        .filter(|path| path.exists())
}

fn appid_from_segment(segment: &str) -> Option<String> {
    let appid = segment
        .split(['?', '&', '/', '\\'])
        .next()
        .unwrap_or("")
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    (!appid.is_empty()).then_some(appid)
}

pub fn appid_from_uri(path: &str) -> Option<String> {
    let lower = path.to_lowercase();
    lower
        .strip_prefix("steam://rungameid/")
        .or_else(|| lower.strip_prefix("steam://run/"))
        .and_then(appid_from_segment)
}

fn appid_after_flag(text: &str) -> Option<String> {
    let mut parts = text.split_whitespace();
    while let Some(part) = parts.next() {
        if part.eq_ignore_ascii_case("-applaunch") {
            return parts.next().and_then(appid_from_segment);
        }
    }
    None
}

fn appid_embedded_in_text(text: &str) -> Option<String> {
    text.split(|ch: char| ch.is_whitespace() || ch == '"' || ch == '\'')
        .find_map(appid_from_uri)
        .or_else(|| appid_after_flag(text))
}

fn appid_from_url_file(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()?
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.get(..4)
                .filter(|prefix| prefix.eq_ignore_ascii_case("url="))
                .and_then(|_| appid_from_uri(&line[4..]))
        })
}

pub fn appid_from_shortcut(path: &str) -> Option<String> {
    let lower = path.to_lowercase();
    if lower.ends_with(".url") {
        return appid_from_url_file(path);
    }
    if lower.ends_with(".lnk") {
        let target = launch::resolve_lnk(path)?;
        return appid_embedded_in_text(&format!("{} {}", target.target, target.arguments));
    }
    None
}

pub fn normalize_launch_path(path: &str) -> String {
    appid_from_uri(path)
        .or_else(|| appid_from_shortcut(path))
        .map(|appid| format!("steam://rungameid/{appid}"))
        .unwrap_or_else(|| path.to_string())
}

pub fn launch_appid(appid: &str) -> bool {
    let Some(steam) = steam_exe() else {
        return false;
    };
    let mut command = Command::new(steam);
    command.arg("-applaunch").arg(appid);
    if let Some(dir) = registry::steam_path() {
        command.current_dir(dir);
    }
    command.spawn().is_ok()
}
