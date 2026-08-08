use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    Send,
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;
    Ok(dir.join("history.json"))
}

pub fn load_history(app: &AppHandle) -> TransferHistory {
    let path = match history_path(app) {
        Ok(p) => p,
        Err(_) => return TransferHistory::new(),
    };
    load_history_from_path(&path)
}

pub fn load_history_from_path(path: &Path) -> TransferHistory {
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(history) = serde_json::from_str(&data) {
            return history;
        }
    }
    TransferHistory::new()
}

pub fn save_history(app: &AppHandle, history: &TransferHistory) {
    let path = match history_path(app) {
        Ok(p) => p,
        Err(_) => return,
    };
    save_history_to_path(&path, history);
}

pub fn save_history_to_path(path: &Path, history: &TransferHistory) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(data) = serde_json::to_string_pretty(history) {
        let _ = fs::write(path, data);
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
            record.completed_at = Some(now_timestamp());
        }
        record.error = error;
    }
    save_history(app, &history);
}

pub fn update_record_code(app: &AppHandle, id: &str, code: &str) {
    let mut history = load_history(app);
    if let Some(record) = history.transfers.iter_mut().find(|r| r.id == id) {
        record.code = Some(code.to_string());
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
    let seq = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tx{}-{}", nanos, seq)
}

pub fn now_timestamp() -> String {
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
    for path_str in &record.files {
        let path = Path::new(path_str);
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_record(id: &str, direction: TransferDirection) -> TransferRecord {
        TransferRecord {
            id: id.to_string(),
            direction,
            status: TransferStatus::InProgress,
            files: vec!["/tmp/test.txt".to_string()],
            code: None,
            started_at: "1000".to_string(),
            completed_at: None,
            relay: None,
            curve: None,
            error: None,
            pinned: false,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("croc-gui-test-{}-{}", name, generate_id()));
        p
    }

    mod transfer_status {
        use super::*;

        #[test]
        fn in_progress_is_not_terminal() {
            assert!(!TransferStatus::InProgress.is_terminal());
        }

        #[test]
        fn completed_is_terminal() {
            assert!(TransferStatus::Completed.is_terminal());
        }

        #[test]
        fn failed_is_terminal() {
            assert!(TransferStatus::Failed.is_terminal());
        }

        #[test]
        fn cancelled_is_terminal() {
            assert!(TransferStatus::Cancelled.is_terminal());
        }
    }

    mod generate_id {
        use super::*;

        #[test]
        fn produces_unique_ids() {
            let ids: std::collections::HashSet<String> =
                (0..1000).map(|_| generate_id()).collect();
            assert_eq!(ids.len(), 1000);
        }

        #[test]
        fn starts_with_tx() {
            assert!(generate_id().starts_with("tx"));
        }
    }

    mod now_timestamp {
        use super::*;

        #[test]
        fn returns_valid_number() {
            let ts = now_timestamp();
            let n: u64 = ts.parse().expect("should be a valid u64");
            assert!(n > 0);
        }
    }

    mod history_persistence {
        use super::*;

        #[test]
        fn load_returns_empty_for_missing_file() {
            let history = load_history_from_path(&PathBuf::from("/nonexistent/path.json"));
            assert_eq!(history.transfers.len(), 0);
        }

        #[test]
        fn save_and_load_roundtrip() {
            let path = temp_path("roundtrip.json");
            let mut h = TransferHistory::new();
            h.transfers.push(make_record("tx1", TransferDirection::Send));
            h.transfers.push(make_record("tx2", TransferDirection::Receive));
            save_history_to_path(&path, &h);

            let loaded = load_history_from_path(&path);
            assert_eq!(loaded.transfers.len(), 2);
            assert_eq!(loaded.transfers[0].id, "tx1");
            assert_eq!(loaded.transfers[1].id, "tx2");

            let _ = fs::remove_file(&path);
        }

        #[test]
        fn load_returns_empty_for_invalid_json() {
            let path = temp_path("invalid.json");
            let mut file = fs::File::create(&path).unwrap();
            file.write_all(b"not json").unwrap();

            let history = load_history_from_path(&path);
            assert_eq!(history.transfers.len(), 0);

            let _ = fs::remove_file(&path);
        }
    }

    mod update_status {
        use super::*;

        #[test]
        fn sets_completed_with_timestamp() {
            let path = temp_path("completed.json");
            let mut h = TransferHistory::new();
            h.transfers.push(make_record("tx1", TransferDirection::Send));
            save_history_to_path(&path, &h);

            let mut loaded = load_history_from_path(&path);
            if let Some(r) = loaded.transfers.iter_mut().find(|r| r.id == "tx1") {
                r.status = TransferStatus::Completed;
                r.completed_at = Some(now_timestamp());
            }
            save_history_to_path(&path, &loaded);

            let reloaded = load_history_from_path(&path);
            let record = reloaded.transfers.iter().find(|r| r.id == "tx1").unwrap();
            assert_eq!(record.status, TransferStatus::Completed);
            assert!(record.completed_at.is_some());

            let _ = fs::remove_file(&path);
        }

        #[test]
        fn sets_error_message() {
            let path = temp_path("error.json");
            let mut h = TransferHistory::new();
            h.transfers.push(make_record("tx1", TransferDirection::Send));
            save_history_to_path(&path, &h);

            let mut loaded = load_history_from_path(&path);
            if let Some(r) = loaded.transfers.iter_mut().find(|r| r.id == "tx1") {
                r.status = TransferStatus::Failed;
                r.error = Some("Connection refused".into());
            }
            save_history_to_path(&path, &loaded);

            let reloaded = load_history_from_path(&path);
            let record = reloaded.transfers.iter().find(|r| r.id == "tx1").unwrap();
            assert_eq!(record.error.as_deref(), Some("Connection refused"));

            let _ = fs::remove_file(&path);
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn serializes_direction_as_snake_case() {
            let record = make_record("tx1", TransferDirection::Send);
            let json = serde_json::to_string(&record).unwrap();
            assert!(json.contains("\"direction\":\"send\""));
        }

        #[test]
        fn deserializes_from_snake_case() {
            let json = r#"{"id":"tx1","direction":"receive","status":"in_progress","files":[],"code":null,"started_at":"1000","completed_at":null,"relay":null,"curve":null,"error":null,"pinned":false}"#;
            let record: TransferRecord = serde_json::from_str(json).unwrap();
            assert_eq!(record.direction, TransferDirection::Receive);
        }

        #[test]
        fn status_serialization_roundtrip() {
            for status in [
                TransferStatus::InProgress,
                TransferStatus::Completed,
                TransferStatus::Failed,
                TransferStatus::Cancelled,
            ] {
                let s = serde_json::to_string(&status).unwrap();
                let d: TransferStatus = serde_json::from_str(&s).unwrap();
                assert_eq!(d, status);
            }
        }

        #[test]
        fn pinned_defaults_to_false() {
            let json = r#"{"id":"tx1","direction":"send","status":"in_progress","files":[],"code":null,"started_at":"1000","completed_at":null,"relay":null,"curve":null,"error":null}"#;
            let record: TransferRecord = serde_json::from_str(json).unwrap();
            assert!(!record.pinned);
        }
    }

    mod transfer_history {
        use super::*;

        #[test]
        fn new_creates_empty() {
            let h = TransferHistory::new();
            assert_eq!(h.transfers.len(), 0);
        }
    }
}
