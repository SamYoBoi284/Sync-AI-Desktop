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

**Phase:** 2 — desktop lifecycle foundation (first slice)  
**Status:** Phase 1 source remains committed. The first Phase 2 slice now adds a first-process startup splash, Rust single-instance enforcement, a native tray icon/menu, close-to-tray behavior and restore/focus handling. These changes are source-committed but have not been built or run on Windows. Chat, inference, persistence and computer-control features are not implemented.

### Verified in this repository

- Public repository exists; default branch is main.
- Detailed Android-to-desktop port specification is committed at docs/ANDROID_TO_DESKTOP_SPEC.md.
- React + TypeScript + Vite frontend files are committed.
- Tauri 2 Rust source/configuration files are committed.
- get_app_info is registered as the only custom Rust command.
- The main-window capability contains core:default only.
- No shell, filesystem, process or opener plugin is added.
- The frontend status page distinguishes the native bridge from browser preview.
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
- SQLite schema or migrations
- Chat persistence or Android backup import/export
- GGUF model manager
- llama.cpp Windows build/integration
- Workspace, File Library, Tasks & Notes or voice
- Permission/approval UI or enforcement
- Windows tools/system adapters
- Automated tests, CI, installer, app icons or packaging

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
| Persistence | SQLite | Proposed; schema not designed yet |
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
- Chat backup uses format sync-ai-chat-backup version 1; raw chats.json arrays are also accepted. Import stages validation and skips existing IDs rather than overwriting.
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
- Preserve conversation queue, streaming, cancellation and Android backup compatibility.
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
| src/App.tsx | Foundation UI, planned navigation labels, runtime bridge status |
| src/styles.css | Responsive baseline and reduced-motion handling |
| src/services/nativeBridge.ts | Typed call to the Rust metadata command |
| src-tauri/Cargo.toml | Rust/Tauri dependencies |
| src-tauri/build.rs | Tauri build integration |
| src-tauri/src/main.rs | Windows desktop entry point |
| src-tauri/src/lib.rs | Tauri builder and command registration |
| src-tauri/src/commands/app_info.rs | Harmless static app/OS/architecture information |
| src-tauri/tauri.conf.json | Main window, Vite commands, restrictive CSP, packaging disabled |
| src-tauri/capabilities/main.json | Minimal main-window capability (core:default only) |

The status/navigation screen is a foundation placeholder, not an implemented chat UI. Navigation labels are explicitly planned and inert; the screen must not imply those features work.

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
3. Phase 2 — Desktop shell, startup/tray lifecycle, sidebar, chat UI, Sync Core, themes and settings. Startup splash, single-instance and tray lifecycle source are implemented; Windows validation is pending.
4. Phase 3 — SQLite schema/migrations, chat CRUD, Android backup import/export.
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

On the personal Windows development machine only, verify Node.js 22.12+ (or 20.19+), Rust stable MSVC 1.90+, Microsoft C++ Build Tools with Desktop development with C++, and WebView2. Run `npm install`, `npm run build`, `cargo check` from `src-tauri`, then `npm run tauri dev`. Test: first launch shows the splash once; tray restore does not; closing hides to tray; tray Open restores; a second executable launch focuses the existing process; tray Exit terminates it. Do not use the work PC or add filesystem/shell access as a shortcut.


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
