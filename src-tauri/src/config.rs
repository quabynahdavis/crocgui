use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub relay: String,
    pub curve: String,
    pub disable_compression: bool,
    pub output_dir: String,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            relay: String::new(),
            curve: "p256".into(),
            disable_compression: false,
            output_dir: String::new(),
            theme: "system".into(),
        }
    }
}

fn config_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_config_dir().unwrap_or_else(|_| PathBuf::from("."));
    dir.join("settings.json")
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    let path = config_path(&app);
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&data) {
            return settings;
        }
    }
    Settings::default()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = config_path(&app);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let data = serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, data).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}
