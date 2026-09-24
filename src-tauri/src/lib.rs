use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
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

#[derive(Default, Serialize, Deserialize)]
struct StoredData {
    all_time_pixels: f64,
    days: BTreeMap<String, f64>,
    settings: Settings,
}
struct Tracker {
    data: StoredData,
    session_pixels: f64,
    paused: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    session_pixels: f64,
    today_pixels: f64,
    week_pixels: f64,
    month_pixels: f64,
    year_pixels: f64,
    all_time_pixels: f64,
    paused: bool,
    settings: Settings,
}
type SharedTracker = Arc<Mutex<Tracker>>;

fn data_path(app: &AppHandle) -> std::path::PathBuf {
    app.path().app_data_dir().expect("app data directory").join("tracker.json")
}
fn load(app: &AppHandle) -> StoredData {
    fs::read_to_string(data_path(app)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
fn persist(app: &AppHandle, tracker: &Tracker) {
    let path = data_path(app);
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    if let Ok(json) = serde_json::to_string_pretty(&tracker.data) { let _ = fs::write(path, json); }
}
fn snapshot(t: &Tracker) -> Snapshot {
    let today = Local::now().date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let (mut today_px, mut week_px, mut month_px, mut year_px) = (0.0, 0.0, 0.0, 0.0);
    for (key, value) in &t.data.days {
        if let Ok(date) = NaiveDate::parse_from_str(key, "%Y-%m-%d") {
            if date == today { today_px += value; }
            if date >= week_start && date <= today { week_px += value; }
            if date.year() == today.year() && date.month() == today.month() { month_px += value; }
            if date.year() == today.year() { year_px += value; }
        }
    }
    Snapshot { session_pixels: t.session_pixels, today_pixels: today_px, week_pixels: week_px, month_pixels: month_px, year_pixels: year_px, all_time_pixels: t.data.all_time_pixels, paused: t.paused, settings: t.data.settings.clone() }
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
    persist(&app, &t);
    snapshot(&t)
}
#[tauri::command]
fn reset_session(app: AppHandle, state: State<'\''_, SharedTracker>) -> Snapshot {
    let mut t = state.lock().unwrap();
    t.session_pixels = 0.0;
    persist(&app, &t);
    snapshot(&t)
}
#[tauri::command]
fn update_settings(app: AppHandle, state: State<'\''_, SharedTracker>, settings: Settings) -> Result<Snapshot, String> {
    if settings.ppi < 20.0 || settings.ppi > 1000.0 { return Err("PPI must be between 20 and 1000".into()); }
    if !["px", "m", "km"].contains(&settings.unit.as_str()) { return Err("Invalid unit".into()); }
    let mut t = state.lock().unwrap();
    t.data.settings = settings;
    persist(&app, &t);
    Ok(snapshot(&t))
}

fn start_tracker(app: AppHandle, shared: SharedTracker) {
    thread::spawn(move || {
        let mut previous: Option<(i32, i32)> = None;
        let mut last_publish = Instant::now();
        let mut last_save = Instant::now();
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
                        t.data.all_time_pixels += distance;
                        *t.data.days.entry(Local::now().format("%Y-%m-%d").to_string()).or_default() += distance;
                    }
                }
                previous = Some(pos);
            }
            if last_publish.elapsed() >= Duration::from_millis(750) {
                let snap = { snapshot(&shared.lock().unwrap()) };
                let _ = app.emit("tracker-update", &snap);
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some(format!("Mouse Distance: {}", formatted(snap.today_pixels, &snap.settings))));
                }
                last_publish = Instant::now();
            }
            if last_save.elapsed() >= Duration::from_secs(10) {
                persist(&app, &shared.lock().unwrap());
                last_save = Instant::now();
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
            let tracker = Arc::new(Mutex::new(Tracker { data: load(app.handle()), session_pixels: 0.0, paused: false }));
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
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event { show_window(tray.app_handle()); }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_window(app),
                    "hide" => { if let Some(window) = app.get_webview_window("main") { let _ = window.hide(); } },
                    "pause" => { let state = app.state::<SharedTracker>(); let mut t = state.lock().unwrap(); t.paused = !t.paused; persist(app, &t); },
                    "reset" => { let state = app.state::<SharedTracker>(); let mut t = state.lock().unwrap(); t.session_pixels = 0.0; persist(app, &t); },
                    "quit" => { let state = app.state::<SharedTracker>(); persist(app, &state.lock().unwrap()); app.exit(0); },
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
