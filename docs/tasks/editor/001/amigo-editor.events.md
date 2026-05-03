# Amigo Editor Events Architecture

This document defines the event-driven interaction model for `amigo-editor`, with a focus on the first screen: the `StartupDialog` / `ProjectLauncher`.

It is a supplement to the startup dialog mockup and the editor architecture document. Its purpose is to make every user action observable, debuggable, and consistent across UI components, backend services, preview generation, validation, and busy indicators.

---

## 1. Core idea

Every meaningful action in `amigo-editor` is represented as an editor event.

The UI should not directly perform domain work such as scanning a mod, validating a scene, generating previews, or saving files. UI components emit semantic events. Centralized systems receive those events, execute the required work, update shared state, report diagnostics, and publish activity status.

Conceptual flow:

```txt
UI component
→ EditorEvent
→ Event Dispatcher
→ Event Handler / Service
→ Task Registry
→ State Store
→ Diagnostics / Activity Log
→ UI update
```

This gives the editor a single place for tracing, debugging, busy indicators, errors, progress, and future automation.

---

## 2. Why this matters

The editor will quickly become a multi-panel application. A single user action can affect several areas at once:

- the mod tree,
- selected asset state,
- metadata panel,
- scene preview cards,
- diagnostics,
- cache state,
- backend calls,
- activity indicators.

Without a central event model, each component starts to own small pieces of logic. That makes behavior difficult to trace.

With an event-driven model, the question:

```txt
"What happened after the user clicked Open Mod?"
```

can be answered by looking at one event/activity log.

---

## 3. Main systems

### 3.1 Editor Event Dispatcher

Central entry point for all editor events.

Responsibilities:

- receive semantic events from UI components,
- assign event IDs and timestamps,
- attach source information,
- route events to handlers,
- emit derived lifecycle events,
- report failures to diagnostics,
- write to the activity log.

Examples of event sources:

- `StartupDialog`,
- `ModTreePanel`,
- `ScenePreviewCard`,
- `OpenModActions`,
- `ValidationStatusBadge`,
- `Toolbar`,
- backend service callback.

---

### 3.2 Editor Task Registry

Central registry of active, completed, failed, and cancelled operations.

Responsibilities:

- track whether an operation is busy,
- track progress,
- expose task state to UI components,
- support cancellation for long-running operations,
- prevent duplicate expensive work,
- classify busy indication level.

Examples:

```txt
scan-known-mods
validate-mod:playground-2d
generate-preview:playground-2d/main.scene.yml
open-mod:ink-wars
```

---

### 3.3 Editor State Store

Central application state used by UI components.

Startup dialog state should include:

- known mod locations,
- selected mod,
- selected scene,
- mod metadata,
- validation result,
- scene preview states,
- active tasks,
- recent errors,
- recently opened mods.

Components should read from this state, but should not mutate it directly.

---

### 3.4 Editor Diagnostics

Central store for validation messages, backend errors, preview failures, missing assets, schema mismatches, and contract violations.

Diagnostics should be linked to:

- mod ID,
- file path,
- scene ID,
- asset ID,
- event ID,
- task ID.

This allows the UI to show errors locally while still keeping a global diagnostic view.

---

### 3.5 Editor Activity Log

A human-readable log of important events and task transitions.

The activity log is useful for:

- debugging UI behavior,
- diagnosing preview generation,
- checking backend calls,
- understanding why a mod cannot be opened,
- building a future developer/debug panel.

The first version can be internal only. Later it can become a visible debug panel.

---

## 4. Event lifecycle

For operations that can take time, events should follow a lifecycle.

Recommended pattern:

```txt
Requested
Started
Progressed
Completed
Failed
Cancelled
```

Example:

```txt
ScenePreviewGenerationRequested
ScenePreviewGenerationStarted
ScenePreviewGenerationProgressed
ScenePreviewGenerationCompleted
ScenePreviewGenerationFailed
ScenePreviewGenerationCancelled
```

Small UI-only actions may only emit a single event:

```txt
ModSelected
SceneSelected
PreviewCardFocused
ThemeChanged
```

---

## 5. Event categories

### 5.1 User intent events

Events emitted directly by UI interactions.

Examples:

