use amigo_assets::{AssetCatalog, AssetKey, PreparedAsset, PreparedAssetKind};
use amigo_composite_plugin::{
    ColorRamp2d, RainGlass2d, RainGlassDebugView, RainGlassRaindropCompose,
};

pub use amigo_film_look_plugin::runtime::FilmGrainProfile2d;

mod catalog;

#[derive(Debug, Clone, PartialEq)]
pub struct LensProfile2d {
    pub id: &'static str,
    pub label: &'static str,
    pub focal_length_mm: f32,
    pub aberration_px: f32,
    pub distortion: f32,
    pub vignette: f32,
    pub edge_softness_px: f32,
    pub glare_strength: f32,
    pub dirt: f32,
    pub halation_bias: f32,
    pub lens_bloom: f32,
    pub flare_ghosts: f32,
    pub anamorphic_squeeze: f32,
    pub coma: f32,
    pub cat_eye_bokeh: f32,
    pub focus_breathing: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilmStockProfile2d {
    pub id: &'static str,
    pub label: &'static str,
    pub base_iso: f32,
    pub color_shift: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub flicker: f32,
    pub vignette: f32,
    pub opacity: f32,
    pub toe: f32,
    pub shoulder: f32,
    pub black_lift: f32,
    pub print_fade: f32,
    pub dust: f32,
    pub scratches: f32,
    pub push_pull: f32,
    pub gate_weave: f32,
    pub scan_softness: f32,
    pub grain: FilmGrainProfile2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPreset2d {
    pub id: &'static str,
    pub label: &'static str,
    pub lens_profile: &'static str,
    pub lens_intensity: f32,
    pub film_profile: &'static str,
    pub film_intensity: f32,
    pub film_seed: u32,
    pub look_profile: &'static str,
    pub look_intensity: f32,
    pub rain_profile: &'static str,
    pub exposure_iso: f32,
    pub exposure_compensation: f32,
    pub shutter_enabled: bool,
    pub shutter_fps: f32,
    pub shutter_angle: f32,
    pub shutter_opacity: f32,
    pub focal_length_mm: f32,
    pub f_stop: f32,
    pub focus_distance_m: f32,
    pub focus_depth: f32,
    pub max_blur_px: f32,
    pub focus_width: f32,
    pub foreground_blur_boost: f32,
    pub background_blur_boost: f32,
    pub aperture_blades: u32,
    pub aperture_roundness: f32,
    pub aperture_rotation_degrees: f32,
    pub sample_count: u32,
    pub highlight_threshold: f32,
    pub highlight_knee: f32,
    pub highlight_gain: f32,
    pub highlight_saturation: f32,
}

pub const BUILTIN_LENS_PROFILES_2D: &[LensProfile2d] = catalog::LENS_PROFILES_2D;
pub const BUILTIN_FILM_STOCKS_2D: &[FilmStockProfile2d] = catalog::FILM_STOCKS_2D;
pub const BUILTIN_CAMERA_PRESETS_2D: &[CameraPreset2d] = catalog::CAMERA_PRESETS_2D;

pub fn lens_profile_2d(id: &str) -> Option<LensProfile2d> {
    BUILTIN_LENS_PROFILES_2D
        .iter()
        .find(|profile| profile.id == id)
        .cloned()
}

pub fn film_stock_2d(id: &str) -> Option<FilmStockProfile2d> {
    BUILTIN_FILM_STOCKS_2D
        .iter()
        .find(|profile| profile.id == id)
        .cloned()
}

pub fn camera_preset_2d(id: &str) -> Option<CameraPreset2d> {
    BUILTIN_CAMERA_PRESETS_2D
        .iter()
        .find(|profile| profile.id == id)
        .copied()
}

pub fn lens_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<LensProfile2d> {
    lens_profile_2d(id_or_key).or_else(|| {
        let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
        lens_profile_2d_from_prepared(&prepared)
    })
}

pub fn film_stock_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<FilmStockProfile2d> {
    film_stock_2d(id_or_key).or_else(|| {
        let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
        film_stock_2d_from_prepared(&prepared)
    })
}

pub fn rain_glass_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<RainGlass2d> {
    let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
    rain_glass_profile_2d_from_prepared(&prepared)
}

pub fn look_profile_2d_from_catalog(
    catalog: &AssetCatalog,
    id_or_key: &str,
) -> Option<ColorRamp2d> {
    let prepared = catalog.prepared_asset(&AssetKey::new(id_or_key))?;
    look_profile_2d_from_prepared(&prepared)
}

pub fn lens_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<LensProfile2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-lens-profile-2d"
    {
        return None;
    }

    let id = metadata_string(prepared, "id")?;
    Some(LensProfile2d {
        id: Box::leak(id.into_boxed_str()),
        label: Box::leak(
            metadata_string(prepared, "label")
                .unwrap_or_else(|| "Custom Lens Profile".to_owned())
                .into_boxed_str(),
        ),
        focal_length_mm: metadata_f32(prepared, "focal_length_mm").unwrap_or(35.0),
        aberration_px: metadata_f32(prepared, "aberration_px").unwrap_or(0.0),
        distortion: metadata_f32(prepared, "distortion").unwrap_or(0.0),
        vignette: metadata_f32(prepared, "vignette").unwrap_or(0.0),
        edge_softness_px: metadata_f32(prepared, "edge_softness_px").unwrap_or(0.0),
        glare_strength: metadata_f32(prepared, "glare_strength").unwrap_or(0.0),
        dirt: metadata_f32(prepared, "dirt").unwrap_or(0.0),
        halation_bias: metadata_f32(prepared, "halation_bias").unwrap_or(0.0),
        lens_bloom: metadata_f32(prepared, "lens_bloom").unwrap_or(0.0),
        flare_ghosts: metadata_f32(prepared, "flare_ghosts").unwrap_or(0.0),
        anamorphic_squeeze: metadata_f32(prepared, "anamorphic_squeeze").unwrap_or(1.0),
        coma: metadata_f32(prepared, "coma").unwrap_or(0.0),
        cat_eye_bokeh: metadata_f32(prepared, "cat_eye_bokeh").unwrap_or(0.0),
        focus_breathing: metadata_f32(prepared, "focus_breathing").unwrap_or(0.0),
    })
}

pub fn film_stock_2d_from_prepared(prepared: &PreparedAsset) -> Option<FilmStockProfile2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-film-stock-2d"
    {
        return None;
    }

