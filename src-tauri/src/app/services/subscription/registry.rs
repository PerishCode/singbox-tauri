use super::types::{SubscriptionAdapterKind, SubscriptionEntryType, SubscriptionSourceKind};

const CURRENT_ENTRY_ID_ENV: &str = "SINGBOX_TAURI_SUBSCRIPTION_ENTRY_ID";

#[derive(Debug, Clone)]
pub struct SubscriptionRegistryEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub entry_type: SubscriptionEntryType,
    pub source_kind: SubscriptionSourceKind,
    pub source_url: &'static str,
    pub adapter_kind: SubscriptionAdapterKind,
}

const ENTRIES: &[SubscriptionRegistryEntry] = &[
    SubscriptionRegistryEntry {
        id: "desktop-passive",
        label: "Desktop Passive",
        entry_type: SubscriptionEntryType::EncryptedArtifact,
        source_kind: SubscriptionSourceKind::Http,
        source_url: "https://justvps.liberte.top/sing-box/profiles/desktop-passive.json.age",
        adapter_kind: SubscriptionAdapterKind::SingboxRaw,
    },
    SubscriptionRegistryEntry {
        id: "desktop-tun",
        label: "Desktop Tun",
        entry_type: SubscriptionEntryType::EncryptedArtifact,
        source_kind: SubscriptionSourceKind::Http,
        source_url: "https://justvps.liberte.top/sing-box/profiles/desktop-tun.json.age",
        adapter_kind: SubscriptionAdapterKind::SingboxRaw,
    },
];

pub fn current_entry() -> SubscriptionRegistryEntry {
    if let Ok(value) = std::env::var(CURRENT_ENTRY_ID_ENV) {
        let trimmed = value.trim();
        if let Some(entry) = ENTRIES.iter().find(|entry| entry.id == trimmed) {
            return entry.clone();
        }
    }

    ENTRIES
        .iter()
        .find(|entry| entry.id == "desktop-passive")
        .cloned()
        .unwrap_or_else(|| ENTRIES[0].clone())
}

pub fn entries() -> Vec<SubscriptionRegistryEntry> {
    ENTRIES.to_vec()
}
