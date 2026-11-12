use tauri::{AppHandle, State};

use crate::{
    commands::models::resolve_model_path,
    inference::{
        gguf::GgufMetadata,
        runtime::{
            estimate_resources, GenerationEvent, GenerationResult, ModelResourceEstimate,
            RamLimiterConfig, RuntimeManager, RuntimeStatus,
        },
    },
    storage::{models::ModelRecord, Database},
};

#[tauri::command]
pub fn get_runtime_status(runtime: State<'_, RuntimeManager>) -> Result<RuntimeStatus, String> {
    runtime.status()
}

#[tauri::command]
pub fn estimate_model_resources(
    app: AppHandle,
    db: State<'_, Database>,
    model_id: String,
) -> Result<ModelResourceEstimate, String> {
    let record = find_model(&db, &model_id)?;
    let path = resolve_model_path(&app, &db, &model_id)?;
    let _metadata = GgufMetadata::read(&path)?;
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let config = RamLimiterConfig::from_json(db.get_setting("ram_limiter")?)?;
    Ok(estimate_resources(record.size_bytes, system.available_memory(), &config))
}

#[tauri::command]
pub async fn load_model(
    app: AppHandle,
    db: State<'_, Database>,
    runtime: State<'_, RuntimeManager>,
    model_id: String,
) -> Result<RuntimeStatus, String> {
    let record = find_model(&db, &model_id)?;
    let path = resolve_model_path(&app, &db, &model_id)?;
    let _ = GgufMetadata::read(&path)?;
    let config = RamLimiterConfig::from_json(db.get_setting("ram_limiter")?)?;
    let memory_limit = config.limit_bytes();
    let manager = runtime.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        manager.load_model(model_id, record.name, path, memory_limit)
    })
    .await
    .map_err(|error| format!("The inference-worker task stopped unexpectedly: {error}"))?
}

#[tauri::command]
pub fn unload_model(runtime: State<'_, RuntimeManager>) -> Result<RuntimeStatus, String> {
    runtime.unload()
}

#[tauri::command]
pub async fn generate_chat(
    runtime: State<'_, RuntimeManager>,
    prompt: String,
    generation_id: String,
    on_event: tauri::ipc::Channel<GenerationEvent>,
) -> Result<GenerationResult, String> {
    let manager = runtime.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.generate_chat(prompt, generation_id, on_event)
    })
    .await
    .map_err(|error| format!("The chat-generation task stopped unexpectedly: {error}"))?
}

#[tauri::command]
pub fn cancel_generation(
    runtime: State<'_, RuntimeManager>,
    generation_id: String,
) -> Result<(), String> {
    runtime.cancel_generation(&generation_id)
}

fn find_model(db: &Database, model_id: &str) -> Result<ModelRecord, String> {
    db.list_models()?.into_iter().find(|model| model.id == model_id)
        .ok_or_else(|| "Model was not found in the local registry.".into())
}
