use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::history;

static TRANSFER_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub struct CrocState {
    pub pid: std::sync::Mutex<Option<u32>>,
    pub history_id: std::sync::Mutex<Option<String>>,
}

impl CrocState {
    pub fn new() -> Self {
        Self {
            pid: std::sync::Mutex::new(None),
            history_id: std::sync::Mutex::new(None),
        }
    }
}

fn croc_binary(app: &AppHandle) -> PathBuf {
    let resource_dir = app.path().resource_dir().ok();
    let binary_name = if cfg!(target_os = "windows") {
        "croc.exe"
    } else {
        "croc"
    };

    if let Some(dir) = resource_dir {
        let bundled = dir.join("binaries").join(binary_name);
        if bundled.exists() {
            return bundled;
        }
    }

    PathBuf::from(binary_name)
}

fn croc_not_supported() -> String {
    #[cfg(target_os = "ios")]
    {
        "croc transfers are not supported on iOS".into()
    }
    #[cfg(not(target_os = "ios"))]
    {
        "croc binary not found. Run 'bun run download-croc' or place croc in PATH.".into()
    }
}

pub fn build_base_args(
    relay: Option<&str>,
    curve: Option<&str>,
    disable_compression: bool,
) -> Vec<String> {
    let mut args = vec!["--yes".to_string()];
    if let Some(r) = relay {
        if !r.is_empty() {
            args.push("--relay".to_string());
            args.push(r.to_string());
        }
    }
    if let Some(c) = curve {
        if !c.is_empty() {
            args.push("--curve".to_string());
            args.push(c.to_string());
        }
    }
    if disable_compression {
        args.push("--no-compress".to_string());
    }
    args
}

pub fn sanitize_filename(filename: &str) -> Result<String, String> {
    let sanitized = Path::new(filename)
        .file_name()
        .ok_or("Invalid filename")?
        .to_string_lossy()
        .into_owned();
    if sanitized != filename || filename.contains("..") {
        return Err("Invalid filename".into());
    }
    Ok(sanitized)
}

pub fn extract_code(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if let Some(idx) = lower.find("code is:") {
        let rest = &line[idx + "code is:".len()..];
        let extracted = rest.trim().to_string();
        if !extracted.is_empty() {
            return Some(extracted);
        }
    }
    None
}

fn spawn_and_monitor(
    app: AppHandle,
    mut cmd: Command,
    complete_event: &'static str,
    code_event: bool,
    history_id: Option<String>,
) {
    if let Ok(mut h) = app.state::<CrocState>().history_id.lock() {
        *h = history_id.clone();
    }

    std::thread::spawn(move || {
        let mut child = match cmd.stderr(Stdio::piped()).stdout(Stdio::null()).spawn() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to spawn croc: {}", e);
                if let Some(id) = &history_id {
                    history::update_status(&app, id, history::TransferStatus::Failed, Some(format!("Failed to start croc: {}", e)));
                }
                let _ = app.emit("croc-error", format!("Failed to start croc: {}", e));
                TRANSFER_IN_PROGRESS.store(false, Ordering::Relaxed);
                return;
            }
        };
        log::info!("croc process spawned (pid: {})", child.id());

        if let Ok(mut p) = app.state::<CrocState>().pid.lock() {
            *p = Some(child.id());
        }

        let stderr = child.stderr.take().unwrap();
        let reader = std::io::BufReader::new(stderr);
        let mut code = String::new();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let _ = app.emit("croc-progress", &line);
            if code_event && code.is_empty() {
                if let Some(extracted) = extract_code(&line) {
                    code = extracted;
                    let _ = app.emit("croc-code", &code);
                    if let Some(id) = &history_id {
                        history::update_record_code(&app, id, &code);
                    }
                }
            }
        }

        if let Ok(mut p) = app.state::<CrocState>().pid.lock() {
            *p = None;
        }
        if let Ok(mut h) = app.state::<CrocState>().history_id.lock() {
            *h = None;
        }

        match child.wait() {
            Ok(status) if status.success() => {
                if let Some(id) = &history_id {
                    history::update_status(&app, id, history::TransferStatus::Completed, None);
                }
                log::info!("Transfer completed (id: {:?})", history_id);
                let _ = app.emit(complete_event, if code_event { code } else { String::new() });
                push_notification(&app, "croc-gui", "Transfer complete!");
            }
            _ => {
                if let Some(id) = &history_id {
                    let record = history::load_history(&app)
                        .transfers
                        .iter()
                        .find(|r| r.id == *id)
                        .map(|r| r.status.clone());
                    if record != Some(history::TransferStatus::Cancelled) {
                        history::update_status(&app, id, history::TransferStatus::Failed, Some("Transfer failed".into()));
                        log::error!("Transfer failed (id: {:?})", history_id);
                    } else {
                        log::warn!("Transfer cancelled (id: {:?})", history_id);
                    }
                }
                let _ = app.emit("croc-error", "Transfer failed or cancelled");
                push_notification(&app, "croc-gui", "Transfer failed or was cancelled");
            }
        }
        TRANSFER_IN_PROGRESS.store(false, Ordering::Relaxed);
    });
}

