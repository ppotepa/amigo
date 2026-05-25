use amigo_scene::SceneComponentDocument as ComponentDocument;
use amigo_scene::{
    Camera2dModeDocument, Camera2dSceneCommand, CameraAperture2dSceneCommand,
    CameraAutoExposure2dSceneCommand, CameraDepthOfField2dSceneCommand,
    CameraExposure2dSceneCommand, CameraExposureMode2dSceneCommand, CameraFilm2dSceneCommand,
    CameraFocus2dDocument, CameraFocus2dSceneCommand, CameraLens2dSceneCommand,
    CameraLensSurface2dSceneCommand, CameraLook2dSceneCommand, CameraShutter2dSceneCommand,
    ComponentHydrationContext, ComponentHydrator, PluginComponentHydrationContext,
    PluginComponentHydrator, RenderContributions2dSceneCommand, SceneCommand,
    SceneComponentDocument, SceneDocumentError, SceneDocumentResult,
};

use super::Camera2dDocument;

pub struct Camera2dComponentHydrator;
pub struct Camera2dPluginComponentHydrator;

impl ComponentHydrator for Camera2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.camera-core"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, ComponentDocument::Camera2d { .. })
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            ComponentDocument::Camera2d { .. } => {
                let Some(document) = Camera2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            _ => return Ok(()),
        };

        push_camera_command(&document, ctx.source_mod, ctx.entity_name, ctx.commands);
        Ok(())
    }
}

impl PluginComponentHydrator for Camera2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.camera-core"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.camera-core.Camera2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<Camera2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "Camera2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_camera_command(document, ctx.source_mod, ctx.entity_name, ctx.commands);
        Ok(())
    }
}

