# Android-to-Desktop Port Specification

> Status: Android source audit and desktop port specification. It records reference behavior and target design; implementation status for the desktop repository is tracked in CONTEXT_TRACKER.md.
>
> Android reference: SamYoBoi284/Sync-AI, branch feat/resource-manager-task-manager-17-10-2026.
> Desktop target: SamYoBoi284/Sync-AI-Desktop, main.

## 1. Audit conclusion

Sync AI Android is not just a chat window. Its current navigation exposes chat history, a recorder, voice mode, Workspace, Tasks & Notes, Appearance, settings, File Library, local models, runtime information, personalization, assistant style, command history, health/diagnostics, and default-app controls.

The Android screens are built programmatically with Java Activities and Views. There are no reusable XML layouts or Compose screens to convert. The desktop UI must therefore be a deliberate reconstruction of the existing visual language and workflows, adapted to mouse, keyboard, resizable windows, and wider layouts.

The audit supports preserving the product identity and major workflows, but not copying the Android implementation literally. Android lifecycle APIs, SharedPreferences, content URIs, Android permission prompts, SpeechRecognizer, TextToSpeech, JNI, camera intents, and foreground services all require desktop-specific replacements.

## 2. Source files inspected

The detailed pass inspected these Android files on the reference branch:

| Android source | Verified responsibility |
|---|---|
| MainActivity.java | Main chat layout, submission, generation lifecycle, streaming, queued messages, model controls, chat history, diagnostics and navigation wiring |
| SideDashboard.java | Main navigation drawer and settings destinations |
| AppTheme.java | Six theme palettes, light/dark variants, contrast helpers and system-bar styling |
| AppearanceActivity.java | Theme, mode, accent, motion controls and live preview |
| SyncCoreView.java | Core visualizer states, ring motion, pulse and reduced-motion handling |
| AppPreferences.java | Theme, accent, memory, active chat, voice, personality, speech and update preferences |
| ChatStore.java | Conversation persistence, title generation, CRUD and versioned JSON backup import/export |
| ChatRecord.java | Conversation identity, title, timestamps and ordered messages |
| ChatMessage.java | Message role, body, timestamp and optional diagnostics |
| ModelManager.java | Model import, SHA-256 duplicate detection, registry, selected model and removal |
| ModelInfo.java | Model metadata and size summary |
| GgufModelBackend.java | Serialized load, generate, cancel, unload and diagnostics orchestration |
| GgufNative.java | Java/JNI inference boundary |
| syncai_native.cpp | llama.cpp model/context/tokenization/sampling/generation implementation and stop handling |
| CMakeLists.txt | Android native build and pinned llama.cpp source reference |
| SyncRuntime.java | Shared runtime composition and service access |
| ToolEngine.java | Deterministic natural-language routing and Android-side actions |
| ResourceManagerActivity.java | Process CPU, memory, storage and component-state dashboard |
| WorkspaceActivity.java | Workspace artifact UI |
| WorkspaceStore.java | Workspace artifact persistence and fenced-block extraction/validation |

The wider source tree also includes File Library, tasks/notes, voice, recorder, camera, and other modules. They are part of the product inventory, but this pass did not complete a full line-by-line implementation audit of every secondary screen.

## 3. Confirmed technology direction

| Area | Decision | Status |
|---|---|---|
| Desktop shell | Tauri 2 | Agreed |
| Backend | Rust | Agreed |
| Frontend | React + TypeScript | Approved |
| Frontend build tool | Vite | Selected for the planned scaffold |
| Package manager | npm initially | Planned default |
| Inference | llama.cpp + GGUF | Agreed direction |
| Persistence | SQLite | Proposed; schema still to be designed |
| Initial platform | Windows x64 | Agreed |

React + TypeScript is compatible with Tauri 2's web-frontend model. Tauri uses the operating system's WebView rather than requiring an Electron-style bundled Chromium runtime. On Windows, the web frontend is rendered through Microsoft Edge WebView2.

