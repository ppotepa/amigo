# Amigo Editor — Tiles / Sheet Editor Overview

## 1. Goal

The first real editor in `amigo-editor` should be a shared **Sheet Editor Core** with an initial **TileSet Editor mode**.

The editor should support image-based sheet resources:

```txt
Image Viewer
→ Sheet Canvas
→ Sheet Editor Core
→ TileSet Editor
→ SpriteSheet Editor
→ TileMap Editor
```

The first practical target is the `ink-wars` dirt tileset.

The goal is not to paint a full level yet. The first goal is to correctly define, validate, preview, and edit an atlas/sheet resource.

## 2. Why this editor first

This editor is the fastest path toward making simple levels later:

```txt
dirt.png + dirt.tileset.yml
→ tileset.editor
→ first-level.tilemap.yml
→ tilemap.editor
→ scene uses tilemap
```

Starting with a sprite animation editor would help character animation, but it would not immediately enable level construction. A tileset editor enables the next step: tilemap editing.

## 3. Main abstraction

The first editor is not just `tileset.editor`. It is a shared sheet editor:

```txt
sheet.editor
```

Modes:

```txt
mode: tileset
mode: spritesheet
mode: fontsheet / later
```

The first implemented mode:

```txt
mode: tileset
```

## 4. Responsibilities

The Sheet Editor Core is responsible for:

```txt
- loading an image atlas
- displaying the image
- drawing a configurable grid
- supporting margin and spacing
- selecting cells
- calculating cell rects
- validating grid metadata
- saving metadata YAML
```

The TileSet Editor mode adds:

```txt
- tile metadata
- tile role
- tile collision
- tile tags
- tile defaults
- validation of tile definitions
```

The SpriteSheet Editor mode later adds:

```txt
- frame count
- fps
- looping
- animation preview
- named animation ranges
```

## 5. File naming convention

Preferred future convention:

```txt
<name>.<resource-kind>.yml
```

Examples:

```txt
dirt.tileset.yml
first-level.tilemap.yml
player.sprite.yml
ink-font.font.yml
smoke.particle.yml
```

Legacy/current compatibility should remain:

```txt
*.semantic.yml → tileset, if schema looks like tileset
*.layout.yml   → tilemap, if schema looks like tilemap
```

Special files remain supported:

```txt
mod.toml
package.yml
scene.yml
scene.rhai
```

## 6. Workspace behavior

The center workspace is a resource host:

```txt
resource selected
→ classify resource
→ resolve viewer/editor
→ open center tab
```

For the first editor:

```txt
*.tileset.yml / *.semantic.yml
→ file.tileset
→ sheet.editor in tileset mode
```

A raw `.png` should open in `image.viewer`, while a `.tileset.yml` should open the full sheet editor using the referenced image.

The current editor already has a workspace resource resolver:

```txt
src/main-window/workspaceResources.ts
```

The TileSet editor must plug into that resolver instead of adding special cases in `MainEditorWindow`.

Required mapping:

```txt
*.tileset.yml       -> file.tileset -> sheet.editor mode=tileset
*.semantic.yml      -> file.tileset -> sheet.editor mode=tileset, legacy schema
*.sprite.yml        -> file.sprite  -> sheet.editor mode=spritesheet, later
*.atlas.yml         -> file.atlas   -> sheet.editor mode=spritesheet/atlas, later
*.png/.jpg/.webp    -> file.texture -> image viewer
```

`file.tileset` remains the registered workspace component ID. `sheet.editor` is the implementation component used by tileset/sprite/atlas resource families.

## 7. Resource URI contract

For MVP, `resourceUri` is the mod-relative path:

```txt
tilesets/dirt.semantic.yml
tilesets/dirt.tileset.yml
```

Do not introduce `sheet://` or another URI scheme yet. The future tab model may wrap this as a virtual URI, but backend commands should accept the plain mod-relative path while the session owns the mod root.

Rules:

```txt
- resourceUri is always normalized to forward slashes
- resourceUri must stay inside the session mod root
- backend resolves image paths relative to the metadata file
- frontend tabs use resourceUri as the resource identity
```

## 8. MVP outcome

The first MVP is complete when:

```txt
- dirt.semantic.yml or dirt.tileset.yml opens in workspace
- referenced image loads
- grid overlays the image
- margin/spacing/cell-size/columns/rows/count are editable
- tile can be selected
- inspector shows tile id, row, column, rect
- tile role/collision/tags can be edited
- YAML metadata can be saved
- validation catches common errors
```
