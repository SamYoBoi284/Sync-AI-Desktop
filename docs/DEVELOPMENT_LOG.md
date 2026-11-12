# Development Log

Chronological record of meaningful implementation sessions. Keep entries factual and link commits or pull requests when available.

## 2026-11-12 — Phase 3 scope revision

- User approved per-assistant-message diagnostics as part of persisted chat messages.
- User requested chat backup export/import for future desktop versions; the desktop format will be versioned and preserve per-message diagnostics.
- Android backup v1 and raw chats.json compatibility are removed from desktop requirements.
- Phase 3 retains SQLite, chat/message CRUD, settings persistence, History search/rename/delete/reopen and transactional import/export.
- Added docs/PHASE3_DATA_CONTRACT.md as the implementation contract.
- This entry records scope only; implementation is tracked in the subsequent Phase 3 source entry.

## 2026-11-12 — Initial repository setup

- Confirmed the new public repository and its main default branch.
- Added README, baseline .gitignore, context tracker and initial architecture notes.
- Added placeholder directories for the planned application layers.
- No application code or dependencies were scaffolded.
- Validation: repository metadata and branch state checked; no build/tests existed yet.

## 2026-11-12 — Android source audit and desktop port specification

- Audited the Android reference branch feat/resource-manager-task-manager-17-10-2026.
- Inspected core chat layout and lifecycle, drawer navigation, theme system, appearance controls, Sync Core animation/state logic, preferences, chat persistence/backup, model registry, GGUF backend, JNI/native llama.cpp code, ToolEngine, ResourceManager and Workspace persistence.
- Added docs/ANDROID_TO_DESKTOP_SPEC.md with verified behaviors, source map, desktop component plan, Android backup compatibility, model/inference plan, deferred features, security approval architecture, Windows-specific concerns and staged acceptance checks.
- Updated docs/ARCHITECTURE.md, README.md and CONTEXT_TRACKER.md.
- User approved React + TypeScript. Vite selected for the frontend scaffold.
- No application code, dependencies, native build or tests were added/run.

## 2026-11-12 — Phase 1 Tauri + React foundation

- Added a React + TypeScript + Vite frontend scaffold based on the maintained Tauri React TypeScript starter structure.
- Added Tauri 2 Rust entry points and one read-only get_app_info command.
- Added a responsive foundation UI and typed frontend call to the Rust command.
- Configured Vite's Tauri environment handling and platform-specific build target using the official Tauri Vite guidance.
- Added a restrictive CSP and a main-window capability limited to core:default. No shell, filesystem, process or opener plugin was added.
- Kept navigation placeholders visibly marked planned and did not claim chat, model, storage or tool functionality.
- Disabled packaging until app icon and installer configuration are ready.
- Source files were committed and fetched back from GitHub for verification.
- Validation attempt: Node.js v22.16.0 and npm 10.9.2 are available in the execution container. Rust/Cargo are absent, and network DNS to GitHub/npm is unavailable, so dependencies could not be installed and no build or Tauri launch could be run. Vite 8 requires Node 20.19+ or 22.12+, so the available Node version meets that requirement.


## 2026-11-12 — Phase 2 startup lifecycle foundation

- Added `tauri-plugin-single-instance` as the first registered Tauri plugin. Subsequent launches restore, unminimize and focus the existing main window.
- Added a Rust atomic startup gate consumed by the first webview bootstrap call, preventing splash replay on tray restore or same-process page reload.
- Added a Sync-branded loading screen that waits for the native metadata handshake and a short minimum display interval; it does not wait for model initialization.
- Added a Rust-created native tray icon with Open, Hide and Exit actions. Left-click restores where supported; closing the main window hides it to tray.
- Updated architecture, project tracker and README with source-versus-runtime status.
- Validation: GitHub source and branch were inspected; no npm install, frontend build, Cargo check, Tauri launch or Windows interaction test was run. The code is committed but runtime-unverified.


## 2026-11-12 — Phase 2 desktop shell and navigation

- Replaced the static foundation/status page with a persistent navigation sidebar and page-aware top bar.
- Added functional navigation destinations for Chat, History, Workspace, Tasks & Notes, File Library, Local models, Appearance and Runtime.
- Built a dedicated chat landing screen with Sync Core-inspired visual treatment, model status and a disabled composer. No model response is fabricated.
- Added feature staging pages that describe intended responsibilities without claiming their underlying services are implemented.
- Added a runtime page that displays only metadata available from the existing Rust bridge.
- Added responsive layouts, keyboard focus treatment and reduced-motion styling.
- Updated README, architecture and the context tracker.
- Validation: source files and branch state were reviewed before commit. No TypeScript build, Rust build, Tauri launch or Windows runtime test was run.


