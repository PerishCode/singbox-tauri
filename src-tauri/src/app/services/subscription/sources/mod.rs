pub mod file;
pub mod http;

use super::registry;
use super::types::{SubscriptionPayload, SubscriptionResolved};
use crate::runtime_paths::RuntimePaths;

pub fn resolve_source() -> SubscriptionResolved {
    if let Ok(path) = std::env::var("SINGBOX_TAURI_SUBSCRIPTION_FILE") {
        let trimmed = path.trim().to_string();
        if !trimmed.is_empty() {
            return SubscriptionResolved {
                source_kind: Some(super::types::SubscriptionSourceKind::File),
                id: Some("debug-file".to_string()),
                label: "Debug File Override".to_string(),
                source_scope: "debugOverride".to_string(),
                source_profile: infer_profile_name(&trimmed),
                source_url: None,
                source_path: Some(trimmed),
                adapter_kind: super::types::SubscriptionAdapterKind::SingboxRaw,
                entry_type: super::types::SubscriptionEntryType::EncryptedArtifact,
            };
        }
    }

    if let Ok(url) = std::env::var("SINGBOX_TAURI_SUBSCRIPTION_URL") {
        let trimmed = url.trim().to_string();
        if !trimmed.is_empty() {
            return SubscriptionResolved {
                source_kind: Some(super::types::SubscriptionSourceKind::Http),
                id: Some("debug-http".to_string()),
                label: "Debug HTTP Override".to_string(),
                source_scope: "debugOverride".to_string(),
                source_profile: infer_profile_name(&trimmed),
                source_url: Some(trimmed),
                source_path: None,
                adapter_kind: super::types::SubscriptionAdapterKind::SingboxRaw,
                entry_type: super::types::SubscriptionEntryType::EncryptedArtifact,
            };
        }
    }

    let entry = registry::current_entry();

    SubscriptionResolved {
        source_kind: Some(entry.source_kind.clone()),
        id: Some(entry.id.to_string()),
        label: entry.label.to_string(),
        source_scope: "registry".to_string(),
        source_profile: Some(entry.id.to_string()),
        source_url: Some(entry.source_url.to_string()),
        source_path: None,
        adapter_kind: entry.adapter_kind,
        entry_type: entry.entry_type,
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
