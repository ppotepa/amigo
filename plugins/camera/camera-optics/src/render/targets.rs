use amigo_plugin_api::{StandardTarget, TargetId};

use crate::api::CameraOpticalCandidate2d;

pub fn targets_scene_highlight_buffer(
    _has_visual_map: bool,
    candidates: &[CameraOpticalCandidate2d],
) -> bool {
    candidates
        .iter()
        .any(|candidate| candidate.targets_scene_highlight())
}

pub fn targets_scene_emissive_buffer(
    _has_visual_map: bool,
    candidates: &[CameraOpticalCandidate2d],
) -> bool {
    candidates
        .iter()
        .any(|candidate| candidate.targets_scene_emissive())
}

pub fn scene_highlight_target_id() -> TargetId {
    StandardTarget::SceneHighlight.id()
}

pub fn scene_emissive_target_id() -> TargetId {
    StandardTarget::SceneEmissive.id()
}

#[cfg(test)]
mod tests {
    use amigo_plugin_api::{RenderContributionSet, render_contributions::roles};

    use super::*;
    use crate::api::{
        CameraOpticalCandidateStatus2d, CameraOpticalCoverage2d, CameraOpticalResponse2d,
    };

    fn candidate_with_roles(roles: RenderContributionSet) -> CameraOpticalCandidate2d {
        let mut candidate = CameraOpticalCandidate2d {
            owner: "source".to_owned(),
            component_kind: "Sprite2D".to_owned(),
            render_layer: Some("world".to_owned()),
            color_rgba: [1.0, 1.0, 1.0, 1.0],
            intensity: 2.0,
            response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 1.0,
                bloom: 1.0,
                glare: 1.0,
                ghosting: 0.0,
                streaks: 0.0,
                chromatic_smear: 0.0,
                dirt_response: 0.0,
                halation: 0.0,
                threshold: 0.0,
            },
            coverage: CameraOpticalCoverage2d::TextureAlpha {
                entity_name: "source".to_owned(),
                render_layer: "world".to_owned(),
            },
            roles,
            status: CameraOpticalCandidateStatus2d::Active,
            reason: "active".to_owned(),
            position_px: None,
            target_ids: Vec::new(),
            trace: None,
        };
        candidate.recompute_targets();
        candidate
    }

    #[test]
    fn visual_map_presence_does_not_route_without_candidate() {
        assert!(!targets_scene_highlight_buffer(true, &[]));
        assert!(!targets_scene_emissive_buffer(true, &[]));
    }

    #[test]
    fn explicit_candidate_roles_route_targets() {
        let candidate = candidate_with_roles(RenderContributionSet::from_pairs([
            (roles::CAMERA_FX_SOURCE, true),
            (roles::BLOOM_SOURCE, true),
        ]));

        assert!(targets_scene_highlight_buffer(false, &[candidate.clone()]));
        assert!(targets_scene_emissive_buffer(false, &[candidate]));
    }
}
