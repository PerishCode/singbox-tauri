use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use age::x25519;
use age::Decryptor;
use secrecy::ExposeSecret;
use serde::Serialize;
use utoipa::ToSchema;

use crate::runtime_paths::RuntimePaths;

const SUBSCRIPTION_URL_ENV: &str = "SINGBOX_TAURI_SUBSCRIPTION_URL";

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionKeyState {
    Missing,
    Ready,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionFetchState {
    Idle,
    Fetched,
    Failed,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionDecryptState {
    Idle,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSnapshot {
    pub key_state: SubscriptionKeyState,
    pub fetch_state: SubscriptionFetchState,
    pub decrypt_state: SubscriptionDecryptState,
    pub source_url: Option<String>,
    pub private_key_path: String,
    pub public_key_path: String,
    pub encrypted_path: String,
    pub decrypted_path: String,
    pub active_config_path: String,
    pub public_key: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Default)]
pub struct SubscriptionService;

impl SubscriptionService {
    pub fn ensure_local_state(&self, paths: &RuntimePaths) -> Result<(), String> {
        let (private_key_path, public_key_path) = key_paths(paths);
        if private_key_path.is_file() {
            let identity = read_identity(&private_key_path)?;
            if !public_key_path.is_file() {
                fs::write(&public_key_path, format!("{}\n", identity.to_public())).map_err(
                    |err| format!("failed to write {}: {err}", public_key_path.display()),
                )?;
            }
            return Ok(());
        }

        let identity = x25519::Identity::generate();
        fs::write(
            &private_key_path,
            format!("{}\n", identity.to_string().expose_secret()),
        )
        .map_err(|err| format!("failed to write {}: {err}", private_key_path.display()))?;
        fs::write(&public_key_path, format!("{}\n", identity.to_public()))
            .map_err(|err| format!("failed to write {}: {err}", public_key_path.display()))?;
        Ok(())
    }

    pub fn refresh(&self, paths: &RuntimePaths) -> Result<SubscriptionSnapshot, String> {
        self.ensure_local_state(paths)?;

        let Some(source_url) = subscription_url() else {
            return Ok(self.snapshot(paths));
        };

        let (encrypted_path, decrypted_path, active_config_path) = artifact_paths(paths);
        let response = reqwest::blocking::get(&source_url)
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
        fs::write(&encrypted_path, &bytes)
            .map_err(|err| format!("failed to write {}: {err}", encrypted_path.display()))?;

        let identity = read_identity(&key_paths(paths).0)?;
        let decryptor = Decryptor::new(Cursor::new(bytes.as_ref()))
            .map_err(|err| format!("failed to parse encrypted payload: {err}"))?;
        let mut plaintext = String::new();
        let mut reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|err| format!("failed to decrypt subscription: {err}"))?;
        reader
            .read_to_string(&mut plaintext)
            .map_err(|err| format!("failed to read decrypted payload: {err}"))?;

        validate_json_payload(&plaintext)?;
        fs::write(&decrypted_path, &plaintext)
            .map_err(|err| format!("failed to write {}: {err}", decrypted_path.display()))?;
        fs::write(&active_config_path, ensure_trailing_newline(&plaintext))
            .map_err(|err| format!("failed to write {}: {err}", active_config_path.display()))?;

        Ok(self.snapshot(paths))
    }

    pub fn snapshot(&self, paths: &RuntimePaths) -> SubscriptionSnapshot {
        let (private_key_path, public_key_path) = key_paths(paths);
        let (encrypted_path, decrypted_path, active_config_path) = artifact_paths(paths);
        let public_key = fs::read_to_string(&public_key_path)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let key_state = if private_key_path.is_file() && public_key.is_some() {
            SubscriptionKeyState::Ready
        } else {
            SubscriptionKeyState::Missing
        };
        let fetch_state = if encrypted_path.is_file() {
            SubscriptionFetchState::Fetched
        } else if subscription_url().is_some() {
            SubscriptionFetchState::Failed
        } else {
            SubscriptionFetchState::Idle
        };
        let decrypt_state = if active_config_path.is_file() && decrypted_path.is_file() {
            SubscriptionDecryptState::Ready
        } else if encrypted_path.is_file() {
            SubscriptionDecryptState::Failed
        } else {
            SubscriptionDecryptState::Idle
        };

        SubscriptionSnapshot {
            key_state,
            fetch_state,
            decrypt_state,
            source_url: subscription_url(),
            private_key_path: private_key_path.display().to_string(),
            public_key_path: public_key_path.display().to_string(),
            encrypted_path: encrypted_path.display().to_string(),
            decrypted_path: decrypted_path.display().to_string(),
            active_config_path: active_config_path.display().to_string(),
            public_key,
            last_error: read_last_error(paths),
        }
    }
}

pub fn runtime_config_source_path(paths: &RuntimePaths) -> PathBuf {
    artifact_paths(paths).2
}

pub fn write_last_error(paths: &RuntimePaths, error: &str) -> Result<(), String> {
    let error_path = paths.state_dir.join("subscription-error.txt");
    fs::write(&error_path, ensure_trailing_newline(error))
        .map_err(|err| format!("failed to write {}: {err}", error_path.display()))
}

pub fn clear_last_error(paths: &RuntimePaths) {
    let error_path = paths.state_dir.join("subscription-error.txt");
    let _ = fs::remove_file(error_path);
}

fn subscription_url() -> Option<String> {
    std::env::var(SUBSCRIPTION_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn key_paths(paths: &RuntimePaths) -> (PathBuf, PathBuf) {
    (
        paths.secrets_dir.join("subscription.agekey"),
        paths.secrets_dir.join("subscription.pub"),
    )
}

fn artifact_paths(paths: &RuntimePaths) -> (PathBuf, PathBuf, PathBuf) {
    (
        paths.subscriptions_dir.join("subscription.json.age"),
        paths.subscriptions_dir.join("subscription.json"),
        paths.subscriptions_dir.join("active-config.json"),
    )
}

fn read_identity(path: &Path) -> Result<x25519::Identity, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    content
        .trim()
        .parse::<x25519::Identity>()
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn validate_json_payload(payload: &str) -> Result<(), String> {
    serde_json::from_str::<serde_json::Value>(payload)
        .map(|_| ())
        .map_err(|err| format!("decrypted payload is not valid JSON: {err}"))
}

fn ensure_trailing_newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_string()
    } else {
        format!("{value}\n")
    }
}

fn read_last_error(paths: &RuntimePaths) -> Option<String> {
    fs::read_to_string(paths.state_dir.join("subscription-error.txt"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
