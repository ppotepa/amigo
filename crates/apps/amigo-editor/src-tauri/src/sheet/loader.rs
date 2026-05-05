use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::dto::{DiagnosticLevel, EditorDiagnosticDto};
use crate::sheet::dto::{
    SheetKind, SheetResourceDto, SheetSourceSchemaKind, SpriteAnimationDto, TileMetadataDto,
    TileRulesetResourceDto, TileRulesetTerrainDto, TileRulesetVariantsDto, TileSetDefaultsDto,
    TileSetPayloadDto, TilemapCellDto, TilemapResourceDto,
};

pub fn load_sheet_resource(root: &Path, resource_uri: &str) -> Result<SheetResourceDto, String> {
    let path = resolve_resource_path(root, resource_uri)?;
    let source = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read sheet resource `{}`: {error}",
            path.display()
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&source).map_err(|error| {
        format!(
            "failed to parse sheet resource `{}`: {error}",
            path.display()
        )
    })?;
    let value = yaml_to_json(yaml);

    let mut dto = load_descriptor_sheet(root, &path, resource_uri, &value)?;

    add_image_info_and_diagnostics(&mut dto);
    Ok(dto)
}

pub fn load_tilemap_resource(
    root: &Path,
    resource_uri: &str,
) -> Result<TilemapResourceDto, String> {
    let path = resolve_resource_path(root, resource_uri)?;
    let source = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read tilemap resource `{}`: {error}",
            path.display()
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&source).map_err(|error| {
        format!(
            "failed to parse tilemap resource `{}`: {error}",
            path.display()
        )
    })?;
    let value = yaml_to_json(yaml);
    let tileset_resource_uri = string_at(&value, &["tileset"])
        .map(|tileset_path| normalize_tileset_resource_uri(root, &path, &tileset_path));
    let grid = value.get("grid").unwrap_or(&Value::Null);
    let layer_cells = value
        .get("layers")
        .and_then(Value::as_object)
        .and_then(|layers| layers.values().find_map(|layer| layer.get("cells")));
    let cells_value = layer_cells
        .or_else(|| value.get("cells"))
        .unwrap_or(&Value::Null);
    let mut dto = TilemapResourceDto {
        resource_uri: resource_uri.to_owned(),
        absolute_path: display_path(&path),
        relative_path: relative_path(root, &path),
        schema_version: u32_at(&value, &["schema_version"]).unwrap_or(1),
        id: string_at(&value, &["id"]).unwrap_or_else(|| stem_id(&path)),
        label: string_at(&value, &["label"]).unwrap_or_else(|| stem_id(&path)),
        tileset_resource_uri,
        width: u32_at(grid, &["width"])
            .or_else(|| u32_at(&value, &["width"]))
            .unwrap_or(1),
        height: u32_at(grid, &["height"])
            .or_else(|| u32_at(&value, &["height"]))
            .unwrap_or(1),
        origin_offset_x: i32_at(&value, &["origin_offset", "x"]).unwrap_or(0),
        origin_offset_y: i32_at(&value, &["origin_offset", "y"]).unwrap_or(0),
        cells: sorted_tilemap_cells(tilemap_cells_from(cells_value)),
        diagnostics: Vec::new(),
    };
    if dto.width == 0 || dto.height == 0 {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Error,
            "invalid_tilemap_size",
            "Tilemap width and height must be greater than zero.",
            Some(dto.relative_path.clone()),
        ));
    }
    if dto.tileset_resource_uri.is_none() {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "tilemap_tileset_missing",
            "Tilemap does not declare a tileset.",
            Some(dto.relative_path.clone()),
        ));
    }
    Ok(dto)
}

