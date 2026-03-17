use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use serde::Serialize;
use utoipa::ToSchema;

use crate::app::services::subscription::runtime_config_source_path;
use crate::app::state::{AppLifecycle, AppRunMode, AppStateSnapshot};
use crate::runtime_paths::RuntimePaths;
use crate::singbox::process::ProcessStatus;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SingboxCheck {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SingboxBootstrapReport {
    pub binary_path: String,
    pub log_path: String,
    pub pid_path: String,
    pub session_path: String,
    pub process_status: ProcessStatus,
    pub version: Option<String>,
    pub checks: Vec<SingboxCheck>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SingboxRuntimeStatus {
    pub binary_path: String,
    pub config_path: String,
    pub log_path: String,
    pub pid_path: String,
    pub session_path: String,
    pub process_status: ProcessStatus,
    pub lifecycle: AppLifecycle,
    pub mode: AppRunMode,
    pub pid: Option<u32>,
    pub version: Option<String>,
}

#[derive(Debug, Default)]
pub struct SingboxService;

impl SingboxService {
    pub fn new() -> Self {
        Self
    }

    pub fn bootstrap(&self, paths: &RuntimePaths) -> Result<SingboxBootstrapReport, String> {
        let binary_path = paths.bin_dir.join("sing-box");
        let log_path = paths.logs_dir.join("sing-box.log");
        let app_log_path = paths.logs_dir.join("app.log");
        let pid_path = paths.state_dir.join("sing-box.pid");
        let session_path = paths.state_dir.join("session.json");
        let state_probe_path = paths.state_dir.join(".write-test");

        if !log_path.exists() {
            fs::write(&log_path, "")
                .map_err(|err| format!("failed to initialize {}: {err}", log_path.display()))?;
        }
        ensure_log_file(&app_log_path)?;

        let mut checks = Vec::new();

        checks.push(SingboxCheck {
            name: "binaryExists",
            ok: binary_path.is_file(),
            detail: binary_path.display().to_string(),
        });

        checks.push(SingboxCheck {
            name: "stateDirWritable",
            ok: write_state_probe(&state_probe_path).is_ok(),
            detail: session_path.display().to_string(),
        });

        let mut version = None;
        if binary_path.is_file() {
            match Command::new(&binary_path).arg("version").output() {
                Ok(output) if output.status.success() => {
                    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    version = Some(text.clone());
                    checks.push(SingboxCheck {
                        name: "binaryVersion",
                        ok: true,
                        detail: text,
                    });
                }
                Ok(output) => checks.push(SingboxCheck {
                    name: "binaryVersion",
                    ok: false,
                    detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                }),
                Err(err) => checks.push(SingboxCheck {
                    name: "binaryVersion",
                    ok: false,
                    detail: err.to_string(),
                }),
            }

            match Command::new(&binary_path).arg("-h").output() {
                Ok(output) => checks.push(SingboxCheck {
                    name: "binaryHelp",
                    ok: output.status.success(),
                    detail: format!("exit={}", output.status),
                }),
                Err(err) => checks.push(SingboxCheck {
                    name: "binaryHelp",
                    ok: false,
                    detail: err.to_string(),
                }),
            }
        }

        let process_status = current_process_status(&pid_path);

        if matches!(process_status, ProcessStatus::Stopped) && pid_path.exists() {
            let _ = fs::remove_file(&pid_path);
        }

        let _ = append_app_log(&app_log_path, "bootstrap completed");

        Ok(SingboxBootstrapReport {
            binary_path: binary_path.display().to_string(),
            log_path: log_path.display().to_string(),
            pid_path: pid_path.display().to_string(),
            session_path: session_path.display().to_string(),
            process_status,
            version,
            checks,
        })
    }

    pub fn status(&self, paths: &RuntimePaths) -> Result<SingboxRuntimeStatus, String> {
        let binary_path = paths.bin_dir.join("sing-box");
        let config_path = runtime_config_path(paths);
        let log_path = paths.logs_dir.join("sing-box.log");
        let app_log_path = paths.logs_dir.join("app.log");
        let pid_path = paths.state_dir.join("sing-box.pid");
        let session_path = paths.state_dir.join("session.json");

        ensure_log_file(&log_path)?;
        ensure_log_file(&app_log_path)?;
        let version = binary_version(&binary_path);
        let process_status = current_process_status(&pid_path);
        let mut snapshot = read_session_snapshot(&session_path)?;

        if matches!(process_status, ProcessStatus::Stopped) {
            snapshot.lifecycle = AppLifecycle::Stopped;
            snapshot.pid = None;
            write_session_snapshot(&session_path, &snapshot)?;
            if pid_path.exists() {
                let _ = fs::remove_file(&pid_path);
            }
        }

        let _ = append_app_log(
            &app_log_path,
            &format!(
                "status lifecycle={:?} process={:?} pid={:?}",
                snapshot.lifecycle, process_status, snapshot.pid
            ),
        );

        Ok(SingboxRuntimeStatus {
            binary_path: binary_path.display().to_string(),
            config_path: config_path.display().to_string(),
            log_path: log_path.display().to_string(),
            pid_path: pid_path.display().to_string(),
            session_path: session_path.display().to_string(),
            process_status,
            lifecycle: snapshot.lifecycle,
            mode: snapshot.mode,
            pid: snapshot.pid,
            version,
        })
    }

    pub fn start(&self, paths: &RuntimePaths) -> Result<SingboxRuntimeStatus, String> {
        let current = self.status(paths)?;
        if matches!(current.process_status, ProcessStatus::Running) {
            return Ok(current);
        }

        let binary_path = PathBuf::from(&current.binary_path);
        let config_path = PathBuf::from(&current.config_path);
        let log_path = PathBuf::from(&current.log_path);
        let app_log_path = paths.logs_dir.join("app.log");
        let pid_path = PathBuf::from(&current.pid_path);
        let session_path = PathBuf::from(&current.session_path);

        if !binary_path.is_file() {
            return Err(format!(
                "sing-box binary missing: {}",
                binary_path.display()
            ));
        }

        write_runtime_config(paths, &config_path)?;
        ensure_log_file(&log_path)?;
        ensure_log_file(&app_log_path)?;

        let mut snapshot = read_session_snapshot(&session_path)?;
        snapshot.lifecycle = AppLifecycle::Starting;
        snapshot.mode = AppRunMode::Passive;
        snapshot.pid = None;
        write_session_snapshot(&session_path, &snapshot)?;
        let _ = append_app_log(&app_log_path, "start requested");

        let stdout = append_log_file(&log_path)?;
        let stderr = append_log_file(&log_path)?;
        let child = Command::new(&binary_path)
            .arg("run")
            .arg("-c")
            .arg(&config_path)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|err| format!("failed to start sing-box: {err}"))?;

        let pid = child.id();
        fs::write(&pid_path, format!("{pid}\n"))
            .map_err(|err| format!("failed to write {}: {err}", pid_path.display()))?;

        thread::sleep(Duration::from_millis(400));
        let process_status = current_process_status(&pid_path);
        if !matches!(process_status, ProcessStatus::Running) {
            snapshot.lifecycle = AppLifecycle::Stopped;
            snapshot.pid = None;
            write_session_snapshot(&session_path, &snapshot)?;
            let _ = fs::remove_file(&pid_path);
            let _ = append_app_log(&app_log_path, "start failed: process exited during startup");
            return Err("sing-box exited during startup".to_string());
        }

        snapshot.lifecycle = AppLifecycle::RunningPassive;
        snapshot.pid = Some(pid);
        write_session_snapshot(&session_path, &snapshot)?;
        let _ = append_app_log(&app_log_path, &format!("start succeeded pid={pid}"));

        self.status(paths)
    }

    pub fn stop(&self, paths: &RuntimePaths) -> Result<SingboxRuntimeStatus, String> {
        let current = self.status(paths)?;
        let app_log_path = paths.logs_dir.join("app.log");
        let pid_path = PathBuf::from(&current.pid_path);
        let session_path = PathBuf::from(&current.session_path);
        let mut snapshot = read_session_snapshot(&session_path)?;

        snapshot.lifecycle = AppLifecycle::Stopping;
        write_session_snapshot(&session_path, &snapshot)?;
        let _ = append_app_log(
            &app_log_path,
            &format!("stop requested pid={:?}", current.pid),
        );

        if let Some(pid) = current.pid {
            let pid_string = pid.to_string();
            let _ = Command::new("kill").args(["-TERM", &pid_string]).status();

            for _ in 0..20 {
                if !matches!(current_process_status(&pid_path), ProcessStatus::Running) {
                    break;
                }
                thread::sleep(Duration::from_millis(150));
            }

            if matches!(current_process_status(&pid_path), ProcessStatus::Running) {
                let _ = Command::new("kill").args(["-KILL", &pid_string]).status();
                thread::sleep(Duration::from_millis(100));
            }
        }

        if pid_path.exists() {
            let _ = fs::remove_file(&pid_path);
        }

        snapshot.lifecycle = AppLifecycle::Stopped;
        snapshot.pid = None;
        write_session_snapshot(&session_path, &snapshot)?;
        let _ = append_app_log(&app_log_path, "stop completed");

        self.status(paths)
    }

    pub fn restart(&self, paths: &RuntimePaths) -> Result<SingboxRuntimeStatus, String> {
        let _ = append_app_log(&paths.logs_dir.join("app.log"), "restart requested");
        let _ = self.stop(paths)?;
        self.start(paths)
    }

    pub fn append_app_event(&self, paths: &RuntimePaths, message: &str) -> Result<(), String> {
        let app_log_path = paths.logs_dir.join("app.log");
        ensure_log_file(&app_log_path)?;
        append_app_log(&app_log_path, message)
    }
}

fn runtime_config_path(paths: &RuntimePaths) -> PathBuf {
    paths.config_dir.join("runtime.json")
}

fn write_runtime_config(paths: &RuntimePaths, config_path: &Path) -> Result<(), String> {
    let subscription_config_path = runtime_config_source_path(paths);
    if subscription_config_path.is_file() {
        fs::copy(&subscription_config_path, config_path).map_err(|err| {
            format!(
                "failed to copy {} -> {}: {err}",
                subscription_config_path.display(),
                config_path.display()
            )
        })?;
        return Ok(());
    }

    let config = serde_json::json!({
        "log": {
            "level": "info",
            "timestamp": true
        },
        "outbounds": [
            {
                "type": "direct",
                "tag": "direct"
            }
        ]
    });

    let body = serde_json::to_string_pretty(&config)
        .map_err(|err| format!("failed to serialize passive config: {err}"))?;
    fs::write(config_path, format!("{body}\n"))
        .map_err(|err| format!("failed to write {}: {err}", config_path.display()))
}

fn ensure_log_file(log_path: &Path) -> Result<(), String> {
    if !log_path.exists() {
        fs::write(log_path, "")
            .map_err(|err| format!("failed to initialize {}: {err}", log_path.display()))?;
    }
    Ok(())
}

fn append_log_file(log_path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|err| format!("failed to open {}: {err}", log_path.display()))
}

