use amigo_camera_optics_plugin::api::{
    CameraOpticalCandidate2d, CameraOpticalEmitterKind2d, CameraOpticalResponse2d,
    CameraOpticalSource2d, CameraOpticalSourceStatus2d,
};
use amigo_render_api::{
    CameraCaptureInput2d, LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon,
    LightSourceStatus2d, RenderContributionSet, VisualSourceAvailability2d,
};
use amigo_material_2d_plugin::Material2d;

use super::visual_2d_items::{Renderable2dItem, Renderable2dPayload};

pub fn collect_light_sources_2d(
    renderables: &[Renderable2dItem],
    beacons: &[amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand],
    global_lights: &[amigo_light_2d_plugin::GlobalLight2dCommand],
    lightmaps: &[amigo_light_2d_plugin::LightMap2dSourceCommand],
    light_groups: &[amigo_light_2d_plugin::LightGroup2dCommand],
    particles: &[amigo_particles_2d_plugin::Particle2dDrawCommand],
    camera_capture_input: Option<&CameraCaptureInput2d>,
) -> Vec<LightSource2dCommon> {
    let mut sources = Vec::new();
    collect_material_emissive_light_sources(renderables, &mut sources);

    for beacon in beacons.iter().take(32) {
        let position_px = Some([beacon.center.x, beacon.center.y]);
        let mut contributions = Vec::new();
        if beacon
            .render_contributions
            .enabled_or(amigo_render_api::render_contribution_roles::RELIGHT_PLATE, true)
        {
            contributions.push(LightContributionKind2d::RelightPlate);
        }
        if beacon
            .render_contributions
            .enabled_or(amigo_render_api::render_contribution_roles::BLOOM_SOURCE, true)
        {
            contributions.push(LightContributionKind2d::BloomSource);
        }
        if beacon
            .render_contributions
            .enabled_or(amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE, true)
        {
            contributions.push(LightContributionKind2d::CameraFxSource);
        }
        let status = if contributions.is_empty() {
            LightSourceStatus2d::Skipped
        } else {
            LightSourceStatus2d::Active
        };
        let reason = if contributions.is_empty() {
            "all_light_roles_disabled".to_owned()
        } else {
            "active_light_emitter".to_owned()
        };

        let source = match status {
            LightSourceStatus2d::Active => LightSource2dCommon::active(
                beacon.entity_name.clone(),
                "BeaconLight2D",
                LightEmitterKind2d::Beacon,
                None,
                Some(beacon.render_layer.clone()),
                Some(color_rgba(beacon.color)),
                Some(beacon.intensity),
                Some(beacon.intensity * beacon.color.a),
                Some(1.0),
                Some(beacon.camera_response),
                Some(beacon.bloom),
                Some(beacon.halo_radius_px.max(beacon.core_radius_px)),
                None,
                beacon.distance_m,
                beacon.z_depth,
                contributions,
                reason,
                position_px,
            ),
            LightSourceStatus2d::Skipped => LightSource2dCommon::skipped(
                beacon.entity_name.clone(),
                "BeaconLight2D",
                LightEmitterKind2d::Beacon,
                None,
                Some(beacon.render_layer.clone()),
                Some(color_rgba(beacon.color)),
                Some(beacon.intensity),
                Some(beacon.intensity * beacon.color.a),
                Some(1.0),
                Some(beacon.camera_response),
                Some(beacon.bloom),
                Some(beacon.halo_radius_px.max(beacon.core_radius_px)),
                None,
                beacon.distance_m,
                beacon.z_depth,
                contributions,
                reason,
                position_px,
            ),
        };
        sources.push(source);
    }

    for global_light in global_lights.iter().take(16) {
        sources.push(LightSource2dCommon::active(
            global_light.entity_name.clone(),
            "GlobalLight2D",
            LightEmitterKind2d::GlobalLight,
            Some(global_light.id.clone()),
            None,
            Some(color_rgba(global_light.color)),
            Some(global_light.intensity),
            Some(global_light.intensity * global_light.color.a),
            Some(1.0),
            None,
            None,
            None,
            None,
            None,
            None,
            vec![LightContributionKind2d::LightingEmit],
            "global_light_command",
            None,
        ));
    }

    for lightmap in lightmaps.iter().take(16) {
        if lightmap.channels.is_empty() {
            sources.push(LightSource2dCommon::skipped(
                lightmap.entity_name.clone(),
                "LightMap2D",
                LightEmitterKind2d::LightMapSource,
                Some(lightmap.id.clone()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                vec![LightContributionKind2d::LightingEmit],
                "lightmap_source_without_channels",
                None,
            ));
            continue;
        }

        for channel in lightmap.channels.iter() {
            let layers = if channel.layers.is_empty() {
                "none".to_owned()
            } else {
                channel.layers.join(",")
            };
            sources.push(LightSource2dCommon::active(
                lightmap.entity_name.clone(),
                "LightMap2D",
                LightEmitterKind2d::LightMapChannel,
                Some(format!("{}:{}", lightmap.id, channel.id)),
                None,
                None,
                Some(1.0),
                Some(1.0),
                Some(1.0),
                None,
                None,
                None,
                None,
                None,
                None,
                vec![LightContributionKind2d::LightingEmit],
                format!("lightmap_channel source={} channel={} layers={layers}", lightmap.id, channel.id),
                None,
            ));
        }
    }

    for group in light_groups.iter().take(16) {
        if group.sources.is_empty() {
            sources.push(LightSource2dCommon::skipped(
                group.id.clone(),
                "LightGroup2D",
                LightEmitterKind2d::LightGroup,
                Some(group.id.clone()),
                None,
                Some(color_rgba(group.color)),
                Some(group.intensity),
                Some(0.0),
                Some(1.0),
                Some(group.camera_response),
                None,
                None,
                None,
                None,
                None,
                light_group_contributions(group),
                "light_group_without_sources",
                None,
            ));
            continue;
        }

        for source in group.sources.iter() {
            match &source.kind {
                amigo_light_2d_plugin::LightGroup2dSourceKind::GlobalLight { id } => {
                    let global = global_lights.iter().find(|light| &light.id == id);
                    let (status, effective_intensity, reason) = match global {
                        Some(light) => (
                            LightSourceStatus2d::Active,
                            group.intensity * source.response * light.intensity,
                            format!("light_group_global_light group={} source={id}", group.id),
                        ),
                        None => (
                            LightSourceStatus2d::Skipped,
                            0.0,
                            format!("missing_global_light_source group={} source={id}", group.id),
                        ),
                    };
                    push_light_group_source(
                        &mut sources,
                        group,
                        Some(format!("{}:global:{}", group.id, id)),
                        source.response,
                        effective_intensity,
                        status,
                        reason,
                    );
                }
                amigo_light_2d_plugin::LightGroup2dSourceKind::LightMapChannel {
                    source: source_id,
                    channel,
                } => {
                    let found = lightmaps.iter().any(|lightmap| {
                        &lightmap.id == source_id
                            && lightmap.channels.iter().any(|entry| &entry.id == channel)
                    });
                    let (status, effective_intensity, reason) = if found {
                        (
                            LightSourceStatus2d::Active,
                            group.intensity * source.response,
                            format!(
                                "light_group_lightmap_channel group={} source={} channel={}",
                                group.id, source_id, channel
                            ),
                        )
                    } else {
                        (
                            LightSourceStatus2d::Skipped,
                            0.0,
                            format!(
                                "missing_lightmap_channel_source group={} source={} channel={}",
                                group.id, source_id, channel
                            ),
                        )
                    };
                    push_light_group_source(
                        &mut sources,
                        group,
                        Some(format!("{}:lightmap:{}:{}", group.id, source_id, channel)),
                        source.response,
                        effective_intensity,
                        status,
                        reason,
                    );
                }
            }
        }
    }

    for particle in particles.iter().take(64) {
        if let Some(light) = particle.light {
            let active = light.intensity > 0.001 && particle.color.a > 0.001 && light.radius > 0.001;
            let position = particle.light_position.unwrap_or(particle.position);
            let position_px = Some([position.x, position.y]);
            let common = if active {
                LightSource2dCommon::active(
                    particle.emitter_entity_name.clone(),
                    "ParticleEmitter2D",
                    LightEmitterKind2d::ParticleLight,
                    Some(particle.emitter_entity_name.clone()),
                    Some(particle.render_layer.clone()),
                    Some(color_rgba(particle.color)),
                    Some(light.intensity),
                    Some(light.intensity * particle.color.a),
                    Some(1.0),
                    Some(particle_light_camera_response(light)),
                    None,
                    None,
                    Some(light.radius),
                    None,
                    None,
                    particle_light_contributions(light),
                    "particle_light_active",
                    position_px,
                )
            } else {
                LightSource2dCommon::skipped(
                    particle.emitter_entity_name.clone(),
                    "ParticleEmitter2D",
                    LightEmitterKind2d::ParticleLight,
                    Some(particle.emitter_entity_name.clone()),
                    Some(particle.render_layer.clone()),
                    Some(color_rgba(particle.color)),
                    Some(light.intensity),
                    Some(light.intensity * particle.color.a),
                    Some(1.0),
                    Some(particle_light_camera_response(light)),
                    None,
                    None,
                    Some(light.radius),
                    None,
                    None,
                    particle_light_contributions(light),
                    "particle_light_zero_intensity",
                    position_px,
                )
            };
            sources.push(common);
        }
    }

    if let Some(input) = camera_capture_input {
        if let Some(emissive) = &input.emissive {
            let source = if matches!(emissive.availability, VisualSourceAvailability2d::Missing) {
                LightSource2dCommon::skipped(
                    emissive.id.0.clone(),
                    "SceneEmissive",
                    LightEmitterKind2d::EmissiveVisualSource,
                    Some(emissive.id.0.clone()),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![LightContributionKind2d::EmissiveBuffer],
                    format!(
                        "availability={} origin={:?}",
                        visual_source_availability_label(emissive.availability),
                        emissive.origin
                    ),
                    None,
                )
            } else {
                LightSource2dCommon::active(
                    emissive.id.0.clone(),
                    "SceneEmissive",
                    LightEmitterKind2d::EmissiveVisualSource,
                    Some(emissive.id.0.clone()),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![LightContributionKind2d::EmissiveBuffer],
                    format!(
                        "availability={} origin={:?}",
                        visual_source_availability_label(emissive.availability),
                        emissive.origin
                    ),
                    None,
                )
            };
            sources.push(source);
        }
    }

    sources
}

fn collect_material_emissive_light_sources(
    renderables: &[Renderable2dItem],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for item in renderables.iter().take(64) {
        let Some((material, color_rgba)) = material_light_payload(item) else {
            continue;
        };
        let response = material.camera_response.normalized();
        let has_camera_response = response.enabled
            && (response.intensity > 0.0
                || response.bloom > 0.0
                || response.glare > 0.0
                || response.dirt_response > 0.0
                || response.halation > 0.0);
        if !has_camera_response {
            sources.push(LightSource2dCommon::skipped(
                item.common.owner_entity.clone(),
                item.common.component_kind.clone(),
                LightEmitterKind2d::EmissiveMaterial,
                Some(format!("material:{}", item.common.owner_entity)),
                Some(item.common.render_layer.clone()),
                Some(color_rgba),
                Some(0.0),
                Some(0.0),
                None,
                Some(response),
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                "material_emissive_no_camera_response",
                None,
            ));
            continue;
        }

        let mut contributions = Vec::new();
        if response.bloom > 0.0 || response.intensity > 0.0 {
            contributions.push(LightContributionKind2d::BloomSource);
            contributions.push(LightContributionKind2d::EmissiveBuffer);
        }
        if response.intensity > 0.0
            || response.glare > 0.0
            || response.ghosting > 0.0
            || response.streaks > 0.0
            || response.dirt_response > 0.0
            || response.halation > 0.0
        {
            contributions.push(LightContributionKind2d::CameraFxSource);
        }
        let intensity = response
            .intensity
            .max(response.glare)
            .max(response.bloom)
            .max(response.halation);
        sources.push(LightSource2dCommon::active(
            item.common.owner_entity.clone(),
            item.common.component_kind.clone(),
            LightEmitterKind2d::EmissiveMaterial,
            Some(format!("material:{}", item.common.owner_entity)),
            Some(item.common.render_layer.clone()),
            Some(color_rgba),
            Some(intensity),
            Some(intensity),
            None,
            Some(response),
            Some(if response.bloom > 0.0 || response.intensity > 0.0 {
                intensity
            } else {
                0.0
            }),
            None,
            None,
            None,
            None,
            contributions,
            "material_emissive_camera_response",
            None,
        ));
    }
}

fn material_light_payload(item: &Renderable2dItem) -> Option<(Material2d, [f32; 4])> {
    match &item.payload {
        Renderable2dPayload::Text(command) => Some((
            command.material?,
            [
                command.text.style.color.r,
                command.text.style.color.g,
                command.text.style.color.b,
                command.text.style.color.a * command.text.style.opacity,
            ],
        )),
        Renderable2dPayload::Sprite(command) => Some((command.material?, [1.0, 1.0, 1.0, 1.0])),
        Renderable2dPayload::Vector(command) => Some((
            command.material?,
            color_rgba(
                command
                    .shape
                    .style
                    .fill_color
                    .unwrap_or(command.shape.style.stroke_color),
            ),
        )),
        _ => None,
    }
}

pub fn format_light_sources_2d(sources: &[LightSource2dCommon]) -> String {
    let mut lines = vec!["render.light.sources:".to_owned()];

    if sources.is_empty() {
        lines.push("none".to_owned());
        return lines.join("\n");
    }

    for source in sources {
        let contributions = if source.contributions.is_empty() {
            "none".to_owned()
        } else {
            source
                .contributions
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join(",")
        };
        let used_by = if source.status == LightSourceStatus2d::Active
            && source
                .contributions
                .contains(&LightContributionKind2d::RelightPlate)
        {
            "plate_relight"
        } else {
            "none"
        };

        lines.push(format!(
            "owner={} component={} emitter={} status={} reason={}",
            source.owner,
            source.component_kind,
            source.emitter_kind.as_str(),
            source.status.as_str(),
            source.reason
        ));
        lines.push(format!(
            "id={} layer={} color={} intensity={} effective_intensity={} response={} bloom={} radius_px={} falloff={} distance_m={} z_depth={} contributions={} used_by={}",
            source.emitter_id.as_deref().unwrap_or("-"),
            source.render_layer.as_deref().unwrap_or("-"),
            format_color(source.color_rgba),
            format_opt(source.intensity),
            format_opt(source.effective_intensity),
            format_opt(source.response),
            format_opt(source.bloom),
            format_opt(source.radius_px),
            format_opt(source.falloff),
            format_opt(source.distance_m),
            format_opt(source.z_depth),
            contributions,
            used_by
        ));
    }

    lines.join("\n")
}

pub(crate) fn collect_camera_optical_candidates_from_light_sources_2d(
    light_sources: &[LightSource2dCommon],
) -> Vec<CameraOpticalCandidate2d> {
    let sources = light_sources
        .iter()
        .filter_map(camera_optical_source_from_light_source)
        .collect::<Vec<_>>();
    amigo_camera_optics_plugin::runtime::collect_camera_optical_candidates_2d(&sources)
}

fn camera_optical_source_from_light_source(
    source: &LightSource2dCommon,
) -> Option<CameraOpticalSource2d> {
    let response = source.camera_response?.normalized();
    let roles = light_source_roles(source);
    let emitter_kind = match source.emitter_kind {
        LightEmitterKind2d::LightGroup => CameraOpticalEmitterKind2d::LightGroup,
        LightEmitterKind2d::Beacon => CameraOpticalEmitterKind2d::Beacon,
        LightEmitterKind2d::ParticleLight => CameraOpticalEmitterKind2d::ParticleLight,
        LightEmitterKind2d::EmissiveMaterial => CameraOpticalEmitterKind2d::EmissiveMaterial,
        _ => CameraOpticalEmitterKind2d::Unsupported,
    };

    Some(CameraOpticalSource2d {
        owner: source.owner.clone(),
        component_kind: source.component_kind.clone(),
        emitter_kind,
        source_id: source.emitter_id.clone(),
        render_layer: source.render_layer.clone(),
        color_rgba: source.color_rgba,
        intensity: source.intensity,
        effective_intensity: source.effective_intensity,
        response,
        roles,
        status: if source.status == LightSourceStatus2d::Active {
            CameraOpticalSourceStatus2d::Active
        } else {
            CameraOpticalSourceStatus2d::Skipped
        },
        reason: source.reason.clone(),
        position_px: source.position_px,
        radius_px: source.radius_px,
    })
}

fn light_source_roles(source: &LightSource2dCommon) -> RenderContributionSet {
    RenderContributionSet::from_pairs(source.contributions.iter().map(|kind| {
        let role = match kind {
            LightContributionKind2d::LightingEmit => {
                amigo_render_api::render_contribution_roles::LIGHTING_EMIT
            }
            LightContributionKind2d::RelightPlate => {
                amigo_render_api::render_contribution_roles::RELIGHT_PLATE
            }
            LightContributionKind2d::BloomSource => {
                amigo_render_api::render_contribution_roles::BLOOM_SOURCE
            }
            LightContributionKind2d::CameraFxSource => {
                amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE
            }
            LightContributionKind2d::EmissiveBuffer => "emissive_buffer",
        };
        (role, true)
    }))
}

fn push_light_group_source(
    sources: &mut Vec<LightSource2dCommon>,
    group: &amigo_light_2d_plugin::LightGroup2dCommand,
    emitter_id: Option<String>,
    response: f32,
    effective_intensity: f32,
    status: LightSourceStatus2d,
    reason: String,
) {
    let common = match status {
        LightSourceStatus2d::Active => LightSource2dCommon::active(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            emitter_id,
            None,
            Some(color_rgba(group.color)),
            Some(group.intensity),
            Some(effective_intensity),
            Some(response),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contributions(group),
            reason,
            None,
        ),
        LightSourceStatus2d::Skipped => LightSource2dCommon::skipped(
            group.id.clone(),
            "LightGroup2D",
            LightEmitterKind2d::LightGroup,
            emitter_id,
            None,
            Some(color_rgba(group.color)),
            Some(group.intensity),
            Some(effective_intensity),
            Some(response),
            Some(group.camera_response),
            None,
            None,
            None,
            None,
            None,
            light_group_contributions(group),
            reason,
            None,
        ),
    };
    sources.push(common);
}

fn light_group_contributions(
    group: &amigo_light_2d_plugin::LightGroup2dCommand,
) -> Vec<LightContributionKind2d> {
    let mut contributions = Vec::new();
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::LIGHTING_EMIT,
        true,
    ) {
        contributions.push(LightContributionKind2d::LightingEmit);
    }
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::BLOOM_SOURCE,
        false,
    ) {
        contributions.push(LightContributionKind2d::BloomSource);
    }
    if group.render_contributions.enabled_or(
        amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
        false,
    ) {
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}

