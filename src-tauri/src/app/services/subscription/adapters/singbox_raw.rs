use serde_json::Value;

use crate::app::services::subscription::types::{
    SubscriptionComposeInput, SubscriptionComposeRouteInput, SubscriptionComposeSharedInput,
    SubscriptionNodesResource, SubscriptionPreparedArtifacts, SubscriptionRulesResource,
    SubscriptionSourceKind, SubscriptionSourceMetadata,
};

pub fn prepare(payload: &str) -> Result<SubscriptionPreparedArtifacts, String> {
    let parsed = serde_json::from_str::<Value>(payload)
        .map_err(|err| format!("decrypted payload is not valid JSON: {err}"))?;

    let normalized = if payload.ends_with('\n') {
        payload.to_string()
    } else {
        format!("{payload}\n")
    };

    let nodes = SubscriptionNodesResource {
        outbounds: parsed
            .get("outbounds")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
    };

    let rules = SubscriptionRulesResource {
        rules: parsed
            .get("route")
            .and_then(|route| route.get("rules"))
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default(),
    };

    let compose_input = SubscriptionComposeInput {
        shared: SubscriptionComposeSharedInput {
            dns: parsed.get("dns").cloned(),
            inbounds: parsed.get("inbounds").cloned(),
            experimental: parsed.get("experimental").cloned(),
            ntp: parsed.get("ntp").cloned(),
        },
        route: SubscriptionComposeRouteInput {
            auto_detect_interface: parsed
                .get("route")
                .and_then(|route| route.get("auto_detect_interface"))
                .cloned(),
            final_outbound: parsed
                .get("route")
                .and_then(|route| route.get("final"))
                .cloned(),
            rule_set: parsed
                .get("route")
                .and_then(|route| route.get("rule_set"))
                .cloned(),
        },
    };

    Ok(SubscriptionPreparedArtifacts {
        normalized,
        nodes,
        rules,
        compose_input,
        groups: vec![],
        source_metadata: SubscriptionSourceMetadata {
            subscription_id: None,
            source_kind: SubscriptionSourceKind::File,
            source_reference: "legacy-inline-config".to_string(),
        },
    })
}
