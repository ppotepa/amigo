use amigo_scene::Camera2dSceneCommand;

use crate::api::{
    MotionShutterCandidate2d, MotionShutterCoverage2d, MotionShutterResponse2d,
    MotionShutterSource2d,
};

pub fn collect_motion_shutter_candidates_2d(
    sources: &[MotionShutterSource2d],
) -> Vec<MotionShutterCandidate2d> {
    sources
        .iter()
        .map(super::resolve_motion_shutter_candidate_2d)
        .collect()
}

pub fn motion_shutter_source_from_camera_2d_command(
    camera: &Camera2dSceneCommand,
) -> MotionShutterSource2d {
    let declared = camera
        .render_contributions
        .roles
        .get("camera.shutter")
        .copied()
        .unwrap_or(false);

    MotionShutterSource2d {
        owner: camera.camera_id.clone(),
        declared,
        coverage: MotionShutterCoverage2d::SceneVelocity,
        response: MotionShutterResponse2d {
            enabled: camera.shutter.enabled,
            shutter_angle: camera.shutter.angle,
            exposure_time_s: camera.shutter.speed_s.unwrap_or(1.0 / 30.0).max(0.0),
            motion_blur: camera.shutter.opacity.max(0.0),
            temporal_accumulation: camera.shutter.history_mix.clamp(0.0, 1.0),
        }
        .normalized(),
    }
}
