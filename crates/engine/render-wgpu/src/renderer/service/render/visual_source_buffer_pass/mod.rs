use super::*;

mod layer_buffers;
mod material_maps;
mod motion;
mod policy;
mod procedural_material;
mod util;

pub(super) fn produce_visual_source_buffers_after_world(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let policy = policy::VisualSourceBufferPolicySet::from_request(request);

    layer_buffers::produce_layer_buffers(renderer, request, scene_color_target, &policy)?;
    material_maps::produce_material_map_buffers(renderer, request, scene_color_target, &policy)?;
    motion::produce_motion_buffer(renderer, request, scene_color_target, &policy)?;

    Ok(())
}
