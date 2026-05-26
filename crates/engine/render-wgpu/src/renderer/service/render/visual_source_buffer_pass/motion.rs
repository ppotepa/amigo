use super::policy::VisualSourceBufferPolicySet;
use super::*;

// Estimates screen-space motion from previous per-draw transform positions.
// This is a produced renderer buffer, not a material-owned motion-vector pass.
pub(super) fn produce_motion_buffer(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    policy: &VisualSourceBufferPolicySet,
) -> AmigoResult<()> {
    if !policy.scene_motion.should_produce() {
        renderer.visual_source_targets_2d.scene_motion = None;
        return Ok(());
    }

    let source_view = scene_color_target.view.clone();
    let mut target = util::take_or_create_target(
        &mut renderer.visual_source_targets_2d.scene_motion,
        scene_color_target,
        "amigo-visual-source-scene-motion",
    );

    if !render_per_draw_motion_buffer(renderer, request, &mut target)? {
        render_shutter_motion_debug_replay(
            renderer,
            request,
            &source_view,
            &mut target,
            &amigo_render_api::PostFxHost2dId::new("camera"),
            &amigo_render_api::PostFx2dId::new("shutter_blur"),
            "shutter_blur",
        )?;
    }

    renderer.visual_source_targets_2d.scene_motion = Some(target);
    Ok(())
}

fn render_shutter_motion_debug_replay(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    source: &wgpu::TextureView,
    target: &mut WgpuOffscreenTarget,
    host_id: &amigo_render_api::PostFxHost2dId,
    effect_id: &amigo_render_api::PostFx2dId,
    feature_id: &str,
) -> AmigoResult<()> {
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| !flags.scene_motion_generated)
    {
        return renderer.clear_offscreen_to_color(
            target,
            util::color_to_wgpu(crate::renderer::service::missing_debug_color_for(
                amigo_render_api::VisualSourceKind2d::SceneMotion,
            )),
        );
    }
    if feature_id == "shutter_blur" {
        if let Some(effect) = super::super::focus_depth_plan::motion_debug_post_fx_for(
            request.post_fx_stacks,
            host_id,
            effect_id,
        ) {
            return crate::renderer::service::post_fx::shutter_blur::execute_motion_debug(
                renderer, host_id, effect_id, effect, source, target,
            );
        }
    }
    for stack in request.post_fx_stacks {
        for instance in &stack.effects {
            if let Some(effect) = instance.effect.as_shutter_blur() {
                if effect.is_active() {
                    return crate::renderer::service::post_fx::shutter_blur::execute_motion_debug(
                        renderer,
                        &stack.host_id,
                        &instance.id,
                        effect.clone(),
                        source,
                        target,
                    );
                }
            }
        }
    }
    renderer.clear_offscreen_to_color(
        target,
        util::color_to_wgpu(crate::renderer::service::missing_debug_color_for(
            amigo_render_api::VisualSourceKind2d::SceneMotion,
        )),
    )
}

fn render_per_draw_motion_buffer(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<bool> {
    let viewport = Viewport::from_offscreen(target);
    let camera = crate::renderer::scene::resolve_camera2d_transform(request.scene_view);
    let mut current_positions = std::collections::BTreeMap::new();
    let mut color_batches = Vec::new();
    let target_size = (target.width, target.height);
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();

    let previous_positions = renderer.visual_source_previous_positions_2d.clone();
    for item in request.world_2d.renderables {
        renderable_adapters.append_motion_batches(
            &mut crate::WgpuMotionAdapterContext {
                color_batches: &mut color_batches,
                viewport: &viewport,
                camera,
                target_size,
                current_positions: &mut current_positions,
                previous_positions: &previous_positions,
            },
            item,
        );
    }

    renderer.visual_source_previous_positions_2d = current_positions;

    if color_batches.is_empty() {
        return Ok(false);
    }

    renderer.render_offscreen_batches(
        target,
        wgpu::LoadOp::Clear(util::color_to_wgpu(
            crate::renderer::service::missing_debug_color_for(
                amigo_render_api::VisualSourceKind2d::SceneMotion,
            ),
        )),
        &[],
        &color_batches,
        &[],
    )?;
    Ok(true)
}
