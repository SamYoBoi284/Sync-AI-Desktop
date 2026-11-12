# Sync AI Desktop

A Windows-first, offline-first desktop companion to the Sync AI Android project.

This repository is intentionally separate from [Sync-AI](https://github.com/SamYoBoi284/Sync-AI). The Android application remains its own product and its source is treated as a reference for desktop parity, not as a target for desktop changes.

## Project direction

- **Desktop framework:** Tauri 2
- **Backend:** Rust
- **Frontend:** React + TypeScript (approved)
- **Frontend build tool:** Vite (selected)
- **Local inference:** llama.cpp with GGUF models
- **Persistence:** SQLite (planned; schema not yet implemented)
- **Initial platform:** Windows x64
- **Operating principle:** offline-first; no cloud service required for core chat or inference

The frontend will recreate and adapt the Android application's visual language and workflows for desktop. It will not attempt to transplant Android Java UI code directly.

## Security principle

Sync AI must never have unrestricted access to the host computer. Every AI-invoked tool operation, including reads, requires fresh one-time approval for its exact action and scope. Approval expires after that operation and cannot become a permanent permission. Model output is an untrusted proposal, never execution authority.

## Repository map

- [Port specification](docs/ANDROID_TO_DESKTOP_SPEC.md): Android source map, UI behavior, feature mapping and implementation sequence.
- [Architecture](docs/ARCHITECTURE.md): desktop trust boundaries and target layers.
- [Context Tracker](CONTEXT_TRACKER.md): authoritative project handoff and development state.
- [Development Log](docs/DEVELOPMENT_LOG.md): chronological implementation record.

## Development status

**Source audit and specification complete.** No desktop application runtime, native inference integration or host-computer tool has been implemented yet.

## Source reference

- Android reference repository: https://github.com/SamYoBoi284/Sync-AI
- Initial desktop audit baseline: feat/resource-manager-task-manager-17-10-2026

## Windows development prerequisites

Before scaffolding/building, verify Rust stable MSVC, Microsoft C++ Build Tools with Desktop development with C++, Microsoft Edge WebView2, and Node.js LTS/npm. See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## License

License has not yet been selected. Do not assume this project is open-source licensed until a LICENSE file is added.