    let id = metadata_string(prepared, "id")?;
    Some(FilmStockProfile2d {
        id: Box::leak(id.into_boxed_str()),
        label: Box::leak(
            metadata_string(prepared, "label")
                .unwrap_or_else(|| "Custom Film Stock".to_owned())
                .into_boxed_str(),
        ),
        base_iso: metadata_f32(prepared, "base_iso").unwrap_or(400.0),
        color_shift: metadata_f32(prepared, "color_shift").unwrap_or(0.0),
        contrast: metadata_f32(prepared, "contrast").unwrap_or(1.0),
        saturation: metadata_f32(prepared, "saturation").unwrap_or(1.0),
        flicker: metadata_f32(prepared, "flicker").unwrap_or(0.0),
        vignette: metadata_f32(prepared, "vignette").unwrap_or(0.0),
        opacity: metadata_f32(prepared, "opacity").unwrap_or(0.25),
        toe: metadata_f32(prepared, "toe").unwrap_or(0.45),
        shoulder: metadata_f32(prepared, "shoulder").unwrap_or(0.65),
        black_lift: metadata_f32(prepared, "black_lift").unwrap_or(0.02),
        print_fade: metadata_f32(prepared, "print_fade").unwrap_or(0.08),
        dust: metadata_f32(prepared, "dust").unwrap_or(0.0),
        scratches: metadata_f32(prepared, "scratches").unwrap_or(0.0),
        push_pull: metadata_f32(prepared, "push_pull").unwrap_or(0.0),
        gate_weave: metadata_f32(prepared, "gate_weave").unwrap_or(0.0),
        scan_softness: metadata_f32(prepared, "scan_softness").unwrap_or(0.0),
        grain: film_grain_profile_2d_from_prepared(prepared, "grain"),
    })
}

