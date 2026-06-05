use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::model::App;
use crate::store::normalize_path_key;
use crate::window::show_window;

const EVENT: &str = "game-state";
const STATE_LAUNCHING: &str = "launching";
const STATE_PLAYING: &str = "playing";
const STATE_STOPPED: &str = "stopped";
const TRACKING_FREQUENCY: Duration = Duration::from_secs(2);
const MAX_STARTUP_MISSES: u8 = 5;
const SLEEP_GRACE: Duration = Duration::from_secs(30);

#[derive(Clone, Serialize)]
pub struct GameState {
    pub path: String,
    pub state: &'static str,
    pub session_secs: Option<u64>,
}

#[derive(Default)]
pub struct GameStates(pub Mutex<HashMap<String, GameState>>);

#[derive(Clone)]
pub struct TrackTarget {
    pub dir: Option<PathBuf>,
    pub seed_pid: Option<u32>,
}

impl TrackTarget {
    pub fn from_app(app: &App, launch_path: &str) -> Self {
        Self {
            dir: app
                .install_dir
                .as_deref()
                .filter(|dir| !dir.is_empty())
                .map(PathBuf::from)
                .or_else(|| local_tracking_dir(launch_path)),
            seed_pid: None,
        }
    }
}

pub fn emit_launching(app: &AppHandle, path: String) {
    emit_state(app, path, STATE_LAUNCHING, None);
}

pub fn snapshot(states: &GameStates) -> Vec<GameState> {
    states.0.lock().unwrap().values().cloned().collect()
}

pub fn spawn(app: AppHandle, window: WebviewWindow, path: String, target: TrackTarget) {
    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut related_ids = target
            .seed_pid
            .map(|pid| HashSet::from([pid]))
            .unwrap_or_default();
        let mut startup_misses = 0u8;

        loop {
            refresh_processes(&mut sys);
            if running_pid(&sys, &target, &mut related_ids).is_some() {
                emit_state(&app, path.clone(), STATE_PLAYING, None);
                hide_window(&app, &window);
                break;
            }

            startup_misses += 1;
            if startup_misses >= MAX_STARTUP_MISSES {
                emit_state(&app, path, STATE_STOPPED, Some(0));
                return;
            }

            thread::sleep(TRACKING_FREQUENCY);
        }

        let mut play_time = Duration::ZERO;
        loop {
            let iteration = Instant::now();
            thread::sleep(TRACKING_FREQUENCY);
            refresh_processes(&mut sys);

            if running_pid(&sys, &target, &mut related_ids).is_none() {
                let session_secs = play_time.as_secs();
                emit_state(&app, path.clone(), STATE_STOPPED, Some(session_secs));
                reopen_window(&app);
                return;
            }

            let elapsed = iteration.elapsed();
            if elapsed <= TRACKING_FREQUENCY + SLEEP_GRACE {
                play_time += elapsed;
            }
        }
    });
}

fn emit_state(app: &AppHandle, path: String, state: &'static str, session_secs: Option<u64>) {
    let payload = GameState {
        path: path.clone(),
        state,
        session_secs,
    };

    if state != STATE_STOPPED {
        app.state::<GameStates>()
            .0
            .lock()
            .unwrap()
            .insert(path.clone(), payload.clone());
    }

    let _ = app.emit(EVENT, payload);

    if state == STATE_STOPPED {
        app.state::<GameStates>().0.lock().unwrap().remove(&path);
    }
}

fn hide_window(app: &AppHandle, window: &WebviewWindow) {
    let win = window.clone();
    let _ = app.run_on_main_thread(move || {
        let _ = win.hide();
    });
}

fn reopen_window(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || show_window(&handle));
}

fn refresh_processes(sys: &mut System) {
    sys.refresh_processes(ProcessesToUpdate::All, true);
}

fn running_pid(sys: &System, target: &TrackTarget, related_ids: &mut HashSet<u32>) -> Option<Pid> {
    let mut running_related = HashSet::new();
    let mut discovered_related = HashSet::new();

    for (pid, process) in sys.processes() {
        let pid_u32 = pid.as_u32();
        if process
            .parent()
            .map(|parent| related_ids.contains(&parent.as_u32()))
            .unwrap_or(false)
        {
            discovered_related.insert(pid_u32);
        }

        if related_ids.contains(&pid_u32) || discovered_related.contains(&pid_u32) {
            running_related.insert(pid_u32);
        }

        if process_in_dir(process.exe(), target.dir.as_deref()) {
            *related_ids = running_related
                .into_iter()
                .chain(discovered_related.into_iter())
                .collect();
            return Some(*pid);
        }
    }

    *related_ids = running_related
        .into_iter()
        .chain(discovered_related.into_iter())
        .collect();
    related_ids.iter().next().and_then(|pid| {
        sys.process(Pid::from_u32(*pid))
            .map(|_| Pid::from_u32(*pid))
    })
}

fn process_in_dir(exe: Option<&Path>, dir: Option<&Path>) -> bool {
    let (Some(exe), Some(dir)) = (exe, dir) else {
        return false;
    };

    let exe_key = normalize_path_key(&exe.to_string_lossy());
    let mut dir_key = normalize_path_key(&dir.to_string_lossy());
    if !dir_key.ends_with('/') {
        dir_key.push('/');
    }

    exe_key.starts_with(&dir_key)
}

fn local_tracking_dir(path: &str) -> Option<PathBuf> {
    let lower = path.to_lowercase();
    if lower.contains("://") || lower.starts_with("shell:") {
        return None;
    }

    let path = Path::new(path);
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("app"))
        .unwrap_or(false)
    {
        return Some(path.to_path_buf());
    }

    path.parent().map(Path::to_path_buf)
}
