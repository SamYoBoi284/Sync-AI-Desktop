# Phase 3 — Local Data Contract

Status: approved contract; initial Rust storage and History UI source committed; build/runtime validation pending.

Sync AI Desktop owns its data model and backup format. Android backup compatibility is not required.

## Goals

- Persist chats and messages in local SQLite.
- Attach optional diagnostics to each individual assistant message.
- Implement History list, search, open, rename and delete.
- Export/import chat backups using a versioned desktop-native format.
- Preserve diagnostics across restarts and backup/restore.
- Make imports atomic and protect local conversations from overwrite.

## Out of scope

- Android sync-ai-chat-backup v1 and raw chats.json compatibility.
- Model weights, model registry, Workspace artifacts, File Library and Tasks & Notes storage.
- Cloud synchronization or backup.
- Exporting secrets, unrelated settings or model files.

## SQLite schema

Enable foreign keys on every connection. Use monotonically increasing schema migrations, applied transactionally.

### schema_migrations
- version INTEGER PRIMARY KEY
- applied_at TEXT NOT NULL (UTC RFC 3339)

### chats
- id TEXT PRIMARY KEY
- title TEXT NOT NULL
- created_at TEXT NOT NULL (UTC RFC 3339)
- updated_at TEXT NOT NULL (UTC RFC 3339)

### messages
- id TEXT PRIMARY KEY
- chat_id TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE
- role TEXT NOT NULL (system, user or assistant)
- body TEXT NOT NULL
- created_at TEXT NOT NULL (UTC RFC 3339)
- position INTEGER NOT NULL CHECK position >= 0
- diagnostics_json TEXT NULL
- UNIQUE(chat_id, position)

Diagnostics are nullable and valid only for assistant messages. Rust validates this in addition to database constraints. Diagnostics are structured metadata, not a second copy of the prompt or response.

### app_settings
- key TEXT PRIMARY KEY
- value_json TEXT NOT NULL
- updated_at TEXT NOT NULL

Model metadata and Workspace tables belong to later phases. Keep databases, backups, model files and user data out of Git.

## Per-message diagnostics

Attach diagnostics to the assistant message produced by one generation. Version 1 should allow nullable fields:

- generation_id
- model_id and model_label at generation time
- backend/runtime identifier and version when available
- status: completed, stopped or failed
- prompt_tokens and generated_tokens
- prompt_eval_ms and generation_ms
- tokens_per_second, derived only from valid measured values
- context_size and max_context_size when reported
- temperature when known
- stop_reason and safe error code/message
- started_at and finished_at

Never fabricate unavailable metrics. Null means unavailable, not zero. Do not persist private prompt text, secrets, file contents or arbitrary native logs. Diagnostics belong to the individual assistant message, not only to the chat.

## Desktop backup format

Use a JSON document with this top-level structure:

    {
      "format": "sync-ai-desktop-backup",
      "formatVersion": 1,
      "exportedAt": "2026-11-12T12:00:00Z",
      "app": { "name": "Sync AI Desktop", "version": "0.1.0" },
      "chats": [
        {
          "id": "chat-id",
          "title": "Conversation title",
          "createdAt": "2026-11-12T11:00:00Z",
          "updatedAt": "2026-11-12T11:30:00Z",
          "messages": [
            {
              "id": "message-id",
              "role": "assistant",
              "body": "Message body",
              "createdAt": "2026-11-12T11:30:00Z",
              "position": 0,
              "diagnostics": {
                "generationId": "generation-id",
                "status": "completed",
                "promptTokens": 120,
                "generatedTokens": 80
              }
            }
          ]
        }
      ]
    }

The example is illustrative. Optional diagnostics fields may be omitted or null. The serializer must use stable naming and tests.

### Export behavior
- Support exporting all chats and, where practical, one selected chat.
- Include chat metadata, ordered messages and each assistant message's diagnostics.
- Export a consistent database snapshot.
- Use a user-initiated save destination; never silently write arbitrary paths.
- Exclude model files, credentials, secrets and unrelated settings.
- Avoid logging message bodies on export failure.

### Import behavior
- User explicitly selects a backup file.
- Validate format identifier and supported version first.
- Enforce file-size, chat-count, message-count, string-length and diagnostics-size limits.
- Validate every chat, message, role, timestamp, ordering value and diagnostics object before writing.
- Reject duplicate IDs inside the backup and malformed references.
- Skip chat IDs that already exist locally; never overwrite local chats.
- Import non-conflicting chats in one transaction. Roll back the entire import if it fails.
- Return imported/skipped counts and a clear failure reason.
- Reject unsupported future versions; never guess their meaning.
- Never modify or delete the source backup.

## Rust command boundary

The frontend never accesses SQLite directly. Expose narrow typed commands for bounded chat search/listing, create/rename/delete, loading one chat with ordered messages and diagnostics, appending/updating messages, and user-initiated backup import/export.

Rust owns validation, transactions, migrations, serialization and file I/O. Do not expose generic SQL commands. AI-invoked host file operations remain subject to the separate one-time approval policy. A user-selected backup action is a direct user action, not model authority.

## UI requirements

- History sorts by most recently updated and searches titles and message text.
- Opening a chat loads messages in stable order.
- Rename and delete have clear actions; deletion is confirmed.
- Each assistant message with diagnostics has its own diagnostics affordance.
- Messages without diagnostics do not show an empty panel.
- Unavailable metrics are labelled unavailable, never shown as zero.
- Backup import/export is accessible from History and preserves diagnostics.

## Acceptance tests

- Migrations create expected tables, indexes, constraints and foreign keys.
- Reopening the app preserves chats, messages and diagnostics.
- Ordering stays stable after insert, update, delete and reload.
- Deleting a chat cascades to its messages.
- Diagnostics cannot be attached to user/system messages.
- Backup export round-trips chats, messages and diagnostics.
- Import skips existing chat IDs without overwriting them.
- Duplicate IDs, malformed records, oversized input and unsupported versions are rejected before partial writes.
- Failed import leaves the database unchanged.
- Export snapshot is consistent while the app is active.
- Tests use temporary databases; no user database or backup is committed.

## Implementation order

1. Add SQLite dependency and migration infrastructure.
2. Implement storage repositories and typed Rust command DTOs.
3. Add Rust tests for migrations, CRUD, diagnostics, ordering and backup validation/round trips.
4. Connect frontend bridge and History UI.
5. Add per-message diagnostics disclosure.
6. Add explicit user-selected backup import/export UI.
7. Run frontend build and Rust tests; validate Tauri integration on the personal Windows machine.

## Compatibility policy

The desktop backup format is independently versioned. Future versions must include migration readers or a documented non-destructive upgrade path. Do not reuse Android format identifiers or claim Android interoperability.
