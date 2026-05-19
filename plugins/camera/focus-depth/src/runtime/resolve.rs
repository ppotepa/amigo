use amigo_plugin_api::CandidateStatus;

use crate::api::{
    focus_field_target_id, scene_depth_target_id, FocusDepthCandidate2d, FocusDepthSource2d,
};

pub fn resolve_focus_depth_candidate_2d(source: &FocusDepthSource2d) -> FocusDepthCandidate2d {
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
        "focus_depth_not_declared".to_owned()
    } else if !coverage_supported {
        source
            .coverage
            .unsupported_reason()
            .map(|reason| format!("focus_depth_coverage_unsupported:{reason}"))
            .unwrap_or_else(|| "focus_depth_coverage_unsupported".to_owned())
    } else if !response_enabled {
        "focus_depth_response_disabled".to_owned()
    } else {
        "focus_depth_candidate_active".to_owned()
    };
    let target_ids = if status == CandidateStatus::Active {
        vec![scene_depth_target_id(), focus_field_target_id()]
    } else {
        Vec::new()
    };
    FocusDepthCandidate2d {
        owner: source.owner.clone(),
        coverage: source.coverage.clone(),
        response,
        status,
        reason,
        target_ids,
    }
}
