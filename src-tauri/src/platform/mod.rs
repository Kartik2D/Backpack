use std::path::Path;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod linux;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use linux::*;

// Extract single-line `"key" "value"` entries from a Valve VDF/ACF file.
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

pub fn file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn folder_name(dir: &str) -> String {
    Path::new(dir)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(dir)
        .to_string()
}
