use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
pub enum SubscriptionEntryType {
    EncryptedArtifact,
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
pub struct SubscriptionDefinitionSnapshot {
    pub id: Option<String>,
    pub label: String,
    pub r#type: SubscriptionEntryType,
    pub scope: String,
    pub profile: Option<String>,
    pub adapter: SubscriptionAdapterKind,
    pub source: SubscriptionSourceDefinition,
    pub entries: Vec<SubscriptionRegistryItem>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRegistryItem {
    pub id: String,
    pub label: String,
    pub r#type: SubscriptionEntryType,
    pub adapter: SubscriptionAdapterKind,
    pub source: SubscriptionSourceDefinition,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSourceDefinition {
    pub r#type: Option<SubscriptionSourceKind>,
    pub url: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRuntimeSnapshot {
    pub subscription_id: Option<String>,
    pub scoped_root_path: String,
    pub source_metadata_path: String,
    pub key_state: SubscriptionKeyState,
    pub fetch_state: SubscriptionFetchState,
    pub decrypt_state: SubscriptionDecryptState,
    pub private_key_path: String,
    pub public_key_path: String,
    pub encrypted_path: String,
    pub decrypted_path: String,
    pub active_config_path: String,
    pub nodes_path: String,
    pub rules_path: String,
    pub compose_input_path: String,
    pub groups_path: String,
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
pub struct SubscriptionPayload {
    pub source_kind: SubscriptionSourceKind,
    pub source_reference: String,
    pub encrypted_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionPreparedArtifacts {
    pub normalized: String,
    pub nodes: SubscriptionNodesResource,
    pub rules: SubscriptionRulesResource,
    pub compose_input: SubscriptionComposeInput,
    pub groups: Vec<Value>,
    pub source_metadata: SubscriptionSourceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionResourceSource {
    pub key: String,
    pub r#type: SubscriptionSourceKind,
    pub url: Option<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroupDefinition {
    pub tag: String,
    pub r#type: String,
    pub source: String,
    pub default: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    pub healthcheck_url: Option<String>,
    pub interval_seconds: Option<u64>,
    pub idle_timeout_ms: Option<u64>,
    pub tolerance: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionNodeOverride {
    pub detour: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionConfigManifest {
    pub version: u64,
    pub profile: String,
    pub shared: SubscriptionComposeSharedInput,
    pub route: SubscriptionComposeRouteInput,
    #[serde(default)]
    pub node_sources: Vec<SubscriptionResourceSource>,
    #[serde(default)]
    pub groups: Vec<SubscriptionGroupDefinition>,
    #[serde(default)]
    pub node_overrides: std::collections::BTreeMap<String, SubscriptionNodeOverride>,
    #[serde(default)]
    pub rule_sources: Vec<SubscriptionResourceSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionNodesResource {
    pub outbounds: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRulesResource {
    pub rules: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionComposeRouteInput {
    pub auto_detect_interface: Option<Value>,
    pub final_outbound: Option<Value>,
    pub rule_set: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionComposeSharedInput {
    pub dns: Option<Value>,
    pub inbounds: Option<Value>,
    pub experimental: Option<Value>,
    pub ntp: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionComposeInput {
    pub shared: SubscriptionComposeSharedInput,
    pub route: SubscriptionComposeRouteInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSourceMetadata {
    pub subscription_id: Option<String>,
    pub source_kind: SubscriptionSourceKind,
    pub source_reference: String,
}

#[derive(Debug, Clone)]
pub struct SubscriptionResolved {
    pub id: Option<String>,
    pub label: String,
    pub entry_type: SubscriptionEntryType,
    pub source_kind: Option<SubscriptionSourceKind>,
    pub source_scope: String,
    pub source_profile: Option<String>,
    pub adapter_kind: SubscriptionAdapterKind,
    pub source_url: Option<String>,
    pub source_path: Option<String>,
}