fn particle_light_camera_response(
    light: amigo_particles_2d_plugin::ParticleLight2d,
) -> CameraOpticalResponse2d {
    CameraOpticalResponse2d {
        enabled: light.intensity > 0.0 && light.glow,
        intensity: light.intensity,
        bloom: if light.glow { light.intensity * 0.35 } else { 0.0 },
        glare: light.intensity * 0.2,
        ghosting: 0.0,
        streaks: 0.0,
        chromatic_smear: 0.0,
        dirt_response: 0.0,
        halation: if light.glow { light.intensity * 0.15 } else { 0.0 },
        threshold: 0.0,
    }
    .normalized()
}

fn particle_light_contributions(
    light: amigo_particles_2d_plugin::ParticleLight2d,
) -> Vec<LightContributionKind2d> {
    let mut contributions = vec![LightContributionKind2d::LightingEmit];
    if light.glow && light.intensity > 0.0 {
        contributions.push(LightContributionKind2d::BloomSource);
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}

fn color_rgba(color: amigo_math::ColorRgba) -> [f32; 4] {
    [color.r, color.g, color.b, color.a]
}

fn format_color(value: Option<[f32; 4]>) -> String {
    match value {
        Some([r, g, b, a]) => format!("{r:.3},{g:.3},{b:.3},{a:.3}"),
        None => "-".to_owned(),
    }
}

fn format_opt(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.3}"))
        .unwrap_or_else(|| "-".to_owned())
}

