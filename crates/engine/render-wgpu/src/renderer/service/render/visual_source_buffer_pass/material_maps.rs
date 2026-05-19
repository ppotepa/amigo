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
                util::color_to_wgpu(crate::renderer::service::fallback_color_for(kind)),
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
    let camera = crate::renderer::scene::resolve_camera2d_transform(
        request.scene,
        request.active_camera_2d_entity,
    );
    let mut texture_batches = Vec::new();
    let mut color_batches = Vec::new();

    for command in request.world_2d.sprites.commands() {
        let Some(asset) = visual_map_for_kind(command.sprite.visual_maps.as_ref(), kind) else {
            continue;
        };
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.transform,
        );
        let sprite = amigo_sprite_2d_plugin::Sprite {
            texture: asset.clone(),
            size: command.sprite.size,
            sheet: None,
            sheet_is_explicit: false,
            animation_override: None,
            visual_maps: None,
            frame_index: command.sprite.frame_index,
            frame_elapsed: command.sprite.frame_elapsed,
        };
        renderer.append_sprite_texture_batch(
            &mut texture_batches,
            &target.device,
            &target.queue,
            request.assets,
            &viewport,
            camera,
            transform,
            &sprite,
        );
    }

    for command in request.world_2d.layered_images.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.transform,
        );
        if let Some(asset) = visual_map_for_kind(command.image.visual_maps.as_ref(), kind) {
            append_visual_map_sprite_batch(
                renderer,
                &mut texture_batches,
                target,
                request,
                &viewport,
                camera,
                transform,
                asset,
                command.image.size,
            );
        }
        for override_ in &command.image.layer_overrides {
            let Some(asset) = visual_map_for_kind(override_.visual_maps.as_ref(), kind) else {
                continue;
            };
            append_visual_map_sprite_batch(
                renderer,
                &mut texture_batches,
                target,
                request,
                &viewport,
                camera,
                transform,
                asset,
                command.image.size,
            );
        }
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
            crate::renderer::service::fallback_color_for(kind),
        )),
        &texture_batches,
        &color_batches,
        &[],
    )?;
    Ok(true)
}

fn append_visual_map_sprite_batch(
    renderer: &mut WgpuSceneRenderer,
    texture_batches: &mut Vec<TextureBatch>,
    target: &WgpuOffscreenTarget,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    asset: &amigo_assets::AssetKey,
    size: Vec2,
) {
    let sprite = amigo_sprite_2d_plugin::Sprite {
        texture: asset.clone(),
        size,
        sheet: None,
        sheet_is_explicit: false,
        animation_override: None,
        visual_maps: None,
        frame_index: 0,
        frame_elapsed: 0.0,
    };
    renderer.append_sprite_texture_batch(
        texture_batches,
        &target.device,
        &target.queue,
        request.assets,
        viewport,
        camera,
        transform,
        &sprite,
    );
}

fn visual_map_for_kind(
    maps: Option<&amigo_scene::VisualMaps2dSceneCommand>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> Option<&amigo_assets::AssetKey> {
    let maps = maps?;
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => maps.normal.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneWetness => maps.wetness.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneEmissive => maps.emissive.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneHighlight => maps.highlight.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneColor
        | amigo_render_api::VisualSourceKind2d::SceneDepth
        | amigo_render_api::VisualSourceKind2d::LayerMask
        | amigo_render_api::VisualSourceKind2d::SceneMotion
        | amigo_render_api::VisualSourceKind2d::Debug => None,
    }
}
