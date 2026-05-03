# amigo-editor — introduction

## 1. Project purpose

`amigo-editor` is a desktop tool application for the Amigo project.

Its main purpose is to quickly browse, validate, preview, and lightly edit assets used by Amigo mods. The application should help with scenes, tilesets, tilemaps, sprite atlases, audio files, and YAML files, but it is not intended to become a full game editor.

## 2. Application character

`amigo-editor` is:

- a desktop asset explorer,
- a lightweight asset editor,
- a diagnostic tool for mods,
- a scene and asset preview tool,
- a helper tool for detecting mismatches between YAML structure and what the runtime sees.

`amigo-editor` is not:

- a full game editor,
- a replacement for the engine,
- a tool for building a complete game from scratch,
- a separate game runtime,
- an editor for every possible Amigo system.

## 3. Name

Current application name:

```txt
amigo-editor
```

Previous working name:

```txt
amigo-viewer
```

The name changed because the application will not only be a viewer. It should also support simple asset editing operations.

## 4. Main assumption

The application should be practical and fast to use.

The priority is a comfortable tool-oriented UI:

- mod explorer,
- file tree,
- asset preview,
- simple inspector,
- basic domain-specific editors,
- simple data import/export,
- asset and scene validation.

The interface should feel like a lightweight desktop tool, not a full IDE.

## 5. Working stack

Main technology candidate:

```txt
Tauri 2
React
TypeScript
Vite
Tailwind CSS
Lucide Icons
Radix UI / shadcn-style components
```

Assumptions:

- Tauri acts as the desktop shell.
- React/TypeScript is responsible for the UI.
- The Rust backend is responsible for file access, mod analysis, validation, and integration with Amigo logic.
- The UI uses ready-made components where they make sense.
- Domain-specific editors are implemented as custom Canvas/WebGL components.

## 6. Technology alternatives

### Qt Widgets

A heavier, more classic alternative for desktop applications.

It may be considered if the project needs very mature native desktop controls.

### Slint / Iced

Reserve options closer to the Rust ecosystem.

At the moment, they are not the primary choice because they would likely require more custom controls for the asset editor.

### egui

Rejected as the primary toolkit for this application.

Reason: it is harder to achieve a classic, comfortable tool UI with ready-made controls such as explorers, panels, inspectors, dialogs, and domain-specific editors.

## 7. Ready-made UI components

Ready-made components should be used for classic application elements:

- file tree,
- folder tree,
- split panels,
- tabs,
- forms,
- inputs,
- dropdowns,
- dialogs,
- context menus,
- toolbar,
- search,
- badges,
- status bar,
- property inspector.

These elements should not be written from scratch when good web components already exist.

## 8. Domain-specific editors

Custom components should be built for Amigo-specific features:

- tileset slicer,
- tilemap mini-editor,
- sprite/atlas preview,
- scene preview,
- simple asset renderer,
- grid overlay,
- tile selection,
- brush/picker/eraser tools for tilemaps.

These elements are domain-specific and should be designed around Amigo formats.

## 9. Audio editing

A ready-made waveform component should be used for audio preview and lightweight editing.

Working candidate:

```txt
wavesurfer.js
```

Audio scope:

- waveform preview,
- play/pause,
- loop region,
- markers,
- basic metadata,
- optionally trim/fade as exported metadata, not necessarily destructive file editing.

## 10. YAML / text editing

A ready-made code editor should be used for YAML, JSON, and text files.

Working candidate:

```txt
CodeMirror 6
```

Scope:

- syntax highlighting,
- YAML validation,
- error preview,
- formatting,
- quick navigation to related assets.

## 11. Tiled

Tiled is treated as:

- UX inspiration,
- a tilemap reference,
- a possible import/export format.

There is no assumption that Tiled will be embedded into the application.

## 12. Working UI architecture

Initial structure:

```txt
app/
  shell/
  routes/
  layout/
  panels/
  editors/
  services/
  stores/
  theme/
```

Main concepts:

```txt
AssetEditorRegistry
AssetTree
SelectedAsset
SelectedScene
OpenedMod
PreviewAdapter
ValidationReport
```

Each asset editor should eventually follow a similar contract:

```txt
canOpen(asset)
load(asset)
render()
save()
export()
dirty
```

## 13. First MVP scope

The first MVP should support:

- opening a mod folder,
- showing a file tree,
- detecting basic asset types,
- image preview,
- YAML preview,
- audio preview,
- simple tileset preview with grid overlay,
- simple scene preview,
- inspector panel,
- validation/error status.

## 14. Project rule

`amigo-editor` should grow window by window and editor by editor.

The full tool should not be designed all at once. First, the application shell should become stable. Then additional panels and editors can be added incrementally.
