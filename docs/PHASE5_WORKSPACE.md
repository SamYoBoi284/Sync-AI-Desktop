# Phase 5 — Workspace and productivity contract

## Scope

Phase 5 adds persistent work and lightweight organization around the local chat engine:

- Workspace artifacts with create, search, edit and delete.
- Save an assistant response into Workspace as a document.
- Tasks with details, status, priority and optional due date.
- Notes with pinning and an explicit context inclusion setting.
- File Library registration for files the user chooses through a native file picker.
- Bounded, opt-in Workspace context for future local prompts.

This phase does not add AI filesystem tools, file-content retrieval, cloud sync, voice, or the Phase 6 approval engine.

## Data model and migration

SQLite migration v4 adds four tables:

| Table | Purpose |
|---|---|
| `workspace_artifacts` | Saved text/code/report artifacts, optional source chat/message references, context opt-in and timestamps |
| `tasks` | Task title/details, status, priority, optional ISO date, timestamps |
| `notes` | Note title/content, pinned flag, context opt-in and timestamps |
| `library_files` | Canonical source path, file name/extension, size, SHA-256 and registration date |

Foreign keys on artifact source references use `ON DELETE SET NULL`. Deleting a chat/message therefore does not delete a saved artifact. Task and note deletion is independent of chat history.

## Workspace behavior

Artifacts support the kinds `document`, `code`, `snippet`, `report` and `other`. The editor supports create, update, search and delete. Chat assistant messages expose a user-invoked “Save to Workspace” action; the user supplies the artifact title. Newly saved chat replies default to **not included** in future context.

Validation:
- Artifact title: 1–200 characters.
- Artifact body: maximum 2 MiB.
- The app stores the artifact body in SQLite.
- Workspace records are local and are not uploaded.

## Tasks and notes

Tasks support:
- Status: `todo`, `in-progress`, `done`.
- Priority: `low`, `normal`, `high`.
- Optional due date in `YYYY-MM-DD` form.
- Title maximum 200 characters; details maximum 128 KiB.

Notes support a title (maximum 200 characters), up to 2 MiB of content, pinning and a separate `include_in_context` flag. Pinning affects ordering; it does not automatically grant context access.

Tasks are not currently supplied to the language model. Notes are supplied only when their context flag is enabled.

## Prompt context boundary

Before each local generation, the chat path requests the context entries selected by the user. Only notes and Workspace artifacts with `include_in_context = 1` are eligible. The query returns at most 20 entries and the storage layer truncates the aggregate material to a maximum of 48 KiB.

The prompt labels these entries as user-selected reference material and tells the model not to treat their contents as overriding instructions. This is a prompt-level boundary, not a guarantee that every local model will perfectly distinguish instructions from quoted data. The selected context remains local to the prompt and is not sent to a cloud service by this feature.

## File Library boundary

The user must invoke “Add file” and select a file through the native file picker. Rust canonicalizes that selected path, verifies it is a regular non-empty file, enforces a 512 MiB maximum, and computes SHA-256 in bounded chunks.

The registry stores:
- Canonical original path.
- File name and extension.
- File size.
- SHA-256 fingerprint.
- Registration timestamp.

Re-importing identical bytes updates/relinks the matching record. Relinking a missing entry verifies the selected file and its hash. Removing an entry deletes only the SQLite registry record. It never moves, copies or deletes the user's original file.

**The File Library is currently a path-and-fingerprint index.** It does not parse or attach file contents to a chat, does not grant the model access to the path, and does not expose a generic filesystem API. Any future AI-requested file read or write must be implemented in Phase 6 behind an exact-scope, one-time approval and independent Rust validation.

## Backup behavior

The existing `sync-ai-desktop-backup` version-1 format remains a chat backup. It exports chats, messages and per-assistant-message diagnostics. Phase 5 tables are not yet included. A future backup-format revision must be versioned and preserve backward compatibility; do not silently alter version 1.

## Native command surface

The frontend uses typed wrappers for the following command groups:

- Workspace: `list_workspace_artifacts`, `get_workspace_artifact`, `create_workspace_artifact`, `update_workspace_artifact`, `delete_workspace_artifact`, `list_workspace_context`.
- Tasks: `list_tasks`, `create_task`, `update_task`, `delete_task`.
- Notes: `list_notes`, `create_note`, `update_note`, `delete_note`.
- File Library: `import_library_file`, `list_library_files`, `relink_library_file`, `remove_library_file`.

The webview does not submit arbitrary paths to File Library commands. File selection and canonicalization occur on the native side.

## Validation status and acceptance checklist

Source is committed, but Phase 5 has not been built or tested as a Windows application. The SQLite migration DDL and selected-context query received a separate Python SQLite smoke test only. This does not replace Rust or Tauri tests.

Required on the personal Windows development machine:
1. Run frontend build/typecheck and Cargo format/check/test.
2. Launch Tauri and verify migration v4 on both fresh and existing databases.
3. Create/edit/delete/search Workspace artifacts and confirm persistence after restart.
4. Save an assistant reply and confirm its source chat/message references remain valid.
5. Create/update/delete tasks and notes; validate status/priority/date constraints.
6. Confirm only opted-in notes/artifacts enter prompts; opt-out items never appear. Test the 20-entry and 48 KiB limits.
7. Add a file, relink after moving it, remove the registry entry, and confirm the original file remains untouched in every case.
8. Confirm File Library content is never inserted into a prompt.
9. Verify chat backup v1 remains unchanged and Phase 5 data is clearly excluded.

No native worker, frontend, full Tauri runtime or Windows integration result is claimed until these checks pass.
