use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum AppLifecycle {
    Stopped,
    Starting,
    RunningPassive,
    RunningSelective,
    RunningFull,
    Stopping,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum AppRunMode {
    Passive,
    Selective,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateSnapshot {
    pub lifecycle: AppLifecycle,
    pub mode: AppRunMode,
    pub pid: Option<u32>,
}

impl Default for AppStateSnapshot {
    fn default() -> Self {
        Self {
            lifecycle: AppLifecycle::Stopped,
            mode: AppRunMode::Passive,
            pid: None,
        }
    }
}
