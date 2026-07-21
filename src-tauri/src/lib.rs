mod config;
mod croc;
mod history;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(croc::CrocState::new())
        .setup(|app| {
            let settings = config::read_settings(app.app_handle());
            let minimize_to_tray = settings.minimize_to_tray;

            use tauri::menu::{MenuBuilder, MenuItemBuilder};
            use tauri::tray::{TrayIconBuilder, TrayIconEvent};

            let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let send = MenuItemBuilder::with_id("send", "Send Files").build(app)?;
            let receive = MenuItemBuilder::with_id("receive", "Receive Files").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit croc-gui").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&send)
                .item(&receive)
                .separator()
                .item(&settings)
                .separator()
                .item(&quit)
                .build()?;

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    route @ ("send" | "receive" | "settings") => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.eval(&format!("window.location.href = '/{route}'"));
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            if window.is_visible().ok().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            if minimize_to_tray {
                // avoid unused variable warning when false
                let _ = _tray;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            let app = window.app_handle();
            let settings = config::read_settings(app);
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if settings.minimize_to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            croc::send_file,
            croc::receive_file,
            croc::cancel_transfer,
            config::get_settings,
            config::save_settings,
            history::get_transfer_history,
            history::clear_transfer_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
