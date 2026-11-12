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

**Phase:** 0 — Repository foundation and source-to-desktop specification  
**Status:** Repository skeleton, architecture notes and detailed Android-to-desktop port specification are committed. No desktop app scaffold or feature runtime exists.

### Verified in this repository

- Public repository exists; default branch is main.
- Initial project structure and planning docs are present.
- Source-to-desktop specification is in docs/ANDROID_TO_DESKTOP_SPEC.md.

### Not yet implemented

- Tauri application scaffold
- React/TypeScript frontend
- Rust backend commands
- SQLite schema or migrations
- llama.cpp Windows build/integration
- GGUF model manager
- Chat persistence
- Permission/approval UI or enforcement
- Windows tools or system adapters
- Automated tests, CI, installer or packaging

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
| Backend | Rust | Agreed |
| Frontend | React + TypeScript | **Approved by user on 2026-11-12** |
| Frontend build tool | Vite | Selected for planned scaffold |
| Package manager | npm | Planned initial default |
| Inference | llama.cpp + GGUF | Agreed direction |
| Persistence | SQLite | Proposed; schema not designed yet |
| Config | Local structured settings | Implementation choice open |
| Platform | Windows x64 first | Agreed |

React + TypeScript is compatible with Tauri 2's web frontend approach. Vite is the selected frontend build tool for the planned app scaffold. No dependencies have been installed and no scaffold has been generated yet.

Windows dev prerequisites to check on the actual build machine: Rust stable MSVC, Microsoft C++ Build Tools (Desktop development with C++), WebView2, and Node.js LTS/npm. WebView2 is included on Windows 10 version 1803 and later in standard installations, but the local machine still needs verification.

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

## 7. Initial architecture

Frontend: React + TypeScript + Vite presentation and state.  
Desktop shell: Tauri 2.  
Backend: Rust typed commands, app services, state, validation, storage, model lifecycle and tool broker.  
Native inference: llama.cpp built for Windows behind a narrow FFI.  
Storage: SQLite planned for conversations, messages, preferences, model metadata and Workspace artifacts.  
Local models: user-managed GGUF files, excluded from Git.  
Security boundary: explicit one-time approval UI plus backend validation and narrow Windows adapters.

Tauri capabilities are an additional boundary, not a replacement for per-operation approval or safe Rust implementation.

## 8. Security model — non-negotiable

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

## 9. Proposed repository layout

Current initialized skeleton is smaller; the following is a target layout, not an assertion that these directories/files exist yet:

~~~text
Sync-AI-Desktop/
├── docs/
│   ├── ANDROID_TO_DESKTOP_SPEC.md
│   ├── ARCHITECTURE.md
│   └── DEVELOPMENT_LOG.md
├── src/
│   ├── app/
│   ├── components/
│   ├── pages/
│   ├── hooks/
│   ├── services/
│   ├── state/
│   ├── themes/
│   └── types/
├── src-tauri/
│   ├── capabilities/
│   ├── migrations/
│   └── src/
├── native/
│   └── llama/
└── tests/
    ├── frontend/
    ├── rust/
    └── security/
~~~

## 10. Development sequence

1. Phase 0 — Repository foundation and source-to-desktop spec. Planning documents committed; no runtime code.
2. Phase 1 — Tauri 2 + React + TypeScript + Vite scaffold; minimal Rust command boundary; Windows dev/build verification.
3. Phase 2 — Desktop shell, sidebar, chat UI, Sync Core, themes and settings.
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
- Update CONTEXT_TRACKER.md in the same commit as every meaningful code/architecture change.
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
- User explicitly approved React + TypeScript; Vite selected for the planned scaffold.
- Updated README to reflect the approved direction.
- Tests/builds: not run; desktop application code still does not exist.

## 13. Exact next step

Phase 1: verify the Windows development environment and create the Tauri 2 + React + TypeScript + Vite scaffold. Start with minimal capabilities and a placeholder typed Rust command boundary. Do not add shell or broad filesystem plugins. Then run both the frontend production build and Tauri development launch, recording the actual results.
