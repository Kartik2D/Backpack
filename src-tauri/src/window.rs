use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub fn ensure_main_window(app: &tauri::AppHandle) -> tauri::Result<WebviewWindow> {
    match app.get_webview_window("main") {
        Some(window) => Ok(window),
        None => WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
            .title("Backpack")
            .inner_size(800.0, 600.0)
            .min_inner_size(360.0, 300.0)
            .theme(Some(tauri::Theme::Dark))
            .focused(true)
            .build(),
    }
}

pub fn hide_on_close(window: &WebviewWindow) {
    let win = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = win.hide();
        }
    });
}

pub fn show_window(app: &tauri::AppHandle) {
    if let Ok(window) = ensure_main_window(app) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
