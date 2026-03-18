use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

use crate::app::services::subscription::transforms::age as age_transform;
use crate::app::services::subscription::transforms::compose;
use crate::app::services::subscription::types::{
    SubscriptionConfigManifest, SubscriptionGroupDefinition, SubscriptionNodeOverride,
    SubscriptionNodesResource, SubscriptionPayload, SubscriptionPreparedArtifacts,
    SubscriptionResourceSource, SubscriptionRulesResource, SubscriptionSourceMetadata,
};
use crate::runtime_paths::RuntimePaths;
use age::x25519::Identity as AgeIdentity;

use super::super::paths::SubscriptionPaths;
use super::super::sources;

pub fn prepare(
    paths: &RuntimePaths,
    scoped: &SubscriptionPaths,
    identity: &AgeIdentity,
    payload: &SubscriptionPayload,
) -> Result<SubscriptionPreparedArtifacts, String> {
    let plaintext = age_transform::decrypt_bytes(identity, &payload.encrypted_bytes)?;
    prepare_plaintext(paths, scoped, identity, payload, &plaintext)
}

pub fn is_manifest(payload: &str) -> bool {
    serde_json::from_str::<SubscriptionConfigManifest>(payload).is_ok()
}

pub fn prepare_plaintext(
    paths: &RuntimePaths,
    scoped: &SubscriptionPaths,
    identity: &AgeIdentity,
    payload: &SubscriptionPayload,
    plaintext: &str,
) -> Result<SubscriptionPreparedArtifacts, String> {
    let manifest = serde_json::from_str::<SubscriptionConfigManifest>(plaintext)
        .map_err(|err| format!("decrypted manifest is not valid JSON: {err}"))?;

    let normalized = ensure_trailing_newline(plaintext);
    let (nodes, groups) = load_nodes(paths, scoped, identity, &manifest)?;
    let rules = load_rules(paths, scoped, identity, &manifest)?;
    let compose_input = crate::app::services::subscription::types::SubscriptionComposeInput {
        shared: manifest.shared.clone(),
        route: manifest.route.clone(),
    };

    let _ = compose::compose_config(&compose_input, &nodes, &rules)?;

    Ok(SubscriptionPreparedArtifacts {
        normalized,
        nodes,
        rules,
        compose_input,
        groups,
        source_metadata: SubscriptionSourceMetadata {
            subscription_id: Some(manifest.profile.clone()),
            source_kind: payload.source_kind.clone(),
            source_reference: payload.source_reference.clone(),
        },
    })
}

fn load_nodes(
    paths: &RuntimePaths,
    scoped: &SubscriptionPaths,
    identity: &AgeIdentity,
    manifest: &SubscriptionConfigManifest,
) -> Result<(SubscriptionNodesResource, Vec<Value>), String> {
    let mut merged: Vec<Value> = Vec::new();
    let mut groups = Vec::new();
    let mut source_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for source in &manifest.node_sources {
        let payload = fetch_resource(paths, source)?;
        persist_source_payload(
            &scoped.source_node_path(&source.key, source.encrypted),
            &payload,
        )?;
        let body = decode_resource(identity, source, &payload)?;
        fs::write(
            scoped.resource_node_path(&source.key),
            ensure_trailing_newline(&body),
        )
        .map_err(|err| {
            format!(
                "failed to write {}: {err}",
                scoped.resource_node_path(&source.key).display()
            )
        })?;
        let resource = serde_json::from_str::<SubscriptionNodesResource>(&body)
            .map_err(|err| format!("node resource '{}' is invalid JSON: {err}", source.key))?;
        let override_cfg = manifest
            .node_overrides
            .get(&source.key)
            .cloned()
            .unwrap_or_default();

        for outbound in resource.outbounds {
            let outbound = apply_node_override(outbound, &override_cfg);
            if let Some(tag) = outbound
                .as_object()
                .and_then(|object| object.get("tag"))
                .and_then(|value| value.as_str())
            {
                source_tags
                    .entry(source.key.clone())
                    .or_default()
                    .push(tag.to_string());
            }
            merged.push(outbound);
        }
    }

    for group in &manifest.groups {
        let outbound = build_group_outbound(group, &source_tags)?;
        groups.push(outbound.clone());
        merged.push(outbound);
    }

    merged.push(serde_json::json!({ "type": "direct", "tag": "direct" }));
    merged.push(serde_json::json!({ "type": "block", "tag": "block" }));

    Ok((SubscriptionNodesResource { outbounds: merged }, groups))
}

