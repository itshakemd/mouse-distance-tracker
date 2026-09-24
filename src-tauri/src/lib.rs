use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{AppHandle, Manager, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    unit: String,
    ppi: f64,
}
impl Default for Settings {
    fn default() -> Self { Self { unit: "km".into(), ppi: 96.0 } }
}

struct Tracker {
    session_pixels: f64,
    all_time_pixels: f64,
    paused: bool,
    settings: Settings,
}
type SharedTracker = Arc<Mutex<Tracker>>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    session_pixels: f64,
    all_time_pixels: f64,
    paused: bool,
    settings: Settings,
}

fn snapshot(t: &Tracker) -> Snapshot {
    Snapshot { session_pixels: t.session_pixels, all_time_pixels: t.all_time_pixels, paused: t.paused, settings: t.settings.clone() }
}
fn formatted(px: f64, settings: &Settings) -> String {
    let inches = px / settings.ppi.max(1.0);
    match settings.unit.as_str() {
        "px" => format!("{:.0} px", px),
        "m" => format!("{:.2} m", inches * 0.0254),
        _ => format!("{:.2} km", inches * 0.0000254),
    }
}

#[tauri::command]
fn get_snapshot(state: State<'\''_, SharedTracker>) -> Snapshot { snapshot(&state.lock().unwrap()) }
#[tauri::command]
fn toggle_tracking(app: AppHandle, state: State<'\''_, SharedTracker>) -> Snapshot {
    let mut t = state.lock().unwrap();
    t.paused = !t.paused;
    snapshot(&t)
}
#[tauri::command]
fn update_settings(app: AppHandle, state: State<'\''_, SharedTracker>, settings: Settings) -> Result<Snapshot, String> {
    if settings.ppi < 20.0 || settings.ppi > 1000.0 { return Err("PPI must be between 20 and 1000".into()); }
    if !["px", "m", "km"].contains(&settings.unit.as_str()) { return Err("Invalid unit".into()); }
    let mut t = state.lock().unwrap();
    t.data_settings(settings);
    Ok(snapshot(&t))
}
impl Tracker { fn data_settings(&mut self, s: Settings) { self.settings = s; } }

fn start_tracker(shared: SharedTracker) {
    thread::spawn(move || {
        let mut previous: Option<(i32, i32)> = None;
        loop {
            #[cfg(target_os = "windows")]
            let current = unsafe {
                let mut p = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                (windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p) != 0).then_some((p.x, p.y))
            };
            #[cfg(not(target_os = "windows"))]
            let current: Option<(i32, i32)> = None;
            if let Some(pos) = current {
                if let Some(old) = previous {
                    let dx = (pos.0 - old.0) as f64;
                    let dy = (pos.1 - old.1) as f64;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let mut t = shared.lock().unwrap();
                    if !t.paused {
                        t.session_pixels += distance;
                        t.all_time_pixels += distance;
                    }
                }
                previous = Some(pos);
            }
            thread::sleep(Duration::from_millis(16));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let tracker = Arc::new(Mutex::new(Tracker { session_pixels: 0.0, all_time_pixels: 0.0, paused: false, settings: Settings::default() }));
            app.manage(tracker.clone());
            start_tracker(tracker);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_snapshot, toggle_tracking, update_settings])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
