use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_2d_post_fx::{
    CameraExposure2d, CameraExposureMode2d as PostFxExposureMode2d, FocusBlur2d,
    FocusBlurDebugView2d, FocusTarget2d, PostFx2d, PostFx2dInstance, PostFxHost2dId,
    PostFxPipelineKind, PostFxScope2d, RainGlass2d, ScopedPostFx2dStack, ShutterBlur2d,
};
use amigo_assets::AssetCatalog;
use amigo_math::Vec2;
use amigo_scene::{CameraFollow2dSceneCommand, Parallax2dSceneCommand};

use crate::optics::Camera2dRuntimeState;
use crate::{Camera, CameraId};

#[derive(Debug, Default)]
pub struct CameraService {
    cameras: Mutex<BTreeMap<CameraId, Camera>>,
    cameras_2d: Mutex<BTreeMap<CameraId, Camera2dRuntimeState>>,
    lens_rain_overrides: Mutex<BTreeMap<CameraId, RainGlass2d>>,
}

impl CameraService {
    pub fn upsert(&self, camera: Camera) {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .insert(camera.id.clone(), camera);
    }

    pub fn get(&self, id: &CameraId) -> Option<Camera> {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn upsert_2d(&self, camera: Camera2dRuntimeState) {
        self.cameras_2d
            .lock()
            .expect("camera 2d service mutex should not be poisoned")
            .insert(camera.id.clone(), camera);
    }

    pub fn get_2d(&self, id: &CameraId) -> Option<Camera2dRuntimeState> {
        self.cameras_2d
            .lock()
            .expect("camera 2d service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn update_camera_2d<F>(&self, camera_id: &CameraId, update: F) -> bool
    where
        F: FnOnce(&mut Camera2dRuntimeState) -> bool,
    {
        let mut cameras = self
            .cameras_2d
            .lock()
            .expect("camera 2d service mutex should not be poisoned");
        let Some(camera) = cameras.get_mut(camera_id) else {
            return false;
        };
        if !update(camera) {
            return false;
        }
        let normalized = camera.clone().normalized();
        *camera = normalized;
        true
    }

    pub fn set_lens_rain_profile_2d(
        &self,
        camera_id: &CameraId,
        profile: impl Into<String>,
    ) -> bool {
        let mut cameras = self
            .cameras_2d
            .lock()
            .expect("camera2d registry mutex should not be poisoned");

        let Some(camera) = cameras.get_mut(camera_id) else {
            return false;
        };

        camera.lens_surface.rain_profile = Some(profile.into());

        self.lens_rain_overrides
            .lock()
            .expect("camera lens rain override mutex should not be poisoned")
            .remove(camera_id);

        true
    }

    pub fn clear_lens_rain_override_2d(&self, camera_id: &CameraId) -> bool {
        self.lens_rain_overrides
            .lock()
            .expect("camera lens rain override mutex should not be poisoned")
            .remove(camera_id)
            .is_some()
    }

    pub fn update_lens_rain_2d<F>(
        &self,
        camera_id: &CameraId,
        assets: Option<&AssetCatalog>,
        update: F,
    ) -> bool
    where
        F: FnOnce(&mut RainGlass2d),
    {
        let Some(camera) = self.get_2d(camera_id) else {
            return false;
        };

        let mut overrides = self
            .lens_rain_overrides
            .lock()
            .expect("camera lens rain override mutex should not be poisoned");

        let mut rain = overrides
            .get(camera_id)
            .copied()
            .or_else(|| camera.resolved_rain_profile(assets))
            .unwrap_or_default();

        update(&mut rain);
        overrides.insert(camera_id.clone(), rain.normalized());
        true
    }

    pub fn resolved_lens_rain_2d(
        &self,
        camera: &Camera2dRuntimeState,
        assets: Option<&AssetCatalog>,
    ) -> Option<RainGlass2d> {
        self.lens_rain_overrides
            .lock()
            .ok()
            .and_then(|overrides| overrides.get(&camera.id).copied())
            .or_else(|| camera.resolved_rain_profile(assets))
            .map(RainGlass2d::normalized)
    }

    pub fn main_camera_id(&self) -> Option<CameraId> {
        let id = CameraId::new("main");
        self.get(&id).map(|camera| camera.id)
    }

    pub fn camera(&self, id: &CameraId) -> Option<Camera> {
        self.get(id)
    }

    pub fn main_camera2d_id(&self) -> Option<CameraId> {
        let cameras = self
            .cameras_2d
            .lock()
            .expect("camera 2d service mutex should not be poisoned");
        if cameras.contains_key(&CameraId::main()) {
            return Some(CameraId::main());
        }
        cameras.keys().next().cloned()
    }

    pub fn main_camera2d(&self) -> Option<Camera2dRuntimeState> {
        let id = self.main_camera2d_id()?;
        self.get_2d(&id)
    }

    pub fn cameras_2d(&self) -> Vec<Camera2dRuntimeState> {
        self.cameras_2d
            .lock()
            .expect("camera 2d service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn camera_by_binding(&self, binding: &amigo_render_api::CameraBinding) -> Option<Camera> {
        let id = CameraId::new(binding.camera_id.clone());
        self.camera(&id).or_else(|| match binding.fallback {
            amigo_render_api::CameraFallback::Main => {
                self.main_camera_id().and_then(|main| self.camera(&main))
            }
            amigo_render_api::CameraFallback::None => None,
        })
    }

    pub fn cameras(&self) -> Vec<Camera> {
        self.cameras
            .lock()
            .expect("camera service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn frame_post_fx_stacks(&self, assets: Option<&AssetCatalog>) -> Vec<ScopedPostFx2dStack> {
        let Some(camera) = self.main_camera2d() else {
            return Vec::new();
        };

        let mut effects = Vec::new();
        let camera_id = camera.id.0.clone();

        effects.push(PostFx2dInstance {
            id: format!("camera:{camera_id}:0:camera_exposure").into(),
            effect: PostFx2d::CameraExposure(
                CameraExposure2d {
                    mode: match camera.mode {
                        crate::optics::CameraExposureMode2d::Auto => PostFxExposureMode2d::Auto,
                        crate::optics::CameraExposureMode2d::Manual => PostFxExposureMode2d::Manual,
                    },
                    iso: camera.exposure.iso,
                    compensation: camera.exposure.compensation,
                    white_balance: camera.exposure.white_balance,
                    nd_stops: camera.exposure.nd_stops,
                    target_luma: camera.exposure.auto.target_luma,
                    adaptation_speed: camera.exposure.auto.adaptation_speed,
                    min_iso: camera.exposure.auto.min_iso,
                    max_iso: camera.exposure.auto.max_iso,
                    opacity: 1.0,
                }
                .normalized(),
            ),
        });

        if camera.shutter.enabled && camera.shutter.opacity > 0.0 && camera.shutter.angle > 0.0 {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:1:shutter_blur").into(),
                effect: PostFx2d::ShutterBlur(
                    ShutterBlur2d {
                        fps: camera.shutter.fps,
                        shutter_angle: camera.shutter.angle,
                        opacity: camera.shutter.opacity,
                        history_mix: camera.shutter.history_mix,
                        history_mix_2: camera.shutter.history_mix_2,
                        edge_rejection: camera.shutter.edge_rejection,
                        luma_threshold: camera.shutter.luma_threshold,
                        frame_hold: camera.shutter.frame_hold,
                    }
                    .normalized(),
                ),
            });
        }

        let lens = camera.resolved_lens_profile(assets);
        if camera.lens.intensity > 0.0 {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:2:camera_optics").into(),
                effect: PostFx2d::CameraOptics(
                    amigo_2d_post_fx::CameraOptics2d {
                        focal_length_mm: lens.focal_length_mm,
                        aberration_px: lens.aberration_px,
                        distortion: lens.distortion,
                        vignette: lens.vignette,
                        edge_softness_px: lens.edge_softness_px,
                        flare_strength: lens.flare_strength,
                        lens_bloom: lens.lens_bloom,
                        flare_ghosts: lens.flare_ghosts,
                        anamorphic_squeeze: lens.anamorphic_squeeze,
                        coma: lens.coma,
                        dirt: lens.dirt,
                        halation_bias: lens.halation_bias,
                        opacity: camera.lens.intensity,
                    }
                    .normalized(),
                ),
            });
        }

        if camera.aperture.enabled {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:3:focus_blur").into(),
                effect: PostFx2d::FocusBlur(
                    FocusBlur2d {
                        focus: match &camera.aperture.focus {
                            crate::optics::CameraFocus2d::None => FocusTarget2d::None,
                            crate::optics::CameraFocus2d::RenderLayer { layer } => {
                                FocusTarget2d::RenderLayer {
                                    layer: layer.clone(),
                                }
                            }
                            crate::optics::CameraFocus2d::SceneObject { object } => {
                                FocusTarget2d::SceneObject {
                                    object: object.clone(),
                                }
                            }
                            crate::optics::CameraFocus2d::Depth { value } => {
                                FocusTarget2d::Depth { value: *value }
                            }
                        },
                        f_stop: camera.aperture.f_stop,
                        focus_distance_m: camera.aperture.focus_distance_m,
                        focus_radius: (camera.aperture.focus_distance_m.recip()
                            * camera.aperture.f_stop)
                            .clamp(0.02, 0.28),
                        blur_radius: ((8.0 - camera.aperture.f_stop).max(0.0) * 1.2
                            + lens.cat_eye_bokeh * 4.0
                            + lens.focus_breathing * 2.0)
                            .clamp(0.0, 18.0),
                        anamorphic_ratio: lens.anamorphic_squeeze,
                        cat_eye_bokeh: lens.cat_eye_bokeh,
                        focus_breathing: lens.focus_breathing,
                        opacity: 1.0,
                        depth_map: camera.aperture.depth_of_field.depth_map.clone(),
                        affected_layers: camera.aperture.depth_of_field.affected_layers.clone(),
                        focal_length_mm: lens.focal_length_mm,
                        max_blur_px: camera.aperture.depth_of_field.max_blur_px,
                        depth_contrast: camera.aperture.depth_of_field.depth_contrast,
                        focus_width: camera.aperture.depth_of_field.focus_width,
                        foreground_blur_boost: camera.aperture.depth_of_field.foreground_blur_boost,
                        background_blur_boost: camera.aperture.depth_of_field.background_blur_boost,
                        edge_aware: camera.aperture.depth_of_field.edge_aware,
                        invert_depth: camera.aperture.depth_of_field.invert_depth,
                        debug_view: FocusBlurDebugView2d::parse(
                            &camera.aperture.depth_of_field.debug_view,
                        ),
                        aperture_blades: camera.aperture.depth_of_field.aperture_blades,
                        aperture_roundness: camera.aperture.depth_of_field.aperture_roundness,
                        aperture_rotation_degrees: camera
                            .aperture
                            .depth_of_field
                            .aperture_rotation_degrees,
                        sample_count: camera.aperture.depth_of_field.sample_count,
                        highlight_threshold: camera.aperture.depth_of_field.highlight_threshold,
                        highlight_knee: camera.aperture.depth_of_field.highlight_knee,
                        highlight_gain: camera.aperture.depth_of_field.highlight_gain,
                        highlight_saturation: camera.aperture.depth_of_field.highlight_saturation,
                    }
                    .normalized(),
                ),
            });
        }

