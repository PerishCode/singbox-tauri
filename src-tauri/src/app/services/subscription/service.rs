use std::fs;
use std::path::PathBuf;

use crate::runtime_paths::RuntimePaths;

use super::adapters::{compose_manifest, singbox_raw};
use super::paths::SubscriptionPaths;
use super::registry;
use super::sources;
use super::state::{
    clear_last_error as clear_state_last_error, now_timestamp, persist_refresh_error,
    read_state_file, write_last_error as write_state_last_error, write_state_file,
};
use super::transforms::{age, compose};
use super::types::{
    SubscriptionApplyState, SubscriptionDecryptState, SubscriptionDefinitionSnapshot,
    SubscriptionFetchState, SubscriptionKeyState, SubscriptionRegistryItem,
    SubscriptionRuntimeSnapshot, SubscriptionSourceDefinition,
};

#[derive(Debug, Default)]
pub struct SubscriptionService;

impl SubscriptionService {
    pub fn ensure_local_state(&self, paths: &RuntimePaths) -> Result<(), String> {
        let scoped = scoped_paths(paths)?;
        age::ensure_keypair(&scoped.private_key_path, &scoped.public_key_path)
    }

    pub fn refresh(&self, paths: &RuntimePaths) -> Result<SubscriptionRuntimeSnapshot, String> {
        self.ensure_local_state(paths)?;
        let scoped = scoped_paths(paths)?;
        let mut state = read_state_file(&scoped);
        state.last_attempt_at = Some(now_timestamp());
        state.last_error = None;
        write_state_file(&scoped, &state)?;

        let Some(payload) =
            sources::fetch(paths).map_err(|err| persist_refresh_error(&scoped, &state, err))?
        else {
            return Ok(self.runtime_snapshot(paths));
        };

        fs::write(&scoped.encrypted_path, &payload.encrypted_bytes).map_err(|err| {
            persist_refresh_error(
                &scoped,
                &state,
                format!("failed to write {}: {err}", scoped.encrypted_path.display()),
            )
        })?;

        let identity = age::read_identity(&scoped.private_key_path)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        let plaintext = age::decrypt_bytes(&identity, &payload.encrypted_bytes)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        let prepared = if compose_manifest::is_manifest(&plaintext) {
            compose_manifest::prepare_plaintext(paths, &scoped, &identity, &payload, &plaintext)
                .map_err(|err| persist_refresh_error(&scoped, &state, err))?
        } else {
            singbox_raw::prepare(&plaintext)
                .map_err(|err| persist_refresh_error(&scoped, &state, err))?
        };
        let composed =
            compose::compose_config(&prepared.compose_input, &prepared.nodes, &prepared.rules)
                .map_err(|err| persist_refresh_error(&scoped, &state, err))?;

        fs::write(&scoped.decrypted_path, &prepared.normalized).map_err(|err| {
            persist_refresh_error(
                &scoped,
                &state,
                format!("failed to write {}: {err}", scoped.decrypted_path.display()),
            )
        })?;
        fs::write(&scoped.active_config_path, &composed).map_err(|err| {
            persist_refresh_error(
                &scoped,
                &state,
                format!(
                    "failed to write {}: {err}",
                    scoped.active_config_path.display()
                ),
            )
        })?;
        write_json_file(&scoped.nodes_path, &prepared.nodes)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        write_json_file(&scoped.rules_path, &prepared.rules)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        write_json_file(&scoped.compose_input_path, &prepared.compose_input)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        write_json_file(&scoped.groups_path, &prepared.groups)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;
        write_json_file(&scoped.source_metadata_path, &prepared.source_metadata)
            .map_err(|err| persist_refresh_error(&scoped, &state, err))?;

        state.last_successful_refresh_at = Some(now_timestamp());
        state.last_error = None;
        write_state_file(&scoped, &state)?;

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
        let scoped = scoped_paths(paths).ok();
        let public_key = scoped
            .as_ref()
            .and_then(|scoped| fs::read_to_string(&scoped.public_key_path).ok())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let state = scoped.as_ref().map(read_state_file).unwrap_or_default();
        let resolved = sources::resolve_source();
        let (apply_state, apply_message) = resolve_apply_state(paths, scoped.as_ref());
        let scoped_root_path = scoped
            .as_ref()
            .map(|scoped| scoped.root.display().to_string())
            .unwrap_or_default();
        let source_metadata_path = scoped
            .as_ref()
            .map(|scoped| scoped.source_metadata_path.display().to_string())
            .unwrap_or_default();

        let private_key_path = scoped
            .as_ref()
            .map(|scoped| scoped.private_key_path.display().to_string())
            .unwrap_or_default();
        let public_key_path = scoped
            .as_ref()
            .map(|scoped| scoped.public_key_path.display().to_string())
            .unwrap_or_default();
        let encrypted_path = scoped
            .as_ref()
            .map(|scoped| scoped.encrypted_path.display().to_string())
            .unwrap_or_default();
        let decrypted_path = scoped
            .as_ref()
            .map(|scoped| scoped.decrypted_path.display().to_string())
            .unwrap_or_default();
        let active_config_path = scoped
            .as_ref()
            .map(|scoped| scoped.active_config_path.display().to_string())
            .unwrap_or_default();
        let nodes_path = scoped
            .as_ref()
            .map(|scoped| scoped.nodes_path.display().to_string())
            .unwrap_or_default();
        let rules_path = scoped
            .as_ref()
            .map(|scoped| scoped.rules_path.display().to_string())
            .unwrap_or_default();
        let compose_input_path = scoped
            .as_ref()
            .map(|scoped| scoped.compose_input_path.display().to_string())
            .unwrap_or_default();
        let groups_path = scoped
            .as_ref()
            .map(|scoped| scoped.groups_path.display().to_string())
            .unwrap_or_default();

        let key_state = if scoped
            .as_ref()
            .is_some_and(|scoped| scoped.private_key_path.is_file())
            && public_key.is_some()
        {
            SubscriptionKeyState::Ready
        } else {
            SubscriptionKeyState::Missing
        };
        let fetch_state = if scoped
            .as_ref()
            .is_some_and(|scoped| scoped.encrypted_path.is_file())
        {
            SubscriptionFetchState::Fetched
        } else if resolved.source_kind.is_some() {
            SubscriptionFetchState::Failed
        } else {
            SubscriptionFetchState::Idle
        };
        let decrypt_state = if scoped.as_ref().is_some_and(|scoped| {
            scoped.active_config_path.is_file() && scoped.decrypted_path.is_file()
        }) {
            SubscriptionDecryptState::Ready
        } else if scoped
            .as_ref()
            .is_some_and(|scoped| scoped.encrypted_path.is_file())
        {
            SubscriptionDecryptState::Failed
        } else {
            SubscriptionDecryptState::Idle
        };

        SubscriptionRuntimeSnapshot {
            subscription_id: resolved.id,
            scoped_root_path,
            source_metadata_path,
            key_state,
            fetch_state,
            decrypt_state,
            private_key_path,
            public_key_path,
            encrypted_path,
            decrypted_path,
            active_config_path,
            nodes_path,
            rules_path,
            compose_input_path,
            groups_path,
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
    scoped_paths(paths)
        .map(|scoped| scoped.active_config_path)
        .unwrap_or_else(|_| {
            paths
                .subscriptions_dir
                .join("_current")
                .join("resources")
                .join("active-config.json")
        })
}

pub fn clear_last_error(paths: &RuntimePaths) {
    if let Ok(scoped) = scoped_paths(paths) {
        clear_state_last_error(&scoped);
    }
}

pub fn write_last_error(paths: &RuntimePaths, error: &str) -> Result<(), String> {
    let scoped = scoped_paths(paths)?;
    write_state_last_error(&scoped, error)
}

fn scoped_paths(paths: &RuntimePaths) -> Result<SubscriptionPaths, String> {
    let resolved = sources::resolve_source();
    let subscription_id = resolved.id.unwrap_or_else(|| "_current".to_string());
    SubscriptionPaths::new(paths, &subscription_id)
}

fn resolve_apply_state(
    paths: &RuntimePaths,
    scoped: Option<&SubscriptionPaths>,
) -> (SubscriptionApplyState, String) {
    let runtime_config_path = paths.config_dir.join("runtime.json");
    let Some(scoped) = scoped else {
        return (
            SubscriptionApplyState::Unknown,
            "No scoped subscription directory is available yet.".to_string(),
        );
    };

    if !scoped.active_config_path.is_file() {
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

    let active = fs::read(&scoped.active_config_path).ok();
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

fn write_json_file<T: serde::Serialize>(path: &std::path::Path, value: &T) -> Result<(), String> {
    let body = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    fs::write(path, format!("{body}\n"))
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_paths::{RuntimeMode, RuntimePaths};
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_runtime_paths() -> RuntimePaths {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("singbox-tauri-refresh-test-{stamp}"));
        for dir in [
            root.join("bin"),
            root.join("config"),
            root.join("logs"),
            root.join("state"),
            root.join("secrets"),
            root.join("subscriptions"),
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
        RuntimePaths {
            mode: RuntimeMode::Dev,
            root: root.clone(),
            bin_dir: root.join("bin"),
            config_dir: root.join("config"),
            logs_dir: root.join("logs"),
            state_dir: root.join("state"),
            secrets_dir: root.join("secrets"),
            subscriptions_dir: root.join("subscriptions"),
        }
    }

    fn write_file(path: &PathBuf, body: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }

    fn encrypt_for(recipient: &::age::x25519::Recipient, plaintext: &str) -> Vec<u8> {
        let encryptor =
            ::age::Encryptor::with_recipients(std::iter::once(recipient as &dyn::age::Recipient))
                .unwrap();
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
        writer.write_all(plaintext.as_bytes()).unwrap();
        writer.finish().unwrap();
        encrypted
    }

    #[test]
    fn refresh_with_manifest_file_override_writes_composed_artifacts() {
        let paths = temp_runtime_paths();
        let service = SubscriptionService;
        std::env::set_var(
            "SINGBOX_TAURI_SUBSCRIPTION_FILE",
            paths
                .root
                .join("fixtures/config.json.age")
                .display()
                .to_string(),
        );
        let scoped = SubscriptionPaths::new(&paths, "debug-file").unwrap();
        service.ensure_local_state(&paths).unwrap();

        let identity = age::read_identity(&scoped.private_key_path).unwrap();
        let recipient = identity.to_public();

        let airport_plain = paths.root.join("fixtures/airport.json");
        let private_plain = paths.root.join("fixtures/private.json");
        let common_rules = paths.root.join("fixtures/common.json");
        let direct_rules = paths.root.join("fixtures/direct.json");
        let airport_age = paths.root.join("fixtures/airport.json.age");
        let private_age = paths.root.join("fixtures/private.json.age");
        let manifest_age = paths.root.join("fixtures/config.json.age");

        write_file(
            &airport_plain,
            r#"{
  "outbounds": [
    { "type": "trojan", "tag": "香港-A", "server": "hk.example", "server_port": 443, "password": "x" },
    { "type": "trojan", "tag": "美国-A", "server": "us.example", "server_port": 443, "password": "x" }
  ]
}
"#,
        );
        write_file(
            &private_plain,
            r#"{
  "outbounds": [
    { "type": "shadowsocks", "tag": "Private A", "server": "private.example", "server_port": 8388, "method": "aes-128-gcm", "password": "x" }
  ]
}
"#,
        );
        write_file(
            &common_rules,
            r#"{
  "rules": [
    { "domain_suffix": ["google.com"], "outbound": "Common" }
  ]
}
"#,
        );
        write_file(
            &direct_rules,
            r#"{
  "rules": [
    { "domain_suffix": ["liberte.top"], "outbound": "direct" }
  ]
}
"#,
        );

        std::fs::write(
            &airport_age,
            encrypt_for(
                &recipient,
                &std::fs::read_to_string(&airport_plain).unwrap(),
            ),
        )
        .unwrap();
        std::fs::write(
            &private_age,
            encrypt_for(
                &recipient,
                &std::fs::read_to_string(&private_plain).unwrap(),
            ),
        )
        .unwrap();

        let manifest = format!(
            r#"{{
  "version": 1,
  "profile": "desktop-passive",
  "shared": {{
    "dns": {{ "servers": [{{ "tag": "alidns", "type": "udp", "server": "223.5.5.5", "server_port": 53 }}], "final": "alidns" }},
    "inbounds": [{{ "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": 7890 }}]
  }},
  "route": {{
    "autoDetectInterface": true,
    "finalOutbound": "Private"
  }},
  "nodeSources": [
    {{ "key": "airport", "type": "file", "path": "{}", "encrypted": true }},
    {{ "key": "private", "type": "file", "path": "{}", "encrypted": true }}
  ],
  "groups": [
    {{ "tag": "Common", "type": "urltest", "source": "airport", "includes": ["香港", "美国"], "healthcheckUrl": "https://health.example", "intervalSeconds": 3600, "idleTimeoutMs": 1800, "tolerance": 100 }},
    {{ "tag": "Tunnel", "type": "urltest", "source": "airport", "includes": ["美国"], "healthcheckUrl": "https://health.example", "intervalSeconds": 3600, "idleTimeoutMs": 1800, "tolerance": 100 }},
    {{ "tag": "Private", "type": "selector", "source": "private", "default": "Private A" }}
  ],
  "nodeOverrides": {{
    "private": {{ "detour": "Tunnel" }}
  }},
  "ruleSources": [
    {{ "key": "common", "type": "file", "path": "{}", "encrypted": false }},
    {{ "key": "direct", "type": "file", "path": "{}", "encrypted": false }}
  ]
}}
"#,
            airport_age.display(),
            private_age.display(),
            common_rules.display(),
            direct_rules.display()
        );
        std::fs::write(&manifest_age, encrypt_for(&recipient, &manifest)).unwrap();

        std::env::set_var(
            "SINGBOX_TAURI_SUBSCRIPTION_FILE",
            manifest_age.display().to_string(),
        );
        let snapshot = service.refresh(&paths).unwrap();
        std::env::remove_var("SINGBOX_TAURI_SUBSCRIPTION_FILE");

        assert_eq!(snapshot.subscription_id.as_deref(), Some("debug-file"));
        assert!(scoped.active_config_path.is_file());
        assert!(scoped.groups_path.is_file());
        assert!(scoped.source_node_path("airport", true).is_file());
        assert!(scoped.source_rule_path("common", false).is_file());

        let active = std::fs::read_to_string(&scoped.active_config_path).unwrap();
        assert!(active.contains("\"inbounds\""));
        assert!(active.contains("\"dns\""));
        assert!(active.contains("\"direct\""));
        assert!(active.contains("\"tag\": \"Tunnel\""));
        assert!(active.contains("\"detour\": \"Tunnel\""));
        assert!(active.contains("\"final\": \"Private\""));

        let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".runtime")
            .join("dev")
            .join("bin")
            .join("sing-box");
        if binary_path.is_file() {
            let output = Command::new(&binary_path)
                .arg("check")
                .arg("-c")
                .arg(&scoped.active_config_path)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "sing-box check failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
