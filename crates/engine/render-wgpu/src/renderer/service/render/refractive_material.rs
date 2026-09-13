use super::material_candidates::WgpuMaterialCandidate2d;
use super::offscreen_ops::{append_fullscreen_texture_vertices, compatible_offscreen_target};
use super::*;
use amigo_material_api::{Material2d, MaterialCandidateDecision2d, MaterialCandidateStatus2d};
use amigo_render_api::RenderAssetSource;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy)]
struct RefractiveMaterialUniform {
    resolution: [f32; 2],
    transmission: f32,
    refraction_px: f32,
    distortion: f32,
    dispersion: f32,
    roughness: f32,
    edge_boost: f32,
    opacity: f32,
    highlight: f32,
}

pub(super) fn execute_refractive_material_2d(
    renderer: &mut WgpuSceneRenderer,
    target: &mut WgpuOffscreenTarget,
    assets: &dyn RenderAssetSource,
    viewport: &Viewport,
    candidates: &[WgpuMaterialCandidate2d],
    decisions: &[MaterialCandidateDecision2d],
) -> AmigoResult<()> {
    let active_candidates = candidates
        .iter()
        .filter(|candidate| candidate.is_refractive())
        .collect::<Vec<_>>();
    if active_candidates.is_empty() {
        renderer.set_render_materials_last_summary(refractive_material_summary(
            candidates,
            decisions,
            MaterialPassState::Inactive("no_refractive_candidates"),
        ));
        return Ok(());
    }

    let mut scene_source = compatible_offscreen_target(target, "amigo-refractive-material-source");
    copy_target_texture(renderer, target, &mut scene_source)?;

    let mut mask_target = compatible_offscreen_target(target, "amigo-refractive-material-mask");
    let mut mask_batches = Vec::new();
    let mut mask_color_batches = Vec::new();
    let mut generated_masks = 0usize;
    let mut mask_sources = Vec::new();
    let renderable_adapters = crate::default_renderable_2d_adapter_registry();

    for candidate in &active_candidates {
        let mut adapter_ctx = crate::WgpuRefractiveMaskAdapterContext {
            renderer,
            texture_batches: &mut mask_batches,
            color_batches: &mut mask_color_batches,
            target: &mask_target,
            assets,
            viewport,
            camera: candidate.camera,
        };
        let outcome = renderable_adapters.append_refractive_mask_batches(
            &mut adapter_ctx,
            &candidate.source.renderable,
            candidate.common.layer_opacity,
        );
        if outcome.appended {
            if outcome.generated_geometry {
                generated_masks += 1;
            }
            mask_sources.push(outcome.source);
        }
    }

    if mask_batches.is_empty() && mask_color_batches.is_empty() {
        renderer.set_render_materials_last_summary(refractive_material_summary(
            candidates,
            decisions,
            MaterialPassState::Inactive("missing_mask_target"),
        ));
        return Ok(());
    }

    renderer.render_offscreen_batches(
        &mut mask_target,
        wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }),
        &mask_batches,
        &mask_color_batches,
        &[],
        &[],
    )?;

    let uniform = aggregate_uniform(target, &active_candidates);
    let device = &target.device;
    let queue = &target.queue;
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("amigo-refractive-material-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-refractive-material-texture-bind-group"),
        layout: &renderer.focus_blur_texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&scene_source.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&mask_target.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&mask_target.view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-refractive-material-uniform-buffer"),
        contents: bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-refractive-material-uniform-bind-group"),
        layout: &renderer.wet_reflections_uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let mut vertices = Vec::new();
    append_fullscreen_texture_vertices(&mut vertices);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("amigo-refractive-material-vertex-buffer"),
        contents: texture_vertices_as_bytes(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("amigo-refractive-material-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("amigo-refractive-material-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(
            renderer.post_fx_pipeline(crate::renderer::service::POST_FX_AUX_REFRACTIVE_MATERIAL),
        );
        pass.set_bind_group(0, &texture_bind_group, &[]);
        pass.set_bind_group(1, &uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    renderer.set_render_materials_last_summary(refractive_material_summary(
        candidates,
        decisions,
        MaterialPassState::Active {
            candidates: active_candidates.len(),
            generated_masks,
            mask_sources,
        },
    ));
    Ok(())
}

fn copy_target_texture(
    _renderer: &mut WgpuSceneRenderer,
    source: &WgpuOffscreenTarget,
    target: &mut WgpuOffscreenTarget,
) -> AmigoResult<()> {
    let mut encoder = target
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("amigo-refractive-material-source-copy-encoder"),
        });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &source.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &target.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: source.width,
            height: source.height,
            depth_or_array_layers: 1,
        },
    );
    target.queue.submit(Some(encoder.finish()));
    Ok(())
}