pub fn load_tile_ruleset_resource(
    root: &Path,
    resource_uri: &str,
) -> Result<TileRulesetResourceDto, String> {
    let path = resolve_resource_path(root, resource_uri)?;
    let source = std::fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read tile ruleset resource `{}`: {error}",
            path.display()
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&source).map_err(|error| {
        format!(
            "failed to parse tile ruleset resource `{}`: {error}",
            path.display()
        )
    })?;
    let value = yaml_to_json(yaml);
    let kind = string_at(&value, &["kind"]).unwrap_or_default();
    if kind != "tile-ruleset-2d" {
        return Err(format!(
            "tile ruleset resource `{}` must be a `tile-ruleset-2d` descriptor",
            path.display()
        ));
    }

    let tile_size = value.get("tile_size").unwrap_or(&Value::Null);
    let terrains = tile_ruleset_terrains_from(value.get("terrains").unwrap_or(&Value::Null));
    let mut dto = TileRulesetResourceDto {
        resource_uri: resource_uri.to_owned(),
        absolute_path: display_path(&path),
        relative_path: relative_path(root, &path),
        schema_version: u32_at(&value, &["schema_version"]).unwrap_or(1),
        id: string_at(&value, &["id"]).unwrap_or_else(|| stem_id(&path)),
        label: string_at(&value, &["label"]).unwrap_or_else(|| stem_id(&path)),
        tile_width: u32_at(tile_size, &["width"])
            .or_else(|| u32_at(tile_size, &["x"]))
            .unwrap_or(128),
        tile_height: u32_at(tile_size, &["height"])
            .or_else(|| u32_at(tile_size, &["y"]))
            .unwrap_or(128),
        tileset_resource_uri: string_at(&value, &["tileset"])
            .map(|tileset_path| normalize_tileset_resource_uri(root, &path, &tileset_path))
            .or_else(|| infer_ruleset_tileset_resource_uri(root, &path)),
        terrains,
        diagnostics: Vec::new(),
    };

    if dto.tile_width == 0 || dto.tile_height == 0 {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Error,
            "invalid_ruleset_tile_size",
            "Ruleset tile size must be greater than zero.",
            Some(dto.relative_path.clone()),
        ));
    }
    if dto.terrains.is_empty() {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "ruleset_terrains_missing",
            "Ruleset does not declare any terrains.",
            Some(dto.relative_path.clone()),
        ));
    }
    if dto.tileset_resource_uri.is_none() {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "ruleset_tileset_missing",
            "Ruleset does not declare a tileset and no sibling tileset was inferred.",
            Some(dto.relative_path.clone()),
        ));
    }

    Ok(dto)
}

pub fn save_tilemap_resource(
    root: &Path,
    resource_uri: &str,
    dto: TilemapResourceDto,
) -> Result<TilemapResourceDto, String> {
    let path = resolve_write_resource_path(root, resource_uri)?;
    let cells = sorted_tilemap_cells(dto.cells)
        .into_iter()
        .map(|cell| {
            serde_json::json!({
                "x": cell.x,
                "y": cell.y,
                "tile": cell.tile_id,
            })
        })
        .collect::<Vec<_>>();
    let modern = serde_json::json!({
        "kind": "tilemap-2d",
        "schema_version": dto.schema_version.max(1),
        "id": dto.id,
        "label": dto.label,
        "tileset": dto.tileset_resource_uri,
        "grid": {
            "width": dto.width,
            "height": dto.height,
            "cell_size": { "width": 256, "height": 256 },
        },
        "origin_offset": {
            "x": dto.origin_offset_x,
            "y": dto.origin_offset_y,
        },
        "layers": {
            "ground": {
                "visible": true,
                "locked": false,
                "cells": cells,
            }
        },
    });
    let yaml = serde_yaml::to_string(&modern).map_err(|error| {
        format!(
            "failed to serialize tilemap resource `{}`: {error}",
            path.display()
        )
    })?;
    std::fs::write(&path, yaml).map_err(|error| {
        format!(
            "failed to save tilemap resource `{}`: {error}",
            path.display()
        )
    })?;
    load_tilemap_resource(root, resource_uri)
}

