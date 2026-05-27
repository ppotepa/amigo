use amigo_render_api::{
    RenderPrimitive2d, Renderable2dCommon, Renderable2dItem, Renderable2dKind, TileMap2dPrimitive,
    TileMapResolved2dPrimitive, TileMapResolvedTile2dPrimitive,
};

use crate::tilemap::TileMap2dDrawCommand;

pub fn tilemap_draw_command_to_render_primitive(
    command: &TileMap2dDrawCommand,
) -> RenderPrimitive2d {
    RenderPrimitive2d::TileBatch(TileMap2dPrimitive {
        tileset: command.tilemap.tileset.clone(),
        tile_size: command.tilemap.tile_size,
        grid: command.tilemap.grid.clone(),
        origin_offset: command.tilemap.origin_offset,
        resolved: command
            .tilemap
            .resolved
            .as_ref()
            .map(|resolved| TileMapResolved2dPrimitive {
                rows: resolved
                    .rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|tile| TileMapResolvedTile2dPrimitive {
                                symbol: tile.symbol,
                                tile_id: tile.tile_id,
                            })
                            .collect()
                    })
                    .collect(),
            }),
    })
}

pub fn tilemap_draw_command_to_renderable_2d(command: &TileMap2dDrawCommand) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "TileMap2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::TileMap,
        ),
        tilemap_draw_command_to_render_primitive(command),
    )
}
