use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub fn show_window(app: &tauri::AppHandle) {
    let window = match app.get_webview_window("main") {
        Some(w) => Some(w),
        None => WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
            .title("Backpack")
            .inner_size(800.0, 600.0)
            .min_inner_size(360.0, 300.0)
            .theme(Some(tauri::Theme::Dark))
            .focused(true)
            .build()
            .ok(),
    };
    if let Some(w) = window {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
