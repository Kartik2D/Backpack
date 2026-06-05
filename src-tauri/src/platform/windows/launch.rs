use std::path::Path;
use std::process::Command;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    STGM_READ,
};
use windows::Win32::System::Threading::{WaitForSingleObject, INFINITE};
use windows::Win32::UI::Shell::{
    IShellLinkW, ShellExecuteExW, ShellLink, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    SLGP_RAWPATH,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use super::steam;

#[derive(Default)]
pub(super) struct LinkTarget {
    pub target: String,
    pub arguments: String,
    pub working_dir: String,
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn from_wide_nul(buf: &[u16]) -> String {
    let len = buf.iter().position(|ch| *ch == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len]).trim().to_string()
}

fn init_com() {
    unsafe {
        // Ignore RPC_E_CHANGED_MODE: if Tauri already initialized COM for this
        // thread in another apartment, the existing apartment is still usable.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
}

pub(super) fn resolve_lnk(path: &str) -> Option<LinkTarget> {
    init_com();
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        let path_w = wide(path);
        persist.Load(PCWSTR(path_w.as_ptr()), STGM_READ).ok()?;

        let mut target = vec![0u16; MAX_PATH as usize];
        link.GetPath(&mut target, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
            .ok()?;

        let mut arguments = vec![0u16; 4096];
        let _ = link.GetArguments(&mut arguments);

        let mut working_dir = vec![0u16; MAX_PATH as usize];
        let _ = link.GetWorkingDirectory(&mut working_dir);

        Some(LinkTarget {
            target: from_wide_nul(&target),
            arguments: from_wide_nul(&arguments),
            working_dir: from_wide_nul(&working_dir),
        })
    }
}

// Resolve a .lnk shortcut's TargetPath. Returns Some("") for Store/UWP
// shortcuts, which point at an AppUserModelID via a shell ID list and have no
// file target.
pub fn resolve_lnk_target(path: &str) -> Option<String> {
    resolve_lnk(path).map(|target| target.target)
}

fn shell_execute(
    path: &str,
    parameters: Option<&str>,
    directory: Option<&str>,
    wait: bool,
) -> bool {
    let path_w = wide(path);
    let params_w = parameters.filter(|s| !s.is_empty()).map(wide);
    let dir_w = directory.filter(|s| !s.is_empty()).map(wide);
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: if wait { SEE_MASK_NOCLOSEPROCESS } else { 0 },
        hwnd: HWND::default(),
        lpVerb: PCWSTR::null(),
        lpFile: PCWSTR(path_w.as_ptr()),
        lpParameters: params_w
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or_else(PCWSTR::null),
        lpDirectory: dir_w
            .as_ref()
            .map(|value| PCWSTR(value.as_ptr()))
            .unwrap_or_else(PCWSTR::null),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };

    let launched = unsafe { ShellExecuteExW(&mut info).is_ok() };
    if launched && wait && !info.hProcess.is_invalid() {
        unsafe {
            let _ = WaitForSingleObject(info.hProcess, INFINITE);
            let _ = CloseHandle(info.hProcess);
        }
    }
    launched
}

fn wait_for_path(path: &str) {
    let mut command = Command::new(path);
    if let Some(parent) = Path::new(path).parent() {
        command.current_dir(parent);
    }
    match command.spawn() {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(_) => {
            let _ = shell_execute(path, None, None, true);
        }
    }
}

pub fn wait_for_app(path: &str) {
    if path.to_lowercase().ends_with(".lnk") {
        if let Some(target) = resolve_lnk(path) {
            if !target.target.is_empty() {
                let directory = (!target.working_dir.is_empty())
                    .then_some(target.working_dir.as_str())
                    .or_else(|| Path::new(&target.target).parent().and_then(|p| p.to_str()));
                if shell_execute(
                    &target.target,
                    (!target.arguments.is_empty()).then_some(target.arguments.as_str()),
                    directory,
                    true,
                ) {
                    return;
                }
            }
        }
        let _ = shell_execute(path, None, None, true);
        return;
    }

    wait_for_path(path);
}

pub fn launch_detached(path: &str) {
    if let Some(appid) = steam::appid_from_uri(path) {
        if steam::launch_appid(&appid) {
            return;
        }
    }

    // Packaged apps (shell:AppsFolder\<AUMID>) activate most reliably through
    // explorer.exe; protocols and regular files go through ShellExecute.
    if path.to_lowercase().starts_with("shell:") {
        let _ = Command::new("explorer.exe").arg(path).spawn();
        return;
    }
    let _ = shell_execute(path, None, None, false);
}

// Whether we can reliably wait for the app to exit. UWP/Store/Game Pass apps
// run inside a container; their launcher returns immediately, so waiting is
// meaningless and the window would reappear instantly.
pub fn is_trackable(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.contains("windowsapps") || lower.contains("shell:appsfolder") || lower.contains("://")
    {
        return false;
    }
    if lower.ends_with(".lnk") {
        match resolve_lnk_target(path) {
            Some(target) => {
                let t = target.to_lowercase();
                if target.is_empty() || t.contains("windowsapps") {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn local_path_exists(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    if path.to_lowercase().ends_with(".lnk") {
        return match resolve_lnk_target(path) {
            Some(target) if target.is_empty() => true,
            Some(target) => Path::new(&target).exists(),
            None => true,
        };
    }
    true
}
