use std::path::Path;

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{db_error, now, Database};

const MAX_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
const MAX_NOTE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TASK_DETAILS_BYTES: usize = 32 * 1024;
const MAX_CONTEXT_BYTES: usize = 48 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceArtifact {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
    pub source_chat_id: Option<String>,
    pub source_message_id: Option<String>,
    pub include_in_context: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceArtifactSummary {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub include_in_context: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWorkspaceArtifact {
    pub title: String,
    pub kind: String,
    pub content: String,
    pub include_in_context: bool,
    pub source_chat_id: Option<String>,
    pub source_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContextEntry {
    pub kind: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub title: String,
    pub details: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub title: String,
    pub details: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub pinned: bool,
    pub include_in_context: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub include_in_context: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    pub title: String,
    pub content: String,
    pub pinned: bool,
    pub include_in_context: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFileRecord {
    pub id: String,
    pub file_name: String,
    pub extension: Option<String>,
    pub source_path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub added_at: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryImportResult {
    pub file: LibraryFileRecord,
    pub duplicate: bool,
}

impl Database {
    pub fn list_workspace_artifacts(&self, query: Option<String>) -> Result<Vec<WorkspaceArtifactSummary>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let needle = query.unwrap_or_default().trim().to_string();
        let pattern = format!("%{needle}%");
        let mut statement = connection.prepare(
            "SELECT id, title, kind, include_in_context, created_at, updated_at
             FROM workspace_artifacts
             WHERE (?1 = '' OR title LIKE ?2 OR content LIKE ?2)
             ORDER BY updated_at DESC LIMIT 200"
        ).map_err(db_error)?;
        let rows = statement.query_map(params![needle, pattern], artifact_summary_from_row).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn get_workspace_artifact(&self, id: &str) -> Result<WorkspaceArtifact, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        load_artifact(&connection, id)?.ok_or_else(|| "Workspace artifact not found".to_string())
    }

    pub fn create_workspace_artifact(&self, input: NewWorkspaceArtifact) -> Result<WorkspaceArtifact, String> {
        validate_artifact(&input)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        validate_artifact_source(&connection, &input)?;
        connection.execute(
            "INSERT INTO workspace_artifacts
             (id, title, kind, content, source_chat_id, source_message_id, include_in_context, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                id, input.title.trim(), input.kind, input.content, input.source_chat_id,
                input.source_message_id, input.include_in_context as i64, timestamp
            ],
        ).map_err(db_error)?;
        load_artifact(&connection, &id)?.ok_or_else(|| "Created workspace artifact could not be loaded".into())
    }

    pub fn update_workspace_artifact(&self, id: &str, input: NewWorkspaceArtifact) -> Result<WorkspaceArtifact, String> {
        validate_artifact(&input)?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE workspace_artifacts
             SET title = ?1, kind = ?2, content = ?3, include_in_context = ?4, updated_at = ?5
             WHERE id = ?6",
            params![input.title.trim(), input.kind, input.content, input.include_in_context as i64, now(), id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Workspace artifact not found".into()); }
        load_artifact(&connection, id)?.ok_or_else(|| "Updated workspace artifact could not be loaded".into())
    }

    pub fn delete_workspace_artifact(&self, id: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("DELETE FROM workspace_artifacts WHERE id = ?1", [id]).map_err(db_error)?;
        if changed == 0 { return Err("Workspace artifact not found".into()); }
        Ok(())
    }

    pub fn list_workspace_context(&self) -> Result<Vec<WorkspaceContextEntry>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = connection.prepare(
            "SELECT source_kind, title, content FROM (
                 SELECT 'note' AS source_kind, title, content, updated_at, pinned AS priority
                 FROM notes WHERE include_in_context = 1
                 UNION ALL
                 SELECT kind AS source_kind, title, content, updated_at, 0 AS priority
                 FROM workspace_artifacts WHERE include_in_context = 1
             )
             ORDER BY priority DESC, updated_at DESC LIMIT 20"
        ).map_err(db_error)?;
        let rows = statement.query_map([], |row| {
            Ok(WorkspaceContextEntry {
                kind: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
            })
        }).map_err(db_error)?;

        let mut output = Vec::new();
        let mut remaining = MAX_CONTEXT_BYTES;
        for row in rows {
            let mut entry = row.map_err(db_error)?;
            if remaining == 0 { break; }
            let overhead = entry.title.len().saturating_add(entry.kind.len()).saturating_add(64);
            if overhead >= remaining { break; }
            let content_budget = remaining - overhead;
            if entry.content.len() > content_budget {
                let mut end = content_budget;
                while !entry.content.is_char_boundary(end) { end -= 1; }
                entry.content.truncate(end);
            }
            remaining = remaining.saturating_sub(overhead.saturating_add(entry.content.len()));
            output.push(entry);
        }
        Ok(output)
    }

    pub fn list_tasks(&self, query: Option<String>) -> Result<Vec<TaskRecord>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let needle = query.unwrap_or_default().trim().to_string();
        let pattern = format!("%{needle}%");
        let mut statement = connection.prepare(
            "SELECT id, title, details, status, priority, due_date, created_at, updated_at
             FROM tasks
             WHERE (?1 = '' OR title LIKE ?2 OR details LIKE ?2)
             ORDER BY CASE status WHEN 'done' THEN 2 WHEN 'in-progress' THEN 1 ELSE 0 END,
                      CASE WHEN due_date IS NULL THEN 1 ELSE 0 END, due_date ASC, updated_at DESC
             LIMIT 200"
        ).map_err(db_error)?;
        let rows = statement.query_map(params![needle, pattern], task_from_row).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn create_task(&self, input: TaskInput) -> Result<TaskRecord, String> {
        validate_task(&input)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.execute(
            "INSERT INTO tasks (id, title, details, status, priority, due_date, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![id, input.title.trim(), input.details, input.status, input.priority, input.due_date, timestamp],
        ).map_err(db_error)?;
        load_task(&connection, &id)?.ok_or_else(|| "Created task could not be loaded".into())
    }

    pub fn update_task(&self, id: &str, input: TaskInput) -> Result<TaskRecord, String> {
        validate_task(&input)?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE tasks SET title = ?1, details = ?2, status = ?3, priority = ?4, due_date = ?5, updated_at = ?6 WHERE id = ?7",
            params![input.title.trim(), input.details, input.status, input.priority, input.due_date, now(), id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Task not found".into()); }
        load_task(&connection, id)?.ok_or_else(|| "Updated task could not be loaded".into())
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("DELETE FROM tasks WHERE id = ?1", [id]).map_err(db_error)?;
        if changed == 0 { return Err("Task not found".into()); }
        Ok(())
    }

    pub fn list_notes(&self, query: Option<String>) -> Result<Vec<NoteSummary>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let needle = query.unwrap_or_default().trim().to_string();
        let pattern = format!("%{needle}%");
        let mut statement = connection.prepare(
            "SELECT id, title, pinned, include_in_context, created_at, updated_at
             FROM notes WHERE (?1 = '' OR title LIKE ?2 OR content LIKE ?2)
             ORDER BY pinned DESC, updated_at DESC LIMIT 500"
        ).map_err(db_error)?;
        let rows = statement.query_map(params![needle, pattern], note_summary_from_row).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn get_note(&self, id: &str) -> Result<NoteRecord, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        load_note(&connection, id)?.ok_or_else(|| "Note not found".to_string())
    }

    pub fn create_note(&self, input: NoteInput) -> Result<NoteRecord, String> {
        validate_note(&input)?;
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        connection.execute(
            "INSERT INTO notes (id, title, content, pinned, include_in_context, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, input.title.trim(), input.content, input.pinned as i64, input.include_in_context as i64, timestamp],
        ).map_err(db_error)?;
        load_note(&connection, &id)?.ok_or_else(|| "Created note could not be loaded".into())
    }

    pub fn update_note(&self, id: &str, input: NoteInput) -> Result<NoteRecord, String> {
        validate_note(&input)?;
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute(
            "UPDATE notes SET title = ?1, content = ?2, pinned = ?3, include_in_context = ?4, updated_at = ?5 WHERE id = ?6",
            params![input.title.trim(), input.content, input.pinned as i64, input.include_in_context as i64, now(), id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("Note not found".into()); }
        load_note(&connection, id)?.ok_or_else(|| "Updated note could not be loaded".into())
    }

    pub fn delete_note(&self, id: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("DELETE FROM notes WHERE id = ?1", [id]).map_err(db_error)?;
        if changed == 0 { return Err("Note not found".into()); }
        Ok(())
    }

    pub fn list_library_files(&self) -> Result<Vec<LibraryFileRecord>, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, file_name, extension, source_path, size_bytes, sha256, added_at FROM library_files ORDER BY added_at DESC"
        ).map_err(db_error)?;
        let rows = statement.query_map([], |row| {
            let source_path: String = row.get(3)?;
            Ok(LibraryFileRecord {
                id: row.get(0)?,
                file_name: row.get(1)?,
                extension: row.get(2)?,
                is_available: Path::new(&source_path).is_file(),
                source_path,
                size_bytes: row.get::<_, i64>(4)?.max(0) as u64,
                sha256: row.get(5)?,
                added_at: row.get(6)?,
            })
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
    }

    pub fn register_library_file(&self, record: LibraryFileRecord) -> Result<LibraryImportResult, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let hash_id: Option<String> = connection.query_row(
            "SELECT id FROM library_files WHERE sha256 = ?1", [&record.sha256], |row| row.get(0)
        ).optional().map_err(db_error)?;
        let path_id: Option<String> = connection.query_row(
            "SELECT id FROM library_files WHERE source_path = ?1", [&record.source_path], |row| row.get(0)
        ).optional().map_err(db_error)?;

        let duplicate = hash_id.is_some();
        if let Some(existing_id) = hash_id {
            if path_id.as_deref().is_some_and(|path_id| path_id != existing_id) {
                return Err("This path is already registered as a different file. Remove or relink that entry first.".into());
            }
            connection.execute(
                "UPDATE library_files SET file_name = ?1, extension = ?2, source_path = ?3, size_bytes = ?4 WHERE id = ?5",
                params![record.file_name, record.extension, record.source_path, record.size_bytes as i64, existing_id],
            ).map_err(db_error)?;
            let file = load_library_file(&connection, &existing_id)?.ok_or("Registered file could not be loaded")?;
            return Ok(LibraryImportResult { file, duplicate: true });
        }

        if let Some(existing_id) = path_id {
            connection.execute(
                "UPDATE library_files SET file_name = ?1, extension = ?2, size_bytes = ?3, sha256 = ?4, added_at = ?5 WHERE id = ?6",
                params![record.file_name, record.extension, record.size_bytes as i64, record.sha256, now(), existing_id],
            ).map_err(db_error)?;
            let file = load_library_file(&connection, &existing_id)?.ok_or("Updated file record could not be loaded")?;
            return Ok(LibraryImportResult { file, duplicate: false });
        }

        connection.execute(
            "INSERT INTO library_files (id, file_name, extension, source_path, size_bytes, sha256, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![record.id, record.file_name, record.extension, record.source_path, record.size_bytes as i64, record.sha256, record.added_at],
        ).map_err(db_error)?;
        let file = load_library_file(&connection, &record.id)?.ok_or("Imported file record could not be loaded")?;
        Ok(LibraryImportResult { file, duplicate: false })
    }

    pub fn relink_library_file(&self, id: &str, record: LibraryFileRecord) -> Result<LibraryFileRecord, String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let existing_hash: Option<String> = connection.query_row(
            "SELECT sha256 FROM library_files WHERE id = ?1",
            [id], |row| row.get(0)
        ).optional().map_err(db_error)?;
        let Some(existing_hash) = existing_hash else {
            return Err("File Library entry not found".into());
        };
        if existing_hash != record.sha256 {
            return Err("The selected file does not match the original SHA-256 fingerprint. Relinking requires the same file contents.".into());
        }
        let duplicate_id: Option<String> = connection.query_row(
            "SELECT id FROM library_files WHERE sha256 = ?1 AND id <> ?2",
            params![record.sha256, id], |row| row.get(0)
        ).optional().map_err(db_error)?;
        if duplicate_id.is_some() {
            return Err("Another File Library entry already tracks these exact bytes.".into());
        }
        let changed = connection.execute(
            "UPDATE library_files SET file_name = ?1, extension = ?2, source_path = ?3, size_bytes = ?4, added_at = ?5 WHERE id = ?6",
            params![record.file_name, record.extension, record.source_path, record.size_bytes as i64, now(), id],
        ).map_err(db_error)?;
        if changed == 0 { return Err("File Library entry not found".into()); }
        load_library_file(&connection, id)?.ok_or_else(|| "Relinked file could not be loaded".into())
    }

    pub fn delete_library_file(&self, id: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database lock poisoned".to_string())?;
        let changed = connection.execute("DELETE FROM library_files WHERE id = ?1", [id]).map_err(db_error)?;
        if changed == 0 { return Err("File Library entry not found".into()); }
        Ok(())
    }
}

fn artifact_summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkspaceArtifactSummary> {
    Ok(WorkspaceArtifactSummary {
        id: row.get(0)?, title: row.get(1)?, kind: row.get(2)?,
        include_in_context: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?, updated_at: row.get(5)?,
    })
}

fn artifact_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkspaceArtifact> {
    Ok(WorkspaceArtifact {
        id: row.get(0)?, title: row.get(1)?, kind: row.get(2)?, content: row.get(3)?,
        source_chat_id: row.get(4)?, source_message_id: row.get(5)?,
        include_in_context: row.get::<_, i64>(6)? != 0, created_at: row.get(7)?, updated_at: row.get(8)?,
    })
}

fn load_artifact(connection: &Connection, id: &str) -> Result<Option<WorkspaceArtifact>, String> {
    connection.query_row(
        "SELECT id, title, kind, content, source_chat_id, source_message_id, include_in_context, created_at, updated_at
         FROM workspace_artifacts WHERE id = ?1",
        [id], artifact_from_row
    ).optional().map_err(db_error)
}

fn task_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskRecord> {
    Ok(TaskRecord {
        id: row.get(0)?, title: row.get(1)?, details: row.get(2)?, status: row.get(3)?,
        priority: row.get(4)?, due_date: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)?,
    })
}

fn load_task(connection: &Connection, id: &str) -> Result<Option<TaskRecord>, String> {
    connection.query_row(
        "SELECT id, title, details, status, priority, due_date, created_at, updated_at FROM tasks WHERE id = ?1",
        [id], task_from_row
    ).optional().map_err(db_error)
}

fn note_summary_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteSummary> {
    Ok(NoteSummary {
        id: row.get(0)?, title: row.get(1)?, pinned: row.get::<_, i64>(2)? != 0,
        include_in_context: row.get::<_, i64>(3)? != 0,
        created_at: row.get(4)?, updated_at: row.get(5)?,
    })
}

fn note_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteRecord> {
    Ok(NoteRecord {
        id: row.get(0)?, title: row.get(1)?, content: row.get(2)?,
        pinned: row.get::<_, i64>(3)? != 0, include_in_context: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?, updated_at: row.get(6)?,
    })
}

fn load_note(connection: &Connection, id: &str) -> Result<Option<NoteRecord>, String> {
    connection.query_row(
        "SELECT id, title, content, pinned, include_in_context, created_at, updated_at FROM notes WHERE id = ?1",
        [id], note_from_row
    ).optional().map_err(db_error)
}

fn load_library_file(connection: &Connection, id: &str) -> Result<Option<LibraryFileRecord>, String> {
    connection.query_row(
        "SELECT id, file_name, extension, source_path, size_bytes, sha256, added_at FROM library_files WHERE id = ?1",
        [id],
        |row| {
            let source_path: String = row.get(3)?;
            Ok(LibraryFileRecord {
                id: row.get(0)?, file_name: row.get(1)?, extension: row.get(2)?,
                is_available: Path::new(&source_path).is_file(), source_path,
                size_bytes: row.get::<_, i64>(4)?.max(0) as u64, sha256: row.get(5)?, added_at: row.get(6)?,
            })
        },
    ).optional().map_err(db_error)
}

fn validate_title(value: &str, label: &str) -> Result<(), String> {
    let title = value.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err(format!("{label} must contain 1 to 200 characters."));
    }
    Ok(())
}

