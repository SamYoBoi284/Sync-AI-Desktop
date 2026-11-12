# Phase 4 — local model management and isolated runtime

This revision supersedes the earlier Stage 1/Stage 2 notes that described a managed model-copy directory or an application-level inference allocation budget.

## Model library behavior

- Import uses the Windows native file picker and accepts GGUF files.
- Sync stores the selected file's canonical absolute path in SQLite. It does not copy or move new model files.
- SHA-256 identifies duplicate content. Re-importing the same bytes from another location updates the registered source path so a missing entry can be relinked.
- The library displays the original path and detects missing or moved files. An unavailable entry cannot be loaded or made the default.
- The default model and currently loaded model are separate states.
- Remove from Sync deletes only the registry entry. It never deletes or moves the model file.
- Existing records from the earlier managed-copy implementation remain readable through a migration compatibility path. Removing one of those records also leaves its old managed file untouched.
- Startup model loading and Windows sign-in auto-start are not part of this phase.

## RAM Limiter: hard worker-process ceiling

The optional RAM Limiter controls a dedicated inference worker process, not the Tauri/WebView application process.

On Windows, Sync creates a Job Object for the child worker and configures JOB_OBJECT_LIMIT_PROCESS_MEMORY before sending it a model-load command. The main Sync AI desktop process is not assigned to this job, so a worker allocation failure or worker termination does not deliberately consume the interface process's memory budget.

The limit is a private committed-memory ceiling, not a resident working-set ceiling. The worker loads GGUF weights without file-backed memory mapping so model allocations are included in private committed memory. Runtime diagnostics report the configured ceiling, worker working set (resident memory), worker private committed memory, PID, and the last worker error when available.

The ceiling includes the model and inference session. A model whose load requires more memory than the configured ceiling may fail to load. A later allocation can also fail or terminate the worker. Sync must report the failure and remain open. The limit is disabled by default. A saved change applies on the next model load; it does not resize an already-running worker.

The model-size/session estimate shown in the library is informational. It is not the enforcement mechanism and must not be described as a guaranteed physical-RAM forecast. The Windows Job Object enforces the private commit ceiling.

## Exactly three stages

1. **Model management + RAM limiter:** original-path GGUF registry, native file picker, SHA-256 identity, missing-file detection, non-destructive removal, default/active separation, host RAM display and configurable worker limit.
2. **Native runtime + model lifecycle:** CPU llama.cpp running in a dedicated worker, Windows Job Object memory ceiling, explicit manual load/unload, lifecycle serialization and Runtime diagnostics.
3. **Chat inference connection:** conversation context, token streaming, cancellation, sequential queued sends, and diagnostics attached to the exact assistant message.

No numbered micro-stages are added. Phase 5 remains Workspace, File Library, Tasks & Notes and broader artifact workflows. Phase 6 remains the secure tool/Device Assist layer. Phase 7 remains voice, optimization, packaging, testing, security review and release.

## Current source status

Stages 1 and 2 have been revised in source. Stage 3 chat generation and streaming are still pending. No Windows build, Cargo test/check, frontend build or real GGUF load has been run for this revision; it must be verified on the personal Windows development machine before treating the behavior as validated.
