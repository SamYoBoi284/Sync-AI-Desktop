# Sync AI Desktop

A Windows-first, offline-first desktop companion to the Sync AI Android project.

This repository is intentionally separate from [Sync-AI](https://github.com/SamYoBoi284/Sync-AI). The Android application remains its own product and its source is treated as a reference for desktop parity, not as a target for desktop changes.

## Project direction

- **Desktop framework:** Tauri 2
- **Backend:** Rust
- **Frontend:** React + TypeScript
- **Frontend build tool:** Vite
- **Package manager:** npm
- **Local inference:** llama.cpp with GGUF models (not integrated yet)
- **Persistence:** SQLite (schema v1 and storage source implemented; runtime validation pending)
- **Initial platform:** Windows x64
- **Operating principle:** offline-first; no cloud service required for core chat or inference

The frontend recreates and adapts the Android application's visual language and workflows for desktop. It does not attempt to transplant Android Java UI code directly.

## Current development status

**Phase 2.2 sidebar refinement source committed.** The app has a navigable desktop shell, a New chat landing view, a Runtime view, and clearly-labelled preview screens. Startup splash, single-instance handling and tray lifecycle source are committed but have not been compiled or runtime-tested on Windows.

The sidebar now includes searchable recent conversations sourced from SQLite History, workspace/system navigation and a dedicated Settings destination. Settings controls are staged and are not presented as implemented.

The New chat composer remains intentionally disabled until local inference is connected. Chat storage and History source are implemented in Phase 3; Windows runtime validation remains pending.

**Phase 4 Stage 1 source is committed.** The GGUF model registry/import UI, SHA-256 deduplication, managed model storage, default selection/removal and host RAM detection are now implemented in source. The optional RAM Limiter preference is persisted in SQLite; model-load enforcement remains Stage 2 work. No build/runtime validation has been performed.

**Phase 3 implementation source is committed.** The repository now contains the SQLite storage layer, typed Rust commands, per-message diagnostics data model, desktop backup validation/round-trip tests, and a connected History UI with search, rename, delete, import and export source. These changes have not been built or runtime-tested yet. Android backup compatibility is explicitly out of scope.

Not implemented yet:
- Individual Settings controls and preferences UI (the Settings destination is currently a staged page)
- Chat generation, streaming, queueing and cancellation
- Windows build/runtime validation of the new SQLite and History implementation
- GGUF model loading/unloading and llama.cpp inference (registry/import source now implemented)
- Workspace storage
- File Library or Tasks & Notes functionality
- Voice, camera or recorder features
- Host-computer tools or approval execution

## Next milestone: Phase 4 local model engine

Phase 4 is deliberately organized into **three stages only**:

1. Model management: GGUF import, hashing, deduplication, metadata, selection/removal, hardware RAM detection and an optional user-enabled managed inference RAM budget. **Stage 1 source is committed; Windows validation pending.**
2. Native runtime: Windows llama.cpp integration, model load/unload, memory estimates and pre-load safeguards.
3. Chat inference: streaming, queueing, cancellation, context handling and diagnostics attached to each assistant message.

The optional RAM budget will initially be an application-level inference allocation guard, not a claim of an OS-enforced hard process memory cap. Workspace, File Library, Tasks & Notes and broader resource metrics remain in Phase 5.

## Run locally on Windows

Install the Windows prerequisites first:
- Rust stable with the MSVC toolchain
- Microsoft C++ Build Tools with **Desktop development with C++**
- Microsoft Edge WebView2
- Node.js compatible with the installed Vite version, plus npm

See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Then, from the repository root:

~~~powershell
npm install
npm run build
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
