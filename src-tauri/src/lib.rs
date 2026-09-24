use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::Manager;

struct Tracker {
    session_pixels: f64,
    paused: bool,
}
type SharedTracker = Arc<Mutex<Tracker>>;

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
            let tracker = Arc::new(Mutex::new(Tracker { session_pixels: 0.0, paused: false }));
            app.manage(tracker.clone());
            start_tracker(tracker);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running application");
}