## 2026-11-12 — Phase 3 initial storage and History implementation

- Added SQLite with bundled SQLite, a version-1 schema migration, chats/messages/app_settings tables, foreign keys and indexes.
- Added Rust storage models and typed Tauri commands for chat listing/search, create/load/rename/delete, message append, per-assistant-message diagnostics, settings and backup import/export.
- Added desktop-native sync-ai-desktop-backup format version 1. Import validates bounds, IDs, roles, timestamps and diagnostics, skips existing chat IDs and runs in a transaction. Export includes per-message diagnostics.
- Added Rust unit tests for migration, diagnostics persistence, backup round-trip, duplicate-chat skipping, invalid diagnostics roles and duplicate IDs in a backup.
- Added a History UI with search, create, open, rename, delete, diagnostics disclosure, backup import and backup export.
- Marked History as an active navigation destination. Android backup compatibility remains explicitly out of scope.
- Validation: source committed and key files fetched back from GitHub. The SQLite DDL, core list query, diagnostics constraint and cascade behavior were smoke-tested separately with Python sqlite3. The execution environment has Node.js 22.16.0/npm 10.9.2 but no Rust/Cargo; dependencies cannot be installed here. No TypeScript build, Cargo test/check or Windows runtime test is claimed.


## 2026-11-12 — Phase 2.2 sidebar redesign

- Replaced the History navigation row with a recent-conversations area, including search and date grouping (Today, Yesterday, Previous 7 days, Previous 30 days and Older).
- Recent rows use the SQLite-backed listChats command and route selection into History with the selected conversation ID.
- Added a Settings entry to the sidebar footer and a Settings destination using the existing staged feature-preview treatment.
- Kept workspace and system destinations in the main navigation and preserved the local-first footer.
- No separate frontend-only history cache or duplicate persistence layer was added.
- Validation: source reviewed and committed. No npm build, Cargo check, Tauri launch or Windows UI test was run.


## 2026-11-12 — Phase 4 Stage 1: model management and resource budget

- Added schema migration v2 and a GGUF model registry with SHA-256 uniqueness and a single default model.
- Added Rust commands for native file selection/import, managed model copy, streaming SHA-256, duplicate detection, listing, default selection and managed model removal.
- Added host installed/available RAM detection.
- Added the Local Models management page and optional RAM Limiter controls in Settings. The limiter is disabled by default and its selected budget persists through SQLite.
- Kept Stage 1 honest about its boundary: the RAM budget is not yet enforced during model loading. Memory estimation and pre-load enforcement belong to Stage 2.
- No arbitrary filesystem path is accepted from the webview for model import; Rust opens the native picker itself.
- Validation: source committed only. No npm build, Cargo check/test or Windows runtime test run.


## 2026-11-12 — Phase 4 Stage 2: GGUF metadata and native model lifecycle

- Added a bounded GGUF v2/v3 metadata parser for architecture, model name, general file type/quantization and dimensions used in memory estimates.
- Changed model import to validate the GGUF header and persist metadata in the existing SQLite v2 model columns; existing imports are metadata-refreshed when the library is opened.
- Added pinned llama_cpp = 0.3.2 Rust bindings, which build bundled llama.cpp and statically link by default.
- Added a CPU-only runtime manager and typed commands for runtime status, model load, model unload and resource estimation. The model is loaded only by explicit user action and is never auto-loaded on startup.
- Added 2,048-token context/128-token batch session setup, conservative metadata-based memory estimation, currently available RAM reserve checks, and a second pre-session check using llama.cpp's native session-size estimate.
- Blocked model deletion while that model is loading or active and canonicalized model paths inside the managed model directory.
- Updated Local Models and Settings with metadata, memory inspection, Load/Unload and RAM enforcement behavior.
- Validation: source changes committed for review only. No npm build, Cargo check/test, C++ compile/link, actual GGUF load or Windows runtime test was run. The C++/libclang toolchain is required on the Windows development machine.
- Stage 3 chat generation/streaming is not started.

## 2026-11-12 — Phase 4 corrections: external model paths and isolated worker

