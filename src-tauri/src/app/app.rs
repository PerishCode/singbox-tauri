use std::fs;
use std::sync::OnceLock;

use tauri::AppHandle;

use crate::app::services::network::{LocalNetworkSnapshot, NetworkService};
use crate::app::services::singbox::{SingboxBootstrapReport, SingboxRuntimeStatus, SingboxService};
use crate::app::services::subscription::{
    clear_last_error, write_last_error, SubscriptionService, SubscriptionSnapshot,
};
use crate::runtime_paths::RuntimePaths;

#[derive(Default)]
pub struct App {
    handle: OnceLock<AppHandle>,
    network: NetworkService,
    singbox: SingboxService,
    subscription: SubscriptionService,
}

impl App {
    pub fn set_handle(&self, handle: AppHandle) {
        let _ = self.handle.set(handle);
    }

    fn handle(&self) -> Result<&AppHandle, String> {
        self.handle
            .get()
            .ok_or_else(|| "app handle is not initialized".to_string())
    }

    pub fn runtime_paths(&self) -> Result<RuntimePaths, String> {
        RuntimePaths::resolve(self.handle()?)
    }

    pub fn bootstrap_singbox(&self) -> Result<SingboxBootstrapReport, String> {
        let paths = self.runtime_paths()?;
        self.singbox.bootstrap(&paths)
    }

    pub fn singbox_status(&self) -> Result<SingboxRuntimeStatus, String> {
        let paths = self.runtime_paths()?;
        self.singbox.status(&paths)
    }

    pub fn start_singbox(&self) -> Result<SingboxRuntimeStatus, String> {
        let paths = self.runtime_paths()?;
        self.singbox.start(&paths)
    }

    pub fn stop_singbox(&self) -> Result<SingboxRuntimeStatus, String> {
        let paths = self.runtime_paths()?;
        self.singbox.stop(&paths)
    }

    pub fn restart_singbox(&self) -> Result<SingboxRuntimeStatus, String> {
        let paths = self.runtime_paths()?;
        self.singbox.restart(&paths)
    }

    pub fn append_app_event(&self, message: &str) -> Result<(), String> {
        let paths = self.runtime_paths()?;
        self.singbox.append_app_event(&paths, message)
    }

    pub fn local_network_snapshot(&self) -> LocalNetworkSnapshot {
        self.network.snapshot()
    }

    pub fn subscription_snapshot(&self) -> Result<SubscriptionSnapshot, String> {
        let paths = self.runtime_paths()?;
        Ok(self.subscription.snapshot(&paths))
    }

    pub fn initialize(&self) -> Result<(), String> {
        let paths = self.runtime_paths()?;
        let _ = self
            .singbox
            .append_app_event(&paths, "app initialization started");

        match self.subscription.refresh(&paths) {
            Ok(snapshot) => {
                clear_last_error(&paths);
                let _ = self.singbox.append_app_event(
                    &paths,
                    &format!(
                        "subscription ready key={:?} fetch={:?} decrypt={:?}",
                        snapshot.key_state, snapshot.fetch_state, snapshot.decrypt_state
                    ),
                );
            }
            Err(err) => {
                let _ = write_last_error(&paths, &err);
                let _ = self
                    .singbox
                    .append_app_event(&paths, &format!("subscription refresh skipped: {err}"));
            }
        }

        match self.singbox.bootstrap(&paths) {
            Ok(report) => {
                let _ = self.singbox.append_app_event(
                    &paths,
                    &format!("bootstrap ready process={:?}", report.process_status),
                );
            }
            Err(err) => {
                let _ = self
                    .singbox
                    .append_app_event(&paths, &format!("bootstrap failed: {err}"));
                return Err(err);
            }
        }

        match self.singbox.start(&paths) {
            Ok(status) => {
                let _ = self.singbox.append_app_event(
                    &paths,
                    &format!(
                        "auto-start settled lifecycle={:?} process={:?}",
                        status.lifecycle, status.process_status
                    ),
                );
                Ok(())
            }
            Err(err) => {
                let _ = self
                    .singbox
                    .append_app_event(&paths, &format!("auto-start failed: {err}"));
                Err(err)
            }
        }
    }

    pub fn read_app_log(&self) -> Result<String, String> {
        let paths = self.runtime_paths()?;
        read_log(paths.logs_dir.join("app.log"))
    }

    pub fn read_singbox_log(&self) -> Result<String, String> {
        let paths = self.runtime_paths()?;
        read_log(paths.logs_dir.join("sing-box.log"))
    }

    pub fn read_session_raw(&self) -> Result<String, String> {
        let paths = self.runtime_paths()?;
        read_text(paths.state_dir.join("session.json"))
    }

    pub fn read_runtime_metadata(&self) -> Result<String, String> {
        let paths = self.runtime_paths()?;
        read_text(paths.root.join("metadata").join("runtime.json"))
    }
}

fn read_log(path: std::path::PathBuf) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))
}

fn read_text(path: std::path::PathBuf) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))
}