fn push_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification().builder()
        .title(title)
        .body(body)
        .show();
}

fn check_binary(app: &AppHandle) -> Result<PathBuf, String> {
    let binary = croc_binary(app);
    if binary.exists() {
        Ok(binary)
    } else {
        Err(croc_not_supported())
    }
}

#[tauri::command]
pub fn send_file(
    app: AppHandle,
    paths: Vec<String>,
    relay: Option<String>,
    curve: Option<String>,
    disable_compression: Option<bool>,
) -> Result<(), String> {
    if TRANSFER_IN_PROGRESS.load(Ordering::Relaxed) {
        return Err("A transfer is already in progress".into());
    }

    let binary = check_binary(&app)?;

    let id = history::generate_id();
    let record = history::TransferRecord {
        id: id.clone(),
        direction: history::TransferDirection::Send,
        status: history::TransferStatus::InProgress,
        files: paths.clone(),
        code: None,
        started_at: history::now_timestamp(),
        completed_at: None,
        relay: relay.clone(),
        curve: curve.clone(),
        error: None,
        pinned: false,
    };
    history::add_record(&app, record);

    let mut args = build_base_args(relay.as_deref(), curve.as_deref(), disable_compression.unwrap_or(false));
    args.push("send".to_string());
    args.extend(paths.clone());

    let mut cmd = Command::new(&binary);
    cmd.args(&args);

    TRANSFER_IN_PROGRESS.store(true, Ordering::Relaxed);
    spawn_and_monitor(app, cmd, "croc-complete", true, Some(id));
    Ok(())
}

#[tauri::command]
pub fn receive_file(
    app: AppHandle,
    code: String,
    output_dir: Option<String>,
    relay: Option<String>,
    curve: Option<String>,
    disable_compression: Option<bool>,
) -> Result<(), String> {
    if TRANSFER_IN_PROGRESS.load(Ordering::Relaxed) {
        return Err("A transfer is already in progress".into());
    }

    let binary = check_binary(&app)?;

    let id = history::generate_id();
    let record = history::TransferRecord {
        id: id.clone(),
        direction: history::TransferDirection::Receive,
        status: history::TransferStatus::InProgress,
        files: Vec::new(),
        code: Some(code.clone()),
        started_at: history::now_timestamp(),
        completed_at: None,
        relay: relay.clone(),
        curve: curve.clone(),
        error: None,
        pinned: false,
    };
    history::add_record(&app, record);

    let args = build_base_args(relay.as_deref(), curve.as_deref(), disable_compression.unwrap_or(false));

    let mut cmd = Command::new(&binary);
    cmd.args(&args);
    cmd.env("CROC_SECRET", &code);

    if let Some(dir) = output_dir {
        if !dir.is_empty() {
            cmd.current_dir(&dir);
        }
    }

    TRANSFER_IN_PROGRESS.store(true, Ordering::Relaxed);
    spawn_and_monitor(app, cmd, "croc-receive-complete", false, Some(id));
    Ok(())
}

#[tauri::command]
pub fn cancel_transfer(app: AppHandle) -> Result<(), String> {
    let state = app.state::<CrocState>();
    let pid = state.pid.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(hid) = state.history_id.lock().unwrap_or_else(|e| e.into_inner()).take() {
        history::update_status(&app, &hid, history::TransferStatus::Cancelled, Some("Cancelled by user".into()));
    }
    if let Some(pid) = pid {
        log::info!("Cancelling transfer (pid: {})", pid);
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg(format!("-{}", pid))
                .output();
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F", "/T"])
                .output();
        }
        let _ = app.emit("croc-error", "Transfer cancelled");
    }
    TRANSFER_IN_PROGRESS.store(false, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn check_croc_available(app: AppHandle) -> bool {
    check_binary(&app).is_ok()
}

pub fn parse_relay_addr(relay: &str) -> Result<(String, u16), String> {
    let trimmed = relay.trim();
    if trimmed.is_empty() {
        return Err("Invalid format".into());
    }
    let (host, port) = trimmed.rsplit_once(':').ok_or("Invalid format")?;
    if host.is_empty() {
        return Err("Invalid format".into());
    }
    let port: u16 = port.parse().map_err(|_| "Invalid port")?;
    if port == 0 {
        return Err("Invalid port".into());
    }
    Ok((host.to_string(), port))
}

#[tauri::command]
pub fn test_relay(relay: String) -> Result<(), String> {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let (host, port) = parse_relay_addr(&relay)?;

    let addrs: Vec<_> = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("Invalid address: {}", e))?
        .collect();

    if addrs.is_empty() {
        return Err("Invalid address".into());
    }

    let mut last_error = String::from("Connection failed");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            Ok(_) => return Ok(()),
            Err(e) => last_error = format!("Connection failed: {}", e),
        }
    }
    Err(last_error)
}

