# Frontend Components — Sheet / TileSet Editor

## 1. Component structure

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

## 2. SheetEditor

Root editor component.

Responsibilities:

```txt
- load resource on mount
- own local editor state
- track dirty/saving/loading states
- call save command
- pass metadata to child panels
- coordinate selection
```

Props:

```ts
{
  sessionId: string;
  resourceUri: string;
}
```

State:

```ts
interface SheetEditorState {
  loading: boolean;
  saving: boolean;
  dirty: boolean;
  resource: SheetResourceDto | null;

  selectedCellId: number | null;
  hoveredCellId: number | null;

  zoom: number;
  panX: number;
  panY: number;

  showGrid: boolean;
  showIds: boolean;
  checkerboard: boolean;
}
```

## 3. SheetToolbar

Toolbar buttons:

```txt
Grid toggle
IDs toggle
Fit
1:1
Validate
Save
```

Shows:

```txt
resource label
relative path
kind badge
unsaved badge
saving state
```

## 4. SheetSettingsPanel

Left settings panel.

Fields:

```txt
Image path
Image size
Cell width
Cell height
Columns
Rows
Count
Margin X
Margin Y
Spacing X
Spacing Y
```

Actions:

```txt
Autodetect by cell size
Autodetect by columns/rows
```

## 5. SheetCanvas

Canvas renderer.

Responsibilities:

```txt
- load image via convertFileSrc
- draw checkerboard
- draw image
- draw grid
- draw IDs
- draw selected cell
- draw hovered cell
- convert pointer coordinate to image coordinate
- resolve selected cell id
```

## 6. sheetMath.ts

Shared functions:

```ts
cellRect(grid, id)
cellIdFromImagePoint(grid, imageX, imageY)
gridPixelBounds(grid)
autodetectByCellSize(resource)
autodetectByColumnsRows(resource)
```

## 7. TileInspectorPanel

Right side panel for selected tile.

Fields:

```txt
Tile ID
Column
Row
Rect
Role
Collision
Damageable
Tags
```

## 8. SheetValidationPanel

Shows diagnostics from backend validation:

```txt
error
warning
info
```

## 9. Workspace integration

Resource resolver:

```ts
*.tileset.yml  -> file.tileset -> sheet.editor mode=tileset
*.semantic.yml -> file.tileset -> sheet.editor mode=tileset, legacy schema
*.sprite.yml   -> file.sprite  -> sheet.editor mode=spritesheet later
*.atlas.yml    -> file.atlas   -> sheet.editor mode=atlas later
*.png/.webp    -> file.texture -> image viewer
```

Open model:

```txt
Project tree click
→ WorkspaceOpenResourceRequested
→ workspaceResolver
→ center tab with file.tileset component
→ file.tileset renders SheetEditor
```

The integration point is:

```txt
src/main-window/workspaceResources.ts
```

Do not add one-off tile editor conditionals to `MainEditorWindow`. Add a descriptor mapping there, then make the registered component render `SheetEditor`.

## 10. Dirty state integration

`SheetEditor` owns local unsaved editor state, but it must report dirty state to the editor store:

```txt
SheetGridChanged
→ FileDirtyStateChanged(resourceUri, true)
→ workspace tab shows dirty state
→ workspace close can be blocked
```

After successful save:

```txt
SheetSaveRequested
→ save_sheet_resource
→ SheetSaved
→ FileDirtyStateChanged(resourceUri, false)
→ refresh project tree / diagnostics if needed
```

The first implementation can keep field edits local until Save. It should not mutate project-wide state directly.

## 11. Save and refresh behavior

After `SheetSaved`, frontend should:

```txt
- replace local resource with backend returned SheetResourceDto
- mark dirty=false
- refresh diagnostics for this sheet
- refresh project tree/content summary if file name or schema kind changed
- emit a cache/preview invalidation later if scenes depend on this tileset
```

For MVP, refreshing the active `get_project_tree` / `get_project_structure_tree` is acceptable. Later this can become targeted invalidation by resource URI.

## 12. Canvas implementation notes

The canvas must be robust on high-DPI displays:

```txt
- maintain CSS size separately from backing pixel size
- multiply backing canvas size by devicePixelRatio
- convert pointer client coordinates to canvas CSS coordinates
- convert canvas coordinates to image coordinates after zoom/pan
- select cell from image coordinates, not screen coordinates
```

Tests/manual checks should include zoomed canvas, fit mode and non-zero margin/spacing.