pub fn save_sheet_resource(
    root: &Path,
    resource_uri: &str,
    mut dto: SheetResourceDto,
) -> Result<SheetResourceDto, String> {
    let target_resource_uri = normalized_save_resource_uri(resource_uri);
    let path = resolve_write_resource_path(root, &target_resource_uri)?;
    dto.source_schema_kind = SheetSourceSchemaKind::Descriptor;
    dto.schema_version = 1;

    let size_key = match dto.kind {
        SheetKind::Tileset => "tile_size",
        SheetKind::Spritesheet => "frame_size",
    };
    let count_key = match dto.kind {
        SheetKind::Tileset => "tile_count",
        SheetKind::Spritesheet => "frame_count",
    };
    let mut atlas = serde_json::Map::new();
    atlas.insert(
        "image_size".to_owned(),
        serde_json::json!({
            "width": dto.declared_image_width.or(dto.image_width).unwrap_or(0),
            "height": dto.declared_image_height.or(dto.image_height).unwrap_or(0),
        }),
    );
    atlas.insert(
        size_key.to_owned(),
        serde_json::json!({ "width": dto.cell_width, "height": dto.cell_height }),
    );
    atlas.insert("columns".to_owned(), serde_json::json!(dto.columns));
    atlas.insert("rows".to_owned(), serde_json::json!(dto.rows));
    atlas.insert(count_key.to_owned(), serde_json::json!(dto.count));
    atlas.insert(
        "margin".to_owned(),
        serde_json::json!({ "x": dto.margin_x, "y": dto.margin_y }),
    );
    atlas.insert(
        "spacing".to_owned(),
        serde_json::json!({ "x": dto.spacing_x, "y": dto.spacing_y }),
    );
    atlas.insert(
        "indexing".to_owned(),
        serde_json::json!("row_major_0_based"),
    );

    let modern = serde_json::json!({
        "kind": match dto.kind {
            SheetKind::Tileset => "tileset-2d",
            SheetKind::Spritesheet => "sprite-sheet-2d",
        },
        "schema_version": 1,
        "id": dto.id,
        "label": dto.label,
        "source": {
            "file": dto.image_path,
        },
        "atlas": Value::Object(atlas),
        "defaults": dto.tileset.as_ref().map(|tileset| serde_json::json!({
            "collision": tileset.defaults.collision,
            "damageable": tileset.defaults.damageable,
        })).unwrap_or_else(|| serde_json::json!({ "collision": "solid", "damageable": true })),
        "tiles": dto.tileset.as_ref().map(|tileset| {
            let mut tiles = serde_json::Map::new();
            for tile in &tileset.tiles {
                tiles.insert(tile.id.to_string(), serde_json::json!({
                    "role": tile.role,
                    "name": tile.name,
                    "category": tile.category,
                    "collision": tile.collision,
                    "damageable": tile.damageable,
                    "tags": tile.tags,
                }));
            }
            Value::Object(tiles)
        }).unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        "animations": dto.animations.as_ref().map(|animations| {
            let mut map = serde_json::Map::new();
            for animation in animations {
                map.insert(animation.id.clone(), serde_json::json!({
                    "frames": animation.frames,
                    "fps": animation.fps,
                    "looping": animation.looping,
                }));
            }
            Value::Object(map)
        }).unwrap_or_else(|| Value::Object(serde_json::Map::new())),
    });

    let yaml = serde_yaml::to_string(&modern).map_err(|error| {
        format!(
            "failed to serialize sheet resource `{}`: {error}",
            path.display()
        )
    })?;
    std::fs::write(&path, yaml).map_err(|error| {
        format!(
            "failed to save sheet resource `{}`: {error}",
            path.display()
        )
    })?;
    load_sheet_resource(root, &target_resource_uri)
}

fn load_descriptor_sheet(
    root: &Path,
    path: &Path,
    resource_uri: &str,
    value: &Value,
) -> Result<SheetResourceDto, String> {
    let kind = string_at(value, &["kind"]).unwrap_or_default();
    if !matches!(
        kind.as_str(),
        "tileset-2d" | "sprite-sheet-2d" | "spritesheet-2d" | "sprite-2d"
    ) {
        return Err(format!(
            "sheet resource `{}` must be a typed descriptor (`tileset-2d` or `sprite-sheet-2d`)",
            path.display()
        ));
    }
    let parent_spritesheet = string_at(value, &["spritesheet"])
        .and_then(|spritesheet| load_related_spritesheet_value(root, path, &spritesheet).ok());
    let parent_value = parent_spritesheet.as_ref();
    let atlas = value
        .get("atlas")
        .or_else(|| value.get("grid"))
        .or_else(|| parent_value.and_then(|parent| parent.get("atlas")))
        .or_else(|| parent_value.and_then(|parent| parent.get("grid")))
        .unwrap_or(value);
    let image_path = string_at(value, &["source", "file"])
        .or_else(|| value.get("source").and_then(Value::as_str).map(ToOwned::to_owned))
        .or_else(|| parent_value.and_then(|parent| string_at(parent, &["source", "file"])))
        .or_else(|| parent_value.and_then(|parent| parent.get("source").and_then(Value::as_str).map(ToOwned::to_owned)))
        .unwrap_or_default();
    let image_owner_path = if parent_value.is_some() {
        parent_spritesheet_path(root, path, &string_at(value, &["spritesheet"]).unwrap_or_default())
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };
    let image_absolute_path = resolve_image_path(root, &image_owner_path, &image_path);

    Ok(SheetResourceDto {
        resource_uri: resource_uri.to_owned(),
        absolute_path: display_path(path),
        relative_path: relative_path(root, path),
        kind: sheet_kind_from(value),
        schema_version: u32_at(value, &["schema_version"]).unwrap_or(1),
        source_schema_kind: SheetSourceSchemaKind::Descriptor,
        id: string_at(value, &["id"]).unwrap_or_else(|| stem_id(path)),
        label: string_at(value, &["label"]).unwrap_or_else(|| stem_id(path)),
        image_path,
        image_absolute_path: display_path(&image_absolute_path),
        image_exists: image_absolute_path.exists(),
        image_width: None,
        image_height: None,
        declared_image_width: u32_at(atlas, &["image_size", "x"])
            .or_else(|| u32_at(atlas, &["image_size", "width"])),
        declared_image_height: u32_at(atlas, &["image_size", "y"])
            .or_else(|| u32_at(atlas, &["image_size", "height"])),
        cell_width: u32_at(atlas, &["cell_size", "x"])
            .or_else(|| u32_at(atlas, &["frame_size", "x"]))
            .or_else(|| u32_at(atlas, &["tile_size", "x"]))
            .or_else(|| u32_at(atlas, &["tile_size", "width"]))
            .or_else(|| u32_at(value, &["tile_size", "x"]))
            .or_else(|| u32_at(value, &["tile_size", "width"]))
            .or_else(|| u32_at(atlas, &["cell_size", "width"]))
            .or_else(|| u32_at(atlas, &["frame_size", "width"]))
            .unwrap_or(1),
        cell_height: u32_at(atlas, &["cell_size", "y"])
            .or_else(|| u32_at(atlas, &["frame_size", "y"]))
            .or_else(|| u32_at(atlas, &["tile_size", "y"]))
            .or_else(|| u32_at(atlas, &["tile_size", "height"]))
            .or_else(|| u32_at(value, &["tile_size", "y"]))
            .or_else(|| u32_at(value, &["tile_size", "height"]))
            .or_else(|| u32_at(atlas, &["cell_size", "height"]))
            .or_else(|| u32_at(atlas, &["frame_size", "height"]))
            .unwrap_or(1),
        columns: u32_at(atlas, &["columns"]).unwrap_or(1),
        rows: u32_at(atlas, &["rows"]).unwrap_or(1),
        count: u32_at(value, &["range", "count"])
            .or_else(|| u32_at(atlas, &["count"]))
            .or_else(|| u32_at(atlas, &["tile_count"]))
            .or_else(|| u32_at(atlas, &["frame_count"]))
            .unwrap_or_else(|| {
                u32_at(atlas, &["columns"]).unwrap_or(1) * u32_at(atlas, &["rows"]).unwrap_or(1)
            }),
        margin_x: u32_at(atlas, &["margin", "x"]).unwrap_or(0),
        margin_y: u32_at(atlas, &["margin", "y"]).unwrap_or(0),
        spacing_x: u32_at(atlas, &["spacing", "x"]).unwrap_or(0),
        spacing_y: u32_at(atlas, &["spacing", "y"]).unwrap_or(0),
        tileset: Some(TileSetPayloadDto {
            defaults: defaults_from(value.get("defaults").unwrap_or(&Value::Null)),
            tiles: tiles_from(value.get("tiles").unwrap_or(&Value::Null)),
        }),
        animations: animations_from(value.get("animations").unwrap_or(&Value::Null)),
        diagnostics: Vec::new(),
    })
}