fn visual_source_availability_label(availability: VisualSourceAvailability2d) -> &'static str {
    match availability {
        VisualSourceAvailability2d::Produced => "produced",
        VisualSourceAvailability2d::Derived => "derived",
        VisualSourceAvailability2d::Asset => "asset",
        VisualSourceAvailability2d::Fallback => "fallback",
        VisualSourceAvailability2d::Missing => "missing",
    }
}

#[cfg(test)]
mod tests {
    use amigo_camera_optics_plugin::api::{
        CameraOpticalCandidateStatus2d, CameraOpticalResponse2d,
    };
    use amigo_render_api::{VisualSourceKind2d, VisualSourceOrigin2d, VisualSourceRef2d};

    use super::{
        collect_camera_optical_candidates_from_light_sources_2d, collect_light_sources_2d,
        format_light_sources_2d,
    };

    #[test]
    fn light_sources_summary_reports_emissive_visual_source() {
        let sources = collect_light_sources_2d(
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            Some(&amigo_render_api::CameraCaptureInput2d {
                depth_space: amigo_2d_spatial::DepthSpace2d::default(),
                color: VisualSourceRef2d::fallback(VisualSourceKind2d::SceneColor, "scene"),
                depth: None,
                layer_mask: None,
                normal: None,
                wetness: None,
                emissive: Some(VisualSourceRef2d::produced(
                    VisualSourceKind2d::SceneEmissive,
                    "scene_emissive",
                    VisualSourceOrigin2d::EmissiveBuffer,
                )),
                highlight: None,
                motion: None,
                layers: Vec::new(),
            }),
        );
        let summary = format_light_sources_2d(&sources);

        assert!(summary.contains("render.light.sources:"));
        assert!(summary.contains("emissive_visual_source"));
        assert!(summary.contains("scene_emissive"));
    }

