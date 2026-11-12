# Sync AI Desktop — Context Tracker

> **Purpose:** This is the durable handoff record for future development sessions. Update it in the same change as every meaningful code edit. Keep verified facts separate from proposals, and never claim work is tested unless it was actually run.

## 1. Project identity

- **Project:** Sync AI Desktop
- **Repository:** https://github.com/SamYoBoi284/Sync-AI-Desktop
- **Android companion/reference:** https://github.com/SamYoBoi284/Sync-AI
- **Default branch:** `main`
- **Initial target:** Windows x64
- **Product direction:** Offline-first desktop AI assistant with local GGUF inference and tightly permissioned host-computer tools.
- **Repository separation:** Desktop work must not modify the Android repository.

## 2. Current status

**Phase:** 0 — Repository foundation and architecture planning  
**Status:** Repository skeleton and initial documentation created. No application code, UI runtime, model inference, or desktop tools have been implemented.

### Verified in this repository

- Repository exists and is public.
- Default branch is `main`.
- Repository was empty before initialization.
- Initial documentation and directory placeholders are being established.

### Not yet implemented

- Tauri application scaffold
- React/TypeScript frontend
- Rust backend commands
- llama.cpp build/integration
- GGUF model manager
- SQLite persistence
- Permission/approval UI and enforcement
- Windows packaging or installer
- Automated tests or CI

## 3. Agreed product requirements

1. Windows first; cross-platform support can be considered later.
2. Preserve the Android repository as a separate project.
3. Adapt the Android UI and design language for desktop rather than directly porting Android Java UI code.
4. Offline-first operation. Core chat and local inference must not depend on cloud services.
5. Use llama.cpp as the native inference engine for GGUF models; Rust should orchestrate it through a defined native boundary rather than reimplementing inference.
6. No unrestricted shell, terminal, filesystem, process, or operating-system access for the model.
7. Every host-computer tool operation requires a fresh, explicit, one-time user approval.
8. Approval is limited to the exact action and scope shown to the user; it expires after that operation. No persistent blanket grants or silent permission memory.
9. The backend must validate every operation independently. Model-generated tool requests are proposals, never executable authority.
10. Keep model weights, secrets, local databases, and user data out of Git.

## 4. Technology decisions

| Area | Current direction | Status |
|---|---|---|
| Desktop shell | Tauri 2 | Agreed |
| Backend | Rust | Agreed |
| Frontend | React + TypeScript | Proposed; confirm before scaffolding |
| Build tooling | Vite | Proposed |
| Inference | llama.cpp + GGUF | Agreed direction |
| Persistence | SQLite | Proposed |
| Config | Local structured config (TOML/JSON) | Proposed |
| First platform | Windows x64 | Agreed |

Do not treat proposed choices as implemented dependencies. Revisit them if source inspection or a proof of concept reveals a concrete issue.

## 5. Android reference audit

Initial audit baseline: branch `feat/resource-manager-task-manager-17-10-2026` in the Android repository.

Important source areas identified:

| Android source | Desktop reference responsibility |
|---|---|
| `MainActivity.java` | Main chat screen and primary interaction flow |
| `SideDashboard.java` | Navigation and settings entry points |
| `AppTheme.java` | Theme definitions and visual tokens |
| `AppearanceActivity.java` | Appearance controls |
| `SyncCoreView.java` | Animated Sync Core and status states |
| `AppPreferences.java` | Preference concepts and defaults |
| `ChatStore.java` | Conversation persistence behavior |
| `ModelManager.java` | Local model lifecycle and metadata |
| `GgufModelBackend.java` | Inference orchestration reference |
| `GgufNative.java` | JNI boundary reference; desktop needs a different native boundary |
| `ToolEngine.java` | Existing tool concepts; desktop security must be stricter |
| `ResourceManagerActivity.java` | Resource monitoring concepts |

The Android UI is built programmatically in Java, not with XML layouts or Jetpack Compose. Desktop UI therefore needs a deliberate React/HTML/CSS reconstruction using Android behavior and appearance as reference.

