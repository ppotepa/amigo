use amigo_plugin_api::CandidateStatus;
use amigo_scene::{
    Camera2dSceneCommand, CameraAutoExposure2dSceneCommand, CameraExposure2dSceneCommand,
    CameraExposureMode2dSceneCommand, CameraFilm2dSceneCommand, CameraLens2dSceneCommand,
    CameraLensSurface2dSceneCommand, CameraLook2dSceneCommand, CameraShutter2dSceneCommand,
    RenderContributions2dSceneCommand,
};
use amigo_shutter_motion_plugin::api::{
    MotionShutterCoverage2d, MotionShutterResponse2d, MotionShutterSource2d,
};
use amigo_shutter_motion_plugin::runtime::{
    motion_shutter_source_from_camera_2d_command, resolve_motion_shutter_candidate_2d,
};
use std::collections::BTreeMap;

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

#[test]
fn camera_2d_shutter_command_becomes_active_motion_shutter_candidate() {
    let camera = Camera2dSceneCommand {
        source_mod: "rotten-club".to_owned(),
        entity_name: "camera".to_owned(),
        camera_id: "rotten-club-camera".to_owned(),
        mode: CameraExposureMode2dSceneCommand::Manual,
        render_contributions: RenderContributions2dSceneCommand {
            roles: BTreeMap::from([
                ("camera.projection".to_owned(), true),
                ("camera.shutter".to_owned(), true),
                ("camera.film".to_owned(), true),
                ("camera.scan_output".to_owned(), true),
            ]),
        },
        exposure: CameraExposure2dSceneCommand {
            iso: 800.0,
            compensation: 0.0,
            white_balance: 4300.0,
            nd_stops: 0.0,
            auto: CameraAutoExposure2dSceneCommand {
                target_luma: 0.45,
                adaptation_speed: 0.0,
                min_iso: 100.0,
                max_iso: 1600.0,
            },
        },
        shutter: CameraShutter2dSceneCommand {
            enabled: true,
            speed_s: Some(1.0 / 30.0),
            fps: 30.0,
            angle: 180.0,
            opacity: 1.2,
            history_mix: 0.35,
            history_mix_2: 0.10,
            edge_rejection: 0.25,
            luma_threshold: 0.03,
            frame_hold: false,
        },
        lens: CameraLens2dSceneCommand {
            profile: "default".to_owned(),
            intensity: 0.0,
            aberration_px: None,
            distortion: None,
            vignette: None,
            edge_softness_px: None,
            glare_strength: None,
            dirt: None,
            focal_length_mm: None,
            lens_bloom: None,
            flare_ghosts: None,
            anamorphic_squeeze: None,
            coma: None,
            cat_eye_bokeh: None,
            focus_breathing: None,
        },
        lens_surface: CameraLensSurface2dSceneCommand { rain_profile: None },
        film: CameraFilm2dSceneCommand {
            profile: "default".to_owned(),
            intensity: 1.0,
            seed: 0,
            color_shift: None,
            contrast: None,
            saturation: None,
            flicker: None,
            vignette: None,
            toe: None,
            shoulder: None,
            black_lift: None,
            print_fade: None,
            dust: None,
            scratches: None,
            push_pull: None,
            gate_weave: None,
            scan_softness: None,
        },
        look: CameraLook2dSceneCommand {
            profile: "default".to_owned(),
            intensity: 0.0,
        },
        aperture: amigo_scene::CameraAperture2dSceneCommand {
            enabled: false,
            f_stop: 0.0,
            focus_distance_m: 0.0,
            focus: amigo_scene::CameraFocus2dSceneCommand::None,
            depth_of_field: amigo_scene::CameraDepthOfField2dSceneCommand {
                depth_map: None,
                affected_layers: Vec::new(),
                max_blur_px: 0.0,
                depth_contrast: 0.0,
                focus_width: 0.0,
                foreground_blur_boost: 0.0,
                background_blur_boost: 0.0,
                edge_aware: false,
                invert_depth: false,
                debug_view: String::new(),
                aperture_blades: 0,
                aperture_roundness: 0.0,
                aperture_rotation_degrees: 0.0,
                sample_count: 0,
                highlight_threshold: 0.0,
                highlight_knee: 0.0,
                highlight_gain: 0.0,
                highlight_saturation: 0.0,
            },
        },
    };

    let source = motion_shutter_source_from_camera_2d_command(&camera);
    let candidate = resolve_motion_shutter_candidate_2d(&source);

    assert_eq!(candidate.status, CandidateStatus::Active);
    assert!(candidate.is_active());
    assert!((candidate.response.exposure_time_s - (1.0 / 30.0)).abs() < 0.0001);
    assert!(
        candidate
            .target_ids
            .iter()
            .any(|target| target.0 == "TemporalExposure")
    );
}