fn load_rules(
    paths: &RuntimePaths,
    scoped: &SubscriptionPaths,
    identity: &AgeIdentity,
    manifest: &SubscriptionConfigManifest,
) -> Result<SubscriptionRulesResource, String> {
    let mut merged = Vec::new();

    for source in &manifest.rule_sources {
        let payload = fetch_resource(paths, source)?;
        persist_source_payload(
            &scoped.source_rule_path(&source.key, source.encrypted),
            &payload,
        )?;
        let body = decode_resource(identity, source, &payload)?;
        fs::write(
            scoped.resource_rule_path(&source.key),
            ensure_trailing_newline(&body),
        )
        .map_err(|err| {
            format!(
                "failed to write {}: {err}",
                scoped.resource_rule_path(&source.key).display()
            )
        })?;
        let resource = serde_json::from_str::<SubscriptionRulesResource>(&body)
            .map_err(|err| format!("rule resource '{}' is invalid JSON: {err}", source.key))?;
        merged.extend(resource.rules);
    }

    Ok(SubscriptionRulesResource { rules: merged })
}

fn fetch_resource(
    paths: &RuntimePaths,
    source: &SubscriptionResourceSource,
) -> Result<SubscriptionPayload, String> {
    match source.r#type {
        crate::app::services::subscription::types::SubscriptionSourceKind::File => {
            let path = source
                .path
                .as_ref()
                .ok_or_else(|| format!("resource '{}' missing file path", source.key))?;
            sources::fetch_file(paths, path)
        }
        crate::app::services::subscription::types::SubscriptionSourceKind::Http => {
            let url = source
                .url
                .as_ref()
                .ok_or_else(|| format!("resource '{}' missing URL", source.key))?;
            sources::fetch_http(url)
        }
    }
}

fn decode_resource(
    identity: &AgeIdentity,
    source: &SubscriptionResourceSource,
    payload: &SubscriptionPayload,
) -> Result<String, String> {
    if source.encrypted {
        age_transform::decrypt_bytes(identity, &payload.encrypted_bytes)
    } else {
        String::from_utf8(payload.encrypted_bytes.clone())
            .map_err(|err| format!("resource '{}' is not valid UTF-8: {err}", source.key))
    }
}

fn apply_node_override(mut outbound: Value, override_cfg: &SubscriptionNodeOverride) -> Value {
    if let Some(detour) = &override_cfg.detour {
        if let Some(object) = outbound.as_object_mut() {
            object.insert("detour".to_string(), Value::String(detour.clone()));
        }
    }
    outbound
}

fn build_group_outbound(
    group: &SubscriptionGroupDefinition,
    source_tags: &BTreeMap<String, Vec<String>>,
) -> Result<Value, String> {
    let mut members: Vec<String> = source_tags
        .get(&group.source)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|tag| {
            if group.includes.is_empty() {
                true
            } else {
                group.includes.iter().any(|needle| tag.contains(needle))
            }
        })
        .collect();

    members.sort();
    members.dedup();

    if members.is_empty() {
        return Err(format!("group '{}' resolved to no members", group.tag));
    }

    let mut object = serde_json::Map::new();
    object.insert("tag".to_string(), Value::String(group.tag.clone()));
    object.insert(
        "outbounds".to_string(),
        Value::Array(members.into_iter().map(Value::String).collect()),
    );

    match group.r#type.as_str() {
        "selector" => {
            object.insert("type".to_string(), Value::String("selector".to_string()));
            if let Some(default) = &group.default {
                object.insert("default".to_string(), Value::String(default.clone()));
            }
        }
        "urltest" => {
            object.insert("type".to_string(), Value::String("urltest".to_string()));
            if let Some(url) = &group.healthcheck_url {
                object.insert("url".to_string(), Value::String(url.clone()));
            }
            if let Some(interval) = group.interval_seconds {
                object.insert(
                    "interval".to_string(),
                    Value::String(format!("{interval}s")),
                );
            }
            if let Some(idle_timeout) = group.idle_timeout_ms {
                object.insert(
                    "idle_timeout".to_string(),
                    Value::String(format!("{idle_timeout}ms")),
                );
            }
            if let Some(tolerance) = group.tolerance {
                object.insert("tolerance".to_string(), Value::Number(tolerance.into()));
            }
        }
        other => return Err(format!("unsupported group type: {other}")),
    }

    Ok(Value::Object(object))
}

