use amigo_focus_depth_plugin::api::{
    FocusDepthCoverage2d, FocusDepthResponse2d, FocusDepthSource2d,
};
use amigo_focus_depth_plugin::runtime::{
    effective_distance_after_camera_z_m, resolve_focus_depth_candidate_2d,
};
use amigo_plugin_api::CandidateStatus;

#[test]
fn focus_depth_candidate_active_when_declared_supported_and_enabled() {
    let candidate = resolve_focus_depth_candidate_2d(&FocusDepthSource2d {
        owner: "main-camera".to_owned(),
        declared: true,
        coverage: FocusDepthCoverage2d::SceneDepth,
        response: FocusDepthResponse2d {
            enabled: true,
            strength: 1.0,
            focus_width_m: 2.0,
            max_blur_px: 12.0,
        },
    });

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate.is_active());
    assert!(candidate
        .target_ids
        .iter()
        .any(|target| target.0 == "FocusField"));
}

#[test]
fn unsupported_focus_depth_coverage_is_not_active() {
    let candidate = resolve_focus_depth_candidate_2d(&FocusDepthSource2d {
        owner: "main-camera".to_owned(),
        declared: true,
        coverage: FocusDepthCoverage2d::Unsupported {
            reason: "no_depth".to_owned(),
        },
        response: FocusDepthResponse2d {
            enabled: true,
            strength: 1.0,
            focus_width_m: 2.0,
            max_blur_px: 12.0,
        },
    });

    assert_eq!(candidate.status, CandidateStatus::Unsupported);
    assert!(!candidate.is_active());
    assert!(candidate.target_ids.is_empty());
}

#[test]
fn effective_distance_accounts_for_camera_z() {
    let effective = effective_distance_after_camera_z_m(6.0, 2.0);

    assert!((effective - 4.0).abs() < 0.001);
}
