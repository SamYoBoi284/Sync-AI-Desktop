pub mod models;

use std::path::Path;
use std::sync::Mutex;

use chrono::{SecondsFormat, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use self::models::{
    BackupAppInfo, BackupChat, BackupDocument, BackupMessage, ChatDetail, ChatMessage,
    ChatSummary, ImportSummary, MessageDiagnostics, MessageRole, NewMessage, ModelRecord,
};

const BACKUP_FORMAT: &str = "sync-ai-desktop-backup";
const BACKUP_VERSION: u32 = 1;
const MAX_BACKUP_BYTES: usize = 50 * 1024 * 1024;
const MAX_CHATS: usize = 1_000;
const MAX_MESSAGES: usize = 50_000;
const MAX_MESSAGE_BYTES: usize = 1_000_000;
const MAX_DIAGNOSTICS_BYTES: usize = 64 * 1024;

pub struct Database {
    connection: Mutex<Connection>,
}
impl Database {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create app data directory: {e}"))?;
        }
        let connection = Connection::open(path).map_err(db_error)?;
        connection.pragma_update(None, "foreign_keys", "ON").map_err(db_error)?;
        migrate(&connection)?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    pub fn list_chats(&self, query: Option<String>, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<ChatSummary>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let limit = limit.unwrap_or(50).clamp(1, 200);
        let offset = offset.unwrap_or(0);
        let needle = query.unwrap_or_default().trim().to_string();
        let pattern = format!("%{needle}%");
        let mut statement = connection.prepare(
            "SELECT c.id, c.title, c.created_at, c.updated_at,
             (SELECT COUNT(*) FROM messages m WHERE m.chat_id = c.id)
             FROM chats c
             WHERE (?1 = '' OR c.title LIKE ?2 OR EXISTS (
               SELECT 1 FROM messages m WHERE m.chat_id = c.id AND m.body LIKE ?2
             ))
             ORDER BY c.updated_at DESC LIMIT ?3 OFFSET ?4"
        ).map_err(db_error)?;
        let rows = statement.query_map(params![needle, pattern, limit, offset], |row| {
            Ok(ChatSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get(4)?,
            })
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn create_chat(&self, title: Option<String>) -> Result<ChatDetail, String> {
        let title = validate_title(title.unwrap_or_else(|| "New conversation".to_string()))?;
        let id = Uuid::new_v4().to_string();
        let now = now();
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.execute("INSERT INTO chats (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)", params![id, title, now]).map_err(db_error)?;
        Ok(ChatDetail { id, title, created_at: now.clone(), updated_at: now, messages: Vec::new() })
    }

    pub fn get_chat(&self, chat_id: &str) -> Result<ChatDetail, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        load_chat(&connection, chat_id)?.ok_or_else(|| "Chat not found".to_string())
    }

    pub fn rename_chat(&self, chat_id: &str, title: String) -> Result<(), String> {
        let title = validate_title(title)?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("UPDATE chats SET title = ?1, updated_at = ?2 WHERE id = ?3", params![title, now(), chat_id]).map_err(db_error)?;
        if changed == 0 { return Err("Chat not found".into()); }
        Ok(())
    }

    pub fn delete_chat(&self, chat_id: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("DELETE FROM chats WHERE id = ?1", [chat_id]).map_err(db_error)?;
        if changed == 0 { return Err("Chat not found".into()); }
        Ok(())
    }

    pub fn append_message(&self, chat_id: &str, message: NewMessage) -> Result<ChatMessage, String> {
        validate_message(&message)?;
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let diagnostics_json = serialize_diagnostics(message.diagnostics.as_ref())?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let tx = connection.unchecked_transaction().map_err(db_error)?;
        let position: i64 = tx.query_row("SELECT COALESCE(MAX(position) + 1, 0) FROM messages WHERE chat_id = ?1", [chat_id], |row| row.get(0)).map_err(db_error)?;
        tx.execute("INSERT INTO messages (id, chat_id, role, body, created_at, position, diagnostics_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, chat_id, message.role.as_str(), message.body, created_at, position, diagnostics_json]).map_err(db_error)?;
        tx.execute("UPDATE chats SET updated_at = ?1 WHERE id = ?2", params![now(), chat_id]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(ChatMessage { id, chat_id: chat_id.to_string(), role: message.role, body: message.body, created_at, position, diagnostics: message.diagnostics })
    }

    pub fn append_assistant_message_after(
        &self,
        chat_id: &str,
        user_message_id: &str,
        message: NewMessage,
    ) -> Result<ChatMessage, String> {
        validate_message(&message)?;
        if !matches!(message.role, MessageRole::Assistant) {
            return Err("Only an assistant message can be inserted after a queued user turn".into());
        }

        let id = Uuid::new_v4().to_string();
        let created_at = now();
        let diagnostics_json = serialize_diagnostics(message.diagnostics.as_ref())?;
        let mut connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let tx = connection.transaction().map_err(db_error)?;
        let user_position: i64 = tx.query_row(
            "SELECT position FROM messages WHERE id = ?1 AND chat_id = ?2 AND role = 'user'",
            params![user_message_id, chat_id],
            |row| row.get(0),
        ).optional().map_err(db_error)?
            .ok_or_else(|| "The queued user message was not found in this conversation".to_string())?;

        // Move later messages out of the active position range first, then
        // shift them by one. This avoids UNIQUE(chat_id, position) collisions.
        tx.execute(
            "UPDATE messages SET position = position + 1000000000 WHERE chat_id = ?1 AND position > ?2",
            params![chat_id, user_position],
        ).map_err(db_error)?;
        tx.execute(
            "UPDATE messages SET position = position - 999999999 WHERE chat_id = ?1 AND position >= 1000000000",
            [chat_id],
        ).map_err(db_error)?;

        let assistant_position = user_position + 1;
        tx.execute(
            "INSERT INTO messages (id, chat_id, role, body, created_at, position, diagnostics_json) VALUES (?1, ?2, 'assistant', ?3, ?4, ?5, ?6)",
            params![id, chat_id, message.body, created_at, assistant_position, diagnostics_json],
        ).map_err(db_error)?;
        tx.execute("UPDATE chats SET updated_at = ?1 WHERE id = ?2", params![now(), chat_id]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;

        Ok(ChatMessage {
            id,
            chat_id: chat_id.to_string(),
            role: MessageRole::Assistant,
            body: message.body,
            created_at,
            position: assistant_position,
            diagnostics: message.diagnostics,
        })
    }

    pub fn update_diagnostics(&self, chat_id: &str, message_id: &str, diagnostics: MessageDiagnostics) -> Result<(), String> {
        let json = serialize_diagnostics(Some(&diagnostics))?.ok_or_else(|| "Diagnostics are required".to_string())?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE messages SET diagnostics_json = ?1 WHERE id = ?2 AND chat_id = ?3 AND role = 'assistant'",
            params![json, message_id, chat_id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Assistant message not found".into()); }
        connection.execute("UPDATE chats SET updated_at = ?1 WHERE id = ?2", params![now(), chat_id]).map_err(db_error)?;
        Ok(())
    }

    pub fn export_backup(&self) -> Result<BackupDocument, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT id, title, created_at, updated_at FROM chats ORDER BY created_at ASC").map_err(db_error)?;
        let chats = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))).map_err(db_error)?;
        let mut exported = Vec::new();
        for chat in chats {
            let (id, title, created_at, updated_at) = chat.map_err(db_error)?;
            let detail = load_chat(&connection, &id)?.ok_or_else(|| "Chat disappeared during export".to_string())?;
            exported.push(BackupChat {
                id, title, created_at, updated_at,
                messages: detail.messages.into_iter().map(|m| BackupMessage {
                    id: m.id, role: m.role, body: m.body, created_at: m.created_at,
                    position: m.position, diagnostics: m.diagnostics,
                }).collect(),
            });
        }
        Ok(BackupDocument {
            format: BACKUP_FORMAT.into(),
            format_version: BACKUP_VERSION,
            exported_at: now(),
            app: BackupAppInfo { name: "Sync AI Desktop".into(), version: env!("CARGO_PKG_VERSION").into() },
            chats: exported,
        })
    }

    pub fn import_backup(&self, backup: BackupDocument) -> Result<ImportSummary, String> {
        validate_backup(&backup)?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let tx = connection.unchecked_transaction().map_err(db_error)?;
        let mut imported = 0_u64;
        let mut skipped = 0_u64;
        for chat in &backup.chats {
            let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM chats WHERE id = ?1)", [&chat.id], |r| r.get(0)).map_err(db_error)?;
            if exists { skipped += 1; continue; }
            tx.execute("INSERT INTO chats (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![chat.id, chat.title, chat.created_at, chat.updated_at]).map_err(db_error)?;
            for message in &chat.messages {
                let diagnostics = serialize_diagnostics(message.diagnostics.as_ref())?;
                tx.execute("INSERT INTO messages (id, chat_id, role, body, created_at, position, diagnostics_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![message.id, chat.id, message.role.as_str(), message.body, message.created_at, message.position, diagnostics]).map_err(db_error)?;
            }
            imported += 1;
        }
        tx.commit().map_err(db_error)?;
        Ok(ImportSummary { imported_chats: imported, skipped_existing_chats: skipped })
    }


    pub fn list_models(&self) -> Result<Vec<ModelRecord>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, name, file_name, size_bytes, sha256, architecture, quantization, imported_at, is_default, source_path, relative_path FROM models ORDER BY imported_at DESC"
        ).map_err(db_error)?;
        let rows = statement.query_map([], |row| {
            let source_path: Option<String> = row.get(9)?;
            let legacy_path: String = row.get(10)?;
            let file_path = source_path.clone().unwrap_or(legacy_path);
            Ok(ModelRecord {
                id: row.get(0)?, name: row.get(1)?, file_name: row.get(2)?, size_bytes: row.get(3)?,
                sha256: row.get(4)?, architecture: row.get(5)?, quantization: row.get(6)?,
                imported_at: row.get(7)?, is_default: row.get::<_, i64>(8)? != 0,
                source_path, file_path, is_available: false,
            })
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn find_model_by_hash(&self, hash: &str) -> Result<Option<ModelRecord>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.query_row(
            "SELECT id, name, file_name, size_bytes, sha256, architecture, quantization, imported_at, is_default, source_path, relative_path FROM models WHERE sha256 = ?1",
            [hash],
            |row| {
                let source_path: Option<String> = row.get(9)?;
                let legacy_path: String = row.get(10)?;
                Ok(ModelRecord {
                    id: row.get(0)?, name: row.get(1)?, file_name: row.get(2)?, size_bytes: row.get(3)?,
                    sha256: row.get(4)?, architecture: row.get(5)?, quantization: row.get(6)?,
                    imported_at: row.get(7)?, is_default: row.get::<_, i64>(8)? != 0,
                    file_path: source_path.clone().unwrap_or(legacy_path),
                    source_path, is_available: false,
                })
            }
        ).optional().map_err(db_error)
    }

    pub fn update_model_metadata(
        &self,
        model_id: &str,
        architecture: Option<String>,
        quantization: Option<String>,
    ) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE models SET architecture = ?1, quantization = ?2 WHERE id = ?3",
            params![architecture, quantization, model_id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Model not found".into()); }
        Ok(())
    }

    pub fn register_model(&self, model: &ModelRecord, source_path: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM models)", [], |r| r.get(0)).map_err(db_error)?;
        // Keep the legacy NOT NULL path column populated for pre-v3 databases.
        // New entries always resolve from source_path and never copy model bytes.
        let legacy_placeholder = format!("external:{}", model.id);
        connection.execute(
            "INSERT INTO models (id, name, file_name, relative_path, size_bytes, sha256, architecture, quantization, imported_at, is_default, source_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                model.id, model.name, model.file_name, legacy_placeholder, model.size_bytes,
                model.sha256, model.architecture, model.quantization, model.imported_at,
                if exists { 0 } else { 1 }, source_path
            ],
        ).map_err(db_error)?;
        Ok(())
    }

    pub fn update_model_source_path(&self, model_id: &str, source_path: &str, file_name: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE models SET source_path = ?1, file_name = ?2 WHERE id = ?3",
            params![source_path, file_name, model_id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Model not found".into()); }
        Ok(())
    }

    pub fn model_path_fields(&self, model_id: &str) -> Result<(Option<String>, String), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.query_row(
            "SELECT source_path, relative_path FROM models WHERE id = ?1",
            [model_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(db_error)?.ok_or_else(|| "Model not found".into())
    }

    pub fn model_relative_path(&self, model_id: &str) -> Result<String, String> {
        self.model_path_fields(model_id).map(|(_, legacy)| legacy)
    }

    pub fn set_default_model(&self, model_id: &str) -> Result<(), String> {
        let mut connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let tx = connection.transaction().map_err(db_error)?;
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM models WHERE id = ?1)", [model_id], |r| r.get(0)).map_err(db_error)?;
        if !exists { return Err("Model not found".into()); }
        tx.execute("UPDATE models SET is_default = 0", []).map_err(db_error)?;
        tx.execute("UPDATE models SET is_default = 1 WHERE id = ?1", [model_id]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }

    pub fn delete_model_record(&self, model_id: &str) -> Result<(), String> {
        let mut connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let tx = connection.transaction().map_err(db_error)?;
        let was_default: bool = tx.query_row(
            "SELECT is_default FROM models WHERE id = ?1",
            [model_id],
            |r| r.get::<_, i64>(0).map(|v| v != 0),
        ).optional().map_err(db_error)?.ok_or_else(|| "Model not found".to_string())?;
        tx.execute("DELETE FROM models WHERE id = ?1", [model_id]).map_err(db_error)?;
        if was_default {
            tx.execute(
                "UPDATE models SET is_default = 1 WHERE id = (SELECT id FROM models ORDER BY imported_at DESC LIMIT 1)",
                [],
            ).map_err(db_error)?;
        }
        tx.commit().map_err(db_error)
    }

    pub fn set_setting(&self, key: String, value_json: String) -> Result<(), String> {
        if key.trim().is_empty() || key.len() > 128 || value_json.len() > 64 * 1024 { return Err("Invalid setting key or value size".into()); }
        serde_json::from_str::<serde_json::Value>(&value_json).map_err(|_| "Setting value must be valid JSON".to_string())?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.execute("INSERT INTO app_settings (key, value_json, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
            params![key, value_json, now()]).map_err(db_error)?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.query_row("SELECT value_json FROM app_settings WHERE key = ?1", [key], |r| r.get(0)).optional().map_err(db_error)
    }
}

fn migrate(connection: &Connection) -> Result<(), String> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);"
    ).map_err(db_error)?;
    let mut version: u32 = connection.query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |r| r.get(0)).map_err(db_error)?;
    if version > 3 { return Err("Database schema is newer than this application version".into()); }
    if version < 1 {
        let tx = connection.unchecked_transaction().map_err(db_error)?;
        tx.execute_batch(
            "CREATE TABLE chats (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
                role TEXT NOT NULL CHECK(role IN ('system', 'user', 'assistant')),
                body TEXT NOT NULL,
                created_at TEXT NOT NULL,
                position INTEGER NOT NULL CHECK(position >= 0),
                diagnostics_json TEXT NULL,
                UNIQUE(chat_id, position),
                CHECK(role = 'assistant' OR diagnostics_json IS NULL)
             );
             CREATE INDEX idx_chats_updated_at ON chats(updated_at DESC);
             CREATE INDEX idx_messages_chat_position ON messages(chat_id, position);
             CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );"
        ).map_err(db_error)?;
        tx.execute("INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)", [now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        version = 1;
    }
    if version < 2 {
        let tx = connection.unchecked_transaction().map_err(db_error)?;
        tx.execute_batch("CREATE TABLE models (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, file_name TEXT NOT NULL,
            relative_path TEXT NOT NULL UNIQUE, size_bytes INTEGER NOT NULL CHECK(size_bytes > 0),
            sha256 TEXT NOT NULL UNIQUE, architecture TEXT NULL, quantization TEXT NULL,
            imported_at TEXT NOT NULL, is_default INTEGER NOT NULL DEFAULT 0 CHECK(is_default IN (0, 1))
        ); CREATE UNIQUE INDEX idx_models_single_default ON models(is_default) WHERE is_default = 1;")
            .map_err(db_error)?;
        tx.execute("INSERT INTO schema_migrations (version, applied_at) VALUES (2, ?1)", [now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        version = 2;
    }
    if version < 3 {
        let tx = connection.unchecked_transaction().map_err(db_error)?;
        tx.execute_batch("ALTER TABLE models ADD COLUMN source_path TEXT NULL;")
            .map_err(db_error)?;
        tx.execute("INSERT INTO schema_migrations (version, applied_at) VALUES (3, ?1)", [now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
    }
    Ok(())
}

fn load_chat(connection: &Connection, chat_id: &str) -> Result<Option<ChatDetail>, String> {
    let header = connection.query_row(
        "SELECT id, title, created_at, updated_at FROM chats WHERE id = ?1", [chat_id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
    ).optional().map_err(db_error)?;
    let Some((id, title, created_at, updated_at)) = header else { return Ok(None); };
    let mut statement = connection.prepare("SELECT id, role, body, created_at, position, diagnostics_json FROM messages WHERE chat_id = ?1 ORDER BY position ASC").map_err(db_error)?;
    let rows = statement.query_map([chat_id], |r| {
        let role: String = r.get(1)?;
        let diagnostics: Option<String> = r.get(5)?;
        Ok((r.get::<_, String>(0)?, role, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, i64>(4)?, diagnostics))
    }).map_err(db_error)?;
    let mut messages = Vec::new();
    for row in rows {
        let (message_id, role, body, created, position, diagnostics) = row.map_err(db_error)?;
        if position < 0 { return Err("Stored message position is invalid".into()); }
        let role = parse_role(&role)?;
        let diagnostics = diagnostics.map(|v| serde_json::from_str::<MessageDiagnostics>(&v).map_err(|e| e.to_string())).transpose()?;
        messages.push(ChatMessage { id: message_id, chat_id: id.clone(), role, body, created_at: created, position, diagnostics });
    }
    Ok(Some(ChatDetail { id, title, created_at, updated_at, messages }))
}

fn validate_title(title: String) -> Result<String, String> {
    let value = title.trim().to_string();
    if value.is_empty() || value.chars().count() > 200 { return Err("Chat title must contain 1 to 200 characters".into()); }
    Ok(value)
}

fn validate_message(message: &NewMessage) -> Result<(), String> {
    if message.body.trim().is_empty() || message.body.len() > MAX_MESSAGE_BYTES { return Err("Message body is empty or exceeds the size limit".into()); }
    if message.diagnostics.is_some() && !matches!(message.role, MessageRole::Assistant) { return Err("Diagnostics can only be attached to assistant messages".into()); }
    let _ = serialize_diagnostics(message.diagnostics.as_ref())?;
    Ok(())
}

fn serialize_diagnostics(value: Option<&MessageDiagnostics>) -> Result<Option<String>, String> {
    let Some(value) = value else { return Ok(None); };
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    if json.len() > MAX_DIAGNOSTICS_BYTES { return Err("Diagnostics exceed the size limit".into()); }
    Ok(Some(json))
}

fn validate_backup(backup: &BackupDocument) -> Result<(), String> {
    let encoded = serde_json::to_vec(backup).map_err(|e| e.to_string())?;
    if encoded.len() > MAX_BACKUP_BYTES { return Err("Backup exceeds the 50 MiB size limit".into()); }
    if backup.format != BACKUP_FORMAT || backup.format_version != BACKUP_VERSION { return Err("Unsupported backup format or version".into()); }
    if backup.chats.len() > MAX_CHATS { return Err("Backup contains too many chats".into()); }
    let mut chat_ids = std::collections::HashSet::new();
    let mut message_ids = std::collections::HashSet::new();
    let mut total_messages = 0_usize;
    for chat in &backup.chats {
        if chat.id.is_empty() || chat.id.len() > 128 || !chat_ids.insert(chat.id.as_str()) { return Err("Backup contains an invalid or duplicate chat ID".into()); }
        if chat.title.trim().is_empty() || chat.title.chars().count() > 200 { return Err("Backup contains an invalid chat title".into()); }
        if chrono::DateTime::parse_from_rfc3339(&chat.created_at).is_err() || chrono::DateTime::parse_from_rfc3339(&chat.updated_at).is_err() { return Err("Backup contains an invalid chat timestamp".into()); }
        total_messages += chat.messages.len();
        if total_messages > MAX_MESSAGES { return Err("Backup contains too many messages".into()); }
        let mut positions = std::collections::HashSet::new();
        for message in &chat.messages {
            if message.id.is_empty() || message.id.len() > 128 || !message_ids.insert(message.id.as_str()) { return Err("Backup contains an invalid or duplicate message ID".into()); }
            if message.body.trim().is_empty() || message.body.len() > MAX_MESSAGE_BYTES { return Err("Backup contains an invalid or oversized message".into()); }
            if chrono::DateTime::parse_from_rfc3339(&message.created_at).is_err() || message.position < 0 || !positions.insert(message.position) { return Err("Backup contains invalid message ordering or timestamp".into()); }
            if message.diagnostics.is_some() && !matches!(message.role, MessageRole::Assistant) { return Err("Only assistant messages may contain diagnostics".into()); }
            let _ = serialize_diagnostics(message.diagnostics.as_ref())?;
        }
    }
    Ok(())
}

fn parse_role(role: &str) -> Result<MessageRole, String> {
    match role { "system" => Ok(MessageRole::System), "user" => Ok(MessageRole::User), "assistant" => Ok(MessageRole::Assistant), _ => Err("Invalid stored message role".into()) }
}

fn now() -> String { Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true) }
fn db_error(error: rusqlite::Error) -> String { format!("Local database error: {error}") }

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_db() -> Database {
        let connection = Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "foreign_keys", "ON").unwrap();
        migrate(&connection).unwrap();
        Database { connection: Mutex::new(connection) }
    }

    #[test]
    fn migration_creates_schema_and_foreign_keys() {
        let db = memory_db();
        let connection = db.connection.lock().unwrap();
        let count: u32 = connection.query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
        let model_table: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'models')", [], |r| r.get(0)).unwrap();
        assert!(model_table);
    }

    #[test]
    fn diagnostics_are_saved_on_the_assistant_message() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let diagnostics = MessageDiagnostics {
            generation_id: Some("gen-1".into()), model_id: None, model_label: None,
            backend: Some("llama.cpp".into()), backend_version: None, status: Some("completed".into()),
            prompt_tokens: Some(12), generated_tokens: Some(8), prompt_eval_ms: None, generation_ms: None,
            tokens_per_second: None, context_size: None, max_context_size: None, temperature: None,
            stop_reason: None, error_code: None, error_message: None, started_at: None, finished_at: None,
        };
        db.append_message(&chat.id, NewMessage { role: MessageRole::Assistant, body: "hello".into(), diagnostics: Some(diagnostics) }).unwrap();
        let loaded = db.get_chat(&chat.id).unwrap();
        assert_eq!(loaded.messages[0].diagnostics.as_ref().unwrap().generation_id.as_deref(), Some("gen-1"));
    }

    #[test]
    fn backup_round_trip_preserves_message_diagnostics() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let diagnostics = MessageDiagnostics {
            generation_id: Some("generation-a".into()), model_id: None, model_label: None,
            backend: None, backend_version: None, status: Some("completed".into()),
            prompt_tokens: Some(5), generated_tokens: Some(3), prompt_eval_ms: None, generation_ms: None,
            tokens_per_second: None, context_size: None, max_context_size: None, temperature: None,
            stop_reason: None, error_code: None, error_message: None, started_at: None, finished_at: None,
        };
        db.append_message(&chat.id, NewMessage { role: MessageRole::Assistant, body: "answer".into(), diagnostics: Some(diagnostics) }).unwrap();
        let backup = db.export_backup().unwrap();
        let other = memory_db();
        let result = other.import_backup(backup).unwrap();
        assert_eq!(result.imported_chats, 1);
        assert!(other.get_chat(&chat.id).unwrap().messages[0].diagnostics.is_some());
    }

    #[test]
    fn import_skips_existing_chat_ids() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let backup = db.export_backup().unwrap();
        let result = db.import_backup(backup).unwrap();
        assert_eq!(result.skipped_existing_chats, 1);
        assert_eq!(result.imported_chats, 0);
    }
    #[test]
    fn diagnostics_are_rejected_on_user_messages() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let diagnostics = MessageDiagnostics {
            generation_id: None, model_id: None, model_label: None, backend: None, backend_version: None,
            status: None, prompt_tokens: None, generated_tokens: None, prompt_eval_ms: None,
            generation_ms: None, tokens_per_second: None, context_size: None, max_context_size: None,
            temperature: None, stop_reason: None, error_code: None, error_message: None,
            started_at: None, finished_at: None,
        };
        let result = db.append_message(&chat.id, NewMessage {
            role: MessageRole::User, body: "hello".into(), diagnostics: Some(diagnostics),
        });
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_chat_ids_inside_backup_are_rejected() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let mut backup = db.export_backup().unwrap();
        let duplicate = backup.chats[0].clone();
        backup.chats.push(duplicate);
        assert!(db.import_backup(backup).is_err());
        assert_eq!(db.list_chats(None, None, None).unwrap().len(), 1);
        assert_eq!(chat.title, "New conversation");
    }

    #[test]
    fn model_registry_keeps_one_default_and_promotes_a_fallback() {
        let db = memory_db();
        let make_model = |id: &str, hash: &str| ModelRecord {
            id: id.into(), name: id.into(), file_name: format!("{id}.gguf"), size_bytes: 1,
            sha256: hash.into(), architecture: None, quantization: None,
            imported_at: now(), is_default: false, source_path: None,
            file_path: String::new(), is_available: false,
        };
        let first = make_model("model-a", "hash-a");
        let second = make_model("model-b", "hash-b");
        db.register_model(&first, "model-a.gguf").unwrap();
        db.register_model(&second, "model-b.gguf").unwrap();
        assert!(db.list_models().unwrap().iter().find(|m| m.id == "model-a").unwrap().is_default);
        db.set_default_model("model-b").unwrap();
        db.delete_model_record("model-b").unwrap();
        let remaining = db.list_models().unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining[0].is_default);
    }

    #[test]
    fn queued_assistant_reply_is_inserted_before_later_user_turns() {
        let db = memory_db();
        let chat = db.create_chat(None).unwrap();
        let first = db.append_message(&chat.id, NewMessage {
            role: MessageRole::User, body: "first question".into(), diagnostics: None,
        }).unwrap();
        let second = db.append_message(&chat.id, NewMessage {
            role: MessageRole::User, body: "queued question".into(), diagnostics: None,
        }).unwrap();

        db.append_assistant_message_after(&chat.id, &first.id, NewMessage {
            role: MessageRole::Assistant, body: "first answer".into(), diagnostics: None,
        }).unwrap();

        let messages = db.get_chat(&chat.id).unwrap().messages;
        assert_eq!(messages.iter().map(|message| message.body.as_str()).collect::<Vec<_>>(),
            vec!["first question", "first answer", "queued question"]);
        assert_eq!(messages[0].position, 0);
        assert_eq!(messages[1].position, 1);
        assert_eq!(messages[2].position, 2);
        assert_eq!(messages[2].id, second.id);
    }

}