Windows development prerequisites include the Rust MSVC toolchain, Microsoft C++ Build Tools with Desktop development with C++, WebView2, and Node.js when using a JavaScript frontend. Windows 10 version 1803 and later generally include WebView2; the actual development machine still needs its toolchain verified before the first build.

Official references:
- https://v2.tauri.app/start/prerequisites/
- https://tauri.app/security/capabilities/
- https://tauri.app/security/permissions/
- https://tauri.app/security/

## 4. Desktop shell and layout

### 4.1 Main window structure

Use a desktop-first shell with persistent left navigation and a central work surface. Do not retain the Android slide-over drawer as the primary navigation pattern.

Recommended structure:

- Application sidebar: persistent navigation, with a compact/collapsed option later.
- Main header: Sync identity, Sync Core status, current model status and screen-specific actions.
- Main content: chat, Workspace, model manager, appearance, or another selected destination.
- Chat composer: anchored at the bottom of the chat surface, with message history taking the flexible space above it.

This adapts the Android drawer and its two internal pages; it is not a pixel-for-pixel conversion of the 320dp mobile panel. Desktop sizing should be responsive and keyboard-accessible. Exact dimensions should be selected during implementation rather than inferred from Android density units.

### 4.2 Navigation inventory

The Android main drawer contains:

| Section | Android entry | Desktop destination |
|---|---|---|
| Chats | New chat | Create a new conversation |
| Chats | Chat history | Conversation list/history |
| Chats | Recorder | Deferred until a desktop recording implementation exists |
| Workspace | Voice mode | Deferred until the desktop voice pipeline exists |
| Workspace | Workspace | Workspace artifact editor/list |
| Workspace | Tasks & notes | Local task and note manager |
| Workspace | Appearance | Appearance settings |
| Settings | Settings | Settings index or grouped settings route |

The Android Settings page exposes:
- Voice output
- Appearance / themes
- Default apps
- Personality & voice
- Current model status
- File Library
- Local models
- Import model
- Runtime information
- Accent colors and memory
- Assistant settings
- About and diagnostics

These should become navigable desktop settings pages or panels, not a long nested mobile-style modal. Items that depend on unimplemented desktop integrations should be clearly marked unavailable or withheld from the first release. Do not display controls that do nothing.

### 4.3 Main chat screen

Preserve these visual and behavioral elements:

- Sync AI identity in the header.
- Sync Core beside the application identity.
- Scrollable conversation history.
- Distinct user and assistant message presentation.
- Optional diagnostics attached to assistant responses.
- Multiline composer.
- Clear Send and Stop-generation states.
- Loading/working indicator while a model is loaded or a response is generated.
- Model status accessible without dominating the chat canvas.

The Android composer currently has a camera button, text field, Voice button, and Send/Stop button. The desktop MVP should implement the text field and Send/Stop behavior first. Camera and voice controls should only appear when their desktop implementations are actually available.

### 4.4 Conversation behavior to preserve

The Android source establishes these behaviors:

1. Submitting a non-empty message immediately stores and displays the user message.
2. A new chat starts with a generated ID and the default title “New chat”.
3. The first submitted user message supplies a short title, capped at 34 characters in Android.
4. If generation is already active, later messages are queued rather than submitted concurrently.
5. The native inference backend serializes generation work.
6. The Send button becomes Stop when generation is active and the composer is empty.
7. If text is present, Send remains the primary action; Stop does not replace it.
8. Tokens stream into the active assistant response.
9. A generation can be cancelled, completed, or failed.
10. Chat history supports opening, renaming, exporting and deleting conversations.
11. Backup import validates format/version, stages data, and skips chat IDs that already exist rather than overwriting them.

The desktop state model should represent these states explicitly. Do not recreate Android behavior through one enormous UI component.

### 4.5 Proposed frontend component boundaries

The approved React + TypeScript frontend should use small components with single responsibilities:

