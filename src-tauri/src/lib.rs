use std::{sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

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
fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
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
fn reset_session(app: AppHandle, state: State<'\''_, SharedTracker>) -> Snapshot {
    let mut t = state.lock().unwrap();
    t.session_pixels = 0.0;
    snapshot(&t)
}
#[tauri::command]
fn update_settings(app: AppHandle, state: State<'\''_, SharedTracker>, settings: Settings) -> Result<Snapshot, String> {
    if settings.ppi < 20.0 || settings.ppi > 1000.0 { return Err("PPI must be between 20 and 1000".into()); }
    if !["px", "m", "km"].contains(&settings.unit.as_str()) { return Err("Invalid unit".into()); }
    let mut t = state.lock().unwrap();
    t.settings = settings;
    Ok(snapshot(&t))
}

fn start_tracker(app: AppHandle, shared: SharedTracker) {
    thread::spawn(move || {
        let mut previous: Option<(i32, i32)> = None;
        let mut last_publish = Instant::now();
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
            if last_publish.elapsed() >= Duration::from_millis(750) {
                let snap = { snapshot(&shared.lock().unwrap()) };
                let _ = app.emit("tracker-update", &snap);
                last_publish = Instant::now();
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
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("valid tray icon");
            let window_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png")).expect("valid window icon");
            if let Some(window) = app.get_webview_window("main") { let _ = window.set_icon(window_icon); }
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause / Resume Tracking", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
            let reset = MenuItem::with_id(app, "reset", "Reset Session", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &hide, &pause, &reset, &quit])?;
            TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .tooltip("Mouse Distance: 0.00 km")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        show_window(tray.app_handle());
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_window(app),
                    "hide" => { if let Some(window) = app.get_webview_window("main") { let _ = window.hide(); } },
                    "pause" => { let state = app.state::<SharedTracker>(); let mut t = state.lock().unwrap(); t.paused = !t.paused; },
                    "reset" => { let state = app.state::<SharedTracker>(); let mut t = state.lock().unwrap(); t.session_pixels = 0.0; },
                    "quit" => { app.exit(0); },
                    _ => {}
                })
                .build(app)?;
            start_tracker(app.handle().clone(), tracker);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![get_snapshot, toggle_tracking, reset_session, update_settings])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
