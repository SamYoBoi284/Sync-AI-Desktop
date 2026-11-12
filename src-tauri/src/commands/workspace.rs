use tauri::State;

use crate::storage::{
    workspace::{NewWorkspaceArtifact, NoteInput, TaskInput, WorkspaceArtifact, NoteRecord, TaskRecord, WorkspaceContextEntry},
    Database,
};

#[tauri::command]
pub fn list_workspace_artifacts(db: State<'_, Database>, query: Option<String>) -> Result<Vec<WorkspaceArtifact>, String> {
    db.list_workspace_artifacts(query)
}

#[tauri::command]
pub fn get_workspace_artifact(db: State<'_, Database>, artifact_id: String) -> Result<WorkspaceArtifact, String> {
    db.get_workspace_artifact(&artifact_id)
}

#[tauri::command]
pub fn create_workspace_artifact(db: State<'_, Database>, artifact: NewWorkspaceArtifact) -> Result<WorkspaceArtifact, String> {
    db.create_workspace_artifact(artifact)
}

#[tauri::command]
pub fn update_workspace_artifact(db: State<'_, Database>, artifact_id: String, artifact: NewWorkspaceArtifact) -> Result<WorkspaceArtifact, String> {
    db.update_workspace_artifact(&artifact_id, artifact)
}

#[tauri::command]
pub fn delete_workspace_artifact(db: State<'_, Database>, artifact_id: String) -> Result<(), String> {
    db.delete_workspace_artifact(&artifact_id)
}

#[tauri::command]
pub fn list_workspace_context(db: State<'_, Database>) -> Result<Vec<WorkspaceContextEntry>, String> {
    db.list_workspace_context()
}

#[tauri::command]
pub fn list_tasks(db: State<'_, Database>, query: Option<String>) -> Result<Vec<TaskRecord>, String> {
    db.list_tasks(query)
}

#[tauri::command]
pub fn create_task(db: State<'_, Database>, task: TaskInput) -> Result<TaskRecord, String> {
    db.create_task(task)
}

#[tauri::command]
pub fn update_task(db: State<'_, Database>, task_id: String, task: TaskInput) -> Result<TaskRecord, String> {
    db.update_task(&task_id, task)
}

#[tauri::command]
pub fn delete_task(db: State<'_, Database>, task_id: String) -> Result<(), String> {
    db.delete_task(&task_id)
}

#[tauri::command]
pub fn list_notes(db: State<'_, Database>, query: Option<String>) -> Result<Vec<NoteRecord>, String> {
    db.list_notes(query)
}

#[tauri::command]
pub fn create_note(db: State<'_, Database>, note: NoteInput) -> Result<NoteRecord, String> {
    db.create_note(note)
}

#[tauri::command]
pub fn update_note(db: State<'_, Database>, note_id: String, note: NoteInput) -> Result<NoteRecord, String> {
    db.update_note(&note_id, note)
}

#[tauri::command]
pub fn delete_note(db: State<'_, Database>, note_id: String) -> Result<(), String> {
    db.delete_note(&note_id)
}
