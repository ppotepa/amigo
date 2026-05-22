use super::policy::VisualSourceBufferPolicySet;
use super::*;
use amigo_render_api::{RenderPrimitive2d, Renderable2dItem};

// V1 SceneMotionBuffer.
// This estimates screen-space motion from previous per-draw transform positions.
// It is a produced renderer buffer, but not yet a full material/motion-vector pass.
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
        render_shutter_motion_fallback(
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

fn render_shutter_motion_fallback(
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
            util::color_to_wgpu(crate::renderer::service::fallback_color_for(
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
            if let amigo_render_api::PostFx2d::ShutterBlur(effect) = &instance.effect {
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
        util::color_to_wgpu(crate::renderer::service::fallback_color_for(
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
    let camera = crate::renderer::scene::resolve_camera2d_transform(
        request.scene,
        request.active_camera_2d_entity,
    );
    let mut current_positions = std::collections::BTreeMap::new();
    let mut color_batches = Vec::new();
    let target_size = (target.width, target.height);

    for (index, item) in request.world_2d.renderables.iter().enumerate() {
        append_renderable_motion(
            renderer,
            &mut current_positions,
            &mut color_batches,
            &viewport,
            camera,
            target_size,
            item,
            index,
        );
    }

    renderer.visual_source_previous_positions_2d = current_positions;

    if color_batches.is_empty() {
        return Ok(false);
    }

    renderer.render_offscreen_batches(
        target,
        wgpu::LoadOp::Clear(util::color_to_wgpu(
            crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::SceneMotion,
            ),
        )),
        &[],
        &color_batches,
        &[],
    )?;
    Ok(true)
}

fn append_renderable_motion(
    renderer: &WgpuSceneRenderer,
    current_positions: &mut std::collections::BTreeMap<String, Vec2>,
    color_batches: &mut Vec<ColorBatch>,
    viewport: &Viewport,
    camera: Transform2,
    target_size: (u32, u32),
    item: &Renderable2dItem,
    index: usize,
) {
    match &item.primitive {
        RenderPrimitive2d::TileBatch(primitive) => {
            let transform = Transform2 {
                translation: primitive.origin_offset,
                ..Transform2::default()
            };
            let key = format!("tilemap:{}", item.owner_entity());
            current_positions.insert(key.clone(), transform.translation);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                transform,
                util::tilemap_primitive_draw_size(primitive),
                motion_vector_color(
                    renderer.visual_source_previous_positions_2d.get(&key).copied(),
                    transform.translation,
                    target_size,
                ),
            );
        }
        RenderPrimitive2d::TexturedQuad(primitive) => {
            let key = format!("quad:{}:{}", item.component_kind(), item.owner_entity());
            current_positions.insert(key.clone(), primitive.transform.translation);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                primitive.transform,
                primitive.size,
                motion_vector_color(
                    renderer.visual_source_previous_positions_2d.get(&key).copied(),
                    primitive.transform.translation,
                    target_size,
                ),
            );
        }
        RenderPrimitive2d::LayeredTexturedQuads(primitive) => {
            let key = format!("layered_image:{}", item.owner_entity());
            current_positions.insert(key.clone(), primitive.transform.translation);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                primitive.transform,
                primitive.size,
                motion_vector_color(
                    renderer.visual_source_previous_positions_2d.get(&key).copied(),
                    primitive.transform.translation,
                    target_size,
                ),
            );
        }
        RenderPrimitive2d::GlyphRun(primitive) => {
            let key = format!("text2d:{}", item.owner_entity());
            current_positions.insert(key.clone(), primitive.transform.translation);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                primitive.transform,
                primitive.bounds,
                motion_vector_color(
                    renderer.visual_source_previous_positions_2d.get(&key).copied(),
                    primitive.transform.translation,
                    target_size,
                ),
            );
        }
        RenderPrimitive2d::VectorMesh(primitive) => {
            let transform =
                crate::renderer::world_2d::vector_primitive_viewport_fit_transform(viewport, primitive);
            let key = format!("vector:{}", item.owner_entity());
            current_positions.insert(key.clone(), transform.translation);
            let color = motion_vector_color(
                renderer.visual_source_previous_positions_2d.get(&key).copied(),
                transform.translation,
                target_size,
            );
            crate::renderer::world_2d::append_vector_primitive_vertices(
                color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha),
                viewport,
                camera,
                primitive,
                Some(transform),
                Some(color),
                Some(color),
            );
        }
        RenderPrimitive2d::RadialLightVisual(primitive) => {
            let key = format!("beacon:{}", item.owner_entity());
            current_positions.insert(key.clone(), primitive.center);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                Transform2 {
                    translation: primitive.center,
                    rotation_radians: primitive.rotation_radians,
                    scale: Vec2::new(1.0, 1.0),
                },
                Vec2::new(
                    primitive.halo_radius_px.max(primitive.core_radius_px) * 2.0,
                    primitive.halo_radius_px.max(primitive.core_radius_px) * 2.0,
                ),
                motion_vector_color(
                    renderer.visual_source_previous_positions_2d.get(&key).copied(),
                    primitive.center,
                    target_size,
                ),
            );
        }
        RenderPrimitive2d::ParticleBatch(primitive) => {
            let key = format!("particle:{}:{index}", primitive.emitter_entity_name);
            current_positions.insert(key, primitive.position);
            util::append_visual_quad(
                color_batches,
                viewport,
                camera,
                Transform2 {
                    translation: primitive.position,
                    rotation_radians: primitive.transform.rotation_radians,
                    scale: primitive.transform.scale,
                },
                Vec2::new(primitive.size.max(1.0), primitive.size.max(1.0)),
                motion_vector_color(Some(primitive.previous_position), primitive.position, target_size),
            );
        }
    }
}

fn motion_vector_color(
    previous: Option<Vec2>,
    current: Vec2,
    target_size: (u32, u32),
) -> ColorRgba {
    let Some(previous) = previous else {
        return ColorRgba::new(0.5, 0.5, 0.0, 1.0);
    };
    let width = (target_size.0.max(1)) as f32;
    let height = (target_size.1.max(1)) as f32;
    let delta = Vec2::new(
        (current.x - previous.x) / width,
        (current.y - previous.y) / height,
    );
    let scale = 8.0;
    let x = (0.5 + delta.x * scale).clamp(0.0, 1.0);
    let y = (0.5 + delta.y * scale).clamp(0.0, 1.0);
    let magnitude = ((delta.x * delta.x + delta.y * delta.y).sqrt() * scale).clamp(0.0, 1.0);
    ColorRgba::new(x, y, magnitude, 1.0)
}
