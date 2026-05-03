# TileSet / Sheet Editor — Functional Specification

## 1. Editor layers

```txt
ImageViewer
  loads and displays image

SheetCanvas
  overlays grid, cell ids, hover, selection

SheetEditorCore
  owns sheet metadata, validation, save/load

TileSetEditor
  adds tile role/collision/tags/defaults

SpriteSheetEditor
  later adds frame/animation controls

TileMapEditor
  later consumes tileset as palette
```

## 2. Supported files

| File | Resource kind | Workspace component |
|---|---|---|
| `*.tileset.yml` | tileset | `file.tileset` -> `sheet.editor` tileset mode |
| `*.semantic.yml` | legacy tileset | `file.tileset` -> `sheet.editor` tileset mode |
| `*.sprite.yml` | spritesheet | `file.sprite` -> `sheet.editor` spritesheet mode, later |
| `*.atlas.yml` | atlas | `file.atlas` -> `sheet.editor` atlas/spritesheet mode, later |
| `*.tilemap.yml` | tilemap | `file.tilemap` -> `tilemap.editor`, later |
| `*.layout.yml` | legacy tilemap | `file.tilemap` -> `tilemap.editor`, later |
| `*.png`, `*.jpg`, `*.webp` | image | `file.texture` / image viewer |

`sheet.editor` is the implementation behind sheet-like file components. The registered center-tab component IDs stay aligned with the current workspace resolver: `file.tileset`, `file.sprite`, `file.atlas`, `file.texture`.

## 3. Shared sheet features

### 3.1 Image loading

Must support:

```txt
- read image path from metadata
- resolve path relative to metadata file
- check existence
- read actual dimensions
- compare actual dimensions with declared metadata dimensions
- display image format/size/status
```

### 3.2 Canvas

Must support:

```txt
- image display
- checkerboard background
- zoom
- pan
- fit to view
- 1:1
- grid overlay
- cell ids overlay
- selected cell outline
- hovered cell outline
- row/column math
```

MVP can start with zoom and fit; drag-pan can come slightly later.

### 3.3 Grid settings

Required common parameters:

```txt
imageWidth
imageHeight
cellWidth
cellHeight
columns
rows
count
marginX
marginY
spacingX
spacingY
```

Important formula:

```txt
cell.x = marginX + column * (cellWidth + spacingX)
cell.y = marginY + row    * (cellHeight + spacingY)
```

### 3.4 Selection

MVP:

```txt
- single cell selection
- hover cell
- selected id
- row/column
- pixel rect
```

Later:

```txt
- multi-select
- range select
- batch metadata edits
```

### 3.5 Validation

Validation must detect:

```txt
- missing image
- undecodable image
- image size mismatch
- zero cell size
- zero columns/rows
- count > columns * rows
- grid exceeds image bounds
- invalid margin/spacing values
- unsupported schema version
```

## 4. TileSet mode features

### 4.1 Tile metadata

Each tile may have:

```txt
id
role
collision
damageable
tags
```

### 4.2 Tile defaults

Tileset can define defaults:

```txt
default collision
default damageable
```

Tile-specific metadata overrides defaults.

### 4.3 Tile roles

Initial suggested roles:

```txt
empty
ground
ground_single
ground_middle
ground_left
ground_right
ground_top
ground_bottom
corner_inner
corner_outer
slope_left
slope_right
decoration
crack
edge
fill
```

MVP does not need a strict enum. A free text role field is acceptable at first. Later this can become presets.

### 4.4 Collision kinds

Initial collision options:

```txt
none
solid
platform
damage
slopeLeft
slopeRight
```

Later:

```txt
custom polygon
half tile
one-way platform
water/lava/custom material
```

## 5. SpriteSheet mode features — later

Sprite sheet mode should reuse the same SheetCanvas.

It should add:

```txt
fps
looping
frame_count
animation preview
start/end frame
named animation clips
```

Proposed animation schema:

```yaml
animations:
  idle:
    frames: [0, 1, 2, 3]
    fps: 8
    looping: true
  run:
    frames: [8, 9, 10, 11, 12, 13]
    fps: 12
    looping: true
```

## 6. Autodetect

Autodetect should not be one magic function. It should have modes.

### MVP autodetect

```txt
- by cell size
- by columns / rows
```

### Later autodetect

```txt
- detect transparent gaps
- detect non-empty rectangles
- detect irregular atlas frames
```

## 7. Toolbar

MVP toolbar:

```txt
[Open Image] [Autodetect] [Grid] [IDs] [Fit] [1:1] [Validate] [Save]
```

## 8. Inspector

Selected tile inspector:

```txt
Tile #12
Column: 4
Row: 1
Rect: x=1024 y=256 w=256 h=256
Role
Collision
Damageable
Tags
```

## 9. Things explicitly out of scope for MVP

```txt
- PNG editing
- tilemap painting
- autotiling rule editor
- collision polygon editor
- transparent gap detection
- named sprite animations
- multi-select tile operations
- drag/drop import
```
