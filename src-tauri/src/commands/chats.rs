use tauri::State;

use crate::storage::models::{
    BackupDocument, ChatDetail, ChatSummary, ImportSummary, MessageDiagnostics, NewMessage,
};
use crate::storage::Database;

#[tauri::command]
pub fn list_chats(db: State<'_, Database>, query: Option<String>, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<ChatSummary>, String> {
    db.list_chats(query, limit, offset)
}

#[tauri::command]
pub fn create_chat(db: State<'_, Database>, title: Option<String>) -> Result<ChatDetail, String> {
    db.create_chat(title)
}

#[tauri::command]
pub fn get_chat(db: State<'_, Database>, chat_id: String) -> Result<ChatDetail, String> {
    db.get_chat(&chat_id)
}

#[tauri::command]
pub fn rename_chat(db: State<'_, Database>, chat_id: String, title: String) -> Result<(), String> {
    db.rename_chat(&chat_id, title)
}

#[tauri::command]
pub fn delete_chat(db: State<'_, Database>, chat_id: String) -> Result<(), String> {
    db.delete_chat(&chat_id)
}

#[tauri::command]
pub fn append_message(db: State<'_, Database>, chat_id: String, message: NewMessage) -> Result<crate::storage::models::ChatMessage, String> {
    db.append_message(&chat_id, message)
}

#[tauri::command]
pub fn update_message_diagnostics(db: State<'_, Database>, chat_id: String, message_id: String, diagnostics: MessageDiagnostics) -> Result<(), String> {
    db.update_diagnostics(&chat_id, &message_id, diagnostics)
}

#[tauri::command]
pub fn export_chat_backup(db: State<'_, Database>) -> Result<BackupDocument, String> {
    db.export_backup()
}

#[tauri::command]
pub fn import_chat_backup(db: State<'_, Database>, backup: BackupDocument) -> Result<ImportSummary, String> {
    db.import_backup(backup)
}
