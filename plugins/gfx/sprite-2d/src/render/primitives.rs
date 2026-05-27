use amigo_material_api::MaterialCoverageKind2d;
use amigo_render_api::{
    RenderMaterialBinding2d, RenderPrimitive2d, Renderable2dCommon, Renderable2dItem,
    Renderable2dKind, TexturedQuad2dPrimitive, TexturedQuad2dSheet, VisualMaps2dPrimitive,
};

use crate::sprite::{SpriteDrawCommand, SpriteSheet};

fn sprite_sheet_to_primitive(sheet: SpriteSheet) -> TexturedQuad2dSheet {
    TexturedQuad2dSheet {
        columns: sheet.columns,
        rows: sheet.rows,
        frame_count: sheet.frame_count,
        frame_size: sheet.frame_size,
    }
}

fn visual_maps_to_primitive(maps: &amigo_scene::VisualMaps2dSceneCommand) -> VisualMaps2dPrimitive {
    VisualMaps2dPrimitive {
        normal: maps.normal.clone(),
        wetness: maps.wetness.clone(),
        emissive: maps.emissive.clone(),
        highlight: maps.highlight.clone(),
    }
}

pub fn sprite_draw_command_to_render_primitive(command: &SpriteDrawCommand) -> RenderPrimitive2d {
    RenderPrimitive2d::TexturedQuad(TexturedQuad2dPrimitive {
        texture: command.sprite.texture.clone(),
        size: command.sprite.size,
        transform: command.transform,
        sheet: command.sprite.sheet.map(sprite_sheet_to_primitive),
        frame_index: command.sprite.frame_index,
        visual_maps: command
            .sprite
            .visual_maps
            .as_ref()
            .map(visual_maps_to_primitive),
        material: RenderMaterialBinding2d::new(
            command.material,
            command.render_contributions.clone(),
            MaterialCoverageKind2d::TextureAlpha,
        ),
    })
}

pub fn sprite_draw_command_to_renderable_2d(command: &SpriteDrawCommand) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "Sprite2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::Sprite,
        ),
        sprite_draw_command_to_render_primitive(command),
    )
}