fn film_grain_profile_2d_from_prepared(
    prepared: &PreparedAsset,
    prefix: &str,
) -> FilmGrainProfile2d {
    let model = metadata_string(prepared, &format!("{prefix}.model"))
        .unwrap_or_else(|| "modern_color_negative".to_owned())
        .to_ascii_lowercase();
    let mut grain = match model.as_str() {
        "clean" | "clean_digital" | "digital" => FilmGrainProfile2d::clean_digital(),
        "fast" | "fast_color" | "fast_color_negative" | "portra_800" | "vision3_500t" => {
            FilmGrainProfile2d::fast_color_negative()
        }
        "bw" | "b&w" | "silver" | "silver_halide" | "bw_silver_pushed" | "tri_x" | "hp5" => {
            FilmGrainProfile2d::bw_silver_pushed()
        }
        "reversal" | "slide" | "ektachrome" | "fine_reversal" => {
            FilmGrainProfile2d::fine_reversal()
        }
        "dirty" | "expired" | "dirty_scan" | "lab_scan" => FilmGrainProfile2d::dirty_scan(),
        _ => FilmGrainProfile2d::modern_color_negative(),
    };

    grain.luma_amount = metadata_f32(prepared, &format!("{prefix}.luma_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.luma")))
        .unwrap_or(grain.luma_amount);
    grain.chroma_amount = metadata_f32(prepared, &format!("{prefix}.chroma_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.chroma")))
        .unwrap_or(grain.chroma_amount);
    grain.shadow_amount = metadata_f32(prepared, &format!("{prefix}.shadow_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.shadows")))
        .unwrap_or(grain.shadow_amount);
    grain.midtone_amount = metadata_f32(prepared, &format!("{prefix}.midtone_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.midtones")))
        .unwrap_or(grain.midtone_amount);
    grain.highlight_amount = metadata_f32(prepared, &format!("{prefix}.highlight_amount"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.highlights")))
        .unwrap_or(grain.highlight_amount);
    grain.highlight_suppression =
        metadata_f32(prepared, &format!("{prefix}.highlight_suppression"))
            .unwrap_or(grain.highlight_suppression);
    grain.fine_grain_px = metadata_f32(prepared, &format!("{prefix}.fine_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.fine_px")))
        .unwrap_or(grain.fine_grain_px);
    grain.medium_grain_px = metadata_f32(prepared, &format!("{prefix}.medium_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.medium_px")))
        .unwrap_or(grain.medium_grain_px);
    grain.coarse_grain_px = metadata_f32(prepared, &format!("{prefix}.coarse_grain_px"))
        .or_else(|| metadata_f32(prepared, &format!("{prefix}.coarse_px")))
        .unwrap_or(grain.coarse_grain_px);
    grain.clumpiness =
        metadata_f32(prepared, &format!("{prefix}.clumpiness")).unwrap_or(grain.clumpiness);
    grain.softness =
        metadata_f32(prepared, &format!("{prefix}.softness")).unwrap_or(grain.softness);
    grain.underexposure_boost = metadata_f32(prepared, &format!("{prefix}.underexposure_boost"))
        .unwrap_or(grain.underexposure_boost);
    grain.push_process_boost = metadata_f32(prepared, &format!("{prefix}.push_process_boost"))
        .unwrap_or(grain.push_process_boost);
    grain.density_pivot =
        metadata_f32(prepared, &format!("{prefix}.density_pivot")).unwrap_or(grain.density_pivot);
    grain.channel_balance[0] =
        metadata_f32(prepared, &format!("{prefix}.channel_r")).unwrap_or(grain.channel_balance[0]);
    grain.channel_balance[1] =
        metadata_f32(prepared, &format!("{prefix}.channel_g")).unwrap_or(grain.channel_balance[1]);
    grain.channel_balance[2] =
        metadata_f32(prepared, &format!("{prefix}.channel_b")).unwrap_or(grain.channel_balance[2]);
    grain.temporal_jitter = metadata_f32(prepared, &format!("{prefix}.temporal_jitter"))
        .unwrap_or(grain.temporal_jitter);
    grain.regenerate_per_frame = metadata_bool(prepared, &format!("{prefix}.regenerate_per_frame"))
        .or_else(|| metadata_bool(prepared, &format!("{prefix}.per_frame")))
        .or_else(|| metadata_bool(prepared, &format!("{prefix}.animated")))
        .unwrap_or(grain.regenerate_per_frame);

    grain
}

