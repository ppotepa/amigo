use super::*;
use amigo_camera_optics_plugin::api::{
    CameraOpticalCandidate2d, CameraOpticalCoverage2d, CameraOpticalResponse2d,
};
use amigo_camera_optics_plugin::render_wgpu::CameraOpticalRenderTargetPlan;

pub(super) fn render_procedural_material_buffer(
    renderer: &mut WgpuSceneRenderer,
    _request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
    _scene_color_view: Option<&wgpu::TextureView>,
) -> AmigoResult<()> {
    renderer.clear_offscreen_to_color(
        target,
        util::color_to_wgpu(crate::renderer::service::fallback_color_for(kind)),
    )
}

pub(super) fn append_procedural_material_buffers(
    color_batches: &mut Vec<ColorBatch>,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
    candidate_buffers_already_appended: bool,
) {
    let optical_plan = CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str());

    if optical_plan.is_none()
        && !matches!(
            kind,
            amigo_render_api::VisualSourceKind2d::SceneNormal
                | amigo_render_api::VisualSourceKind2d::SceneWetness
        )
    {
        return;
    }

    if optical_plan
        .as_ref()
        .is_some_and(|plan| plan.accepts_color_candidates)
        && append_camera_optical_candidate_color_buffers(color_batches, request, viewport, camera, kind)
    {
        return;
    }

    if candidate_buffers_already_appended {
        return;
    }

    if optical_plan.is_some() {
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

pub(super) fn append_camera_optical_candidate_texture_buffers(
    renderer: &mut WgpuSceneRenderer,
    texture_batches: &mut Vec<TextureBatch>,
    target: &WgpuOffscreenTarget,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    let Some(plan) =
        CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str())
    else {
        return false;
    };
    if !plan.accepts_texture_candidates {
        return false;
    };

    let mut appended = false;
    for candidate in request
        .world_2d
        .camera_optical_candidates
        .iter()
        .filter(|candidate| candidate.is_active())
    {
        let Some((source, channel)) =
            amigo_camera_optics_plugin::render_wgpu::lightmap_channel_parts(&candidate.coverage)
        else {
            continue;
        };
        let lightmaps = request.world_2d.lightmaps.commands();
        let Some(lightmap) = lightmaps.iter().find(|lightmap| &lightmap.id == source)
        else {
            continue;
        };
        let Some(channel) = lightmap.channels.iter().find(|entry| &entry.id == channel) else {
            continue;
        };
        let layered_images = request.world_2d.layered_images.commands();
        let Some(command) = layered_images
            .iter()
            .find(|command| command.entity_name == lightmap.source.entity_name)
        else {
            continue;
        };
        let included_parts = channel.layers.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let transform = crate::renderer::scene::resolve_transform2(
            request.scene,
            &command.entity_name,
            command.transform,
        );
        appended |= renderer.append_layered_image_texture_batches_filtered_tinted(
            texture_batches,
            &target.device,
            &target.queue,
            request.assets,
            viewport,
            camera,
            transform,
            command,
            Some(&included_parts),
            None,
            false,
            optical_candidate_color_for_kind(candidate, kind),
        );
    }
    appended
}

fn append_camera_optical_candidate_color_buffers(
    color_batches: &mut Vec<ColorBatch>,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    let mut appended = false;
    for candidate in request
        .world_2d
        .camera_optical_candidates
        .iter()
        .filter(|candidate| candidate.is_active())
    {
        let color = optical_candidate_color_for_kind(candidate, kind);
        match &candidate.coverage {
            CameraOpticalCoverage2d::Hotspot {
                entity_name: _,
                radius_px,
            } => {
                let center = candidate.position_px.unwrap_or([0.0, 0.0]);
                util::append_visual_quad(
                    color_batches,
                    viewport,
                    camera,
                    Transform2 {
                        translation: Vec2::new(center[0], center[1]),
                        rotation_radians: 0.0,
                        scale: Vec2::new(1.0, 1.0),
                    },
                    Vec2::new(radius_px.max(1.0) * 2.0, radius_px.max(1.0) * 2.0),
                    color,
                );
                appended = true;
            }
            CameraOpticalCoverage2d::ParticleCoverage {
                emitter_entity_name,
            } => {
                for command in request
                    .world_2d
                    .particles
                    .iter()
                    .filter(|command| &command.emitter_entity_name == emitter_entity_name)
                {
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
                        color,
                    );
                    appended = true;
                }
            }
            CameraOpticalCoverage2d::Glyphs {
                entity_name,
                render_layer,
            } => {
                for command in request.world_2d.text2d.commands().iter().filter(|command| {
                    &command.entity_name == entity_name && &command.render_layer == render_layer
                }) {
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
                        color,
                    );
                    appended = true;
                }
            }
            CameraOpticalCoverage2d::TextureAlpha {
                entity_name,
                render_layer,
            } => {
                for command in request.world_2d.sprites.commands().iter().filter(|command| {
                    &command.entity_name == entity_name && &command.render_layer == render_layer
                }) {
                    let transform = crate::renderer::scene::resolve_transform2(
                        request.scene,
                        &command.entity_name,
                        command.transform,
                    );
                    util::append_visual_quad(
                        color_batches,
                        viewport,
                        camera,
                        transform,
                        command.sprite.size,
                        color,
                    );
                    appended = true;
                }
            }
            CameraOpticalCoverage2d::VectorCoverage {
                entity_name,
                render_layer,
            } => {
                for command in request.world_2d.vectors.commands().iter().filter(|command| {
                    &command.entity_name == entity_name && &command.render_layer == render_layer
                }) {
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
                    shape.style.stroke_color = color;
                    shape.style.fill_color = Some(color);
                    crate::renderer::world_2d::append_vector_shape_vertices(
                        color_batch_vertices(color_batches, ParticleBlendMode2d::Alpha),
                        viewport,
                        camera,
                        transform,
                        &shape,
                    );
                    appended = true;
                }
            }
            coverage if amigo_camera_optics_plugin::render_wgpu::coverage_uses_texture_path(coverage) => {
                // Texture-backed LightMapChannel candidates are handled by
                // append_camera_optical_candidate_texture_buffers.
            }
            CameraOpticalCoverage2d::LightMapChannel { .. } => {
                // Handled above through plugin-owned texture-path policy.
            }
            CameraOpticalCoverage2d::Unsupported { .. } => {}
        }
    }
    appended
}

