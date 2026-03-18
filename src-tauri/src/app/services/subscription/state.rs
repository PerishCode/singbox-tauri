use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::paths::SubscriptionPaths;
use super::types::SubscriptionStateFile;

pub fn read_state_file(paths: &SubscriptionPaths) -> SubscriptionStateFile {
    fs::read_to_string(&paths.state_path)
        .ok()
        .and_then(|value| serde_json::from_str::<SubscriptionStateFile>(&value).ok())
        .unwrap_or_default()
}

pub fn write_state_file(
    paths: &SubscriptionPaths,
    state: &SubscriptionStateFile,
) -> Result<(), String> {
    let body = serde_json::to_string_pretty(state)
        .map_err(|err| format!("failed to serialize subscription state: {err}"))?;
    fs::write(&paths.state_path, ensure_trailing_newline(&body))
        .map_err(|err| format!("failed to write {}: {err}", paths.state_path.display()))
}

pub fn persist_refresh_error(
    paths: &SubscriptionPaths,
    state: &SubscriptionStateFile,
    error: String,
) -> String {
    let mut next = state.clone();
    next.last_error = Some(error.clone());
    let _ = write_state_file(paths, &next);
    error
}

pub fn clear_last_error(paths: &SubscriptionPaths) {
    let mut state = read_state_file(paths);
    state.last_error = None;
    let _ = write_state_file(paths, &state);
}

pub fn write_last_error(paths: &SubscriptionPaths, error: &str) -> Result<(), String> {
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