fn ensure_trailing_newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_string()
    } else {
        format!("{value}\n")
    }
}

fn persist_source_payload(
    path: &std::path::Path,
    payload: &SubscriptionPayload,
) -> Result<(), String> {
    fs::write(path, &payload.encrypted_bytes)
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_paths::{RuntimeMode, RuntimePaths};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_runtime_paths() -> RuntimePaths {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("singbox-tauri-manifest-test-{stamp}"));
        std::fs::create_dir_all(&root).unwrap();
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

    #[test]
    fn manifest_prepare_builds_groups_and_rules_from_local_sources() {
        let paths = temp_runtime_paths();
        let scoped = SubscriptionPaths::new(&paths, "desktop-passive").unwrap();
        let identity = AgeIdentity::generate();

        let airport_path = paths.root.join("fixtures/airport.json");
        let private_path = paths.root.join("fixtures/private.json");
        let common_rules_path = paths.root.join("fixtures/common.json");
        let direct_rules_path = paths.root.join("fixtures/direct.json");

        write_file(
            &airport_path,
            r#"{
  "outbounds": [
    { "type": "trojan", "tag": "香港-A", "server": "hk.example", "server_port": 443, "password": "x" },
    { "type": "trojan", "tag": "美国-A", "server": "us.example", "server_port": 443, "password": "x" }
  ]
}
"#,
        );
        write_file(
            &private_path,
            r#"{
  "outbounds": [
    { "type": "shadowsocks", "tag": "Private A", "server": "private.example", "server_port": 8388, "method": "aes-128-gcm", "password": "x" }
  ]
}
"#,
        );
        write_file(
            &common_rules_path,
            r#"{
  "rules": [
    { "domain_suffix": ["google.com"], "outbound": "Common" }
  ]
}
"#,
        );
        write_file(
            &direct_rules_path,
            r#"{
  "rules": [
    { "domain_suffix": ["liberte.top"], "outbound": "direct" }
  ]
}
"#,
        );

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
    {{ "key": "airport", "type": "file", "path": "{}", "encrypted": false }},
    {{ "key": "private", "type": "file", "path": "{}", "encrypted": false }}
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
            airport_path.display(),
            private_path.display(),
            common_rules_path.display(),
            direct_rules_path.display()
        );

        let payload = SubscriptionPayload {
            source_kind: crate::app::services::subscription::types::SubscriptionSourceKind::File,
            source_reference: "local-manifest".to_string(),
            encrypted_bytes: vec![],
        };

        let prepared = prepare_plaintext(&paths, &scoped, &identity, &payload, &manifest).unwrap();
        let composed =
            compose::compose_config(&prepared.compose_input, &prepared.nodes, &prepared.rules)
                .unwrap();
        let parsed: Value = serde_json::from_str(&composed).unwrap();

        let outbounds = parsed
            .get("outbounds")
            .and_then(|value| value.as_array())
            .unwrap();
        assert!(outbounds
            .iter()
            .any(|item| item.get("tag").and_then(|v| v.as_str()) == Some("Common")));
        assert!(outbounds
            .iter()
            .any(|item| item.get("tag").and_then(|v| v.as_str()) == Some("Tunnel")));
        assert!(outbounds
            .iter()
            .any(|item| item.get("tag").and_then(|v| v.as_str()) == Some("Private")));

        let private_node = outbounds
            .iter()
            .find(|item| item.get("tag").and_then(|v| v.as_str()) == Some("Private A"))
            .unwrap();
        assert_eq!(
            private_node.get("detour").and_then(|v| v.as_str()),
            Some("Tunnel")
        );

        let route = parsed
            .get("route")
            .and_then(|value| value.as_object())
            .unwrap();
        assert!(parsed.get("inbounds").is_some());
        assert!(parsed.get("dns").is_some());
        assert_eq!(route.get("final").and_then(|v| v.as_str()), Some("Private"));
        assert!(route
            .get("rules")
            .and_then(|value| value.as_array())
            .unwrap()
            .iter()
            .any(|rule| rule.get("outbound").and_then(|v| v.as_str()) == Some("Common")));
        assert!(prepared
            .groups
            .iter()
            .any(|group| group.get("tag").and_then(|v| v.as_str()) == Some("Tunnel")));
    }
}
