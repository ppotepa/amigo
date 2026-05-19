use super::*;
use amigo_material_2d_plugin::{
    Material2d, MaterialCandidate2dCommon, MaterialCandidateDecision2d, MaterialCoverageKind2d,
};
use amigo_render_api::{
    render_contribution_roles as roles, RenderContributionSet,
};

#[derive(Debug, Clone)]
pub(super) enum MaterialCoveragePayload2d {
    Text(amigo_text_2d_plugin::Text2dDrawCommand),
    Sprite(amigo_sprite_2d_plugin::SpriteDrawCommand),
    Vector(amigo_vector_2d_plugin::VectorShape2dDrawCommand),
}

#[derive(Debug, Clone)]
pub(super) struct WgpuMaterialCandidate2d {
    pub(super) common: MaterialCandidate2dCommon,
    pub(super) payload: MaterialCoveragePayload2d,
    pub(super) camera: Transform2,
}

impl WgpuMaterialCandidate2d {
    pub(super) fn coverage_label(&self) -> &'static str {
        self.common.coverage_kind.as_str()
    }

    pub(super) fn is_refractive(&self) -> bool {
        self.common.is_refractive()
    }
}

pub(super) fn collect_material_candidate_2d(
    item: &Renderable2dItem,
    layer_camera: Transform2,
    layer_opacity: f32,
    out: &mut Vec<WgpuMaterialCandidate2d>,
    decisions: &mut Vec<MaterialCandidateDecision2d>,
) {
    match &item.payload {
        Renderable2dPayload::Text(command) => collect_candidate_from_parts(
            item,
            command.material,
            &command.render_contributions,
            MaterialCoverageKind2d::Glyphs,
            MaterialCoveragePayload2d::Text(command.clone()),
            layer_camera,
            layer_opacity,
            out,
            decisions,
        ),
        Renderable2dPayload::Sprite(command) => collect_candidate_from_parts(
            item,
            command.material,
            &command.render_contributions,
            MaterialCoverageKind2d::TextureAlpha,
            MaterialCoveragePayload2d::Sprite(command.clone()),
            layer_camera,
            layer_opacity,
            out,
            decisions,
        ),
        Renderable2dPayload::Vector(command) => collect_candidate_from_parts(
            item,
            command.material,
            &command.render_contributions,
            MaterialCoverageKind2d::VectorCoverage,
            MaterialCoveragePayload2d::Vector(command.clone()),
            layer_camera,
            layer_opacity,
            out,
            decisions,
        ),
        Renderable2dPayload::LayeredImage(_) => {
            decisions.push(MaterialCandidateDecision2d::skipped(
                item.owner_entity(),
                item.component_kind(),
                item.render_layer(),
                MaterialCoverageKind2d::LayeredImageAlpha,
                "material_pipeline_out_of_scope_v1",
            ))
        }
        Renderable2dPayload::Particle(_) => decisions.push(MaterialCandidateDecision2d::skipped(
            item.owner_entity(),
            item.component_kind(),
            item.render_layer(),
            MaterialCoverageKind2d::ParticleCoverage,
            "particle_material_not_mapped_to_material2d",
        )),
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

fn collect_candidate_from_parts(
    item: &Renderable2dItem,
    material: Option<Material2d>,
    contributions: &RenderContributionSet,
    coverage_kind: MaterialCoverageKind2d,
    payload: MaterialCoveragePayload2d,
    camera: Transform2,
    layer_opacity: f32,
    out: &mut Vec<WgpuMaterialCandidate2d>,
    decisions: &mut Vec<MaterialCandidateDecision2d>,
) {
    let Some(material) = material else {
        return;
    };

    if !material.requires_material_mask() {
        decisions.push(MaterialCandidateDecision2d::skipped(
            item.owner_entity(),
            item.component_kind(),
            item.render_layer(),
            coverage_kind,
            "material_does_not_require_mask",
        ));
        return;
    }

    if !material_pipeline_enabled(contributions, material) {
        decisions.push(MaterialCandidateDecision2d::skipped(
            item.owner_entity(),
            item.component_kind(),
            item.render_layer(),
            coverage_kind,
            "material_pipeline_role_disabled",
        ));
        return;
    }

    let visible = layer_opacity > 0.001;
    let common = MaterialCandidate2dCommon {
        owner: item.owner_entity().to_owned(),
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
        payload,
        camera,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_assets::AssetKey;
    use amigo_math::{ColorRgba, Transform2, Vec2};
    use amigo_render_api::{RenderSpace2d, Renderable2dCommon, Renderable2dKind};
    use amigo_scene::SceneEntityId;

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
        if let Renderable2dPayload::Sprite(command) = &mut item.payload {
            command
                .render_contributions
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
        if let Renderable2dPayload::Vector(command) = &mut item.payload {
            command.render_contributions.set(roles::MATERIAL_MASK, true);
        }
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].coverage_label(), "vector_coverage");
    }

    #[test]
    fn layered_image_material_path_reports_out_of_scope_v1() {
        let item = Renderable2dItem {
            common: common(
                "poster-stack",
                "LayeredImage2D",
                Renderable2dKind::LayeredImage,
            ),
            payload: Renderable2dPayload::LayeredImage(
                amigo_layered_image_2d_plugin::LayeredImageDrawCommand {
                    entity_id: SceneEntityId::new(1),
                    entity_name: "poster-stack".to_owned(),
                    render_layer: "foreground.props".to_owned(),
                    image: amigo_layered_image_2d_plugin::LayeredImageInstance {
                        asset: AssetKey::new("test/poster-stack"),
                        size: Vec2::new(64.0, 64.0),
                        base_opacity: 1.0,
                        viewport_fit: amigo_layered_image_2d_plugin::LayeredImageViewportFit2d::Fixed,
                        visual_maps: None,
                        layer_overrides: Vec::new(),
                    },
                    z_index: 0.0,
                    transform: Transform2::default(),
                },
            ),
        };
        let mut out = Vec::new();
        let mut decisions = Vec::new();

        collect_material_candidate_2d(&item, Transform2::default(), 1.0, &mut out, &mut decisions);

        assert!(out.is_empty());
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].reason, "material_pipeline_out_of_scope_v1");
    }

    #[test]
    fn particle_material_path_reports_not_mapped_to_material2d() {
        let item = Renderable2dItem {
            common: common("rain", "ParticleEmitter2D", Renderable2dKind::Particle),
            payload: Renderable2dPayload::Particle(amigo_particles_2d_plugin::Particle2dDrawCommand {
                emitter_entity_name: "rain".to_owned(),
                previous_position: Vec2::ZERO,
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                size: 1.0,
                color: ColorRgba::WHITE,
                render_layer: "weather.rain.near".to_owned(),
                z_index: 0.0,
                shape: amigo_particles_2d_plugin::ParticleShape2d::Quad,
                line_anchor: amigo_particles_2d_plugin::ParticleLineAnchor2d::Center,
                blend_mode: amigo_particles_2d_plugin::ParticleBlendMode2d::Alpha,
                motion_stretch: None,
                material: amigo_particles_2d_plugin::ParticleMaterial2d {
                    lighting_mode: amigo_light_2d_plugin::Material2dLightingMode::Unlit,
                    receives_light: false,
                    light_response: 0.0,
                    light_receiver: None,
                },
                light: None,
                light_position: None,
                transform: Transform2::default(),
            }),
        };
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
        Renderable2dItem {
            common: common("title", "Text2D", Renderable2dKind::Text),
            payload: Renderable2dPayload::Text(amigo_text_2d_plugin::Text2dDrawCommand {
                entity_id: SceneEntityId::new(1),
                entity_name: "title".to_owned(),
                render_layer: "title.depth2d".to_owned(),
                text: amigo_text_2d_plugin::Text2d {
                    content: "ROTTEN CLUB".to_owned(),
                    font: AssetKey::new("test/font"),
                    bounds: Vec2::new(100.0, 40.0),
                    transform: Transform2::default(),
                    style: amigo_text_2d_plugin::Text2dStyle {
                        color: ColorRgba::new(1.0, 1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    post_fx_host_id: None,
                },
                z_index: 0.0,
                material: Some(refractive_material()),
                render_contributions: RenderContributionSet::default(),
            }),
        }
    }

    fn sprite_item() -> Renderable2dItem {
        Renderable2dItem {
            common: common("poster", "Sprite2D", Renderable2dKind::Sprite),
            payload: Renderable2dPayload::Sprite(amigo_sprite_2d_plugin::SpriteDrawCommand {
                entity_id: SceneEntityId::new(1),
                entity_name: "poster".to_owned(),
                render_layer: "foreground.props".to_owned(),
                sprite: amigo_sprite_2d_plugin::Sprite {
                    texture: AssetKey::new("test/poster"),
                    size: Vec2::new(32.0, 32.0),
                    sheet: None,
                    sheet_is_explicit: false,
                    animation_override: None,
                    visual_maps: None,
                    frame_index: 0,
                    frame_elapsed: 0.0,
                },
                z_index: 0.0,
                transform: Transform2::default(),
                material: Some(refractive_material()),
                render_contributions: RenderContributionSet::default(),
            }),
        }
    }

    fn vector_item() -> Renderable2dItem {
        Renderable2dItem {
            common: common("glass", "VectorShape2D", Renderable2dKind::Vector),
            payload: Renderable2dPayload::Vector(amigo_vector_2d_plugin::VectorShape2dDrawCommand {
                entity_id: SceneEntityId::new(1),
                entity_name: "glass".to_owned(),
                render_layer: "foreground.props".to_owned(),
                shape: amigo_vector_2d_plugin::VectorShape2d {
                    kind: amigo_vector_2d_plugin::VectorShapeKind2d::Circle {
                        radius: 10.0,
                        segments: 8,
                    },
                    style: amigo_vector_2d_plugin::VectorStyle2d {
                        stroke_color: ColorRgba::WHITE,
                        stroke_width: 0.0,
                        fill_color: Some(ColorRgba::new(1.0, 1.0, 1.0, 1.0)),
                    },
                },
                z_index: 0.0,
                transform: Transform2::default(),
                viewport_fit: amigo_vector_2d_plugin::VectorViewportFit2d::Fixed,
                viewport_canvas_size: None,
                material: Some(refractive_material()),
                render_contributions: RenderContributionSet::default(),
            }),
        }
    }

    fn common(owner: &str, component_kind: &str, kind: Renderable2dKind) -> Renderable2dCommon {
        Renderable2dCommon {
            owner_entity: owner.to_owned(),
            component_kind: component_kind.to_owned(),
            render_space: RenderSpace2d::World,
            render_layer: "foreground.props".to_owned(),
            z_index: 0.0,
            kind,
        }
    }

    fn refractive_material() -> Material2d {
        Material2d {
            optical: amigo_material_2d_plugin::Material2dOptical {
                mode: amigo_material_2d_plugin::Material2dOpticalMode::Refractive,
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