pub fn rain_glass_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<RainGlass2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-rain-glass-profile-2d"
    {
        return None;
    }

    let mut rain = RainGlass2d::default();

    if let Some(value) = metadata_bool(prepared, "spawn.enabled") {
        rain.enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "spawn.spawn_rate") {
        rain.spawn_rate = value;
    }
    if let Some(value) = metadata_u32(prepared, "spawn.spawn_limit") {
        rain.spawn_limit = value;
    }
    if let Some(value) = metadata_u32(prepared, "spawn.seed") {
        rain.seed = value;
    }

    if let Some(value) = metadata_f32(prepared, "droplets.min_radius_px") {
        rain.min_radius_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.max_radius_px") {
        rain.max_radius_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.gravity_px_per_sec2") {
        rain.gravity_px_per_sec2 = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.slip_rate") {
        rain.slip_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.initial_spread") {
        rain.initial_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.shrink_rate") {
        rain.shrink_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.evaporate") {
        rain.evaporate = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.velocity_spread") {
        rain.velocity_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.motion_interval_min") {
        rain.motion_interval_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.motion_interval_max") {
        rain.motion_interval_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.x_shift_min") {
        rain.x_shift_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.x_shift_max") {
        rain.x_shift_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "droplets.collider_scale") {
        rain.collider_scale = value;
    }

    if let Some(value) = metadata_bool(prepared, "trails.enabled") {
        rain.trails_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_density") {
        rain.trail_drop_density = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_size_min") {
        rain.trail_drop_size_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_drop_size_max") {
        rain.trail_drop_size_max = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_distance_min_px") {
        rain.trail_distance_min_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_distance_max_px") {
        rain.trail_distance_max_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_spread") {
        rain.trail_spread = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_shrink_rate") {
        rain.trail_shrink_rate = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_evaporate") {
        rain.trail_evaporate = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_taper") {
        rain.trail_taper = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_refract_scale") {
        rain.trail_refract_scale = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.trail_opacity") {
        rain.trail_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.streak_boost") {
        rain.streak_boost = value;
    }
    if let Some(value) = metadata_f32(prepared, "trails.streak_length") {
        rain.streak_length = value;
    }

    if let Some(value) = metadata_bool(prepared, "micro_droplets.enabled") {
        rain.micro_droplets_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplets_per_second") {
        rain.micro_droplets_per_second = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplet_min_px") {
        rain.micro_droplet_min_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "micro_droplets.micro_droplet_max_px") {
        rain.micro_droplet_max_px = value;
    }

    if let Some(value) = metadata_bool(prepared, "mist.enabled") {
        rain.mist_enabled = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_opacity") {
        rain.mist_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_blur_px") {
        rain.mist_blur_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_accumulation") {
        rain.mist_accumulation = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_time") {
        rain.mist_time = value;
    }
    if let Some(value) = metadata_f32(prepared, "mist.mist_color_strength") {
        rain.mist_color_strength = value;
    }
    if let Some(value) = metadata_u32(prepared, "mist.mist_blur_step") {
        rain.mist_blur_step = value;
    }

    if let Some(value) = metadata_f32(prepared, "optics.refract_base") {
        rain.refract_base = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.refract_scale") {
        rain.refract_scale = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.distortion_px") {
        rain.distortion_px = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.normal_strength") {
        rain.normal_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.chromatic_aberration") {
        rain.chromatic_aberration = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.focus_blur_strength") {
        rain.focus_blur_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.background_blur_px") {
        rain.background_blur_px = value;
    }
    if let Some(value) = metadata_u32(prepared, "optics.background_blur_steps") {
        rain.background_blur_steps = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.smooth_edge_min") {
        rain.smooth_edge_min = value;
    }
    if let Some(value) = metadata_f32(prepared, "optics.smooth_edge_max") {
        rain.smooth_edge_max = value;
    }

    if let Some(value) = metadata_f32(prepared, "compose.opacity") {
        rain.opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.body_opacity") {
        rain.body_opacity = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.scene_blend") {
        rain.scene_blend = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.scene_darken") {
        rain.scene_darken = value;
    }
    if let Some(value) = metadata_f32(prepared, "compose.drop_plane_blur_px") {
        rain.drop_plane_blur_px = value;
    }
    if let Some(value) = metadata_bool(prepared, "compose.reference_mode") {
        rain.reference_mode = value;
    }
    if let Some(value) = metadata_string(prepared, "compose.raindrop_compose") {
        rain.raindrop_compose = parse_rain_glass_raindrop_compose(&value);
    }
    if let Some(value) = metadata_vec2(prepared, "compose.raindrop_eraser_size") {
        rain.raindrop_eraser_size = value;
    }

    if let Some(value) = metadata_bool(prepared, "lighting.receives_scene_light") {
        rain.receives_scene_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_light_response") {
        rain.scene_light_response = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_light_tint_strength") {
        rain.scene_light_tint_strength = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.scene_shadow_floor") {
        rain.scene_shadow_floor = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.rim_strength") {
        rain.rim_strength = value;
    }
    if let Some(value) = metadata_vec4(prepared, "lighting.light_pos") {
        rain.light_pos = value;
    }
    if let Some(value) = metadata_vec3(prepared, "lighting.diffuse_light") {
        rain.diffuse_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.shadow_offset") {
        rain.shadow_offset = value;
    }
    if let Some(value) = metadata_vec3(prepared, "lighting.specular_light") {
        rain.specular_light = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.specular_shininess") {
        rain.specular_shininess = value;
    }
    if let Some(value) = metadata_f32(prepared, "lighting.light_bump") {
        rain.light_bump = value;
    }

    if let Some(value) = metadata_f32(prepared, "depth.z_depth") {
        rain.z_depth = Some(value);
    }
    if let Some(value) = metadata_f32(prepared, "depth.blur_scale") {
        rain.z_depth_blur_scale = value;
    }
    if let Some(value) = metadata_f32(prepared, "depth.focus_response") {
        rain.z_depth_focus_response = value;
    }

    if let Some(value) = metadata_vec3(prepared, "contamination.blood_tint") {
        rain.blood_tint = value;
    }
    if let Some(value) = metadata_f32(prepared, "contamination.blood_amount") {
        rain.blood_amount = value;
    }

    if let Some(value) = metadata_string(prepared, "debug.view") {
        rain.debug_view = parse_rain_glass_debug_view(Some(&value));
    }

    Some(rain.normalized())
}