    #[test]
    fn light_sources_collects_lightmap_channels() {
        let lightmap = amigo_light_2d_plugin::LightMap2dSourceCommand {
            source_mod: "test".to_owned(),
            entity_name: "neon-map".to_owned(),
            id: "neon-alley-lightmap".to_owned(),
            source: amigo_light_2d_plugin::LightMap2dSourceRef {
                kind: amigo_light_2d_plugin::LightMap2dSourceKind::LayeredImage2d,
                entity_name: "neon-map".to_owned(),
            },
            channels: vec![amigo_light_2d_plugin::LightMap2dChannel {
                id: "mid_neon".to_owned(),
                layers: vec!["club.mid".to_owned()],
            }],
        };

        let sources = collect_light_sources_2d(&[], &[], &[], &[lightmap], &[], &[], None);
        assert_eq!(sources.len(), 1);
        assert_eq!(
            sources[0].emitter_kind,
            amigo_render_api::LightEmitterKind2d::LightMapChannel
        );
        assert_eq!(sources[0].effective_intensity, Some(1.0));
        assert!(sources[0].reason.contains("mid_neon"));
    }

    #[test]
    fn light_sources_resolves_light_group_effective_intensity() {
        let global = amigo_light_2d_plugin::GlobalLight2dCommand {
            source_mod: "test".to_owned(),
            entity_name: "sky-light".to_owned(),
            id: "sky".to_owned(),
            color: amigo_math::ColorRgba::new(1.0, 0.9, 0.8, 1.0),
            intensity: 0.5,
        };
        let group = amigo_light_2d_plugin::LightGroup2dCommand {
            source_mod: "test".to_owned(),
            id: "street-neon".to_owned(),
            label: None,
            color: amigo_math::ColorRgba::new(1.0, 0.8, 0.6, 1.0),
            intensity: 2.0,
            render_contributions: amigo_render_api::RenderContributionSet::default(),
            camera_response: CameraOpticalResponse2d::default(),
            sources: vec![amigo_light_2d_plugin::LightGroup2dSourceCommand {
                kind: amigo_light_2d_plugin::LightGroup2dSourceKind::GlobalLight {
                    id: "sky".to_owned(),
                },
                response: 0.25,
            }],
        };

        let sources = collect_light_sources_2d(&[], &[], &[global], &[], &[group], &[], None);
        let group_source = sources
            .iter()
            .find(|source| source.emitter_kind == amigo_render_api::LightEmitterKind2d::LightGroup)
            .expect("light group source should be collected");
        assert_eq!(group_source.effective_intensity, Some(0.25));
        assert!(group_source.reason.contains("street-neon"));
    }