fn optical_candidate_color_for_kind(
    candidate: &CameraOpticalCandidate2d,
    kind: amigo_render_api::VisualSourceKind2d,
) -> ColorRgba {
    let Some(plan) =
        CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str())
    else {
        return ColorRgba::new(0.0, 0.0, 0.0, candidate.color_rgba[3]);
    };
    let rgba =
        amigo_camera_optics_plugin::render_wgpu::optical_candidate_color_rgba_for_target(
            candidate,
            &plan.target,
        );
    ColorRgba::new(rgba[0], rgba[1], rgba[2], rgba[3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_camera_optics_plugin::api::CameraOpticalCandidateStatus2d;

    fn candidate(roles: &[&str]) -> CameraOpticalCandidate2d {
        let roles =
            amigo_render_api::RenderContributionSet::from_pairs(roles.iter().map(|role| (*role, true)));
        let mut target_ids = Vec::new();
        if roles.enabled_or(amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE, false) {
            target_ids.push(amigo_camera_optics_plugin::render_wgpu::scene_highlight_target_id());
        }
        if roles.enabled_or(amigo_render_api::render_contribution_roles::BLOOM_SOURCE, false) {
            target_ids.push(amigo_camera_optics_plugin::render_wgpu::scene_emissive_target_id());
        }
        CameraOpticalCandidate2d {
            owner: "neon.mid".to_owned(),
            component_kind: "LightGroup2D".to_owned(),
            render_layer: None,
            color_rgba: [0.5, 0.25, 1.0, 0.8],
            intensity: 2.0,
            response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.25,
                bloom: 0.5,
                glare: 0.75,
                ..CameraOpticalResponse2d::default()
            },
            coverage: CameraOpticalCoverage2d::LightMapChannel {
                source: "neon-alley-lightmap".to_owned(),
                channel: "mid_neon".to_owned(),
            },
            roles,
            status: CameraOpticalCandidateStatus2d::Active,
            reason: "test".to_owned(),
            position_px: None,
            target_ids,
            trace: None,
        }
    }

    #[test]
    fn optical_candidate_color_uses_candidate_highlight_gain() {
        let color = optical_candidate_color_for_kind(
            &candidate(&[amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE]),
            amigo_render_api::VisualSourceKind2d::SceneHighlight,
        );
        assert!((color.r - 0.75).abs() < 0.001);
        assert!((color.g - 0.375).abs() < 0.001);
        assert!((color.b - 1.5).abs() < 0.001);
        assert!((color.a - 0.8).abs() < 0.001);
    }

    #[test]
    fn optical_candidate_color_uses_candidate_emissive_gain() {
        let color = optical_candidate_color_for_kind(
            &candidate(&[amigo_render_api::render_contribution_roles::BLOOM_SOURCE]),
            amigo_render_api::VisualSourceKind2d::SceneEmissive,
        );
        assert!((color.r - 0.5).abs() < 0.001);
        assert!((color.g - 0.25).abs() < 0.001);
        assert!((color.b - 1.0).abs() < 0.001);
        assert!((color.a - 0.8).abs() < 0.001);
    }

    #[test]
    fn optical_candidate_color_is_black_without_required_role() {
        let color = optical_candidate_color_for_kind(
            &candidate(&[amigo_render_api::render_contribution_roles::BLOOM_SOURCE]),
            amigo_render_api::VisualSourceKind2d::SceneHighlight,
        );
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert!((color.a - 0.8).abs() < 0.001);
    }

    #[test]
    fn lightmap_channel_candidate_resolves_source_and_channel() {
        let candidate = candidate(&[amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE]);
        let Some((source, channel)) =
            amigo_camera_optics_plugin::render_wgpu::lightmap_channel_parts(&candidate.coverage)
        else {
            panic!("expected lightmap channel coverage");
        };
        assert_eq!(source, "neon-alley-lightmap");
        assert_eq!(channel, "mid_neon");
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
        // Highlight/emissive are candidate-driven. This keeps accidental retired
        // procedural calls from reintroducing scene-color luma extraction.
        amigo_render_api::VisualSourceKind2d::SceneHighlight
        | amigo_render_api::VisualSourceKind2d::SceneEmissive => source,
        _ => source,
    }
}