- Added SQLite migration v3 for nullable source-path metadata while preserving the legacy managed-path column for old library rows.
- Reworked GGUF import to register a canonical original path, calculate SHA-256 without copying the source file, and relink duplicate hashes when the same model is selected from another location.
- Reworked the model library to detect missing files, display the stored path and remove only the Sync registry record. The original file is never deleted.
- Moved llama.cpp model/session ownership into a dedicated child process launched in worker mode.
- Added Windows Job Object creation, optional per-process committed-memory limit, and kill-on-job-close lifecycle. The main Tauri/WebView application is not placed in the worker Job Object.
- Disabled model mmap in the worker so GGUF weights are allocated as worker-private memory subject to the configured commit ceiling.
- Added worker telemetry to Runtime: configured cap, PID, working set, private committed memory, and last worker error.
- Replaced the earlier generation-only RAM budget copy and stale model-load estimate gate with the hard worker-ceiling behavior. The pre-load model estimate is now informational only.
- Updated settings to save the ceiling explicitly and clarified that a smaller limit may prevent a model from loading.
- Validation: source changes authored and reviewed through GitHub API only. No npm build, Cargo check/test, Windows Job Object test, native compilation or actual GGUF load was run. Personal Windows device validation remains required.
- Phase 4 remains exactly three stages. Stage 3 chat generation/streaming is still pending.

## 2026-11-11 — Phase 4 source implementation

- Changed new GGUF imports to original-path registration with SHA-256 deduplication/relinking and SQLite v3 compatibility for legacy records.
- Added missing-file detection, Rust validation before load/default, non-destructive removal and unload-before-replacement.
- Moved llama.cpp into a dedicated worker. Windows Job Objects enforce the optional worker private-commit ceiling outside the main UI process. Runtime telemetry displays limit, PID, working set, private commit and errors.
- Connected Stage 3 chat persistence, role-labelled context, token streaming, cancellation, sequential queueing and per-assistant-message diagnostics. Added positional insertion so each answer stays after its triggering user turn.
- Model-specific templates remain deferred. No auto-load, sign-in startup, multimodal feature set or DAM category added.
- No frontend/Cargo/Windows/native build or real-model test has run.


## 2026-11-12 — Phase 5 Workspace and productivity source implementation

- Added SQLite schema migration v4 for Workspace artifacts, tasks, notes and File Library registry records.
- Added typed Rust storage models and native commands for Workspace artifact CRUD/search, context selection, Tasks & Notes CRUD, and File Library import/list/relink/remove.
- Workspace artifacts and notes are included in future local chat prompts only when the user explicitly enables their context checkbox. The combined context query is limited to 20 entries and 48 KiB.
- Added a “Save to Workspace” action to assistant replies. Newly saved replies default to excluded from chat context.
- Added Workspace editor/search, Tasks & Notes tabs and controls, and File Library UI.
- File Library imports use a native picker and register canonical original paths plus file size and SHA-256. The application does not copy, move or delete the original. File contents are not loaded into chat.
- Activated Workspace, Tasks & Notes and File Library navigation routes. Added responsive styles.
- Updated the README and Context Tracker, and added Phase 5 data/security documentation.
- Validation boundary: source consistency was reviewed through the GitHub API. No npm build, TypeScript typecheck, Cargo fmt/check/test, native compilation, Tauri launch or Windows runtime test was run. The Phase 5 source must be validated on the personal Windows development machine.


### Phase 5 follow-up — validation and safety refinements

- Added summary-only Workspace and Notes list queries; full content is fetched only when a user opens an artifact or note. Added a native `get_note` command.
- Added full task editing for title, details, status, priority and due date. Task details are capped at 32 KiB and task list results at 200.
- Added Rust validation that saved assistant-message provenance uses a source message belonging to the supplied source conversation.
- File Library hashing now uses one open file handle, verifies size and modification time after hashing, and rejects a file that changes while its fingerprint is being calculated. Non-UTF-8 paths are rejected rather than stored lossily.
- File Library relinking now requires an exact match with the original SHA-256 fingerprint. A changed file must be registered as a new library entry.
- Added Rust unit-test source for lightweight summary/detail behavior, mismatched chat/message provenance and mismatched File Library relinking.
- Re-ran a separate Python SQLite smoke test against the Phase 5 migration shape. It passed checks for tables/indexes, opt-in context query, 48 KiB clipping, summary-only query, FK nulling, task status/priority constraints and unique library path/hash constraints.
- This SQL smoke test does not compile or test Rust. Frontend build, Cargo fmt/check/test, native picker, Tauri launch, worker and Windows runtime remain unverified.
