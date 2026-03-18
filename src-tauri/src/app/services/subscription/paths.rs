use std::fs;
use std::path::PathBuf;

use crate::runtime_paths::RuntimePaths;

#[derive(Debug, Clone)]
pub struct SubscriptionPaths {
    pub root: PathBuf,
    pub sources_dir: PathBuf,
    pub source_nodes_dir: PathBuf,
    pub source_rules_dir: PathBuf,
    pub resources_dir: PathBuf,
    pub resource_nodes_dir: PathBuf,
    pub resource_rules_dir: PathBuf,
    pub keys_dir: PathBuf,
    pub state_path: PathBuf,
    pub source_metadata_path: PathBuf,
    pub encrypted_path: PathBuf,
    pub decrypted_path: PathBuf,
    pub active_config_path: PathBuf,
    pub nodes_path: PathBuf,
    pub rules_path: PathBuf,
    pub compose_input_path: PathBuf,
    pub groups_path: PathBuf,
    pub private_key_path: PathBuf,
    pub public_key_path: PathBuf,
}

impl SubscriptionPaths {
    pub fn new(paths: &RuntimePaths, subscription_id: &str) -> Result<Self, String> {
        let root = paths.subscriptions_dir.join(subscription_id);
        let scoped = Self {
            sources_dir: root.join("sources"),
            source_nodes_dir: root.join("sources").join("nodes"),
            source_rules_dir: root.join("sources").join("rules"),
            resources_dir: root.join("resources"),
            resource_nodes_dir: root.join("resources").join("nodes"),
            resource_rules_dir: root.join("resources").join("rules"),
            keys_dir: root.join("keys"),
            state_path: root.join("state.json"),
            source_metadata_path: root.join("sources").join("source.json"),
            encrypted_path: root.join("sources").join("subscription.json.age"),
            decrypted_path: root.join("resources").join("subscription.json"),
            active_config_path: root.join("resources").join("active-config.json"),
            nodes_path: root.join("resources").join("nodes.json"),
            rules_path: root.join("resources").join("rules.json"),
            compose_input_path: root.join("resources").join("compose-input.json"),
            groups_path: root.join("resources").join("groups.json"),
            private_key_path: root.join("keys").join("subscription.agekey"),
            public_key_path: root.join("keys").join("subscription.pub"),
            root,
        };

        scoped.ensure_dirs()?;
        Ok(scoped)
    }

    fn ensure_dirs(&self) -> Result<(), String> {
        for dir in [
            &self.root,
            &self.sources_dir,
            &self.source_nodes_dir,
            &self.source_rules_dir,
            &self.resources_dir,
            &self.resource_nodes_dir,
            &self.resource_rules_dir,
            &self.keys_dir,
        ] {
            fs::create_dir_all(dir)
                .map_err(|err| format!("failed to create {}: {err}", dir.display()))?;
        }

        Ok(())
    }

    pub fn source_node_path(&self, key: &str, encrypted: bool) -> PathBuf {
        let name = sanitize_key(key);
        if encrypted {
            self.source_nodes_dir.join(format!("{name}.json.age"))
        } else {
            self.source_nodes_dir.join(format!("{name}.json"))
        }
    }

    pub fn source_rule_path(&self, key: &str, encrypted: bool) -> PathBuf {
        let name = sanitize_key(key);
        if encrypted {
            self.source_rules_dir.join(format!("{name}.json.age"))
        } else {
            self.source_rules_dir.join(format!("{name}.json"))
        }
    }

    pub fn resource_node_path(&self, key: &str) -> PathBuf {
        self.resource_nodes_dir
            .join(format!("{}.json", sanitize_key(key)))
    }

    pub fn resource_rule_path(&self, key: &str) -> PathBuf {
        self.resource_rules_dir
            .join(format!("{}.json", sanitize_key(key)))
    }
}

fn sanitize_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect()
}
