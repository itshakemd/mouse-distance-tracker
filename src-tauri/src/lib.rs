use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("Mouse Distance Tracker");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running application");
}
