use std::fs;
use std::process::Command;

use serde::Serialize;

use crate::runtime_paths::RuntimePaths;
use crate::singbox::process::ProcessStatus;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingboxCheck {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Default)]
pub struct SingboxService;

impl SingboxService {
    pub fn new() -> Self {
        Self
    }

    pub fn bootstrap(&self, paths: &RuntimePaths) -> Result<SingboxBootstrapReport, String> {
        let binary_path = paths.bin_dir.join("sing-box");
        let log_path = paths.logs_dir.join("sing-box.log");
        let pid_path = paths.state_dir.join("sing-box.pid");
        let session_path = paths.state_dir.join("session.json");

        if !log_path.exists() {
            fs::write(&log_path, "")
                .map_err(|err| format!("failed to initialize {}: {err}", log_path.display()))?;
        }

        let mut checks = Vec::new();

        checks.push(SingboxCheck {
            name: "binaryExists",
            ok: binary_path.is_file(),
            detail: binary_path.display().to_string(),
        });

        checks.push(SingboxCheck {
            name: "stateDirWritable",
            ok: fs::write(&session_path, "{\n  \"status\": \"stopped\"\n}\n").is_ok(),
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