- AppShell
- AppSidebar and SidebarSection
- AppHeader
- SyncCore
- ChatPage
- ConversationView
- MessageItem
- MessageDiagnostics
- MessageComposer
- ModelStatus
- ChatHistoryPage or ChatHistoryPanel
- ModelManagerPage
- AppearancePage
- SettingsLayout
- ApprovalRequestDialog

These are proposed component boundaries, not files created yet. Components consume typed application state and invoke narrow backend commands. No component calls native operating-system APIs directly.

## 5. Sync Core behavior

The Android visualizer defines eight states:

| State | Meaning |
|---|---|
| IDLE | No active model or operation |
| READY | Available and waiting |
| LISTENING | Voice input is active |
| THINKING | Local model generation is active |
| SPEAKING | Speech output is active |
| EXECUTING | A validated operation is being processed |
| SUCCESS | An operation completed successfully |
| ERROR | An operation failed or needs attention |

Behavior verified in SyncCoreView.java:

- Orbit rings rotate only while the state is THINKING.
- The center core pulses gently in other active states.
- Reduced-motion mode stops continuous motion.
- SUCCESS and ERROR are transient states and return to READY after approximately 1.8 seconds.
- LISTENING and SPEAKING use the theme's secondary color.
- ERROR uses a red/pink signal color.
- SUCCESS uses a green signal color.
- THINKING blends the current primary and secondary colors.

Desktop implementation should keep the same state meanings and color logic. CSS/SVG animation is a suitable starting point for the small vector visualizer; it should not need a heavyweight graphics engine. The visualizer must expose an accessible text label for its current state, and reduced-motion preferences must disable continuous animation.

## 6. Appearance and preferences

### 6.1 Theme identities

Preserve all six identity names and meanings:

| Theme | Android description |
|---|---|
| Obsidian | Premium dark sci-fi |
| Nullverox | Cyberpunk signal and glitchcore |
| Minimal | Clean graphite and focus |
| Holographic | Glass-like translucent layers |
| Command | Technical system HUD |
| Aurora | Ambient violet and teal |

Each identity has a light and dark palette. The palette includes background, surface, secondary surface, button, primary text, body text, muted text, border, user bubble, assistant bubble, primary accent and secondary accent.

The desktop should define semantic theme tokens rather than hardcoding colors in individual components. Suggested tokens: background, surface, surface-elevated, text-primary, text-body, text-muted, border, bubble-user, bubble-assistant, accent-primary and accent-secondary.

The Android source includes contrast helpers for foreground-on-accent text and a light-mode adjustment to keep the accent legible. Preserve that behavior rather than selecting white or dark text by guesswork.

### 6.2 Independent controls

The Android app keeps theme identity, theme mode and accent color independent:

- Theme mode: System, Dark, Light.
- Accent color: Purple, Blue, Cyan, Green, Lime, Orange, Red, Pink.
- Motion intensity: Minimal, Balanced, Immersive.

Do not collapse these into a single preset. Theme selection must not reset the accent, and changing the accent must not silently change the theme.

### 6.3 Other preference concepts

The source also stores optional local memory, active conversation ID, voice output, wake-word state, assistant personality (Balanced, Concise, Warm, Professional, Playful), speech rate and pitch, speech voice, speech-recognition backend and language, text-to-speech language, and update preferences.

Desktop should persist only settings that have a meaningful desktop implementation. Voice and wake-word preferences should not be surfaced until their corresponding Windows features exist. A stored preference must never imply that an operating-system permission has been granted.

## 7. Conversation persistence and desktop-native backups

### 7.1 Desktop-owned data model

The desktop application owns its own chat schema and backup contract. A conversation contains an ID, title, creation/update timestamps and ordered messages. A message contains an ID, chat ID, role, body, timestamp, ordering metadata and optional diagnostics. Each assistant message owns its own diagnostics record, which must survive app restarts and backup restore.

