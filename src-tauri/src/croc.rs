use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};

pub struct CrocState {
    pub pid: std::sync::Mutex<Option<u32>>,
}

impl CrocState {
    pub fn new() -> Self {
        Self {
            pid: std::sync::Mutex::new(None),
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

fn build_base_args(
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

fn spawn_and_monitor(
    app: AppHandle,
    mut cmd: Command,
    complete_event: &'static str,
    code_event: bool,
) {
    std::thread::spawn(move || {
        let mut child = match cmd.stderr(Stdio::piped()).stdout(Stdio::null()).spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("croc-error", format!("Failed to start croc: {}", e));
                return;
            }
        };

        *app.state::<CrocState>().pid.lock().unwrap() = Some(child.id());

        let stderr = child.stderr.take().unwrap();
        let reader = std::io::BufReader::new(stderr);
        let mut code = String::new();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let _ = app.emit("croc-progress", &line);
            if code_event && (line.contains("Code is:") || line.contains("code is:")) && code.is_empty() {
                if let Some(c) = line.split(':').nth(1) {
                    code = c.trim().to_string();
                    let _ = app.emit("croc-code", &code);
                }
            }
        }

        *app.state::<CrocState>().pid.lock().unwrap() = None;

        match child.wait() {
            Ok(status) if status.success() => {
                let _ = app.emit(complete_event, if code_event { code } else { String::new() });
            }
            _ => {
                let _ = app.emit("croc-error", "Transfer failed or cancelled");
            }
        }
    });
}

#[tauri::command]
pub fn send_file(
    app: AppHandle,
    paths: Vec<String>,
    relay: Option<String>,
    curve: Option<String>,
    disable_compression: Option<bool>,
) -> Result<(), String> {
    let binary = croc_binary(&app);
    let mut args = build_base_args(relay.as_deref(), curve.as_deref(), disable_compression.unwrap_or(false));
    args.push("send".to_string());
    args.extend(paths);

    let mut cmd = Command::new(&binary);
    cmd.args(&args);

    spawn_and_monitor(app, cmd, "croc-complete", true);
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
    let binary = croc_binary(&app);
    let args = build_base_args(relay.as_deref(), curve.as_deref(), disable_compression.unwrap_or(false));

    let mut cmd = Command::new(&binary);
    cmd.args(&args);
    cmd.env("CROC_SECRET", &code);

    if let Some(dir) = output_dir {
        if !dir.is_empty() {
            cmd.current_dir(&dir);
        }
    }

    spawn_and_monitor(app, cmd, "croc-receive-complete", false);
    Ok(())
}

#[tauri::command]
pub fn cancel_transfer(app: AppHandle) -> Result<(), String> {
    let pid = app.state::<CrocState>().pid.lock().unwrap().take();
    if let Some(pid) = pid {
        #[cfg(unix)]
        {
            let _ = Command::new("kill").arg(pid.to_string()).output();
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }
        let _ = app.emit("croc-error", "Transfer cancelled");
    }
    Ok(())
}
