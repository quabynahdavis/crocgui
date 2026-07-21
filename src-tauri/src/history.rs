use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    Send,
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub id: String,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub files: Vec<String>,
    pub code: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub relay: Option<String>,
    pub curve: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistory {
    pub transfers: Vec<TransferRecord>,
}

impl TransferHistory {
    pub fn new() -> Self {
        Self {
            transfers: Vec::new(),
        }
    }
}

fn history_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("history.json")
}

pub fn load_history(app: &AppHandle) -> TransferHistory {
    let path = history_path(app);
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(history) = serde_json::from_str(&data) {
            return history;
        }
    }
    TransferHistory::new()
}

pub fn save_history(app: &AppHandle, history: &TransferHistory) {
    let path = history_path(app);
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(data) = serde_json::to_string_pretty(history) {
        let _ = fs::write(&path, data);
    }
}

pub fn add_record(app: &AppHandle, record: TransferRecord) {
    let mut history = load_history(app);
    history.transfers.push(record);
    save_history(app, &history);
}

pub fn update_status(app: &AppHandle, id: &str, new_status: TransferStatus, error: Option<String>) {
    let mut history = load_history(app);
    if let Some(record) = history.transfers.iter_mut().find(|r| r.id == id) {
        let terminal = new_status.is_terminal();
        record.status = new_status;
        if terminal {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            record.completed_at = Some(now.to_string());
        }
        record.error = error;
    }
    save_history(app, &history);
}

impl TransferStatus {
    fn is_terminal(&self) -> bool {
        matches!(self, TransferStatus::Completed | TransferStatus::Failed | TransferStatus::Cancelled)
    }
}

pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("tx{}", nanos)
}

pub fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

#[tauri::command]
pub fn get_transfer_history(app: AppHandle) -> TransferHistory {
    load_history(&app)
}

#[tauri::command]
pub fn clear_transfer_history(app: AppHandle) -> Result<(), String> {
    save_history(&app, &TransferHistory::new());
    Ok(())
}

#[tauri::command]
pub fn set_record_pinned(app: AppHandle, id: String, pinned: bool) -> Result<(), String> {
    let mut history = load_history(&app);
    if let Some(record) = history.transfers.iter_mut().find(|r| r.id == id) {
        record.pinned = pinned;
        save_history(&app, &history);
        Ok(())
    } else {
        Err("Record not found".into())
    }
}

#[tauri::command]
pub fn delete_transfer_record(app: AppHandle, id: String) -> Result<(), String> {
    let mut history = load_history(&app);
    let len = history.transfers.len();
    history.transfers.retain(|r| r.id != id);
    if history.transfers.len() < len {
        save_history(&app, &history);
        Ok(())
    } else {
        Err("Record not found".into())
    }
}

#[tauri::command]
pub fn delete_record_files(app: AppHandle, id: String) -> Result<(), String> {
    let history = load_history(&app);
    let record = history.transfers.iter().find(|r| r.id == id).ok_or("Record not found")?;
    if !matches!(record.direction, TransferDirection::Send) {
        return Err("Can only delete files for sent transfers".into());
    }
    for path in &record.files {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir_all(path);
    }
    Ok(())
}
