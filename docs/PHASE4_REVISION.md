# Phase 4 revision — runtime memory budget and model-library behavior

This document supersedes earlier Phase 4 wording where it describes the RAM Limiter as a model-load eligibility check.

## RAM Limiter: corrected requirement

The optional RAM Limiter is a ceiling for memory Sync allocates for inference work while generating, not a filter that rejects a model because its GGUF file or weights exceed the slider value.

The intended control surface is the native inference session: context/KV-cache sizing, batch and micro-batch sizing, and other allocations the selected backend exposes. The runtime should choose the largest safe configuration within the configured budget where possible, and explain when even its minimum supported configuration cannot fit.

The setting is not a hard process-wide Windows memory cap. Model weights, memory-mapped pages, allocator overhead, WebView memory, and unrelated process allocations cannot all be reliably controlled through the current high-level llama.cpp binding. The UI and diagnostics must distinguish:
- model file/weight size;
- inference-session allocation estimate or measurement;
- total process memory, when measured separately;
- configured inference allocation budget.

The limiter remains off by default. Enabling or changing it must not silently unload a loaded model. The runtime applies the setting when preparing a session for inference and must report the effective context/batch configuration.

## Model library behavior

- Import through the Windows native file picker.
- Register the selected GGUF's original absolute path; do not copy the multi-GB file into application storage.
- Persist stable model identity using SHA-256 plus metadata.
- Check that the original path still exists before inspection or loading. A moved or missing file is shown as unavailable and is not loaded.
- Keep the default model separate from the currently loaded model.
- Model actions include Load, Set as Default, Unload (when active), and Remove from Sync.
- Remove from Sync deletes only the registry entry. It never deletes or moves the user's GGUF file.
- Legacy entries created by earlier managed-copy imports must remain non-destructive. Their existing managed files must not be deleted by the new Remove from Sync action.

## Scope exclusions

No multimodal capability registry, image-generation support, Device Assist model category, or DAM-specific model architecture is being designed or implemented in this revision. These are exploratory ideas only and require a later product decision after the desktop app has been built and tested by the user.

## Phase 4 remains exactly three stages

1. **Model management + resource budget:** model registry, Windows file picker, original-path tracking, missing-file detection, non-destructive library removal, model selection/default state, hardware information and inference allocation budget.
2. **Native runtime + model lifecycle:** CPU llama.cpp runtime, explicit load/unload, safe lifecycle, effective session sizing under the configured inference budget.
3. **Chat inference connection:** conversation context, streaming, cancellation, sequential queued sends, and diagnostics attached to the exact assistant message.

Do not add numbered micro-stages. Phase 5 remains streaming/workspace expansion beyond the initial chat inference pipeline; Phase 6 remains secure tools and Device Assist; Phase 7 remains voice, diagnostics/optimization, packaging, testing, security review and release.
