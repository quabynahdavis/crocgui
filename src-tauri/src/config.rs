use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub relay: String,
    pub curve: String,
    pub disable_compression: bool,
    pub output_dir: String,
    pub theme: String,
    pub autostart: bool,
    pub minimize_to_tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            relay: String::new(),
            curve: "p256".into(),
            disable_compression: false,
            output_dir: String::new(),
            theme: "system".into(),
            autostart: false,
            minimize_to_tray: true,
        }
    }
}

pub struct SettingsState {
    pub settings: std::sync::Mutex<Option<Settings>>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            settings: std::sync::Mutex::new(None),
        }
    }
}

pub fn read_settings(app: &AppHandle) -> Settings {
    cached_settings(app)
}

fn cached_settings(app: &AppHandle) -> Settings {
    let state = app.state::<SettingsState>();
    let mut guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(settings) = guard.as_ref() {
        return settings.clone();
    }
    let settings = read_settings_from_path(&config_path(app));
    *guard = Some(settings.clone());
    settings
}

pub fn read_settings_from_path(path: &PathBuf) -> Settings {
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str(&data) {
            return settings;
        }
    }
    Settings::default()
}

fn config_path(app: &AppHandle) -> PathBuf {
    let dir = match app.path().app_config_dir() {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Failed to get config dir, using temp dir: {}", e);
            std::env::temp_dir().join("croc-gui")
        }
    };
    dir.join("settings.json")
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    cached_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = config_path(&app);
    save_settings_to_path(&path, settings.clone())?;
    let state = app.state::<SettingsState>();
    let mut guard = state.settings.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(settings);
    Ok(())
}

pub fn save_settings_to_path(path: &PathBuf, settings: Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let data = serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(path, data).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("croc-gui-config-test-{}-{}", name, std::process::id()));
        p
    }

    mod defaults {
        use super::*;

        #[test]
        fn default_relay_is_empty() {
            let s = Settings::default();
            assert_eq!(s.relay, "");
        }

        #[test]
        fn default_curve_is_p256() {
            let s = Settings::default();
            assert_eq!(s.curve, "p256");
        }

        #[test]
        fn default_compression_enabled() {
            let s = Settings::default();
            assert!(!s.disable_compression);
        }

        #[test]
        fn default_theme_is_system() {
            let s = Settings::default();
            assert_eq!(s.theme, "system");
        }

        #[test]
        fn default_autostart_disabled() {
            let s = Settings::default();
            assert!(!s.autostart);
        }

        #[test]
        fn default_minimize_to_tray_enabled() {
            let s = Settings::default();
            assert!(s.minimize_to_tray);
        }
    }

    mod read_settings_from_path {
        use super::*;

        #[test]
        fn missing_file_returns_default() {
            let s = read_settings_from_path(&PathBuf::from("/nonexistent/settings.json"));
            assert_eq!(s, Settings::default());
        }

        #[test]
        fn invalid_json_returns_default() {
            let path = temp_path("invalid.json");
            fs::write(&path, "not json").unwrap();
            let s = read_settings_from_path(&path);
            assert_eq!(s, Settings::default());
            let _ = fs::remove_file(&path);
        }

        #[test]
        fn reads_custom_settings() {
            let path = temp_path("custom.json");
            let custom = Settings {
                relay: "my-relay:9009".into(),
                curve: "p521".into(),
                disable_compression: true,
                output_dir: "/home/user/downloads".into(),
                theme: "dark".into(),
                autostart: true,
                minimize_to_tray: false,
            };
            save_settings_to_path(&path, custom.clone()).unwrap();
            let loaded = read_settings_from_path(&path);
            assert_eq!(loaded.relay, "my-relay:9009");
            assert_eq!(loaded.curve, "p521");
            assert!(loaded.disable_compression);
            assert_eq!(loaded.output_dir, "/home/user/downloads");
            assert_eq!(loaded.theme, "dark");
            assert!(loaded.autostart);
            assert!(!loaded.minimize_to_tray);
            let _ = fs::remove_file(&path);
        }
    }

    mod save_and_load_roundtrip {
        use super::*;

        #[test]
        fn roundtrip_preserves_all_fields() {
            let path = temp_path("roundtrip.json");
            let original = Settings {
                relay: "r".into(),
                curve: "ed25519".into(),
                disable_compression: true,
                output_dir: "/tmp/out".into(),
                theme: "light".into(),
                autostart: true,
                minimize_to_tray: false,
            };
            save_settings_to_path(&path, original.clone()).unwrap();
            let loaded = read_settings_from_path(&path);
            assert_eq!(loaded.relay, original.relay);
            assert_eq!(loaded.curve, original.curve);
            assert_eq!(loaded.disable_compression, original.disable_compression);
            assert_eq!(loaded.output_dir, original.output_dir);
            assert_eq!(loaded.theme, original.theme);
            assert_eq!(loaded.autostart, original.autostart);
            assert_eq!(loaded.minimize_to_tray, original.minimize_to_tray);
            let _ = fs::remove_file(&path);
        }

        #[test]
        fn creates_parent_directories() {
            let mut path = std::env::temp_dir();
            path.push(format!("croc-gui-nested-{}", std::process::id()));
            path.push("deep");
            path.push("settings.json");
            let s = Settings::default();
            save_settings_to_path(&path, s).unwrap();
            assert!(path.exists());
            let _ = fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn serializes_as_snake_case() {
            let s = Settings::default();
            let json = serde_json::to_string(&s).unwrap();
            assert!(json.contains("\"disable_compression\""));
            assert!(json.contains("\"output_dir\""));
            assert!(json.contains("\"minimize_to_tray\""));
        }
    }

    mod settings_state {
        use super::*;

        #[test]
        fn new_starts_unpopulated() {
            let state = SettingsState::new();
            let guard = state.settings.lock().unwrap();
            assert!(guard.is_none());
        }

        #[test]
        fn cache_can_be_populated() {
            let state = SettingsState::new();
            {
                let mut guard = state.settings.lock().unwrap();
                *guard = Some(Settings::default());
            }
            let guard = state.settings.lock().unwrap();
            assert_eq!(*guard, Some(Settings::default()));
        }
    }
}
