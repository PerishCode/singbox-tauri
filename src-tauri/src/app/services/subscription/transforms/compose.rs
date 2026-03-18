use serde_json::{Map, Value};

use crate::app::services::subscription::types::{
    SubscriptionComposeInput, SubscriptionNodesResource, SubscriptionRulesResource,
};

pub fn compose_config(
    compose_input: &SubscriptionComposeInput,
    nodes: &SubscriptionNodesResource,
    rules: &SubscriptionRulesResource,
) -> Result<String, String> {
    let mut root = Map::new();

    root.insert(
        "log".to_string(),
        serde_json::json!({
            "level": "info",
            "timestamp": true
        }),
    );

    insert_optional(&mut root, "dns", compose_input.shared.dns.clone());
    insert_optional(&mut root, "inbounds", compose_input.shared.inbounds.clone());
    insert_optional(
        &mut root,
        "experimental",
        compose_input.shared.experimental.clone(),
    );
    insert_optional(&mut root, "ntp", compose_input.shared.ntp.clone());

    root.insert(
        "outbounds".to_string(),
        Value::Array(nodes.outbounds.clone()),
    );

    if !nodes
        .outbounds
        .iter()
        .any(|outbound| outbound.get("tag").and_then(|value| value.as_str()) == Some("direct"))
    {
        return Err("composed config is missing required outbound 'direct'".to_string());
    }

    let mut route = Map::new();
    insert_optional(
        &mut route,
        "auto_detect_interface",
        compose_input.route.auto_detect_interface.clone(),
    );
    insert_optional(
        &mut route,
        "final",
        compose_input.route.final_outbound.clone(),
    );
    insert_optional(&mut route, "rule_set", compose_input.route.rule_set.clone());
    route.insert("rules".to_string(), Value::Array(rules.rules.clone()));

    if !route.is_empty() {
        root.insert("route".to_string(), Value::Object(route));
    }

    if !root.contains_key("inbounds") {
        return Err(
            "composed config is missing required inbounds for passive proxy mode".to_string(),
        );
    }

    if !root.contains_key("dns") {
        return Err("composed config is missing required dns section".to_string());
    }

    let body = serde_json::to_string_pretty(&Value::Object(root))
        .map_err(|err| format!("failed to serialize composed config: {err}"))?;

    if body.ends_with('\n') {
        Ok(body)
    } else {
        Ok(format!("{body}\n"))
    }
}

fn insert_optional(target: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        target.insert(key.to_string(), value);
    }
}
