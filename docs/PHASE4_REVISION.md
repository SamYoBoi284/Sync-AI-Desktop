# Phase 4 — all three stages in source

## Model library
New imports use the Windows native file picker, validate the GGUF header, hash the source with SHA-256 and register its canonical original path. Model bytes are not copied or moved. Duplicate re-import can relink the existing record. Missing files are marked unavailable, and Rust checks availability before load or default selection. Default and active models are separate; loading a replacement unloads the active worker first. Remove from Sync deletes only the registry record, never the source file. Older managed-copy rows remain readable and are removed non-destructively.

## Isolated runtime and hard RAM ceiling
llama.cpp runs in a dedicated child worker. On Windows, the supervisor places the worker in a Job Object with an optional JOB_OBJECT_LIMIT_PROCESS_MEMORY ceiling. The Tauri/WebView process remains outside that job. Model mmap is disabled so GGUF weights use worker-private committed memory. Runtime diagnostics show the configured limit, PID, resident working set, private committed bytes and latest worker error. The ceiling includes model and session allocations, so a model can fail to load if it exceeds the limit. The limiter is off by default and changes apply on the next load. Model estimates are informational only.

## Exactly three Phase 4 stages
1. **Model management + RAM limiter.**
2. **Native runtime + model lifecycle.**
3. **Chat inference connection:** SQLite chat persistence, role-labelled context, streamed output, cancellation, sequential queueing and diagnostics attached to the exact assistant message. Each assistant reply is inserted immediately after its triggering user turn to preserve queued conversation order.

Initial prompts use generic role-labelled text from the latest 24 messages, with a 512-token output cap and 2,048-token context. Model-specific GGUF chat templates and configurable sampling are deferred. Phases 5–7 remain Workspace/artifacts, secure tools/Device Assist, then voice/optimization/packaging/testing/release. No micro-stages, auto-load, Windows sign-in startup, multimodal registry or DAM category were added.

## Validation
All three stages have source changes. No frontend build, Cargo check/test, Windows Job Object test, native llama.cpp compile/link, real model load or end-to-end chat test has run for this revision.