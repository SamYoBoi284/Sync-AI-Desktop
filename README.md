# Sync AI Desktop

A Windows-first, offline-first desktop companion to the Sync AI Android project. The desktop application is maintained in this repository; the Android repository remains separate and is used only as a behavioral and design reference.

## Technology

- **Desktop:** Tauri 2
- **Frontend:** React + TypeScript + Vite
- **Native backend:** Rust
- **Local inference:** pinned `llama_cpp = 0.3.2`, bundled llama.cpp, GGUF models
- **Persistence:** SQLite with versioned schema migrations
- **Initial target:** Windows x64
- **Core operation:** offline-first; no cloud service is required for local chat or local persistence

## Current project status

**Phases 0–5 have source implementations committed. Windows build and runtime validation remain outstanding.** “Implemented” means source is in `main`; it does not mean the desktop application has passed a Windows build or runtime test.

### Completed source scope

- **Phase 0 · Android audit:** desktop port specification and source map.
- **Phase 1 · Foundation:** Tauri 2, React/TypeScript, Rust command bridge and restrictive capability baseline.
- **Phase 2 · Desktop shell:** startup splash, single-instance handling, tray lifecycle, navigation, recent chats and settings foundation.
- **Phase 3 · Chat data:** SQLite chats/messages/settings, History UI, per-assistant-message diagnostics and versioned desktop backup import/export.
- **Phase 4 · Local model engine:** exactly three stages:
  1. GGUF library, SHA-256 registration/deduplication, original-path tracking and non-destructive registry removal.
  2. CPU llama.cpp runtime in an isolated worker, optional Windows Job Object private-commit ceiling, load/unload lifecycle and runtime telemetry.
  3. SQLite-backed chat inference, streaming, cancellation, sequential queueing and per-message diagnostics.
- **Phase 5 · Workspace and productivity:** persistent Workspace artifacts, Tasks & Notes, selected-file File Library, saving assistant replies into Workspace and explicit opt-in context from selected notes/artifacts.

### Phase 5 details

- Workspace artifacts are stored in SQLite and support create, search, edit and delete. Assistant replies can be saved as Workspace documents.
- Notes can be pinned and individually marked for inclusion in future chat context.
- Workspace artifacts have their own explicit “Include in chat context” setting.
- Only opted-in notes and Workspace artifacts are sent as reference material with future local prompts. The current context builder caps the combined material at 48 KiB and 20 entries.
- Tasks support title, details, status, priority and optional due date.
- File Library opens a native file picker, records the selected file's canonical path, size and SHA-256, and detects missing paths. It does not move, copy or delete the original file.
- **File Library contents are not read into chat.** The current feature is a path-and-fingerprint registry, not an AI file reader. Any future AI-initiated file operation must go through Phase 6's explicit one-time approval model.

Phase 5 uses SQLite schema migration v4. The previous chat backup format remains version 1 and still exports chats/messages/diagnostics only; Workspace, tasks, notes and File Library records are not yet included in chat backup export/import.

## Roadmap

- **Phase 6 · Secure tools and Device Assist:** registered operations, exact-scope one-time approval, Rust-side validation and narrow Windows adapters. No unrestricted shell, filesystem or process access.
- **Phase 7 · Finalization and release:** voice, diagnostics/optimization, packaging, installer, security review and complete testing.

Appearance theme/accent/motion controls, model-specific GGUF chat templates, voice, multimodal support and startup model auto-load remain deferred. Phase 4 remains exactly three stages.

## Windows development setup

Install and verify:

- Rust stable with the MSVC toolchain
- Microsoft C++ Build Tools with **Desktop development with C++**
- LLVM/libclang for the native llama.cpp binding build
- Microsoft Edge WebView2
- Node.js compatible with the installed Vite version, plus npm

See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

From the repository root:

~~~powershell
npm install
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
~~~

Running only `npm run dev` opens a browser preview. Native Rust commands and local inference are available only inside the Tauri desktop host.

## Security boundary

The model is not an authority to operate the host computer. The webview has no shell, filesystem, process or opener plugin. File Library registration and GGUF import use native, user-invoked pickers; these features record metadata and do not grant model access to arbitrary files.

Every future AI-invoked host operation, including reads, must require fresh, explicit, one-time approval for its exact operation and scope. Rust independently validates the request. Model output is an untrusted proposal, never execution authority.

## Repository map

- [Android-to-desktop port specification](docs/ANDROID_TO_DESKTOP_SPEC.md)
- [Architecture and trust boundaries](docs/ARCHITECTURE.md)
- [Phase 3 data contract](docs/PHASE3_DATA_CONTRACT.md)
- [Phase 4 implementation and validation status](docs/PHASE4_REVISION.md)
- [Phase 5 Workspace and productivity contract](docs/PHASE5_WORKSPACE.md)
- [Context tracker](CONTEXT_TRACKER.md)
- [Development log](docs/DEVELOPMENT_LOG.md)

## License

A project license has not yet been selected. Do not assume the repository is open-source licensed until a LICENSE file is added.
