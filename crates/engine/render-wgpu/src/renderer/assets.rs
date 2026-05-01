use crate::renderer::*;

pub(crate) fn resolve_image_path(prepared: &PreparedAsset) -> Option<PathBuf> {
    let extension = prepared
        .resolved_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());
    match extension.as_deref() {
        Some("png" | "jpg" | "jpeg" | "webp") => Some(prepared.resolved_path.clone()),
        _ => prepared.metadata.get("image").and_then(|image| {
            prepared
                .resolved_path
                .parent()
                .map(|parent| parent.join(image))
        }),
    }
}

pub(crate) fn infer_sprite_sheet_from_asset(prepared: &PreparedAsset) -> Option<SpriteSheet> {
    if !matches!(prepared.kind, PreparedAssetKind::SpriteSheet2d) {
        return None;
    }

    let columns = metadata_u32(prepared, "columns")?.max(1);
    let rows = metadata_u32(prepared, "rows")?.max(1);
    let frame_width = metadata_f32(prepared, "frame_size.x")?;
    let frame_height = metadata_f32(prepared, "frame_size.y")?;
    Some(SpriteSheet {
        columns,
        rows,
        frame_count: metadata_u32(prepared, "frame_count")
            .unwrap_or(columns.saturating_mul(rows))
            .max(1),
        frame_size: Vec2::new(frame_width, frame_height),
        fps: metadata_f32(prepared, "fps").unwrap_or(0.0),
        looping: prepared
            .metadata
            .get("looping")
            .and_then(|value| value.parse::<bool>().ok())
            .unwrap_or(true),
    })
}

pub(crate) fn infer_tileset_from_asset(
    prepared: &PreparedAsset,
    fallback_tile_size: Vec2,
) -> Option<TileSetRenderInfo> {
    if !matches!(prepared.kind, PreparedAssetKind::TileSet2d) {
        return None;
    }

    let columns = metadata_u32(prepared, "columns")?.max(1);
    let _rows = metadata_u32(prepared, "rows")?.max(1);
    let tile_size = Vec2::new(
        metadata_f32(prepared, "tile_size.x").unwrap_or(fallback_tile_size.x),
        metadata_f32(prepared, "tile_size.y").unwrap_or(fallback_tile_size.y),
    );
    let tile_ids = infer_tileset_tile_ids(prepared);
    Some(TileSetRenderInfo {
        tile_size,
        columns,
        ground_tile_id: metadata_u32(prepared, "tiles.ground.id")
            .or_else(|| metadata_u32(prepared, "tiles.ground_single.id"))
            .unwrap_or(1),
        platform_tile_id: metadata_u32(prepared, "tiles.platform.id")
            .or_else(|| metadata_u32(prepared, "tiles.ground_middle.id")),
        derived_tiles: infer_derived_tile_render_info(prepared, &tile_ids),
    })
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared.metadata.get(key)?.parse().ok()
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse().ok()
}

pub(crate) fn metadata_bool(prepared: &PreparedAsset, key: &str) -> bool {
    prepared
        .metadata
        .get(key)
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false)
}

pub(crate) fn sprite_uv_rect(
    texture_size: Vec2,
    sheet: Option<SpriteSheet>,
    frame_index: u32,
) -> TextureUvRect {
    let Some(sheet) = sheet else {
        return TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        };
    };

    let columns = sheet.columns.max(1);
    let rows = sheet.rows.max(1);
    let frame = frame_index.min(sheet.visible_frame_count().saturating_sub(1));
    let column = frame % columns;
    let row = frame / columns;
    let frame_width = if sheet.frame_size.x > 0.0 {
        sheet.frame_size.x
    } else {
        texture_size.x / columns as f32
    };
    let frame_height = if sheet.frame_size.y > 0.0 {
        sheet.frame_size.y
    } else {
        texture_size.y / rows as f32
    };
    let u0 = (column as f32 * frame_width) / texture_size.x.max(1.0);
    let v0 = (row as f32 * frame_height) / texture_size.y.max(1.0);
    let u1 = ((column as f32 + 1.0) * frame_width) / texture_size.x.max(1.0);
    let v1 = ((row as f32 + 1.0) * frame_height) / texture_size.y.max(1.0);
    TextureUvRect { u0, v0, u1, v1 }
}

fn infer_tileset_tile_ids(prepared: &PreparedAsset) -> BTreeMap<String, u32> {
    prepared
        .metadata
        .iter()
        .filter_map(|(key, value)| {
            let tile_name = key.strip_prefix("tiles.")?.strip_suffix(".id")?;
            value
                .parse::<u32>()
                .ok()
                .map(|id| (tile_name.to_owned(), id))
        })
        .collect()
}

