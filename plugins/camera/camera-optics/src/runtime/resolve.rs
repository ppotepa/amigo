use amigo_plugin_api::CandidateTrace;

use crate::api::{
    CameraOpticalCandidate2d, CameraOpticalCandidateStatus2d, CameraOpticalCoverage2d,
    CameraOpticalEmitterKind2d, CameraOpticalSource2d, CameraOpticalSourceStatus2d,
};

pub fn resolve_camera_optical_candidate_2d(
    source: &CameraOpticalSource2d,
) -> Option<CameraOpticalCandidate2d> {
    if matches!(source.status, CameraOpticalSourceStatus2d::Skipped)
        && source.roles.is_empty()
        && !source.response.enabled
    {
        return None;
    }

    let response = source.response.normalized();
    let role_enabled =
        source.roles.enabled_or(amigo_plugin_api::roles::CAMERA_FX_SOURCE, false)
            || source.roles.enabled_or(amigo_plugin_api::roles::BLOOM_SOURCE, false);
    let response_enabled = response.enabled
        && response
            .intensity
            .max(response.bloom)
            .max(response.glare)
            .max(response.ghosting)
            .max(response.streaks)
            .max(response.dirt_response)
            .max(response.halation)
            > 0.0;
    let coverage = optical_coverage_for_source(source);
    let coverage_supported = coverage.is_supported();
    let active = matches!(source.status, CameraOpticalSourceStatus2d::Active)
        && role_enabled
        && response_enabled
        && coverage_supported;
    let reason = if !matches!(source.status, CameraOpticalSourceStatus2d::Active) {
        source.reason.clone()
    } else if !role_enabled {
        "camera_optical_role_disabled".to_owned()
    } else if !response_enabled {
        "camera_optical_response_disabled".to_owned()
    } else if !coverage_supported {
        coverage
            .unsupported_reason()
            .map(|reason| format!("camera_optical_coverage_unsupported:{reason}"))
            .unwrap_or_else(|| "camera_optical_coverage_unsupported".to_owned())
    } else {
        "camera_optical_candidate_active".to_owned()
    };

    let status = if active {
        CameraOpticalCandidateStatus2d::Active
    } else {
        CameraOpticalCandidateStatus2d::Skipped
    };

    let trace_status = if !coverage_supported {
        amigo_plugin_api::CandidateStatus::Unsupported
    } else if active {
        amigo_plugin_api::CandidateStatus::Active
    } else {
        amigo_plugin_api::CandidateStatus::Inactive
    };

    let mut candidate = CameraOpticalCandidate2d {
        owner: source.owner.clone(),
        component_kind: source.component_kind.clone(),
        render_layer: source.render_layer.clone(),
        color_rgba: source.color_rgba.unwrap_or([1.0, 1.0, 1.0, 1.0]),
        intensity: source.effective_intensity.or(source.intensity).unwrap_or(0.0),
        response,
        coverage,
        roles: source.roles.clone(),
        status,
        reason: reason.clone(),
        position_px: source.position_px,
        target_ids: Vec::new(),
        trace: Some(CandidateTrace {
            domain: amigo_plugin_api::DomainId("camera.optics".to_string()),
            status: trace_status,
            reason: Some(reason),
            targets: Vec::new(),
        }),
    };
    candidate.recompute_targets();
    if let Some(trace) = candidate.trace.as_mut() {
        trace.targets = candidate.target_ids.clone();
    }
    Some(candidate)
}

fn optical_coverage_for_source(source: &CameraOpticalSource2d) -> CameraOpticalCoverage2d {
    match source.emitter_kind {
        CameraOpticalEmitterKind2d::LightGroup => light_group_coverage(source),
        CameraOpticalEmitterKind2d::Beacon => CameraOpticalCoverage2d::Hotspot {
            entity_name: source.owner.clone(),
            radius_px: source.radius_px.unwrap_or(0.0),
        },
        CameraOpticalEmitterKind2d::ParticleLight => CameraOpticalCoverage2d::ParticleCoverage {
            emitter_entity_name: source.owner.clone(),
        },
        CameraOpticalEmitterKind2d::EmissiveMaterial => {
            let Some(render_layer) = source.render_layer.clone() else {
                return CameraOpticalCoverage2d::Unsupported {
                    reason: "emissive_material_without_render_layer".to_owned(),
                };
            };
            match source.component_kind.as_str() {
                "Text2D" => CameraOpticalCoverage2d::Glyphs {
                    entity_name: source.owner.clone(),
                    render_layer,
                },
                "Sprite2D" => CameraOpticalCoverage2d::TextureAlpha {
                    entity_name: source.owner.clone(),
                    render_layer,
                },
                "VectorShape2D" => CameraOpticalCoverage2d::VectorCoverage {
                    entity_name: source.owner.clone(),
                    render_layer,
                },
                _ => CameraOpticalCoverage2d::Unsupported {
                    reason: "emissive_material_component_coverage_unsupported_v1".to_owned(),
                },
            }
        }
        CameraOpticalEmitterKind2d::Unsupported => CameraOpticalCoverage2d::Unsupported {
            reason: "coverage_not_mapped_for_optical_candidate_v1".to_owned(),
        },
    }
}

fn light_group_coverage(source: &CameraOpticalSource2d) -> CameraOpticalCoverage2d {
    let Some(source_id) = source.source_id.as_deref() else {
        return CameraOpticalCoverage2d::Unsupported {
            reason: "light_group_missing_emitter_id".to_owned(),
        };
    };

    if let Some(rest) = source_id.strip_prefix("lightmap:") {
        let mut parts = rest.splitn(2, ':');
        let source = parts.next().unwrap_or_default();
        let channel = parts.next().unwrap_or_default();
        if !source.is_empty() && !channel.is_empty() {
            return CameraOpticalCoverage2d::LightMapChannel {
                source: source.to_owned(),
                channel: channel.to_owned(),
            };
        }
    }

    if let Some(rest) = source_id.split_once(":lightmap:").map(|(_, rest)| rest) {
        let mut parts = rest.splitn(2, ':');
        let source = parts.next().unwrap_or_default();
        let channel = parts.next().unwrap_or_default();
        if !source.is_empty() && !channel.is_empty() {
            return CameraOpticalCoverage2d::LightMapChannel {
                source: source.to_owned(),
                channel: channel.to_owned(),
            };
        }
    }

    CameraOpticalCoverage2d::Unsupported {
        reason: format!("light_group_source_not_optical_v1:{source_id}"),
    }
}