Android sync-ai-chat-backup v1 and raw chats.json compatibility are not requirements. No Android import adapter will be built for this separate application.

### 7.2 Desktop storage direction

SQLite remains the planned persistence layer; the schema and migration strategy are not implemented. A normalized design should include at least:

- chats: id, title, created_at, updated_at.
- messages: id, chat_id, role, body, created_at, position, diagnostics_json (nullable; assistant messages only).
- app_settings: key and serialized value.
- schema_migrations: applied migration version and timestamp.

Model metadata and Workspace artifacts will be added in their later phases, not as unused Phase 3 tables.

Use foreign keys and transactions for chat/message deletion, backup import and metadata changes. Decide on backup consistency, database migration/versioning and crash-safe writes before implementing the store.

### 7.3 Desktop backup requirements

Use the versioned sync-ai-desktop-backup JSON format with an explicit formatVersion, export timestamp, application metadata and chat records. Export/import preserves diagnostics on each assistant message. Import validates the complete file before committing, enforces size/count limits, rejects malformed records and unsupported future versions, and skips conflicting chat IDs rather than overwriting local conversations. A failed import rolls back completely. The source backup is never modified.

The exact schema and backup contract are defined in docs/PHASE3_DATA_CONTRACT.md.

## 8. Model management and inference

### 8.1 Verified Android behavior

The Android ModelManager:
- Copies an imported file into app-managed storage.
- Computes SHA-256 while copying.
- Deduplicates models by hash.
- Stores ID, original name, file path, format, size, hash and import timestamp.
- Remembers a selected model ID.
- Removes the model file when the user deletes the model.

The backend accepts GGUF for actual load. Although ModelManager recognizes other extensions as metadata formats, GgufModelBackend rejects non-GGUF formats. Desktop MVP should therefore present GGUF as the supported loadable format; it must not imply that safetensors, ONNX, TFLite, GGML or BIN can be loaded by the current runtime.

The Android backend serializes model loading, generation and unload work through a single-thread executor. It streams token callbacks, exposes cancellation and keeps diagnostics. Preserve these lifecycle semantics in the desktop service.

### 8.2 Desktop model manager

The desktop manager should:
- Import a GGUF using a user-selected file.
- Hash the file and detect duplicates.
- Keep the selected model path and metadata in the local registry.
- Validate file existence and readable access before load.
- Show format, file size, hash, loaded/selected state and load errors.
- Support explicit load, unload and delete actions.
- Avoid moving or deleting the user's source file unless a separate, clearly described action is approved.
- Keep model files outside the Git repository and outside application source directories.

Do not assume a model that loads on Android will be performant on every Windows PC. Model size, context length, quantization, RAM, CPU instruction support and optional GPU backends affect feasibility.

### 8.3 Native boundary

The Android native module links llama.cpp at pinned revision 4a89937 and uses JNI plus Android-specific logging and process APIs. It has a global model pointer, a mutex, a stop flag, prompt/token conversion, chat-template handling, context initialization, sampling and token streaming. The Java bridge passes message roles/text and generation settings (max tokens, temperature and top-p).

Do not copy the Android JNI boundary or its Android-specific CMake configuration. The desktop integration should instead have:

- A Windows native build of llama.cpp isolated under the native integration.
- A narrow C ABI or carefully designed Rust FFI boundary with opaque model/session handles.
- Explicit load, unload, generate, cancel and diagnostics operations.
- A token callback/stream channel that cannot call back into destroyed UI state.
- Clear ownership and lifetime rules for native model, context, sampler and callback data.
- Serialized model lifecycle operations, with cancellation safe under concurrent UI requests.
- Error translation into typed Rust errors rather than panics crossing the FFI boundary.

A small proof of concept must validate MSVC/CMake integration, model load, one streamed generation, cancellation, unload and repeat load before the full chat flow is built. The exact native integration mechanism remains an implementation spike, not a settled dependency.

## 9. Tool catalog and platform mapping

