use std::path::PathBuf;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, REG_EXPAND_SZ, REG_SZ, REG_VALUE_TYPE,
};

#[derive(Clone)]
pub struct GogInstall {
    pub name: String,
    pub exe: String,
}

#[derive(Clone)]
pub struct UbisoftInstall {
    pub id: String,
    pub dir: String,
}

#[derive(Clone)]
pub struct EaInstall {
    pub name: String,
    pub dir: String,
}

#[derive(Clone)]
pub struct BlizzardInstall {
    pub name: String,
    pub dir: String,
}

struct RegKey(HKEY);

impl Drop for RegKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn from_wide_nul(buf: &[u16]) -> String {
    let len = buf.iter().position(|ch| *ch == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len]).trim().to_string()
}

fn open_key(root: HKEY, subkey: &str, wow64_32: bool) -> Option<RegKey> {
    let mut key = HKEY(std::ptr::null_mut());
    let access = if wow64_32 {
        KEY_READ | KEY_WOW64_32KEY
    } else {
        KEY_READ
    };
    let subkey = wide(subkey);
    let status = unsafe {
        RegOpenKeyExW(
            root,
            PCWSTR(subkey.as_ptr()),
            None,
            access,
            &mut key as *mut HKEY,
        )
    };
    (status == ERROR_SUCCESS).then_some(RegKey(key))
}

fn read_string_value(key: &RegKey, value: &str) -> Option<String> {
    let value = wide(value);
    let mut ty = REG_VALUE_TYPE::default();
    let mut bytes = 0u32;
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(value.as_ptr()),
            None,
            Some(&mut ty),
            None,
            Some(&mut bytes),
        )
    };
    if status != ERROR_SUCCESS || bytes == 0 || (ty != REG_SZ && ty != REG_EXPAND_SZ) {
        return None;
    }

    let mut data = vec![0u8; bytes as usize];
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(value.as_ptr()),
            None,
            Some(&mut ty),
            Some(data.as_mut_ptr()),
            Some(&mut bytes),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }

    let words = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let result = from_wide_nul(&words);
    (!result.is_empty()).then_some(result)
}

fn subkey_names(key: &RegKey) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = 0u32;
    loop {
        let mut name = vec![0u16; 260];
        let mut len = (name.len() - 1) as u32;
        let status = unsafe {
            RegEnumKeyExW(
                key.0,
                index,
                Some(PWSTR(name.as_mut_ptr())),
                &mut len,
                None,
                None,
                None,
                None,
            )
        };
        if status == ERROR_NO_MORE_ITEMS {
            break;
        }
        if status == ERROR_SUCCESS {
            out.push(String::from_utf16_lossy(&name[..len as usize]));
        }
        index += 1;
    }
    out
}

fn subkeys(root: HKEY, path: &str, wow64_32: bool) -> Vec<(String, RegKey)> {
    let Some(parent) = open_key(root, path, wow64_32) else {
        return Vec::new();
    };
    subkey_names(&parent)
        .into_iter()
        .filter_map(|name| {
            let full = format!("{path}\\{name}");
            open_key(root, &full, wow64_32).map(|key| (name, key))
        })
        .collect()
}

pub fn steam_path() -> Option<PathBuf> {
    let key = open_key(HKEY_CURRENT_USER, "Software\\Valve\\Steam", false)?;
    read_string_value(&key, "SteamPath").map(PathBuf::from)
}

pub fn gog_installs() -> Vec<GogInstall> {
    subkeys(HKEY_LOCAL_MACHINE, "SOFTWARE\\GOG.com\\Games", true)
        .into_iter()
        .filter_map(|(_, key)| {
            let exe = read_string_value(&key, "exe")?;
            let name = read_string_value(&key, "gameName").unwrap_or_default();
            Some(GogInstall { name, exe })
        })
        .collect()
}

pub fn ubisoft_installs() -> Vec<UbisoftInstall> {
    subkeys(
        HKEY_LOCAL_MACHINE,
        "SOFTWARE\\Ubisoft\\Launcher\\Installs",
        true,
    )
    .into_iter()
    .filter_map(|(id, key)| {
        let dir = read_string_value(&key, "InstallDir")?;
        Some(UbisoftInstall { id, dir })
    })
    .collect()
}

pub fn ea_installs() -> Vec<EaInstall> {
    ["SOFTWARE\\EA Games", "SOFTWARE\\Origin Games"]
        .into_iter()
        .flat_map(|base| subkeys(HKEY_LOCAL_MACHINE, base, true))
        .filter_map(|(fallback_name, key)| {
            let dir = read_string_value(&key, "Install Dir")
                .or_else(|| read_string_value(&key, "InstallDir"))?;
            let name = read_string_value(&key, "DisplayName").unwrap_or(fallback_name);
            Some(EaInstall { name, dir })
        })
        .collect()
}

pub fn blizzard_installs() -> Vec<BlizzardInstall> {
    [
        ("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall", false),
        (
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            true,
        ),
    ]
    .into_iter()
    .flat_map(|(base, wow64_32)| subkeys(HKEY_LOCAL_MACHINE, base, wow64_32))
    .filter_map(|(_, key)| {
        let publisher = read_string_value(&key, "Publisher")?;
        let name = read_string_value(&key, "DisplayName")?;
        let dir = read_string_value(&key, "InstallLocation")?;
        (publisher == "Blizzard Entertainment" && name != "Battle.net")
            .then_some(BlizzardInstall { name, dir })
    })
    .collect()
}