```txt
StartupDialogOpened
KnownModsRefreshRequested
ModSelected
ModOpenRequested
ModBrowseRequested
SceneSelected
ScenePreviewRefreshRequested
CancelRequested
```

These events represent what the user wants to happen.

---

### 5.2 System lifecycle events

Events emitted by handlers and services while work is happening.

Examples:

```txt
KnownModsScanStarted
KnownModsScanCompleted
KnownModsScanFailed
ModValidationStarted
ModValidationCompleted
ScenePreviewGenerationStarted
ScenePreviewGenerationCompleted
```

These events represent what the editor is doing.

---

### 5.3 State mutation events

Events that describe a state change.

Examples:

```txt
SelectedModChanged
SelectedSceneChanged
ModMetadataLoaded
ScenePreviewCacheUpdated
DiagnosticsUpdated
```

These are useful for tracing why the UI changed.

---

### 5.4 Backend bridge events

Events around Tauri/Rust calls.

Examples:

```txt
BackendCommandInvoked
BackendCommandCompleted
BackendCommandFailed
BackendCommandTimedOut
```

The frontend should not treat backend calls as invisible implementation details. Expensive or failure-prone backend operations should be visible in the event/task model.

---

## 6. Busy indication model

Busy indication should be derived from the centralized task registry, not implemented independently by each component.

Recommended busy levels:

| Level | Meaning | UI behavior |
|---|---|---|
| `none` | Instant action | No visible loader |
| `local` | Short operation tied to one component | Small spinner inside button, row, or card |
| `panel` | Operation affects one panel | Panel-level loading overlay or skeleton |
| `background` | Longer non-blocking operation | Activity indicator, progress text, cancellable task |
| `blocking` | User cannot safely continue | Dialog-level overlay or disabled primary actions |

Examples:

| Operation | Busy level |
|---|---|
| Selecting a mod | `none` or `local` |
| Loading mod metadata | `local` |
| Scanning known mods | `panel` |
| Validating selected mod | `background` |
| Generating one scene preview | `local` on preview card |
| Generating all previews for selected mod | `background` |
| Opening a mod into the editor workspace | `blocking` |

---

## 7. Startup Dialog event flow

The startup dialog should be the first place where the event system is visible in practice.

### 7.1 Dialog opened

```txt
StartupDialogOpened
→ KnownModsRefreshRequested
→ KnownModsScanStarted
→ KnownModsScanCompleted
→ KnownModsTreeUpdated
```

UI result:

- left mod tree is populated,
- recent mods are highlighted,
- invalid mods get warning/error badges.

---

### 7.2 User selects a mod

```txt
ModSelected
→ SelectedModChanged
→ ModMetadataLoadRequested
→ ModValidationRequested
→ SceneListRequested
→ CachedScenePreviewsRequested
```

UI result:

- metadata panel updates,
- validation badge changes,
- scene preview cards appear,
- missing previews show placeholder states.

---

### 7.3 Metadata loading

```txt
ModMetadataLoadRequested
→ ModMetadataLoadStarted
→ ModMetadataLoaded
or
→ ModMetadataLoadFailed
```

UI result:

- selected mod metadata becomes visible,
- missing or invalid `mod.yml` is shown as diagnostics.

---

### 7.4 Validation

```txt
ModValidationRequested
→ ModValidationStarted
→ ModValidationCompleted
or
→ ModValidationFailed
```

UI result:

- status badge changes to `valid`, `warning`, or `error`,
- diagnostics panel/card receives validation messages,
- `Open` button may be disabled for critical errors.

---

### 7.5 Scene preview generation

```txt
ScenePreviewGenerationRequested
→ ScenePreviewGenerationStarted
→ ScenePreviewFrameCaptured
→ ScenePreviewGenerationCompleted
or
→ ScenePreviewGenerationFailed
```

Rules:

- preview generation is capped at 5 FPS,
- scene previews are cached,
- cache keys depend on scene content, related assets, preview renderer version, and contract version,
- a failed preview should not block the whole dialog.

UI result:

- each preview card can show its own busy state,
- successful previews show static or animated thumbnails,
- failed previews show a warning state.

---

### 7.6 User opens a mod

