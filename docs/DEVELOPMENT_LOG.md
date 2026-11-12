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
