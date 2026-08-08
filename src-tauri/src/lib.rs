mod config;
mod croc;
mod history;

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(croc::CrocState::new())
        .manage(history::HistoryState::new())
        .manage(config::SettingsState::new());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ));
    }

    builder = builder
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};

                let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
                let send = MenuItemBuilder::with_id("send", "Send Files").build(app)?;
                let receive = MenuItemBuilder::with_id("receive", "Receive Files").build(app)?;
                let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit croc-gui").build(app)?;
                let menu = MenuBuilder::new(app)
                    .item(&show)
                    .separator()
                    .item(&send)
                    .item(&receive)
                    .separator()
                    .item(&settings_item)
                    .separator()
                    .item(&quit)
                    .build()?;

                let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                    .expect("Failed to load tray icon");

                let tray = TrayIconBuilder::with_id("main-tray")
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
                                log::info!("Tray: navigating to /{}", route);
                                let _ = window.emit("navigate", format!("/{route}"));
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

                drop(tray);
            }

            Ok(())
        });

    #[cfg(desktop)]
    {
        builder = builder.on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let settings = config::read_settings(app);
                if settings.minimize_to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        });
    }

    builder
        .invoke_handler(tauri::generate_handler![
            croc::send_file,
            croc::receive_file,
            croc::cancel_transfer,
            croc::check_croc_available,
            croc::save_temp_text,
            config::get_settings,
            config::save_settings,
            history::get_transfer_history,
            history::clear_transfer_history,
            history::set_record_pinned,
            history::delete_transfer_record,
            history::delete_record_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
