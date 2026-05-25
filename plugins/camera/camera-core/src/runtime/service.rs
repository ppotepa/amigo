use std::collections::BTreeMap;
use std::sync::Mutex;

mod focus_transition;

use crate::api::CameraDepthMotion2d;
use amigo_assets::AssetCatalog;
use amigo_composite_plugin::{
    CameraExposure2d, CameraExposureMode2d as PostFxExposureMode2d, FocusBlur2d,
    FocusBlurDebugView2d, FocusTarget2d, PostFx2dInstance, PostFxHost2dId, PostFxPipelineKind,
    PostFxScope2d, RainGlass2d, ScopedPostFx2dStack, ShutterBlur2d,
};
use amigo_math::Vec2;
use amigo_render_api::{
    post_fx_camera_exposure, post_fx_camera_optics, post_fx_color_ramp, post_fx_film_emulsion,
    post_fx_focus_blur, post_fx_rain_glass, post_fx_scan_output, post_fx_shutter_blur,
    render_contribution_roles as roles,
};
use amigo_scene::{CameraFollow2dSceneCommand, Parallax2dSceneCommand};

use crate::runtime::rig::{
    apply_camera_depth_motion_to_rig, resolve_camera_rig_2d, ResolvedCameraRig2d,
};
use crate::{Camera, CameraFocusTarget2dService, CameraFocusTransition2d, CameraId};
use amigo_camera_optics_plugin::runtime::Camera2dRuntimeState;
use amigo_camera_profiles_plugin::api::{CameraQualityProfile2d, CameraQualitySettings2d};
use self::focus_transition::{
    apply_focus_transition_target, focus_transition_start_for, lerp_focus_transition_target,
    transition_target_for_depth,
};

#[derive(Debug, Default)]
pub struct CameraService {
    cameras: Mutex<BTreeMap<CameraId, Camera>>,
    cameras_2d: Mutex<BTreeMap<CameraId, Camera2dRuntimeState>>,
    lens_rain_overrides: Mutex<BTreeMap<CameraId, RainGlass2d>>,
    quality_profiles_2d: Mutex<BTreeMap<CameraId, CameraQualityProfile2d>>,
    debug_views_2d: Mutex<BTreeMap<CameraId, amigo_render_api::CameraDebugView2d>>,
    focus_transitions_2d: Mutex<BTreeMap<CameraId, CameraFocusTransition2d>>,
    sway_2d: Mutex<BTreeMap<CameraId, CameraSway2d>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSway2d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub z_offset: f32,
    pub zoom: f32,
    pub rotation: f32,
    pub frequency: f32,
    pub phase_seconds: f32,
    pub affects_focus: bool,
    pub camera_z_m: f32,
    pub focus_residual_m: f32,
    pub dolly_signal: f32,
}

impl Default for CameraSway2d {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            z_offset: 0.0,
            zoom: 0.0,
            rotation: 0.0,
            frequency: 0.0,
            phase_seconds: 0.0,
            affects_focus: false,
            camera_z_m: 0.0,
            focus_residual_m: 0.0,
            dolly_signal: 0.0,
        }
    }
}

impl CameraSway2d {
    pub fn normalized(mut self) -> Self {
        self.x = finite_or_zero(self.x).clamp(-1.0, 1.0);
        self.y = finite_or_zero(self.y).clamp(-1.0, 1.0);
        self.z = finite_or_zero(self.z).clamp(-1.0, 1.0);
        self.z_offset = finite_or_zero(self.z_offset).clamp(-1.0, 1.0);
        self.zoom = finite_or_zero(self.zoom).clamp(-1.0, 1.0);
        self.rotation = finite_or_zero(self.rotation).clamp(-1.0, 1.0);
        self.frequency = finite_or_zero(self.frequency).clamp(0.0, 20.0);
        self.phase_seconds = finite_or_zero(self.phase_seconds).max(0.0);
        self.camera_z_m = finite_or_zero(self.camera_z_m).clamp(-50.0, 50.0);
        self.focus_residual_m = finite_or_zero(self.focus_residual_m).clamp(-5.0, 5.0);
        self.dolly_signal = finite_or_zero(self.dolly_signal).clamp(-1.0, 1.0);
        self
    }

    pub fn depth_motion(self) -> CameraDepthMotion2d {
        CameraDepthMotion2d {
            camera_z_m: self.camera_z_m,
            focus_residual_m: self.focus_residual_m,
            dolly_signal: self.dolly_signal,
        }
        .normalized()
    }

