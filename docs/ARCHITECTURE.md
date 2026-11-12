# Sync AI Desktop Architecture

> This document describes the intended architecture and boundaries, distinguishing source-implemented Phase 2 shell/lifecycle work from planned or runtime-unverified systems.

## 1. Product and technology direction

- Windows x64 first.
- Tauri 2 desktop shell.
- React + TypeScript frontend.
- Vite build tooling.
- npm package manager.
- Rust backend.
- llama.cpp for local GGUF inference.
- SQLite persistence with schema v1, chat/message storage and settings table implemented in source; Windows runtime validation remains pending.
- Offline-first core chat and inference.
- Desktop project remains separate from the Android repository.

Tauri 2 supports React and other JavaScript frameworks. Tauri's maintained create-tauri-app templates include React TypeScript, and its current frontend guidance recommends Vite for SPA frameworks. On Windows Tauri uses Microsoft Edge WebView2.

Official references:
- https://v2.tauri.app/start/create-project/
- https://v2.tauri.app/start/frontend/
- https://v2.tauri.app/start/prerequisites/

## 2. High-level structure

~~~text
React + TypeScript + Vite
        │
        │ typed Tauri IPC
        ▼
Rust application layer
  ├── command boundary and app state
  ├── chat and preference services
  ├── model registry and lifecycle
  ├── inference orchestration
  ├── SQLite storage and migrations
  ├── tool registry and approval broker
  ├── audit/diagnostics
  └── narrow Windows adapters
        │
        ├── llama.cpp native library through a narrow FFI
        └── explicitly approved, operation-specific Windows APIs
~~~

## 3. Frontend responsibilities

The React frontend owns presentation and user interaction:
- Desktop navigation, chat history and message rendering.
- Composer, send/stop behavior and streamed response display.
- Sync Core state visualization.
- Model manager, appearance settings, Workspace and later secondary screens.
- Approval/denial UI for exact proposed operations.
- Accessibility, keyboard navigation and responsive layout.

The Android implementation is the design and behavior reference. Its screens are built programmatically in Java and are not reusable web layouts. Recreate the visual language in HTML/CSS/React instead of trying to embed or translate Android Views.

No React component should directly perform privileged filesystem, process or OS operations. It calls narrow Tauri commands.

## 4. Rust application layer

Rust is the trust and policy boundary. It owns:
- Tauri command handlers and typed IPC structures.
- Input validation and fixed tool registry.
- Chat, settings, model metadata and Workspace persistence.
- Model lifecycle and inference orchestration.
- Approval request creation, validation, expiry and consumption.
- Windows platform adapter access.
- Audit event writing and privacy-aware diagnostics.

The frontend is not a security boundary. Rust independently validates every operation and must fail closed.

## 5. Native inference boundary

The Android app uses llama.cpp pinned at revision 4a89937 through JNI. Its native code has Android-specific dependencies and must not be reused as the desktop boundary.

Desktop uses the pinned high-level Rust crate llama_cpp = 0.3.2. It builds its bundled llama.cpp source and statically links the native libraries by default. The wrapper provides GGUF model loading, context/session creation and native session-size estimation. The first runtime is CPU-only (n_gpu_layers = 0) and keeps a single model/context active.

Stage 2 source adds a bounded GGUF v2/v3 header parser, metadata extraction, a runtime manager, manual load/unload commands, a fixed 2,048-token context, a 128-token batch and pre-load resource checks. The managed RAM budget is compared against a conservative GGUF-based estimate before model weights are loaded. After loading model weights, llama.cpp's own session-size estimate is checked against the configured budget and current available RAM before the context session is allocated. Failures release temporary native allocations.

The GGUF estimates are estimates, not an OS-enforced cap or measured resident-memory values. The runtime must not auto-load at application startup. Chat generation, streamed tokens, stop/cancel and session/context assembly remain Stage 3 work. Windows C++ compilation, linking and actual GGUF load tests remain pending.

## 6. Storage

SQLite data currently includes chats, ordered messages, app settings and the GGUF model registry. Workspace artifacts and their generation state remain future storage work. The existing v2 model columns store architecture and quantization metadata; no new migration was required for Stage 2.

Use schema migrations, foreign keys and transactions for multi-row changes. Keep GGUF model files external to the repository. Keep user data, local databases, prompts, model contents and secrets out of Git.