pub fn look_profile_2d_from_prepared(prepared: &PreparedAsset) -> Option<ColorRamp2d> {
    if !matches!(
        prepared.kind,
        PreparedAssetKind::Unknown(_) | PreparedAssetKind::Image2d
    ) && prepared.kind.as_str() != "camera-look-profile-2d"
    {
        return None;
    }

    let _id = metadata_string(prepared, "id")?;
    Some(
        ColorRamp2d {
            palette_size: metadata_u32(prepared, "palette_size")
                .or_else(|| metadata_u32(prepared, "colors"))
                .unwrap_or(32),
            dither_strength: metadata_f32(prepared, "dither_strength")
                .or_else(|| metadata_f32(prepared, "dither"))
                .unwrap_or(0.12),
            dither_scale: metadata_f32(prepared, "dither_scale")
                .or_else(|| metadata_f32(prepared, "scale"))
                .unwrap_or(1.0),
            layered_dither: metadata_f32(prepared, "layered_dither")
                .or_else(|| metadata_f32(prepared, "layered"))
                .unwrap_or(0.22),
            opacity: metadata_f32(prepared, "opacity").unwrap_or(1.0),
            luma_preserve: metadata_f32(prepared, "luma_preserve")
                .or_else(|| metadata_f32(prepared, "luma"))
                .unwrap_or(0.55),
            highlight_bias: metadata_f32(prepared, "highlight_bias")
                .or_else(|| metadata_f32(prepared, "highlight"))
                .or_else(|| metadata_f32(prepared, "light_bias"))
                .unwrap_or(0.0),
            shadow_bias: metadata_f32(prepared, "shadow_bias")
                .or_else(|| metadata_f32(prepared, "shadow"))
                .unwrap_or(0.0),
            contrast: metadata_f32(prepared, "contrast").unwrap_or(1.0),
            saturation: metadata_f32(prepared, "saturation")
                .or_else(|| metadata_f32(prepared, "sat"))
                .unwrap_or(1.0),
            gamma: metadata_f32(prepared, "gamma").unwrap_or(1.0),
            seed: metadata_u32(prepared, "seed").unwrap_or(0),
        }
        .normalized(),
    )
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(prepared: &PreparedAsset, key: &str) -> Option<bool> {
    prepared.metadata.get(key)?.parse::<bool>().ok()
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared.metadata.get(key)?.parse::<u32>().ok()
}

fn metadata_string(prepared: &PreparedAsset, key: &str) -> Option<String> {
    let value = prepared.metadata.get(key)?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
}

fn metadata_vec2(prepared: &PreparedAsset, key: &str) -> Option<[f32; 2]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
    ])
}

