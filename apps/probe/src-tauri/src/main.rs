//! My Path Probe — technical diagnostics for Phase 1.
//!
//! The Rust side is intentionally small: it starts/stops `mp-input`, forwards
//! every captured event to the UI (`mp-input` Tauri event), and answers a few
//! questions (environment, devices, CPU/RAM). Report generation happens in the
//! UI; Rust only writes the files to the Downloads folder.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mp_core::{DeviceInfo, Environment, InputEvent};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager, State};

struct AppState {
    capture: Mutex<Option<mp_input::Capture>>,
    sys: Mutex<System>,
}

#[derive(Serialize)]
struct EnvReport {
    env: Environment,
    cpu_brand: String,
    cpu_logical: usize,
    cpu_physical: Option<usize>,
    total_ram_mb: u64,
    webview_version: Option<String>,
}

#[derive(Serialize)]
struct Stats {
    /// CPU of the Probe's own process, as % of the whole machine.
    cpu_self: f32,
    /// CPU of the Probe + its WebView child processes, as % of the whole machine.
    cpu_tree: f32,
    ram_self_mb: f64,
    ram_tree_mb: f64,
    processes_in_tree: usize,
}

#[tauri::command]
fn environment(state: State<'_, AppState>) -> EnvReport {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_default();
    EnvReport {
        env: mp_input::env::detect(env!("CARGO_PKG_VERSION")),
        cpu_brand,
        cpu_logical: sys.cpus().len(),
        cpu_physical: System::physical_core_count(),
        total_ram_mb: sys.total_memory() / (1024 * 1024),
        webview_version: tauri::webview_version().ok(),
    }
}

#[tauri::command]
fn devices() -> Vec<DeviceInfo> {
    mp_input::list_devices()
}

#[tauri::command]
fn capture_running(state: State<'_, AppState>) -> bool {
    state.capture.lock().unwrap().is_some()
}

#[tauri::command]
fn start_capture(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut slot = state.capture.lock().unwrap();
    if slot.is_some() {
        return Ok(());
    }
    let (tx, rx) = mp_input::unbounded::<InputEvent>();
    let capture = mp_input::start(tx).map_err(|e| e.to_string())?;
    std::thread::Builder::new()
        .name("probe-forwarder".into())
        .spawn(move || {
            // Ends by itself when the capture stops and every sender is dropped.
            for ev in rx {
                let _ = app.emit("mp-input", &ev);
            }
        })
        .map_err(|e| e.to_string())?;
    *slot = Some(capture);
    Ok(())
}

#[tauri::command]
fn stop_capture(state: State<'_, AppState>) {
    if let Some(c) = state.capture.lock().unwrap().take() {
        c.stop();
    }
}

#[tauri::command]
fn stats(state: State<'_, AppState>) -> Result<Stats, String> {
    let me = sysinfo::get_current_pid().map_err(|e| e.to_string())?;
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let logical = sys.cpus().len().max(1) as f32;

    // The Probe plus every descendant (WebView2 / WebKitGTK helper processes).
    let mut tree: HashSet<Pid> = HashSet::from([me]);
    loop {
        let before = tree.len();
        for (pid, p) in sys.processes() {
            if let Some(parent) = p.parent()
                && tree.contains(&parent)
            {
                tree.insert(*pid);
            }
        }
        if tree.len() == before {
            break;
        }
    }
    let (mut cpu_tree, mut ram_tree) = (0f32, 0u64);
    for pid in &tree {
        if let Some(p) = sys.process(*pid) {
            cpu_tree += p.cpu_usage();
            ram_tree += p.memory();
        }
    }
    let own = sys.process(me);
    let mb = |b: u64| b as f64 / (1024.0 * 1024.0);
    Ok(Stats {
        cpu_self: own.map(|p| p.cpu_usage()).unwrap_or(0.0) / logical,
        cpu_tree: cpu_tree / logical,
        ram_self_mb: mb(own.map(|p| p.memory()).unwrap_or(0)),
        ram_tree_mb: mb(ram_tree),
        processes_in_tree: tree.len(),
    })
}

fn output_dir(app: &AppHandle) -> PathBuf {
    let path = app.path();
    path.download_dir()
        .or_else(|_| path.home_dir())
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// Writes `<base>.html` and `<base>.json` to the Downloads folder and returns both paths.
#[tauri::command]
fn save_report(
    app: AppHandle,
    system: String,
    html: String,
    json: String,
) -> Result<Vec<String>, String> {
    let safe: String = system
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let dir = output_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let base = dir.join(format!("relatorio-probe-{safe}-{stamp}"));
    let html_path = base.with_extension("html");
    let json_path = base.with_extension("json");
    std::fs::write(&html_path, html).map_err(|e| e.to_string())?;
    std::fs::write(&json_path, json).map_err(|e| e.to_string())?;
    Ok(vec![
        html_path.display().to_string(),
        json_path.display().to_string(),
    ])
}

/// Opens the file manager showing the given file.
#[tauri::command]
fn reveal(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    #[cfg(windows)]
    let r = std::process::Command::new("explorer")
        .arg(format!("/select,{}", p.display()))
        .spawn();
    #[cfg(not(windows))]
    let r = std::process::Command::new("xdg-open")
        .arg(p.parent().unwrap_or(&p))
        .spawn();
    r.map(|_| ()).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            capture: Mutex::new(None),
            sys: Mutex::new(System::new()),
        })
        .invoke_handler(tauri::generate_handler![
            environment,
            devices,
            capture_running,
            start_capture,
            stop_capture,
            stats,
            save_report,
            reveal
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event
                && let Some(c) = window.state::<AppState>().capture.lock().unwrap().take()
            {
                c.stop();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running My Path Probe");
}
