use serde::Serialize;
use utoipa::ToSchema;

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
pub enum SubscriptionSourceKind {
    File,
    Http,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionAdapterKind {
    SingboxRaw,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionApplyState {
    Unknown,
    PendingApply,
    Applied,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSnapshot {
    pub key_state: SubscriptionKeyState,
    pub fetch_state: SubscriptionFetchState,
    pub decrypt_state: SubscriptionDecryptState,
    pub source_kind: Option<SubscriptionSourceKind>,
    pub adapter_kind: SubscriptionAdapterKind,
    pub source_url: Option<String>,
    pub source_path: Option<String>,
    pub private_key_path: String,
    pub public_key_path: String,
    pub encrypted_path: String,
    pub decrypted_path: String,
    pub active_config_path: String,
    pub public_key: Option<String>,
    pub apply_state: SubscriptionApplyState,
    pub apply_message: String,
    pub last_attempt_at: Option<String>,
    pub last_successful_refresh_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionStateFile {
    pub last_attempt_at: Option<String>,
    pub last_successful_refresh_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionArtifacts {
    pub encrypted_path: std::path::PathBuf,
    pub decrypted_path: std::path::PathBuf,
    pub active_config_path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SubscriptionPayload {
    pub source_kind: SubscriptionSourceKind,
    pub source_reference: String,
    pub encrypted_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionResolved {
    pub source_kind: Option<SubscriptionSourceKind>,
    pub source_url: Option<String>,
    pub source_path: Option<String>,
}