fn add_image_info_and_diagnostics(dto: &mut SheetResourceDto) {
    if dto.image_path.trim().is_empty() || !dto.image_exists {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "sheet_image_missing",
            format!("Sheet image `{}` was not found.", dto.image_path),
            Some(dto.relative_path.clone()),
        ));
        return;
    }

    if let Ok((width, height)) = image::image_dimensions(&dto.image_absolute_path) {
        dto.image_width = Some(width);
        dto.image_height = Some(height);
        if dto
            .declared_image_width
            .is_some_and(|declared| declared != width)
            || dto
                .declared_image_height
                .is_some_and(|declared| declared != height)
        {
            dto.diagnostics.push(diagnostic(
                DiagnosticLevel::Warning,
                "image_size_mismatch",
                format!("Declared image size does not match actual image size {width}x{height}."),
                Some(dto.relative_path.clone()),
            ));
        }
        let grid_width = dto.margin_x
            + dto.columns.saturating_mul(dto.cell_width)
            + dto.columns.saturating_sub(1).saturating_mul(dto.spacing_x);
        let grid_height = dto.margin_y
            + dto.rows.saturating_mul(dto.cell_height)
            + dto.rows.saturating_sub(1).saturating_mul(dto.spacing_y);
        if grid_width > width || grid_height > height {
            dto.diagnostics.push(diagnostic(
                DiagnosticLevel::Error,
                "grid_exceeds_image_bounds",
                format!(
                    "Sheet grid {grid_width}x{grid_height} exceeds image bounds {width}x{height}."
                ),
                Some(dto.relative_path.clone()),
            ));
        }
    }

    if dto.cell_width == 0 || dto.cell_height == 0 {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Error,
            "invalid_cell_size",
            "Cell size must be greater than zero.",
            Some(dto.relative_path.clone()),
        ));
    }
    if dto.columns == 0 || dto.rows == 0 {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Error,
            "invalid_grid_size",
            "Grid columns and rows must be greater than zero.",
            Some(dto.relative_path.clone()),
        ));
    }
    if dto.count > dto.columns.saturating_mul(dto.rows) {
        dto.diagnostics.push(diagnostic(
            DiagnosticLevel::Warning,
            "tile_count_overflow",
            "Tile count is larger than columns * rows.",
            Some(dto.relative_path.clone()),
        ));
    }
}

