use std::fs;

use crate::app::services::subscription::types::{SubscriptionPayload, SubscriptionSourceKind};
use crate::runtime_paths::RuntimePaths;

pub fn fetch(paths: &RuntimePaths, value: &str) -> Result<SubscriptionPayload, String> {
    let path = if value.starts_with('/') {
        std::path::PathBuf::from(value)
    } else {
        paths.root.join(value)
    };
    let bytes =
        fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;

    Ok(SubscriptionPayload {
        source_kind: SubscriptionSourceKind::File,
        source_reference: path.display().to_string(),
        encrypted_bytes: bytes,
    })
}