    #[test]
    fn camera_optical_candidates_report_light_group_lightmap_coverage() {
        let lightmap = amigo_light_2d_plugin::LightMap2dSourceCommand {
            source_mod: "test".to_owned(),
            entity_name: "neon-map".to_owned(),
            id: "neon-alley-lightmap".to_owned(),
            source: amigo_light_2d_plugin::LightMap2dSourceRef {
                kind: amigo_light_2d_plugin::LightMap2dSourceKind::LayeredImage2d,
                entity_name: "neon-map".to_owned(),
            },
            channels: vec![amigo_light_2d_plugin::LightMap2dChannel {
                id: "mid_neon".to_owned(),
                layers: vec!["club.mid".to_owned()],
            }],
        };
        let group = amigo_light_2d_plugin::LightGroup2dCommand {
            source_mod: "test".to_owned(),
            id: "neon.mid".to_owned(),
            label: None,
            color: amigo_math::ColorRgba::new(1.0, 0.2, 0.8, 1.0),
            intensity: 1.5,
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([
                (amigo_render_api::render_contribution_roles::LIGHTING_EMIT, true),
                (amigo_render_api::render_contribution_roles::BLOOM_SOURCE, true),
                (amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE, true),
            ]),
            camera_response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.75,
                bloom: 0.45,
                ghosting: 0.22,
                ..CameraOpticalResponse2d::default()
            },
            sources: vec![amigo_light_2d_plugin::LightGroup2dSourceCommand {
                kind: amigo_light_2d_plugin::LightGroup2dSourceKind::LightMapChannel {
                    source: "neon-alley-lightmap".to_owned(),
                    channel: "mid_neon".to_owned(),
                },
                response: 1.0,
            }],
        };

