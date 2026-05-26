use crate::renderer::*;
use amigo_render_api::RenderSceneView;

pub(crate) fn resolve_transform3(
    scene_view: &RenderSceneView,
    entity_name: &str,
    default_transform: Transform3,
) -> Transform3 {
    scene_view
        .transform3_of(entity_name)
        .unwrap_or(default_transform)
}

pub(crate) fn resolve_camera_transform(scene_view: &RenderSceneView) -> Transform3 {
    scene_view.camera_3d()
}

pub(crate) fn resolve_camera2d_transform(scene_view: &RenderSceneView) -> Transform2 {
    scene_view.camera_2d()
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
    item: &Renderable2dItem,
    render_layers: &BTreeMap<String, RenderLayer2dCommand>,
) -> (i32, i32, u8) {
    let layer_order = render_layers
        .get(item.render_layer())
        .map(|layer| layer.order)
        .unwrap_or(0.0);
    let priority = item.common.kind.sort_priority();
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
    use amigo_assets::AssetKey;
    use amigo_material_api::MaterialCoverageKind2d;
    use amigo_render_api::{
        GlyphRun2dBlendMode, GlyphRun2dPrimitive, RenderMaterialBinding2d, RenderPrimitive2d,
        RenderSpace2d, Renderable2dCommon, Renderable2dKind, TexturedQuad2dPrimitive,
    };

    fn sprite_item(render_layer: &str, z_index: f32) -> Renderable2dItem {
        Renderable2dItem::new(
            Renderable2dCommon {
                source_id: amigo_render_api::RenderSourceId::for_component(
                    &format!("sprite-{render_layer}"),
                    "component",
                ),
                object_id: amigo_render_api::RenderObjectId::for_scene_object(&format!(
                    "sprite-{render_layer}"
                )),
                owner_entity: format!("sprite-{render_layer}"),
                component_kind: "component".to_owned(),
                render_space: RenderSpace2d::World,
                render_layer: render_layer.to_owned(),
                z_index,
                kind: Renderable2dKind::Sprite,
                overlay_visible: false,
            },
            RenderPrimitive2d::TexturedQuad(TexturedQuad2dPrimitive {
                texture: AssetKey::new("test/sprite"),
                size: Vec2::new(16.0, 16.0),
                transform: Transform2::default(),
                sheet: None,
                frame_index: 0,
                visual_maps: None,
                material: RenderMaterialBinding2d::none(MaterialCoverageKind2d::TextureAlpha),
            }),
        )
    }

    fn text_item(render_layer: &str, z_index: f32) -> Renderable2dItem {
        Renderable2dItem::new(
            Renderable2dCommon {
                source_id: amigo_render_api::RenderSourceId::for_component(
                    &format!("text-{render_layer}"),
                    "component",
                ),
                object_id: amigo_render_api::RenderObjectId::for_scene_object(&format!(
                    "text-{render_layer}"
                )),
                owner_entity: format!("text-{render_layer}"),
                component_kind: "component".to_owned(),
                render_space: RenderSpace2d::World,
                render_layer: render_layer.to_owned(),
                z_index,
                kind: Renderable2dKind::Text,
                overlay_visible: false,
            },
            RenderPrimitive2d::GlyphRun(GlyphRun2dPrimitive {
                font: AssetKey::new("test/font"),
                text: "title".to_owned(),
                bounds: Vec2::new(200.0, 60.0),
                transform: Transform2::default(),
                color: ColorRgba::WHITE,
                font_size: Some(24.0),
                blend: GlyphRun2dBlendMode::Alpha,
                shadow: None,
                outline: None,
                glow: None,
                material: RenderMaterialBinding2d::none(MaterialCoverageKind2d::Glyphs),
            }),
        )
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
                depth: amigo_render_api::RenderDepth2d::default(),
                optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
            },
            RenderLayer2dCommand {
                source_mod: "test".to_owned(),
                id: "weather.rain.near".to_owned(),
                label: None,
                order: -14.0,
                visible: true,
                opacity: 1.0,
                depth: amigo_render_api::RenderDepth2d::default(),
                optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
            },
        ]);

        let background = sprite_item("background.city", 999.0);
        let near_rain = sprite_item("weather.rain.near", 0.0);

        assert!(world2d_sort_key(&background, &layers) < world2d_sort_key(&near_rain, &layers));
    }

    #[test]
    fn world2d_sort_key_orders_renderable_text_by_layer_order_and_z_index() {
        let layers = render_layer_lookup(&[
            RenderLayer2dCommand {
                source_mod: "test".to_owned(),
                id: "title.depth2d".to_owned(),
                label: None,
                order: 20.0,
                visible: true,
                opacity: 1.0,
                depth: amigo_render_api::RenderDepth2d::default(),
                optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
            },
            RenderLayer2dCommand {
                source_mod: "test".to_owned(),
                id: "weather.rain.near".to_owned(),
                label: None,
                order: 35.0,
                visible: true,
                opacity: 1.0,
                depth: amigo_render_api::RenderDepth2d::default(),
                optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
            },
        ]);

        let title_low = text_item("title.depth2d", 0.0);
        let title_high = text_item("title.depth2d", 10.0);
        let near_rain = sprite_item("weather.rain.near", 0.0);

        assert!(world2d_sort_key(&title_low, &layers) < world2d_sort_key(&title_high, &layers));
        assert!(world2d_sort_key(&title_high, &layers) < world2d_sort_key(&near_rain, &layers));
    }
}
