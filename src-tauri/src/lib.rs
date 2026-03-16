mod app;
mod runtime_paths;
mod singbox;

use app::services::singbox::{SingboxBootstrapReport, SingboxService};
use runtime_paths::RuntimePaths;

#[tauri::command]
fn get_runtime_paths(app: tauri::AppHandle) -> Result<RuntimePaths, String> {
    RuntimePaths::resolve(&app)
}

#[tauri::command]
fn bootstrap_singbox(app: tauri::AppHandle) -> Result<SingboxBootstrapReport, String> {
    let paths = RuntimePaths::resolve(&app)?;
    SingboxService::new().bootstrap(&paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_runtime_paths,
            bootstrap_singbox
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