        let light_sources =
            collect_light_sources_2d(&[], &[], &[], &[lightmap], &[group], &[], None);
        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&light_sources);
        let summary =
            amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
                &candidates,
            );

        assert!(summary.contains("camera.optical.candidates:"));
        assert!(summary.contains("coverage=lightmap_channel"));
        assert!(summary.contains("source=neon-alley-lightmap"));
        assert!(summary.contains("channel=mid_neon"));
        assert!(summary.contains("intensity=1.500"));
        assert!(summary.contains("bloom:0.450"));
        assert!(summary.contains("status=active"));
        assert!(summary.contains("ghosting:0.220"));
        assert!(summary.contains("targets=scene_highlight,scene_emissive"));
        assert!(summary.contains("highlight_gain=1.125"));
        assert!(summary.contains("emissive_gain=1.125"));
    }

    #[test]
    fn camera_optical_candidate_unsupported_coverage_is_skipped() {
        let source = amigo_render_api::LightSource2dCommon {
            owner: "global-backed-group".to_owned(),
            component_kind: "LightGroup2D".to_owned(),
            emitter_kind: amigo_render_api::LightEmitterKind2d::LightGroup,
            emitter_id: Some("light_group:global:sky".to_owned()),
            render_layer: None,
            color_rgba: Some([1.0, 1.0, 1.0, 1.0]),
            intensity: Some(1.0),
            effective_intensity: Some(1.0),
            response: Some(1.0),
            camera_response: Some(CameraOpticalResponse2d {
                enabled: true,
                intensity: 1.0,
                glare: 1.0,
                bloom: 1.0,
                ..CameraOpticalResponse2d::default()
            }),
            bloom: None,
            
            radius_px: None,
            falloff: None,
            distance_m: None,
            z_depth: None,
            contributions: vec![
                amigo_render_api::LightContributionKind2d::CameraFxSource,
                amigo_render_api::LightContributionKind2d::BloomSource,
            ],
            status: amigo_render_api::LightSourceStatus2d::Active,
            reason: "light_group_active".to_owned(),
            position_px: None,
        };

        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&[source]);
        let summary =
            amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
                &candidates,
            );

        assert_eq!(candidates.len(), 1);
        assert_eq!(
            candidates[0].status,
            CameraOpticalCandidateStatus2d::Skipped
        );
        assert!(!candidates[0].is_active());
        assert_eq!(candidates[0].highlight_gain(), 0.0);
        assert_eq!(candidates[0].emissive_gain(), 0.0);
        assert!(summary.contains("status=skipped"));
        assert!(summary.contains("reason=camera_optical_coverage_unsupported"));
        assert!(summary.contains("targets="));
        assert!(summary.contains("highlight_gain=0.000"));
        assert!(summary.contains("emissive_gain=0.000"));
    }

    #[test]
    fn camera_optical_candidates_report_beacon_hotspot_coverage() {
        let beacon = amigo_beacon_light_2d_plugin::BeaconLight2dDrawCommand {
            entity_name: "beacon-a".to_owned(),
            render_layer: "foreground.lights".to_owned(),
            z_index: 0.0,
            center: amigo_math::Vec2::new(42.0, 84.0),
            color: amigo_math::ColorRgba::new(1.0, 0.2, 0.1, 1.0),
            intensity: 1.0,
            pulse: 1.0,
            core_radius_px: 8.0,
            halo_radius_px: 42.0,
            glow_strength: 0.6,
            rotation_radians: 0.0,
            beam_enabled: false,
            beam_length_px: 0.0,
            beam_width_degrees: 0.0,
            beam_strength: 0.0,
            aberration_px: 4.0,
            
            
            bloom: 0.5,
            camera_response: CameraOpticalResponse2d { enabled: true, intensity: 0.8, bloom: 0.5, glare: 0.8, ghosting: 0.7, streaks: 0.18, chromatic_smear: 4.0 / 32.0, dirt_response: 0.4, halation: 0.5 * 0.35, threshold: 0.0 },
            distance_m: Some(2.0),
            z_depth: Some(0.75),
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([
                (amigo_render_api::render_contribution_roles::BLOOM_SOURCE, true),
                (
                    amigo_render_api::render_contribution_roles::CAMERA_FX_SOURCE,
                    true,
                ),
            ]),
            viewport_fit: amigo_scene::LayeredImageViewportFit2dSceneCommand::Fixed,
            viewport_canvas_size: None,
        };

        let light_sources = collect_light_sources_2d(&[], &[beacon], &[], &[], &[], &[], None);
        let candidates = collect_camera_optical_candidates_from_light_sources_2d(&light_sources);
        let summary =
            amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
                &candidates,
            );

        assert!(summary.contains("component=BeaconLight2D"));
        assert!(summary.contains("coverage=hotspot"));
        assert!(summary.contains("entity=beacon-a"));
        assert!(summary.contains("radius_px=42.000"));
        assert!(summary.contains("position_px=42.000,84.000"));
        assert!(summary.contains("status=active"));
        assert!(summary.contains("ghosting:0.700"));
        assert!(summary.contains("streaks:0.180"));
        assert!(summary.contains("chromatic_smear:0.125"));
        assert!(summary.contains("dirt:0.400"));
    }
}
