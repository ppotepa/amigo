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

pub(crate) fn resolve_camera2d_transform(scene: &SceneService) -> Transform2 {
    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("2d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() <= 0.01)
        })
        .map(|entity| transform2_from_transform3(entity.transform))
        .unwrap_or_default()
}

pub(crate) fn material_lookup_from_commands(materials: &[MaterialDrawCommand]) -> BTreeMap<String, ColorRgba> {
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

pub(crate) fn world2d_sort_key(item: &World2dItem) -> (f32, u8) {
    match item {
        World2dItem::TileMap(command) => (command.z_index, 0),
        World2dItem::Vector(command) => (command.z_index, 1),
        World2dItem::Particle(command) => (command.z_index, 2),
        World2dItem::Sprite(command) => (command.z_index, 3),
    }
}

