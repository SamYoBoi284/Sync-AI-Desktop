# Sync AI Desktop — Context Tracker

> **Purpose:** Durable handoff record for future development sessions. Update this file in the same commit as every meaningful implementation or architecture change. Keep verified facts separate from proposals, and never claim work is tested unless it was actually run.

## 1. Project identity

- **Project:** Sync AI Desktop
- **Repository:** https://github.com/SamYoBoi284/Sync-AI-Desktop
- **Android companion/reference:** https://github.com/SamYoBoi284/Sync-AI
- **Default branch:** main
- **Initial target:** Windows x64
- **Product direction:** Offline-first desktop AI assistant with local GGUF inference and tightly permissioned host-computer tools.
- **Repository separation:** Desktop work must not modify the Android repository.

## 2. Current status

**Phase:** 3 — local data foundation (source implementation in progress; validation pending)  
**Status:** SQLite schema/migration v1, Rust chat/message/settings commands, per-assistant-message diagnostics persistence, versioned desktop backup validation/import/export, and a connected History UI are committed to main. Android backup compatibility is out of scope. No frontend build, Cargo test/check or Windows runtime test has been run.

### Verified in this repository

- Public repository exists; default branch is main.
- Detailed Android-to-desktop port specification is committed at docs/ANDROID_TO_DESKTOP_SPEC.md.
- React + TypeScript + Vite frontend files are committed.
- Tauri 2 Rust source/configuration files are committed.
- Rust registers get_app_info and consume_startup_splash as custom commands.
- The main-window capability contains core:default only.
- No shell, filesystem, process or opener plugin is added.
- The frontend distinguishes the native bridge from browser preview.
- Sidebar navigation is functional; History now has a source-implemented local chat interface, while other unimplemented feature destinations remain clearly labelled previews.
- Rust registers the Tauri single-instance plugin first and restores/focuses the existing main window on subsequent process launches.
- Rust owns an atomic startup-splash gate consumed once per process.
- A native tray menu provides Open, Hide and Exit; left-click restores where supported.
- Closing the main window hides it instead of terminating the process.
- Key scaffold files and updated branch head were fetched back from GitHub.

### Not yet implemented or verified

- npm dependency installation and package lockfile
- Frontend typecheck/production build
- Rust compilation of the single-instance and tray lifecycle implementation
- Windows runtime verification of first-start splash, same-process reload behavior, tray restore, close-to-tray, second-launch focus and Exit
- Tauri development launch on Windows
- Cargo check/build on Windows
- Local SQLite runtime/build validation and end-to-end Tauri IPC tests
- Chat generation, streaming, queueing and cancellation
- End-to-end backup file dialog/download testing on Windows
- GGUF model manager
- llama.cpp Windows build/integration
- Workspace, File Library, Tasks & Notes or voice
- Permission/approval UI or enforcement
- Windows tools/system adapters
- Frontend automated tests, CI, installer, app icons or packaging (Rust unit test source exists but has not been run)

### Validation environment check

- Available in the assistant execution container: Node.js v22.16.0 and npm 10.9.2.
- Vite 8's current documentation requires Node.js 20.19+ or 22.12+; the available Node 22.16.0 satisfies that requirement.
- Rust and Cargo are not installed in that container.
- Outbound DNS/network access to GitHub was unavailable, so project cloning and npm dependency installation could not be completed.
- Therefore no frontend build, Cargo check, Tauri launch or Windows runtime result is claimed. These checks must be run in the actual Windows development environment after dependency installation.

## 3. Product requirements

1. Windows first; cross-platform support can be considered later.
2. Preserve the Android repository as a separate project.
3. Adapt Android visual design and workflows for desktop; do not attempt a direct Java UI port.
4. Offline-first core chat and local inference.
5. Use llama.cpp for GGUF inference behind a narrow native boundary.
6. Never give the model unrestricted shell, terminal, filesystem, process or operating-system access.
7. Every AI-invoked host tool operation requires fresh, explicit, one-time user approval, including reads.
8. Approval covers only the exact action and scope shown, expires after one operation and cannot become persistent.
9. The Rust backend validates every operation independently. Model output is an untrusted proposal.
10. Keep model weights, secrets, local databases and user data out of Git.