        if let Some(rain) = self
            .resolved_lens_rain_2d(&camera, assets)
            .filter(|rain| rain.is_active())
        {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:4:rain_glass").into(),
                effect: PostFx2d::RainGlass(rain),
            });
        }

        let film = camera.resolved_film_stock(assets);
        if camera.film.intensity > 0.0 {
            let iso_factor = (camera.exposure.iso / film.base_iso)
                .sqrt()
                .clamp(0.35, 3.0);

            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:5:film_emulsion").into(),
                effect: PostFx2d::FilmEmulsion(
                    amigo_2d_post_fx::FilmEmulsion2d {
                        color_shift: film.color_shift,
                        contrast: film.contrast,
                        saturation: film.saturation,
                        toe: film.toe,
                        shoulder: film.shoulder,
                        black_lift: film.black_lift,
                        push_pull: film.push_pull,
                        opacity: camera.film.intensity,
                    }
                    .normalized(),
                ),
            });

            if let Some(mut look) = camera
                .resolved_look_profile(assets)
                .filter(|look| look.is_active())
            {
                look.opacity *= camera.look.intensity;
                effects.push(PostFx2dInstance {
                    id: format!("camera:{camera_id}:6:look").into(),
                    effect: PostFx2d::ColorRamp(look.normalized()),
                });
            }

            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:7:scan_output").into(),
                effect: PostFx2d::ScanOutput(
                    amigo_2d_post_fx::ScanOutput2d {
                        iso: camera.exposure.iso,
                        flicker: film.flicker,
                        vignette: film.vignette,
                        print_fade: film.print_fade,
                        dust: film.dust,
                        scratches: film.scratches,
                        gate_weave: film.gate_weave,
                        scan_softness: film.scan_softness,
                        opacity: (film.opacity * iso_factor * camera.film.intensity)
                            .clamp(0.0, 1.0),
                        seed: camera.film.seed,
                        grain_chroma: film.grain.chroma_amount * iso_factor,
                        grain_luma: film.grain.luma_amount,
                        shadow_grain: film.grain.shadow_amount,
                        midtone_grain: film.grain.midtone_amount,
                        highlight_grain: film.grain.highlight_amount,
                        highlight_suppression: film.grain.highlight_suppression,
                        fine_grain_px: film.grain.fine_grain_px,
                        medium_grain_px: film.grain.medium_grain_px,
                        coarse_grain_px: film.grain.coarse_grain_px,
                        clumpiness: film.grain.clumpiness,
                        grain_softness: film.grain.softness,
                        underexposure_grain_boost: film.grain.underexposure_boost,
                        push_process_boost: film.grain.push_process_boost,
                        density_pivot: film.grain.density_pivot,
                        channel_balance: film.grain.channel_balance,
                        temporal_jitter: film.grain.temporal_jitter,
                        grain_regenerate_per_frame: film.grain.regenerate_per_frame,
                    }
                    .normalized(),
                ),
            });
        }

        if effects.is_empty() {
            return Vec::new();
        }

        vec![ScopedPostFx2dStack {
            host_id: PostFxHost2dId::new(format!("camera:{camera_id}:frame")),
            scope: PostFxScope2d::Frame,
            pipeline: PostFxPipelineKind::FrameGraph,
            effects,
        }]
    }
}