```txt
ModOpenRequested
→ ModOpenStarted
→ WorkspaceBootstrapStarted
→ ModOpenCompleted
or
→ ModOpenFailed
```

UI result:

- primary button enters busy state,
- dialog may show a blocking overlay,
- editor workspace opens only after the mod is successfully bootstrapped.

---

## 8. Startup Dialog event list

Initial event vocabulary for the first mockup and later implementation:

```txt
StartupDialogOpened
StartupDialogClosed
KnownModsRefreshRequested
KnownModsScanStarted
KnownModsScanCompleted
KnownModsScanFailed
KnownModsTreeUpdated
ModBrowseRequested
ModBrowseCompleted
ModBrowseCancelled
ModSelected
SelectedModChanged
ModMetadataLoadRequested
ModMetadataLoadStarted
ModMetadataLoaded
ModMetadataLoadFailed
ModValidationRequested
ModValidationStarted
ModValidationCompleted
ModValidationFailed
SceneListRequested
SceneListLoaded
SceneListLoadFailed
SceneSelected
SelectedSceneChanged
CachedScenePreviewsRequested
CachedScenePreviewsLoaded
ScenePreviewGenerationRequested
ScenePreviewGenerationStarted
ScenePreviewFrameCaptured
ScenePreviewGenerationCompleted
ScenePreviewGenerationFailed
ScenePreviewGenerationCancelled
AllScenePreviewsGenerationRequested
AllScenePreviewsGenerationStarted
AllScenePreviewsGenerationCompleted
ModOpenRequested
ModOpenStarted
ModOpenCompleted
ModOpenFailed
TaskCancelled
DiagnosticsUpdated
ActivityLogUpdated
```

This list is intentionally explicit. It can be reduced or grouped later if it becomes too noisy.

---

## 9. Event metadata

Every event should carry enough metadata to be useful in debugging.

Recommended metadata:

```txt
eventId
eventName
timestamp
source
correlationId
parentEventId
taskId
modId
sceneId
assetId
filePath
severity
message
```

Important concepts:

- `eventId` identifies one event.
- `correlationId` groups events caused by one user action.
- `parentEventId` links derived events to the event that caused them.
- `taskId` links events to task registry entries.

Example:

```txt
User clicks Refresh
correlationId: refresh-known-mods-2026-05-03-001

KnownModsRefreshRequested
KnownModsScanStarted
KnownModsScanCompleted
KnownModsTreeUpdated
```

---

## 10. Event naming rules

Use semantic names, not UI implementation names.

Prefer:

```txt
ModOpenRequested
ScenePreviewGenerationRequested
KnownModsScanCompleted
```

Avoid:

```txt
OpenButtonClicked
PreviewDivUpdated
LeftPanelReloaded
```

Reason:

- UI can change,
- event meaning should remain stable,
- backend and diagnostics should not depend on component names.

Acceptable exception:

- debug-only events may include component-level names if they are clearly marked as UI debug events.

---

## 11. Component responsibilities

### Components should do

- render current state,
- emit semantic events,
- show busy state from the task registry,
- show diagnostics relevant to their area,
- remain replaceable.

### Components should not do

- parse mod layout directly,
- validate engine contracts directly,
- generate previews directly,
- mutate global mod state directly,
- start backend calls outside the event system,
- maintain separate local busy logic for shared tasks.

---

## 12. Startup Dialog component mapping

### StartupDialog

Emits:

```txt
StartupDialogOpened
StartupDialogClosed
ModOpenRequested
```

Reads:

```txt
selectedMod
activeTasks
criticalDiagnostics
```

---

### ModTreePanel

Emits:

```txt
KnownModsRefreshRequested
ModSelected
ModBrowseRequested
```

Reads:

```txt
knownModsTree
selectedMod
scanKnownMods task
mod validation summaries
```

---

### ModMetadataPanel

Emits:

```txt
ModValidationRequested
```

Reads:

```txt
selectedModMetadata
validationResult
diagnostics for selected mod
```

---

### ScenePreviewGallery

Emits:

```txt
AllScenePreviewsGenerationRequested
SceneSelected
```

Reads:

```txt
selectedModScenes
preview states
preview generation tasks
```

---

### ScenePreviewCard

