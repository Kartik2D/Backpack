use std::fs;

fn main() {
    // Bake credentials from a gitignored .env (next to Cargo.toml) into the binary
    // at compile time. Each KEY=VALUE line becomes available via option_env!("KEY").
    println!("cargo:rerun-if-changed=.env");
    if let Ok(contents) = fs::read_to_string(".env") {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                println!(
                    "cargo:rustc-env={}={}",
                    key.trim(),
                    value.trim().trim_matches('"')
                );
            }
        }
    }

    tauri_build::build()
}
