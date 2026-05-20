use super::*;

pub(super) fn collect_camera_capture_visual_sources(
    camera_capture_input: Option<&CameraCaptureInput2d>,
    sources: &mut Vec<LightSource2dCommon>,
) {
    if let Some(input) = camera_capture_input {
        if let Some(emissive) = &input.emissive {
            let source = if matches!(emissive.availability, VisualSourceAvailability2d::Missing) {
                skipped_light_source!(
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
                active_light_source!(
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
