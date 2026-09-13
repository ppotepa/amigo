use amigo_plugin_api::CandidateStatus;
use amigo_shutter_motion_plugin::api::{
    MotionShutterCoverage2d, MotionShutterResponse2d, MotionShutterSource2d,
};
use amigo_shutter_motion_plugin::runtime::resolve_motion_shutter_candidate_2d;

#[test]
fn motion_shutter_candidate_active_when_declared_supported_and_enabled() {
    let candidate = resolve_motion_shutter_candidate_2d(&MotionShutterSource2d {
        owner: "main-camera".to_owned(),
        declared: true,
        coverage: MotionShutterCoverage2d::SceneVelocity,
        response: MotionShutterResponse2d {
            enabled: true,
            shutter_angle: 180.0,
            exposure_time_s: 1.0 / 48.0,
            motion_blur: 1.0,
            temporal_accumulation: 0.25,
        },
    });

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate.is_active());
    assert!(candidate
        .target_ids
        .iter()
        .any(|target| target.0 == "TemporalExposure"));
}

#[test]
fn unsupported_motion_shutter_coverage_is_not_active() {
    let candidate = resolve_motion_shutter_candidate_2d(&MotionShutterSource2d {
        owner: "main-camera".to_owned(),
        declared: true,
        coverage: MotionShutterCoverage2d::Unsupported {
            reason: "no_velocity".to_owned(),
        },
        response: MotionShutterResponse2d {
            enabled: true,
            shutter_angle: 180.0,
            exposure_time_s: 1.0 / 48.0,
            motion_blur: 1.0,
            temporal_accumulation: 0.25,
        },
    });

    assert_eq!(candidate.status, CandidateStatus::Unsupported);
    assert!(!candidate.is_active());
    assert!(candidate.target_ids.is_empty());
}
