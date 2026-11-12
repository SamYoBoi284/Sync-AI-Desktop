# Sync AI Desktop — Phase 2.1 Shell

This bundle contains the Phase 2.1 desktop shell implementation for the existing `main` branch.

## Files

- Replace `src/App.tsx` and `src/main.tsx` with the versions in this bundle.
- Add `src/shell.css`.
- Add `src/types/navigation.ts`.
- Add `src/components/layout/Sidebar.tsx` and `Topbar.tsx`.
- Add `src/pages/NewChatPage.tsx`, `RuntimePage.tsx`, and `FeaturePreviewPage.tsx`.

## Behavior

- Sidebar navigation is interactive and updates the active content view.
- `Ctrl+N` / `Cmd+N` returns to the New chat view.
- Runtime shows only metadata returned by the existing Rust `get_app_info` command.
- History, Workspace, Tasks & Notes, File Library, Local models and Appearance are explicit preview pages, not fake working features.
- The New chat page includes the initial chat composition layout, but the composer is intentionally disabled until message storage and inference exist.
- Existing startup splash and one-process gate wiring are retained.

## Validation status

No npm install, TypeScript build, Cargo check, Tauri launch or Windows interaction test was run for this bundle. It is source work only and must be validated on the user's personal Windows development machine.