fn aggregate_uniform(
    target: &WgpuOffscreenTarget,
    candidates: &[&WgpuMaterialCandidate2d],
) -> RefractiveMaterialUniform {
    let material = candidates
        .last()
        .map(|candidate| candidate.common.material)
        .unwrap_or_else(Material2d::default);
    RefractiveMaterialUniform {
        resolution: [target.width as f32, target.height as f32],
        transmission: material.optical.transmission,
        refraction_px: material.optical.refraction_px,
        distortion: material.optical.distortion,
        dispersion: material.optical.dispersion,
        roughness: material.optical.roughness,
        edge_boost: material.optical.edge_boost,
        opacity: candidates
            .iter()
            .map(|candidate| candidate.common.layer_opacity)
            .fold(0.0, f32::max),
        highlight: material
            .camera_response
            .glare
            .max(material.camera_response.intensity),
    }
}

enum MaterialPassState {
    Active {
        candidates: usize,
        generated_masks: usize,
        mask_sources: Vec<&'static str>,
    },
    Inactive(&'static str),
}

fn refractive_material_summary(
    candidates: &[WgpuMaterialCandidate2d],
    decisions: &[MaterialCandidateDecision2d],
    state: MaterialPassState,
) -> String {
    let refractive = candidates
        .iter()
        .filter(|candidate| candidate.is_refractive())
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    lines.push("render.materials:".to_owned());
    lines.push(String::new());
    lines.push("refractive_material_2d:".to_owned());

    match &state {
        MaterialPassState::Active {
            candidates,
            generated_masks,
            mask_sources,
        } => {
            lines.push("active=true".to_owned());
            lines.push(format!("candidates={candidates}"));
            lines.push(format!("generated_masks={generated_masks}"));
            lines.push("mask=input_ok".to_owned());
            lines.push(format!("mask_source={}", mask_sources.join("+")));
            lines.push("scene_color=input_ok".to_owned());
            lines.push("output=composited_scene_color".to_owned());
        }
        MaterialPassState::Inactive(reason) => {
            lines.push("active=false".to_owned());
            lines.push(format!("reason={reason}"));
            lines.push(format!("candidates={}", refractive.len()));
        }
    }

    for candidate in candidates {
        let optical = candidate.common.material.optical;
        let pass_state =
            if candidate.is_refractive() && matches!(state, MaterialPassState::Active { .. }) {
                "active"
            } else {
                "skipped"
            };
        let reason = decisions
            .iter()
            .find(|decision| {
                decision.owner == candidate.common.owner
                    && decision.component_kind == candidate.common.component_kind
                    && decision.render_layer == candidate.common.render_layer
                    && decision.coverage_kind == candidate.common.coverage_kind
            })
            .map(|decision| decision.reason.as_str())
            .unwrap_or("no_decision");

        lines.push(String::new());
        lines.push(format!(
            "entity={} component={} layer={}",
            candidate.common.owner, candidate.common.component_kind, candidate.common.render_layer
        ));
        lines.push("material=optical.refractive".to_owned());
        lines.push(format!("coverage_source={}", candidate.coverage_label()));
        lines.push(format!("transmission={:.2}", optical.transmission));
        lines.push(format!("refraction_px={:.2}", optical.refraction_px));
        lines.push(format!("distortion={:.2}", optical.distortion));
        lines.push(format!(
            "highlight={:.2}",
            candidate
                .common
                .material
                .camera_response
                .glare
                .max(candidate.common.material.camera_response.intensity)
        ));
        lines.push(format!(
            "layer_opacity={:.2}",
            candidate.common.layer_opacity
        ));
        lines.push(format!("pass={pass_state}"));
        lines.push(format!("reason={reason}"));
    }

    for decision in decisions {
        if decision.status != MaterialCandidateStatus2d::Skipped {
            continue;
        }
        if candidates.iter().any(|candidate| {
            candidate.common.owner == decision.owner
                && candidate.common.component_kind == decision.component_kind
                && candidate.common.render_layer == decision.render_layer
                && candidate.common.coverage_kind == decision.coverage_kind
        }) {
            continue;
        }

        lines.push(String::new());
        lines.push(format!(
            "entity={} component={} layer={}",
            decision.owner, decision.component_kind, decision.render_layer
        ));
        lines.push(format!(
            "coverage_source={}",
            decision.coverage_kind.as_str()
        ));
        lines.push("pass=skipped".to_owned());
        lines.push(format!("reason={}", decision.reason));
    }

    lines.join("\n")
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_assets::AssetKey;
    use amigo_camera::CameraOpticalResponse2d;
    use amigo_material_api::{
        Material2dOptical, Material2dOpticalMode, MaterialCandidate2dCommon, MaterialCoverageKind2d,
    };
    use amigo_math::Transform2;

    #[test]
    fn refractive_material_summary_reports_real_composite_inputs() {
        let summary = refractive_material_summary(
            &[candidate(
                "title",
                "component",
                "title.depth2d",
                MaterialCoverageKind2d::Glyphs,
            )],
            &[MaterialCandidateDecision2d::active(
                "title",
                "component",
                "title.depth2d",
                MaterialCoverageKind2d::Glyphs,
                "material_pipeline_enabled",
            )],
            MaterialPassState::Active {
                candidates: 1,
                generated_masks: 0,
                mask_sources: vec!["ttf_font"],
            },
        );

        assert!(summary.contains("active=true"));
        assert!(summary.contains("candidates=1"));
        assert!(summary.contains("generated_masks=0"));
        assert!(summary.contains("mask=input_ok"));
        assert!(summary.contains("mask_source=ttf_font"));
        assert!(summary.contains("scene_color=input_ok"));
        assert!(summary.contains("output=composited_scene_color"));
        assert!(summary.contains("entity=title component=component layer=title.depth2d"));
        assert!(summary.contains("highlight=0.46"));
    }

    #[test]
    fn refractive_material_summary_reports_inactive_reason() {
        let summary = refractive_material_summary(
            &[],
            &[],
            MaterialPassState::Inactive("no_refractive_candidates"),
        );

        assert!(summary.contains("active=false"));
        assert!(summary.contains("reason=no_refractive_candidates"));
    }

    #[test]
    fn refractive_material_summary_reports_texture_alpha_candidate() {
        let summary = refractive_material_summary(
            &[candidate(
                "poster",
                "component",
                "foreground.props",
                MaterialCoverageKind2d::TextureAlpha,
            )],
            &[MaterialCandidateDecision2d::active(
                "poster",
                "component",
                "foreground.props",
                MaterialCoverageKind2d::TextureAlpha,
                "material_pipeline_enabled",
            )],
            MaterialPassState::Active {
                candidates: 1,
                generated_masks: 0,
                mask_sources: vec!["texture_alpha"],
            },
        );

        assert!(summary.contains("entity=poster component=component layer=foreground.props"));
        assert!(summary.contains("coverage_source=texture_alpha"));
    }

    #[test]
    fn refractive_material_summary_reports_vector_coverage_candidate() {
        let summary = refractive_material_summary(
            &[candidate(
                "vector-glass",
                "component",
                "foreground.props",
                MaterialCoverageKind2d::VectorCoverage,
            )],
            &[MaterialCandidateDecision2d::active(
                "vector-glass",
                "component",
                "foreground.props",
                MaterialCoverageKind2d::VectorCoverage,
                "material_pipeline_enabled",
            )],
            MaterialPassState::Active {
                candidates: 1,
                generated_masks: 0,
                mask_sources: vec!["vector_coverage"],
            },
        );

        assert!(summary.contains("entity=vector-glass component=component layer=foreground.props"));
        assert!(summary.contains("coverage_source=vector_coverage"));
    }

    #[test]
    fn refractive_material_summary_reports_skipped_reason_for_disabled_role() {
        let summary = refractive_material_summary(
            &[],
            &[MaterialCandidateDecision2d::skipped(
                "title",
                "component",
                "title.depth2d",
                MaterialCoverageKind2d::Glyphs,
                "material_pipeline_role_disabled",
            )],
            MaterialPassState::Inactive("no_refractive_candidates"),
        );

        assert!(summary.contains("entity=title component=component layer=title.depth2d"));
        assert!(summary.contains("pass=skipped"));
        assert!(summary.contains("reason=material_pipeline_role_disabled"));
    }

    #[test]
    fn refractive_material_summary_reports_out_of_scope_layered_image_reason() {
        let summary = refractive_material_summary(
            &[],
            &[MaterialCandidateDecision2d::skipped(
                "poster-stack",
                "component",
                "foreground.props",
                MaterialCoverageKind2d::LayeredImageAlpha,
                "material_pipeline_out_of_scope_v1",
            )],
            MaterialPassState::Inactive("no_refractive_candidates"),
        );

        assert!(summary.contains("entity=poster-stack component=component layer=foreground.props"));
        assert!(summary.contains("coverage_source=layered_image_alpha"));
        assert!(summary.contains("reason=material_pipeline_out_of_scope_v1"));
    }

    #[test]
    fn refractive_material_summary_reports_particle_material_not_mapped_reason() {
        let summary = refractive_material_summary(
            &[],
            &[MaterialCandidateDecision2d::skipped(
                "rain",
                "component",
                "weather.rain.near",
                MaterialCoverageKind2d::ParticleCoverage,
                "particle_material_not_mapped_to_material2d",
            )],
            MaterialPassState::Inactive("no_refractive_candidates"),
        );

        assert!(summary.contains("entity=rain component=component layer=weather.rain.near"));
        assert!(summary.contains("coverage_source=particle_coverage"));
        assert!(summary.contains("reason=particle_material_not_mapped_to_material2d"));
    }

    fn candidate(
        owner: &str,
        component_kind: &str,
        render_layer: &str,
        coverage_kind: MaterialCoverageKind2d,
    ) -> WgpuMaterialCandidate2d {
        use amigo_render_api::RenderPrimitive2d as Primitive;

        let primitive = Primitive::GlyphRun(amigo_render_api::GlyphRun2dPrimitive {
            font: AssetKey::new("test/font"),
            text: owner.to_owned(),
            bounds: amigo_math::Vec2::new(100.0, 40.0),
            transform: Transform2::default(),
            color: ColorRgba::WHITE,
            font_size: None,
            blend: amigo_render_api::GlyphRun2dBlendMode::Alpha,
            shadow: None,
            outline: None,
            glow: None,
            material: amigo_render_api::RenderMaterialBinding2d::new(
                Some(refractive_material()),
                amigo_render_api::RenderContributionSet::default(),
                coverage_kind,
            ),
        });
        WgpuMaterialCandidate2d {
            common: MaterialCandidate2dCommon {
                owner: owner.to_owned(),
                component_kind: component_kind.to_owned(),
                render_layer: render_layer.to_owned(),
                z_index: 40.0,
                layer_opacity: 0.72,
                visible: true,
                material: refractive_material(),
                coverage_kind,
            },
            source: super::material_candidates::MaterialCandidateSource2d {
                renderable: Renderable2dItem::new(
                    amigo_render_api::Renderable2dCommon {
                        source_id: amigo_render_api::RenderSourceId::for_component(
                            owner,
                            component_kind,
                        ),
                        object_id: amigo_render_api::RenderObjectId::for_scene_object(owner),
                        owner_entity: owner.to_owned(),
                        component_kind: component_kind.to_owned(),
                        render_space: amigo_render_api::RenderSpace2d::World,
                        render_layer: render_layer.to_owned(),
                        z_index: 40.0,
                        kind: amigo_render_api::Renderable2dKind::Text,
                        overlay_visible: false,
                    },
                    primitive,
                ),
            },
            camera: Transform2::default(),
        }
    }

    fn refractive_material() -> Material2d {
        Material2d {
            optical: Material2dOptical {
                mode: Material2dOpticalMode::Refractive,
                transmission: 0.58,
                refraction_px: 4.5,
                distortion: 0.22,
                dispersion: 0.08,
                roughness: 0.32,
                edge_boost: 0.45,
            },
            camera_response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.46,
                glare: 0.46,
                ..Default::default()
            },
            ..Default::default()
        }
        .normalized()
    }
}