Emits:

```txt
SceneSelected
ScenePreviewGenerationRequested
ScenePreviewGenerationCancelled
```

Reads:

```txt
scene metadata
preview thumbnail/animation
preview task state
preview diagnostics
```

---

### OpenModActions

Emits:

```txt
ModOpenRequested
ModBrowseRequested
StartupDialogClosed
```

Reads:

```txt
selectedMod
canOpenSelectedMod
open mod task
blocking diagnostics
```

---

## 13. Diagnostics and errors

Errors should not be handled only by local components.

When an operation fails:

```txt
EventFailed
→ DiagnosticsUpdated
→ ActivityLogUpdated
→ TaskRegistryUpdated
```

Example:

```txt
ScenePreviewGenerationFailed
- sceneId: main.scene.yml
- reason: missing texture asset
- severity: warning
```

UI result:

- preview card shows a warning state,
- metadata panel can show a warning count,
- activity log records the failure,
- opening the mod may still be allowed if the error is non-critical.

---

## 14. Cancellation

Long-running operations should be cancellable where possible.

Good candidates:

- generating all previews,
- scanning large mod directories,
- validating large mods,
- importing external assets,
- future batch operations.

Startup dialog cancellation examples:

```txt
ScenePreviewGenerationCancelled
KnownModsScanCancelled
ModValidationCancelled
```

A cancelled task is not an error. It should be logged as cancelled, not failed.

---

## 15. Deduplication and concurrency

The task registry should avoid duplicate expensive operations.

Examples:

- selecting the same mod twice should not trigger two identical scans,
- preview generation for the same scene/cache key should run once,
- repeated refresh clicks should either debounce or cancel/restart the current scan,
- opening a mod should block duplicate open requests.

Recommended behavior:

```txt
same taskId already running
→ attach listener to existing task
or
→ ignore duplicate request
or
→ cancel previous and restart
```

The behavior should be explicit per task type.

---

## 16. Preview-specific rules

Scene previews are expensive enough to be treated as real tasks.

Rules:

- preview generation should be centralized,
- preview state should be visible in the task registry,
- each preview card should read busy state by `taskId`,
- preview generation should be capped at 5 FPS,
- preview results should be cached,
- failed previews should produce diagnostics,
- preview generation should never freeze the startup dialog UI.

Example task IDs:

```txt
generate-preview:playground-2d:main.scene.yml
generate-preview:ink-wars:terrain-test.scene.yml
generate-all-previews:playground-3d
```

---

## 17. Debug panel direction

The event architecture should prepare the editor for a future debug panel.

Potential debug panel sections:

```txt
Recent Events
Active Tasks
Failed Events
Diagnostics
Backend Calls
State Changes
Preview Cache
```

This does not need to exist in the first mockup, but the architecture should be designed so it can be added without rewriting the application.

---

## 18. Mockup implications

The first HTML mockup should visually introduce the event/task model without implementing it.

Recommended visual cues:

- refresh button with local busy state,
- selected mod row with validation status,
- preview cards with `5 FPS` label,
- preview cards with loading/warning/ready states,
- footer activity strip,
- primary Open button with disabled/busy variant,
- small status text such as `Generating previews...` or `Validation passed`.

The mockup should establish reusable patterns for future screens:

- status badges,
- task indicators,
- progress text,
- warning/error states,
- icon + label composition,
- panel-local busy state,
- global activity strip.

---

## 19. First implementation milestone

For the first real implementation, only a minimal version is needed:

```txt
EditorEventDispatcher
EditorTaskRegistry
EditorActivityLog
StartupDialog events
Known mod scan task
Selected mod state
Mock validation task
Mock preview generation task
```

The first implementation does not need the full engine preview renderer yet. It only needs the structure that later allows real backend services to plug in.

---

## 20. Architectural decision

`amigo-editor` should use an event-driven architecture. UI components emit semantic editor events instead of directly performing domain work. Events are handled by centralized dispatchers and services, which update shared editor state, task status, diagnostics, activity logs, and busy indicators.

The startup dialog is the first screen where this model should be reflected visually. It should show mod scanning, mod validation, scene preview generation, and mod opening as observable tasks with clear status and debug-friendly event flow.