fn tiles_from(value: &Value) -> Vec<TileMetadataDto> {
    match value {
        Value::Object(map) => map
            .iter()
            .filter_map(|(key, tile)| {
                let id = u32_at(tile, &["id"]).or_else(|| key.parse::<u32>().ok())?;
                Some(tile_from(key.clone(), id, tile))
            })
            .collect(),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| {
                let id = u32_at(tile, &["index"])
                    .or_else(|| u32_at(tile, &["id"]))
                    .unwrap_or(index as u32);
                Some(tile_from(id.to_string(), id, tile))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn animations_from(value: &Value) -> Option<Vec<SpriteAnimationDto>> {
    let animations = match value {
        Value::Object(map) => map
            .iter()
            .map(|(key, animation)| SpriteAnimationDto {
                id: key.clone(),
                frames: animation
                    .get("frames")
                    .and_then(Value::as_array)
                    .map(|frames| {
                        frames
                            .iter()
                            .filter_map(|frame| frame.as_u64().and_then(|value| u32::try_from(value).ok()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                fps: animation.get("fps").and_then(Value::as_f64).map(|value| value as f32),
                looping: animation.get("looping").and_then(Value::as_bool),
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    (!animations.is_empty()).then_some(animations)
}

fn tile_from(key: String, id: u32, value: &Value) -> TileMetadataDto {
    TileMetadataDto {
        key,
        id,
        role: string_at(value, &["role"]),
        name: string_at(value, &["name"]),
        category: string_at(value, &["category"]),
        collision: string_at(value, &["collision"]),
        damageable: bool_at(value, &["damageable"]),
        tags: string_vec_at(value, &["tags"]),
    }
}

fn defaults_from(value: &Value) -> TileSetDefaultsDto {
    TileSetDefaultsDto {
        collision: string_at(value, &["collision"])
            .or_else(|| string_at(value, &["solid"]))
            .unwrap_or_else(|| "solid".to_owned()),
        damageable: bool_at(value, &["damageable"]).unwrap_or(true),
    }
}

fn sheet_kind_from(value: &Value) -> SheetKind {
    let kind = string_at(value, &["kind"])
        .unwrap_or_default()
        .to_ascii_lowercase();
    if kind.contains("sprite") || kind.contains("atlas") {
        SheetKind::Spritesheet
    } else {
        SheetKind::Tileset
    }
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))?
        .as_str()
        .map(ToOwned::to_owned)
}

fn string_vec_at(value: &Value, path: &[&str]) -> Vec<String> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn i32_at(value: &Value, path: &[&str]) -> Option<i32> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn u32_at(value: &Value, path: &[&str]) -> Option<u32> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool)
}

fn tile_ruleset_terrains_from(value: &Value) -> Vec<TileRulesetTerrainDto> {
    let mut terrains = match value {
        Value::Object(map) => map
            .iter()
            .map(|(id, terrain)| TileRulesetTerrainDto {
                id: id.clone(),
                symbol: string_at(terrain, &["symbol"]).unwrap_or_else(|| id.chars().next().unwrap_or('#').to_string()),
                collision: string_at(terrain, &["collision"]),
                variants: tile_ruleset_variants_from(terrain.get("variants").unwrap_or(&Value::Null)),
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    terrains.sort_by(|left, right| left.id.cmp(&right.id));
    terrains
}

fn tile_ruleset_variants_from(value: &Value) -> TileRulesetVariantsDto {
    TileRulesetVariantsDto {
        single: u32_at(value, &["single"]),
        left_cap: u32_at(value, &["left_cap"]),
        middle: u32_at(value, &["middle"]),
        right_cap: u32_at(value, &["right_cap"]),
        side_left: u32_at(value, &["side_left"]),
        side_right: u32_at(value, &["side_right"]),
        center: u32_at(value, &["center"]),
        top_cap: u32_at(value, &["top_cap"]),
        bottom_cap: u32_at(value, &["bottom_cap"]),
        vertical_middle: u32_at(value, &["vertical_middle"]),
        outer_corner_top_left: u32_at(value, &["outer_corner_top_left"]),
        outer_corner_top_right: u32_at(value, &["outer_corner_top_right"]),
        outer_corner_bottom_left: u32_at(value, &["outer_corner_bottom_left"]),
        outer_corner_bottom_right: u32_at(value, &["outer_corner_bottom_right"]),
        inner_corner_top_left: u32_at(value, &["inner_corner_top_left"]),
        inner_corner_top_right: u32_at(value, &["inner_corner_top_right"]),
        inner_corner_bottom_left: u32_at(value, &["inner_corner_bottom_left"]),
        inner_corner_bottom_right: u32_at(value, &["inner_corner_bottom_right"]),
    }
}

fn tilemap_cells_from(value: &Value) -> Vec<TilemapCellDto> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|cell| {
                    Some(TilemapCellDto {
                        x: u32_at(cell, &["x"])?,
                        y: u32_at(cell, &["y"])?,
                        tile_id: u32_at(cell, &["tile"]).or_else(|| u32_at(cell, &["tile_id"]))?,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn sorted_tilemap_cells(mut cells: Vec<TilemapCellDto>) -> Vec<TilemapCellDto> {
    cells.sort_by_key(|cell| (cell.y, cell.x, cell.tile_id));
    cells
}

fn display_path(path: &Path) -> String {
    let display_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = display_path.display().to_string();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
}

fn resolve_resource_path(root: &Path, resource_uri: &str) -> Result<PathBuf, String> {
    let normalized = resource_uri.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains("../")
        || normalized == ".."
    {
        return Err(format!(
            "sheet resource URI `{resource_uri}` is not a safe project-relative path"
        ));
    }
    let candidate = root.join(normalized);
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize sheet resource `{}`: {error}",
            candidate.display()
        )
    })?;
    if !canonical_candidate.starts_with(canonical_root) {
        return Err(format!("sheet resource `{resource_uri}` escapes mod root"));
    }
    Ok(canonical_candidate)
}

fn resolve_write_resource_path(root: &Path, resource_uri: &str) -> Result<PathBuf, String> {
    let normalized = resource_uri.trim().replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.contains("../")
        || normalized == ".."
    {
        return Err(format!(
            "sheet resource URI `{resource_uri}` is not a safe project-relative path"
        ));
    }
    let candidate = root.join(normalized);
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize mod root `{}`: {error}",
            root.display()
        )
    })?;
    let parent = candidate
        .parent()
        .ok_or_else(|| format!("sheet resource `{resource_uri}` has no parent directory"))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create sheet resource parent `{}`: {error}",
            parent.display()
        )
    })?;
    let canonical_parent = parent.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize sheet resource parent `{}`: {error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(canonical_root) {
        return Err(format!("sheet resource `{resource_uri}` escapes mod root"));
    }
    Ok(candidate)
}

fn normalized_save_resource_uri(resource_uri: &str) -> String {
    resource_uri.trim().replace('\\', "/")
}

fn resolve_image_path(root: &Path, sheet_path: &Path, image_path: &str) -> PathBuf {
    let normalized = image_path.trim().replace('\\', "/");
    if normalized.is_empty() {
        return sheet_path.parent().unwrap_or(root).join("");
    }
    if normalized.contains('/') {
        let root_candidate = root.join(&normalized);
        if root_candidate.exists() {
            return root_candidate;
        }
    }
    sheet_path.parent().unwrap_or(root).join(normalized)
}

fn normalize_related_resource_uri(root: &Path, owner_path: &Path, related_path: &str) -> String {
    let normalized = related_path.trim().replace('\\', "/");
    let candidate = if normalized.contains('/') {
        let root_candidate = root.join(&normalized);
        if root_candidate.exists() {
            root_candidate
        } else {
            owner_path.parent().unwrap_or(root).join(&normalized)
        }
    } else {
        owner_path.parent().unwrap_or(root).join(&normalized)
    };
    relative_path(root, &candidate)
}

fn normalize_tileset_resource_uri(root: &Path, owner_path: &Path, tileset_ref: &str) -> String {
    let normalized = tileset_ref.trim().replace('\\', "/");
    if normalized.ends_with(".yml") || normalized.ends_with(".yaml") {
        return normalize_related_resource_uri(root, owner_path, &normalized);
    }

    if let Some(index) = normalized.find("spritesheets/") {
        let relative = &normalized[index..];
        let candidate = root.join(format!("{relative}.yml"));
        if candidate.exists() {
            return relative_path(root, &candidate);
        }
        let yaml_candidate = root.join(format!("{relative}.yaml"));
        if yaml_candidate.exists() {
            return relative_path(root, &yaml_candidate);
        }
        return format!("{relative}.yml");
    }

    if let Some(sheet_id) = spritesheet_family_id(owner_path) {
        let nested = normalized
            .strip_prefix(&format!("{sheet_id}/"))
            .unwrap_or(&normalized);
        let candidate = root
            .join("spritesheets")
            .join(&sheet_id)
            .join("tilesets")
            .join(format!("{nested}.yml"));
        if candidate.exists() {
            return relative_path(root, &candidate);
        }
    }

    normalized
}

fn spritesheet_family_id(path: &Path) -> Option<String> {
    let parts = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    parts
        .windows(2)
        .find_map(|window| (window[0] == "spritesheets").then(|| window[1].clone()))
}

fn infer_ruleset_tileset_resource_uri(root: &Path, ruleset_path: &Path) -> Option<String> {
    let file_name = ruleset_path.file_name()?.to_str()?;
    let asset_id = file_name
        .strip_suffix(".tile-ruleset.yml")
        .or_else(|| file_name.strip_suffix(".tile-ruleset.yaml"))
        .or_else(|| file_name.strip_suffix("-rules.yml"))
        .or_else(|| file_name.strip_suffix("-rules.yaml"))
        .unwrap_or(file_name)
        .trim_end_matches("-rules")
        .trim_end_matches("_rules");

    let candidates = [
        ruleset_path
            .parent()
            .unwrap_or(root)
            .join(format!("{asset_id}.tileset.yml")),
        root.join("assets")
            .join("tilesets")
            .join(format!("{asset_id}.tileset.yml")),
    ];

    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| relative_path(root, &candidate))
}

fn parent_spritesheet_path(root: &Path, owner_path: &Path, spritesheet_ref: &str) -> Option<PathBuf> {
    let normalized = spritesheet_ref.trim().replace('\\', "/");
    if normalized.ends_with("spritesheet.yml") || normalized.ends_with("spritesheet.yaml") {
        let related = normalize_related_resource_uri(root, owner_path, &normalized);
        return Some(root.join(related));
    }
    let id = normalized.rsplit('/').next().filter(|value| !value.is_empty())?;
    Some(root.join("spritesheets").join(id).join("spritesheet.yml"))
}

fn load_related_spritesheet_value(
    root: &Path,
    owner_path: &Path,
    spritesheet_ref: &str,
) -> Result<Value, String> {
    let path = parent_spritesheet_path(root, owner_path, spritesheet_ref)
        .ok_or_else(|| format!("invalid spritesheet reference `{spritesheet_ref}`"))?;
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read parent spritesheet `{}`: {error}", path.display()))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&source)
        .map_err(|error| format!("failed to parse parent spritesheet `{}`: {error}", path.display()))?;
    Ok(yaml_to_json(yaml))
}

fn relative_path(root: &Path, path: &Path) -> String {
    let comparable_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    path.strip_prefix(&comparable_root)
        .or_else(|_| path.strip_prefix(root))
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn stem_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("sheet")
        .to_owned()
}

fn diagnostic(
    level: DiagnosticLevel,
    code: &str,
    message: impl Into<String>,
    path: Option<String>,
) -> EditorDiagnosticDto {
    EditorDiagnosticDto {
        level,
        code: code.to_owned(),
        message: message.into(),
        path,
    }
}

fn yaml_to_json(value: serde_yaml::Value) -> Value {
    match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(value) => Value::Bool(value),
        serde_yaml::Value::Number(value) => {
            if let Some(value) = value.as_u64() {
                Value::Number(value.into())
            } else if let Some(value) = value.as_i64() {
                Value::Number(value.into())
            } else if let Some(value) = value.as_f64() {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(value) => Value::String(value),
        serde_yaml::Value::Sequence(items) => {
            Value::Array(items.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut object = serde_json::Map::new();
            for (key, value) in map {
                object.insert(yaml_key_to_string(key), yaml_to_json(value));
            }
            Value::Object(object)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

fn yaml_key_to_string(value: serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(value) => value,
        serde_yaml::Value::Number(value) => value.to_string(),
        serde_yaml::Value::Bool(value) => value.to_string(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use image::{ImageBuffer, Rgba};

    use super::{load_sheet_resource, load_tilemap_resource, save_tilemap_resource};
    #[test]
    fn loads_descriptor_tileset_schema() {
        let root = test_root("descriptor");
        write_png(root.join("raw/images/dirt.png"), 64, 64);
        fs::write(
            root.join("spritesheets/dirt/spritesheet.yml"),
            r#"
kind: spritesheet-2d
schema_version: 1
id: dirt
label: Dirt
source:
  file: ../../raw/images/dirt.png
  width: 64
  height: 64
grid:
  tile_size: { x: 16, y: 16 }
  columns: 4
  rows: 4
  frame_count: 16
  margin: { x: 0, y: 0 }
  spacing: { x: 0, y: 0 }
"#,
        )
        .unwrap();
        fs::write(
            root.join("spritesheets/dirt/tilesets/platform/base.yml"),
            r#"
kind: tileset-2d
schema_version: 1
id: platform/base
label: Dirt
spritesheet: dirt
range:
  start: 0
  count: 16
tile_size: { x: 16, y: 16 }
defaults:
  collision: solid
  damageable: true
tiles:
  0:
    role: empty
    collision: none
    damageable: false
    tags: []
  1:
    role: ground
    collision: solid
    tags: [ground]
"#,
        )
        .unwrap();

        let sheet = load_sheet_resource(&root, "spritesheets/dirt/tilesets/platform/base.yml").unwrap();

        assert_eq!(sheet.id, "platform/base");
        assert_eq!(sheet.image_width, Some(64));
        assert_eq!(sheet.cell_width, 16);
        assert_eq!(sheet.columns, 4);
        assert_eq!(sheet.tileset.unwrap().tiles.len(), 2);
        assert!(sheet.diagnostics.is_empty());
    }

    #[test]
    fn rejects_non_descriptor_sheet_schema() {
        let root = test_root("reject");
        fs::write(
            root.join("spritesheets/dirt/tilesets/platform/base.yml"),
            r#"
tileset:
  id: dirt
  label: Dirt
  tile_count: 16
  atlas:
    image: dirt.png
    image_size: { width: 64, height: 64 }
    tile_size: { width: 16, height: 16 }
    columns: 4
    rows: 4
defaults:
  collision: full
  damageable: true
tiles:
  - { id: DIRT_00, index: 0, name: dirt_full_a, category: full, tags: [solid], collision: full }
  - { id: DIRT_01, index: 1, name: dirt_full_b, category: full, tags: [solid], collision: full }
"#,
        )
        .unwrap();

        let error = load_sheet_resource(&root, "spritesheets/dirt/tilesets/platform/base.yml")
            .expect_err("legacy sheet descriptors are no longer an active editor path");
        assert!(error.contains("typed descriptor"));
    }

    #[test]
    fn loads_and_saves_tilemap_schema() {
        let root = test_root("tilemap");
        fs::write(
            root.join("data/tilemaps/map.tilemap.yml"),
            r#"
kind: tilemap-2d
schema_version: 1
id: map
label: Test Map
tileset: ink-wars/spritesheets/dirt/tilesets/platform/base
grid:
  width: 3
  height: 2
  cell_size: { width: 16, height: 16 }
origin_offset: { x: 1, y: -1 }
layers:
  ground:
    visible: true
    locked: false
    cells:
      - { x: 0, y: 0, tile: 4 }
      - { x: 1, y: 0, tile: 5 }
"#,
        )
        .unwrap();
        fs::write(
            root.join("spritesheets/dirt/spritesheet.yml"),
            "kind: spritesheet-2d\nschema_version: 1\nid: dirt\nsource: { file: ../../raw/images/dirt.png }\ngrid: { tile_size: { x: 1, y: 1 }, columns: 1, rows: 1, frame_count: 1 }\n",
        )
        .unwrap();
        fs::write(
            root.join("spritesheets/dirt/tilesets/platform/base.yml"),
            "kind: tileset-2d\nschema_version: 1\nid: platform/base\nspritesheet: dirt\nrange: { start: 0, count: 1 }\ntile_size: { x: 1, y: 1 }\n",
        )
        .unwrap();

        let mut tilemap = load_tilemap_resource(&root, "data/tilemaps/map.tilemap.yml").unwrap();
        assert_eq!(tilemap.width, 3);
        assert_eq!(tilemap.height, 2);
        assert_eq!(
            tilemap.tileset_resource_uri.as_deref(),
            Some("spritesheets/dirt/tilesets/platform/base.yml")
        );
        assert_eq!(tilemap.cells.len(), 2);

        tilemap.cells.push(crate::sheet::dto::TilemapCellDto {
            x: 2,
            y: 1,
            tile_id: 6,
        });
        tilemap.cells.push(crate::sheet::dto::TilemapCellDto {
            x: 0,
            y: 1,
            tile_id: 2,
        });
        let saved =
            save_tilemap_resource(&root, "data/tilemaps/map.tilemap.yml", tilemap).unwrap();
        assert_eq!(saved.cells.len(), 4);
        assert_eq!(
            saved
                .cells
                .iter()
                .map(|cell| (cell.x, cell.y, cell.tile_id))
                .collect::<Vec<_>>(),
            vec![(0, 0, 4), (1, 0, 5), (0, 1, 2), (2, 1, 6)]
        );
        assert!(
            saved
                .cells
                .iter()
                .any(|cell| cell.x == 2 && cell.y == 1 && cell.tile_id == 6)
        );
    }

    fn test_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("amigo-editor-sheet-test-{name}-{stamp}"));
        fs::create_dir_all(root.join("raw/images")).unwrap();
        fs::create_dir_all(root.join("spritesheets/dirt/tilesets/platform")).unwrap();
        fs::create_dir_all(root.join("data/tilemaps")).unwrap();
        root
    }

    fn write_png(path: PathBuf, width: u32, height: u32) {
        let image =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        image.save(path).unwrap();
    }
}
