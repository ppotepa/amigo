use crate::api::{CameraOpticalCandidate2d, CameraOpticalCoverage2d};

pub fn format_camera_optical_candidates_2d(candidates: &[CameraOpticalCandidate2d]) -> String {
    let mut lines = vec!["camera.optical.candidates:".to_owned()];
    if candidates.is_empty() {
        lines.push("none".to_owned());
        return lines.join("\n");
    }

    for candidate in candidates {
        lines.push(format!(
            "owner={} component={} coverage={} status={} reason={} coverage_details={}",
            candidate.owner,
            candidate.component_kind,
            candidate.coverage.kind(),
            candidate.status.as_str(),
            candidate.reason,
            format_coverage_details(candidate)
        ));
        lines.push(format!(
            "layer={} color={} intensity={:.3} targets={} highlight_gain={:.3} emissive_gain={:.3} response=intensity:{:.3},bloom:{:.3},glare:{:.3},ghosting:{:.3},streaks:{:.3},chromatic_smear:{:.3},dirt:{:.3},halation:{:.3},threshold:{:.3}",
            candidate.render_layer.as_deref().unwrap_or("-"),
            format_color(Some(candidate.color_rgba)),
            candidate.intensity,
            candidate_buffer_targets(candidate).join(","),
            candidate.highlight_gain(),
            candidate.emissive_gain(),
            candidate.response.intensity,
            candidate.response.bloom,
            candidate.response.glare,
            candidate.response.ghosting,
            candidate.response.streaks,
            candidate.response.chromatic_smear,
            candidate.response.dirt_response,
            candidate.response.halation,
            candidate.response.threshold
        ));
    }

    lines.join("\n")
}

fn candidate_buffer_targets(candidate: &CameraOpticalCandidate2d) -> Vec<&'static str> {
    let mut targets = Vec::new();
    if candidate.targets_scene_highlight() {
        targets.push("scene_highlight");
    }
    if candidate.targets_scene_emissive() {
        targets.push("scene_emissive");
    }
    targets
}

fn format_coverage_details(candidate: &CameraOpticalCandidate2d) -> String {
    match &candidate.coverage {
        CameraOpticalCoverage2d::LightMapChannel { source, channel } => {
            format!("source={source} channel={channel}")
        }
        CameraOpticalCoverage2d::Hotspot {
            entity_name,
            radius_px,
        } => {
            let position = candidate
                .position_px
                .map(|position| format!(" position_px={:.3},{:.3}", position[0], position[1]))
                .unwrap_or_default();
            format!("entity={entity_name} radius_px={radius_px:.3}{position}")
        }
        CameraOpticalCoverage2d::Glyphs {
            entity_name,
            render_layer,
        }
        | CameraOpticalCoverage2d::TextureAlpha {
            entity_name,
            render_layer,
        }
        | CameraOpticalCoverage2d::VectorCoverage {
            entity_name,
            render_layer,
        } => format!("entity={entity_name} layer={render_layer}"),
        CameraOpticalCoverage2d::ParticleCoverage {
            emitter_entity_name,
        } => format!("emitter={emitter_entity_name}"),
        CameraOpticalCoverage2d::Unsupported { reason } => format!("reason={reason}"),
    }
}

fn format_color(color: Option<[f32; 4]>) -> String {
    match color {
        Some([r, g, b, a]) => format!("{r:.3},{g:.3},{b:.3},{a:.3}"),
        None => "-".to_owned(),
    }
}