#[derive(Debug, Default)]
pub struct CameraFollow2dSceneService {
    commands: Mutex<Vec<CameraFollow2dSceneCommand>>,
}

impl CameraFollow2dSceneService {
    pub fn queue(&self, command: CameraFollow2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<CameraFollow2dSceneCommand> {
        self.commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned")
            .clone()
    }

    pub fn follow(&self, entity_name: &str) -> Option<CameraFollow2dSceneCommand> {
        self.commands()
            .into_iter()
            .find(|command| command.entity_name == entity_name)
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optics::{
        Camera2dRuntimeState, CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d,
        CameraExposure2d, CameraExposureMode2d, CameraFilm2d, CameraFocus2d, CameraLens2d,
        CameraLensSurface2d, CameraLook2d, CameraShutter2d,
    };
    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn camera_state_with_rain_profile(profile: Option<String>) -> Camera2dRuntimeState {
        Camera2dRuntimeState {
            id: CameraId("main".to_owned()),
            entity_name: "camera.main".to_owned(),
            mode: CameraExposureMode2d::Manual,
            exposure: CameraExposure2d {
                iso: 800.0,
                compensation: 0.0,
                white_balance: 5600.0,
                nd_stops: 0.0,
                auto: CameraAutoExposure2d {
                    target_luma: 0.42,
                    adaptation_speed: 0.8,
                    min_iso: 100.0,
                    max_iso: 3200.0,
                },
            },
            shutter: CameraShutter2d {
                enabled: false,
                fps: 24.0,
                angle: 180.0,
                opacity: 0.0,
                history_mix: 0.0,
                history_mix_2: 0.0,
                edge_rejection: 0.35,
                luma_threshold: 0.04,
                frame_hold: false,
            },
            lens: CameraLens2d {
                profile: "clean_modern_35mm".to_owned(),
                intensity: 0.0,
                aberration_px: None,
                distortion: None,
                vignette: None,
                edge_softness_px: None,
                flare_strength: None,
                dirt: None,
                focal_length_mm: None,
                lens_bloom: None,
                flare_ghosts: None,
                anamorphic_squeeze: 1.0,
                coma: None,
                cat_eye_bokeh: None,
                focus_breathing: None,
            },
            lens_surface: CameraLensSurface2d {
                rain_profile: profile,
            },
            film: CameraFilm2d {
                profile: "neutral_digital_400".to_owned(),
                intensity: 1.0,
                seed: 1986,
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
            look: CameraLook2d {
                profile: "mod/camera/look/custom".to_owned(),
                intensity: 0.75,
            },
            aperture: CameraAperture2d {
                enabled: false,
                f_stop: 8.0,
                focus_distance_m: 5.0,
                focus: CameraFocus2d::None,
                depth_of_field: CameraDepthOfField2d {
                    depth_map: None,
                    affected_layers: Vec::new(),
                    max_blur_px: 28.0,
                    depth_contrast: 1.0,
                    focus_width: 0.055,
                    foreground_blur_boost: 1.15,
                    background_blur_boost: 1.0,
                    edge_aware: true,
                    invert_depth: false,
                    debug_view: "final".to_owned(),
                    aperture_blades: 7,
                    aperture_roundness: 0.72,
                    aperture_rotation_degrees: 0.0,
                    sample_count: 64,
                    highlight_threshold: 0.68,
                    highlight_knee: 0.18,
                    highlight_gain: 1.45,
                    highlight_saturation: 1.10,
                },
            },
        }
        .normalized()
    }

    fn test_asset_catalog_with_rain_profile() -> AssetCatalog {
        let assets = AssetCatalog::default();
        assets.mark_prepared(PreparedAsset {
            key: AssetKey::new("rotten-club/camera/rain/test-rain"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/rain/test-rain.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-rain-glass-profile-2d".to_owned()),
            label: Some("Test Rain".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("spawn.spawn_rate".to_owned(), "4.0".to_owned()),
                ("compose.opacity".to_owned(), "0.25".to_owned()),
            ]),
        });
        assets
    }

    fn find_first_rain_glass(stacks: &[ScopedPostFx2dStack]) -> Option<RainGlass2d> {
        stacks.iter().find_map(|stack| {
            stack
                .effects
                .iter()
                .find_map(|instance| match instance.effect {
                    PostFx2d::RainGlass(rain) => Some(rain),
                    _ => None,
                })
        })
    }

    #[test]
    fn frame_post_fx_stack_inserts_look_between_film_emulsion_and_scan_output() {
        let assets = AssetCatalog::default();
        assets.mark_prepared(PreparedAsset {
            key: AssetKey::new("mod/camera/look/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/look/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-look-profile-2d".to_owned()),
            label: Some("Custom Look".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_look".to_owned()),
                ("palette_size".to_owned(), "24".to_owned()),
                ("opacity".to_owned(), "0.8".to_owned()),
            ]),
        });

        let service = CameraService::default();
        service.upsert_2d(camera_state_with_rain_profile(None));

        let stacks = service.frame_post_fx_stacks(Some(&assets));
        let effects = &stacks[0].effects;
        let kinds = effects
            .iter()
            .map(|effect| effect.effect.kind())
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                "camera_exposure",
                "film_emulsion",
                "color_ramp",
                "scan_output"
            ]
        );
        assert_eq!(effects[2].id.as_str(), "camera:main:6:look");
        assert_eq!(effects[3].id.as_str(), "camera:main:7:scan_output");

        let PostFx2d::ColorRamp(effect) = &effects[2].effect else {
            panic!("expected color_ramp look effect");
        };
        assert_eq!(effect.palette_size, 24);
        assert!((effect.opacity - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn lens_rain_override_replaces_profile_in_camera_stack() {
        let service = CameraService::default();
        let camera =
            camera_state_with_rain_profile(Some("rotten-club/camera/rain/test-rain".to_owned()));
        service.upsert_2d(camera.clone());

        let assets = test_asset_catalog_with_rain_profile();

        let before = service.frame_post_fx_stacks(Some(&assets));
        let before_rain = find_first_rain_glass(&before).expect("rain profile should resolve");
        assert_eq!(before_rain.spawn_rate, 4.0);
        assert_eq!(before_rain.opacity, 0.25);

        assert!(
            service.update_lens_rain_2d(&camera.id, Some(&assets), |rain| {
                rain.spawn_rate = 22.0;
                rain.opacity = 0.7;
            })
        );

        let after = service.frame_post_fx_stacks(Some(&assets));
        let after_rain = find_first_rain_glass(&after).expect("rain override should resolve");
        assert_eq!(after_rain.spawn_rate, 22.0);
        assert_eq!(after_rain.opacity, 0.7);
    }

    #[test]
    fn clearing_lens_rain_override_restores_profile() {
        let service = CameraService::default();
        let camera =
            camera_state_with_rain_profile(Some("rotten-club/camera/rain/test-rain".to_owned()));
        service.upsert_2d(camera.clone());

        let assets = test_asset_catalog_with_rain_profile();

        assert!(
            service.update_lens_rain_2d(&camera.id, Some(&assets), |rain| {
                rain.spawn_rate = 22.0;
            })
        );
        assert!(service.clear_lens_rain_override_2d(&camera.id));

        let stacks = service.frame_post_fx_stacks(Some(&assets));
        let rain = find_first_rain_glass(&stacks).expect("rain profile should resolve");
        assert_eq!(rain.spawn_rate, 4.0);
    }
}

#[derive(Debug, Default)]
pub struct Parallax2dSceneService {
    commands: Mutex<Vec<Parallax2dSceneCommand>>,
}

impl Parallax2dSceneService {
    pub fn queue(&self, command: Parallax2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<Parallax2dSceneCommand> {
        self.commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_camera_origin(&self, entity_name: &str, camera_origin: Vec2) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.camera_origin = Some(camera_origin);
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}
