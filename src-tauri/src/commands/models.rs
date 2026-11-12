use std::{fs::{self, File}, io::Read, path::{Path, PathBuf}};
use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use sysinfo::System;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;
use crate::storage::{models::ModelRecord, Database};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRamInfo { pub total_bytes: u64, pub available_bytes: u64 }
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelImportResult { pub model: ModelRecord, pub duplicate: bool }

#[tauri::command]
pub fn get_host_ram_info() -> HostRamInfo {
    let mut system = System::new(); system.refresh_memory();
    HostRamInfo { total_bytes: system.total_memory(), available_bytes: system.available_memory() }
}
#[tauri::command]
pub fn list_models(db: State<'_, Database>) -> Result<Vec<ModelRecord>, String> { db.list_models() }

#[tauri::command]
pub fn import_gguf(app: AppHandle, db: State<'_, Database>) -> Result<Option<ModelImportResult>, String> {
    let Some(source) = rfd::FileDialog::new().add_filter("GGUF model", &["gguf"]).pick_file() else { return Ok(None); };
    if !source.extension().and_then(|v| v.to_str()).is_some_and(|v| v.eq_ignore_ascii_case("gguf")) { return Err("Only GGUF model files can be imported.".into()); }
    let metadata = fs::metadata(&source).map_err(|e| format!("Cannot inspect selected model: {e}"))?;
    if !metadata.is_file() || metadata.len() == 0 { return Err("Selected model is empty or is not a regular file.".into()); }
    let size_bytes = metadata.len();
    let source_name = source.file_name().and_then(|v| v.to_str()).ok_or("Selected filename is not valid UTF-8")?.to_string();
    let mut input = File::open(&source).map_err(|e| format!("Cannot read selected model: {e}"))?;
    let mut hasher = Sha256::new(); let mut buffer = [0_u8; 1024 * 1024];
    loop { let count = input.read(&mut buffer).map_err(|e| format!("Cannot hash selected model: {e}"))?; if count == 0 { break; } hasher.update(&buffer[..count]); }
    let hash = format!("{:x}", hasher.finalize());
    if let Some(existing) = db.find_model_by_hash(&hash)? { return Ok(Some(ModelImportResult { model: existing, duplicate: true })); }
    let app_dir = app.path().app_data_dir().map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
    let model_dir = app_dir.join("models"); fs::create_dir_all(&model_dir).map_err(|e| format!("Cannot create model directory: {e}"))?;
    let id = Uuid::new_v4().to_string();
    let safe_name: String = source_name.chars().map(|c| if matches!(c, '/' | '\\' | ':') { '_' } else { c }).collect();
    let stored_name = format!("{id}_{safe_name}"); let destination = model_dir.join(&stored_name);
    fs::copy(&source, &destination).map_err(|e| format!("Cannot copy model into managed storage: {e}"))?;
    let copied_size = fs::metadata(&destination).map_err(|e| format!("Cannot verify imported model: {e}"))?.len();
    if copied_size != size_bytes { let _ = fs::remove_file(&destination); return Err("Imported model size did not match the selected source file.".into()); }
    let name = Path::new(&source_name).file_stem().and_then(|v| v.to_str()).unwrap_or("GGUF model").to_string();
    let model = ModelRecord { id, name, file_name: source_name, size_bytes, sha256: hash, architecture: None, quantization: None, imported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true), is_default: false };
    if let Err(error) = db.register_model(&model, &stored_name) { let _ = fs::remove_file(&destination); return Err(error); }
    let registered = db.list_models()?.into_iter().find(|item| item.id == model.id).ok_or("Imported model was not registered")?;
    Ok(Some(ModelImportResult { model: registered, duplicate: false }))
}
#[tauri::command]
pub fn set_default_model(db: State<'_, Database>, model_id: String) -> Result<(), String> { db.set_default_model(&model_id) }
#[tauri::command]
pub fn remove_model(app: AppHandle, db: State<'_, Database>, model_id: String) -> Result<(), String> {
    let relative = db.model_relative_path(&model_id)?;
    let app_dir = app.path().app_data_dir().map_err(|e| format!("Cannot resolve Sync data directory: {e}"))?;
    let relative_path = PathBuf::from(&relative);
    if relative_path.components().count() != 1 || relative_path.components().any(|part| !matches!(part, std::path::Component::Normal(_))) { return Err("Stored model path failed the managed-directory safety check.".into()); }
    let target = app_dir.join("models").join(relative_path);
    if target.exists() { fs::remove_file(&target).map_err(|e| format!("Cannot remove model file: {e}"))?; }
    db.delete_model_record(&model_id)
}
