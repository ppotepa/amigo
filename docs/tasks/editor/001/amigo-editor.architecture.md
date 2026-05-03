# Amigo Editor Architecture

## Purpose

`amigo-editor` is a desktop asset explorer and lightweight asset editor for Amigo mods. It is not a full game editor. Its main responsibility is to open a mod, show its structure, validate it against engine-defined contracts, preview assets/scenes, and allow small focused edits to assets and metadata.

The editor must stay consistent with the engine. It should not invent its own interpretation of mod folders, scene files, asset metadata, or validation rules.

## Core Rule

The engine side owns the contracts. The editor consumes them.

```txt
shared contracts -> editor backend -> frontend UI
```

The editor must not hardcode assumptions such as:

```txt
assets/
scenes/
scripts/
audio/
textures/
```

Instead, it should ask the shared contract layer what a valid mod layout looks like.

## Shared Contract Layer

Recommended crate:

```txt
crates/foundation/mod-contracts
```

Potential Rust crate name:

```txt
amigo_mod_contracts
```

This crate should define the canonical structure of an Amigo mod:

- allowed root folders,
- optional and required folders,
- supported asset types,
- supported file extensions,
- `mod.yml` manifest schema,
- scene file schema,
- tileset metadata schema,
- tilemap metadata schema,
- script package metadata schema,
- audio metadata schema,
- validation rules,
- contract version.

The engine/runtime and `amigo-editor` should both depend on this crate.

## Contract Responsibilities

### `ModLayoutContract`

Describes the expected folder layout of a mod.

Examples:

```txt
ModLayoutContract
- root files
- known folders
- folder labels
- folder icons
- accepted file kinds per folder
- required/optional markers
```

### `AssetTypeRegistry`

Maps files to asset kinds.

Examples:

```txt
.png  -> image/sprite/tileset candidate
.webp -> image/sprite candidate
.wav  -> audio
.ogg  -> audio
.yml  -> manifest/scene/asset metadata
.rhai -> script
```

### `SceneSchemaRegistry`

Defines what scene documents may contain and how they are validated.

### `ModManifestContract`

Defines expected fields for `mod.yml`:

```txt
id
name
version
author
description
capabilities
entry scenes
contract version
```

### `ValidationReport`

Canonical output for validation.

```txt
ValidationReport
- errors
- warnings
- info
- affected file path
- code
- message
- optional fix suggestion
```


## Icon and Asset Kind Contract

Icons should not be random frontend-only choices. The shared contract layer may expose semantic icon keys for folders, asset kinds, validation states, and editor tools.

The contract should not depend on the concrete icon library. It should expose stable semantic keys:

```txt
IconKey
- folder
- folder-open
- scene
- image
- tileset
- tilemap
- audio
- script
- yaml
- warning
- error
- valid
```

The frontend maps those keys to the selected icon library:

```txt
IconKey::Scene   -> Lucide Film or PanelsTopLeft
IconKey::Audio   -> Lucide AudioLines
IconKey::Tileset -> Lucide Grid3X3
IconKey::Script  -> Lucide FileCode
```

This keeps the engine/editor contract stable while allowing the UI to change icon libraries later.

Recommended flow:

```txt
ModLayoutContract / AssetTypeRegistry
  -> semantic folder kind / asset kind / icon key
  -> editor backend DTO
  -> frontend Icon component
  -> Lucide React icon
```

The first startup dialog should be treated as the initial visual precedent for the whole editor icon language.

## Editor Backend

The Tauri/Rust backend acts as an adapter between the filesystem, engine contracts, and the frontend.

Recommended package area:

```txt
apps/amigo-editor/src-tauri/
```

Potential internal modules:

```txt
src-tauri/src/
├─ commands/
├─ mod_host/
├─ scanning/
├─ validation/
├─ preview/
├─ cache/
└─ io/
```

Backend responsibilities:

- open a mod folder,
- scan the mod tree,
- classify files using `AssetTypeRegistry`,
- parse manifests and metadata,
- validate files using shared contracts,
- generate diagnostics,
- generate scene previews,
- cache previews,
- save edited files,
- export/import supported formats.

The frontend should not directly parse the mod structure. It receives typed results from backend commands.

## Backend Command Surface

Initial command list:

```txt
list_known_mods()
open_mod(path)
scan_mod(path)
validate_mod(path)
get_mod_metadata(path)
get_mod_tree(path)
list_mod_scenes(path)
get_asset_details(path)
generate_scene_preview(mod_path, scene_id)
get_cached_scene_preview(mod_path, scene_id)
save_asset(path, content)
```

