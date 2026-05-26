use super::policy::{VisualSourceBufferPolicySet, VisualSourceBufferResolutionPolicy};
use super::*;

pub(super) fn produce_material_map_buffers(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    policy: &VisualSourceBufferPolicySet,
) -> AmigoResult<()> {
    produce_one_material_source(
        renderer,
        request,
        scene_color_target,
        amigo_render_api::VisualSourceKind2d::SceneNormal,
        policy.scene_normal,
        "amigo-visual-source-scene-normal",
    )?;
    produce_one_material_source(
        renderer,
        request,
        scene_color_target,
        amigo_render_api::VisualSourceKind2d::SceneWetness,
        policy.scene_wetness,
        "amigo-visual-source-scene-wetness",
    )?;
    produce_scene_highlight(
        renderer,
        request,
        scene_color_target,
        policy.scene_highlight,
    )?;
    produce_scene_emissive(renderer, request, scene_color_target, policy.scene_emissive)?;
    Ok(())
}

fn produce_scene_highlight(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    policy: VisualSourceBufferResolutionPolicy,
) -> AmigoResult<()> {
    produce_one_material_source(
        renderer,
        request,
        scene_color_target,
        amigo_render_api::VisualSourceKind2d::SceneHighlight,
        policy,
        "amigo-visual-source-scene-highlight",
    )
}

fn produce_scene_emissive(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    policy: VisualSourceBufferResolutionPolicy,
) -> AmigoResult<()> {
    produce_one_material_source(
        renderer,
        request,
        scene_color_target,
        amigo_render_api::VisualSourceKind2d::SceneEmissive,
        policy,
        "amigo-visual-source-scene-emissive",
    )
}

fn produce_one_material_source(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    scene_color_target: &WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
    policy: VisualSourceBufferResolutionPolicy,
    label: &'static str,
) -> AmigoResult<()> {
    if !policy.should_produce() {
        set_material_target(renderer, kind, None);
        return Ok(());
    }

    let existing = match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => {
            renderer.visual_source_targets_2d.scene_normal.take()
        }
        amigo_render_api::VisualSourceKind2d::SceneWetness => {
            renderer.visual_source_targets_2d.scene_wetness.take()
        }
        amigo_render_api::VisualSourceKind2d::SceneHighlight => {
            renderer.visual_source_targets_2d.scene_highlight.take()
        }
        amigo_render_api::VisualSourceKind2d::SceneEmissive => {
            renderer.visual_source_targets_2d.scene_emissive.take()
        }
        _ => return Ok(()),
    };

    let mut target = existing.unwrap_or_else(|| {
        super::super::offscreen_ops::compatible_offscreen_target(scene_color_target, label)
    });
    if !render_per_draw_visual_map_buffer(renderer, request, &mut target, kind)? {
        if matches!(
            kind,
            amigo_render_api::VisualSourceKind2d::SceneHighlight
                | amigo_render_api::VisualSourceKind2d::SceneEmissive
        ) {
            renderer.clear_offscreen_to_color(
                &mut target,
                util::color_to_wgpu(crate::renderer::service::missing_debug_color_for(kind)),
            )?;
        } else {
            let scene_color_view = scene_color_target.view.clone();
            procedural_material::render_procedural_material_buffer(
                renderer,
                request,
                &mut target,
                kind,
                Some(&scene_color_view),
            )?;
        }
    }
    set_material_target(renderer, kind, Some(target));
    Ok(())
}

fn set_material_target(
    renderer: &mut WgpuSceneRenderer,
    kind: amigo_render_api::VisualSourceKind2d,
    target: Option<WgpuOffscreenTarget>,
) {
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => {
            renderer.visual_source_targets_2d.scene_normal = target;
        }
        amigo_render_api::VisualSourceKind2d::SceneWetness => {
            renderer.visual_source_targets_2d.scene_wetness = target;
        }
        amigo_render_api::VisualSourceKind2d::SceneHighlight => {
            renderer.visual_source_targets_2d.scene_highlight = target;
        }
        amigo_render_api::VisualSourceKind2d::SceneEmissive => {
            renderer.visual_source_targets_2d.scene_emissive = target;
        }
        _ => {}
    }
}

fn render_per_draw_visual_map_buffer(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
) -> AmigoResult<bool> {
    let viewport = Viewport::from_offscreen(target);
    let camera = crate::renderer::scene::resolve_camera2d_transform(request.scene_view);
    let mut texture_batches = Vec::new();
    let mut color_batches = Vec::new();
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();

    for item in request.world_2d.renderables {
        let mut adapter_ctx = crate::WgpuVisualMapAdapterContext {
            renderer,
            texture_batches: &mut texture_batches,
            target,
            assets: request.assets,
            viewport: &viewport,
            camera,
            kind,
        };
        let _ = renderable_adapters.append_visual_map_batches(&mut adapter_ctx, item);
    }

    let candidate_texture_appended =
        procedural_material::append_camera_optical_candidate_texture_buffers(
            renderer,
            &mut texture_batches,
            target,
            request,
            &viewport,
            camera,
            kind,
        );

    procedural_material::append_procedural_material_buffers(
        &mut color_batches,
        request,
        &viewport,
        camera,
        kind,
        candidate_texture_appended,
    );

    if texture_batches.is_empty() && color_batches.is_empty() {
        return Ok(false);
    }

    renderer.render_offscreen_batches(
        target,
        wgpu::LoadOp::Clear(util::color_to_wgpu(
            crate::renderer::service::missing_debug_color_for(kind),
        )),
        &texture_batches,
        &color_batches,
        &[],
    )?;
    Ok(true)
}