Android ToolEngine mixes deterministic intent recognition with Android-specific side effects. The desktop must not transplant its action execution model.

### 9.1 Suitable early tools

Potential first tools, once individually implemented and approved:
- Calculator: deterministic local calculation.
- Date/time: read local clock and timezone.
- System overview: narrowly defined hardware/OS summary.
- Resource overview: process/app resource metrics.
- Selected-file read: read only a user-selected file under the exact confirmed scope.
- Selected-file write/export: write only to an explicitly selected destination and show the exact path and overwrite effect.

Even read-only AI-invoked tool operations require fresh approval under the product requirement. User-initiated app UI workflows such as opening a settings screen are not a way to grant the model broader authority.

### 9.2 Windows-specific or deferred tools

These Android categories need a desktop design and should not be assumed to have direct equivalents:
- Android app launch and default-app selection.
- Contacts lookup and phone calls.
- Android alarms and reminders.
- Camera inventory and camera intents.
- Bluetooth, Wi-Fi, flashlight, brightness and volume controls.
- Android foreground wake-word service.
- Android media transport intents.
- Android SpeechRecognizer and TextToSpeech.
- Android-specific recording and voice session services.

Windows support should be evaluated one capability at a time. Do not add broad registry editing, arbitrary process launch, generic PowerShell/CMD, or unrestricted automation as shortcuts.

### 9.3 Model proposals are untrusted

The model may propose a registered tool by structured name and typed arguments. It cannot select a native function, issue a shell command, grant itself permission, choose an arbitrary executable, or expand a previously approved scope.

Unknown tool names, malformed arguments, oversized input, unsupported paths and unsupported operations must fail closed.

## 10. One-time approval and Windows security architecture

This is a hard product requirement, not a future polish item.

### 10.1 Request lifecycle

Every AI-invoked host operation follows this sequence:

1. The model proposes a structured operation.
2. Rust validates the tool ID and argument schema against a fixed registry.
3. Rust resolves the exact target and constructs a human-readable action summary, including the target path/resource and expected effect.
4. The UI presents that exact request with Approve and Deny controls.
5. Approval is bound to the exact request payload and a unique, short-lived request ID.
6. Rust checks that the approval is valid, unused, unexpired, and bound to the same canonicalized operation and target.
7. Rust revalidates the target and executes only that registered operation.
8. The approval is consumed regardless of success or failure.
9. The result is returned to the conversation and a privacy-conscious audit event is recorded.

Denial, cancellation, malformed requests, stale approvals, replay attempts and changed arguments must never trigger execution.

### 10.2 Required implementation properties

- Fresh approval for every tool action, including reads.
- No persistent blanket permission and no “remember my choice”.
- No generic shell/terminal execution.
- No arbitrary script execution.
- No model-controlled executable path.
- No unrestricted filesystem traversal.
- No path or target expansion after approval.
- Canonicalize Windows paths; account for case-insensitivity, junctions, symlinks/reparse points, alternate data streams and path traversal.
- Avoid time-of-check/time-of-use gaps: open or resolve the approved target safely and revalidate immediately before the operation.
- Bound file size, operation duration and payload size.
- Use allowlisted operations and typed parameters.
- Keep Tauri capabilities minimal. Capabilities reduce which frontend calls are exposed; they do not make unsafe Rust command implementations safe.
- Log operation ID, tool ID, decision, timestamp, outcome and safe target summary. Do not log private file contents, model prompt contents or secrets unnecessarily.

A generic command like read_file(path) or write_file(path, content) is too broad unless the backend enforces an exact user-selected and approved scope for that request. Prefer operation-specific commands with narrow input structures.

### 10.3 Separation of responsibilities

- React: renders proposals and captures the user's approval or denial.
- Tauri IPC: exposes only the minimum backend commands required by the bundled UI.
- Rust: owns policy, validation, approval state, path handling, execution, expiry and audit events.
- Native/Windows adapter: performs only the single narrow operation after Rust authorization.

