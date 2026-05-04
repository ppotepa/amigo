# Amigo Asset Convention

Amigo uses descriptor-first assets.

A managed editor/runtime asset is a typed YAML descriptor. Raw files such as images, audio, fonts
and source references are source files referenced by descriptors. Raw files can be previewed in the
editor, but they are unmanaged until a descriptor exists.

## Canonical Naming

```text
<asset-id>.<asset-kind>.yml
```

Examples:

```text
dirt.tileset.yml
first-level.tilemap.yml
notebook-paper.image.yml
player.sprite.yml
```

## Active Editor MVP

The current editor MVP exposes only these asset editors/viewers:

```text
*.image.yml      -> image asset viewer/editor
*.sprite.yml     -> spritesheet SheetEditor
*.atlas.yml      -> spritesheet/atlas SheetEditor
*.tileset.yml    -> tileset SheetEditor
*.tilemap.yml    -> TilemapEditor
raw .png/.jpg/.webp -> raw image viewer
```

Descriptor suffixes for font, audio, particle, material, UI and input remain part of the future
runtime/editor contract, but they are not active in the current Assets UI and cannot be created from
the current Create Descriptor action.

## Canonical Folder Layout

```text
assets/
  raw/
    images/
    audio/
    fonts/
    other/

  images/
  sprites/
  tilesets/
  tilemaps/
  fonts/
  audio/
  particles/
  materials/
  ui/
```

`assets/raw/...` contains binary/source files. `assets/<kind>/...` contains typed YAML descriptors.

## Supported Descriptor Suffixes

```text
*.image.yml
*.sprite.yml
*.tileset.yml
*.tilemap.yml
*.font.yml
*.audio.yml
*.particle.yml
*.material.yml
*.ui.yml
*.input.yml
```

The workspace resolver opens dedicated editors from typed suffixes. It does not infer domain assets
from folder names or binary filenames.

## No Legacy Active Path

Legacy descriptors such as `*.semantic.yml`, `*.layout.yml` and untyped image YAML files are not an
active editor path. Existing projects must be migrated to descriptor-first assets before being edited
with the new asset tools.

One-time migration may read old files to produce new descriptors, but normal editor code should load,
save, list and resolve only the descriptor-first structure.

## Asset Registry

The editor backend owns the asset registry for a session.

```txt
get_asset_registry(sessionId)
```

The registry separates:

```txt
managed assets  -> typed YAML descriptors under assets/<kind>/
raw files       -> source files under assets/raw/
orphan raw      -> raw files not referenced by any descriptor
missing source  -> descriptors whose source.file does not exist
```

The registry is the source of truth for the `Assets` dock tab. The Project tab remains a structure
tree, while Assets is a filtered registry view.

## Create Descriptor Flow

Raw files can be promoted to managed assets:

```txt
raw file -> create_asset_descriptor(sessionId, rawFilePath, kind, assetId)
```

The command writes a typed descriptor in `assets/<kind>/`, keeps the raw file in place, emits
`asset-descriptor-changed`, emits `asset-registry-changed`, and invalidates asset-related cache.

In the current MVP, `create_asset_descriptor` accepts only:

```text
image
tileset
sprite
```

Tilemaps are created from editor/menu actions rather than directly from a raw image.

## Runtime Resolver

Runtime asset keys resolve descriptor-first paths:

```txt
ink-wars/images/notebook-paper-16x9 -> assets/images/notebook-paper-16x9.image.yml
ink-wars/tilesets/dirt              -> assets/tilesets/dirt.tileset.yml
ink-wars/fonts/notebook-ink         -> assets/fonts/notebook-ink.font.yml
```

The parser aliases descriptor fields such as `source.file` and `atlas.tile_size.width` to the
existing prepared-asset metadata keys required by render/runtime systems.
