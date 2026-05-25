use super::*;
use amigo_camera::{CameraOpticalCandidate2d, CameraOpticalCoverage2d};
use amigo_render_api::{CameraOpticalRenderTargetPlan, Renderable2dItem};

pub(super) fn render_procedural_material_buffer(
    renderer: &mut WgpuSceneRenderer,
    _request: &WgpuFrameRenderRequest<'_>,
    target: &mut WgpuOffscreenTarget,
    kind: amigo_render_api::VisualSourceKind2d,
    _scene_color_view: Option<&wgpu::TextureView>,
) -> AmigoResult<()> {
    renderer.clear_offscreen_to_color(
        target,
        util::color_to_wgpu(crate::renderer::service::missing_debug_color_for(kind)),
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
        && append_camera_optical_candidate_color_buffers(
            color_batches,
            request,
            viewport,
            camera,
            kind,
        )
    {
        return;
    }

    if candidate_buffers_already_appended {
        return;
    }

    if optical_plan.is_some() {
        return;
    }

    for item in request.world_2d.renderables {
        append_renderable_material_proxy(color_batches, viewport, camera, kind, item);
    }
}

fn append_renderable_material_proxy(
    color_batches: &mut Vec<ColorBatch>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
    item: &Renderable2dItem,
) {
    if let (Some(transform), Some(size)) = (
        item.primitive.proxy_quad_transform(),
        item.primitive.proxy_quad_size(),
    ) {
        let source_color = item
            .primitive
            .proxy_quad_color()
            .unwrap_or(ColorRgba::new(0.08, 0.08, 0.09, 1.0));
        util::append_visual_quad(
            color_batches,
            viewport,
            camera,
            transform,
            size,
            material_color_for_kind(kind, source_color),
        );
        return;
    }

    // Vector/material coverage needs explicit candidate geometry; do not proxy it
    // from render primitive shape in the visual-source path.
}

pub(super) fn append_camera_optical_candidate_texture_buffers(
    _renderer: &mut WgpuSceneRenderer,
    _texture_batches: &mut Vec<TextureBatch>,
    _target: &WgpuOffscreenTarget,
    _request: &WgpuFrameRenderRequest<'_>,
    _viewport: &Viewport,
    _camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    let Some(plan) = CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str()) else {
        return false;
    };
    if !plan.accepts_texture_candidates {
        return false;
    };

    // LightMapChannel candidates do not yet carry a backend drawable. Skip until
    // the candidate contract provides explicit texture source data.
    false
}

fn append_camera_optical_candidate_color_buffers(
    color_batches: &mut Vec<ColorBatch>,
    request: &WgpuFrameRenderRequest<'_>,
    viewport: &Viewport,
    camera: Transform2,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    let Some(plan) = CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str()) else {
        return false;
    };
    let mut appended = false;
    for candidate in request
        .world_2d
        .camera_optical_candidates
        .iter()
        .filter(|candidate| candidate.is_active() && candidate_targets_plan(candidate, &plan))
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
                for renderable in request.world_2d.renderables.iter().filter(|item| {
                    item.primitive.particle_batch().is_some_and(|primitive| {
                        &primitive.emitter_entity_name == emitter_entity_name
                    })
                }) {
                    let Some(command) = renderable.primitive.particle_batch() else {
                        continue;
                    };
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
                entity_name: _,
                render_layer: _,
            } => {
                // Requires explicit candidate geometry before the backend can draw it.
            }
            CameraOpticalCoverage2d::TextureAlpha {
                entity_name: _,
                render_layer: _,
            } => {
                // Requires explicit candidate geometry before the backend can draw it.
            }
            CameraOpticalCoverage2d::VectorCoverage {
                entity_name: _,
                render_layer: _,
            } => {
                // Requires explicit candidate geometry before the backend can draw it.
            }
            coverage if coverage_uses_texture_path(coverage) => {
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

fn candidate_targets_plan(
    candidate: &CameraOpticalCandidate2d,
    plan: &CameraOpticalRenderTargetPlan,
) -> bool {
    candidate
        .target_ids
        .iter()
        .any(|candidate_target| candidate_target == &plan.target)
}

fn optical_candidate_color_for_kind(
    candidate: &CameraOpticalCandidate2d,
    kind: amigo_render_api::VisualSourceKind2d,
) -> ColorRgba {
    let Some(plan) = CameraOpticalRenderTargetPlan::for_visual_kind_name(kind.as_str()) else {
        return ColorRgba::new(0.0, 0.0, 0.0, candidate.color_rgba[3]);
    };
    let rgba = optical_candidate_color_rgba_for_target(candidate, &plan.target);
    ColorRgba::new(rgba[0], rgba[1], rgba[2], rgba[3])
}

fn coverage_uses_texture_path(coverage: &CameraOpticalCoverage2d) -> bool {
    matches!(coverage, CameraOpticalCoverage2d::LightMapChannel { .. })
}

fn optical_candidate_color_rgba_for_target(
    candidate: &CameraOpticalCandidate2d,
    target: &amigo_plugin_api::TargetId,
) -> [f32; 4] {
    let gain = if candidate
        .target_ids
        .iter()
        .any(|candidate_target| candidate_target == target)
    {
        if target.0 == "SceneHighlight" {
            candidate.highlight_gain()
        } else if target.0 == "SceneEmissive" {
            candidate.emissive_gain()
        } else {
            0.0
        }
    } else {
        0.0
    };

    [
        candidate.color_rgba[0] * gain,
        candidate.color_rgba[1] * gain,
        candidate.color_rgba[2] * gain,
        candidate.color_rgba[3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_camera::{CameraOpticalCandidateStatus2d, CameraOpticalResponse2d};
    use amigo_render_api::{scene_emissive_target_id, scene_highlight_target_id};
    fn candidate(roles: &[&str]) -> CameraOpticalCandidate2d {
        let roles = amigo_render_api::RenderContributionSet::from_pairs(
            roles.iter().map(|role| (*role, true)),
        );
        let mut target_ids = Vec::new();
        if roles.enabled_or(
            amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
            false,
        ) {
            target_ids.push(scene_highlight_target_id());
        }
        if roles.enabled_or(
            amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
            false,
        ) {
            target_ids.push(scene_emissive_target_id());
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