fn push_camera_command(
    document: &Camera2dDocument,
    source_mod: &str,
    entity_name: &str,
    commands: &mut Vec<SceneCommand>,
) {
    commands.push(SceneCommand::Plugin {
        command: amigo_scene::camera_2d_plugin_scene_command(Camera2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            camera_id: document.id.clone(),
            mode: camera_mode_from_document(document.mode),
            render_contributions: RenderContributions2dSceneCommand {
                roles: document
                    .render_contributions
                    .clone()
                    .with_defaults(camera_render_contribution_defaults())
                    .into_roles(),
            },
            exposure: CameraExposure2dSceneCommand {
                iso: document.exposure.iso,
                compensation: document.exposure.compensation,
                white_balance: document.exposure.white_balance,
                nd_stops: document.exposure.nd_stops,
                auto: CameraAutoExposure2dSceneCommand {
                    target_luma: document.exposure.auto.target_luma,
                    adaptation_speed: document.exposure.auto.adaptation_speed,
                    min_iso: document.exposure.auto.min_iso,
                    max_iso: document.exposure.auto.max_iso,
                },
            },
            shutter: CameraShutter2dSceneCommand {
                enabled: document.shutter.enabled,
                speed_s: document.shutter.speed_s,
                fps: document.shutter.fps,
                angle: document.shutter.angle,
                opacity: document.shutter.opacity,
                history_mix: document.shutter.history_mix,
                history_mix_2: document.shutter.history_mix_2,
                edge_rejection: document.shutter.edge_rejection,
                luma_threshold: document.shutter.luma_threshold,
                frame_hold: document.shutter.frame_hold,
            },
            lens: CameraLens2dSceneCommand {
                profile: document.lens.profile.clone(),
                intensity: document.lens.intensity,
                aberration_px: document.lens.aberration_px,
                distortion: document.lens.distortion,
                vignette: document.lens.vignette,
                edge_softness_px: document.lens.edge_softness_px,
                glare_strength: document.lens.glare_strength,
                dirt: document.lens.dirt,
                focal_length_mm: document.lens.focal_length_mm,
                lens_bloom: document.lens.lens_bloom,
                flare_ghosts: document.lens.flare_ghosts,
                anamorphic_squeeze: document.lens.anamorphic_squeeze,
                coma: document.lens.coma,
                cat_eye_bokeh: document.lens.cat_eye_bokeh,
                focus_breathing: document.lens.focus_breathing,
            },
            lens_surface: CameraLensSurface2dSceneCommand {
                rain_profile: document.lens_surface.rain_profile.clone(),
            },
            film: CameraFilm2dSceneCommand {
                profile: document.film.profile.clone(),
                intensity: document.film.intensity,
                seed: document.film.seed,
                color_shift: document.film.color_shift,
                contrast: document.film.contrast,
                saturation: document.film.saturation,
                flicker: document.film.flicker,
                vignette: document.film.vignette,
                toe: document.film.toe,
                shoulder: document.film.shoulder,
                black_lift: document.film.black_lift,
                print_fade: document.film.print_fade,
                dust: document.film.dust,
                scratches: document.film.scratches,
                push_pull: document.film.push_pull,
                gate_weave: document.film.gate_weave,
                scan_softness: document.film.scan_softness,
            },
            look: CameraLook2dSceneCommand {
                profile: document.look.profile.clone(),
                intensity: document.look.intensity,
            },
            aperture: CameraAperture2dSceneCommand {
                enabled: document.aperture.enabled,
                f_stop: document.aperture.f_stop,
                focus_distance_m: document.aperture.focus_distance_m,
                focus: camera_focus_from_document(&document.aperture.focus),
                depth_of_field: CameraDepthOfField2dSceneCommand {
                    depth_map: document.aperture.depth_of_field.depth_map.clone(),
                    affected_layers: document.aperture.depth_of_field.affected_layers.clone(),
                    max_blur_px: document.aperture.depth_of_field.max_blur_px,
                    depth_contrast: document.aperture.depth_of_field.depth_contrast,
                    focus_width: document.aperture.depth_of_field.focus_width,
                    foreground_blur_boost: document.aperture.depth_of_field.foreground_blur_boost,
                    background_blur_boost: document.aperture.depth_of_field.background_blur_boost,
                    edge_aware: document.aperture.depth_of_field.edge_aware,
                    invert_depth: document.aperture.depth_of_field.invert_depth,
                    debug_view: document.aperture.depth_of_field.debug_view.clone(),
                    aperture_blades: document.aperture.depth_of_field.aperture_blades,
                    aperture_roundness: document.aperture.depth_of_field.aperture_roundness,
                    aperture_rotation_degrees: document
                        .aperture
                        .depth_of_field
                        .aperture_rotation_degrees,
                    sample_count: document.aperture.depth_of_field.sample_count,
                    highlight_threshold: document.aperture.depth_of_field.highlight_threshold,
                    highlight_knee: document.aperture.depth_of_field.highlight_knee,
                    highlight_gain: document.aperture.depth_of_field.highlight_gain,
                    highlight_saturation: document.aperture.depth_of_field.highlight_saturation,
                },
            },
        }),
    });
}

fn camera_render_contribution_defaults() -> [(&'static str, bool); 9] {
    [
        ("camera.projection", true),
        ("camera.exposure", false),
        ("camera.shutter", false),
        ("camera.optics", false),
        ("camera.focus_blur", false),
        ("camera.lens_surface", false),
        ("camera.film", false),
        ("camera.look", false),
        ("camera.scan_output", false),
    ]
}

fn camera_mode_from_document(mode: Camera2dModeDocument) -> CameraExposureMode2dSceneCommand {
    match mode {
        Camera2dModeDocument::Auto => CameraExposureMode2dSceneCommand::Auto,
        Camera2dModeDocument::Manual => CameraExposureMode2dSceneCommand::Manual,
    }
}

fn camera_focus_from_document(focus: &CameraFocus2dDocument) -> CameraFocus2dSceneCommand {
    match focus {
        CameraFocus2dDocument::None => CameraFocus2dSceneCommand::None,
        CameraFocus2dDocument::RenderLayer { layer } => CameraFocus2dSceneCommand::RenderLayer {
            layer: layer.clone(),
        },
        CameraFocus2dDocument::SceneObject { object } => CameraFocus2dSceneCommand::SceneObject {
            object: object.clone(),
        },
        CameraFocus2dDocument::Distance { distance_m } => CameraFocus2dSceneCommand::Distance {
            distance_m: distance_m.max(0.0),
        },
        CameraFocus2dDocument::Depth { value } => {
            CameraFocus2dSceneCommand::Depth { value: *value }
        }
    }
}
