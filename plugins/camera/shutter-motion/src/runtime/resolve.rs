use amigo_plugin_api::CandidateStatus;

use crate::api::{
    scene_velocity_target_id, temporal_exposure_target_id, MotionShutterCandidate2d,
    MotionShutterSource2d,
};

pub fn resolve_motion_shutter_candidate_2d(
    source: &MotionShutterSource2d,
) -> MotionShutterCandidate2d {
    let response = source.response.normalized();
    let coverage_supported = source.coverage.is_supported();
    let response_enabled = response.is_enabled();
    let status = if !source.declared {
        CandidateStatus::NotDeclared
    } else if !coverage_supported {
        CandidateStatus::Unsupported
    } else if !response_enabled {
        CandidateStatus::Inactive
    } else {
        CandidateStatus::Active
    };
    let reason = if !source.declared {
        "motion_shutter_not_declared".to_owned()
    } else if !coverage_supported {
        source
            .coverage
            .unsupported_reason()
            .map(|reason| format!("motion_shutter_coverage_unsupported:{reason}"))
            .unwrap_or_else(|| "motion_shutter_coverage_unsupported".to_owned())
    } else if !response_enabled {
        "motion_shutter_response_disabled".to_owned()
    } else {
        "motion_shutter_candidate_active".to_owned()
    };
    let target_ids = if status == CandidateStatus::Active {
        vec![scene_velocity_target_id(), temporal_exposure_target_id()]
    } else {
        Vec::new()
    };
    MotionShutterCandidate2d {
        owner: source.owner.clone(),
        coverage: source.coverage.clone(),
        response,
        status,
        reason,
        target_ids,
    }
}
