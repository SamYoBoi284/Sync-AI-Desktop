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
- SQLite as the planned persistence layer; schema/migrations remain open.
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

Desktop should compile llama.cpp for Windows and expose a narrow, stable interface to Rust. Use opaque handles and explicit load/unload/generate/cancel/diagnostics operations. Token streaming must have clear callback lifetime and cancellation rules. Do not allow C++ exceptions, Rust panics or invalid pointers to cross the FFI boundary.

Before integrating inference into chat, build a minimal Windows proof of concept and verify load, streamed generation, cancellation, unload and repeated lifecycle.

The exact C ABI / Rust FFI wrapper mechanism remains an implementation spike.

## 6. Storage

Planned SQLite data:
- Chats and ordered messages.
- Preferences and application settings.
- Model registry metadata.
- Workspace artifacts and their generation state.

Use schema migrations, foreign keys and transactions for multi-row changes. Keep GGUF model files external to the repository. Keep user data, local databases, prompts, model contents and secrets out of Git.

Chat backups use a desktop-native, versioned format named sync-ai-desktop-backup. Android backup compatibility is intentionally out of scope. Import validates the complete backup before committing, enforces bounds, skips conflicting chat IDs rather than overwriting local data, and never mutates the source file. Export/import preserves diagnostics attached to assistant messages.

The schema and backup contract are specified in docs/PHASE3_DATA_CONTRACT.md; implementation remains pending.

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
- Persistent React sidebar with functional page navigation and page-aware top bar.
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

The metadata and splash-gate commands do not inspect user files or perform privileged operations. Sidebar navigation is functional, but the feature pages do not claim that their storage, inference or OS services are connected.

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

- SQLite migration hardening, rollback tests and Windows runtime validation.
- llama.cpp build configuration and FFI wrapper.
- Minimum supported Windows version and CPU baseline.
- Scoped file access model and import strategy.
- Windows voice recognition and TTS engines.
- Packaging, signing and update strategy.
- License selection and third-party notices.

## 14. Validation status

The scaffold, lifecycle slice and navigable desktop shell have been committed to GitHub and the source files have been retrieved for review. The project has not yet been built on Windows in this session. Dependency installation, npm typecheck/build, cargo check, Tauri development launch, single-instance forwarding, tray interactions, close-to-tray behavior and installer packaging are therefore unverified.
