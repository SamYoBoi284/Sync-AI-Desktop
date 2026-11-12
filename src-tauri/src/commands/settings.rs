use tauri::State;

use crate::storage::Database;

#[tauri::command]
pub fn get_setting(db: State<'_, Database>, key: String) -> Result<Option<String>, String> {
    db.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(db: State<'_, Database>, key: String, value_json: String) -> Result<(), String> {
    db.set_setting(key, value_json)
}
