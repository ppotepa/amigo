use super::policy::VisualSourceBufferPolicySet;
use super::*;

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
            &amigo_composite_plugin::PostFxHost2dId::new("camera"),
            &amigo_composite_plugin::PostFx2dId::new("shutter_blur"),
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
    host_id: &amigo_composite_plugin::PostFxHost2dId,
    effect_id: &amigo_composite_plugin::PostFx2dId,
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
        if let Some(effect) = super::super::focus_depth_plan::shutter_blur_effect_for(
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
            if let amigo_composite_plugin::PostFx2d::ShutterBlur(effect) = &instance.effect {
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

    for command in request.world_2d.tilemaps.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            Transform2::default(),
        );
        let key = format!("tilemap:{}", command.entity_name);
        current_positions.insert(key.clone(), transform.translation);
        util::append_visual_quad(
            &mut color_batches,
            &viewport,
            camera,
            transform,
            util::tilemap_draw_size(&command.tilemap),
            motion_vector_color(
                renderer
                    .visual_source_previous_positions_2d
                    .get(&key)
                    .copied(),
                transform.translation,
                target_size,
            ),
        );
    }

    for command in request.world_2d.sprites.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.transform,
        );
        let key = format!("sprite:{}", command.entity_name);
        current_positions.insert(key.clone(), transform.translation);
        let color = motion_vector_color(
            renderer
                .visual_source_previous_positions_2d
                .get(&key)
                .copied(),
            transform.translation,
            target_size,
        );
        let sprite = amigo_sprite_2d_plugin::Sprite {
            texture: command.sprite.texture.clone(),
            size: command.sprite.size,
            sheet: None,
            sheet_is_explicit: false,
            animation_override: None,
            visual_maps: None,
            frame_index: command.sprite.frame_index,
            frame_elapsed: command.sprite.frame_elapsed,
        };
        crate::renderer::world_2d::append_sprite_vertices(
            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha),
            &viewport,
            camera,
            transform,
            &sprite,
            color,
        );
    }

    for command in request.world_2d.layered_images.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.transform,
        );
        let key = format!("layered_image:{}", command.entity_name);
        current_positions.insert(key.clone(), transform.translation);
        let color = motion_vector_color(
            renderer
                .visual_source_previous_positions_2d
                .get(&key)
                .copied(),
            transform.translation,
            target_size,
        );
        let sprite = amigo_sprite_2d_plugin::Sprite {
            texture: command.image.asset.clone(),
            size: command.image.size,
            sheet: None,
            sheet_is_explicit: false,
            animation_override: None,
            visual_maps: None,
            frame_index: 0,
            frame_elapsed: 0.0,
        };
        crate::renderer::world_2d::append_sprite_vertices(
            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha),
            &viewport,
            camera,
            transform,
            &sprite,
            color,
        );
    }

    for command in request.world_2d.vectors.commands() {
        let transform = crate::renderer::world_2d::vector_viewport_fit_transform(
            &viewport,
            crate::renderer::scene::resolve_transform2(
                request.scene,
                &command.entity_name,
                command.transform,
            ),
            command.viewport_fit,
            command.viewport_canvas_size,
        );
        let key = format!("vector:{}", command.entity_name);
        current_positions.insert(key.clone(), transform.translation);
        let color = motion_vector_color(
            renderer
                .visual_source_previous_positions_2d
                .get(&key)
                .copied(),
            transform.translation,
            target_size,
        );
        let mut shape = command.shape.clone();
        shape.style.stroke_color = color;
        shape.style.fill_color = Some(color);
        crate::renderer::world_2d::append_vector_shape_vertices(
            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha),
            &viewport,
            camera,
            transform,
            &shape,
        );
    }

    for command in request.world_2d.text2d.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.text.transform,
        );
        let key = format!("text2d:{}", command.entity_name);
        current_positions.insert(key.clone(), transform.translation);
        util::append_visual_quad(
            &mut color_batches,
            &viewport,
            camera,
            transform,
            command.text.bounds,
            motion_vector_color(
                renderer
                    .visual_source_previous_positions_2d
                    .get(&key)
                    .copied(),
                transform.translation,
                target_size,
            ),
        );
    }

    for command in request.world_2d.beacons {
        let key = format!("beacon:{}", command.entity_name);
        current_positions.insert(key.clone(), command.center);
        let color = motion_vector_color(
            renderer
                .visual_source_previous_positions_2d
                .get(&key)
                .copied(),
            command.center,
            target_size,
        );
        util::append_visual_quad(
            &mut color_batches,
            &viewport,
            camera,
            Transform2 {
                translation: command.center,
                rotation_radians: command.rotation_radians,
                scale: Vec2::new(1.0, 1.0),
            },
            Vec2::new(
                command.halo_radius_px.max(command.core_radius_px) * 2.0,
                command.halo_radius_px.max(command.core_radius_px) * 2.0,
            ),
            color,
        );
    }

    for (index, command) in request.world_2d.particles.iter().enumerate() {
        let key = format!("particle:{}:{index}", command.emitter_entity_name);
        current_positions.insert(key, command.position);
        util::append_visual_quad(
            &mut color_batches,
            &viewport,
            camera,
            Transform2 {
                translation: command.position,
                rotation_radians: command.transform.rotation_radians,
                scale: command.transform.scale,
            },
            Vec2::new(command.size.max(1.0), command.size.max(1.0)),
            motion_vector_color(
                Some(command.previous_position),
                command.position,
                target_size,
            ),
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
