use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

pub const RUNTIME_ROOT_ENV: &str = "SINGBOX_TAURI_RUNTIME_ROOT_PATH";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePaths {
    pub mode: RuntimeMode,
    pub root: PathBuf,
    pub bin_dir: PathBuf,
    pub config_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub state_dir: PathBuf,
    pub secrets_dir: PathBuf,
    pub subscriptions_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeMode {
    EnvOverride,
    Dev,
    Production,
}

impl RuntimePaths {
    pub fn resolve(app: &AppHandle) -> Result<Self, String> {
        let (mode, root) = resolve_runtime_root(app)?;
        let paths = Self {
            mode,
            bin_dir: root.join("bin"),
            config_dir: root.join("config"),
            logs_dir: root.join("logs"),
            state_dir: root.join("state"),
            secrets_dir: root.join("secrets"),
            subscriptions_dir: root.join("subscriptions"),
            root,
        };

        paths.ensure_dirs()?;
        Ok(paths)
    }

    fn ensure_dirs(&self) -> Result<(), String> {
        for dir in [
            &self.root,
            &self.bin_dir,
            &self.config_dir,
            &self.logs_dir,
            &self.state_dir,
            &self.secrets_dir,
            &self.subscriptions_dir,
        ] {
            fs::create_dir_all(dir)
                .map_err(|err| format!("failed to create {}: {err}", dir.display()))?;
        }

        Ok(())
    }
}

fn resolve_runtime_root(app: &AppHandle) -> Result<(RuntimeMode, PathBuf), String> {
    if let Some(path) = env_override_root()? {
        return Ok((RuntimeMode::EnvOverride, path));
    }

    if cfg!(debug_assertions) {
        return Ok((RuntimeMode::Dev, dev_runtime_root()));
    }

    let root = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?
        .join("runtime");

    Ok((RuntimeMode::Production, root))
}

fn env_override_root() -> Result<Option<PathBuf>, String> {
    let Some(value) = env::var_os(RUNTIME_ROOT_ENV) else {
        return Ok(None);
    };

    if value.is_empty() {
        return Ok(None);
    }

    let path = PathBuf::from(value);
    let absolute = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .map_err(|err| format!("failed to resolve current dir: {err}"))?
            .join(path)
    };

    Ok(Some(absolute))
}

fn dev_runtime_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("src-tauri parent should exist")
        .join(".runtime")
        .join("dev")
}
