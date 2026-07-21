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

fn run_croc(app: AppHandle, args: Vec<String>, complete_event: &'static str, code_event: Option<&'static str>) {
    let binary = croc_binary(&app);

    std::thread::spawn(move || {
        let mut child = match Command::new(&binary)
            .args(&args)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
        {
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
            if let Some(event) = code_event {
                if (line.contains("Code is:") || line.contains("code is:")) && code.is_empty() {
                    if let Some(c) = line.split(':').nth(1) {
                        code = c.trim().to_string();
                        let _ = app.emit(event, &code);
                    }
                }
            }
        }

        *app.state::<CrocState>().pid.lock().unwrap() = None;

        match child.wait() {
            Ok(status) if status.success() => {
                let _ = app.emit(complete_event, code);
            }
            _ => {
                let _ = app.emit("croc-error", "Transfer failed or cancelled");
            }
        }
    });
}

#[tauri::command]
pub fn send_file(app: AppHandle, path: String) -> Result<(), String> {
    run_croc(
        app,
        vec!["send".into(), "--yes".into(), path],
        "croc-complete",
        Some("croc-code"),
    );
    Ok(())
}

#[tauri::command]
pub fn receive_file(app: AppHandle, code: String) -> Result<(), String> {
    run_croc(
        app,
        vec!["--yes".into(), code],
        "croc-receive-complete",
        None,
    );
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
