use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::{
    inference::{
        gguf::GgufMetadata,
        runtime::{
            estimate_resources, load_native_model, validate_preflight, ModelResourceEstimate,
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
    let path = managed_model_path(&app, &db, &model_id)?;
    let metadata = GgufMetadata::read(&path)?;
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let config = RamLimiterConfig::from_json(db.get_setting("ram_limiter")?)?;
    Ok(estimate_resources(record.size_bytes, &metadata, system.available_memory(), &config))
}

#[tauri::command]
pub async fn load_model(
    app: AppHandle,
    db: State<'_, Database>,
    runtime: State<'_, RuntimeManager>,
    model_id: String,
) -> Result<RuntimeStatus, String> {
    runtime.begin_loading(&model_id)?;

    let prepared = (|| {
        let record = find_model(&db, &model_id)?;
        let path = managed_model_path(&app, &db, &model_id)?;
        let metadata = GgufMetadata::read(&path)?;
        let config = RamLimiterConfig::from_json(db.get_setting("ram_limiter")?)?;
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        let estimate = estimate_resources(record.size_bytes, &metadata, system.available_memory(), &config);
        validate_preflight(&estimate)?;
        Ok::<_, String>((record, path, config, system.available_memory()))
    })();

    let (record, path, config, available) = match prepared {
        Ok(value) => value,
        Err(error) => {
            runtime.finish_loading(None)?;
            return Err(error);
        }
    };

    let load = tauri::async_runtime::spawn_blocking(move || {
        load_native_model(path, model_id, record.name, record.size_bytes, config, available)
    }).await;

    match load {
        Ok(Ok(model)) => {
            runtime.finish_loading(Some(model))?;
            runtime.status()
        }
        Ok(Err(error)) => {
            runtime.finish_loading(None)?;
            Err(error)
        }
        Err(error) => {
            runtime.finish_loading(None)?;
            Err(format!("The native model-loading worker stopped unexpectedly: {error}"))
        }
    }
}

#[tauri::command]
pub fn unload_model(runtime: State<'_, RuntimeManager>) -> Result<RuntimeStatus, String> {
    runtime.unload()?;
    runtime.status()
}

pub(crate) fn model_is_in_use(runtime: &RuntimeManager, model_id: &str) -> Result<bool, String> {
    runtime.is_in_use(model_id)
}

fn find_model(db: &Database, model_id: &str) -> Result<ModelRecord, String> {
    db.list_models()?.into_iter().find(|model| model.id == model_id)
        .ok_or_else(|| "Model was not found in the local registry.".into())
}

pub(crate) fn managed_models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Cannot prepare Sync data directory: {e}"))?;
    let canonical_app_dir = app_dir.canonicalize()
        .map_err(|e| format!("Cannot resolve canonical Sync data directory: {e}"))?;
    let model_dir = canonical_app_dir.join("models");
    std::fs::create_dir_all(&model_dir)
        .map_err(|e| format!("Cannot prepare managed model directory: {e}"))?;
    let canonical_model_dir = model_dir.canonicalize()
        .map_err(|e| format!("Managed model directory is unavailable: {e}"))?;
    if canonical_model_dir.parent() != Some(canonical_app_dir.as_path()) {
        return Err("Managed model directory resolves outside Sync's application data directory.".into());
    }
    Ok(canonical_model_dir)
}

pub(crate) fn managed_model_path(
    app: &AppHandle,
    db: &Database,
    model_id: &str,
) -> Result<PathBuf, String> {
    let relative = db.model_relative_path(model_id)?;
    let relative_path = PathBuf::from(&relative);
    if relative_path.components().count() != 1
        || relative_path.components().any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err("Stored model path failed the managed-directory safety check.".into());
    }

    let canonical_dir = managed_models_dir(app)?;
    let target = canonical_dir.join(relative_path).canonicalize()
        .map_err(|e| format!("Managed model file is unavailable: {e}"))?;
    if target.parent() != Some(canonical_dir.as_path()) || !target.is_file() {
        return Err("Stored model path escaped the managed model directory or is not a regular file.".into());
    }
    Ok(target)
}