#[tauri::command]
pub fn save_temp_text(filename: String, content: String) -> Result<String, String> {
    let sanitized = sanitize_filename(&filename)?;
    let dir = std::env::temp_dir().join("croc-gui");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&sanitized);
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod build_base_args {
        use super::*;

        fn args_contain(args: &[String], expected: &str) -> bool {
            args.iter().any(|a| a == expected)
        }

        #[test]
        fn base_yes_flag_always_present() {
            let args = build_base_args(None, None, false);
            assert!(args_contain(&args, "--yes"));
        }

        #[test]
        fn empty_relay_not_added() {
            let args = build_base_args(Some(""), None, false);
            assert!(!args_contain(&args, "--relay"));
        }

        #[test]
        fn relay_added_when_provided() {
            let args = build_base_args(Some("my-relay:9009"), None, false);
            assert!(args_contain(&args, "--relay"));
            assert!(args_contain(&args, "my-relay:9009"));
        }

        #[test]
        fn empty_curve_not_added() {
            let args = build_base_args(None, Some(""), false);
            assert!(!args_contain(&args, "--curve"));
        }

        #[test]
        fn curve_added_when_provided() {
            let args = build_base_args(None, Some("p384"), false);
            assert!(args_contain(&args, "--curve"));
            assert!(args_contain(&args, "p384"));
        }

        #[test]
        fn compression_flag_added_when_disabled() {
            let args = build_base_args(None, None, true);
            assert!(args_contain(&args, "--no-compress"));
        }

        #[test]
        fn compression_flag_absent_when_enabled() {
            let args = build_base_args(None, None, false);
            assert!(!args_contain(&args, "--no-compress"));
        }

        #[test]
        fn all_options_combined() {
            let args = build_base_args(Some("r"), Some("p521"), true);
            assert!(args_contain(&args, "--yes"));
            assert!(args_contain(&args, "--relay"));
            assert!(args_contain(&args, "r"));
            assert!(args_contain(&args, "--curve"));
            assert!(args_contain(&args, "p521"));
            assert!(args_contain(&args, "--no-compress"));
        }
    }

    mod sanitize_filename {
        use super::*;

        #[test]
        fn accepts_simple_filename() {
            assert_eq!(sanitize_filename("note.txt").unwrap(), "note.txt");
        }

        #[test]
        fn accepts_filename_with_dots() {
            assert_eq!(sanitize_filename("my.file.txt").unwrap(), "my.file.txt");
        }

        #[test]
        fn rejects_path_traversal() {
            assert!(sanitize_filename("../../etc/passwd").is_err());
        }

        #[test]
        fn rejects_double_dot() {
            assert!(sanitize_filename("file..txt").is_err());
        }

        #[test]
        fn rejects_directory_prefix() {
            assert!(sanitize_filename("dir/file.txt").is_err());
        }

        #[test]
        fn rejects_absolute_path() {
            assert!(sanitize_filename("/etc/passwd").is_err());
        }

        #[test]
        fn rejects_empty() {
            assert!(sanitize_filename("").is_err());
        }

        #[test]
        fn rejects_path_with_directory_components() {
            assert!(sanitize_filename("path/to/file.txt").is_err());
        }
    }

    mod parse_relay_addr {
        use super::*;

        #[test]
        fn parses_host_and_port() {
            assert_eq!(
                parse_relay_addr("croc.schuermann.io:9009").unwrap(),
                ("croc.schuermann.io".to_string(), 9009)
            );
        }

        #[test]
        fn rejects_missing_port() {
            assert!(parse_relay_addr("croc.schuermann.io").is_err());
        }

        #[test]
        fn rejects_empty() {
            assert!(parse_relay_addr("").is_err());
        }

        #[test]
        fn rejects_non_numeric_port() {
            assert!(parse_relay_addr("host:abc").is_err());
        }

        #[test]
        fn rejects_zero_port() {
            assert!(parse_relay_addr("host:0").is_err());
        }

        #[test]
        fn rejects_missing_host() {
            assert!(parse_relay_addr(":9009").is_err());
        }
    }

    mod extract_code {
        use super::*;

        #[test]
        fn extracts_simple_code() {
            assert_eq!(
                extract_code("Code is: 1234-ABCD-5678"),
                Some("1234-ABCD-5678".to_string())
            );
        }

        #[test]
        fn extracts_case_insensitive() {
            assert_eq!(
                extract_code("code is: abc-def"),
                Some("abc-def".to_string())
            );
        }

        #[test]
        fn extracts_with_extra_whitespace() {
            assert_eq!(
                extract_code("Code is:   1234-ABCD   "),
                Some("1234-ABCD".to_string())
            );
        }

        #[test]
        fn returns_none_without_code() {
            assert_eq!(extract_code("some other output"), None);
        }

        #[test]
        fn returns_none_on_empty_after_colon() {
            assert_eq!(extract_code("Code is:   "), None);
        }

        #[test]
        fn handles_code_in_middle_of_line() {
            assert_eq!(
                extract_code("Sending... Code is: XYZ-123 done"),
                Some("XYZ-123 done".to_string())
            );
        }
    }
}
