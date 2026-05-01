use crate::renderer::*;

pub(crate) fn append_textured_sprite_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    size: Vec2,
    uv: TextureUvRect,
) {
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let points = [
        transform_point_2d(Vec2::new(-half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, half.y), transform),
        transform_point_2d(Vec2::new(-half.x, half.y), transform),
    ];
    push_textured_quad(
        vertices,
        ndc_from_world_2d(points[0], camera, viewport),
        ndc_from_world_2d(points[1], camera, viewport),
        ndc_from_world_2d(points[2], camera, viewport),
        ndc_from_world_2d(points[3], camera, viewport),
        uv,
        ColorRgba::WHITE,
    );
}

pub(crate) fn append_textured_tilemap_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    tilemap: &TileMap2d,
    texture_size: Vec2,
    tileset: &TileSetRenderInfo,
) {
    let row_count = tilemap.grid.len();
    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        let row: &str = row;
        for (column_index, symbol) in row.chars().enumerate() {
            let tile_id = if let Some(resolved) = &tilemap.resolved {
                resolved
                    .rows
                    .get(row_index)
                    .and_then(|row| row.get(column_index))
                    .and_then(|tile| tile.tile_id)
            } else {
                None
            }
            .or_else(|| tile_id_for_symbol(symbol, tileset));
            let Some(tile_id) = tile_id else {
                continue;
            };
            let uv = tile_uv_rect(texture_size, tileset, tile_id);
            let min = Vec2::new(
                column_index as f32 * tilemap.tile_size.x,
                row_from_bottom as f32 * tilemap.tile_size.y,
            );
            let max = Vec2::new(min.x + tilemap.tile_size.x, min.y + tilemap.tile_size.y);
            let points = [
                transform_point_2d(min, transform),
                transform_point_2d(Vec2::new(max.x, min.y), transform),
                transform_point_2d(max, transform),
                transform_point_2d(Vec2::new(min.x, max.y), transform),
            ];
            push_textured_quad(
                vertices,
                ndc_from_world_2d_snapped(points[0], camera, viewport),
                ndc_from_world_2d_snapped(points[1], camera, viewport),
                ndc_from_world_2d_snapped(points[2], camera, viewport),
                ndc_from_world_2d_snapped(points[3], camera, viewport),
                uv,
                ColorRgba::WHITE,
            );
        }
    }
}

pub(crate) fn append_tilemap_fallback_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    tilemap: &TileMap2d,
) {
    let row_count = tilemap.grid.len();
    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        let row: &str = row;
        for (column_index, symbol) in row.chars().enumerate() {
            if symbol != '#' && symbol != '=' {
                continue;
            }
            let min = Vec2::new(
                column_index as f32 * tilemap.tile_size.x,
                row_from_bottom as f32 * tilemap.tile_size.y,
            );
            let max = Vec2::new(min.x + tilemap.tile_size.x, min.y + tilemap.tile_size.y);
            let points = [
                transform_point_2d(min, transform),
                transform_point_2d(Vec2::new(max.x, min.y), transform),
                transform_point_2d(max, transform),
                transform_point_2d(Vec2::new(min.x, max.y), transform),
            ];
            push_quad(
                vertices,
                ndc_from_world_2d_snapped(points[0], camera, viewport),
                ndc_from_world_2d_snapped(points[1], camera, viewport),
                ndc_from_world_2d_snapped(points[2], camera, viewport),
                ndc_from_world_2d_snapped(points[3], camera, viewport),
                ColorRgba::new(0.28, 0.31, 0.38, 1.0),
            );
        }
    }
}

