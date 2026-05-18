use super::policy::VisualSourceBufferPolicySet;
use super::*;

pub(super) fn produce_layer_buffers(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    policy: &VisualSourceBufferPolicySet,
) -> AmigoResult<()> {
    if policy.layer_mask.should_produce() {
        let mut target = util::take_or_create_target(
            &mut renderer.visual_source_targets_2d.layer_mask,
            scene_color_target,
            "amigo-visual-source-layer-mask",
        );
        super::super::visual_debug::render_layer_mask_visual_source(
            renderer,
            request,
            &mut target,
        )?;
        renderer.visual_source_targets_2d.layer_mask = Some(target);
    } else {
        renderer.visual_source_targets_2d.layer_mask = None;
    }

    if policy.layer_roles.should_produce() {
        let mut target = util::take_or_create_target(
            &mut renderer.visual_source_targets_2d.layer_roles,
            scene_color_target,
            "amigo-visual-source-layer-roles",
        );
        super::super::visual_debug::render_layer_roles_debug_source(
            renderer,
            request,
            &mut target,
        )?;
        renderer.visual_source_targets_2d.layer_roles = Some(target);
    } else {
        renderer.visual_source_targets_2d.layer_roles = None;
    }

    Ok(())
}
