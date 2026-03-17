pub fn validate_and_normalize(payload: &str) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(payload)
        .map_err(|err| format!("decrypted payload is not valid JSON: {err}"))?;

    if payload.ends_with('\n') {
        Ok(payload.to_string())
    } else {
        Ok(format!("{payload}\n"))
    }
}