## 4. Technology decisions

| Area | Direction | Status |
|---|---|---|
| Desktop shell | Tauri 2 | Agreed |
| Single instance | tauri-plugin-single-instance | Source added; runtime unverified |
| System tray | Tauri tray-icon API | Source added; runtime unverified |
| Backend | Rust | Agreed |
| Frontend | React + TypeScript | **Approved by user on 2026-11-12** |
| Frontend build tool | Vite | Selected |
| Package manager | npm | Selected |
| Inference | llama.cpp + GGUF | Agreed direction |
| Persistence | SQLite | Schema v1 and storage source committed; build/runtime validation pending |
| Config | Local structured settings | Implementation choice open |
| Platform | Windows x64 first | Agreed |

The Tauri-maintained create-tauri-app project includes a React TypeScript template. Tauri's frontend documentation recommends Vite for SPA frameworks. The starter uses React 19, TypeScript 6 and Vite 8 dependency ranges from the maintained React-TS template. Dependency installation is not yet verified.

Windows dev prerequisites to check on the actual build machine: Rust stable MSVC (at least 1.90 for the current single-instance plugin release), Microsoft C++ Build Tools (Desktop development with C++), WebView2, and a Node.js version compatible with the selected Vite release. WebView2 is included on many Windows 10 version 1803+ installations, but the local machine still needs verification.

## 5. Android source audit baseline

Repository: https://github.com/SamYoBoi284/Sync-AI  
Branch: feat/resource-manager-task-manager-17-10-2026  
Initial audit branch commit previously recorded: 70e6f22b3514fa695008d524d1631c25a9a040c1

### Core files inspected in detail during the 2026-11-12 pass

| Android source | Verified responsibility |
|---|---|
| MainActivity.java | Chat layout, send/queue/stream/stop lifecycle, model status, chat navigation and tool routing |
| SideDashboard.java | Main navigation and settings routes |
| AppTheme.java | Six palettes, light/dark variants, contrast helpers |
| AppearanceActivity.java | Theme/mode/accent/motion selection and preview |
| SyncCoreView.java | Eight visual states, orbit/pulse animation and reduced motion |
| AppPreferences.java | Appearance, memory, active chat, voice, personality and update preferences |
| ChatStore.java | Chat CRUD, title generation, persistence and JSON backup import/export |
| ChatRecord.java / ChatMessage.java | Chat and message fields |
| ModelManager.java / ModelInfo.java | Model import/hash/registry/selection/removal |
| GgufModelBackend.java | Serial model lifecycle, generation, cancellation and diagnostics |
| GgufNative.java | JNI boundary and generation callback mapping |
| syncai_native.cpp / CMakeLists.txt | llama.cpp implementation, Android-specific JNI/CMake and pinned source revision |
| SyncRuntime.java | Shared runtime composition |
| ToolEngine.java | Deterministic intent routing and Android-specific side effects |
| ResourceManagerActivity.java | Process CPU/PSS/storage/component activity monitoring |
| WorkspaceActivity.java / WorkspaceStore.java | Workspace UI and artifact persistence/extraction |

The wider source tree includes File Library, tasks/notes, recorder, camera, voice and diagnostics modules. The current port document identifies these, but does not claim every secondary screen was fully audited line by line.

### Verified behavior notes

