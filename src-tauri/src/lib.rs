mod runtime_paths;

use runtime_paths::RuntimePaths;

#[tauri::command]
fn get_runtime_paths(app: tauri::AppHandle) -> Result<RuntimePaths, String> {
    RuntimePaths::resolve(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_runtime_paths])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
