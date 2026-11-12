use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    app_name: &'static str,
    version: &'static str,
    operating_system: &'static str,
    architecture: &'static str,
}

/// Returns static runtime metadata. This command does not access host files,
/// launch processes, inspect private system data, or perform privileged work.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        app_name: "Sync AI Desktop",
        version: env!("CARGO_PKG_VERSION"),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
    }
}