fn append_app_log(log_path: &Path, message: &str) -> Result<(), String> {
    let mut file = append_log_file(log_path)?;
    use std::io::Write;
    writeln!(file, "{message}")
        .map_err(|err| format!("failed to write {}: {err}", log_path.display()))
}

fn binary_version(binary_path: &Path) -> Option<String> {
    if !binary_path.is_file() {
        return None;
    }

    match Command::new(binary_path).arg("version").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    }
}

fn read_session_snapshot(session_path: &Path) -> Result<AppStateSnapshot, String> {
    if !session_path.exists() {
        let snapshot = AppStateSnapshot::default();
        write_session_snapshot(session_path, &snapshot)?;
        return Ok(snapshot);
    }

    let content = fs::read_to_string(session_path)
        .map_err(|err| format!("failed to read {}: {err}", session_path.display()))?;

    serde_json::from_str(&content)
        .map_err(|err| format!("failed to parse {}: {err}", session_path.display()))
}

fn write_session_snapshot(session_path: &Path, snapshot: &AppStateSnapshot) -> Result<(), String> {
    let body = serde_json::to_string_pretty(snapshot)
        .map_err(|err| format!("failed to serialize session snapshot: {err}"))?;
    fs::write(session_path, format!("{body}\n"))
        .map_err(|err| format!("failed to write {}: {err}", session_path.display()))
}

fn write_state_probe(probe_path: &Path) -> Result<(), String> {
    fs::write(probe_path, "ok\n")
        .map_err(|err| format!("failed to write {}: {err}", probe_path.display()))?;
    fs::remove_file(probe_path)
        .map_err(|err| format!("failed to remove {}: {err}", probe_path.display()))
}

fn current_process_status(pid_path: &std::path::Path) -> ProcessStatus {
    let Ok(pid) = fs::read_to_string(pid_path) else {
        return ProcessStatus::Stopped;
    };

    let pid = pid.trim();
    if pid.is_empty() {
        return ProcessStatus::Stopped;
    }

    match Command::new("kill").args(["-0", pid]).status() {
        Ok(status) if status.success() => ProcessStatus::Running,
        _ => ProcessStatus::Stopped,
    }
}
