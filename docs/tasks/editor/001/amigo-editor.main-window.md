# Amigo Editor Main Window

## Purpose

`StartupDialog` remains a launcher. After `Open Mod`, the editor opens a large workspace window for working on the selected mod session.

```txt
StartupDialog
-> Open Mod
-> EditorSession
-> MainEditorWindow
```

The main window is not an expanded startup dialog. It is the primary editor workspace.

## Window Model

Initial workspace target:

```txt
width: 1440
height: 900
minWidth: 1200
minHeight: 720
resizable: true
maximizable: true
```

The first implementation may render in the same Tauri webview after `appMode` switches to `editor`. A later iteration can split this into a separate Tauri window keyed by `sessionId`.

## Session Model

The workspace is session-based.

```txt
EditorSession
- sessionId
- modId
- rootPath
- projectCacheId
- manifest/details
- scenes
- contentSummary
- diagnostics
- selectedSceneId
- openedTabs
- layoutState
```

Backend owns session creation and mod data. Frontend owns selected dock tabs, opened workspace tabs, and current inspector context.

## Dock Layout

Main layout:

```txt
Titlebar
Topbar
Workspace
  left dock
  center workspace
  right dock
  bottom dock
Statusbar
```

Default dock tabs:

```txt
left:
- Project Explorer
- Asset Browser
- Scene Hierarchy

center:
- Scene Preview
- Scene document tabs later

right:
- Inspector
- Diagnostics
- Properties

bottom:
- Problems
- Event Log
- Tasks
- Console
- Preview Cache
```

## Dock Plugins

Dock panels are UI plugins. The first version uses an internal registry, not dynamic extensions.

```ts
interface DockPlugin {
  id: string;
  title: string;
  icon: string;
  defaultDock: "left" | "right" | "bottom" | "center";
  canOpen(context: EditorContext): boolean;
  render(context: EditorContext): React.ReactNode;
}
```

## MVP Scope

MainEditorWindow v1:

```txt
- render after Open Mod
- use existing EditorSession result
- show titlebar/topbar/statusbar
- fixed left/right/bottom/center dock zones
- tabs inside dock zones
- project explorer from real mod details
- center scene preview from engine slideshow cache
- right inspector from selected scene/mod details
- bottom event log and tasks from editor store
```

Out of scope for v1:

```txt
- drag-and-drop docking
- persisted layout
- multiple native Tauri windows
- full entity hierarchy extraction
- full asset thumbnails
```

## Event Flow

```txt
OpenModRequested
-> OpenModCompleted
-> EditorSessionLoaded
-> DockLayoutLoaded
-> WorkspaceReady
```

Scene selection:

```txt
SceneSelected
-> ScenePreviewRequested
-> InspectorContextChanged
```

Dock interaction:

```txt
DockTabSelected
-> WorkspaceContextChanged
```

## Implementation Order

1. Add `MainEditorWindow` shell.
2. Replace placeholder `EditorWorkspace`.
3. Add static dock tabs.
4. Populate Project Explorer from `EditorModDetailsDto`.
5. Reuse `EngineSlideshowPreview` in center.
6. Add right inspector and diagnostics.
7. Add bottom events/tasks.
8. Add workspace statusbar.
9. Later add layout persistence and real native window split.
