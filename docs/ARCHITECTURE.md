# Initial Architecture

This document records the intended boundaries for Sync AI Desktop. It is a planning document, not a description of already implemented code.

## High-level flow

```text
React UI
  │ Tauri IPC
  ▼
Rust application layer
  ├── Chat and conversation services
  ├── Model lifecycle and inference orchestration
  ├── Storage (SQLite + local config)
  ├── Permission and approval broker
  └── Narrow Windows system adapters
          │
          ├── llama.cpp native library
          └── Explicitly approved OS operations
```

## Frontend

The frontend owns presentation and user interaction:
- Chat history, message bubbles and composer
- Desktop navigation and settings
- Sync Core visualization and state display
- Model manager and resource information
- Explicit operation approval and denial dialogs

The Android application is the design and behavior reference. Its Java UI is not directly reusable as a desktop frontend.

React + TypeScript is the proposed frontend, pending explicit confirmation before scaffolding.

## Rust backend

Rust owns application state and all privileged decisions:
- Typed Tauri commands
- Input validation and request normalization
- Conversation and preference persistence
- Model lifecycle and inference coordination
- Tool request mediation
- Permission checks and audit events
- Narrow operating-system adapters

Frontend checks improve usability but are not a security boundary. The backend must independently validate every request.

## Native inference

Use llama.cpp for GGUF inference. Rust coordinates model loading, generation, cancellation and streaming through a stable native interface. Keep platform-specific compiler settings and native dependencies isolated under the native integration.

The Android JNI bridge is a reference for behavior, not a desktop integration to reuse unchanged.

## Persistence

Planned:
- SQLite for conversations, messages and structured application metadata.
- Local config for user preferences and model registry metadata where appropriate.
- Model files remain in user-selected local storage and are excluded from Git.

Database schema and migration strategy must be designed before persistence is implemented.

## Tool permission boundary

Every tool action follows this conceptual sequence:

1. The model emits a structured tool proposal.
2. Rust parses and validates the proposal against a fixed tool registry.
3. The UI displays the exact action, target and scope.
4. The user explicitly approves or denies that single action.
5. Rust revalidates the approved request and executes only the registered operation.
6. Approval expires; the result is returned to the conversation.

No generic shell execution or unrestricted model-controlled OS access. No persistent blanket permissions. A tool must not be able to expand its own scope after approval.

## Windows-first strategy

Start with Windows x64. Keep platform-specific operations behind Rust interfaces so other operating systems can be added later without allowing the model to choose arbitrary native commands.

## Open architecture questions

- Confirm React + TypeScript and Vite before app scaffolding.
- Decide the exact llama.cpp integration mechanism after a minimal Windows build spike.
- Define the first version's narrow tool catalog and safe path model.
- Define database schema and migration approach.
- Decide installer and update strategy after the app shell works.