Chat backups use a desktop-native, versioned format named sync-ai-desktop-backup. Android backup compatibility is intentionally out of scope. Import validates the complete backup before committing, enforces bounds, skips conflicting chat IDs rather than overwriting local data, and never mutates the source file. Export/import preserves diagnostics attached to assistant messages.

The schema and backup contract are specified in docs/PHASE3_DATA_CONTRACT.md; Phase 3 storage and History source are implemented, with Windows runtime validation pending.

## 7. Tool registry and approval broker

Model output is an untrusted proposal, never execution authority.

Every AI-invoked host operation, including reads, follows this flow:

1. Model proposes a registered tool ID and typed arguments.
2. Rust validates the proposal against the allowlisted tool registry.
3. Rust resolves the exact target and presents a human-readable operation summary.
4. User approves or denies that exact operation.
5. Rust validates the single-use approval binding, expiry and request identity.
6. Rust revalidates the canonical target/arguments and executes only the registered operation.
7. Approval is consumed, and a limited result plus safe audit event is returned.

Non-negotiable properties:
- Fresh one-time approval for every operation.
- No blanket grants and no “remember my choice”.
- No generic shell, terminal, script runner or model-controlled executable path.
- No unrestricted filesystem APIs.
- Approval cannot be reused or expanded.
- Windows paths must be canonicalized and checked for traversal, junctions, symlinks/reparse points and time-of-check/time-of-use gaps.
- Limits on file size, payload size, operation duration and scope.
- Denial, cancellation, malformed requests and uncertain targets fail closed.
- Logs omit private file contents, secrets and prompt bodies by default.

Tauri capabilities are a second boundary that controls which plugin/core commands are exposed to a window. They do not make unsafe Rust command implementations safe. Keep capabilities minimal; do not enable broad shell or filesystem command surfaces.

Official security references:
- https://tauri.app/security/
- https://tauri.app/security/capabilities/
- https://tauri.app/security/permissions/
- https://v2.tauri.app/security/csp/

## 8. Windows platform adapters

Keep Windows-specific functionality behind narrow Rust interfaces. Implement only individually reviewed operations. The model must not decide which executable to run or construct raw PowerShell/CMD commands.

Potential future capabilities such as opening an application, reading a selected file, writing an export, querying system information, controlling media or recording audio each need their own operation-specific request, approval summary, validation and test suite.

Android-specific features such as contacts/calls, Android intents, wake-word foreground services, camera routing, Android alarms and SpeechRecognizer/TTS are not portable and are deferred until separately designed.

## 9. Resource monitoring

Report only what the Windows APIs actually measure. Distinguish process-wide CPU and memory from host-wide metrics. Do not invent per-library or per-model memory splits. Mark estimates as estimates and make diagnostics export an explicit user action.

## 10. Startup, single-instance and tray lifecycle

The desktop process owns this lifecycle; React only presents the startup state.

- Register `tauri-plugin-single-instance` before any other plugin. A second launch is forwarded to the existing process, whose callback restores, unminimizes and focuses the `main` window.
- A Rust-managed atomic startup gate is consumed by the first webview bootstrap call. It authorizes the branded startup splash once per process; page reloads and tray restores do not consume another splash.
- The splash is not a model-loading screen. It covers the initial native bootstrap handshake only and must never wait for a GGUF model to load.
- A native tray icon is created by Rust. Left-click restores the window on platforms that emit tray click events; the context menu offers Open, Hide and Exit.
- Closing the main window prevents destruction and hides it to the tray. Exit from the tray menu terminates the process. The next process launch receives a fresh startup gate and splash.
- The tray icon artwork is generated as a small RGBA image in Rust until final application branding assets are added.
- This lifecycle source is implemented, but Windows compilation and interaction tests remain pending.

## 11. Current scaffold and implementation status

