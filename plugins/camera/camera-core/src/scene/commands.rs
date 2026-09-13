use amigo_core::{AmigoError, AmigoResult};
use amigo_render_api::{render_contribution_roles as roles, RenderContributionSet};
use amigo_scene::{
    format_scene_command, Camera2dSceneCommand, CameraController3dSceneCommand,
    CameraExposureMode2dSceneCommand, CameraFocus2dSceneCommand, CameraFollow2dSceneCommand,
    Parallax2dSceneCommand, RuntimeSceneCommandHandler, SceneCommand, SceneEvent, SceneEventQueue,
    SceneService, CAMERA_2D_PLUGIN_SCENE_COMMAND_TYPE,
    CAMERA_CONTROLLER_3D_PLUGIN_SCENE_COMMAND_TYPE, CAMERA_FOLLOW_2D_PLUGIN_SCENE_COMMAND_TYPE,
    PARALLAX_2D_PLUGIN_SCENE_COMMAND_TYPE,
};

use crate::{
    CameraController3dSceneService, CameraFollow2dSceneService, CameraId, CameraService,
    Parallax2dSceneService,
};
use amigo_camera_optics_plugin::runtime::{
    Camera2dRuntimeState, CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d,
    CameraExposure2d, CameraExposureMode2d, CameraFilm2d, CameraFocus2d, CameraLens2d,
    CameraLensSurface2d, CameraLook2d, CameraShutter2d,
};

pub struct CameraSceneCommandHandler;

