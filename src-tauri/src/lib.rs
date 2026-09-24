use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{Manager, State};

struct Tracker {
    session_pixels: f64,
    all_time_pixels: f64,
    paused: bool,
}
type SharedTracker = Arc<Mutex<Tracker>>;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    session_pixels: f64,
    all_time_pixels: f64,
    paused: bool,
}

fn snapshot(t: &Tracker) -> Snapshot {
    Snapshot { session_pixels: t.session_pixels, all_time_pixels: t.all_time_pixels, paused: t.paused }
}

#[tauri::command]
fn get_snapshot(state: State<'\''_, SharedTracker>) -> Snapshot {
    snapshot(&state.lock().unwrap())
}

fn start_tracker(shared: SharedTracker) {
    thread::spawn(move || {
        let mut previous: Option<(i32, i32)> = None;
        loop {
            #[cfg(target_os = "windows")]
            let current = unsafe {
                let mut p = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                (windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p) != 0)
                    .then_some((p.x, p.y))
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
            let tracker = Arc::new(Mutex::new(Tracker { session_pixels: 0.0, all_time_pixels: 0.0, paused: false }));
            app.manage(tracker.clone());
            start_tracker(tracker);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