fn infer_derived_tile_render_info(
    prepared: &PreparedAsset,
    tile_ids: &BTreeMap<String, u32>,
) -> BTreeMap<u32, DerivedTileRenderInfo> {
    let mut variant_names = Vec::new();
    for key in prepared.metadata.keys() {
        if let Some(rest) = key.strip_prefix("derived_variants.") {
            if let Some((variant_name, _)) = rest.split_once('.') {
                if !variant_names
                    .iter()
                    .any(|name: &String| name == variant_name)
                {
                    variant_names.push(variant_name.to_owned());
                }
            }
        }
    }

    let mut derived_tiles = BTreeMap::new();
    for variant_name in variant_names {
        let target_id = tile_ids
            .get(&variant_name)
            .copied()
            .or_else(|| metadata_u32(prepared, &format!("derived_variants.{variant_name}.id")));
        let source_tile_id = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.from_tile"))
            .and_then(|source_name| tile_ids.get(source_name))
            .copied()
            .or_else(|| {
                metadata_u32(
                    prepared,
                    &format!("derived_variants.{variant_name}.from_id"),
                )
            });
        let mode = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.mode"))
            .map(String::as_str);
        let segment = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.segment"))
            .or_else(|| {
                prepared
                    .metadata
                    .get(&format!("derived_variants.{variant_name}.side"))
            })
            .map(String::as_str);

        let crop = match (mode, segment) {
            (Some("split_x"), Some("left")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.0,
                x1: 0.5,
                y1: 1.0,
            }),
            (Some("split_x"), Some("right")) => Some(TileCropRect {
                x0: 0.5,
                y0: 0.0,
                x1: 1.0,
                y1: 1.0,
            }),
            (Some("split_y"), Some("top")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.0,
                x1: 1.0,
                y1: 0.5,
            }),
            (Some("split_y"), Some("bottom")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.5,
                x1: 1.0,
                y1: 1.0,
            }),
            _ => None,
        };

        if let (Some(target_id), Some(source_tile_id), Some(crop)) =
            (target_id, source_tile_id, crop)
        {
            derived_tiles.insert(
                target_id,
                DerivedTileRenderInfo {
                    source_tile_id,
                    crop,
                },
            );
        }
    }

    derived_tiles
}

fn atlas_tile_uv_rect(
    texture_size: Vec2,
    tileset: &TileSetRenderInfo,
    tile_id: u32,
) -> TextureUvRect {
    let column = tile_id % tileset.columns;
    let row = tile_id / tileset.columns;
    let tile_width = tileset.tile_size.x.max(1.0);
    let tile_height = tileset.tile_size.y.max(1.0);
    let u0 = (column as f32 * tile_width) / texture_size.x.max(1.0);
    let v0 = (row as f32 * tile_height) / texture_size.y.max(1.0);
    let u1 = ((column as f32 + 1.0) * tile_width) / texture_size.x.max(1.0);
    let v1 = ((row as f32 + 1.0) * tile_height) / texture_size.y.max(1.0);
    TextureUvRect { u0, v0, u1, v1 }
}

fn inset_uv_rect(texture_size: Vec2, uv: TextureUvRect, inset_pixels: f32) -> TextureUvRect {
    let width = (uv.u1 - uv.u0).max(0.0);
    let height = (uv.v1 - uv.v0).max(0.0);
    let inset_u = (inset_pixels / texture_size.x.max(1.0)).min(width * 0.25);
    let inset_v = (inset_pixels / texture_size.y.max(1.0)).min(height * 0.25);
    TextureUvRect {
        u0: uv.u0 + inset_u,
        v0: uv.v0 + inset_v,
        u1: uv.u1 - inset_u,
        v1: uv.v1 - inset_v,
    }
}

pub(crate) fn tile_uv_rect(texture_size: Vec2, tileset: &TileSetRenderInfo, tile_id: u32) -> TextureUvRect {
    let uv = if let Some(derived) = tileset.derived_tiles.get(&tile_id).copied() {
        let base = atlas_tile_uv_rect(texture_size, tileset, derived.source_tile_id);
        let du = base.u1 - base.u0;
        let dv = base.v1 - base.v0;
        TextureUvRect {
            u0: base.u0 + du * derived.crop.x0,
            v0: base.v0 + dv * derived.crop.y0,
            u1: base.u0 + du * derived.crop.x1,
            v1: base.v0 + dv * derived.crop.y1,
        }
    } else {
        atlas_tile_uv_rect(texture_size, tileset, tile_id)
    };

    inset_uv_rect(texture_size, uv, 0.5)
}

pub(crate) fn tile_id_for_symbol(symbol: char, tileset: &TileSetRenderInfo) -> Option<u32> {
    match symbol {
        '#' => Some(tileset.ground_tile_id),
        '=' => tileset.platform_tile_id.or(Some(tileset.ground_tile_id)),
        _ => None,
    }
}