- Android UI is built programmatically in Java, not XML or Jetpack Compose.
- Main chat persists user messages before inference; queues later messages while generation is active; streams tokens; supports cancellation, completion and errors.
- Send becomes Stop only while generation is active and the composer is empty.
- Android reference behavior: its ChatStore uses sync-ai-chat-backup v1 and accepts raw chats.json arrays. Desktop intentionally uses a separate format.
- ModelManager copies model imports, computes SHA-256, deduplicates by hash, and stores model metadata.
- GgufModelBackend actually loads GGUF only; other recognized extensions are rejected for inference.
- Android native llama.cpp is pinned at revision 4a89937 and uses JNI/Android-specific code. Do not copy its JNI/CMake integration to Windows.
- Sync Core states: IDLE, READY, LISTENING, THINKING, SPEAKING, EXECUTING, SUCCESS, ERROR. Orbit rotation is limited to THINKING; reduced motion stops continuous animation.
- Appearance has six identities, eight accents, three theme modes and three motion intensities.
- Android ToolEngine can execute deterministic actions before the model and can request Android runtime permissions afterward. Desktop must not copy this permission sequence; every model-invoked host action requires fresh approval before execution.
- Android resource metrics carefully label process-wide CPU and PSS rather than inventing per-library memory splits. Preserve that measurement honesty.

## 6. Desktop port specification

Authoritative source map and implementation notes: docs/ANDROID_TO_DESKTOP_SPEC.md.

Key desktop decisions:
- Persistent left navigation; adapt Android slide drawer to desktop shell.
- Main chat: Sync Core/header, model status, message list, diagnostics, multiline composer, send/stop state.
- Preserve conversation queue, streaming and cancellation; use a desktop-native, versioned chat backup format. Android backup compatibility is not required.
- Preserve six themes, light/dark/system mode, eight independent accents and motion settings.
- GGUF-only loadable model manager for MVP; import/hash/deduplicate/metadata lifecycle.
- Build llama.cpp for Windows behind an opaque-handle C ABI or safe Rust FFI. Exact mechanism requires a proof-of-concept spike.
- Defer Android-specific camera, calls, contacts, intents, wake-word services, SpeechRecognizer/TTS and media intents until desktop-specific designs exist.

## 7. Phase 1 scaffold inventory

| Path | Purpose |
|---|---|
| package.json | React, TypeScript, Vite, Tauri CLI/API dependencies and scripts |
| index.html | WebView entry point |
| vite.config.ts | React plugin, fixed dev port 1420, Tauri environment variables, platform build target and file-watch exclusion |
| tsconfig.json / tsconfig.node.json | Strict frontend and Vite configuration TypeScript |
| src/main.tsx | React root |
| src/App.tsx | App shell, active-page state, shared top bar and runtime bridge status |
| src/components/navigation/Sidebar.tsx | Functional sidebar navigation and navigation metadata |
| src/components/pages/ChatPage.tsx | Chat landing screen and inactive composer shell |
| src/components/pages/FeaturePage.tsx | Explicitly staged history/workspace/tasks/files/models/appearance destinations |
| src/components/pages/RuntimePage.tsx | Read-only display of currently available native metadata |
| src/styles.css | Responsive baseline and reduced-motion handling |
| src/services/nativeBridge.ts | Typed call to the Rust metadata command |
| src-tauri/Cargo.toml | Rust/Tauri dependencies |
| src-tauri/build.rs | Tauri build integration |
| src-tauri/src/main.rs | Windows desktop entry point |
| src-tauri/src/lib.rs | Tauri builder and command registration |
| src-tauri/src/commands/app_info.rs | Harmless static app/OS/architecture information |
| src-tauri/tauri.conf.json | Main window, Vite commands, restrictive CSP, packaging disabled |
| src-tauri/capabilities/main.json | Minimal main-window capability (core:default only) |

The sidebar and page navigation are functional frontend behavior. The chat destination is a visual shell only: its composer is disabled until model integration exists. History, Workspace, Tasks & Notes, File Library, Local Models and Appearance currently explain their planned responsibilities; their backing services are not connected.

Tauri packaging is disabled pending an app icon and installer configuration. No external font/CDN is used; the UI stylesheet is local. No privileged plugin is installed.

