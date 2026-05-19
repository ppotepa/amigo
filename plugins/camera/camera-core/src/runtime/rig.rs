use amigo_composite_plugin::{ColorRamp2d, RainGlass2d};
use amigo_2d_spatial::{DepthSpace2d, distance_to_z_depth};
use amigo_assets::AssetCatalog;
use crate::api::CameraDepthMotion2d;

use crate::CameraId;
use amigo_camera_optics_plugin::runtime::{
    Camera2dRuntimeState, CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d,
    CameraExposureMode2d, CameraFilm2d, CameraFocus2d, CameraLensSurface2d, CameraShutter2d,
};
use amigo_camera_profiles_plugin::runtime::{FilmStockProfile2d, LensProfile2d};
use amigo_camera_profiles_plugin::api::{CameraQualityProfile2d, CameraQualitySettings2d};

/// Fully resolved 2D camera rig used by render/camera-owned post-fx construction.
/// Do not build camera-owned effects from raw Camera2dRuntimeState when a rig is available.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCameraRig2d {
    pub camera_id: CameraId,
    pub entity_name: String,
    pub depth_space: DepthSpace2d,
    pub render_contributions: amigo_render_api::RenderContributionSet,
    pub exposure: ResolvedExposure2d,
    pub shutter: ResolvedShutter2d,
    pub lens: ResolvedLens2d,
    pub aperture: ResolvedAperture2d,
    pub lens_surface: ResolvedLensSurface2d,
    pub film: ResolvedFilm2d,
    pub look: ResolvedLook2d,
    pub quality: CameraQualityProfile2d,
    pub quality_settings: CameraQualitySettings2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedExposure2d {
    pub mode: CameraExposureMode2d,
    pub iso: f32,
    pub compensation: f32,
    pub white_balance: f32,
    pub nd_stops: f32,
    pub auto: CameraAutoExposure2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedShutter2d {
    pub state: CameraShutter2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLens2d {
    pub profile: LensProfile2d,
    pub intensity: f32,
    pub focal_length_mm: f32,
    pub anamorphic_squeeze: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedAperture2d {
    pub state: CameraAperture2d,
    pub focus: CameraFocus2d,
    pub focus_distance_m: f32,
    pub base_focus_distance_m: Option<f32>,
    pub effective_focus_distance_m: Option<f32>,
    pub camera_z_m: f32,
    pub focus_residual_m: f32,
    pub computed_focus_z_depth: Option<f32>,
    pub depth_of_field: CameraDepthOfField2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLensSurface2d {
    pub state: CameraLensSurface2d,
    pub rain: Option<RainGlass2d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFilm2d {
    pub state: CameraFilm2d,
    pub stock: FilmStockProfile2d,
    pub intensity: f32,
    pub seed: u32,
    pub iso_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLook2d {
    pub profile: Option<ColorRamp2d>,
    pub intensity: f32,
}

pub fn resolve_camera_rig_2d(
    camera: &Camera2dRuntimeState,
    assets: Option<&AssetCatalog>,
    depth_space: DepthSpace2d,
    quality: CameraQualityProfile2d,
) -> ResolvedCameraRig2d {
    let depth_space = depth_space.normalized();
    let lens_profile = camera.resolved_lens_profile(assets);
    let film_stock = camera.resolved_film_stock(assets);
    let iso_factor = (camera.exposure.iso / film_stock.base_iso)
        .sqrt()
        .clamp(0.35, 3.0);
    let focus = camera.aperture.focus.clone();
    let base_focus_distance_m = match focus {
        CameraFocus2d::Distance { meters } => Some(meters),
        _ => None,
    };
    let computed_focus_z_depth = match focus {
        CameraFocus2d::Distance { meters } => Some(distance_to_z_depth(meters, depth_space)),
        CameraFocus2d::Depth { value } => Some(value.clamp(0.0, 1.0)),
        _ => None,
    };

    ResolvedCameraRig2d {
        camera_id: camera.id.clone(),
        entity_name: camera.entity_name.clone(),
        depth_space,
        render_contributions: camera.render_contributions.clone(),
        exposure: ResolvedExposure2d {
            mode: camera.mode,
            iso: camera.exposure.iso,
            compensation: camera.exposure.compensation,
            white_balance: camera.exposure.white_balance,
            nd_stops: camera.exposure.nd_stops,
            auto: camera.exposure.auto.clone(),
        },
        shutter: ResolvedShutter2d {
            state: camera.shutter.clone(),
        },
        lens: ResolvedLens2d {
            focal_length_mm: lens_profile.focal_length_mm,
            anamorphic_squeeze: lens_profile.anamorphic_squeeze,
            profile: lens_profile,
            intensity: camera.lens.intensity,
        },
        aperture: ResolvedAperture2d {
            state: camera.aperture.clone(),
            focus,
            focus_distance_m: camera.aperture.focus_distance_m,
            base_focus_distance_m,
            effective_focus_distance_m: base_focus_distance_m,
            camera_z_m: 0.0,
            focus_residual_m: 0.0,
            computed_focus_z_depth,
            depth_of_field: camera.aperture.depth_of_field.clone(),
        },
        lens_surface: ResolvedLensSurface2d {
            state: camera.lens_surface.clone(),
            rain: camera.resolved_rain_profile(assets),
        },
        film: ResolvedFilm2d {
            state: camera.film.clone(),
            stock: film_stock,
            intensity: camera.film.intensity,
            seed: camera.film.seed,
            iso_factor,
        },
        look: ResolvedLook2d {
            profile: camera.resolved_look_profile(assets),
            intensity: camera.look.intensity,
        },
        quality,
        quality_settings: quality.settings(),
    }
}

pub fn effective_distance_after_camera_z_m(distance_m: f32, camera_z_m: f32) -> f32 {
    if !distance_m.is_finite() {
        return 1.0;
    }
    (distance_m - finite_or_zero(camera_z_m)).max(0.05)
}

pub fn apply_camera_depth_motion_to_rig(
    rig: &mut ResolvedCameraRig2d,
    motion: CameraDepthMotion2d,
) {
    let motion = motion.normalized();
    rig.aperture.camera_z_m = motion.camera_z_m;
    rig.aperture.focus_residual_m = motion.focus_residual_m;

    match &mut rig.aperture.focus {
        CameraFocus2d::Distance { meters } => {
            let base = *meters;
            let effective = (effective_distance_after_camera_z_m(base, motion.camera_z_m)
                + motion.focus_residual_m)
                .max(0.05);
            rig.aperture.base_focus_distance_m = Some(base);
            rig.aperture.effective_focus_distance_m = Some(effective);
            rig.aperture.computed_focus_z_depth =
                Some(distance_to_z_depth(effective, rig.depth_space));
        }
        CameraFocus2d::Depth { value } => {
            rig.aperture.base_focus_distance_m = None;
            rig.aperture.effective_focus_distance_m = None;
            rig.aperture.computed_focus_z_depth = Some(value.clamp(0.0, 1.0));
        }
        _ => {}
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_camera_optics_plugin::runtime::{
        CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d, CameraExposure2d,
        CameraFilm2d, CameraLens2d, CameraLook2d,
    };

    #[test]
    fn resolved_camera_rig_computes_focus_z_depth_from_distance() {
        let camera = Camera2dRuntimeState {
            id: CameraId::new("main"),
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
                intensity: 1.0,
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
            lens_surface: CameraLensSurface2d { rain_profile: None },
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
                profile: String::new(),
                intensity: 0.0,
            },
            aperture: CameraAperture2d {
                enabled: true,
                f_stop: 2.8,
                focus_distance_m: 6.0,
                focus: CameraFocus2d::Distance { meters: 6.0 },
                depth_of_field: CameraDepthOfField2d {
                    depth_map: None,
                    affected_layers: Vec::new(),
                    max_blur_px: 24.0,
                    depth_contrast: 1.0,
                    focus_width: 0.052,
                    foreground_blur_boost: 1.0,
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
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([(
                amigo_render_api::render_contribution_roles::CAMERA_PROJECTION,
                true,
            )]),
        }
        .normalized();

        let depth_space = DepthSpace2d::default();
        let rig =
            resolve_camera_rig_2d(&camera, None, depth_space, CameraQualityProfile2d::Gameplay);
        let expected = distance_to_z_depth(6.0, depth_space);

        assert!(matches!(
            rig.aperture.computed_focus_z_depth,
            Some(value) if (value - expected).abs() < 0.0001
        ));
        assert_eq!(rig.aperture.base_focus_distance_m, Some(6.0));
        assert_eq!(rig.aperture.effective_focus_distance_m, Some(6.0));
    }

}