Visual concepts identified:
- Six identity themes: Obsidian, Nullverox, Minimal, Holographic, Command, Aurora.
- Eight accent colors: Purple, Blue, Cyan, Green, Lime, Orange, Red, Pink.
- Three appearance modes: System, Dark, Light.
- Sync Core states: Idle, Ready, Listening, Thinking, Speaking, Executing, Success, Error.

These are audit notes, not a claim that every Android screen or behavior has been fully mapped. Inspect the relevant source again before implementing parity.

## 6. Initial target architecture

- Frontend: presentation, navigation, chat composition, approval dialogs, settings and model-management screens.
- Tauri/Rust backend: IPC commands, application state, validation, model lifecycle, persistence, tool mediation, and logging.
- Native inference: llama.cpp compiled for the desktop target and called through a narrow, documented FFI boundary.
- Storage: SQLite for conversations and metadata; local configuration for preferences.
- Models: user-managed local files, never bundled into Git.
- Security boundary: explicit approval UI plus backend-side validation and narrowly scoped operating-system adapters.

Tauri capabilities are an additional boundary, not a replacement for per-operation approval.

## 7. Proposed development sequence

1. **0.0.1 — Foundation:** Tauri shell, frontend scaffold, Rust command boundary, project conventions.
2. **0.0.2 — UI parity:** desktop chat layout, navigation, Sync Core, composer and message rendering.
3. **0.0.3 — Preferences:** themes, appearance settings and persistence.
4. **0.0.4 — Model management:** GGUF discovery, metadata, load/unload and resource checks.
5. **0.0.5 — Inference:** llama.cpp desktop build and streamed generation.
6. **0.0.6 — Tool framework:** narrow desktop tools and structured request/response types.
7. **0.0.7 — Approval enforcement:** one-time approval flow, backend validation, audit trail and denial paths.
8. **1.0.0 — Release:** packaging, installer, documentation, tests and security review.

Sequence may change as implementation evidence comes in.

## 8. Security model — non-negotiable

- No generic `execute_shell`, arbitrary command execution, or unrestricted script runner exposed to the model.
- No model-controlled path expansion or implicit access to arbitrary directories.
- Each tool request must identify the operation, target, parameters, and expected effect.
- Show the exact operation and scope to the user before execution.
- Require a fresh approval for every operation, including read-only operations.
- Approval must be bound to that single request and expire immediately after use or cancellation.
- Revalidate paths, arguments, target state, and allowed operation in Rust after approval.
- Reject malformed, stale, replayed, or modified approval requests.
- Denial and cancellation must be safe defaults.
- Log decisions without unnecessarily logging private file contents or secrets.
- Never add a “remember my choice” permission shortcut unless the owner explicitly revises this requirement.

## 9. Working conventions

- Inspect before editing.
- Prefer small, reviewable changes.
- Do not overwrite user work or unrelated files.
- Keep Android and desktop repositories isolated.
- Update this tracker in the same change as meaningful code edits.
- Record files changed, behavior added, tests actually run, outcomes, known limitations, and the next exact step.
- Distinguish **planned**, **implemented**, **tested**, and **verified**.
- Do not invent test results, commit hashes, file contents, or repository state.
- Keep documentation synchronized with implementation.

## 10. Change log

### 2026-11-12 — Repository initialization

- Created initial project documentation and baseline ignore rules.
- Established the desktop repository as separate from the Android project.
- Added directory placeholders for the planned frontend, Rust backend, native inference, docs and tests.
- No application runtime or feature code added.
- Tests: not applicable; no application code exists yet.

## 11. Exact next step

Continue the source-to-desktop port specification before generating the Tauri application. Inspect the Android reference components in detail, beginning with:
1. MainActivity chat hierarchy, layout dimensions, message/composer behavior.
2. SideDashboard navigation entries and settings routes.
3. AppTheme and AppearanceActivity theme tokens and persistence.
4. SyncCoreView animation and state transitions.
5. ChatStore and model-manager behavior.

Then write a component-by-component desktop UI specification. Confirm the proposed React + TypeScript frontend before scaffolding dependencies.
