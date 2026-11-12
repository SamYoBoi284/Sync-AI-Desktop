# Sync AI Desktop

A Windows-first, offline-first desktop companion to the Sync AI Android project.

This repository is intentionally separate from [Sync-AI](https://github.com/SamYoBoi284/Sync-AI). The Android application remains its own product and its source is treated as a reference for desktop parity, not as a target for desktop changes.

## Project direction

- **Desktop framework:** Tauri 2
- **Backend:** Rust
- **Frontend:** React + TypeScript
- **Frontend build tool:** Vite
- **Package manager:** npm
- **Local inference:** pinned llama_cpp Rust bindings with bundled llama.cpp and GGUF (Stage 2 source added; Windows build pending)
- **Persistence:** SQLite (chat/history and model registry source implemented; runtime validation pending)
- **Initial platform:** Windows x64
- **Operating principle:** offline-first; no cloud service required for core chat or inference

The frontend recreates and adapts the Android application's visual language and workflows for desktop. It does not attempt to transplant Android Java UI code directly.

## Current development status

**Current milestone: Phase 4 Stage 2 source committed.** The desktop shell, Phase 2.2 sidebar, Phase 3 SQLite chat/history layer and Phase 4 Stage 1 model registry and Stage 2 native runtime source are committed. Startup/tray, database, model import, native dependency and UI changes have not been compiled or runtime-tested on Windows.

The sidebar now includes searchable recent conversations sourced from SQLite History, workspace/system navigation and a dedicated Settings destination. Settings includes the optional RAM Limiter; appearance and other preference controls remain staged.

The New chat composer remains intentionally disabled until local inference is connected. Chat storage and History source are implemented in Phase 3; Windows runtime validation remains pending.

**Phase 4 Stage 2 details:** The Rust GGUF header reader extracts model architecture, name, quantization and memory dimensions. The pinned llama_cpp 0.3.2 wrapper builds llama.cpp from source; the first runtime is CPU-only, loads only on explicit user action, and creates a bounded 2,048-token context session. Model load/unload status, memory estimates, RAM-budget refusal and currently-available-RAM checks are connected in source. The limiter is still an application-level guard, not a hard OS process cap. No native build or Windows runtime validation has been performed.

**Phase 3 implementation source is committed.** The repository now contains the SQLite storage layer, typed Rust commands, per-message diagnostics data model, desktop backup validation/round-trip tests, and a connected History UI with search, rename, delete, import and export source. These changes have not been built or runtime-tested yet. Android backup compatibility is explicitly out of scope.

Not implemented yet:
- Appearance and broader preference controls (the RAM Limiter is implemented in source; theme/accent/motion controls remain planned)
- Chat generation, streaming, queueing and cancellation
- Windows build/runtime validation of the new SQLite and History implementation
- Chat generation, token streaming, queueing and cancellation (the native model/context lifecycle is Stage 2; chat generation is Stage 3)
- Workspace storage
- File Library or Tasks & Notes functionality
- Voice, camera or recorder features
- Host-computer tools or approval execution

## Next milestone: Phase 4 local model engine

Phase 4 is deliberately organized into **three stages only**:

1. Model management: GGUF import, hashing, deduplication, metadata, selection/removal, hardware RAM detection and an optional user-enabled managed inference RAM budget. **Stage 1 source is committed; Windows validation pending.**
2. Native runtime: Windows llama.cpp integration, GGUF metadata parsing, explicit model load/unload, context session creation, memory estimates and pre-load safeguards. **Stage 2 source is committed; native build/runtime validation pending.**
3. Chat inference: streaming, queueing, cancellation, context handling and diagnostics attached to each assistant message.

The optional RAM budget will initially be an application-level inference allocation guard, not a claim of an OS-enforced hard process memory cap. Workspace, File Library, Tasks & Notes and broader resource metrics remain in Phase 5.

## Run locally on Windows

Install the Windows prerequisites first:
- Rust stable with the MSVC toolchain
- Microsoft C++ Build Tools with **Desktop development with C++**
- LLVM/libclang available to the Rust native dependency build
- Microsoft Edge WebView2
- Node.js compatible with the installed Vite version, plus npm

See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Then, from the repository root:

~~~powershell
npm install
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
~~~

Running only **npm run dev** opens the browser preview. The Rust bridge is available only inside the Tauri desktop host.

## Security principle

Sync AI must never have unrestricted access to the host computer. Every AI-invoked tool operation, including reads, requires fresh one-time approval for its exact action and scope. Approval expires after that operation and cannot become a permanent permission. Model output is an untrusted proposal, never execution authority.

The main-window capability still exposes only the core default capability. No shell, filesystem, process or opener plugin is installed. The tray mark is currently generated in Rust; final application icon assets and installer configuration are still pending, so packaging remains disabled.

## Repository map

- [Port specification](docs/ANDROID_TO_DESKTOP_SPEC.md): Android source map, UI behavior, feature mapping and implementation sequence.
- [Architecture](docs/ARCHITECTURE.md): desktop trust boundaries and target layers.
- [Context Tracker](CONTEXT_TRACKER.md): authoritative project handoff and development state.
- [Development Log](docs/DEVELOPMENT_LOG.md): chronological implementation record.

## Source reference

- Android reference repository: https://github.com/SamYoBoi284/Sync-AI
- Initial desktop audit baseline: feat/resource-manager-task-manager-17-10-2026
- Official Tauri project creation guide: https://v2.tauri.app/start/create-project/
- Official Tauri Vite frontend guide: https://v2.tauri.app/start/frontend/

## License

License has not yet been selected. Do not assume this project is open-source licensed until a LICENSE file is added.
