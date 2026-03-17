pub mod file;
pub mod http;

use super::types::{SubscriptionPayload, SubscriptionResolved};
use crate::runtime_paths::RuntimePaths;

pub fn resolve_source() -> SubscriptionResolved {
    if let Ok(path) = std::env::var("SINGBOX_TAURI_SUBSCRIPTION_FILE") {
        let trimmed = path.trim().to_string();
        if !trimmed.is_empty() {
            return SubscriptionResolved {
                source_kind: Some(super::types::SubscriptionSourceKind::File),
                source_profile: infer_profile_name(&trimmed),
                source_url: None,
                source_path: Some(trimmed),
            };
        }
    }

    if let Ok(url) = std::env::var("SINGBOX_TAURI_SUBSCRIPTION_URL") {
        let trimmed = url.trim().to_string();
        if !trimmed.is_empty() {
            return SubscriptionResolved {
                source_kind: Some(super::types::SubscriptionSourceKind::Http),
                source_profile: infer_profile_name(&trimmed),
                source_url: Some(trimmed),
                source_path: None,
            };
        }
    }

    SubscriptionResolved {
        source_kind: None,
        source_profile: None,
        source_url: None,
        source_path: None,
    }
}

pub fn fetch(paths: &RuntimePaths) -> Result<Option<SubscriptionPayload>, String> {
    let resolved = resolve_source();
    match resolved.source_kind {
        Some(super::types::SubscriptionSourceKind::File) => {
            let path = resolved
                .source_path
                .clone()
                .ok_or_else(|| "missing subscription file path".to_string())?;
            Ok(Some(file::fetch(paths, &path)?))
        }
        Some(super::types::SubscriptionSourceKind::Http) => {
            let url = resolved
                .source_url
                .clone()
                .ok_or_else(|| "missing subscription URL".to_string())?;
            Ok(Some(http::fetch(&url)?))
        }
        None => Ok(None),
    }
}

fn infer_profile_name(value: &str) -> Option<String> {
    let candidate = value.rsplit('/').next().unwrap_or(value).trim();
    let candidate = candidate.strip_suffix(".age").unwrap_or(candidate);
    let candidate = candidate.strip_suffix(".json").unwrap_or(candidate);
    if candidate.is_empty() {
        None
    } else {
        Some(candidate.to_string())
    }
}