fn validate_artifact_source(connection: &Connection, input: &NewWorkspaceArtifact) -> Result<(), String> {
    match (&input.source_chat_id, &input.source_message_id) {
        (None, None) => Ok(()),
        (Some(chat_id), Some(message_id)) => {
            let message_chat_id: Option<String> = connection.query_row(
                "SELECT chat_id FROM messages WHERE id = ?1",
                [message_id],
                |row| row.get(0),
            ).optional().map_err(db_error)?;
            if message_chat_id.as_deref() != Some(chat_id.as_str()) {
                return Err("Workspace source message does not belong to the supplied source conversation.".into());
            }
            Ok(())
        }
        _ => Err("Workspace source conversation and source message must be supplied together.".into()),
    }
}

fn validate_artifact(input: &NewWorkspaceArtifact) -> Result<(), String> {
    validate_title(&input.title, "Artifact title")?;
    if !matches!(input.kind.as_str(), "document" | "code" | "snippet" | "report" | "other") {
        return Err("Unsupported Workspace artifact type.".into());
    }
    if input.content.len() > MAX_ARTIFACT_BYTES {
        return Err("Workspace artifact exceeds the 2 MiB content limit.".into());
    }
    Ok(())
}

fn validate_task(input: &TaskInput) -> Result<(), String> {
    validate_title(&input.title, "Task title")?;
    if input.details.len() > MAX_TASK_DETAILS_BYTES { return Err("Task details exceed the 32 KiB limit.".into()); }
    if !matches!(input.status.as_str(), "todo" | "in-progress" | "done") { return Err("Invalid task status.".into()); }
    if !matches!(input.priority.as_str(), "low" | "normal" | "high") { return Err("Invalid task priority.".into()); }
    if let Some(date) = &input.due_date {
        if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() { return Err("Task due date must use YYYY-MM-DD.".into()); }
    }
    Ok(())
}

