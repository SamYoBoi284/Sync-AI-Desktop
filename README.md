# Sync AI Desktop

A Windows-first, offline-first desktop companion to the Sync AI Android project.

This repository is intentionally separate from [Sync-AI](https://github.com/SamYoBoi284/Sync-AI). The Android application remains its own product and its source is treated as a reference for desktop parity, not as a target for desktop changes.

## Project direction

- **Desktop framework:** Tauri 2
- **Backend:** Rust
- **Frontend:** React + TypeScript (proposed; confirm before scaffolding)
- **Local inference:** llama.cpp with GGUF models
- **Persistence:** SQLite (planned)
- **Initial platform:** Windows x64
- **Operating principle:** offline-first; no cloud service required for core chat or inference

The frontend will recreate and adapt the Android application's visual language and workflows for desktop. It will not attempt to transplant Android Java UI code directly.

## Security principle

Sync AI must never have unrestricted access to the host computer. Every tool operation must be presented for explicit, one-time user approval. Approval applies only to the exact proposed action and scope, expires after that action, and cannot silently become a permanent permission. Model output must never directly execute arbitrary commands.

## Repository map

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the initial architecture and [CONTEXT_TRACKER.md](CONTEXT_TRACKER.md) for the authoritative project handoff and development state.

## Development status

**Foundation planning.** No application runtime or inference integration has been implemented yet.

## Source reference

- Android reference repository: https://github.com/SamYoBoi284/Sync-AI
- Initial desktop audit baseline: `feat/resource-manager-task-manager-17-10-2026`

## License

License has not yet been selected. Do not assume this project is open-source licensed until a LICENSE file is added.
