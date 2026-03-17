use std::fs;
use std::path::PathBuf;

use crate::runtime_paths::RuntimePaths;

use super::adapters::singbox_raw;
use super::registry;
use super::sources;
use super::state::{
    clear_last_error as clear_state_last_error, now_timestamp, persist_refresh_error,
    read_state_file, write_last_error as write_state_last_error, write_state_file,
};
use super::transforms::age;
use super::types::{
    SubscriptionApplyState, SubscriptionArtifacts, SubscriptionDecryptState,
    SubscriptionDefinitionSnapshot, SubscriptionFetchState, SubscriptionKeyState,
    SubscriptionRegistryItem, SubscriptionRuntimeSnapshot, SubscriptionSourceDefinition,
};

#[derive(Debug, Default)]
pub struct SubscriptionService;

impl SubscriptionService {
    pub fn ensure_local_state(&self, paths: &RuntimePaths) -> Result<(), String> {
        let (private_key_path, public_key_path) = key_paths(paths);
        age::ensure_keypair(&private_key_path, &public_key_path)
    }

    pub fn refresh(&self, paths: &RuntimePaths) -> Result<SubscriptionRuntimeSnapshot, String> {
        self.ensure_local_state(paths)?;
        let mut state = read_state_file(paths);
        state.last_attempt_at = Some(now_timestamp());
        state.last_error = None;
        write_state_file(paths, &state)?;

        let Some(payload) =
            sources::fetch(paths).map_err(|err| persist_refresh_error(paths, &state, err))?
        else {
            return Ok(self.runtime_snapshot(paths));
        };

        let artifacts = artifact_paths(paths);
        fs::write(&artifacts.encrypted_path, &payload.encrypted_bytes).map_err(|err| {
            persist_refresh_error(
                paths,
                &state,
                format!(
                    "failed to write {}: {err}",
                    artifacts.encrypted_path.display()
                ),
            )
        })?;

        let identity = age::read_identity(&key_paths(paths).0)
            .map_err(|err| persist_refresh_error(paths, &state, err))?;
        let plaintext = age::decrypt_bytes(&identity, &payload.encrypted_bytes)
            .map_err(|err| persist_refresh_error(paths, &state, err))?;
        let normalized = singbox_raw::validate_and_normalize(&plaintext)
            .map_err(|err| persist_refresh_error(paths, &state, err))?;

        fs::write(&artifacts.decrypted_path, &normalized).map_err(|err| {
            persist_refresh_error(
                paths,
                &state,
                format!(
                    "failed to write {}: {err}",
                    artifacts.decrypted_path.display()
                ),
            )
        })?;
        fs::write(&artifacts.active_config_path, &normalized).map_err(|err| {
            persist_refresh_error(
                paths,
                &state,
                format!(
                    "failed to write {}: {err}",
                    artifacts.active_config_path.display()
                ),
            )
        })?;

        state.last_successful_refresh_at = Some(now_timestamp());
        state.last_error = None;
        write_state_file(paths, &state)?;

        Ok(self.runtime_snapshot(paths))
    }

    pub fn definition_snapshot(&self, _paths: &RuntimePaths) -> SubscriptionDefinitionSnapshot {
        let resolved = sources::resolve_source();

        SubscriptionDefinitionSnapshot {
            id: resolved.id,
            label: resolved.label,
            r#type: resolved.entry_type,
            scope: resolved.source_scope,
            profile: resolved.source_profile,
            adapter: resolved.adapter_kind,
            source: SubscriptionSourceDefinition {
                r#type: resolved.source_kind,
                url: resolved.source_url,
                path: resolved.source_path,
            },
            entries: registry::entries()
                .into_iter()
                .map(|entry| SubscriptionRegistryItem {
                    id: entry.id.to_string(),
                    label: entry.label.to_string(),
                    r#type: entry.entry_type,
                    adapter: entry.adapter_kind,
                    source: SubscriptionSourceDefinition {
                        r#type: Some(entry.source_kind),
                        url: Some(entry.source_url.to_string()),
                        path: None,
                    },
                })
                .collect(),
        }
    }