## 8. Architecture

Frontend: React + TypeScript + Vite.  
Desktop shell: Tauri 2.  
Backend: Rust typed commands, services, validation, storage, model lifecycle and tool broker.  
Native inference: llama.cpp built for Windows behind a narrow FFI.  
Storage: SQLite planned for conversations, messages, preferences, model metadata and Workspace artifacts.  
Local models: user-managed GGUF files, excluded from Git.  
Security boundary: explicit one-time approval UI plus backend validation and narrow Windows adapters.

Tauri capabilities are an additional boundary, not a replacement for per-operation approval or safe Rust implementation.

## 9. Security model — non-negotiable

- No generic execute_shell, arbitrary command execution, terminal or script runner exposed to the model.
- No unrestricted filesystem API or model-controlled path expansion.
- Each proposal identifies a registered operation, target, parameters and expected effect.
- Show exact operation and scope to the user before execution.
- Require fresh approval for every operation, including read-only operations.
- Bind approval to the exact request; single use; expire on use/cancellation/timeout.
- Revalidate canonical paths, arguments and target state in Rust after approval.
- Windows path validation must account for traversal, case-insensitivity, junctions, symlinks/reparse points, alternate data streams and TOCTOU risks.
- Reject malformed, stale, replayed or modified approvals.
- Denial and cancellation are safe defaults.
- Log decisions without exposing private file contents, prompts or secrets.
- No remember-my-choice or blanket permissions unless the owner explicitly revises the requirement.

## 10. Development sequence

1. Phase 0 — Repository foundation and source-to-desktop spec. Complete.
2. Phase 1 — Tauri 2 + React + TypeScript + Vite scaffold and minimal metadata IPC command. Source committed; builds not yet verified.
3. Phase 2 — Desktop shell, startup/tray lifecycle, sidebar, chat UI, Sync Core, themes and settings. Startup/tray lifecycle and navigable shell source are implemented; chat services and Windows validation are pending.
4. Phase 3 — SQLite schema/migrations, chat/message CRUD, per-assistant-message diagnostics, History CRUD/search, and versioned desktop-native chat backup import/export.
5. Phase 4 — GGUF manager and minimal Windows llama.cpp FFI proof of concept.
6. Phase 5 — Streamed local inference, Workspace, File Library, Tasks & Notes and resource metrics.
7. Phase 6 — Tool registry, one-time approval broker, Windows validation and security test suite.
8. Phase 7 — Voice/recording, packaging, clean-machine test and security review.

Do not implement computer-control tools before the approval model and its tests exist.

## 11. Working conventions

- Inspect before editing.
- Prefer small, reviewable changes.
- Do not overwrite user work or unrelated files.
- Keep Android and desktop repositories isolated.
- Update CONTEXT_TRACKER.md in the same commit as every meaningful implementation/architecture change.
- Record files changed, behavior added, tests actually run, results, limitations and the next exact step.
- Distinguish planned, implemented, tested and verified.
- Never invent test results, commit hashes, file contents or repository state.
- Keep README, architecture and development log synchronized with actual project state.

## 12. Change log

### 2026-11-12 — Repository initialization
- Created README, .gitignore, context tracker, architecture and development log.
- Established desktop repository separation and directory placeholders.
- No application runtime or feature code added.

### 2026-11-12 — Android-to-desktop audit and specification
- Inspected core Android chat, navigation, themes, preferences, chat store, model manager, inference boundary/native implementation, tool engine, resource manager and Workspace store.
- Created docs/ANDROID_TO_DESKTOP_SPEC.md with verified behavior, component plan, Windows-specific mapping, compatibility requirements, security design and implementation phases.
- Updated architecture notes for the approved frontend and one-time approval broker.
- User explicitly approved React + TypeScript; Vite selected.
- Tests/builds: not run.

