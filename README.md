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

**Phase 2.1 desktop shell source prepared.** The app has a navigable desktop shell, a New chat landing view, a Runtime view, and clearly-labelled preview screens. Startup splash, single-instance handling and tray lifecycle source are committed but have not been compiled or runtime-tested on Windows.

The New chat composer remains intentionally disabled until chat persistence and inference are connected.

**Phase 3 implementation is in progress.** The repository now contains the SQLite storage layer, typed Rust commands, per-message diagnostics data model, desktop backup validation/round-trip tests, and a connected History UI with search, rename, delete, import and export source. These changes have not been built or runtime-tested yet. Android backup compatibility is explicitly out of scope.

Not implemented yet:
- Chat generation, streaming, queueing and cancellation
- Windows build/runtime validation of the new SQLite and History implementation
- GGUF model import/load/unload and llama.cpp inference
- Workspace storage
- File Library or Tasks & Notes functionality
- Voice, camera or recorder features
- Host-computer tools or approval execution

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