    pub fn runtime_snapshot(&self, paths: &RuntimePaths) -> SubscriptionRuntimeSnapshot {
        let (private_key_path, public_key_path) = key_paths(paths);
        let artifacts = artifact_paths(paths);
        let public_key = fs::read_to_string(&public_key_path)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let state = read_state_file(paths);
        let resolved = sources::resolve_source();
        let (apply_state, apply_message) = resolve_apply_state(paths, &artifacts);

        let key_state = if private_key_path.is_file() && public_key.is_some() {
            SubscriptionKeyState::Ready
        } else {
            SubscriptionKeyState::Missing
        };
        let fetch_state = if artifacts.encrypted_path.is_file() {
            SubscriptionFetchState::Fetched
        } else if resolved.source_kind.is_some() {
            SubscriptionFetchState::Failed
        } else {
            SubscriptionFetchState::Idle
        };
        let decrypt_state =
            if artifacts.active_config_path.is_file() && artifacts.decrypted_path.is_file() {
                SubscriptionDecryptState::Ready
            } else if artifacts.encrypted_path.is_file() {
                SubscriptionDecryptState::Failed
            } else {
                SubscriptionDecryptState::Idle
            };

        SubscriptionRuntimeSnapshot {
            subscription_id: resolved.id,
            key_state,
            fetch_state,
            decrypt_state,
            private_key_path: private_key_path.display().to_string(),
            public_key_path: public_key_path.display().to_string(),
            encrypted_path: artifacts.encrypted_path.display().to_string(),
            decrypted_path: artifacts.decrypted_path.display().to_string(),
            active_config_path: artifacts.active_config_path.display().to_string(),
            public_key,
            apply_state,
            apply_message,
            last_attempt_at: state.last_attempt_at,
            last_successful_refresh_at: state.last_successful_refresh_at,
            last_error: state.last_error,
        }
    }
}

pub fn runtime_config_source_path(paths: &RuntimePaths) -> PathBuf {
    artifact_paths(paths).active_config_path
}

pub fn clear_last_error(paths: &RuntimePaths) {
    clear_state_last_error(paths);
}

pub fn write_last_error(paths: &RuntimePaths, error: &str) -> Result<(), String> {
    write_state_last_error(paths, error)
}

fn key_paths(paths: &RuntimePaths) -> (PathBuf, PathBuf) {
    (
        paths.secrets_dir.join("subscription.agekey"),
        paths.secrets_dir.join("subscription.pub"),
    )
}

fn artifact_paths(paths: &RuntimePaths) -> SubscriptionArtifacts {
    SubscriptionArtifacts {
        encrypted_path: paths.subscriptions_dir.join("subscription.json.age"),
        decrypted_path: paths.subscriptions_dir.join("subscription.json"),
        active_config_path: paths.subscriptions_dir.join("active-config.json"),
    }
}

fn resolve_apply_state(
    paths: &RuntimePaths,
    artifacts: &SubscriptionArtifacts,
) -> (SubscriptionApplyState, String) {
    let runtime_config_path = paths.config_dir.join("runtime.json");
    if !artifacts.active_config_path.is_file() {
        return (
            SubscriptionApplyState::Unknown,
            "No decrypted subscription config is available yet.".to_string(),
        );
    }
    if !runtime_config_path.is_file() {
        return (
            SubscriptionApplyState::PendingApply,
            "A refreshed subscription config exists, but sing-box has not written an active runtime config yet.".to_string(),
        );
    }

    let active = fs::read(&artifacts.active_config_path).ok();
    let runtime = fs::read(&runtime_config_path).ok();
    match (active, runtime) {
        (Some(active), Some(runtime)) if active == runtime => (
            SubscriptionApplyState::Applied,
            "The latest subscription config matches the runtime config currently handed to sing-box.".to_string(),
        ),
        (Some(_), Some(_)) => (
            SubscriptionApplyState::PendingApply,
            "A newer subscription config is staged locally, but the running sing-box process has not applied it yet.".to_string(),
        ),
        _ => (
            SubscriptionApplyState::Unknown,
            "Could not compare the staged subscription config with the runtime config.".to_string(),
        ),
    }
}