### 2026-11-12 — Phase 1 application scaffold
- Added the Tauri 2 + React + TypeScript + Vite project structure.
- Added a minimal Rust get_app_info command and typed frontend bridge.
- Added a static foundation UI; planned navigation remains nonfunctional and labelled planned.
- Added a restrictive CSP and a main-window capability with core:default only. No shell/filesystem/process/opener plugin.
- Disabled packaging until application icon and installer configuration are ready.
- Source files and GitHub branch state verified.
- Validation attempt: the execution container reports Node v22.16.0/npm 10.9.2; no Rust/Cargo toolchain. DNS/network access to GitHub was unavailable, preventing clone and npm dependency installation. Vite 8's official Node requirement (20.19+ or 22.12+) is satisfied by this Node version, but no frontend build or Windows Tauri launch was run.

## 13. Exact next step

Continue Phase 3 validation and hardening:
1. Run npm install and npm run build on the personal Windows development machine.
2. Run cargo test and cargo check from src-tauri.
3. Launch with npm run tauri dev and verify database creation, History CRUD/search, diagnostics display, backup export/import and duplicate handling.
4. Fix compile/runtime issues found by those checks before moving to Phase 4.
3. Add theme and appearance controls.
4. Once the inference interface exists, connect chat generation to the Rust runtime.
5. Run TypeScript, Rust and Windows lifecycle validation on the user's personal Windows development machine. Do not use the work PC.



## 14. Phase 2 startup lifecycle slice — 2026-11-12

### Implemented in source
- Added the Tauri single-instance plugin as the first registered plugin.
- Added the Rust atomic startup gate and frontend startup-splash handshake.
- Added the branded loading screen with reduced-motion handling. It waits for the initial app metadata handshake and a brief minimum presentation interval; it does not wait for a model.
- Added a native tray icon generated in Rust and Open/Hide/Exit menu actions.
- Added left-click tray restoration where the platform emits tray click events.
- Changed main-window close behavior to hide the window. Explicit tray Exit terminates the process.
- Updated architecture, README, development log and this tracker.

