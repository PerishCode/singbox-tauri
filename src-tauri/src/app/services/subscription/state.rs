use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtime_paths::RuntimePaths;

use super::types::SubscriptionStateFile;

pub fn state_file_path(paths: &RuntimePaths) -> PathBuf {
    paths.state_dir.join("subscription-state.json")
}

pub fn read_state_file(paths: &RuntimePaths) -> SubscriptionStateFile {
    let path = state_file_path(paths);
    fs::read_to_string(path)
        .ok()
        .and_then(|value| serde_json::from_str::<SubscriptionStateFile>(&value).ok())
        .unwrap_or_default()
}

pub fn write_state_file(paths: &RuntimePaths, state: &SubscriptionStateFile) -> Result<(), String> {
    let path = state_file_path(paths);
    let body = serde_json::to_string_pretty(state)
        .map_err(|err| format!("failed to serialize subscription state: {err}"))?;
    fs::write(&path, ensure_trailing_newline(&body))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

pub fn persist_refresh_error(
    paths: &RuntimePaths,
    state: &SubscriptionStateFile,
    error: String,
) -> String {
    let mut next = state.clone();
    next.last_error = Some(error.clone());
    let _ = write_state_file(paths, &next);
    error
}

pub fn clear_last_error(paths: &RuntimePaths) {
    let mut state = read_state_file(paths);
    state.last_error = None;
    let _ = write_state_file(paths, &state);
}

pub fn write_last_error(paths: &RuntimePaths, error: &str) -> Result<(), String> {
    let mut state = read_state_file(paths);
    state.last_error = Some(error.trim().to_string());
    write_state_file(paths, &state)
}

pub fn now_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => format!("{}", value.as_secs()),
        Err(_) => "0".to_string(),
    }
}

pub fn ensure_trailing_newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_string()
    } else {
        format!("{value}\n")
    }
}
