# Amigo Editor Window Event Bus

## Problem

Amigo Editor is moving from one launcher window to a multi-window editor:

- `StartupWindow` selects and previews mods.
- `WorkspaceWindow` owns the main editing session.
- `ThemeWindow` edits global visual settings.
- `SettingsWindow` edits global editor settings.
- future project/session windows will edit mod-specific state.

Without a shared communication layer, each window can update only its own React tree. That caused settings such as theme/font changes to apply in one window while other open windows stayed stale.

## Purpose

The window event bus is the single app-level channel for cross-window state notifications. It is not a replacement for commands or stores:

- commands still perform work and persist state,
- frontend stores still own local UI state,
- the event bus broadcasts that shared app state changed.

## Current Responsibilities

The bus currently carries:

- `theme-settings-changed`: emitted after backend theme settings are persisted.
- `font-settings-changed`: emitted after backend font settings are persisted.
- `workspace-opened`: emitted after a workspace window has been opened or focused.
- `workspace-closed`: emitted before a workspace window closes.
- `window-close-requested`: emitted before a frontend-controlled window close.
- `window-focused`: emitted when an editor window receives browser focus.
- `session-closed`: emitted after the backend session registry closes a session.
- `cache-invalidated`: emitted after cache clear/maintenance operations.
Preview frame progress is no longer a window bus event. It uses `preview-progress` through `src/app/previewProgressBus.ts`, because it is transient task progress rather than a durable app fact.

Frontend entry points:

```txt
crates/apps/amigo-editor/src/app/windowBus.ts
crates/apps/amigo-editor/src/app/windowBusTypes.ts
```

Backend entry points:

```txt
crates/apps/amigo-editor/src-tauri/src/events/bus.rs
crates/apps/amigo-editor/src-tauri/src/events/envelope.rs
crates/apps/amigo-editor/src-tauri/src/events/names.rs
crates/apps/amigo-editor/src-tauri/src/events/payloads.rs
```

UI components and services should use these helpers instead of importing `@tauri-apps/api/event` directly for app/window communication.

## Envelope

Every bus event is delivered inside a small typed envelope:

```txt
eventId
eventType
sourceWindow
sessionId
timestampMs
schemaVersion
payload
```

The envelope makes events visible in the workspace event log and gives future handlers enough metadata to filter by source window, session, or schema version.

## Theme Flow

```txt
ThemeWindow
→ set_theme_settings(themeId)
→ backend persists EditorSettings
→ backend emits theme-settings-changed
→ every open window ThemeService receives the envelope
→ each document updates data-theme
```

Fonts use the same persisted backend settings path:

```txt
ThemeWindow
→ applyFont(fontId)
→ set_font_settings(fontId)
→ backend persists EditorSettings
→ backend emits font-settings-changed
→ every open window ThemeService updates --font-ui-active
```

## Preview Progress Flow

```txt
Workspace or Startup preview request
→ request_scene_preview()
→ backend renderer captures frames
→ backend emits preview-progress
→ editor store updates preview task progress
```

This is not window bus traffic. The Window Event Bus can still record preview started/completed/failed summaries later, but per-frame progress stays outside it.

## Cache Flow

```txt
SettingsWindow
→ clear_preview_cache(projectCacheId)
→ backend deletes files
→ backend emits cache-invalidated
→ open windows mark cache-dependent state stale or reload cache stats
```

The payload carries IDs and a reason. Full cache details still come from cache commands.

## Lifecycle Flow

```txt
StartupWindow
→ openModWorkspace(modId)
→ frontend opens/focuses WorkspaceWindow
→ frontend emits workspace-opened
```

Workspace close currently emits:

```txt
window-close-requested
workspace-closed
session-closed
```

Focus changes emit `window-focused`, which lets the event log and future window registry understand which editor surface is active.

## Window Registry

The editor now keeps a lightweight runtime registry for windows:

```txt
src-tauri/src/windows/registry.rs
```

It tracks:

- window label,
- window kind,
- optional sessionId,
- focused state,
- last seen timestamp.

Frontend windows register themselves through backend commands during route hydration. The registry is intentionally an application-shell subsystem, not an engine service.

## Settings Migration

`editor-settings.json` now has `settingsVersion: 1`.

Legacy settings are migrated on load:

- missing `activeFontId` becomes `source-sans-3`,
- missing/legacy `activeThemeId` becomes a normalized current theme,
- migrated settings are written back to disk.

The first migration test lives in `settings::editor_settings`.

## Future Responsibilities

The same bus should carry these categories as the main editor grows:

- settings changed: cache root, startup behavior, layout behavior, editor preferences,
- session lifecycle: workspace opened, session closed, session focused,
- cache updates: cache cleared, maintenance completed, preview cache refreshed,
- project changes: mod metadata reloaded, diagnostics refreshed, content index updated,
- layout sync: layout reset, layout saved, workspace dock state changed,
- dirty state: file dirty/clean, save completed, close blocked by unsaved changes,
- diagnostics: new project/session-level diagnostics available.

## Design Rules

1. One place owns event names.
   Frontend event names live in `windowBus.ts`; backend event names live in `events::names`.

2. Commands mutate or load state.
   Events announce that state changed. They should not be the only source of truth.

3. Windows do not talk to each other directly.
   They call commands or emit through the bus.

4. Components should not import Tauri event APIs directly for cross-window behavior.
   They should use `windowBus` helpers.

5. Every persistent setting change should round-trip through backend storage before broadcasting.
   Theme already follows this rule.

## Why Not Pipes

Pipes are useful for point-to-point streaming and external process IO. The editor needs app-wide fan-out: one settings change must reach every open Tauri window. Tauri events match that better:

```txt
one emitter → all listeners
```

For future high-volume data, such as live rendered frames, a dedicated stream can be added. Settings, lifecycle, progress, and invalidation events should stay on the window event bus.

## Acceptance Criteria

The system is correct when:

- changing theme in one window updates every open window,
- changing dev font in one window updates every open window,
- preview progress still reaches the active UI,
- preview progress is outside the window bus,
- native/window close lifecycle cleans up sessions,
- settings load migrates legacy files,
- event names are not duplicated across components,
- new cross-window features have a documented bus event instead of ad hoc listeners.
