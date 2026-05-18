use std::sync::Arc;

use amigo_2d_post_fx::RainGlassPatch;
use amigo_assets::AssetCatalog;
use amigo_camera::{
    CameraDebugView2d, CameraFocus2d, CameraFocusTarget2dService, CameraId,
    CameraQualityProfile2d, CameraService,
};

#[derive(Clone)]
pub struct CameraApi {
    pub(crate) camera_service: Option<Arc<CameraService>>,
    pub(crate) focus_targets_2d: Option<Arc<CameraFocusTarget2dService>>,
    pub(crate) asset_catalog: Option<Arc<AssetCatalog>>,
}

impl CameraApi {
    pub fn set_main_lens_rain(&mut self, updates: &str) -> bool {
        self.set_lens_rain("main", updates)
    }

    pub fn set_lens_rain(&mut self, camera_id: &str, updates: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }

        let id = CameraId::new(camera_id);
        let assets = self.asset_catalog.as_deref();

        let mut applied = false;
        let changed = service.update_lens_rain_2d(&id, assets, |rain| {
            applied = RainGlassPatch::apply_update_string(rain, updates);
        });

        changed && applied
    }

    pub fn set_main_lens_rain_profile(&mut self, profile: &str) -> bool {
        self.set_lens_rain_profile("main", profile)
    }

    pub fn set_lens_rain_profile(&mut self, camera_id: &str, profile: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let camera_id = camera_id.trim();
        let profile = profile.trim();

        if camera_id.is_empty() || profile.is_empty() {
            return false;
        }

        service.set_lens_rain_profile_2d(&CameraId::new(camera_id), profile.to_owned())
    }

    pub fn clear_main_lens_rain_override(&mut self) -> bool {
        self.clear_lens_rain_override("main")
    }

    pub fn clear_lens_rain_override(&mut self, camera_id: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }

        service.clear_lens_rain_override_2d(&CameraId::new(camera_id))
    }

    pub fn set_main_focal_length_mm(&mut self, value: rhai::FLOAT) -> bool {
        self.set_focal_length_mm("main", value)
    }

    pub fn set_main_preset(&mut self, preset: &str) -> bool {
        self.set_preset("main", preset)
    }

    pub fn set_preset(&mut self, camera_id: &str, preset: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let camera_id = camera_id.trim();
        let preset = preset.trim();
        if camera_id.is_empty() || preset.is_empty() {
            return false;
        }

        service.apply_builtin_preset_2d(&CameraId::new(camera_id), preset)
    }

    pub fn set_focal_length_mm(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 8.0, 300.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.lens.focal_length_mm = Some(value);
            true
        })
    }

    pub fn set_main_aperture_f_stop(&mut self, value: rhai::FLOAT) -> bool {
        self.set_aperture_f_stop("main", value)
    }

    pub fn set_aperture_f_stop(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 0.7, 32.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.aperture.enabled = true;
            camera.aperture.f_stop = value;
            true
        })
    }

    pub fn set_main_focus_depth(&mut self, value: rhai::FLOAT) -> bool {
        self.set_focus_depth("main", value)
    }

    pub fn set_main_focus_distance_m(&mut self, value: rhai::FLOAT) -> bool {
        self.set_focus_distance_m("main", value)
    }

    pub fn focus_main(&mut self, selector: &str) -> bool {
        self.focus_main_over(selector, 0.0)
    }

    pub fn focus_main_over(&mut self, selector: &str, seconds: rhai::FLOAT) -> bool {
        self.focus_over("main", selector, seconds)
    }

    pub fn focus_over(&mut self, camera_id: &str, selector: &str, seconds: rhai::FLOAT) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let Some(targets) = self.focus_targets_2d.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        let selector = selector.trim();
        if camera_id.is_empty() || selector.is_empty() || !seconds.is_finite() {
            return false;
        }
        service.focus_2d_on_target(
            &CameraId::new(camera_id),
            selector,
            targets,
            seconds.max(0.0) as f32,
        )
    }

    pub fn has_focus_target(&mut self, selector: &str) -> bool {
        self.focus_targets_2d
            .as_ref()
            .is_some_and(|targets| targets.has(selector))
    }

    pub fn focus_target_summary(&mut self) -> String {
        self.focus_targets_2d
            .as_ref()
            .map(|targets| targets.summary())
            .unwrap_or_else(|| "camera.focus.targets: service unavailable".to_owned())
    }

    pub fn set_focus_distance_m(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 0.2, 1000.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.aperture.enabled = true;
            camera.aperture.focus_distance_m = value;
            camera.aperture.focus = CameraFocus2d::Distance { meters: value };
            true
        })
    }

    pub fn set_focus_depth(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 0.0, 1.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.aperture.enabled = true;
            camera.aperture.focus = CameraFocus2d::Depth { value };
            true
        })
    }

    pub fn set_main_sway_amounts(
        &mut self,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        zoom: rhai::FLOAT,
        rotation: rhai::FLOAT,
    ) -> bool {
        self.set_sway_amounts("main", x, y, z, zoom, rotation)
    }

    pub fn set_sway_amounts(
        &mut self,
        camera_id: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        z: rhai::FLOAT,
        zoom: rhai::FLOAT,
        rotation: rhai::FLOAT,
    ) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_sway_amounts_2d(
            &CameraId::new(camera_id),
            x as f32,
            y as f32,
            z as f32,
            zoom as f32,
            rotation as f32,
        )
    }

    pub fn set_main_sway_frequency(&mut self, frequency: rhai::FLOAT) -> bool {
        self.set_sway_frequency("main", frequency)
    }

    pub fn set_sway_frequency(&mut self, camera_id: &str, frequency: rhai::FLOAT) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_sway_frequency_2d(&CameraId::new(camera_id), frequency as f32)
    }

    pub fn set_main_sway_z_offset(&mut self, z_offset: rhai::FLOAT) -> bool {
        self.set_sway_z_offset("main", z_offset)
    }

    pub fn set_main_camera_z_m(&mut self, value: rhai::FLOAT) -> bool {
        self.set_camera_z_m("main", value)
    }

    pub fn set_camera_z_m(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, -50.0, 50.0) else {
            return false;
        };
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let Some(camera_id) = resolve_camera_id(service, camera_id) else {
            return false;
        };
        service.set_camera_z_m_2d(&camera_id, value)
    }

    pub fn set_main_focus_residual_m(&mut self, value: rhai::FLOAT) -> bool {
        self.set_focus_residual_m("main", value)
    }

    pub fn set_focus_residual_m(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, -5.0, 5.0) else {
            return false;
        };
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let Some(camera_id) = resolve_camera_id(service, camera_id) else {
            return false;
        };
        service.set_focus_residual_m_2d(&camera_id, value)
    }

    pub fn set_main_dolly_signal(&mut self, value: rhai::FLOAT) -> bool {
        self.set_dolly_signal("main", value)
    }

    pub fn set_dolly_signal(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, -1.0, 1.0) else {
            return false;
        };
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let Some(camera_id) = resolve_camera_id(service, camera_id) else {
            return false;
        };
        service.set_dolly_signal_2d(&camera_id, value)
    }

    pub fn set_main_shutter_speed_s(&mut self, value: rhai::FLOAT) -> bool {
        self.set_shutter_speed_s("main", value)
    }

    pub fn set_shutter_speed_s(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 1.0 / 8000.0, 2.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.shutter.speed_s = Some(value);
            true
        })
    }

    pub fn set_main_shutter_fraction(&mut self, denominator: rhai::FLOAT) -> bool {
        let denominator = denominator as f32;
        if !denominator.is_finite() || denominator <= 0.0 {
            return false;
        }
        self.set_main_shutter_speed_s((1.0 / denominator) as rhai::FLOAT)
    }

    pub fn set_main_shutter_enabled(&mut self, enabled: bool) -> bool {
        self.set_shutter_enabled("main", enabled)
    }

    pub fn set_shutter_enabled(&mut self, camera_id: &str, enabled: bool) -> bool {
        self.update_camera_2d(camera_id, |camera| {
            camera.shutter.enabled = enabled;
            true
        })
    }

    pub fn set_main_shutter_opacity(&mut self, value: rhai::FLOAT) -> bool {
        self.set_shutter_opacity("main", value)
    }

    pub fn set_shutter_opacity(&mut self, camera_id: &str, value: rhai::FLOAT) -> bool {
        let Some(value) = finite_clamped(value, 0.0, 1.0) else {
            return false;
        };

        self.update_camera_2d(camera_id, |camera| {
            camera.shutter.opacity = value;
            true
        })
    }

    pub fn set_sway_z_offset(&mut self, camera_id: &str, z_offset: rhai::FLOAT) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_sway_z_offset_2d(&CameraId::new(camera_id), z_offset as f32)
    }

    pub fn set_main_sway_affects_focus(&mut self, affects_focus: bool) -> bool {
        self.set_sway_affects_focus("main", affects_focus)
    }

    pub fn set_sway_affects_focus(&mut self, camera_id: &str, affects_focus: bool) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_sway_affects_focus_2d(&CameraId::new(camera_id), affects_focus)
    }

    pub fn clear_main_sway(&mut self) -> bool {
        self.clear_sway("main")
    }

    pub fn clear_sway(&mut self, camera_id: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.clear_sway_2d(&CameraId::new(camera_id))
    }

    pub fn set_main_quality(&mut self, profile: &str) -> bool {
        self.set_quality("main", profile)
    }

    pub fn set_quality(&mut self, camera_id: &str, profile: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_quality_profile_2d(
            &CameraId::new(camera_id),
            CameraQualityProfile2d::parse(profile),
        )
    }

    pub fn set_main_debug_view(&mut self, view: &str) -> bool {
        self.set_debug_view("main", view)
    }

    pub fn set_debug_view(&mut self, camera_id: &str, view: &str) -> bool {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };
        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }
        service.set_debug_view_2d(&CameraId::new(camera_id), CameraDebugView2d::parse(view))
    }

    fn update_camera_2d<F>(&mut self, camera_id: &str, update: F) -> bool
    where
        F: FnOnce(&mut amigo_camera::Camera2dRuntimeState) -> bool,
    {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let Some(camera_id) = resolve_camera_id(service, camera_id) else {
            return false;
        };

        service.update_camera_2d(&camera_id, update)
    }
}

fn finite_clamped(value: rhai::FLOAT, min: f32, max: f32) -> Option<f32> {
    let value = value as f32;
    value.is_finite().then(|| value.clamp(min, max))
}

fn resolve_camera_id(service: &CameraService, camera_id: &str) -> Option<CameraId> {
    let camera_id = camera_id.trim();
    if camera_id.is_empty() {
        return None;
    }
    if camera_id == "main" {
        return service.main_camera2d_id();
    }
    Some(CameraId::new(camera_id))
}
