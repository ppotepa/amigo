use std::collections::{BTreeMap, BTreeSet};

use super::WorldPassLoadExt;
use super::world_filters::WorldPassLoad;
use super::*;
use amigo_core::AmigoResult;
use amigo_render_api::RenderAssetSource;

pub(super) fn execute_layered_image_parts_to_offscreen(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    renderables: &[Renderable2dItem],
    assets: &dyn RenderAssetSource,
    render_layers: &[RenderLayer2dCommand],
    part_targets: &BTreeMap<String, BTreeSet<String>>,
    pass_load: WorldPassLoad,
) -> AmigoResult<()> {
    let viewport = Viewport::from_offscreen(target);
    let render_layer_lookup = render_layer_lookup(render_layers);
    let mut texture_batches = Vec::new();

    let mut items = renderables
        .iter()
        .filter_map(|item| {
            item.primitive
                .layered_textured_quads()
                .filter(|_| part_targets.contains_key(item.owner_entity()))
                .map(|layered| (item, layered))
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|(item, _)| {
        let layer_order = render_layer_lookup
            .get(item.render_layer())
            .map(|layer| layer.order)
            .unwrap_or(0.0);
        (
            (layer_order * 1000.0).round() as i32,
            (item.z_index() * 1000.0).round() as i32,
        )
    });

    for (item, layered) in items {
        let Some(parts) = part_targets.get(item.owner_entity()) else {
            continue;
        };
        renderer.append_layered_image_primitive_texture_batches_filtered(
            &mut texture_batches,
            &target.device,
            &target.queue,
            assets,
            &viewport,
            Transform2::default(),
            layered,
            Some(parts),
            None,
            false,
        );
    }

    renderer.render_offscreen_batches(target, pass_load.to_load_op(), &texture_batches, &[], &[])
}
