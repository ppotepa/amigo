use super::*;
use amigo_material_api::{
    Material2d, MaterialCandidate2dCommon, MaterialCandidateDecision2d, MaterialCoverageKind2d,
};
use amigo_render_api::{
    render_contribution_roles as roles, RenderContributionSet, RenderMaterialBinding2d,
    RenderPrimitive2dKind,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct MaterialCandidateSource2d {
    pub(super) renderable: Renderable2dItem,
}

#[derive(Debug, Clone)]
pub(crate) struct WgpuMaterialCandidate2d {
    pub(super) common: MaterialCandidate2dCommon,
    pub(super) source: MaterialCandidateSource2d,
    pub(super) camera: Transform2,
}

impl WgpuMaterialCandidate2d {
    pub(crate) fn coverage_label(&self) -> &'static str {
        self.common.coverage_kind.as_str()
    }

    pub(crate) fn is_refractive(&self) -> bool {
        self.common.is_refractive()
    }
}

pub(crate) fn collect_material_candidate_2d(
    item: &Renderable2dItem,
    layer_camera: Transform2,
    layer_opacity: f32,
    out: &mut Vec<WgpuMaterialCandidate2d>,
    decisions: &mut Vec<MaterialCandidateDecision2d>,
) {
    if let Some(binding) = item.primitive.material_binding() {
        collect_candidate_from_binding(
            item,
            binding,
            layer_camera,
            layer_opacity,
            out,
            decisions,
        );
        return;
    }

    match item.primitive.kind() {
        RenderPrimitive2dKind::LayeredTexturedQuads => {
            decisions.push(MaterialCandidateDecision2d::skipped(
                item.source_id().as_str(),
                item.component_kind(),
                item.render_layer(),
                MaterialCoverageKind2d::LayeredImageAlpha,
                "material_pipeline_out_of_scope_v1",
            ))
        }
        RenderPrimitive2dKind::ParticleBatch => {
            decisions.push(MaterialCandidateDecision2d::skipped(
                item.source_id().as_str(),
                item.component_kind(),
                item.render_layer(),
                MaterialCoverageKind2d::ParticleCoverage,
                "particle_material_not_mapped_to_material2d",
            ))
        }
        _ => {}
    }
}

pub(super) fn material_pipeline_enabled(
    contributions: &RenderContributionSet,
    material: Material2d,
) -> bool {
    if !material.requires_material_mask() {
        return false;
    }

    contributions.enabled_or(roles::MATERIAL_MASK, false)
        || contributions.enabled_or(roles::OPTICS_REFRACT, false)
        || contributions.enabled_or(roles::BLOOM_SOURCE, false)
        || contributions.enabled_or(roles::CAMERA_FX_SOURCE, false)
}

