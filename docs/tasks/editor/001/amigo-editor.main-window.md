# Amigo Editor Main Window

## Purpose

`StartupDialog` remains a launcher. After `Open Mod`, the backend creates an `EditorSession` and opens a separate, large workspace window for working on that session.

```txt
StartupDialog
-> Open Mod
-> EditorSession
-> MainEditorWindow / #/workspace?sessionId=...
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

Startup is the only boot window. Workspace windows are created dynamically by the backend through `open_mod_workspace`.

```txt
startup window:
- label: startup
- fixed size
- non-resizable
- launcher only

workspace window:
- label: workspace-<sessionId>
- route: index.html#/workspace?sessionId=<sessionId>
- resizable
- maximizable
- fullscreen-capable
```

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
- open as a separate Tauri WebviewWindow after Open Mod
- load EditorSession by sessionId from route hash
- show titlebar/topbar/statusbar
- fixed left/right/bottom/center dock zones
- tabs inside dock zones
- project explorer from real indexed project files
- asset browser from real indexed project files
- center scene preview from engine slideshow cache
- scene hierarchy from real scene.yml entities
- right inspector from selected scene/entity/file details
- bottom event log and tasks from editor store
- readonly text preview for mod.toml, scene.yml, .yaml, .rhai
- image preview for texture/spritesheet files
```

Out of scope for v1:

```txt
- drag-and-drop docking
- persisted layout
- editable file buffers
- persisted file tab layout
- full asset thumbnails
```

## Event Flow

```txt
OpenModRequested
-> open_mod_workspace
-> backend creates EditorSession
-> backend opens workspace-<sessionId>
-> OpenModCompleted
-> new webview loads #/workspace?sessionId=...
-> get_editor_session(sessionId)
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

Project file selection:

```txt
ProjectFileSelected
-> WorkspaceTabOpened
-> ProjectFileReadRequested, for supported text files
-> ProjectFileReadCompleted
-> InspectorContextChanged(file|asset)
```

Scene document/script selection:

```txt
ProjectFileSelected(scene.yml|scene.rhai)
-> SceneSelected, if the file belongs to a manifest scene
-> WorkspaceTabOpened(file:<relative-path>)
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
9. Add `get_project_tree` for indexed files.
10. Add file tabs and readonly file preview.
11. Split `Open Mod` into backend-created workspace window.
12. Later add layout persistence.

## Current v1 Architecture

Backend remains the source of truth for project data:

```txt
get_mod_details
get_project_tree
get_scene_hierarchy
read_project_file
reveal_project_file
request_scene_preview
```

Window/session flow:

```txt
open_mod_workspace(modId, selectedSceneId?)
-> discover selected mod
-> create session in EditorSessionRegistry
-> create/focus workspace-<sessionId>
-> return OpenModResultDto

App route bridge:
-> StartupDialog for default hash
-> MainEditorWindow for #/workspace
-> get_editor_session(sessionId)
-> load mod details, project tree, selected scene, preview, hierarchy
```

Frontend owns workspace state:

```txt
selectedSceneId
selectedEntityId
selectedFilePath
openedFilePaths
activeWorkspaceTabId
projectTrees
projectFileContents
sceneHierarchies
previews
tasks/events
```

This keeps the main window as a thin dock workspace over engine/editor contracts, not a separate file parser or renderer.
