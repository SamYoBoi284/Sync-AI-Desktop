use std::{
    fs::File,
    io::Read,
    path::Path,
};

use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::storage::{workspace::{LibraryFileRecord, LibraryImportResult}, Database};

const MAX_LIBRARY_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[tauri::command]
pub fn import_library_file(db: State<'_, Database>) -> Result<Option<LibraryImportResult>, String> {
    let Some(selected) = rfd::FileDialog::new()
        .set_title("Add a file to Sync AI File Library")
        .pick_file()
    else {
        return Ok(None);
    };
    let record = inspect_selected_file(&selected)?;
    db.register_library_file(record).map(Some)
}

#[tauri::command]
pub fn relink_library_file(db: State<'_, Database>, file_id: String) -> Result<Option<LibraryFileRecord>, String> {
    let Some(selected) = rfd::FileDialog::new()
        .set_title("Relink File Library entry")
        .pick_file()
    else {
        return Ok(None);
    };
    let record = inspect_selected_file(&selected)?;
    db.relink_library_file(&file_id, record).map(Some)
}

#[tauri::command]
pub fn list_library_files(db: State<'_, Database>) -> Result<Vec<LibraryFileRecord>, String> {
    db.list_library_files()
}

#[tauri::command]
pub fn remove_library_file(db: State<'_, Database>, file_id: String) -> Result<(), String> {
    // Only the registry entry is removed. The user's original file is never changed or deleted.
    db.delete_library_file(&file_id)
}

fn inspect_selected_file(selected: &Path) -> Result<LibraryFileRecord, String> {
    let source = selected.canonicalize()
        .map_err(|error| format!("Cannot resolve selected file: {error}"))?;
    let metadata = std::fs::metadata(&source)
        .map_err(|error| format!("Cannot inspect selected file: {error}"))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("Selected item is empty or is not a regular file.".into());
    }
    if metadata.len() > MAX_LIBRARY_FILE_BYTES {
        return Err("File Library currently accepts files up to 512 MiB so hashing stays predictable.".into());
    }

    let file_name = source.file_name().and_then(|value| value.to_str())
        .ok_or("Selected filename is not valid UTF-8")?.to_string();
    let extension = source.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase);
    let mut file = File::open(&source).map_err(|error| format!("Cannot read selected file: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| format!("Cannot hash selected file: {error}"))?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }

    Ok(LibraryFileRecord {
        id: Uuid::new_v4().to_string(),
        file_name,
        extension,
        source_path: source.to_string_lossy().into_owned(),
        size_bytes: metadata.len(),
        sha256: format!("{:x}", hasher.finalize()),
        added_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        is_available: true,
    })
}
