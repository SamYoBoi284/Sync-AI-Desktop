use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use sysinfo::System;
use tauri::{Manager, State};
use uuid::Uuid;

use crate::{
    inference::{gguf::GgufMetadata, runtime::RuntimeManager},
    storage::{models::ModelRecord, Database},
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRamInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelImportResult {
    pub model: ModelRecord,
    pub duplicate: bool,
}

#[tauri::command]
pub fn get_host_ram_info() -> HostRamInfo {
    let mut system = System::new();
    system.refresh_memory();
    HostRamInfo {
        total_bytes: system.total_memory(),
        available_bytes: system.available_memory(),
    }
}

#[tauri::command]
pub fn list_models(
    app: tauri::AppHandle,
    db: State<'_, Database>,
) -> Result<Vec<ModelRecord>, String> {
    let mut models = db.list_models()?;
    for model in &mut models {
        match resolve_model_path(&app, &db, &model.id) {
            Ok(path) => {
                model.file_path = path.to_string_lossy().into_owned();
                model.is_available = true;
                if model.architecture.is_none() || model.quantization.is_none() {
                    if let Ok(metadata) = GgufMetadata::read(&path) {
                        db.update_model_metadata(
                            &model.id,
                            metadata.architecture.clone(),
                            metadata.quantization.clone(),
                        )?;
                        model.architecture = metadata.architecture;
                        model.quantization = metadata.quantization;
                    }
                }
            }
            Err(_) => {
                model.is_available = false;
                if let Some(path) = &model.source_path {
                    model.file_path = path.clone();
                } else {
                    let app_dir = app.path().app_data_dir()
                        .map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
                    let legacy = app_dir.join("models").join(&model.file_path);
                    model.file_path = legacy.to_string_lossy().into_owned();
                }
            }
        }
    }
    Ok(models)
}

#[tauri::command]
pub fn import_gguf(
    db: State<'_, Database>,
) -> Result<Option<ModelImportResult>, String> {
    let Some(selected) = rfd::FileDialog::new()
        .add_filter("GGUF model", &["gguf"])
        .pick_file()
    else {
        return Ok(None);
    };

    if !selected.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("gguf")) {
        return Err("Only GGUF model files can be registered.".into());
    }

    let source = selected.canonicalize()
        .map_err(|e| format!("Cannot resolve selected GGUF file: {e}"))?;
    let metadata = GgufMetadata::read(&source)?;
    let file_metadata = fs::metadata(&source)
        .map_err(|e| format!("Cannot inspect selected model: {e}"))?;
    if !file_metadata.is_file() || file_metadata.len() == 0 {
        return Err("Selected model is empty or is not a regular file.".into());
    }

    let size_bytes = file_metadata.len();
    let file_name = source.file_name().and_then(|v| v.to_str())
        .ok_or("Selected filename is not valid UTF-8")?.to_string();
    let source_path = source.to_string_lossy().into_owned();

    let mut input = File::open(&source)
        .map_err(|e| format!("Cannot read selected model: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = input.read(&mut buffer)
            .map_err(|e| format!("Cannot hash selected model: {e}"))?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    let hash = format!("{:x}", hasher.finalize());

    if let Some(mut existing) = db.find_model_by_hash(&hash)? {
        // Re-importing the same bytes from a new location relinks the registry
        // record without creating a duplicate or touching either file.
        db.update_model_source_path(&existing.id, &source_path, &file_name)?;
        db.update_model_metadata(
            &existing.id,
            metadata.architecture.clone(),
            metadata.quantization.clone(),
        )?;
        existing.source_path = Some(source_path.clone());
        existing.file_path = source_path;
        existing.file_name = file_name;
        existing.architecture = metadata.architecture;
        existing.quantization = metadata.quantization;
        existing.is_available = true;
        return Ok(Some(ModelImportResult { model: existing, duplicate: true }));
    }

    let name = metadata.model_name.clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            Path::new(&file_name).file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("GGUF model")
                .to_string()
        });

    let model = ModelRecord {
        id: Uuid::new_v4().to_string(),
        name,
        file_name,
        size_bytes,
        sha256: hash,
        architecture: metadata.architecture,
        quantization: metadata.quantization,
        imported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        is_default: false,
        source_path: Some(source_path.clone()),
        file_path: source_path.clone(),
        is_available: true,
    };

    db.register_model(&model, &source_path)?;
    let mut registered = db.list_models()?.into_iter()
        .find(|item| item.id == model.id)
        .ok_or("Selected GGUF file was not registered")?;
    registered.file_path = source_path;
    registered.is_available = true;
    Ok(Some(ModelImportResult { model: registered, duplicate: false }))
}

#[tauri::command]
pub fn set_default_model(db: State<'_, Database>, model_id: String) -> Result<(), String> {
    db.set_default_model(&model_id)
}

#[tauri::command]
pub fn remove_model(
    db: State<'_, Database>,
    runtime: State<'_, RuntimeManager>,
    model_id: String,
) -> Result<(), String> {
    let _removal = runtime.reserve_removal(&model_id)?;
    // Library removal deletes only the database entry. Original and legacy GGUF
    // files are deliberately left in place.
    db.delete_model_record(&model_id)
}

pub(crate) fn resolve_model_path(
    app: &tauri::AppHandle,
    db: &Database,
    model_id: &str,
) -> Result<std::path::PathBuf, String> {
    let (source_path, legacy_relative_path) = db.model_path_fields(model_id)?;
    let candidate = if let Some(source_path) = source_path {
        std::path::PathBuf::from(source_path)
    } else {
        let relative = std::path::PathBuf::from(legacy_relative_path);
        if relative.components().count() != 1
            || relative.components().any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("Stored legacy model path failed its safety check.".into());
        }
        let app_dir = app.path().app_data_dir()
            .map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
        app_dir.join("models").join(relative)
    };

    let canonical = candidate.canonicalize()
        .map_err(|_| "The original GGUF file is missing or has moved. Re-register it from its current location.".to_string())?;
    if !canonical.is_file() {
        return Err("The registered GGUF path is not a regular file.".into());
    }
    Ok(canonical)
}