## Preview Architecture

Scene previews should be generated from the same data model and contracts as the engine.

Startup dialog preview requirements:

- bootstrap the selected mod and scene through the engine preview host,
- render through the existing offscreen WGPU pipeline,
- capture one initial frame with `capture_rgba8()`,
- capture subsequent slideshow frames with `capture_next_frame()`,
- play cached frames in the frontend at 5 FPS,
- cache generated previews,
- invalidate cache when scene files or referenced assets change.

Recommended cache location:

```txt
target/amigo-editor/cache/projects/<project-cache-id>/previews/scenes/<scene-id>/<hash>/
```

Cache invalidation inputs:

- scene file hash,
- referenced asset hashes,
- preview renderer version,
- mod contract version.

The startup dialog must not generate decorative metadata/SVG placeholders. If the engine preview cannot bootstrap or render, the backend returns `PreviewStatus::Failed` with diagnostics and no fake image.

See `amigo-editor.cache.md` for the dedicated cache subsystem contract.

## Frontend Architecture

Frontend stack:

```txt
React + TypeScript + Vite + Tailwind/CSS variables + Lucide icons
```

The frontend is organized as composable panels and embeddable editors.

Recommended structure:

```txt
src/
├─ app/
├─ contexts/
├─ panels/
├─ editors/
├─ components/
├─ services/
├─ types/
└─ themes/
```

## Global Contexts

### `ModContext`

The central frontend context for the currently opened mod.

```txt
ModContext
- openedMod
- modTree
- selectedAsset
- selectedScene
- diagnostics
- dirtyFiles
- editorTabs
- activeTool
```

### `EditorWorkspaceContext`

Tracks the current UI workspace state.

```txt
EditorWorkspaceContext
- activePanel
- activeTab
- panelLayout
- searchQuery
- commandPaletteState
```

### `ThemeContext`

Tracks visual theme and design tokens.

```txt
ThemeContext
- themeId
- density
- accentColor
```

## Component Model

Editor components should be embeddable in panels and dialogs.

Examples:

```txt
ModTreePanel
AssetBrowserPanel
InspectorPanel
ScenePreviewPanel
TilesetEditorPanel
TilemapEditorPanel
AudioPreviewPanel
YamlEditorPanel
StartupDialog
```

Each component may have a local context for local state, but it should communicate with `ModContext` through actions rather than mutating global state directly.

Example actions:

```txt
selectAsset(path)
openEditor(path)
updateAssetMetadata(path, patch)
saveFile(path)
validateMod()
refreshPreview(sceneId)
```

## Editor Registry

Asset editors should be selected through a registry.

```txt
AssetEditorRegistry
- canOpen(asset)
- load(asset)
- render()
- save()
- export()
- dirty state
```

Initial editor types:

```txt
YamlEditor
ImagePreviewEditor
SpriteAtlasEditor
TilesetSlicerEditor
TilemapMiniEditor
AudioPreviewEditor
ScenePreviewEditor
```

## Ready-Made vs Custom Components

Use ready-made UI components for classic desktop controls:

- file tree,
- lists,
- tabs,
- split panels,
- dialogs,
- forms,
- inspectors,
- toolbars,
- search.

Use custom Canvas/WebGL components for Amigo-specific editors:

- tileset slicing,
- tilemap editing,
- sprite/atlas preview,
- scene preview.

Use specialized existing libraries where they clearly fit:

- CodeMirror for YAML/text editing,
- wavesurfer.js for waveform/audio preview.

## Dependency Direction

The intended dependency direction is:

```txt
amigo_mod_contracts
  -> amigo runtime / engine domains
  -> amigo-editor backend
  -> amigo-editor frontend
```

The engine should not depend on the editor.

The editor can depend on contracts and selected engine-side read/preview utilities.

## First Milestone

The first milestone is the startup dialog:

```txt
StartupDialog
- list known mods
- show mod tree
- show selected mod details and content summary
- show validation status
- show scene preview as the primary interactive center workspace
- show preview playback capped at 5 FPS
- show task/event activity in the footer
- keep mod/scene inspector scrollable on the right
- allow opening a selected mod
- allow browsing for a mod folder
```

Startup selection should prefer a user-facing mod with visible scenes over internal runtime/core mods. Core modules may still be selectable for debugging, but the first screen should demonstrate a real playable/visual scene preview when one is available.

The startup window is fixed at `1340x880`. Scene preview playback always fits the central preview canvas.
