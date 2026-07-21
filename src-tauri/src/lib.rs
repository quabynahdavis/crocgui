mod config;
mod croc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(croc::CrocState::new())
        .invoke_handler(tauri::generate_handler![
            croc::send_file,
            croc::receive_file,
            croc::cancel_transfer,
            config::get_settings,
            config::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
