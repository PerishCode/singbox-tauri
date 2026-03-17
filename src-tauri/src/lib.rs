mod app;
mod openapi;
mod runtime_paths;
mod singbox;

use std::sync::Arc;

use app::app::App;
use app::services::singbox::{SingboxBootstrapReport, SingboxRuntimeStatus};
use runtime_paths::RuntimePaths;
use tauri::Manager;

#[tauri::command]
fn get_runtime_paths(app: tauri::State<'_, Arc<App>>) -> Result<RuntimePaths, String> {
    app.runtime_paths()
}

#[tauri::command]
fn bootstrap_singbox(app: tauri::State<'_, Arc<App>>) -> Result<SingboxBootstrapReport, String> {
    app.bootstrap_singbox()
}

#[tauri::command]
fn get_singbox_status(app: tauri::State<'_, Arc<App>>) -> Result<SingboxRuntimeStatus, String> {
    app.singbox_status()
}

#[tauri::command]
fn start_singbox(app: tauri::State<'_, Arc<App>>) -> Result<SingboxRuntimeStatus, String> {
    app.start_singbox()
}

#[tauri::command]
fn stop_singbox(app: tauri::State<'_, Arc<App>>) -> Result<SingboxRuntimeStatus, String> {
    app.stop_singbox()
}

#[tauri::command]
fn restart_singbox(app: tauri::State<'_, Arc<App>>) -> Result<SingboxRuntimeStatus, String> {
    app.restart_singbox()
}

#[tauri::command]
fn append_app_event(app: tauri::State<'_, Arc<App>>, message: String) -> Result<(), String> {
    app.append_app_event(&message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(App::default());
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = app.state::<Arc<App>>();
            state.set_handle(app.handle().clone());

            crate::app::control_server::spawn(state.inner().clone());
            if let Err(err) = state.inner().initialize() {
                eprintln!("app initialization failed: {err}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_runtime_paths,
            bootstrap_singbox,
            get_singbox_status,
            start_singbox,
            stop_singbox,
            restart_singbox,
            append_app_event
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