    pub fn focus_offset(self) -> f32 {
        if self.affects_focus {
            let periodic = if self.frequency > 0.0 && self.z.abs() > f32::EPSILON {
                (self.phase_seconds * self.frequency).sin() * self.z
            } else {
                0.0
            };
            (periodic + self.z_offset).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }
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

    pub fn set_quality_profile_2d(
        &self,
        camera_id: &CameraId,
        profile: CameraQualityProfile2d,
    ) -> bool {
        if self.get_2d(camera_id).is_none() {
            return false;
        }
        self.quality_profiles_2d
            .lock()
            .expect("camera quality profile mutex should not be poisoned")
            .insert(camera_id.clone(), profile);
        true
    }

    pub fn quality_profile_2d(&self, camera_id: &CameraId) -> CameraQualityProfile2d {
        self.quality_profiles_2d
            .lock()
            .expect("camera quality profile mutex should not be poisoned")
            .get(camera_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_debug_view_2d(
        &self,
        camera_id: &CameraId,
        debug_view: amigo_render_api::CameraDebugView2d,
    ) -> bool {
        if self.get_2d(camera_id).is_none() {
            return false;
        }
        self.debug_views_2d
            .lock()
            .expect("camera debug view mutex should not be poisoned")
            .insert(camera_id.clone(), debug_view);
        true
    }

    pub fn debug_view_2d(&self, camera_id: &CameraId) -> amigo_render_api::CameraDebugView2d {
        self.debug_views_2d
            .lock()
            .expect("camera debug view mutex should not be poisoned")
            .get(camera_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn focus_2d_on_target(
        &self,
        camera_id: &CameraId,
        selector: &str,
        targets: &CameraFocusTarget2dService,
        duration_seconds: f32,
    ) -> bool {
        let Some(resolved) = targets.resolve(selector) else {
            return false;
        };
        let Some(camera) = self.get_2d(camera_id) else {
            return false;
        };
        let end = transition_target_for_depth(&resolved.target.depth);

        if duration_seconds <= 0.0 {
            self.focus_transitions_2d
                .lock()
                .expect("camera focus transition mutex should not be poisoned")
                .remove(camera_id);
            return self.update_camera_2d(camera_id, |camera| {
                camera.aperture.enabled = true;
                apply_focus_transition_target(&mut camera.aperture, &end);
                true
            });
        }

        let start = focus_transition_start_for(&camera, &end);
        self.focus_transitions_2d
            .lock()
            .expect("camera focus transition mutex should not be poisoned")
            .insert(
                camera_id.clone(),
                CameraFocusTransition2d {
                    selector: resolved.selector,
                    elapsed_seconds: 0.0,
                    duration_seconds: duration_seconds.max(0.001),
                    start,
                    end,
                },
            );
        self.update_camera_2d(camera_id, |camera| {
            camera.aperture.enabled = true;
            true
        })
    }

    pub fn tick_focus_transitions_2d(&self, delta_seconds: f32) {
        let delta_seconds = if delta_seconds.is_finite() {
            delta_seconds.max(0.0)
        } else {
            0.0
        };
        let transitions = self
            .focus_transitions_2d
            .lock()
            .expect("camera focus transition mutex should not be poisoned")
            .clone();
        let mut finished = Vec::new();

        for (camera_id, mut transition) in transitions {
            transition.elapsed_seconds += delta_seconds;
            let t = (transition.elapsed_seconds / transition.duration_seconds.max(0.001))
                .clamp(0.0, 1.0);
            let eased = t * t * (3.0 - 2.0 * t);
            let target = lerp_focus_transition_target(&transition.start, &transition.end, eased);
            let _ = self.update_camera_2d(&camera_id, |camera| {
                camera.aperture.enabled = true;
                apply_focus_transition_target(&mut camera.aperture, &target);
                true
            });

            if t >= 1.0 {
                finished.push(camera_id);
            } else {
                self.focus_transitions_2d
                    .lock()
                    .expect("camera focus transition mutex should not be poisoned")
                    .insert(camera_id, transition);
            }
        }

        if !finished.is_empty() {
            let mut active = self
                .focus_transitions_2d
                .lock()
                .expect("camera focus transition mutex should not be poisoned");
            for camera_id in finished {
                active.remove(&camera_id);
            }
        }
    }

    pub fn active_focus_transition_2d(
        &self,
        camera_id: &CameraId,
    ) -> Option<CameraFocusTransition2d> {
        self.focus_transitions_2d
            .lock()
            .expect("camera focus transition mutex should not be poisoned")
            .get(camera_id)
            .cloned()
    }

    pub fn set_sway_amounts_2d(
        &self,
        camera_id: &CameraId,
        x: f32,
        y: f32,
        z: f32,
        zoom: f32,
        rotation: f32,
    ) -> bool {
        if ![x, y, z, zoom, rotation]
            .iter()
            .all(|value| value.is_finite())
        {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.x = x;
        sway.y = y;
        sway.z = z;
        sway.zoom = zoom;
        sway.rotation = rotation;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_sway_frequency_2d(&self, camera_id: &CameraId, frequency: f32) -> bool {
        if !frequency.is_finite() {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.frequency = frequency;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_sway_z_offset_2d(&self, camera_id: &CameraId, z_offset: f32) -> bool {
        if !z_offset.is_finite() {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.z_offset = z_offset;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_camera_z_m_2d(&self, camera_id: &CameraId, camera_z_m: f32) -> bool {
        if !camera_z_m.is_finite() {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.camera_z_m = camera_z_m;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_focus_residual_m_2d(&self, camera_id: &CameraId, focus_residual_m: f32) -> bool {
        if !focus_residual_m.is_finite() {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.focus_residual_m = focus_residual_m;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_dolly_signal_2d(&self, camera_id: &CameraId, dolly_signal: f32) -> bool {
        if !dolly_signal.is_finite() {
            return false;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.dolly_signal = dolly_signal;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn set_sway_affects_focus_2d(&self, camera_id: &CameraId, affects_focus: bool) -> bool {
        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        let mut sway = sways.get(camera_id).copied().unwrap_or_default();
        sway.affects_focus = affects_focus;
        sways.insert(camera_id.clone(), sway.normalized());
        true
    }

    pub fn clear_sway_2d(&self, camera_id: &CameraId) -> bool {
        self.sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned")
            .remove(camera_id)
            .is_some()
    }

    pub fn active_sway_2d(&self, camera_id: &CameraId) -> CameraSway2d {
        self.sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned")
            .get(camera_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn camera_depth_motion_2d(&self, camera_id: &CameraId) -> CameraDepthMotion2d {
        self.active_sway_2d(camera_id).depth_motion()
    }

    pub fn main_camera_depth_motion_2d(&self) -> Option<CameraDepthMotion2d> {
        let camera_id = self.main_camera2d_id()?;
        Some(self.camera_depth_motion_2d(&camera_id))
    }

    pub fn tick_sway_2d(&self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }

        let mut sways = self
            .sway_2d
            .lock()
            .expect("camera sway mutex should not be poisoned");
        for sway in sways.values_mut() {
            sway.phase_seconds = (sway.phase_seconds + delta_seconds).rem_euclid(3600.0);
        }
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

    pub fn apply_builtin_preset_2d(&self, camera_id: &CameraId, preset_id: &str) -> bool {
        let Some(preset) =
            amigo_camera_profiles_plugin::runtime::camera_preset_2d(preset_id.trim())
        else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.mode = amigo_camera_optics_plugin::runtime::CameraExposureMode2d::Manual;
            camera.exposure.iso = preset.exposure_iso;
            camera.exposure.compensation = preset.exposure_compensation;
            camera.shutter.enabled = preset.shutter_enabled;
            camera.shutter.fps = preset.shutter_fps;
            camera.shutter.angle = preset.shutter_angle;
            camera.shutter.opacity = preset.shutter_opacity;

            camera.lens.profile = preset.lens_profile.to_owned();
            camera.lens.intensity = preset.lens_intensity;
            camera.lens.focal_length_mm = Some(preset.focal_length_mm);
            camera.lens.anamorphic_squeeze = None;

            camera.film.profile = preset.film_profile.to_owned();
            camera.film.intensity = preset.film_intensity;
            camera.film.seed = preset.film_seed;

            camera.look.profile = preset.look_profile.to_owned();
            camera.look.intensity = preset.look_intensity;
            camera.lens_surface.rain_profile = if preset.rain_profile.is_empty() {
                None
            } else {
                Some(preset.rain_profile.to_owned())
            };

            camera.aperture.enabled = true;
            camera.aperture.f_stop = preset.f_stop;
            camera.aperture.focus_distance_m = preset.focus_distance_m;
            camera.aperture.focus = amigo_camera_optics_plugin::runtime::CameraFocus2d::Distance {
                meters: preset.focus_distance_m,
            };
            camera.aperture.depth_of_field.max_blur_px = preset.max_blur_px;
            camera.aperture.depth_of_field.focus_width = preset.focus_width;
            camera.aperture.depth_of_field.foreground_blur_boost = preset.foreground_blur_boost;
            camera.aperture.depth_of_field.background_blur_boost = preset.background_blur_boost;
            camera.aperture.depth_of_field.aperture_blades = preset.aperture_blades;
            camera.aperture.depth_of_field.aperture_roundness = preset.aperture_roundness;
            camera.aperture.depth_of_field.aperture_rotation_degrees =
                preset.aperture_rotation_degrees;
            camera.aperture.depth_of_field.sample_count = preset.sample_count;
            camera.aperture.depth_of_field.highlight_threshold = preset.highlight_threshold;
            camera.aperture.depth_of_field.highlight_knee = preset.highlight_knee;
            camera.aperture.depth_of_field.highlight_gain = preset.highlight_gain;
            camera.aperture.depth_of_field.highlight_saturation = preset.highlight_saturation;
            true
        })
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
        self.camera(&id).or_else(|| match binding.recovery {
            amigo_render_api::CameraRecovery::Main => {
                self.main_camera_id().and_then(|main| self.camera(&main))
            }
            amigo_render_api::CameraRecovery::None => None,
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

    pub fn resolved_camera_rig_2d(
        &self,
        camera_id: &CameraId,
        assets: Option<&AssetCatalog>,
        depth_space: amigo_2d_spatial::DepthSpace2d,
    ) -> Option<ResolvedCameraRig2d> {
        self.get_2d(camera_id).map(|camera| {
            let quality = self.quality_profile_2d(camera_id);
            let mut rig = resolve_camera_rig_2d(&camera, assets, depth_space, quality);
            apply_camera_depth_motion_to_rig(&mut rig, self.camera_depth_motion_2d(camera_id));
            rig
        })
    }

    pub fn main_resolved_camera_rig_2d(
        &self,
        assets: Option<&AssetCatalog>,
        depth_space: amigo_2d_spatial::DepthSpace2d,
    ) -> Option<ResolvedCameraRig2d> {
        let camera_id = self.main_camera2d_id()?;
        self.resolved_camera_rig_2d(&camera_id, assets, depth_space)
    }

    pub fn frame_post_fx_stacks(&self, assets: Option<&AssetCatalog>) -> Vec<ScopedPostFx2dStack> {
        self.frame_post_fx_stacks_for_depth_space(assets, amigo_2d_spatial::DepthSpace2d::default())
    }

    pub fn frame_post_fx_stacks_for_depth_space(
        &self,
        assets: Option<&AssetCatalog>,
        depth_space: amigo_2d_spatial::DepthSpace2d,
    ) -> Vec<ScopedPostFx2dStack> {
        let Some(camera) = self.main_camera2d() else {
            return Vec::new();
        };
        let quality = self.quality_profile_2d(&camera.id);
        let mut rig = resolve_camera_rig_2d(&camera, assets, depth_space, quality);
        apply_camera_depth_motion_to_rig(&mut rig, self.camera_depth_motion_2d(&camera.id));
        rig.lens_surface.rain = self.resolved_lens_rain_2d(&camera, assets);

        let mut effects = Vec::new();
        let camera_id = rig.camera_id.0.clone();
        let contributions = &rig.render_contributions;

        if contributions.enabled_or(roles::CAMERA_EXPOSURE, false) {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:0:camera_exposure").into(),
                effect: post_fx_camera_exposure(
                    CameraExposure2d {
                        mode: match rig.exposure.mode {
                            amigo_camera_optics_plugin::runtime::CameraExposureMode2d::Auto => {
                                PostFxExposureMode2d::Auto
                            }
                            amigo_camera_optics_plugin::runtime::CameraExposureMode2d::Manual => {
                                PostFxExposureMode2d::Manual
                            }
                        },
                        iso: rig.exposure.iso,
                        compensation: rig.exposure.compensation,
                        white_balance: rig.exposure.white_balance,
                        nd_stops: rig.exposure.nd_stops,
                        target_luma: rig.exposure.auto.target_luma,
                        adaptation_speed: rig.exposure.auto.adaptation_speed,
                        min_iso: rig.exposure.auto.min_iso,
                        max_iso: rig.exposure.auto.max_iso,
                        opacity: 1.0,
                    }
                    .normalized(),
                ),
            });
        }

        if contributions.enabled_or(roles::CAMERA_SHUTTER, false)
            && rig.shutter.state.enabled
            && rig.shutter.state.opacity > 0.0
            && rig.shutter.state.exposure_seconds() > 0.0
        {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:1:shutter_blur").into(),
                effect: post_fx_shutter_blur(
                    ShutterBlur2d {
                        exposure_seconds: rig.shutter.state.exposure_seconds(),
                        fps: rig.shutter.state.fps,
                        shutter_angle: rig.shutter.state.angle,
                        opacity: rig.shutter.state.opacity,
                        history_mix: rig.shutter.state.history_mix,
                        history_mix_2: rig.shutter.state.history_mix_2,
                        edge_rejection: rig.shutter.state.edge_rejection,
                        luma_threshold: rig.shutter.state.luma_threshold,
                        frame_hold: rig.shutter.state.frame_hold,
                    }
                    .normalized(),
                ),
            });
        }

        let lens = &rig.lens.profile;
        if contributions.enabled_or(roles::CAMERA_OPTICS, false) && rig.lens.intensity > 0.0 {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:2:camera_optics").into(),
                effect: post_fx_camera_optics(
                    amigo_composite_plugin::CameraOptics2d {
                        focal_length_mm: lens.focal_length_mm,
                        aberration_px: lens.aberration_px,
                        distortion: lens.distortion,
                        vignette: lens.vignette,
                        edge_softness_px: lens.edge_softness_px,
                        glare_strength: lens.glare_strength,
                        lens_bloom: lens.lens_bloom,
                        flare_ghosts: lens.flare_ghosts,
                        anamorphic_squeeze: lens.anamorphic_squeeze,
                        coma: lens.coma,
                        dirt: lens.dirt,
                        halation_bias: lens.halation_bias,
                        opacity: rig.lens.intensity,
                    }
                    .normalized(),
                ),
            });
        }

        if contributions.enabled_or(roles::CAMERA_FOCUS_BLUR, false) && rig.aperture.state.enabled {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:3:focus_blur").into(),
                effect: post_fx_focus_blur(
                    FocusBlur2d {
                        focus: match &rig.aperture.focus {
                            amigo_camera_optics_plugin::runtime::CameraFocus2d::None => {
                                FocusTarget2d::None
                            }
                            amigo_camera_optics_plugin::runtime::CameraFocus2d::RenderLayer {
                                layer,
                            } => FocusTarget2d::RenderLayer {
                                layer: layer.clone(),
                            },
                            amigo_camera_optics_plugin::runtime::CameraFocus2d::SceneObject {
                                object,
                            } => FocusTarget2d::SceneObject {
                                object: object.clone(),
                            },
                            amigo_camera_optics_plugin::runtime::CameraFocus2d::Distance {
                                meters,
                            } => FocusTarget2d::Depth {
                                value: rig.aperture.computed_focus_z_depth.unwrap_or_else(|| {
                                    amigo_2d_spatial::distance_to_z_depth(*meters, rig.depth_space)
                                }),
                            },
                            amigo_camera_optics_plugin::runtime::CameraFocus2d::Depth { value } => {
                                FocusTarget2d::Depth { value: *value }
                            }
                        },
                        f_stop: rig.aperture.state.f_stop,
                        focus_distance_m: rig
                            .aperture
                            .effective_focus_distance_m
                            .unwrap_or(rig.aperture.focus_distance_m),
                        focus_radius: (rig
                            .aperture
                            .effective_focus_distance_m
                            .unwrap_or(rig.aperture.focus_distance_m)
                            .recip()
                            * rig.aperture.state.f_stop)
                            .clamp(0.02, 0.28),
                        blur_radius: ((8.0 - rig.aperture.state.f_stop).max(0.0) * 1.2
                            + lens.cat_eye_bokeh * 4.0
                            + lens.focus_breathing * 2.0)
                            .clamp(0.0, 18.0),
                        anamorphic_ratio: lens.anamorphic_squeeze,
                        cat_eye_bokeh: lens.cat_eye_bokeh,
                        focus_breathing: lens.focus_breathing,
                        opacity: 1.0,
                        depth_map: rig.aperture.depth_of_field.depth_map.clone(),
                        affected_layers: rig.aperture.depth_of_field.affected_layers.clone(),
                        focal_length_mm: lens.focal_length_mm,
                        max_blur_px: rig.aperture.depth_of_field.max_blur_px,
                        depth_contrast: rig.aperture.depth_of_field.depth_contrast,
                        focus_width: rig.aperture.depth_of_field.focus_width,
                        foreground_blur_boost: rig.aperture.depth_of_field.foreground_blur_boost,
                        background_blur_boost: rig.aperture.depth_of_field.background_blur_boost,
                        edge_aware: rig.aperture.depth_of_field.edge_aware,
                        invert_depth: rig.aperture.depth_of_field.invert_depth,
                        debug_view: FocusBlurDebugView2d::parse(
                            &rig.aperture.depth_of_field.debug_view,
                        ),
                        aperture_blades: rig.aperture.depth_of_field.aperture_blades,
                        aperture_roundness: rig.aperture.depth_of_field.aperture_roundness,
                        aperture_rotation_degrees: rig
                            .aperture
                            .depth_of_field
                            .aperture_rotation_degrees,
                        sample_count: ((rig.aperture.depth_of_field.sample_count as f32
                            * rig.quality_settings.dof_sample_scale)
                            .round() as u32)
                            .clamp(12, 96),
                        highlight_threshold: rig.aperture.depth_of_field.highlight_threshold,
                        highlight_knee: rig.aperture.depth_of_field.highlight_knee,
                        highlight_gain: rig.aperture.depth_of_field.highlight_gain
                            * rig.quality_settings.highlight_bokeh_scale,
                        highlight_saturation: rig.aperture.depth_of_field.highlight_saturation,
                    }
                    .normalized(),
                ),
            });
        }

        if contributions.enabled_or(roles::CAMERA_LENS_SURFACE, false) {
            if let Some(mut rain) = rig
                .lens_surface
                .rain
                .clone()
                .filter(|rain| rain.is_active())
            {
                apply_camera_focus_to_rain_glass(&mut rain, &rig);
                apply_camera_quality_to_rain_glass(&mut rain, rig.quality_settings);
                effects.push(PostFx2dInstance {
                    id: format!("camera:{camera_id}:4:rain_glass").into(),
                    effect: post_fx_rain_glass(rain),
                });
            }
        }

        let film = &rig.film.stock;
        if contributions.enabled_or(roles::CAMERA_FILM, false) && rig.film.intensity > 0.0 {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:5:film_emulsion").into(),
                effect: post_fx_film_emulsion(
                    amigo_composite_plugin::FilmEmulsion2d {
                        color_shift: film.color_shift,
                        contrast: film.contrast,
                        saturation: film.saturation,
                        toe: film.toe,
                        shoulder: film.shoulder,
                        black_lift: film.black_lift,
                        push_pull: film.push_pull,
                        opacity: rig.film.intensity,
                    }
                    .normalized(),
                ),
            });
        }

        if contributions.enabled_or(roles::CAMERA_LOOK, false) {
            if let Some(mut look) = rig.look.profile.clone().filter(|look| look.is_active()) {
                look.opacity *= rig.look.intensity;
                effects.push(PostFx2dInstance {
                    id: format!("camera:{camera_id}:6:look").into(),
                    effect: post_fx_color_ramp(look.normalized()),
                });
            }
        }

        if contributions.enabled_or(roles::CAMERA_SCAN_OUTPUT, false) && rig.film.intensity > 0.0 {
            effects.push(PostFx2dInstance {
                id: format!("camera:{camera_id}:7:scan_output").into(),
                effect: post_fx_scan_output(
                    amigo_composite_plugin::ScanOutput2d {
                        iso: rig.exposure.iso,
                        flicker: film.flicker,
                        vignette: film.vignette,
                        print_fade: film.print_fade,
                        dust: film.dust,
                        scratches: film.scratches,
                        gate_weave: film.gate_weave,
                        scan_softness: film.scan_softness,
                        opacity: (film.opacity * rig.film.iso_factor * rig.film.intensity)
                            .clamp(0.0, 1.0),
                        seed: rig.film.seed,
                        grain_chroma: film.grain.chroma_amount * rig.film.iso_factor,
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

    pub fn camera_render_contributions_summary_for_depth_space(
        &self,
        assets: Option<&AssetCatalog>,
        depth_space: amigo_2d_spatial::DepthSpace2d,
    ) -> String {
        let Some(camera) = self.main_camera2d() else {
            return "render.contributions:\ncomponent=Camera2D skipped reason=no_main_camera"
                .to_owned();
        };
        let quality = self.quality_profile_2d(&camera.id);
        let mut rig = resolve_camera_rig_2d(&camera, assets, depth_space, quality);
        rig.lens_surface.rain = self.resolved_lens_rain_2d(&camera, assets);
        let contributions = &rig.render_contributions;
        let mut lines = Vec::new();
        lines.push("render.contributions:".to_owned());
        lines.push(format!("entity={} component=Camera2D", rig.entity_name));
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_PROJECTION,
            contributions.enabled_or(roles::CAMERA_PROJECTION, true),
            "main_camera",
            "disabled_by_authoring",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_EXPOSURE,
            contributions.enabled_or(roles::CAMERA_EXPOSURE, false),
            "enabled_by_authoring",
            "disabled_by_authoring_or_default",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_SHUTTER,
            contributions.enabled_or(roles::CAMERA_SHUTTER, false) && rig.shutter.state.enabled,
            "enabled_by_authoring+shutter_enabled",
            "disabled_by_authoring_or_shutter_disabled",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_OPTICS,
            contributions.enabled_or(roles::CAMERA_OPTICS, false) && rig.lens.intensity > 0.0,
            "enabled_by_authoring+lens_intensity",
            "disabled_by_authoring_or_zero_intensity",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_FOCUS_BLUR,
            contributions.enabled_or(roles::CAMERA_FOCUS_BLUR, false) && rig.aperture.state.enabled,
            "enabled_by_authoring+aperture_enabled",
            "disabled_by_authoring_or_aperture_disabled",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_LENS_SURFACE,
            contributions.enabled_or(roles::CAMERA_LENS_SURFACE, false)
                && rig
                    .lens_surface
                    .rain
                    .as_ref()
                    .is_some_and(|rain| rain.is_active()),
            "enabled_by_authoring+active_lens_surface",
            "disabled_by_authoring_or_no_active_lens_surface",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_FILM,
            contributions.enabled_or(roles::CAMERA_FILM, false) && rig.film.intensity > 0.0,
            &format!(
                "enabled_by_authoring+film_profile profile={} intensity={} black_lift={}",
                rig.film.state.profile, rig.film.intensity, rig.film.stock.black_lift
            ),
            "disabled_by_authoring_or_zero_intensity",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_LOOK,
            contributions.enabled_or(roles::CAMERA_LOOK, false)
                && rig
                    .look
                    .profile
                    .as_ref()
                    .is_some_and(|look| look.is_active()),
            "enabled_by_authoring+active_look",
            "disabled_by_authoring_or_no_active_look",
        );
        push_camera_role_line(
            &mut lines,
            roles::CAMERA_SCAN_OUTPUT,
            contributions.enabled_or(roles::CAMERA_SCAN_OUTPUT, false) && rig.film.intensity > 0.0,
            &format!(
                "enabled_by_authoring+film_profile print_fade={} grain_luma={}",
                rig.film.stock.print_fade, rig.film.stock.grain.luma_amount
            ),
            "disabled_by_authoring_or_zero_film_intensity",
        );
        lines.join("\n")
    }
}

fn push_camera_role_line(
    lines: &mut Vec<String>,
    role: &str,
    active: bool,
    active_reason: &str,
    skipped_reason: &str,
) {
    if active {
        lines.push(format!("role {role}: active reason={active_reason}"));
    } else {
        lines.push(format!("role {role}: skipped reason={skipped_reason}"));
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

fn apply_camera_focus_to_rain_glass(rain: &mut RainGlass2d, rig: &ResolvedCameraRig2d) {
    let Some(_) = rain.z_depth else {
        rain.camera_focus_enabled = false;
        return;
    };
    let value = rig
        .aperture
        .computed_focus_z_depth
        .or_else(|| match &rig.aperture.focus {
            amigo_camera_optics_plugin::runtime::CameraFocus2d::Distance { meters } => Some(
                amigo_2d_spatial::distance_to_z_depth(*meters, rig.depth_space),
            ),
            amigo_camera_optics_plugin::runtime::CameraFocus2d::Depth { value } => {
                Some(value.clamp(0.0, 1.0))
            }
            _ => None,
        });
    let Some(value) = value else {
        rain.camera_focus_enabled = false;
        return;
    };
    rain.camera_focus_enabled = rig.aperture.state.enabled;
    rain.camera_focus_depth = value;
    rain.camera_focus_width = rig.aperture.depth_of_field.focus_width;
}

fn apply_camera_quality_to_rain_glass(rain: &mut RainGlass2d, quality: CameraQualitySettings2d) {
    let blur_scale = quality.blur_pass_scale.clamp(0.35, 1.5);
    rain.background_blur_px *= blur_scale;
    rain.background_blur_steps = ((rain.background_blur_steps as f32 * blur_scale).round() as u32)
        .clamp(1, rain.background_blur_steps.max(1));

    let mut resolution_scale = quality.rain_glass_resolution_scale.clamp(0.35, 1.0);
    if quality.debug_buffers {
        resolution_scale = 1.0;
    }
    rain.quality_scale = resolution_scale;
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CameraFocusTargetDepth2d;
    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
    use amigo_camera_optics_plugin::runtime::{
        Camera2dRuntimeState, CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d,
        CameraExposure2d, CameraExposureMode2d, CameraFilm2d, CameraFocus2d, CameraLens2d,
        CameraLensSurface2d, CameraLook2d, CameraShutter2d,
    };
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
                speed_s: None,
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
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([
                (roles::CAMERA_PROJECTION, true),
                (roles::CAMERA_EXPOSURE, true),
                (roles::CAMERA_SHUTTER, true),
                (roles::CAMERA_OPTICS, true),
                (roles::CAMERA_FOCUS_BLUR, true),
                (roles::CAMERA_LENS_SURFACE, true),
                (roles::CAMERA_FILM, true),
                (roles::CAMERA_LOOK, true),
                (roles::CAMERA_SCAN_OUTPUT, true),
            ]),
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
                .find_map(|instance| instance.effect.as_rain_glass().cloned())
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

        let Some(effect) = effects[2].effect.as_color_ramp() else {
            panic!("expected color_ramp look effect");
        };
        assert_eq!(effect.palette_size, 24);
        assert!((effect.opacity - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn bare_camera_render_contributions_do_not_emit_camera_post_fx() {
        let service = CameraService::default();
        let mut camera = camera_state_with_rain_profile(None);

        camera.render_contributions = amigo_render_api::RenderContributionSet::from_pairs([
            (roles::CAMERA_PROJECTION, true),
            (roles::CAMERA_EXPOSURE, false),
            (roles::CAMERA_SHUTTER, false),
            (roles::CAMERA_OPTICS, false),
            (roles::CAMERA_FOCUS_BLUR, false),
            (roles::CAMERA_LENS_SURFACE, false),
            (roles::CAMERA_FILM, false),
            (roles::CAMERA_LOOK, false),
            (roles::CAMERA_SCAN_OUTPUT, false),
        ]);
        camera.shutter.enabled = true;
        camera.shutter.opacity = 0.72;
        camera.lens.intensity = 1.0;
        camera.film.intensity = 1.0;
        camera.aperture.enabled = true;

        service.upsert_2d(camera);

        assert!(service.frame_post_fx_stacks(None).is_empty());
    }

    #[test]
    fn explicit_camera_render_contributions_emit_only_enabled_camera_effects() {
        let service = CameraService::default();
        let mut camera = camera_state_with_rain_profile(None);

        camera.render_contributions = amigo_render_api::RenderContributionSet::from_pairs([
            (roles::CAMERA_PROJECTION, true),
            (roles::CAMERA_EXPOSURE, true),
            (roles::CAMERA_SHUTTER, false),
            (roles::CAMERA_OPTICS, false),
            (roles::CAMERA_FOCUS_BLUR, true),
            (roles::CAMERA_LENS_SURFACE, false),
            (roles::CAMERA_FILM, false),
            (roles::CAMERA_LOOK, false),
            (roles::CAMERA_SCAN_OUTPUT, false),
        ]);
        camera.aperture.enabled = true;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 6.0 };

        service.upsert_2d(camera);

        let stacks = service.frame_post_fx_stacks(None);
        let kinds = stacks
            .iter()
            .flat_map(|stack| &stack.effects)
            .map(|effect| effect.effect.kind())
            .collect::<Vec<_>>();

        assert_eq!(kinds, vec!["camera_exposure", "focus_blur"]);
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

    #[test]
    fn builtin_camera_preset_updates_main_camera_state() {
        let service = CameraService::default();
        service.upsert_2d(camera_state_with_rain_profile(None));

        assert!(service.apply_builtin_preset_2d(&CameraId::new("main"), "anamorphic_rain"));

        let updated = service
            .main_camera2d()
            .expect("main camera should exist after preset apply");
        assert_eq!(updated.lens.profile, "anamorphic_rain_streak");
        assert_eq!(updated.film.profile, "cinestill_800t_halation");
        assert_eq!(updated.film.seed, 7007);
        assert_eq!(updated.look.profile, "rotten_noir_print");
        assert!((updated.look.intensity - 0.35).abs() < f32::EPSILON);
        assert_eq!(
            updated.lens_surface.rain_profile.as_deref(),
            Some("thin_neon_drizzle")
        );
        assert!(!updated.shutter.enabled);
        assert!((updated.shutter.fps - 24.0).abs() < f32::EPSILON);
        assert!((updated.shutter.angle - 180.0).abs() < f32::EPSILON);
        assert!((updated.shutter.opacity - 0.0).abs() < f32::EPSILON);
        assert_eq!(updated.aperture.f_stop, 2.0);
        assert_eq!(updated.aperture.depth_of_field.sample_count, 64);
        assert!(matches!(
            updated.aperture.focus,
            CameraFocus2d::Distance { meters } if (meters - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn camera_focus_distance_is_normalized_to_positive_meters() {
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.focus_distance_m = f32::NAN;
        camera.aperture.focus = CameraFocus2d::Distance { meters: -10.0 };

        let normalized = camera.normalized();

        assert!(matches!(
            normalized.aperture.focus,
            CameraFocus2d::Distance { meters } if meters >= 0.2
        ));
        assert!(normalized.aperture.focus_distance_m >= 0.2);
    }

    fn focus_target_service_with(
        id: &str,
        depth: CameraFocusTargetDepth2d,
    ) -> CameraFocusTarget2dService {
        let service = CameraFocusTarget2dService::default();
        service.replace_all([crate::CameraFocusTarget2d {
            id: id.to_owned(),
            aliases: ["title".to_owned()].into_iter().collect(),
            kind: crate::CameraFocusTarget2dKind::SceneObject,
            entity_name: Some("title".to_owned()),
            render_layer: Some("title.depth2d".to_owned()),
            source_component: Some("Text2D".to_owned()),
            world_position: None,
            depth,
            visible: true,
        }]);
        service
    }

    #[test]
    fn camera_focus_transition_reaches_distance_target() {
        let service = CameraService::default();
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.focus_distance_m = 6.0;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 6.0 };
        service.upsert_2d(camera);
        let targets = focus_target_service_with(
            "entity:title",
            CameraFocusTargetDepth2d::Distance {
                meters: 1.0,
                z_depth: 0.8,
            },
        );

        assert!(service.focus_2d_on_target(&CameraId::main(), "title", &targets, 2.0));
        service.tick_focus_transitions_2d(2.0);

        let camera = service
            .get_2d(&CameraId::main())
            .expect("camera should exist");
        assert!(matches!(
            camera.aperture.focus,
            CameraFocus2d::Distance { meters } if (meters - 1.0).abs() < 0.001
        ));
        assert!(service
            .active_focus_transition_2d(&CameraId::main())
            .is_none());
    }

    #[test]
    fn camera_focus_transition_reaches_depth_target() {
        let service = CameraService::default();
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.focus = CameraFocus2d::Depth { value: 0.2 };
        service.upsert_2d(camera);
        let targets = focus_target_service_with(
            "entity:title",
            CameraFocusTargetDepth2d::Depth { z_depth: 0.66 },
        );

        assert!(service.focus_2d_on_target(&CameraId::main(), "title", &targets, 1.0));
        service.tick_focus_transitions_2d(1.0);

        let camera = service
            .get_2d(&CameraId::main())
            .expect("camera should exist");
        assert!(matches!(
            camera.aperture.focus,
            CameraFocus2d::Depth { value } if (value - 0.66).abs() < 0.001
        ));
    }

    #[test]
    fn camera_depth_motion_modulates_distance_focus() {
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.focus_distance_m = 8.0;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 8.0 };
        let mut rig = resolve_camera_rig_2d(
            &camera,
            None,
            amigo_2d_spatial::DepthSpace2d::default(),
            CameraQualityProfile2d::default(),
        );

        apply_camera_depth_motion_to_rig(
            &mut rig,
            CameraDepthMotion2d {
                camera_z_m: 2.0,
                ..Default::default()
            },
        );

        assert_eq!(rig.aperture.base_focus_distance_m, Some(8.0));
        assert_eq!(rig.aperture.effective_focus_distance_m, Some(6.0));
    }

    #[test]
    fn previous_z_offset_does_not_directly_change_computed_focus_z_depth() {
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.focus_distance_m = 8.0;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 8.0 };
        let mut rig = resolve_camera_rig_2d(
            &camera,
            None,
            amigo_2d_spatial::DepthSpace2d::default(),
            CameraQualityProfile2d::default(),
        );
        let base_focus_z_depth = rig.aperture.computed_focus_z_depth;

        apply_camera_depth_motion_to_rig(
            &mut rig,
            CameraSway2d {
                z_offset: 0.10,
                affects_focus: true,
                ..Default::default()
            }
            .depth_motion(),
        );

        assert_eq!(rig.aperture.computed_focus_z_depth, base_focus_z_depth);
    }

    #[test]
    fn frame_post_fx_stack_maps_distance_focus_to_computed_depth() {
        let service = CameraService::default();
        let mut camera = camera_state_with_rain_profile(None);
        camera.aperture.enabled = true;
        camera.aperture.focus_distance_m = 6.0;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 6.0 };
        service.upsert_2d(camera);

        let stacks = service.frame_post_fx_stacks(None);
        let focus = stacks
            .iter()
            .flat_map(|stack| &stack.effects)
            .find_map(|instance| instance.effect.as_focus_blur().map(|effect| &effect.focus))
            .expect("focus blur should be present");

        let expected =
            amigo_2d_spatial::distance_to_z_depth(6.0, amigo_2d_spatial::DepthSpace2d::default());
        assert!(matches!(
            focus,
            FocusTarget2d::Depth { value } if (*value - expected).abs() < 0.0001
        ));
    }

    #[test]
    fn rain_glass_focus_uses_resolved_rig_depth_space() {
        let mut camera =
            camera_state_with_rain_profile(Some("rotten-club/camera/rain/test-rain".to_owned()));
        camera.aperture.enabled = true;
        camera.aperture.focus_distance_m = 75.0;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 75.0 };
        let custom_space = amigo_2d_spatial::DepthSpace2d {
            near_m: 1.0,
            far_m: 1500.0,
            curve: amigo_2d_spatial::DepthCurve2d::Logarithmic,
        };
        let rig = crate::runtime::rig::resolve_camera_rig_2d(
            &camera,
            None,
            custom_space,
            CameraQualityProfile2d::Gameplay,
        );
        let mut rain = RainGlass2d {
            enabled: true,
            opacity: 0.5,
            spawn_limit: 8,
            z_depth: Some(0.5),
            ..RainGlass2d::default()
        };
        apply_camera_focus_to_rain_glass(&mut rain, &rig);

        let expected = amigo_2d_spatial::distance_to_z_depth(75.0, custom_space);
        assert!(rain.camera_focus_enabled);
        assert!((rain.camera_focus_depth - expected).abs() < 0.0001);
    }

    #[test]
    fn preview_quality_reduces_dof_samples_without_changing_style() {
        let service = CameraService::default();
        let mut camera =
            camera_state_with_rain_profile(Some("rotten-club/camera/rain/test-rain".to_owned()));
        camera.aperture.enabled = true;
        camera.aperture.depth_of_field.sample_count = 64;
        camera.aperture.depth_of_field.highlight_gain = 1.45;
        camera.aperture.focus = CameraFocus2d::Distance { meters: 6.0 };
        let camera_id = camera.id.clone();
        let lens_profile = camera.lens.profile.clone();
        let film_profile = camera.film.profile.clone();
        service.upsert_2d(camera);

        assert!(service.set_quality_profile_2d(&camera_id, CameraQualityProfile2d::Preview));

        let assets = test_asset_catalog_with_rain_profile();
        let stacks = service.frame_post_fx_stacks(Some(&assets));
        let focus_blur = stacks
            .iter()
            .flat_map(|stack| &stack.effects)
            .find_map(|instance| instance.effect.as_focus_blur())
            .expect("focus blur should be present");
        let rain = find_first_rain_glass(&stacks).expect("rain glass should be present");

        assert_eq!(focus_blur.sample_count, 32);
        assert!((focus_blur.highlight_gain - 1.16).abs() < 0.0001);
        assert!(rain.quality_scale < 1.0);
        let camera = service.get_2d(&camera_id).expect("camera should exist");
        assert_eq!(camera.lens.profile, lens_profile);
        assert_eq!(camera.film.profile, film_profile);
    }

    #[test]
    fn debug_quality_keeps_rain_buffers_full_resolution() {
        let service = CameraService::default();
        let mut camera =
            camera_state_with_rain_profile(Some("rotten-club/camera/rain/test-rain".to_owned()));
        camera.aperture.enabled = true;
        let camera_id = camera.id.clone();
        service.upsert_2d(camera);

        assert!(service.set_quality_profile_2d(&camera_id, CameraQualityProfile2d::Debug));

        let assets = test_asset_catalog_with_rain_profile();
        let stacks = service.frame_post_fx_stacks(Some(&assets));
        let rain = find_first_rain_glass(&stacks).expect("rain glass should be present");

        assert_eq!(rain.quality_scale, 1.0);
    }

    #[test]
    fn builtin_camera_presets_keep_expected_feature_separation() {
        let service = CameraService::default();
        service.upsert_2d(camera_state_with_rain_profile(None));

        assert!(service.apply_builtin_preset_2d(&CameraId::new("main"), "default"));
        let default_camera = service
            .main_camera2d()
            .expect("default camera should resolve");
        assert_eq!(default_camera.look.profile, "");
        assert_eq!(
            default_camera.lens_surface.rain_profile.as_deref(),
            Some("realistic_lens_rain")
        );
        assert!((default_camera.film.intensity - 0.42).abs() < f32::EPSILON);

        assert!(service.apply_builtin_preset_2d(&CameraId::new("main"), "cctv"));
        let cctv = service.main_camera2d().expect("cctv camera should resolve");
        assert!(cctv.shutter.enabled);
        assert!((cctv.shutter.fps - 12.0).abs() < f32::EPSILON);

        assert!(service.apply_builtin_preset_2d(&CameraId::new("main"), "anamorphic_rain"));
        let anamorphic = service
            .main_camera2d()
            .expect("anamorphic camera should resolve");
        assert_eq!(
            anamorphic.lens_surface.rain_profile.as_deref(),
            Some("thin_neon_drizzle")
        );

        assert!(service.apply_builtin_preset_2d(&CameraId::new("main"), "noir"));
        let noir = service.main_camera2d().expect("noir camera should resolve");
        assert_eq!(noir.film.profile, "noir_mono_soft");
        assert_eq!(noir.look.profile, "rotten_noir_print");
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