fn metadata_vec3(prepared: &PreparedAsset, key: &str) -> Option<[f32; 3]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
        metadata_f32(prepared, &format!("{key}.2"))?,
    ])
}

fn metadata_vec4(prepared: &PreparedAsset, key: &str) -> Option<[f32; 4]> {
    Some([
        metadata_f32(prepared, &format!("{key}.0"))?,
        metadata_f32(prepared, &format!("{key}.1"))?,
        metadata_f32(prepared, &format!("{key}.2"))?,
        metadata_f32(prepared, &format!("{key}.3"))?,
    ])
}

fn parse_rain_glass_raindrop_compose(value: &str) -> RainGlassRaindropCompose {
    match value.trim().to_ascii_lowercase().as_str() {
        "hard" | "harder" => RainGlassRaindropCompose::Harder,
        _ => RainGlassRaindropCompose::Smoother,
    }
}

fn parse_rain_glass_debug_view(value: Option<&str>) -> RainGlassDebugView {
    match value
        .unwrap_or("final")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "scene" | "scene_input" => RainGlassDebugView::SceneInput,
        "blur" | "blurred" | "blurred_scene" => RainGlassDebugView::BlurredScene,
        "raindrop_map" | "raindrops" => RainGlassDebugView::RaindropMap,
        "droplet_map" | "droplets" => RainGlassDebugView::DropletMap,
        "trail_map" | "trails" => RainGlassDebugView::TrailMap,
        "drop_normals" | "normals" => RainGlassDebugView::DropNormals,
        "drop_mask" | "mask" => RainGlassDebugView::DropMask,
        "mist" => RainGlassDebugView::Mist,
        "refraction" => RainGlassDebugView::Refraction,
        _ => RainGlassDebugView::Final,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn has_exactly_twenty_lens_profiles() {
        assert_eq!(BUILTIN_LENS_PROFILES_2D.len(), 20);
    }

    #[test]
    fn has_exactly_twenty_film_stocks() {
        assert_eq!(BUILTIN_FILM_STOCKS_2D.len(), 20);
    }

    #[test]
    fn has_exactly_ten_camera_presets() {
        assert_eq!(BUILTIN_CAMERA_PRESETS_2D.len(), 10);
    }

    #[test]
    fn resolves_default_profiles() {
        assert!(lens_profile_2d("clean_modern_35mm").is_some());
        assert!(film_stock_2d("neutral_digital_400").is_some());
        assert!(camera_preset_2d("default").is_some());
    }

    #[test]
    fn cinematic_profiles_are_numerically_distinct() {
        let anamorphic = lens_profile_2d("anamorphic_rain_streak").expect("anamorphic lens");
        let clean = lens_profile_2d("clean_modern_35mm").expect("clean lens");
        let cctv = lens_profile_2d("cheap_cctv_1996").expect("cctv lens");
        let cinestill = film_stock_2d("cinestill_800t_halation").expect("cinestill film");
        let surveillance = film_stock_2d("surveillance_tape_color").expect("surveillance film");
        let noir = film_stock_2d("noir_mono_soft").expect("noir film");

        assert!(anamorphic.anamorphic_squeeze > 1.2);
        assert!(cctv.distortion > clean.distortion);
        assert_ne!(cinestill.toe, surveillance.toe);
        assert_ne!(cinestill.shoulder, surveillance.shoulder);
        assert!(surveillance.saturation < cinestill.saturation);
        assert!(noir.saturation <= 0.0);
    }

    #[test]
    fn parses_custom_lens_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/lens/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/lens/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-lens-profile-2d".to_owned()),
            label: Some("Custom Lens".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_lens".to_owned()),
                ("label".to_owned(), "Custom Lens".to_owned()),
                ("focal_length_mm".to_owned(), "42".to_owned()),
                ("glare_strength".to_owned(), "0.5".to_owned()),
            ]),
        };

        let profile = lens_profile_2d_from_prepared(&prepared).expect("custom lens should parse");
        assert_eq!(profile.id, "custom_lens");
        assert_eq!(profile.focal_length_mm, 42.0);
        assert_eq!(profile.glare_strength, 0.5);
    }

    #[test]
    fn parses_custom_film_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/film/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/film/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-film-stock-2d".to_owned()),
            label: Some("Custom Film".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_film".to_owned()),
                ("label".to_owned(), "Custom Film".to_owned()),
                ("base_iso".to_owned(), "500".to_owned()),
                ("saturation".to_owned(), "0.8".to_owned()),
            ]),
        };

        let profile = film_stock_2d_from_prepared(&prepared).expect("custom film should parse");
        assert_eq!(profile.id, "custom_film");
        assert_eq!(profile.base_iso, 500.0);
        assert_eq!(profile.saturation, 0.8);
    }

    #[test]
    fn parses_custom_look_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/look/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/look/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-look-profile-2d".to_owned()),
            label: Some("Custom Look".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("id".to_owned(), "custom_look".to_owned()),
                ("label".to_owned(), "Custom Look".to_owned()),
                ("palette_size".to_owned(), "24".to_owned()),
                ("contrast".to_owned(), "1.12".to_owned()),
                ("shadow_bias".to_owned(), "-0.15".to_owned()),
            ]),
        };

        let profile = look_profile_2d_from_prepared(&prepared).expect("custom look should parse");
        assert_eq!(profile.palette_size, 24);
        assert_eq!(profile.contrast, 1.12);
        assert_eq!(profile.shadow_bias, 0.0);
    }

    #[test]
    fn parses_custom_rain_glass_profile_from_prepared_asset() {
        let prepared = PreparedAsset {
            key: AssetKey::new("mod/camera/rain/custom"),
            source: AssetSourceKind::Generated,
            resolved_path: PathBuf::from("mod/camera/rain/custom.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::Unknown("camera-rain-glass-profile-2d".to_owned()),
            label: Some("Custom Rain".to_owned()),
            format: Some("yaml".to_owned()),
            metadata: BTreeMap::from([
                ("spawn.spawn_rate".to_owned(), "9.5".to_owned()),
                ("droplets.min_radius_px".to_owned(), "28".to_owned()),
                ("optics.refract_scale".to_owned(), "0.9".to_owned()),
                ("mist.enabled".to_owned(), "false".to_owned()),
                ("debug.view".to_owned(), "refraction".to_owned()),
            ]),
        };

        let profile =
            rain_glass_profile_2d_from_prepared(&prepared).expect("custom rain should parse");
        assert_eq!(profile.spawn_rate, 9.5);
        assert_eq!(profile.min_radius_px, 28.0);
        assert_eq!(profile.refract_scale, 0.9);
        assert!(!profile.mist_enabled);
        assert_eq!(profile.debug_view, RainGlassDebugView::Refraction);
    }
}
