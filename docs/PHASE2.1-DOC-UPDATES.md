# Documentation updates for Phase 2.1

Apply these small documentation changes when the source bundle is merged into `main`.

## CONTEXT_TRACKER.md

Under **Verified in this repository**, replace:

> `get_app_info is registered as the only custom Rust command.`

with:

> `get_app_info returns static application/OS metadata; consume_startup_splash consumes the one-time startup gate. Both commands are registered.`

Append a Phase 2.1 entry to the change log:

### Phase 2.1 — Desktop shell and navigation

- Replaced the static foundation screen with a desktop shell and interactive sidebar navigation.
- Added New chat and Runtime views, plus honest preview states for History, Workspace, Tasks & Notes, File Library, Local models and Appearance.
- Added Ctrl+N / Cmd+N navigation to the New chat view.
- Kept the composer disabled until chat persistence and inference are implemented; no fake response path is present.
- Added reduced-motion handling for the new Sync Core artwork and focus-visible navigation states.
- Validation: source syntax inspection only. Dependency installation, full TypeScript/Vite build, Cargo check and Windows runtime testing remain outstanding.

Update **Exact next step** to:

> On the personal Windows development machine, run `npm install`, `npm run build`, `cargo check` from `src-tauri`, then `npm run tauri dev`. Verify first-start splash, same-process tray restore, close-to-tray, second-launch focus, Exit, sidebar navigation and Ctrl+N / Cmd+N. Do not use the work PC.

## docs/ARCHITECTURE.md

In the current scaffold section, change the sentence saying there is one read-only command to say there are two narrow commands: `get_app_info` returns static metadata and `consume_startup_splash` consumes a process-scoped one-time startup gate. Neither command exposes host files or process execution.

Add the shell components to the code organization tree:

~~~text
src/
  components/
    layout/Sidebar.tsx
    layout/Topbar.tsx
    startup/LoadingScreen.tsx
  pages/
    FeaturePreviewPage.tsx
    NewChatPage.tsx
    RuntimePage.tsx
  types/navigation.ts
  App.tsx
  main.tsx
  shell.css
~~~

## docs/DEVELOPMENT_LOG.md

Append the Phase 2.1 entry above after the startup lifecycle entry.
