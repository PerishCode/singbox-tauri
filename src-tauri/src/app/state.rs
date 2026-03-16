use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AppLifecycle {
    Stopped,
    Starting,
    RunningPassive,
    RunningSelective,
    RunningFull,
    Stopping,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateSnapshot {
    pub lifecycle: AppLifecycle,
}

impl Default for AppStateSnapshot {
    fn default() -> Self {
        Self {
            lifecycle: AppLifecycle::Stopped,
        }
    }
}
