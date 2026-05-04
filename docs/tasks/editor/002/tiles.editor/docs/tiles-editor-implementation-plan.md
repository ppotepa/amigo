# TileSet / Sheet Editor — Step-by-Step Implementation Plan

## 1. Implementation order

### Commit 1 — Workspace resource resolver

Use the current workspace resolver:

```txt
src/main-window/workspaceResources.ts
```

Add or verify resource classification:

```txt
*.tileset.yml -> file.tileset -> SheetEditor mode=tileset
*.sprite.yml                  -> file.sprite  -> sheet.editor later
*.atlas.yml                   -> file.atlas   -> sheet.editor later
*.png/.jpg/.webp              -> file.texture -> image viewer
*.yml/.yaml fallback          -> file.config
```

Register `sheet.editor` implementation behind `file.tileset`. Do not add direct tile editor conditionals to `MainEditorWindow`.

Acceptance:

```txt
Click dirt.tileset.yml -> opens SheetEditor through file.tileset.
```

---

### Commit 2 — Backend sheet DTO and loader command

Add backend modules:

```txt
src-tauri/src/sheet/
├─ mod.rs
├─ dto.rs
├─ loader.rs
├─ image_info.rs
├─ validator.rs
└─ saver.rs
```

Commands:

```txt
load_sheet_resource(sessionId, resourceUri)
save_sheet_resource(sessionId, resourceUri, dto)
```

Acceptance:

```txt
Frontend can call load_sheet_resource and receive image/grid metadata.
```

Also define the MVP `resourceUri` contract:

```txt
resourceUri = mod-relative path using forward slashes
example: assets/tilesets/dirt.tileset.yml
```

---

### Commit 3 — Basic SheetEditor shell

Frontend files:

```txt
src/editors/sheet/SheetEditor.tsx
src/editors/sheet/SheetToolbar.tsx
src/editors/sheet/SheetSettingsPanel.tsx
src/editors/sheet/sheetTypes.ts
src/editors/sheet/sheetApi.ts
```

Acceptance:

```txt
Editor shows image path, image size, cell size, columns, rows, count, margin, spacing.
```

---

### Commit 4 — Image display canvas

Add:

```txt
SheetCanvas.tsx
```

Responsibilities:

```txt
- convertFileSrc image path
- draw image on canvas
- checkerboard background
- basic zoom
```

Acceptance:

```txt
dirt.png is visible in workspace.
```

---

### Commit 5 — Grid overlay

Add:

```txt
sheetMath.ts
```

Implement:

```txt
cellRect(grid, id)
cellIdFromImagePoint(grid, x, y)
gridPixelBounds(grid)
```

Acceptance:

```txt
8x8 grid appears over dirt.png and tile IDs 0–63 are visible.
```

---

### Commit 6 — Selection + tile inspector

Add:

```txt
TileInspectorPanel.tsx
```

Features:

```txt
- hover tile
- click select tile
- show id/row/column/rect
```

Acceptance:

```txt
Clicking tile #12 shows its row, column and pixel rect.
```

---

### Commit 7 — Editable grid settings

Make fields editable:

```txt
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

Acceptance:

```txt
Changing margin/spacing updates grid immediately.
```

---

### Commit 8 — Save YAML

Add saver command:

```txt
save_sheet_resource
```

Save policy:

```txt
modern source -> save same file
migration tool -> create descriptor-first file, e.g. old tileset metadata -> assets/tilesets/dirt.tileset.yml
```

Acceptance:

```txt
Changing margin/spacing saves to YAML and survives reload.
Saving legacy paths is not supported; migrate first, then save assets/tilesets/*.tileset.yml.
```

---

### Commit 9 — Tile metadata

Tile inspector supports:

```txt
role
collision
damageable
tags
```

Acceptance:

```txt
Tile #12 can be assigned role/collision/tags and saved to YAML.
```

---

### Commit 10 — Engine margin/spacing support

Update engine/runtime models and UV math:

```txt
SpriteSheet margin/spacing
TileSet2d margin/spacing
UV calculation with margin/spacing
parser fallback to zero
tests
```

Acceptance:

```txt
Runtime renders sheets with margin/spacing the same way as editor.
```

## 2. Frontend file structure

```txt
src/editors/sheet/
├─ SheetEditor.tsx
├─ SheetCanvas.tsx
├─ SheetToolbar.tsx
├─ SheetSettingsPanel.tsx
├─ TileInspectorPanel.tsx
├─ SheetValidationPanel.tsx
├─ sheetTypes.ts
├─ sheetMath.ts
├─ sheetApi.ts
└─ sheetEditor.css
```

Optional shared viewer:

```txt
src/viewers/image/
├─ ImageViewer.tsx
└─ imageViewer.css
```

## 3. Backend file structure

```txt
src-tauri/src/sheet/
├─ mod.rs
├─ dto.rs
├─ loader.rs
├─ saver.rs
├─ validator.rs
├─ image_info.rs
└─ legacy.rs
```

## 4. Backend commands

```rust
#[tauri::command]
pub fn load_sheet_resource(
    session_id: String,
    resource_uri: String,
) -> Result<SheetResourceDto, String>

#[tauri::command]
pub fn save_sheet_resource(
    session_id: String,
    resource_uri: String,
    dto: SheetResourceDto,
) -> Result<SheetResourceDto, String>
```

## 5. Key data flow

```txt
Project Tree node activated
-> Workspace resolver classifies resource
-> file.tileset descriptor opens center tab
-> file.tileset renders SheetEditor
-> SheetEditor calls load_sheet_resource
-> backend normalizes YAML to SheetResourceDto
-> frontend renders image + grid
-> user edits metadata
-> SheetGridChanged marks resource dirty
-> save_sheet_resource writes YAML
-> backend reloads and validates
-> frontend updates clean state
-> project tree / structure tree refresh
```

## 6. Required event names

```txt
SheetResourceLoadRequested
SheetResourceLoaded
SheetResourceLoadFailed
SheetGridChanged
SheetCellSelected
SheetValidationCompleted
SheetSaveRequested
SheetSaved
SheetSaveFailed
FileDirtyStateChanged
ProjectTreeInvalidated
```

## 7. Tests

Backend:

```txt
load_new_tileset_schema
reject_legacy_semantic_schema
validate_missing_image
validate_grid_exceeds_image
validate_count_exceeds_grid
save_tileset_roundtrip
save_descriptor_first_tileset_schema
resource_uri_cannot_escape_session_root
```

Frontend/manual:

```txt
open dirt.tileset.yml
see image
see grid
select tile 0
select tile 63
change spacing
change margin
save
reload
close workspace with dirty sheet shows dirty-state behavior
```

Engine:

```txt
sprite_sheet_uv_without_margin_spacing_same_as_before
sprite_sheet_uv_with_margin_spacing
tileset_uv_with_margin_spacing
```