Created in the repository:
- React 19 + TypeScript 6 frontend structure with Vite 8 tooling.
- Tauri 2 Rust application entry point.
- Read-only get_app_info command returning static app version, OS and architecture metadata.
- consume_startup_splash command backed by a Rust atomic gate.
- Typed frontend bridge for both commands.
- Persistent React sidebar with searchable SQLite-backed recent conversations, workspace/system navigation, dedicated Settings destination and page-aware top bar.
- Chat landing screen with Sync Core-inspired artwork and a disabled composer until model integration.
- Staging pages for History, Workspace, Tasks & Notes, File Library, Local Models and Appearance.
- Runtime page that displays only metadata returned by the Rust bridge.
- First-process startup splash with an atomic Rust gate so tray restores and same-process webview reloads do not replay it.
- Rust single-instance plugin registration and main-window focus/restore callback.
- Native system tray with generated Sync mark, Open/Hide/Exit actions, left-click restore where supported, and close-to-tray behavior.
- Reduced-motion CSS handling.
- Restrictive CSP with no remote resource sources.
- Minimal main-window capability with core:default only.
- No shell, filesystem, process, opener or other plugin surface.
- Tauri bundling disabled until icon assets and installer decisions are ready.

The app metadata and splash-gate commands do not inspect user files or perform privileged operations. Sidebar navigation, SQLite History, Local Models, Runtime, Workspace, Tasks & Notes and File Library are source-connected. Phase 4 Stage 3 chat generation is source-connected; Phase 5 adds bounded opt-in context from selected Workspace artifacts and notes. Full application validation remains pending.

## 12. Current code organization

~~~text
src/
  App.tsx
  main.tsx
  components/
    navigation/Sidebar.tsx
    pages/
      ChatPage.tsx
      FeaturePage.tsx
      RuntimePage.tsx
    startup/LoadingScreen.tsx
  services/
    nativeBridge.ts
  styles.css

src-tauri/
  capabilities/
  src/
    commands/
      app_info.rs
    lib.rs
    main.rs
  Cargo.toml
  build.rs
  tauri.conf.json
~~~

Additional folders will be created as implementation boundaries are needed; do not populate the repository with empty placeholders.

## 13. Open implementation decisions

- Windows C++/libclang toolchain setup, Cargo compilation and llama.cpp runtime validation.
- Minimum supported Windows version and CPU baseline.
- Scoped file access model and import strategy.
- Windows voice recognition and TTS engines.
- Packaging, signing and update strategy.
- License selection and third-party notices.

## 14. Validation status

The project has source implementations through Phase 5, but a full Windows validation pass has not been completed. GitHub source review and a separate SQLite smoke test do not establish that the frontend, Rust backend, native inference worker or complete Tauri application builds and runs.

Outstanding checks include npm dependency installation, TypeScript/production build, Cargo formatting/check/tests, MSVC/libclang/llama.cpp compilation, clean and upgrade SQLite migration tests, Tauri IPC, tray/single-instance behavior, real GGUF loading, Job Object enforcement, Phase 5 UI persistence and end-to-end chat/workspace integration.

A Python SQLite smoke test has checked the Phase 5 migration DDL, selected-context query, foreign-key nulling and task-status constraint. It is only a database SQL smoke test; Rust and Windows integration remain unverified.

## 15. Phase 4 delivery shape

Phase 4 remains exactly **three stages**:

1. **Model management:** GGUF header validation, SHA-256 hashing, deduplication, original-path registry, missing-file detection, default selection and non-destructive removal.
2. **Native runtime:** pinned llama.cpp CPU runtime inside a dedicated child worker; explicit load/unload; optional Windows Job Object private-commit ceiling; worker telemetry. The Tauri/WebView process stays outside the Job Object.
3. **Chat inference connection:** SQLite-backed conversation context, streamed tokens, sequential queueing, cancellation and diagnostics attached to the exact assistant response.

The process memory ceiling includes model and session allocations. It is disabled by default; a cap that is too small can prevent model loading. Model-size estimates are informational.

## 16. Phase 4 Stage 1 — model registry

New GGUF imports use the native file picker, validate GGUF metadata, hash the selected original file and register its canonical path. The model is not copied or moved. Duplicate hashes can relink an existing registry entry. Missing files are marked unavailable and checked again by Rust before load/default selection. Removing a model from Sync removes only its registry row and does not delete the source file. SQLite migration v3 preserves compatibility with legacy managed-copy records.

## 17. Phase 4 Stage 2 and Stage 3 status

