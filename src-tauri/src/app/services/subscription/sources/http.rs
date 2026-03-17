use crate::app::services::subscription::types::{SubscriptionPayload, SubscriptionSourceKind};

pub fn fetch(url: &str) -> Result<SubscriptionPayload, String> {
    let response = reqwest::blocking::get(url)
        .map_err(|err| format!("failed to fetch subscription: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "subscription fetch failed with status {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|err| format!("failed to read subscription body: {err}"))?;

    Ok(SubscriptionPayload {
        source_kind: SubscriptionSourceKind::Http,
        source_reference: url.to_string(),
        encrypted_bytes: bytes.to_vec(),
    })
}