fn camera_render_contribution_defaults() -> [(&'static str, bool); 9] {
    [
        (roles::CAMERA_PROJECTION, true),
        (roles::CAMERA_EXPOSURE, false),
        (roles::CAMERA_SHUTTER, false),
        (roles::CAMERA_OPTICS, false),
        (roles::CAMERA_FOCUS_BLUR, false),
        (roles::CAMERA_LENS_SURFACE, false),
        (roles::CAMERA_FILM, false),
        (roles::CAMERA_LOOK, false),
        (roles::CAMERA_SCAN_OUTPUT, false),
    ]
}

impl RuntimeSceneCommandHandler for CameraSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::Plugin { command }
                if matches!(
                    command.command_type.as_str(),
                    CAMERA_2D_PLUGIN_SCENE_COMMAND_TYPE
                        | CAMERA_FOLLOW_2D_PLUGIN_SCENE_COMMAND_TYPE
                        | CAMERA_CONTROLLER_3D_PLUGIN_SCENE_COMMAND_TYPE
                        | PARALLAX_2D_PLUGIN_SCENE_COMMAND_TYPE
                )
        )
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let camera_service = runtime.required::<CameraService>()?;
        let camera_follow_scene_service = runtime.required::<CameraFollow2dSceneService>()?;
        let camera_controller_3d_scene_service =
            runtime.required::<CameraController3dSceneService>()?;
        let parallax_scene_service = runtime.required::<Parallax2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        match command {
            SceneCommand::Plugin { command }
                if command.command_type == CAMERA_2D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let command = command
                    .payload_as::<Camera2dSceneCommand>()
                    .ok_or_else(|| {
                        AmigoError::Message(
                            "camera 2d plugin scene command payload type mismatch".to_owned(),
                        )
                    })?
                    .clone();
                let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                let mut render_contributions =
                    RenderContributionSet::from_pairs(command.render_contributions.roles.clone());
                render_contributions.merge_defaults(camera_render_contribution_defaults());

                let camera = Camera2dRuntimeState {
                    id: CameraId(command.camera_id.clone()),
                    entity_name: command.entity_name.clone(),
                    mode: match command.mode {
                        CameraExposureMode2dSceneCommand::Auto => CameraExposureMode2d::Auto,
                        CameraExposureMode2dSceneCommand::Manual => CameraExposureMode2d::Manual,
                    },
                    exposure: CameraExposure2d {
                        iso: command.exposure.iso,
                        compensation: command.exposure.compensation,
                        white_balance: command.exposure.white_balance,
                        nd_stops: command.exposure.nd_stops,
                        auto: CameraAutoExposure2d {
                            target_luma: command.exposure.auto.target_luma,
                            adaptation_speed: command.exposure.auto.adaptation_speed,
                            min_iso: command.exposure.auto.min_iso,
                            max_iso: command.exposure.auto.max_iso,
                        },
                    },
                    shutter: CameraShutter2d {
                        enabled: command.shutter.enabled,
                        speed_s: command.shutter.speed_s,
                        fps: command.shutter.fps,
                        angle: command.shutter.angle,
                        opacity: command.shutter.opacity,
                        history_mix: command.shutter.history_mix,
                        history_mix_2: command.shutter.history_mix_2,
                        edge_rejection: command.shutter.edge_rejection,
                        luma_threshold: command.shutter.luma_threshold,
                        frame_hold: command.shutter.frame_hold,
                    },
                    lens: CameraLens2d {
                        profile: command.lens.profile.clone(),
                        intensity: command.lens.intensity,
                        aberration_px: command.lens.aberration_px,
                        distortion: command.lens.distortion,
                        vignette: command.lens.vignette,
                        edge_softness_px: command.lens.edge_softness_px,
                        glare_strength: command.lens.glare_strength,
                        dirt: command.lens.dirt,
                        focal_length_mm: command.lens.focal_length_mm,
                        lens_bloom: command.lens.lens_bloom,
                        flare_ghosts: command.lens.flare_ghosts,
                        anamorphic_squeeze: command.lens.anamorphic_squeeze,
                        coma: command.lens.coma,
                        cat_eye_bokeh: command.lens.cat_eye_bokeh,
                        focus_breathing: command.lens.focus_breathing,
                    },
                    lens_surface: CameraLensSurface2d {
                        rain_profile: command.lens_surface.rain_profile.clone(),
                    },
                    film: CameraFilm2d {
                        profile: command.film.profile.clone(),
                        intensity: command.film.intensity,
                        seed: command.film.seed,
                        color_shift: command.film.color_shift,
                        contrast: command.film.contrast,
                        saturation: command.film.saturation,
                        flicker: command.film.flicker,
                        vignette: command.film.vignette,
                        toe: command.film.toe,
                        shoulder: command.film.shoulder,
                        black_lift: command.film.black_lift,
                        print_fade: command.film.print_fade,
                        dust: command.film.dust,
                        scratches: command.film.scratches,
                        push_pull: command.film.push_pull,
                        gate_weave: command.film.gate_weave,
                        scan_softness: command.film.scan_softness,
                    },
                    look: CameraLook2d {
                        profile: command.look.profile.clone(),
                        intensity: command.look.intensity,
                    },
                    aperture: CameraAperture2d {
                        enabled: command.aperture.enabled,
                        f_stop: command.aperture.f_stop,
                        focus_distance_m: command.aperture.focus_distance_m,
                        focus: match &command.aperture.focus {
                            CameraFocus2dSceneCommand::None => CameraFocus2d::None,
                            CameraFocus2dSceneCommand::RenderLayer { layer } => {
                                CameraFocus2d::RenderLayer {
                                    layer: layer.clone(),
                                }
                            }
                            CameraFocus2dSceneCommand::SceneObject { object } => {
                                CameraFocus2d::SceneObject {
                                    object: object.clone(),
                                }
                            }
                            CameraFocus2dSceneCommand::Distance { distance_m } => {
                                CameraFocus2d::Distance {
                                    meters: *distance_m,
                                }
                            }
                            CameraFocus2dSceneCommand::Depth { value } => {
                                CameraFocus2d::Depth { value: *value }
                            }
                        },
                        depth_of_field: CameraDepthOfField2d {
                            depth_map: command.aperture.depth_of_field.depth_map.clone(),
                            affected_layers: command
                                .aperture
                                .depth_of_field
                                .affected_layers
                                .clone(),
                            max_blur_px: command.aperture.depth_of_field.max_blur_px,
                            depth_contrast: command.aperture.depth_of_field.depth_contrast,
                            focus_width: command.aperture.depth_of_field.focus_width,
                            foreground_blur_boost: command
                                .aperture
                                .depth_of_field
                                .foreground_blur_boost,
                            background_blur_boost: command
                                .aperture
                                .depth_of_field
                                .background_blur_boost,
                            edge_aware: command.aperture.depth_of_field.edge_aware,
                            invert_depth: command.aperture.depth_of_field.invert_depth,
                            debug_view: command.aperture.depth_of_field.debug_view.clone(),
                            aperture_blades: command.aperture.depth_of_field.aperture_blades,
                            aperture_roundness: command.aperture.depth_of_field.aperture_roundness,
                            aperture_rotation_degrees: command
                                .aperture
                                .depth_of_field
                                .aperture_rotation_degrees,
                            sample_count: command.aperture.depth_of_field.sample_count,
                            highlight_threshold: command
                                .aperture
                                .depth_of_field
                                .highlight_threshold,
                            highlight_knee: command.aperture.depth_of_field.highlight_knee,
                            highlight_gain: command.aperture.depth_of_field.highlight_gain,
                            highlight_saturation: command
                                .aperture
                                .depth_of_field
                                .highlight_saturation,
                        },
                    },
                    render_contributions,
                }
                .normalized();

                camera_service.upsert_2d(camera);

                scene_event_queue.publish(SceneEvent::Camera2dQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    camera_id: command.camera_id.clone(),
                });

                Ok(())
            }
            SceneCommand::Plugin { command }
                if command.command_type == CAMERA_CONTROLLER_3D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let command = command
                    .payload_as::<CameraController3dSceneCommand>()
                    .ok_or_else(|| {
                        AmigoError::Message(
                            "camera controller 3d plugin scene command payload type mismatch"
                                .to_owned(),
                        )
                    })?
                    .clone();
                let _entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                camera_controller_3d_scene_service.queue(command);
                Ok(())
            }
            SceneCommand::Plugin { command }
                if command.command_type == CAMERA_FOLLOW_2D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let command = command
                    .payload_as::<CameraFollow2dSceneCommand>()
                    .ok_or_else(|| {
                        AmigoError::Message(
                            "camera follow 2d plugin scene command payload type mismatch"
                                .to_owned(),
                        )
                    })?
                    .clone();
                let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                camera_follow_scene_service.queue(CameraFollow2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    target: command.target.clone(),
                    offset: command.offset,
                    lerp: command.lerp,
                    lookahead_velocity_scale: command.lookahead_velocity_scale,
                    lookahead_max_distance: command.lookahead_max_distance,
                    sway_amount: command.sway_amount,
                    sway_frequency: command.sway_frequency,
                });
                scene_event_queue.publish(SceneEvent::CameraFollowQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name,
                    target: command.target,
                });
                Ok(())
            }
            SceneCommand::Plugin { command }
                if command.command_type == PARALLAX_2D_PLUGIN_SCENE_COMMAND_TYPE =>
            {
                let command = command
                    .payload_as::<Parallax2dSceneCommand>()
                    .ok_or_else(|| {
                        AmigoError::Message(
                            "parallax 2d plugin scene command payload type mismatch".to_owned(),
                        )
                    })?
                    .clone();
                let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                parallax_scene_service.queue(Parallax2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    camera: command.camera.clone(),
                    factor: command.factor,
                    anchor: command.anchor,
                    camera_origin: None,
                });
                scene_event_queue.publish(SceneEvent::ParallaxQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name,
                    camera: command.camera,
                });
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "camera scene handler cannot handle command {}",
                format_scene_command(&command)
            ))),
        }
    }
}
