# Backend Contract — Sheet / TileSet Editor

## 1. Commands

```rust
load_sheet_resource(session_id, resource_uri) -> SheetResourceDto
save_sheet_resource(session_id, resource_uri, dto) -> SheetResourceDto
validate_sheet_resource(session_id, resource_uri) -> SheetValidationDto
read_image_info(path) -> ImageInfoDto
```

MVP requires only:

```txt
load_sheet_resource
save_sheet_resource
```

## 2. SheetResourceDto

```ts
export type SheetKind = "tileset" | "spritesheet";

export interface SheetResourceDto {
  resourceUri: string;
  absolutePath: string;
  relativePath: string;

  kind: SheetKind;
  schemaVersion: number;
  sourceSchemaKind: "modern" | "legacy";

  id: string;
  label: string;

  imagePath: string;
  imageAbsolutePath: string;
  imageExists: boolean;

  imageWidth: number | null;
  imageHeight: number | null;

  declaredImageWidth: number | null;
  declaredImageHeight: number | null;

  cellWidth: number;
  cellHeight: number;

  columns: number;
  rows: number;
  count: number;

  marginX: number;
  marginY: number;
  spacingX: number;
  spacingY: number;

  tileset?: TileSetPayloadDto | null;

  diagnostics: EditorDiagnosticDto[];
}
```

## 3. TileSetPayloadDto

```ts
export interface TileSetPayloadDto {
  defaults: TileSetDefaultsDto;
  tiles: Record<string, TileMetadataDto>;
}

export interface TileSetDefaultsDto {
  collision: string;
  damageable: boolean;
}

export interface TileMetadataDto {
  id: number;
  role?: string | null;
  collision?: string | null;
  damageable?: boolean | null;
  tags: string[];
}
```

## 4. Diagnostics

```ts
export interface EditorDiagnosticDto {
  level: "info" | "warning" | "error";
  code: string;
  message: string;
  path?: string | null;
}
```

Validation codes:

```txt
sheet_image_missing
image_size_mismatch
invalid_cell_size
invalid_grid_size
tile_count_overflow
grid_exceeds_image_bounds
unsupported_schema_version
```

## 5. Loader behavior

The backend loader must:

```txt
1. Resolve resourceUri relative to session root.
2. Read YAML.
3. Detect schema style.
4. Normalize legacy/new YAML into SheetResourceDto.
5. Resolve image path relative to metadata file.
6. Read image dimensions if possible.
7. Validate DTO.
8. Return normalized DTO.
```

## 6. Supported YAML schemas

### New schema

```yaml
kind: tileset
schema_version: 1
id: dirt
label: Dirt
atlas:
  image: dirt.png
  image_size: { x: 2048, y: 2048 }
  cell_size: { x: 256, y: 256 }
  columns: 8
  rows: 8
  count: 64
  margin: { x: 0, y: 0 }
  spacing: { x: 0, y: 0 }
defaults:
  collision: solid
  damageable: true
tiles: {}
```

### Legacy/current schema

```yaml
tileset:
  id: dirt
  label: Dirt
  atlas:
    image: dirt.png
    image_size: { x: 2048, y: 2048 }
    tile_size: { x: 256, y: 256 }
    columns: 8
    rows: 8
    tile_count: 64
```

Loader normalizes both to the same DTO.

## 7. Saver behavior

MVP saver writes normalized modern schema.

Important legacy rule:

```txt
Do not silently overwrite a legacy `*.semantic.yml` file with modern schema unless that behavior is explicit in the command.
```

Recommended MVP behavior:

```txt
save_sheet_resource(session_id, resource_uri, dto)
```

If `sourceSchemaKind == modern`, save back to the same file.

If `sourceSchemaKind == legacy`, save to a modern sibling path by default:

```txt
tilesets/dirt.semantic.yml
-> tilesets/dirt.tileset.yml
```

Return the saved resource with updated:

```txt
resourceUri
absolutePath
relativePath
sourceSchemaKind: modern
```

Later improvement: add explicit save policy:

```rust
pub enum SheetSavePolicy {
    PreserveSourcePath,
    WriteModernSibling,
    WriteAs { relative_path: String },
}
```

This avoids surprising destructive rewrites of existing legacy files.

## 8. Path rules

Image path should be stored as relative to the metadata file when possible:

```txt
dirt.tileset.yml
└─ image: dirt.png
```

If image is outside the metadata folder, use relative path:

```txt
image: ../textures/dirt.png
```

## 9. Safety and validation rules

Path safety:

```txt
- resourceUri is mod-relative and normalized with `/`
- resolved resource path must stay inside session root
- image path is resolved relative to the metadata file
- image path must also stay inside session root unless explicitly allowed later
```

Write safety:

```txt
- serialize YAML to a temp file in the same directory
- flush/write completely
- replace target file atomically where platform support allows
- on failure, keep the previous file untouched
- reload saved YAML and validate before returning DTO
```

Validation after save is mandatory. A save that writes invalid YAML should return `SheetSaveFailed` and leave dirty state true.

## 10. Project refresh after save

After save, backend or frontend must trigger refresh of:

```txt
get_project_tree
get_project_structure_tree
get_mod_details content summary
diagnostics for the saved sheet
```

MVP may let frontend call the existing commands after `SheetSaved`. Later this should become targeted `ResourceChanged` / `ProjectTreeInvalidated` events.