The frontend must not be the authority that decides whether an operation is allowed. The model must never receive a tool capability token that allows it to execute directly.

## 11. Resource monitoring and diagnostics

Android's ResourceManagerActivity is intentionally honest about measurement boundaries:
- CPU is process-wide and expressed as a percentage of one CPU core; multi-threaded work can exceed 100%.
- Memory is process PSS, with Java/native heap details, rather than invented per-library totals.
- Component CPU values are rough sampled estimates assigned by active-window priority.
- The UI explains that Android does not expose dependable per-library memory splits.

Desktop should preserve that honesty. Show application process CPU/memory and available host metrics only when the platform API provides them. Label estimates as estimates. Do not claim exact llama.cpp memory or CPU use without instrumentation that genuinely measures it.

Diagnostics should be exportable by explicit user action and should omit private prompts, message bodies, file contents, secrets and model data by default.

## 12. Deferred features and release scope

Recommended first useful desktop slice:
1. Tauri shell and React/TypeScript app.
2. Desktop navigation and the main chat layout.
3. Sync Core visualizer and appearance tokens.
4. Chat persistence, per-message diagnostics and desktop-native versioned backup import/export.
5. GGUF model registry and lifecycle.
6. Native llama.cpp proof of concept, then streamed inference.

After the core slice:
- Workspace and artifact persistence.
- File Library with explicit user-selected import and scoped access.
- Tasks & Notes.
- Resource Manager and diagnostics.
- Tool proposal/approval framework and a deliberately small tool catalog.
- Voice input/output and recording only after Windows-specific design and testing.

The tool approval foundation must be designed before any computer-control feature is implemented. It need not block the visual chat shell or local model proof of concept.

## 13. Proposed desktop repository layout

This is a target layout for the approved React + TypeScript + Vite + Tauri 2 direction. It is a proposal until the application scaffold is created.

~~~text
Sync-AI-Desktop/
├── docs/
│   ├── ANDROID_TO_DESKTOP_SPEC.md
│   ├── ARCHITECTURE.md
│   └── DEVELOPMENT_LOG.md
├── src/
│   ├── app/
│   ├── components/
│   │   ├── chat/
│   │   ├── core/
│   │   ├── layout/
│   │   └── dialogs/
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
│       ├── commands/
│       ├── inference/
│       ├── models/
│       ├── permissions/
│       ├── platform/
│       │   └── windows/
│       ├── storage/
│       ├── tools/
│       └── main.rs
├── native/
│   └── llama/
├── tests/
│   ├── frontend/
│   ├── rust/
│   └── security/
├── package.json
├── tsconfig.json
└── vite.config.ts
~~~

Avoid creating every proposed folder as empty scaffolding. Add directories as actual implementation boundaries emerge.

## 14. Implementation sequence and acceptance checks

### Phase 0 — Completed planning work

- Separate repository initialized.
- Android source reference audited for the core chat/theme/model/tool architecture.
- React + TypeScript approved.
- Component and behavior specification recorded.
- No desktop application code created.

### Phase 1 — App shell

- Create Tauri 2 + React + TypeScript + Vite scaffold.
- Ensure app starts in development on Windows.
- Ensure a production build succeeds.
- Define strict TypeScript settings and lint/test conventions.
- Keep capabilities minimal and do not add shell or broad filesystem plugins.
- Add a placeholder native command boundary with no privileged actions.

### Phase 2 — UI parity

- Implement desktop shell, sidebar, header, chat list, message rendering, composer and Sync Core.
- Implement the six theme identities, three appearance modes, eight accents and motion preference.
- Verify keyboard navigation, focus order, responsive resizing and reduced-motion behavior.

### Phase 3 — Local data

