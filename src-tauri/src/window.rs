use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub fn ensure_main_window(app: &tauri::AppHandle) -> tauri::Result<WebviewWindow> {
    match app.get_webview_window("main") {
        Some(window) => Ok(window),
        None => {
            let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("Backpack")
                .inner_size(800.0, 600.0)
                .min_inner_size(360.0, 300.0)
                .theme(Some(tauri::Theme::Dark))
                .focused(true);

            #[cfg(target_os = "macos")]
            {
                builder = builder
                    .title_bar_style(tauri::TitleBarStyle::Overlay)
                    .hidden_title(true)
                    .traffic_light_position(tauri::LogicalPosition::new(19.0, 22.0));
            }

            let window = builder.build()?;

            // Windows: replace native decorations with decorum's overlay
            // window controls (keeps Snap Layouts working).
            #[cfg(windows)]
            {
                use tauri_plugin_decorum::WebviewWindowExt;
                let _ = window.create_overlay_titlebar();
            }

            Ok(window)
        }
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
