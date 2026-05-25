use crate::{CameraFocusTargetDepth2d, CameraFocusTransitionTarget2d};
use amigo_camera_optics_plugin::runtime::{Camera2dRuntimeState, CameraAperture2d, CameraFocus2d};

pub(super) fn transition_target_for_depth(
    depth: &CameraFocusTargetDepth2d,
) -> CameraFocusTransitionTarget2d {
    match depth {
        CameraFocusTargetDepth2d::Distance { meters, .. } => {
            CameraFocusTransitionTarget2d::Distance {
                meters: (*meters).max(0.2),
            }
        }
        CameraFocusTargetDepth2d::Depth { z_depth } => CameraFocusTransitionTarget2d::Depth {
            value: (*z_depth).clamp(0.0, 1.0),
        },
    }
}

pub(super) fn focus_transition_start_for(
    camera: &Camera2dRuntimeState,
    end: &CameraFocusTransitionTarget2d,
) -> CameraFocusTransitionTarget2d {
    match end {
        CameraFocusTransitionTarget2d::Distance { .. } => {
            let meters = match camera.aperture.focus {
                CameraFocus2d::Distance { meters } => meters,
                _ => camera.aperture.focus_distance_m,
            };
            CameraFocusTransitionTarget2d::Distance {
                meters: meters.max(0.2),
            }
        }
        CameraFocusTransitionTarget2d::Depth { .. } => {
            let value = match camera.aperture.focus {
                CameraFocus2d::Depth { value } => value,
                _ => 0.5,
            };
            CameraFocusTransitionTarget2d::Depth {
                value: value.clamp(0.0, 1.0),
            }
        }
    }
}

pub(super) fn lerp_focus_transition_target(
    start: &CameraFocusTransitionTarget2d,
    end: &CameraFocusTransitionTarget2d,
    t: f32,
) -> CameraFocusTransitionTarget2d {
    match (start, end) {
        (
            CameraFocusTransitionTarget2d::Distance { meters: start },
            CameraFocusTransitionTarget2d::Distance { meters: end },
        ) => CameraFocusTransitionTarget2d::Distance {
            meters: lerp(*start, *end, t).max(0.2),
        },
        (
            CameraFocusTransitionTarget2d::Depth { value: start },
            CameraFocusTransitionTarget2d::Depth { value: end },
        ) => CameraFocusTransitionTarget2d::Depth {
            value: lerp(*start, *end, t).clamp(0.0, 1.0),
        },
        _ => end.clone(),
    }
}

pub(super) fn apply_focus_transition_target(
    aperture: &mut CameraAperture2d,
    target: &CameraFocusTransitionTarget2d,
) {
    match target {
        CameraFocusTransitionTarget2d::Distance { meters } => {
            aperture.focus_distance_m = meters.max(0.2);
            aperture.focus = CameraFocus2d::Distance {
                meters: meters.max(0.2),
            };
        }
        CameraFocusTransitionTarget2d::Depth { value } => {
            aperture.focus = CameraFocus2d::Depth {
                value: value.clamp(0.0, 1.0),
            };
        }
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t.clamp(0.0, 1.0)
}
