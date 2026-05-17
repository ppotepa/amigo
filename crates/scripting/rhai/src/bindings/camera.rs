use std::sync::Arc;

use amigo_2d_post_fx::RainGlassPatch;
use amigo_assets::AssetCatalog;
use amigo_camera::{CameraFocus2d, CameraId, CameraService};

#[derive(Clone)]
pub struct CameraApi {
    pub(crate) camera_service: Option<Arc<CameraService>>,
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

    fn update_camera_2d<F>(&mut self, camera_id: &str, update: F) -> bool
    where
        F: FnOnce(&mut amigo_camera::Camera2dRuntimeState) -> bool,
    {
        let Some(service) = self.camera_service.as_ref() else {
            return false;
        };

        let camera_id = camera_id.trim();
        if camera_id.is_empty() {
            return false;
        }

        service.update_camera_2d(&CameraId::new(camera_id), update)
    }
}

fn finite_clamped(value: rhai::FLOAT, min: f32, max: f32) -> Option<f32> {
    let value = value as f32;
    value.is_finite().then(|| value.clamp(min, max))
}