fn collect_candidate_from_binding(
    item: &Renderable2dItem,
    binding: &RenderMaterialBinding2d,
    camera: Transform2,
    layer_opacity: f32,
    out: &mut Vec<WgpuMaterialCandidate2d>,
    decisions: &mut Vec<MaterialCandidateDecision2d>,
) {
    let material = binding.material;
    let contributions = &binding.contributions;
    let coverage_kind = binding.coverage_kind;
    let Some(material) = material else {
        return;
    };

    if !material.requires_material_mask() {
        decisions.push(MaterialCandidateDecision2d::skipped(
            item.source_id().as_str(),
            item.component_kind(),
            item.render_layer(),
            coverage_kind,
            "material_does_not_require_mask",
        ));
        return;
    }

    if !material_pipeline_enabled(contributions, material) {
        decisions.push(MaterialCandidateDecision2d::skipped(
            item.source_id().as_str(),
            item.component_kind(),
            item.render_layer(),
            coverage_kind,
            "material_pipeline_role_disabled",
        ));
        return;
    }

    let visible = layer_opacity > 0.001;
    let common = MaterialCandidate2dCommon {
        owner: item.source_id().as_str().to_owned(),
        component_kind: item.component_kind().to_owned(),
        render_layer: item.render_layer().to_owned(),
        z_index: item.z_index(),
        layer_opacity,
        visible,
        material,
        coverage_kind,
    };

    if !visible {
        decisions.push(MaterialCandidateDecision2d::skipped(
            &common.owner,
            &common.component_kind,
            &common.render_layer,
            common.coverage_kind,
            "layer_hidden_or_zero_opacity",
        ));
        return;
    }

    decisions.push(MaterialCandidateDecision2d::active(
        &common.owner,
        &common.component_kind,
        &common.render_layer,
        common.coverage_kind,
        "material_pipeline_enabled",
    ));
    out.push(WgpuMaterialCandidate2d {
        common,
        source: MaterialCandidateSource2d {
            renderable: item.clone(),
        },
        camera,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_assets::AssetKey;
    use amigo_math::{ColorRgba, Transform2, Vec2};
    use amigo_render_api::{
        GlyphRun2dBlendMode, GlyphRun2dPrimitive, LayeredImage2dPrimitive,
        LayeredImageViewportFit2dPrimitive, Particle2dPrimitive, ParticleBlendMode2dPrimitive,
        ParticleLineAnchor2dPrimitive, ParticleShape2dPrimitive, RenderPrimitive2d, RenderSpace2d,
        Renderable2dCommon, Renderable2dItem, Renderable2dKind, TexturedQuad2dPrimitive,
        VectorShape2dKindPrimitive, VectorShape2dPrimitive, VectorShape2dStylePrimitive,
        VectorShape2dViewportFit,
    };

    #[test]
    fn text_material_candidate_requires_material_mask_or_optics_refract() {
        let item = text_item();
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert!(out.is_empty());
        assert_eq!(decisions[0].reason, "material_pipeline_role_disabled");
    }

    #[test]
    fn sprite_material_candidate_uses_same_role_gating_as_text() {
        let mut item = sprite_item();
        if let RenderPrimitive2d::TexturedQuad(primitive) = &mut item.primitive {
            primitive
                .material
                .contributions
                .set(roles::OPTICS_REFRACT, true);
        }
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].coverage_label(), "texture_alpha");
    }

    #[test]
    fn vector_material_candidate_uses_same_role_gating_as_text() {
        let mut item = vector_item();
        if let RenderPrimitive2d::VectorMesh(primitive) = &mut item.primitive {
            primitive
                .material
                .contributions
                .set(roles::MATERIAL_MASK, true);
        }
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].coverage_label(), "vector_coverage");
    }

    #[test]
    fn layered_image_material_path_reports_out_of_scope_v1() {
        let item = Renderable2dItem::new(
            common(
                "poster-stack",
                "LayeredImage2D",
                Renderable2dKind::LayeredImage,
            ),
            RenderPrimitive2d::LayeredTexturedQuads(LayeredImage2dPrimitive {
                asset: AssetKey::new("test/poster-stack"),
                size: Vec2::new(64.0, 64.0),
                base_opacity: 1.0,
                viewport_fit: LayeredImageViewportFit2dPrimitive::Fixed,
                transform: Transform2::default(),
                visual_maps: None,
                layer_overrides: Vec::new(),
            }),
        );
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert!(out.is_empty());
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].reason, "material_pipeline_out_of_scope_v1");
    }

    #[test]
    fn particle_material_path_reports_not_mapped_to_material2d() {
        let item = Renderable2dItem::new(
            common("rain", "component", Renderable2dKind::Particle),
            RenderPrimitive2d::ParticleBatch(Particle2dPrimitive {
                emitter_entity_name: "rain".to_owned(),
                render_layer: "default".to_owned(),
                position: Vec2::ZERO,
                previous_position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                size: 1.0,
                color: ColorRgba::WHITE,
                shape: ParticleShape2dPrimitive::Quad,
                line_anchor: ParticleLineAnchor2dPrimitive::Center,
                blend_mode: ParticleBlendMode2dPrimitive::Alpha,
                motion_stretch: None,
                material: amigo_render_api::ParticleMaterial2dPrimitive {
                    lighting_mode: amigo_render_api::ParticleMaterialLightingMode2dPrimitive::Unlit,
                    receives_light: false,
                    light_response: 0.0,
                    light_receiver: None,
                },
                light: None,
                light_position: None,
                transform: Transform2::default(),
            }),
        );
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert!(out.is_empty());
        assert_eq!(decisions.len(), 1);
        assert_eq!(
            decisions[0].reason,
            "particle_material_not_mapped_to_material2d"
        );
    }

    fn text_item() -> Renderable2dItem {
        Renderable2dItem::new(
            common("title", "component", Renderable2dKind::Text),
            RenderPrimitive2d::GlyphRun(GlyphRun2dPrimitive {
                font: AssetKey::new("test/font"),
                text: "ROTTEN CLUB".to_owned(),
                bounds: Vec2::new(100.0, 40.0),
                transform: Transform2::default(),
                color: ColorRgba::WHITE,
                font_size: None,
                blend: GlyphRun2dBlendMode::Alpha,
                shadow: None,
                outline: None,
                glow: None,
                material: RenderMaterialBinding2d::new(
                    Some(refractive_material()),
                    RenderContributionSet::default(),
                    MaterialCoverageKind2d::Glyphs,
                ),
            }),
        )
    }

    fn sprite_item() -> Renderable2dItem {
        Renderable2dItem::new(
            common("poster", "component", Renderable2dKind::Sprite),
            RenderPrimitive2d::TexturedQuad(TexturedQuad2dPrimitive {
                texture: AssetKey::new("test/poster"),
                size: Vec2::new(32.0, 32.0),
                transform: Transform2::default(),
                visual_maps: None,
                sheet: None,
                frame_index: 0,
                material: RenderMaterialBinding2d::new(
                    Some(refractive_material()),
                    RenderContributionSet::default(),
                    MaterialCoverageKind2d::TextureAlpha,
                ),
            }),
        )
    }

    fn vector_item() -> Renderable2dItem {
        Renderable2dItem::new(
            common("glass", "component", Renderable2dKind::Vector),
            RenderPrimitive2d::VectorMesh(VectorShape2dPrimitive {
                shape: VectorShape2dKindPrimitive::Circle {
                    radius: 10.0,
                    segments: 8,
                },
                style: VectorShape2dStylePrimitive {
                    stroke_color: ColorRgba::WHITE,
                    stroke_width: 0.0,
                    fill_color: Some(ColorRgba::WHITE),
                },
                transform: Transform2::default(),
                viewport_fit: VectorShape2dViewportFit::Fixed,
                viewport_canvas_size: None,
                material: RenderMaterialBinding2d::new(
                    Some(refractive_material()),
                    RenderContributionSet::default(),
                    MaterialCoverageKind2d::VectorCoverage,
                ),
            }),
        )
    }

    fn common(owner: &str, component_kind: &str, kind: Renderable2dKind) -> Renderable2dCommon {
        Renderable2dCommon {
            source_id: amigo_render_api::RenderSourceId::for_component(owner, component_kind),
            object_id: amigo_render_api::RenderObjectId::for_scene_object(owner),
            owner_entity: owner.to_owned(),
            component_kind: component_kind.to_owned(),
            render_space: RenderSpace2d::World,
            render_layer: "foreground.props".to_owned(),
            z_index: 0.0,
            kind,
            overlay_visible: false,
        }
    }

    fn refractive_material() -> Material2d {
        Material2d {
            optical: amigo_material_api::Material2dOptical {
                mode: amigo_material_api::Material2dOpticalMode::Refractive,
                transmission: 0.5,
                refraction_px: 4.0,
                distortion: 0.2,
                dispersion: 0.1,
                roughness: 0.0,
                edge_boost: 0.0,
            },
            ..Default::default()
        }
        .normalized()
    }
}