fn validate_note(input: &NoteInput) -> Result<(), String> {
    validate_title(&input.title, "Note title")?;
    if input.content.len() > MAX_NOTE_BYTES { return Err("Note exceeds the 2 MiB content limit.".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn memory_db() -> Database {
        let connection = Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "foreign_keys", "ON").unwrap();
        super::migrate(&connection).unwrap();
        Database { connection: Mutex::new(connection) }
    }

    #[test]
    fn artifacts_and_notes_are_persistent_context_sources() {
        let db = memory_db();
        let note = db.create_note(NoteInput {
            title: "Project goals".into(), content: "Keep the UI local-first.".into(),
            pinned: true, include_in_context: true,
        }).unwrap();
        let artifact = db.create_workspace_artifact(NewWorkspaceArtifact {
            title: "Plan".into(), kind: "document".into(), content: "Phase 5 notes".into(),
            include_in_context: true, source_chat_id: None, source_message_id: None,
        }).unwrap();
        let context = db.list_workspace_context().unwrap();
        assert_eq!(context.len(), 2);
        assert_eq!(context[0].title, note.title);
        assert!(db.get_workspace_artifact(&artifact.id).is_ok());
    }

    #[test]
    fn list_views_return_summaries_and_detail_fetches_full_content() {
        let db = memory_db();
        let artifact = db.create_workspace_artifact(NewWorkspaceArtifact {
            title: "Long report".into(),
            kind: "report".into(),
            content: "Full artifact body remains available on demand.".into(),
            include_in_context: false,
            source_chat_id: None,
            source_message_id: None,
        }).unwrap();
        let artifacts = db.list_workspace_artifacts(None).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].id, artifact.id);
        assert_eq!(db.get_workspace_artifact(&artifact.id).unwrap().content, "Full artifact body remains available on demand.");

        let note = db.create_note(NoteInput {
            title: "Private note".into(),
            content: "Full note body".into(),
            pinned: false,
            include_in_context: false,
        }).unwrap();
        let notes = db.list_notes(None).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note.id);
        assert_eq!(db.get_note(&note.id).unwrap().content, "Full note body");
    }

    #[test]
    fn artifact_source_message_must_belong_to_source_chat() {
        let db = memory_db();
        {
            let connection = db.connection.lock().unwrap();
            connection.execute(
                "INSERT INTO chats (id, title, created_at, updated_at) VALUES ('chat-a', 'A', 'now', 'now')",
                [],
            ).unwrap();
            connection.execute(
                "INSERT INTO chats (id, title, created_at, updated_at) VALUES ('chat-b', 'B', 'now', 'now')",
                [],
            ).unwrap();
            connection.execute(
                "INSERT INTO messages (id, chat_id, role, body, created_at, position, diagnostics_json) VALUES ('message-a', 'chat-a', 'assistant', 'reply', 'now', 0, NULL)",
                [],
            ).unwrap();
        }
        let result = db.create_workspace_artifact(NewWorkspaceArtifact {
            title: "Mismatched source".into(),
            kind: "document".into(),
            content: "This source link is invalid.".into(),
            include_in_context: false,
            source_chat_id: Some("chat-b".into()),
            source_message_id: Some("message-a".into()),
        });
        assert!(result.is_err());
    }

    #[test]
    fn library_relink_rejects_different_file_contents() {
        let db = memory_db();
        db.register_library_file(LibraryFileRecord {
            id: "file-1".into(),
            file_name: "original.txt".into(),
            extension: Some("txt".into()),
            source_path: "C:\\sync\\original.txt".into(),
            size_bytes: 4,
            sha256: "original-hash".into(),
            added_at: "now".into(),
            is_available: false,
        }).unwrap();

        let result = db.relink_library_file("file-1", LibraryFileRecord {
            id: "candidate".into(),
            file_name: "different.txt".into(),
            extension: Some("txt".into()),
            source_path: "C:\\sync\\different.txt".into(),
            size_bytes: 4,
            sha256: "different-hash".into(),
            added_at: "now".into(),
            is_available: true,
        });
        assert!(result.is_err());
    }

    #[test]
    fn task_validation_rejects_unknown_statuses() {
        let db = memory_db();
        let result = db.create_task(TaskInput {
            title: "Test".into(), details: String::new(), status: "finished-ish".into(),
            priority: "normal".into(), due_date: None,
        });
        assert!(result.is_err());
    }
}
