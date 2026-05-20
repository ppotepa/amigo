use super::world_filters::WorldPassLoad;
use super::world_selection::OwnedWorldRenderSelection;
use super::*;
use amigo_render_api::render_contribution_roles as roles;

pub(super) fn first_read(
    node: &amigo_render_api::FrameGraphNode,
    name: &str,
) -> AmigoResult<amigo_render_api::FrameResourceId> {
    node.reads.first().copied().ok_or_else(|| {
        amigo_core::AmigoError::Message(format!("{name} graph node is missing a read target"))
    })
}

pub(super) fn first_write(
    node: &amigo_render_api::FrameGraphNode,
    name: &str,
) -> AmigoResult<amigo_render_api::FrameResourceId> {
    node.writes.first().copied().ok_or_else(|| {
        amigo_core::AmigoError::Message(format!("{name} graph node is missing a write target"))
    })
}

pub(super) fn execute_world_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    let write_id = first_write(node, "world")?;
    let target = resources.target_mut(write_id).ok_or_else(|| {
        amigo_core::AmigoError::Message("world node missing render target".into())
    })?;

    let beacon_layers = request
        .world_2d
        .beacons
        .iter()
        .map(|beacon| beacon.render_layer.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let overlay_beacon_layers = request
        .world_2d
        .beacons
        .iter()
        .filter(|beacon| {
            beacon
                .render_contributions
                .enabled_or(roles::OVERLAY_VISIBLE, true)
        })
        .map(|beacon| beacon.render_layer.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let visible_beacon_layers = overlay_beacon_layers
        .iter()
        .filter(|layer_id| {
            request
                .world_2d
                .render_layers
                .iter()
                .find(|layer| layer.id == **layer_id)
                .is_none_or(|layer| layer.visible && layer.opacity > 0.001)
        })
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let world_ctx = WorldRenderContext::from_request(request);
    let selection = base_world_selection(request.post_fx_stacks, request.world_2d.render_layers)
        .with_excluded_layers(&beacon_layers);
    world::execute_world_to_offscreen(renderer, target, world_ctx, selection.borrowed(), &[])?;
    super::visual_source_buffer_pass::produce_visual_source_buffers_after_world(
        renderer, request, target,
    )?;
    super::plate_relight::apply_plate_relight_after_world(renderer, request, target)?;
    if !visible_beacon_layers.is_empty()
        && !amigo_relight_2d_plugin::is_plate_relight_render_debug_view(&request.camera_debug_view)
    {
        let overlay_selection =
            OwnedWorldRenderSelection::include_layers(visible_beacon_layers, false, WorldPassLoad::Load);
        world::execute_world_to_offscreen(
            renderer,
            target,
            WorldRenderContext::from_request(request),
            overlay_selection.borrowed(),
            &[],
        )?;
    }
    Ok(())
}

pub(super) fn execute_post_fx_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    post_fx: scoped_post_fx::PostFxGraphNodeContext<'_>,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    scoped_post_fx::execute_post_fx_graph_node(renderer, request, node, post_fx, resources)
}

pub(super) fn execute_game_ui_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    let read = first_read(node, "game-ui")?;
    let write = first_write(node, "game-ui")?;

    if read != write {
        let source = resources
            .target(read)
            .ok_or_else(|| {
                amigo_core::AmigoError::Message("game-ui read target unavailable".into())
            })?
            .view
            .clone();
        let target = resources.target_mut(write).ok_or_else(|| {
            amigo_core::AmigoError::Message("game-ui write target unavailable".into())
        })?;
        renderer.copy_offscreen_to_offscreen(target, &source)?;
    }

    let target = resources.target_mut(write).ok_or_else(|| {
        amigo_core::AmigoError::Message("game-ui write target unavailable".into())
    })?;
    renderer.render_ui_documents_to_offscreen(
        target,
        request.assets,
        request.game_ui,
        wgpu::LoadOp::Load,
    )
}

pub(super) fn execute_debug_overlay_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    let read = first_read(node, "debug-overlay")?;
    let write = first_write(node, "debug-overlay")?;

    if read != write {
        let source = resources
            .target(read)
            .ok_or_else(|| {
                amigo_core::AmigoError::Message("debug-overlay read target unavailable".into())
            })?
            .view
            .clone();
        let target = resources.target_mut(write).ok_or_else(|| {
            amigo_core::AmigoError::Message("debug-overlay write target unavailable".into())
        })?;
        renderer.copy_offscreen_to_offscreen(target, &source)?;
    }

    let target = resources.target_mut(write).ok_or_else(|| {
        amigo_core::AmigoError::Message("debug-overlay write target unavailable".into())
    })?;
    renderer.render_ui_documents_to_offscreen(
        target,
        request.assets,
        request.debug_ui,
        wgpu::LoadOp::Load,
    )
}

pub(super) fn execute_present_graph_node(
    renderer: &mut WgpuSceneRenderer,
    request: &mut WgpuFrameRenderRequest<'_>,
    node: &amigo_render_api::FrameGraphNode,
    resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
) -> AmigoResult<()> {
    let read =
        node.reads.first().copied().ok_or_else(|| {
            amigo_core::AmigoError::Message("present node has no read target".into())
        })?;
    let source = resources
        .target(read)
        .ok_or_else(|| amigo_core::AmigoError::Message("present read target unavailable".into()))?
        .view
        .clone();

    match &mut request.target {
        WgpuFrameRenderTarget::Surface(surface) => renderer.render_texture_to_surface(
            surface,
            &source,
            request.assets,
            request.debug_ui,
            request.game_viewport,
            &super::emergency_overlay_lines(
                request.emergency_overlay,
                &renderer.emergency_overlay_lines,
            ),
        ),
        WgpuFrameRenderTarget::Offscreen(target) => {
            renderer.copy_offscreen_to_offscreen(target, &source)?;
            renderer.render_emergency_overlay_to_offscreen(
                target,
                &super::emergency_overlay_lines(
                    request.emergency_overlay,
                    &renderer.emergency_overlay_lines,
                ),
            )?;
            Ok(())
        }
    }
}