- Native binding: `llama_cpp = 0.3.2`, exact version pin; bundled llama.cpp requires the Windows C++ toolchain and LLVM/libclang.
- Runtime: CPU-only, one loaded model, explicit load/unload and a 2,048-token context.
- Worker: llama.cpp model/session ownership runs in a dedicated child process. On Windows, an optional Job Object committed-private-memory ceiling is assigned to the worker before model load. mmap is disabled so model weights contribute to worker private allocations.
- Isolation: the Tauri/WebView UI process stays outside the worker Job Object. Runtime telemetry reports worker PID, configured ceiling, working set, private committed bytes and worker errors.
- Stage 3: chat generation is source-connected to SQLite persistence, generic role-labelled context, streamed output, cancellation, sequential queueing and per-assistant-message diagnostics.
- Deferred: model-specific GGUF chat templates, configurable sampling and automatic model loading.
- Validation: frontend build, Cargo check/test, Windows native compilation, Job Object behavior, real model load and end-to-end chat remain pending.

## Phase 4 correction — isolated inference worker

The Phase 4 model registry stores canonical original GGUF paths for new imports; it does not copy model files. Missing paths are marked unavailable. Removing a registry entry is non-destructive, including for legacy managed-copy entries. Schema migration v3 adds source_path while preserving the legacy path column for older rows.

llama.cpp model/session state now belongs to a dedicated child process launched in inference-worker mode. On Windows the supervisor creates a Job Object, configures an optional JOB_OBJECT_LIMIT_PROCESS_MEMORY ceiling and assigns the worker before requesting model load. The UI/WebView process remains outside the job. Model loading disables mmap so model weights use private worker allocations subject to the commit ceiling. A too-small ceiling can prevent model loading; this is a hard limit, not a context-only budget.

Runtime diagnostics expose the configured cap, worker PID, resident working set, private committed memory and the latest worker error. Working set and private commit are distinct values. The library's model memory estimate is informational and does not enforce the cap. The cap is disabled by default and changes apply on a subsequent model load.

**Validation note:** these Phase 4 changes are committed as source only until checked on Windows. Cargo/API compatibility, frontend build, Job Object assignment/limit behavior, real GGUF loading and worker-failure recovery must still be tested. Stage 3 chat generation/streaming is source-connected; Windows/native validation is pending.

## Phase 4 revision — isolated worker and chat connection

New desktop GGUF imports register canonical original paths and SHA-256 without copying files. Migration v3 retains legacy managed-copy compatibility. Missing paths are checked in Rust before loading/default selection. Registry removal never deletes the GGUF. Loading a replacement explicitly unloads the current worker.

llama.cpp runs in a dedicated child process. Windows Job Objects apply an optional JOB_OBJECT_LIMIT_PROCESS_MEMORY ceiling to the worker while the Tauri/WebView interface remains outside the job. Model mmap is disabled; Runtime reports ceiling, PID, working set, private committed bytes and worker errors.

The chat path now uses SQLite persistence, role-labelled context, streamed output, cancellation, sequential queueing and diagnostics on the exact assistant response. Queued assistant replies are inserted after their triggering user turns. Generic role-labelled prompts are used initially; model-specific GGUF chat-template rendering is not yet implemented.

Validation remains pending: frontend build, Cargo check/test, Job Object behavior, native compilation, real model load, streaming/cancellation and Windows UI testing.


## 18. Phase 5 — Workspace and productivity

SQLite migration v4 adds `workspace_artifacts`, `tasks`, `notes` and `library_files`. Rust storage and Tauri commands back the Workspace editor, Tasks & Notes and File Library pages.

Workspace supports artifact creation/search/edit/delete and saving assistant replies. Notes can be pinned and marked for context. Only artifacts and notes explicitly selected for context enter future local prompts, with a 20-entry and 48 KiB aggregate cap. Task data is not sent to the model.

File Library uses a native file picker and stores a canonical original path, name/extension, size and SHA-256. It can detect missing paths and relink entries. Removal deletes only the registry row. The original file is not copied, moved or deleted, and the library does not read its content into chat. Any future AI-requested file operation belongs to Phase 6 and requires explicit one-time approval.

The existing chat backup format v1 remains unchanged and does not yet include Phase 5 tables. See `docs/PHASE5_WORKSPACE.md` for data constraints, native command contracts and acceptance tests.

Phase 5 source is committed but has not been built or runtime-tested on Windows. The separate SQLite smoke test does not validate Rust command wiring, frontend TypeScript or native picker behavior.
