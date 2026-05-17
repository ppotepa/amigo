use crate::renderer::*;

pub(crate) fn resolve_transform2(
    scene: &SceneService,
    entity_name: &str,
    fallback: Transform2,
) -> Transform2 {
    scene
        .transform_of(entity_name)
        .map(transform2_from_transform3)
        .unwrap_or(fallback)
}

pub(crate) fn resolve_transform3(
    scene: &SceneService,
    entity_name: &str,
    fallback: Transform3,
) -> Transform3 {
    scene.transform_of(entity_name).unwrap_or(fallback)
}

pub(crate) fn resolve_camera_transform(scene: &SceneService) -> Transform3 {
    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("3d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() > 0.01)
        })
        .map(|entity| entity.transform)
        .unwrap_or(Transform3 {
            translation: Vec3::new(0.0, 0.0, 6.0),
            ..Transform3::default()
        })
}

pub(crate) fn resolve_camera2d_transform(
    scene: &SceneService,
    active_camera_entity: Option<&str>,
) -> Transform2 {
    if let Some(active_camera_entity) = active_camera_entity {
        if let Some(entity) = scene
            .entities()
            .into_iter()
            .find(|entity| entity.name == active_camera_entity)
        {
            return Transform2 {
                translation: Vec2 {
                    x: entity.transform.translation.x,
                    y: entity.transform.translation.y,
                },
                rotation_radians: entity.transform.rotation_euler.z,
                scale: Vec2 {
                    x: entity.transform.scale.x,
                    y: entity.transform.scale.y,
                },
            };
        }
    }

    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("2d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() <= 0.01)
        })
        .map(|entity| Transform2 {
            translation: Vec2 {
                x: entity.transform.translation.x,
                y: entity.transform.translation.y,
            },
            rotation_radians: entity.transform.rotation_euler.z,
            scale: Vec2 {
                x: entity.transform.scale.x,
                y: entity.transform.scale.y,
            },
        })
        .unwrap_or_default()
}

pub(crate) fn material_lookup_from_commands(
    materials: &[MaterialDrawCommand],
) -> BTreeMap<String, ColorRgba> {
    materials
        .iter()
        .cloned()
        .map(|command| (command.entity_name, command.material.albedo))
        .collect()
}

#[derive(Clone)]
pub(crate) struct TileSetRenderInfo {
    pub(crate) tile_size: Vec2,
    pub(crate) columns: u32,
    pub(crate) ground_tile_id: u32,
    pub(crate) platform_tile_id: Option<u32>,
    pub(crate) derived_tiles: BTreeMap<u32, DerivedTileRenderInfo>,
}

#[derive(Clone, Copy)]
pub(crate) struct DerivedTileRenderInfo {
    pub(crate) source_tile_id: u32,
    pub(crate) crop: TileCropRect,
}

#[derive(Clone, Copy)]
pub(crate) struct TileCropRect {
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
}

pub(crate) fn world2d_sort_key(
    item: &World2dItem,
    render_layers: &BTreeMap<String, RenderLayer2dCommand>,
) -> (i32, i32, u8) {
    let layer_order = render_layers
        .get(item.render_layer())
        .map(|layer| layer.order)
        .unwrap_or(0.0);
    let priority = match item {
        World2dItem::TileMap(_) => 0,
        World2dItem::LayeredImage(_) => 1,
        World2dItem::Vector(_) => 2,
        World2dItem::Beacon(_) => 3,
        World2dItem::Particle(_) => 4,
        World2dItem::Sprite(_) => 5,
    };
    (
        (layer_order * 1000.0).round() as i32,
        (item.z_index() * 1000.0).round() as i32,
        priority,
    )
}

pub(crate) fn render_layer_lookup(
    render_layers: &[RenderLayer2dCommand],
) -> BTreeMap<String, RenderLayer2dCommand> {
    render_layers
        .iter()
        .cloned()
        .map(|layer| (layer.id.clone(), layer))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_2d_sprite::{Sprite, SpriteDrawCommand};
    use amigo_assets::AssetKey;
    use amigo_scene::SceneEntityId;

    fn sprite_item(render_layer: &str, z_index: f32) -> World2dItem {
        World2dItem::Sprite(SpriteDrawCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: format!("sprite-{render_layer}"),
            sprite: Sprite {
                texture: AssetKey::new("test/sprite"),
                size: Vec2::new(16.0, 16.0),
                sheet: None,
                sheet_is_explicit: false,
                animation_override: None,
                frame_index: 0,
                frame_elapsed: 0.0,
            },
            render_layer: render_layer.to_owned(),
            z_index,
            transform: Transform2::default(),
        })
    }

    #[test]
    fn world2d_sort_key_uses_render_layer_order_before_z_index() {
        let layers = render_layer_lookup(&[
            RenderLayer2dCommand {
                source_mod: "test".to_owned(),
                id: "background.city".to_owned(),
                label: None,
                order: -100.0,
                visible: true,
                opacity: 1.0,
                depth: amigo_2d_composition::RenderDepth2d::default(),
            },
            RenderLayer2dCommand {
                source_mod: "test".to_owned(),
                id: "weather.rain.near".to_owned(),
                label: None,
                order: -14.0,
                visible: true,
                opacity: 1.0,
                depth: amigo_2d_composition::RenderDepth2d::default(),
            },
        ]);

        let background = sprite_item("background.city", 999.0);
        let near_rain = sprite_item("weather.rain.near", 0.0);

        assert!(world2d_sort_key(&background, &layers) < world2d_sort_key(&near_rain, &layers));
    }
}
