use amigo_composite_plugin::{ColorRamp2d, RainGlass2d};
use amigo_assets::AssetCatalog;
use amigo_plugin_api::{roles, RenderContributionSet};

use amigo_camera_profiles_plugin::runtime::{
    film_stock_2d, film_stock_2d_from_catalog, lens_profile_2d, lens_profile_2d_from_catalog,
    look_profile_2d_from_catalog, rain_glass_profile_2d_from_catalog,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CameraId(pub String);

impl CameraId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn main() -> Self {
        Self::new("main")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraExposureMode2d {
    Auto,
    Manual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera2dRuntimeState {
    pub id: CameraId,
    pub entity_name: String,
    pub mode: CameraExposureMode2d,
    pub exposure: CameraExposure2d,
    pub shutter: CameraShutter2d,
    pub lens: CameraLens2d,
    pub lens_surface: CameraLensSurface2d,
    pub film: CameraFilm2d,
    pub look: CameraLook2d,
    pub aperture: CameraAperture2d,
    pub render_contributions: RenderContributionSet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraExposure2d {
    pub iso: f32,
    pub compensation: f32,
    pub white_balance: f32,
    pub nd_stops: f32,
    pub auto: CameraAutoExposure2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraAutoExposure2d {
    pub target_luma: f32,
    pub adaptation_speed: f32,
    pub min_iso: f32,
    pub max_iso: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraShutter2d {
    pub enabled: bool,
    pub speed_s: Option<f32>,
    pub fps: f32,
    pub angle: f32,
    pub opacity: f32,
    pub history_mix: f32,
    pub history_mix_2: f32,
    pub edge_rejection: f32,
    pub luma_threshold: f32,
    pub frame_hold: bool,
}

impl CameraShutter2d {
    pub fn exposure_seconds(&self) -> f32 {
        if let Some(speed_s) = self.speed_s {
            return speed_s.clamp(1.0 / 8000.0, 2.0);
        }

        let fps = self.fps.max(1.0);
        let angle_fraction = (self.angle / 360.0).clamp(0.0, 1.0);
        (angle_fraction / fps).clamp(1.0 / 8000.0, 2.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLens2d {
    pub profile: String,
    pub intensity: f32,
    pub aberration_px: Option<f32>,
    pub distortion: Option<f32>,
    pub vignette: Option<f32>,
    pub edge_softness_px: Option<f32>,
    pub glare_strength: Option<f32>,
    pub dirt: Option<f32>,
    pub focal_length_mm: Option<f32>,
    pub lens_bloom: Option<f32>,
    pub flare_ghosts: Option<f32>,
    pub anamorphic_squeeze: Option<f32>,
    pub coma: Option<f32>,
    pub cat_eye_bokeh: Option<f32>,
    pub focus_breathing: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLensSurface2d {
    pub rain_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFilm2d {
    pub profile: String,
    pub intensity: f32,
    pub seed: u32,
    pub color_shift: Option<f32>,
    pub contrast: Option<f32>,
    pub saturation: Option<f32>,
    pub flicker: Option<f32>,
    pub vignette: Option<f32>,
    pub toe: Option<f32>,
    pub shoulder: Option<f32>,
    pub black_lift: Option<f32>,
    pub print_fade: Option<f32>,
    pub dust: Option<f32>,
    pub scratches: Option<f32>,
    pub push_pull: Option<f32>,
    pub gate_weave: Option<f32>,
    pub scan_softness: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLook2d {
    pub profile: String,
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraAperture2d {
    pub enabled: bool,
    pub f_stop: f32,
    pub focus_distance_m: f32,
    pub focus: CameraFocus2d,
    pub depth_of_field: CameraDepthOfField2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDepthOfField2d {
    pub depth_map: Option<String>,
    pub affected_layers: Vec<String>,
    pub max_blur_px: f32,
    pub depth_contrast: f32,
    pub focus_width: f32,
    pub foreground_blur_boost: f32,
    pub background_blur_boost: f32,
    pub edge_aware: bool,
    pub invert_depth: bool,
    pub debug_view: String,
    pub aperture_blades: u32,
    pub aperture_roundness: f32,
    pub aperture_rotation_degrees: f32,
    pub sample_count: u32,
    pub highlight_threshold: f32,
    pub highlight_knee: f32,
    pub highlight_gain: f32,
    pub highlight_saturation: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraFocus2d {
    None,
    RenderLayer { layer: String },
    SceneObject { object: String },
    Distance { meters: f32 },
    Depth { value: f32 },
}

impl Camera2dRuntimeState {
    pub fn normalized(mut self) -> Self {
        self.exposure.iso = self.exposure.iso.clamp(25.0, 12800.0);
        self.exposure.compensation = self.exposure.compensation.clamp(-8.0, 8.0);
        self.exposure.white_balance = self.exposure.white_balance.clamp(1800.0, 12000.0);
        self.exposure.nd_stops = self.exposure.nd_stops.clamp(0.0, 16.0);

        self.exposure.auto.target_luma = self.exposure.auto.target_luma.clamp(0.01, 1.0);
        self.exposure.auto.adaptation_speed = self.exposure.auto.adaptation_speed.clamp(0.0, 20.0);
        self.exposure.auto.min_iso = self.exposure.auto.min_iso.clamp(25.0, 12800.0);
        self.exposure.auto.max_iso = self
            .exposure
            .auto
            .max_iso
            .clamp(self.exposure.auto.min_iso, 12800.0);

        self.shutter.fps = self.shutter.fps.clamp(1.0, 240.0);
        self.shutter.speed_s = self
            .shutter
            .speed_s
            .map(|speed_s| speed_s.clamp(1.0 / 8000.0, 2.0));
        self.shutter.angle = self.shutter.angle.clamp(0.0, 360.0);
        self.shutter.opacity = self.shutter.opacity.clamp(0.0, 1.0);
        self.shutter.history_mix = self.shutter.history_mix.clamp(0.0, 0.98);
        self.shutter.history_mix_2 = self.shutter.history_mix_2.clamp(0.0, 0.98);
        self.shutter.edge_rejection = self.shutter.edge_rejection.clamp(0.0, 1.0);
        self.shutter.luma_threshold = self.shutter.luma_threshold.clamp(0.0, 1.0);

        self.lens.intensity = self.lens.intensity.clamp(0.0, 1.0);
        self.lens.anamorphic_squeeze = self
            .lens
            .anamorphic_squeeze
            .map(|value| value.clamp(1.0, 2.5));
        self.film.intensity = self.film.intensity.clamp(0.0, 1.0);
        self.look.intensity = self.look.intensity.clamp(0.0, 1.0);

        self.aperture.f_stop = self.aperture.f_stop.clamp(0.7, 32.0);
        self.aperture.focus_distance_m = self.aperture.focus_distance_m.clamp(0.2, 1000.0);
        match &mut self.aperture.focus {
            CameraFocus2d::Distance { meters } => {
                *meters = if meters.is_finite() {
                    meters.clamp(0.2, 1000.0)
                } else {
                    self.aperture.focus_distance_m
                };
                self.aperture.focus_distance_m = *meters;
            }
            CameraFocus2d::Depth { value } => {
                *value = value.clamp(0.0, 1.0);
            }
            _ => {}
        }
        self.aperture.depth_of_field.max_blur_px =
            self.aperture.depth_of_field.max_blur_px.clamp(0.0, 90.0);
        self.aperture.depth_of_field.depth_contrast =
            self.aperture.depth_of_field.depth_contrast.clamp(0.4, 2.4);
        self.aperture.depth_of_field.focus_width =
            self.aperture.depth_of_field.focus_width.clamp(0.005, 0.22);
        self.aperture.depth_of_field.foreground_blur_boost = self
            .aperture
            .depth_of_field
            .foreground_blur_boost
            .clamp(0.25, 2.5);
        self.aperture.depth_of_field.background_blur_boost = self
            .aperture
            .depth_of_field
            .background_blur_boost
            .clamp(0.25, 2.5);
        self.aperture.depth_of_field.affected_layers = normalized_layer_list(std::mem::take(
            &mut self.aperture.depth_of_field.affected_layers,
        ));
        self.aperture.depth_of_field.aperture_blades =
            self.aperture.depth_of_field.aperture_blades.clamp(0, 12);
        if self.aperture.depth_of_field.aperture_blades > 0
            && self.aperture.depth_of_field.aperture_blades < 3
        {
            self.aperture.depth_of_field.aperture_blades = 3;
        }
        self.aperture.depth_of_field.aperture_roundness = self
            .aperture
            .depth_of_field
            .aperture_roundness
            .clamp(0.0, 1.0);
        self.aperture.depth_of_field.aperture_rotation_degrees = self
            .aperture
            .depth_of_field
            .aperture_rotation_degrees
            .rem_euclid(360.0);
        self.aperture.depth_of_field.sample_count =
            self.aperture.depth_of_field.sample_count.clamp(12, 96);
        self.aperture.depth_of_field.highlight_threshold = self
            .aperture
            .depth_of_field
            .highlight_threshold
            .clamp(0.0, 4.0);
        self.aperture.depth_of_field.highlight_knee = self
            .aperture
            .depth_of_field
            .highlight_knee
            .clamp(0.001, 2.0);
        self.aperture.depth_of_field.highlight_gain =
            self.aperture.depth_of_field.highlight_gain.clamp(0.0, 8.0);
        self.aperture.depth_of_field.highlight_saturation = self
            .aperture
            .depth_of_field
            .highlight_saturation
            .clamp(0.0, 3.0);
        self.render_contributions
            .merge_defaults([(roles::CAMERA_PROJECTION, true)]);

        self
    }

    pub fn resolved_lens_profile(
        &self,
        assets: Option<&AssetCatalog>,
    ) -> amigo_camera_profiles_plugin::runtime::LensProfile2d {
        let mut profile = assets
            .and_then(|assets| lens_profile_2d_from_catalog(assets, &self.lens.profile))
            .or_else(|| lens_profile_2d(&self.lens.profile))
            .unwrap_or_else(|| lens_profile_2d("clean_modern_35mm").unwrap());

        if let Some(value) = self.lens.aberration_px {
            profile.aberration_px = value;
        }
        if let Some(value) = self.lens.distortion {
            profile.distortion = value;
        }
        if let Some(value) = self.lens.vignette {
            profile.vignette = value;
        }
        if let Some(value) = self.lens.edge_softness_px {
            profile.edge_softness_px = value;
        }
        if let Some(value) = self.lens.glare_strength {
            profile.glare_strength = value;
        }
        if let Some(value) = self.lens.dirt {
            profile.dirt = value;
        }
        if let Some(value) = self.lens.focal_length_mm {
            profile.focal_length_mm = value;
        }
        if let Some(value) = self.lens.lens_bloom {
            profile.lens_bloom = value;
        }
        if let Some(value) = self.lens.flare_ghosts {
            profile.flare_ghosts = value;
        }
        if let Some(value) = self.lens.anamorphic_squeeze {
            profile.anamorphic_squeeze = value;
        }
        if let Some(value) = self.lens.coma {
            profile.coma = value;
        }
        if let Some(value) = self.lens.cat_eye_bokeh {
            profile.cat_eye_bokeh = value;
        }
        if let Some(value) = self.lens.focus_breathing {
            profile.focus_breathing = value;
        }

        profile
    }

    pub fn resolved_film_stock(
        &self,
        assets: Option<&AssetCatalog>,
    ) -> amigo_camera_profiles_plugin::runtime::FilmStockProfile2d {
        let mut profile = assets
            .and_then(|assets| film_stock_2d_from_catalog(assets, &self.film.profile))
            .or_else(|| film_stock_2d(&self.film.profile))
            .unwrap_or_else(|| film_stock_2d("neutral_digital_400").unwrap());

        if let Some(value) = self.film.color_shift {
            profile.color_shift = value;
        }
        if let Some(value) = self.film.contrast {
            profile.contrast = value;
        }
        if let Some(value) = self.film.saturation {
            profile.saturation = value;
        }
        if let Some(value) = self.film.flicker {
            profile.flicker = value;
        }
        if let Some(value) = self.film.vignette {
            profile.vignette = value;
        }
        if let Some(value) = self.film.toe {
            profile.toe = value;
        }
        if let Some(value) = self.film.shoulder {
            profile.shoulder = value;
        }
        if let Some(value) = self.film.black_lift {
            profile.black_lift = value;
        }
        if let Some(value) = self.film.print_fade {
            profile.print_fade = value;
        }
        if let Some(value) = self.film.dust {
            profile.dust = value;
        }
        if let Some(value) = self.film.scratches {
            profile.scratches = value;
        }
        if let Some(value) = self.film.push_pull {
            profile.push_pull = value;
        }
        if let Some(value) = self.film.gate_weave {
            profile.gate_weave = value;
        }
        if let Some(value) = self.film.scan_softness {
            profile.scan_softness = value;
        }

        profile
    }

    pub fn resolved_rain_profile(&self, assets: Option<&AssetCatalog>) -> Option<RainGlass2d> {
        let rain_profile = self.lens_surface.rain_profile.as_deref()?;
        assets.and_then(|assets| rain_glass_profile_2d_from_catalog(assets, rain_profile))
    }

    pub fn resolved_look_profile(&self, assets: Option<&AssetCatalog>) -> Option<ColorRamp2d> {
        let look_profile = self.look.profile.trim();
        if look_profile.is_empty() {
            return None;
        }
        assets.and_then(|assets| look_profile_2d_from_catalog(assets, look_profile))
    }
}

fn normalized_layer_list(layers: Vec<String>) -> Vec<String> {
    let mut layers = layers
        .into_iter()
        .map(|layer| layer.trim().to_owned())
        .filter(|layer| !layer.is_empty())
        .collect::<Vec<_>>();
    layers.sort();
    layers.dedup();
    layers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shutter_with(speed_s: Option<f32>, fps: f32, angle: f32) -> CameraShutter2d {
        CameraShutter2d {
            enabled: true,
            speed_s,
            fps,
            angle,
            opacity: 1.0,
            history_mix: 0.0,
            history_mix_2: 0.0,
            edge_rejection: 0.0,
            luma_threshold: 0.0,
            frame_hold: false,
        }
    }

    #[test]
    fn shutter_speed_seconds_override_previous_angle_model() {
        let shutter = shutter_with(Some(0.1), 24.0, 180.0);

        assert!((shutter.exposure_seconds() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn shutter_angle_fps_remain_previous_exposure_fallback() {
        let shutter = shutter_with(None, 24.0, 180.0);

        assert!((shutter.exposure_seconds() - (1.0 / 48.0)).abs() < 0.0001);
    }
}
