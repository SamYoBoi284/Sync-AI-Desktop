use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use sysinfo::System;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::{
    commands::runtime::{managed_model_path, model_is_in_use},
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
pub fn list_models(app: AppHandle, db: State<'_, Database>) -> Result<Vec<ModelRecord>, String> {
    let mut models = db.list_models()?;
    for model in &mut models {
        if model.architecture.is_some() && model.quantization.is_some() {
            continue;
        }
        let Ok(path) = managed_model_path(&app, &db, &model.id) else { continue; };
        let Ok(metadata) = GgufMetadata::read(&path) else { continue; };
        db.update_model_metadata(&model.id, metadata.architecture.clone(), metadata.quantization.clone())?;
        model.architecture = metadata.architecture;
        model.quantization = metadata.quantization;
    }
    Ok(models)
}

#[tauri::command]
pub fn import_gguf(
    app: AppHandle,
    db: State<'_, Database>,
) -> Result<Option<ModelImportResult>, String> {
    let Some(source) = rfd::FileDialog::new()
        .add_filter("GGUF model", &["gguf"])
        .pick_file()
    else {
        return Ok(None);
    };

    if !source.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("gguf")) {
        return Err("Only GGUF model files can be imported.".into());
    }

    let metadata = GgufMetadata::read(&source)?;
    let file_metadata = fs::metadata(&source)
        .map_err(|e| format!("Cannot inspect selected model: {e}"))?;
    if !file_metadata.is_file() || file_metadata.len() == 0 {
        return Err("Selected model is empty or is not a regular file.".into());
    }

    let size_bytes = file_metadata.len();
    let source_name = source.file_name().and_then(|v| v.to_str())
        .ok_or("Selected filename is not valid UTF-8")?.to_string();

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
        db.update_model_metadata(&existing.id, metadata.architecture.clone(), metadata.quantization.clone())?;
        existing.architecture = metadata.architecture;
        existing.quantization = metadata.quantization;
        return Ok(Some(ModelImportResult { model: existing, duplicate: true }));
    }

    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
    let model_dir = app_dir.join("models");
    fs::create_dir_all(&model_dir)
        .map_err(|e| format!("Cannot create model directory: {e}"))?;

    let id = Uuid::new_v4().to_string();
    let safe_name: String = source_name.chars().map(|c| {
        if c.is_control() || matches!(c, '<' | '>' | '"' | '|' | '?' | '*' | ':' | '/' | '\\') {
            '_'
        } else {
            c
        }
    }).collect();
    let stored_name = format!("{id}_{safe_name}");
    let destination = model_dir.join(&stored_name);

    fs::copy(&source, &destination)
        .map_err(|e| {
            let _ = fs::remove_file(&destination);
            format!("Cannot copy model into managed storage: {e}")
        })?;

    let verification = (|| -> Result<(), String> {
        let copied_size = fs::metadata(&destination)
            .map_err(|e| format!("Cannot verify imported model: {e}"))?.len();
        let mut copied = File::open(&destination)
            .map_err(|e| format!("Cannot verify imported model hash: {e}"))?;
        let mut verify_hasher = Sha256::new();
        loop {
            let count = copied.read(&mut buffer)
                .map_err(|e| format!("Cannot verify imported model hash: {e}"))?;
            if count == 0 { break; }
            verify_hasher.update(&buffer[..count]);
        }
        let copied_hash = format!("{:x}", verify_hasher.finalize());
        if copied_size != size_bytes || copied_hash != hash {
            return Err("Imported model did not match the selected source file after copying.".into());
        }
        Ok(())
    })();

    if let Err(error) = verification {
        let _ = fs::remove_file(&destination);
        return Err(error);
    }

    let fallback_name = Path::new(&source_name).file_stem()
        .and_then(|v| v.to_str()).unwrap_or("GGUF model");
    let name = metadata.model_name.clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| fallback_name.to_string());
    let model = ModelRecord {
        id,
        name,
        file_name: source_name,
        size_bytes,
        sha256: hash,
        architecture: metadata.architecture,
        quantization: metadata.quantization,
        imported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        is_default: false,
    };

    if let Err(error) = db.register_model(&model, &stored_name) {
        let _ = fs::remove_file(&destination);
        if let Some(existing) = db.find_model_by_hash(&model.sha256)? {
            return Ok(Some(ModelImportResult { model: existing, duplicate: true }));
        }
        return Err(error);
    }

    let registered = db.list_models()?.into_iter()
        .find(|item| item.id == model.id)
        .ok_or("Imported model was not registered")?;
    Ok(Some(ModelImportResult { model: registered, duplicate: false }))
}

#[tauri::command]
pub fn set_default_model(db: State<'_, Database>, model_id: String) -> Result<(), String> {
    db.set_default_model(&model_id)
}

#[tauri::command]
pub fn remove_model(
    app: AppHandle,
    db: State<'_, Database>,
    runtime: State<'_, RuntimeManager>,
    model_id: String,
) -> Result<(), String> {
    if model_is_in_use(&runtime, &model_id)? {
        return Err("Wait for this model to finish loading or unload it before removing it.".into());
    }
    let target = managed_model_path(&app, &db, &model_id)?;
    fs::remove_file(&target).map_err(|e| format!("Cannot remove model file: {e}"))?;
    db.delete_model_record(&model_id)
}
