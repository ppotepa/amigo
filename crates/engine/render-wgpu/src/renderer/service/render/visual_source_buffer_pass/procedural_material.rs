use super::*;

pub(super) fn render_procedural_material_buffer(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
    scene_color_view: Option<&wgpu::TextureView>,
) -> AmigoResult<()> {
    // V1 fallback: converts scene color/light presence into highlight/emissive buffer.
    // This is produced by the renderer, but remains derived until material/light MRT exists.
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneHighlight => {
            if let Some(view) = scene_color_view {
                render_scene_highlight_fallback(renderer, request, view, target)
            } else {
                renderer.clear_offscreen_to_color(
                    target,
                    util::color_to_wgpu(material_color_for_kind(
                        kind,
                        ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                    )),
                )
            }
        }
        amigo_render_api::VisualSourceKind2d::SceneEmissive => {
            if let Some(view) = scene_color_view {
                render_scene_emissive_fallback(renderer, request, view, target)
            } else {
                renderer.clear_offscreen_to_color(
                    target,
                    util::color_to_wgpu(material_color_for_kind(
                        kind,
                        ColorRgba::new(0.0, 0.0, 0.0, 1.0),
                    )),
                )
            }
        }
        _ => renderer.clear_offscreen_to_color(
            target,
            util::color_to_wgpu(crate::renderer::service::fallback_color_for(kind)),
        ),
    }
}

fn render_scene_highlight_fallback(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    source: &wgpu::TextureView,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| !flags.scene_highlight_generated)
    {
        return renderer.clear_offscreen_to_color(
            target,
            util::color_to_wgpu(crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::SceneHighlight,
            )),
        );
    }
    crate::renderer::service::post_fx::dirty_bloom::execute_highlight_extract_debug(
        renderer, source, target,
    )
}

fn render_scene_emissive_fallback(
    renderer: &mut WgpuSceneRenderer,
    request: &WgpuFrameRenderRequest<'_>,
    source: &wgpu::TextureView,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    if request
        .visual_source_flags_2d
        .is_some_and(|flags| !flags.scene_emissive_generated)
    {
        return renderer.clear_offscreen_to_color(
            target,
            util::color_to_wgpu(crate::renderer::service::fallback_color_for(
                amigo_render_api::VisualSourceKind2d::SceneEmissive,
            )),
        );
    }
    crate::renderer::service::post_fx::dirty_bloom::execute_luma_extract_debug(
        renderer,
        source,
        target,
        0.45,
        0.18,
        "amigo-visual-source-scene-emissive",
    )
}

pub(super) fn append_procedural_material_buffers(
    color_batches: &mut Vec<ColorBatch>,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
) {
    if !matches!(
        kind,
        amigo_render_api::VisualSourceKind2d::SceneNormal
            | amigo_render_api::VisualSourceKind2d::SceneWetness
            | amigo_render_api::VisualSourceKind2d::SceneHighlight
            | amigo_render_api::VisualSourceKind2d::SceneEmissive
    ) {
        return;
    }

    for command in request.world_2d.tilemaps.commands() {
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            Transform2::default(),
        );
        util::append_visual_quad(
            color_batches,
            viewport,
            camera,
            transform,
            util::tilemap_draw_size(&command.tilemap),
            material_color_for_kind(kind, ColorRgba::new(0.08, 0.08, 0.09, 1.0)),
        );
    }

    for command in request.world_2d.vectors.commands() {
        let transform = crate::renderer::world_2d::vector_viewport_fit_transform(
            viewport,
            crate::renderer::scene::resolve_transform2(
                request.scene,
                &command.entity_name,
                command.transform,
            ),
            command.viewport_fit,
            command.viewport_canvas_size,
        );
        let mut shape = command.shape.clone();
        let source_color = shape.style.fill_color.unwrap_or(shape.style.stroke_color);
        let color = material_color_for_kind(kind, source_color);
        shape.style.stroke_color = color;
        shape.style.fill_color = Some(color);
        crate::renderer::world_2d::append_vector_shape_vertices(
            color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha),
            viewport,
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
        util::append_visual_quad(
            color_batches,
            viewport,
            camera,
            transform,
            command.text.bounds,
            material_color_for_kind(kind, command.text.style.color),
        );
    }

    for command in request.world_2d.beacons {
        util::append_visual_quad(
            color_batches,
            viewport,
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
            material_color_for_kind(kind, command.color),
        );
    }

    for command in request.world_2d.particles {
        util::append_visual_quad(
            color_batches,
            viewport,
            camera,
            Transform2 {
                translation: command.position,
                rotation_radians: command.transform.rotation_radians,
                scale: command.transform.scale,
            },
            Vec2::new(command.size.max(1.0), command.size.max(1.0)),
            material_color_for_kind(kind, command.color),
        );
    }
}

pub(super) fn material_color_for_kind(
    kind: amigo_render_api::VisualSourceKind2d,
    source: ColorRgba,
) -> ColorRgba {
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => {
            ColorRgba::new(0.5, 0.5, 1.0, source.a)
        }
        amigo_render_api::VisualSourceKind2d::SceneWetness => {
            let wet = ((source.r + source.g + source.b) / 3.0 * 0.35).clamp(0.0, 1.0);
            ColorRgba::new(0.0, wet * 0.55, wet * 0.72, source.a)
        }
        amigo_render_api::VisualSourceKind2d::SceneHighlight => {
            let peak = source.r.max(source.g).max(source.b);
            ColorRgba::new(source.r * peak, source.g * peak, source.b * peak, source.a)
        }
        amigo_render_api::VisualSourceKind2d::SceneEmissive => {
            ColorRgba::new(source.r * 0.80, source.g * 0.72, source.b * 0.64, source.a)
        }
        _ => source,
    }
}
