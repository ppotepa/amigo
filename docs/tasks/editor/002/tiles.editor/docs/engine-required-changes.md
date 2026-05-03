# Engine Changes Required for Sheet / TileSet Editor

## 1. Why engine changes are required

The editor must not write metadata that the runtime cannot use. If the editor supports `margin` and `spacing`, the engine/runtime must use the same math for sprite and tileset UV calculation.

The current engine already has core sheet concepts such as:

```txt
SpriteSheet:
- columns
- rows
- frame_count
- frame_size
- fps
- looping
```

and tilemap / tileset concepts such as:

```txt
TileSet2d:
- asset
- tile_size
- columns
- rows

TileMap2d:
- tileset
- ruleset
- tile_size
- grid
- origin_offset
- resolved
```

But margin and spacing are not part of the current practical metadata model. They need to be added.

## 2. Add margin and spacing to SpriteSheet

Current model should be extended conceptually:

```rust
pub struct SpriteSheet {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,

    pub margin: Vec2,
    pub spacing: Vec2,
}
```

Defaults:

```rust
margin: Vec2::ZERO,
spacing: Vec2::ZERO,
```

## 3. Add margin and spacing to TileSet2d

Extend tileset model / render info:

```rust
pub struct TileSet2d {
    pub asset: AssetKey,
    pub tile_size: Vec2,
    pub columns: u32,
    pub rows: u32,

    pub margin: Vec2,
    pub spacing: Vec2,
}
```

If the engine has a separate `TileSetRenderInfo`, it must receive the same fields.

## 4. Update UV calculation

Old simplified math:

```txt
x = column * frame_width
y = row * frame_height
```

Required math:

```txt
x = margin.x + column * (frame_width + spacing.x)
y = margin.y + row    * (frame_height + spacing.y)
```

Then:

```txt
u0 = x / image_width
v0 = y / image_height
u1 = (x + frame_width) / image_width
v1 = (y + frame_height) / image_height
```

## 5. Parser changes

Asset metadata parser should accept:

```yaml
margin:
  x: 0
  y: 0
spacing:
  x: 0
  y: 0
```

Also accept nested atlas style:

```yaml
atlas:
  margin:
    x: 0
    y: 0
  spacing:
    x: 0
    y: 0
```

Fallback values:

```txt
missing margin → 0,0
missing spacing → 0,0
```

## 6. Formal sheet contract

Add a reusable sheet/atlas contract somewhere appropriate:

```rust
pub struct SheetGridDefinition {
    pub cell_size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub count: u32,
    pub margin: Vec2,
    pub spacing: Vec2,
}
```

Both sprite and tileset metadata can normalize into this.

## 7. Formal tileset semantic model

Add or formalize:

```rust
pub struct TileSetDefinition {
    pub id: String,
    pub label: String,
    pub atlas: SheetGridDefinition,
    pub defaults: TileSetDefaults,
    pub tiles: BTreeMap<u32, TileDefinition>,
}

pub struct TileDefinition {
    pub id: u32,
    pub role: Option<String>,
    pub collision: Option<TileCollisionKind>,
    pub damageable: Option<bool>,
    pub tags: Vec<String>,
}
```

Initial collision kinds:

```txt
none
solid
platform
damage
slopeLeft
slopeRight
```

## 8. Tests to add

```txt
sprite_sheet_uv_without_margin_spacing_same_as_before
sprite_sheet_uv_with_margin_spacing
tileset_uv_without_margin_spacing_same_as_before
tileset_uv_with_margin_spacing
metadata_parser_defaults_margin_spacing_to_zero
metadata_parser_reads_margin_spacing
```

## 9. Compatibility requirement

Existing sheets without margin/spacing must render exactly as before.

```txt
No margin/spacing in metadata
→ margin = 0
→ spacing = 0
→ old UV math preserved
```