### Changed files
- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src/main.tsx`
- `src/App.tsx`
- `src/services/nativeBridge.ts`
- `src/components/startup/LoadingScreen.tsx` (new)
- `src/styles.css`
- `docs/ARCHITECTURE.md`
- `docs/DEVELOPMENT_LOG.md`
- `CONTEXT_TRACKER.md`
- `README.md`

### Validation
- Source files and repository branch head were inspected before editing.
- No npm install, frontend build, Cargo check, Tauri launch or Windows interaction test was run in this environment.
- Treat all new behavior as implemented in source but not yet runtime-verified.


## 15. Phase 2 desktop shell and navigation — 2026-11-12

### Implemented in source
- Replaced the static foundation screen with a persistent desktop sidebar and page-aware top bar.
- Added frontend navigation for New chat, History, Workspace, Tasks & Notes, File Library, Local models, Appearance and Runtime.
- Added a chat landing screen with Sync Core-inspired artwork, model status and an intentionally disabled composer until inference is connected.
- Added dedicated staging pages for the not-yet-connected feature areas. Each page distinguishes planned behavior from implemented services.
- Added a runtime page that displays only metadata actually returned by the existing Rust command.
- Added responsive layouts, keyboard focus styling and reduced-motion handling.

### Changed files
- \`src/App.tsx\`
- \`src/styles.css\`
- \`src/components/navigation/Sidebar.tsx\` (new)
- \`src/components/pages/ChatPage.tsx\` (new)
- \`src/components/pages/FeaturePage.tsx\` (new)
- \`src/components/pages/RuntimePage.tsx\` (new)
- \`README.md\`
- \`CONTEXT_TRACKER.md\`
- \`docs/ARCHITECTURE.md\`
- \`docs/DEVELOPMENT_LOG.md\`

### Validation
- Source changes are committed to GitHub after review of the previous main-branch files.
- No dependency installation, TypeScript build, Rust build, Tauri launch or Windows UI test has been run from this environment.
- The navigation is implemented in React source; native runtime and feature backends remain unverified or unimplemented.

### Exact next implementation step
- Build the chat interaction layer and Sync Core state component without faking model output: define message/composer state and queue/stop UI contracts, then connect them to the Rust runtime only when the inference interface exists.


## 16. Phase 3 local data foundation — 2026-11-12

### Scope approved
- Persist diagnostics on each individual assistant message.
- Build chat history with search, open, create, rename and delete.
- Support desktop-native, versioned chat backup export/import.
- Explicitly exclude Android backup v1 and raw chats.json compatibility.
- Preserve diagnostics in exported/restored backups.

### Implemented in source
- Added SQLite dependency with bundled SQLite and schema migration v1.
- Added chats, messages, app_settings and schema_migrations tables, foreign keys and indexes.
- Added Rust storage models and Tauri commands for chat CRUD/search, message append, diagnostics update, settings and backup import/export.
- Added sync-ai-desktop-backup format version 1 with validation limits, timestamp/role/ID checks, duplicate chat skip behavior and transactional import.
- Added History UI with conversation search, create/open/rename/delete, per-assistant-message diagnostics disclosure, JSON backup import and export.
- Added typed frontend bridge methods and marked History as an active navigation destination.

### Changed files
- src-tauri/Cargo.toml
- src-tauri/src/lib.rs
- src-tauri/src/commands/mod.rs
- src-tauri/src/commands/chats.rs
- src-tauri/src/commands/settings.rs
- src-tauri/src/storage/mod.rs
- src-tauri/src/storage/models.rs
- src/services/nativeBridge.ts
- src/pages/HistoryPage.tsx
- src/types/navigation.ts
- src/App.tsx
- src/main.tsx
- src/history.css
- README.md
- docs/ARCHITECTURE.md
- docs/ANDROID_TO_DESKTOP_SPEC.md
- docs/PHASE3_DATA_CONTRACT.md
- docs/DEVELOPMENT_LOG.md

### Validation and limitations
- GitHub source files and current main branch head were fetched back after changes.
- Rust unit test source was added for migration, per-message diagnostics persistence, backup round-trip, duplicate chat skipping, invalid diagnostics role and duplicate IDs in an imported backup.
- The SQLite DDL, core list query, diagnostics-role constraint and cascade behavior were smoke-tested separately with Python sqlite3. No Rust/Cargo toolchain is available in the assistant execution environment. No Cargo test/check, npm build, Tauri launch or Windows runtime test has been run.
- Treat all Phase 3 behavior as source-implemented but unverified until checked on the personal Windows development machine.


## Latest update — Phase 2.2 sidebar redesign (source committed)

- Sidebar now follows the approved layout: New chat; searchable recent conversations grouped by Today, Yesterday, Previous 7 days, Previous 30 days and Older; workspace destinations; system destinations; Settings in the footer; local-first footer retained.
- Recent conversations load through the existing SQLite-backed listChats command. Selecting one opens History with that chat selected; See all opens full History.
- Settings is a navigation destination using the staged preview treatment. Individual preference controls are not claimed as implemented.
- No duplicate chat store was created. Android repository remains untouched.
- Validation: source changes committed; no frontend build, Cargo check or Windows runtime/UI test run.

## Phase 4 scope correction — exactly three stages

1. **Model management + resource budget:** GGUF registry/import/hash/deduplication/metadata/selection/removal, host RAM detection, and optional user-enabled managed inference RAM budget with pre-load checks. This is not an OS-enforced hard process memory cap.
2. **Native runtime + lifecycle:** Windows llama.cpp FFI, model load/unload, memory estimation and safe lifecycle handling.
3. **Chat inference connection:** streaming, queueing, cancellation, context handling and per-assistant-message diagnostics.

Workspace, File Library, Tasks & Notes and broader resource metrics remain Phase 5. No 4.1/4.2/4.3-style micro-numbering will be used.