- Design and migrate SQLite schema.
- Implement chat CRUD, transactional message writes and settings persistence.
- Persist and display diagnostics attached to each individual assistant message.
- Implement History search, rename, delete and reopen flows.
- Implement versioned desktop-native chat backup export/import, preserving diagnostics.
- Test malformed backups, unsupported versions, size/count limits, duplicate IDs and transaction rollback.
- Android backup compatibility is explicitly out of scope.

### Phase 4 — Model manager and inference spike

- Implement GGUF import/hash/duplicate checks and local metadata.
- Build native llama.cpp on Windows with a narrow FFI.
- Verify load, streamed tokens, stop, unload, diagnostics and repeated lifecycle.
- Test with a small known model before larger models.

### Phase 5 — Workspace and desktop services

- Port Workspace semantics and artifact continuity.
- Add File Library only with explicit user-selected scopes.
- Add Tasks & Notes and accurate process/resource diagnostics.

### Phase 6 — Tool approvals

- Implement typed proposal registry.
- Implement one-time approval, exact request binding, expiry and replay protection.
- Implement Windows path validation and a small safe operation set.
- Add allow/deny/expiry/replay/path traversal/symlink/TOCTOU tests before expanding the catalog.

### Phase 7 — Voice, recording and release

- Design Windows voice input/output and recording separately from Android services.
- Add visible microphone/recording status and explicit start/stop behavior.
- Package, sign or document unsigned builds, test on a clean Windows environment, and complete a security review.

## 15. Open decisions before or during implementation

- Exact SQLite schema and migration library.
- Exact llama.cpp CMake/MSVC and Rust FFI strategy.
- Minimum supported Windows version and CPU instruction baseline.
- Whether the first File Library is app-managed imports only or supports scoped folders.
- Backup export evolution if desktop-only fields are added.
- Windows voice recognition and TTS engines.
- Packaging target and update mechanism.
- License selection and third-party notices.

## 16. Validation status

This document is based on source inspection and official Tauri documentation. It is a specification, not a build result.

- Android reference files inspected: see Section 2.
- Tauri 2 + React/TypeScript compatibility: checked against Tauri's current frontend prerequisites model.
- Desktop application build: not run; no scaffold exists.
- Rust tests: not run; no Rust application code exists.
- Frontend tests: not run; no frontend application code exists.
- Native Windows inference: not built or tested.
- Security approval flow: designed, not implemented or audited.

## Desktop-specific Phase 4 correction

The Android source behavior documented above remains a historical/source-audit record. The desktop implementation decision is now different by design:

- New desktop GGUF imports store the canonical original file path and do not copy the model. Existing desktop managed-copy records remain readable for compatibility.
- Desktop library removal deletes only the Sync registry record; it does not delete the underlying GGUF, including for legacy managed-copy entries.
- Desktop inference runs in a dedicated child worker. Windows Job Objects enforce an optional private committed-memory ceiling on that worker, while the Tauri/WebView process stays outside the job.
- Desktop Runtime diagnostics expose the worker's ceiling, working-set memory, private commit, PID and latest worker failure.
- The hard ceiling includes model and session allocations. A limit below model load requirements can prevent loading. This is not the previous generation-only allocation-budget design.
- This is the current desktop-specific decision and supersedes any earlier desktop model-copy or application-level RAM-budget recommendation in this document. Phase 4 remains exactly three stages; chat streaming is Stage 3 and is not yet implemented in this revision.

## Desktop-only Phase 4 revision

New desktop GGUF imports store canonical original paths. Migration v3 preserves older managed-copy records. Registry removal never deletes the model file. Desktop inference runs in a dedicated worker with an optional Windows Job Object committed-memory ceiling, while Tauri/WebView stays outside the job. Runtime diagnostics report cap, PID, working set, private commit and worker errors.

Stage 3 connects SQLite chat history to streamed llama.cpp responses, cancellation, sequential queueing and per-assistant-message diagnostics. Assistant responses are inserted after the exact user turn they answer. The initial prompt is generic role-labelled text, not model-specific chat-template support. Phase 4 stays three stages and is not yet Windows-validated.
