use super::*;
use std::collections::BTreeMap;

pub fn post_fx_stack_from_flat_metadata(
    metadata: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<PostFx2dStack> {
    let mut effects = Vec::new();

    if metadata.contains_key(&format!("{prefix}.kind")) {
        if let Some(effect) = post_fx_from_flat_metadata(metadata, prefix) {
            effects.push(effect);
        }
    }

    let effect_count = infer_indexed_count(metadata, &format!("{prefix}.effects"));
    for index in 0..effect_count {
        if let Some(effect) =
            post_fx_from_flat_metadata(metadata, &format!("{prefix}.effects.{index}"))
        {
            effects.push(effect);
        }
    }

    if effects.is_empty() {
        None
    } else {
        Some(PostFx2dStack { effects }.normalized())
    }
}

pub fn cached_image_post_fx_stack_from_flat_metadata(
    metadata: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<PostFx2dStack> {
    let stack = post_fx_stack_from_flat_metadata(metadata, prefix)?;
    let effects = stack
        .effects
        .into_iter()
        .filter(|effect| effect.is_cached_image_compatible())
        .collect::<Vec<_>>();

    if effects.is_empty() {
        None
    } else {
        Some(PostFx2dStack { effects }.normalized())
    }
}

pub fn post_fx_from_flat_metadata(
    metadata: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<PostFx2d> {
    let kind = metadata_string(metadata, &format!("{prefix}.kind"))?;
    match kind.trim().to_ascii_lowercase().as_str() {
        "blur" | "gaussian_blur" | "lens_blur" => {
            let defaults = PostFxBlur2d::default();
            Some(PostFx2d::Blur(
                PostFxBlur2d {
                    radius: metadata_f32(metadata, &format!("{prefix}.radius"))
                        .unwrap_or(defaults.radius),
                    downsample: metadata_f32(metadata, &format!("{prefix}.downsample"))
                        .unwrap_or(defaults.downsample),
                    intensity: metadata_f32(metadata, &format!("{prefix}.intensity"))
                        .unwrap_or(defaults.intensity),
                }
                .normalized(),
            ))
        }
        "embossed_edges" | "emboss_edges" | "emboss" => {
            let defaults = PostFxEmbossEdges2d::default();
            Some(PostFx2d::EmbossEdges(
                PostFxEmbossEdges2d {
                    mode: metadata_string(metadata, &format!("{prefix}.mode"))
                        .as_deref()
                        .map(parse_emboss_mode)
                        .unwrap_or(defaults.mode),
                    intensity: metadata_f32(metadata, &format!("{prefix}.intensity"))
                        .unwrap_or(defaults.intensity),
                    edge_strength: metadata_f32(metadata, &format!("{prefix}.edge_strength"))
                        .unwrap_or(defaults.edge_strength),
                    sample_offset_px: metadata_f32(metadata, &format!("{prefix}.sample_offset_px"))
                        .unwrap_or(defaults.sample_offset_px),
                    luma_threshold: metadata_f32(metadata, &format!("{prefix}.luma_threshold"))
                        .unwrap_or(defaults.luma_threshold),
                    luma_gamma: metadata_f32(metadata, &format!("{prefix}.luma_gamma"))
                        .unwrap_or(defaults.luma_gamma),
                    specular_radius_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.specular_radius_px"),
                    )
                    .unwrap_or(defaults.specular_radius_px),
                    distance_falloff: metadata_f32(metadata, &format!("{prefix}.distance_falloff"))
                        .unwrap_or(defaults.distance_falloff),
                    tint: metadata_string(metadata, &format!("{prefix}.tint"))
                        .and_then(parse_color_triplet)
                        .unwrap_or(defaults.tint),
                }
                .normalized(),
            ))
        }
        "dirty_bloom" | "dirtybloom" => {
            let defaults = DirtyBloom2d::default();
            Some(PostFx2d::DirtyBloom(
                DirtyBloom2d {
                    threshold: metadata_f32(metadata, &format!("{prefix}.threshold"))
                        .unwrap_or(defaults.threshold),
                    strength: metadata_f32(metadata, &format!("{prefix}.strength"))
                        .unwrap_or(defaults.strength),
                    small_radius_px: metadata_f32(metadata, &format!("{prefix}.small_radius_px"))
                        .unwrap_or(defaults.small_radius_px),
                    medium_radius_px: metadata_f32(metadata, &format!("{prefix}.medium_radius_px"))
                        .unwrap_or(defaults.medium_radius_px),
                    large_radius_px: metadata_f32(metadata, &format!("{prefix}.large_radius_px"))
                        .unwrap_or(defaults.large_radius_px),
                    dirty_noise: metadata_f32(metadata, &format!("{prefix}.dirty_noise"))
                        .unwrap_or(defaults.dirty_noise),
                    halation_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.halation_strength"),
                    )
                    .unwrap_or(defaults.halation_strength),
                    reflection_smear_x_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.reflection_smear_x_px"),
                    )
                    .unwrap_or(defaults.reflection_smear_x_px),
                    reflection_smear_y_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.reflection_smear_y_px"),
                    )
                    .unwrap_or(defaults.reflection_smear_y_px),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "color_quantize" | "quantize" | "palette_dither" | "gif_dither" => {
            let defaults = ColorQuantize2d::default();
            Some(PostFx2d::ColorQuantize(
                ColorQuantize2d {
                    palette_size: metadata_u32(metadata, &format!("{prefix}.palette_size"))
                        .or_else(|| metadata_u32(metadata, &format!("{prefix}.colors")))
                        .unwrap_or(defaults.palette_size),
                    dither_strength: metadata_f32(metadata, &format!("{prefix}.dither_strength"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.dither")))
                        .unwrap_or(defaults.dither_strength),
                    dither_scale: metadata_f32(metadata, &format!("{prefix}.dither_scale"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.scale")))
                        .unwrap_or(defaults.dither_scale),
                    layered_dither: metadata_f32(metadata, &format!("{prefix}.layered_dither"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.layered")))
                        .unwrap_or(defaults.layered_dither),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    luma_preserve: metadata_f32(metadata, &format!("{prefix}.luma_preserve"))
                        .unwrap_or(defaults.luma_preserve),
                    highlight_bias: metadata_f32(metadata, &format!("{prefix}.highlight_bias"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.light_bias")))
                        .unwrap_or(defaults.highlight_bias),
                    shadow_bias: metadata_f32(metadata, &format!("{prefix}.shadow_bias"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.shadow")))
                        .unwrap_or(defaults.shadow_bias),
                    contrast: metadata_f32(metadata, &format!("{prefix}.contrast"))
                        .unwrap_or(defaults.contrast),
                    saturation: metadata_f32(metadata, &format!("{prefix}.saturation"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.sat")))
                        .unwrap_or(defaults.saturation),
                    gamma: metadata_f32(metadata, &format!("{prefix}.gamma"))
                        .unwrap_or(defaults.gamma),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "downscale" | "pixelate" | "pixel_scale" => {
            let defaults = Downscale2d::default();
            Some(PostFx2d::Downscale(
                Downscale2d {
                    factor: metadata_f32(metadata, &format!("{prefix}.factor"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.scale")))
                        .unwrap_or(defaults.factor),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                }
                .normalized(),
            ))
        }
        "shutter_blur" | "shutter" | "temporal_blur" | "motion_blur_24fps" => {
            let defaults = ShutterBlur2d::default();
            Some(PostFx2d::ShutterBlur(
                ShutterBlur2d {
                    fps: metadata_f32(metadata, &format!("{prefix}.fps")).unwrap_or(defaults.fps),
                    shutter_angle: metadata_f32(metadata, &format!("{prefix}.shutter_angle"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.angle")))
                        .unwrap_or(defaults.shutter_angle),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    history_mix: metadata_f32(metadata, &format!("{prefix}.history_mix"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.previous_mix")))
                        .unwrap_or(defaults.history_mix),
                    history_mix_2: metadata_f32(metadata, &format!("{prefix}.history_mix_2"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.previous_mix_2")))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.older_mix")))
                        .unwrap_or(defaults.history_mix_2),
                    edge_rejection: metadata_f32(metadata, &format!("{prefix}.edge_rejection"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.edge_reject")))
                        .unwrap_or(defaults.edge_rejection),
                    luma_threshold: metadata_f32(metadata, &format!("{prefix}.luma_threshold"))
                        .unwrap_or(defaults.luma_threshold),
                    frame_hold: metadata_bool(metadata, &format!("{prefix}.frame_hold"))
                        .unwrap_or(defaults.frame_hold),
                }
                .normalized(),
            ))
        }
        "crt" | "crt_screen" => {
            let defaults = Crt2d::default();
            Some(PostFx2d::Crt(
                Crt2d {
                    scanline_opacity: metadata_f32(metadata, &format!("{prefix}.scanline_opacity"))
                        .unwrap_or(defaults.scanline_opacity),
                    scanline_frequency_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.scanline_frequency_px"),
                    )
                    .unwrap_or(defaults.scanline_frequency_px),
                    rgb_split_px: metadata_f32(metadata, &format!("{prefix}.rgb_split_px"))
                        .unwrap_or(defaults.rgb_split_px),
                    curvature: metadata_f32(metadata, &format!("{prefix}.curvature"))
                        .unwrap_or(defaults.curvature),
                    vignette: metadata_f32(metadata, &format!("{prefix}.vignette"))
                        .unwrap_or(defaults.vignette),
                    phosphor_mask: metadata_f32(metadata, &format!("{prefix}.phosphor_mask"))
                        .unwrap_or(defaults.phosphor_mask),
                    brightness_compensation: metadata_f32(
                        metadata,
                        &format!("{prefix}.brightness_compensation"),
                    )
                    .unwrap_or(defaults.brightness_compensation),
                }
                .normalized(),
            ))
        }
        "film_noise" | "film_grain" | "noise_overlay" => {
            let defaults = FilmNoise2d::default();
            Some(PostFx2d::FilmNoise(
                FilmNoise2d {
                    iso: metadata_f32(metadata, &format!("{prefix}.iso")).unwrap_or(defaults.iso),
                    grain_size: metadata_f32(metadata, &format!("{prefix}.grain_size"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.scale")))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.grain_scale")))
                        .unwrap_or(defaults.grain_size),
                    chroma_noise: metadata_f32(metadata, &format!("{prefix}.chroma_noise"))
                        .unwrap_or(defaults.chroma_noise),
                    color_shift: metadata_f32(metadata, &format!("{prefix}.color_shift"))
                        .unwrap_or(defaults.color_shift),
                    contrast: metadata_f32(metadata, &format!("{prefix}.contrast"))
                        .unwrap_or(defaults.contrast),
                    saturation: metadata_f32(metadata, &format!("{prefix}.saturation"))
                        .unwrap_or(defaults.saturation),
                    flicker: metadata_f32(metadata, &format!("{prefix}.flicker"))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.speed")))
                        .or_else(|| metadata_f32(metadata, &format!("{prefix}.flicker_speed")))
                        .unwrap_or(defaults.flicker),
                    vignette: metadata_f32(metadata, &format!("{prefix}.vignette"))
                        .unwrap_or(defaults.vignette),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                }
                .normalized(),
            ))
        }
        "lens_droplets" | "lens_drops" | "droplets" => {
            let defaults = PostFxLensDroplets2d::default();
            Some(PostFx2d::LensDroplets(
                PostFxLensDroplets2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    stage: LensDroplets2dStage::AfterWorldBeforeUi,
                    max_droplets: metadata_u32(metadata, &format!("{prefix}.droplets.max"))
                        .unwrap_or(defaults.max_droplets),
                    spawn_rate: metadata_f32(metadata, &format!("{prefix}.droplets.spawn_rate"))
                        .unwrap_or(defaults.spawn_rate),
                    min_radius_px: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.radius_range"),
                    )
                    .unwrap_or(defaults.min_radius_px),
                    max_radius_px: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.radius_range"),
                    )
                    .unwrap_or(defaults.max_radius_px),
                    min_opacity: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.opacity_range"),
                    )
                    .unwrap_or(defaults.min_opacity),
                    max_opacity: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.opacity_range"),
                    )
                    .unwrap_or(defaults.max_opacity),
                    min_lifetime: metadata_range_min(
                        metadata,
                        &format!("{prefix}.droplets.lifetime_range"),
                    )
                    .unwrap_or(defaults.min_lifetime),
                    max_lifetime: metadata_range_max(
                        metadata,
                        &format!("{prefix}.droplets.lifetime_range"),
                    )
                    .unwrap_or(defaults.max_lifetime),
                    dirt_opacity: metadata_f32(metadata, &format!("{prefix}.surface.dirt_opacity"))
                        .unwrap_or(defaults.dirt_opacity),
                    darken: metadata_f32(metadata, &format!("{prefix}.surface.darken"))
                        .unwrap_or(defaults.darken),
                    blur_px: metadata_f32(metadata, &format!("{prefix}.surface.blur_px"))
                        .unwrap_or(defaults.blur_px),
                    blur_samples: metadata_u32(metadata, &format!("{prefix}.surface.blur_samples"))
                        .unwrap_or(defaults.blur_samples),
                    distortion: metadata_f32(metadata, &format!("{prefix}.surface.distortion"))
                        .unwrap_or(defaults.distortion),
                    downsample: metadata_f32(metadata, &format!("{prefix}.surface.downsample"))
                        .unwrap_or(defaults.downsample),
                    streaks_enabled: metadata_bool(metadata, &format!("{prefix}.streaks.enabled"))
                        .unwrap_or(defaults.streaks_enabled),
                    streak_chance: metadata_f32(metadata, &format!("{prefix}.streaks.chance"))
                        .unwrap_or(defaults.streak_chance),
                    gravity_px_per_sec: metadata_f32(
                        metadata,
                        &format!("{prefix}.streaks.gravity_px_per_sec"),
                    )
                    .unwrap_or(defaults.gravity_px_per_sec),
                    max_streak_length: metadata_f32(
                        metadata,
                        &format!("{prefix}.streaks.max_length"),
                    )
                    .unwrap_or(defaults.max_streak_length),
                    wobble: metadata_f32(metadata, &format!("{prefix}.streaks.wobble"))
                        .unwrap_or(defaults.wobble),
                    affects_world: metadata_bool(metadata, &format!("{prefix}.affects.world"))
                        .unwrap_or(defaults.affects_world),
                    affects_game_ui: metadata_bool(metadata, &format!("{prefix}.affects.game_ui"))
                        .unwrap_or(defaults.affects_game_ui),
                    affects_debug_ui: metadata_bool(
                        metadata,
                        &format!("{prefix}.affects.debug_ui"),
                    )
                    .unwrap_or(defaults.affects_debug_ui),
                    strict_certification: metadata_bool(
                        metadata,
                        &format!("{prefix}.certification.strict"),
                    )
                    .unwrap_or(defaults.strict_certification),
                }
                .normalized(),
            ))
        }
        "rain_glass" | "rainglass" | "rain_drops" => {
            let defaults = RainGlass2d::default();
            Some(PostFx2d::RainGlass(
                RainGlass2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    spawn_rate: metadata_f32(metadata, &format!("{prefix}.spawn_rate"))
                        .unwrap_or(defaults.spawn_rate),
                    spawn_limit: metadata_u32(metadata, &format!("{prefix}.spawn_limit"))
                        .unwrap_or(defaults.spawn_limit),
                    min_radius_px: metadata_range_min(metadata, &format!("{prefix}.spawn_size"))
                        .unwrap_or(defaults.min_radius_px),
                    max_radius_px: metadata_range_max(metadata, &format!("{prefix}.spawn_size"))
                        .unwrap_or(defaults.max_radius_px),
                    refract_base: metadata_f32(metadata, &format!("{prefix}.refract_base"))
                        .unwrap_or(defaults.refract_base),
                    refract_scale: metadata_f32(metadata, &format!("{prefix}.refract_scale"))
                        .unwrap_or(defaults.refract_scale),
                    opacity: metadata_f32(metadata, &format!("{prefix}.opacity"))
                        .unwrap_or(defaults.opacity),
                    light_bump: metadata_f32(metadata, &format!("{prefix}.light_bump"))
                        .unwrap_or(defaults.light_bump),
                    drop_plane_blur_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.drop_plane_blur_px"),
                    )
                    .unwrap_or(defaults.drop_plane_blur_px),
                    receives_scene_light: metadata_bool(
                        metadata,
                        &format!("{prefix}.receives_scene_light"),
                    )
                    .unwrap_or(defaults.receives_scene_light),
                    scene_light_tint_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.scene_light_tint_strength"),
                    )
                    .unwrap_or(defaults.scene_light_tint_strength),
                    scene_shadow_floor: metadata_f32(
                        metadata,
                        &format!("{prefix}.scene_shadow_floor"),
                    )
                    .unwrap_or(defaults.scene_shadow_floor),
                    blood_tint: [
                        metadata_f32(metadata, &format!("{prefix}.blood_r"))
                            .unwrap_or(defaults.blood_tint[0]),
                        metadata_f32(metadata, &format!("{prefix}.blood_g"))
                            .unwrap_or(defaults.blood_tint[1]),
                        metadata_f32(metadata, &format!("{prefix}.blood_b"))
                            .unwrap_or(defaults.blood_tint[2]),
                    ],
                    blood_amount: metadata_f32(metadata, &format!("{prefix}.blood_amount"))
                        .unwrap_or(defaults.blood_amount),
                    scene_darken: metadata_f32(metadata, &format!("{prefix}.scene_darken"))
                        .unwrap_or(defaults.scene_darken),
                    seed: metadata_u32(metadata, &format!("{prefix}.seed"))
                        .unwrap_or(defaults.seed),
                    ..defaults
                }
                .normalized(),
            ))
        }
        "wet_reflections" | "wet_reflection" | "wet_surface" => {
            let defaults = PostFxWetReflections2d::default();
            Some(PostFx2d::WetReflections(
                PostFxWetReflections2d {
                    enabled: metadata_bool(metadata, &format!("{prefix}.enabled"))
                        .unwrap_or(defaults.enabled),
                    reflection_mask: metadata_string(metadata, &format!("{prefix}.mask"))
                        .or_else(|| metadata_string(metadata, &format!("{prefix}.reflection_mask")))
                        .unwrap_or_default(),
                    reflection_mask_invert: metadata_bool(
                        metadata,
                        &format!("{prefix}.mask_invert"),
                    )
                    .unwrap_or(defaults.reflection_mask_invert),
                    edge_map: metadata_string(metadata, &format!("{prefix}.edge_map")),
                    reflection_color: metadata_string(
                        metadata,
                        &format!("{prefix}.reflection_color"),
                    ),
                    noise_normal: metadata_string(metadata, &format!("{prefix}.noise_normal")),
                    blur_px: metadata_f32(metadata, &format!("{prefix}.surface.blur_px"))
                        .unwrap_or(defaults.blur_px),
                    distortion_px: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.distortion_px"),
                    )
                    .unwrap_or(defaults.distortion_px),
                    shimmer_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.shimmer_strength"),
                    )
                    .unwrap_or(defaults.shimmer_strength),
                    ripple_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.ripple_strength"),
                    )
                    .unwrap_or(defaults.ripple_strength),
                    wet_darken: metadata_f32(metadata, &format!("{prefix}.surface.wet_darken"))
                        .unwrap_or(defaults.wet_darken),
                    specular_boost: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.specular_boost"),
                    )
                    .unwrap_or(defaults.specular_boost),
                    edge_power: metadata_f32(metadata, &format!("{prefix}.surface.edge_power"))
                        .unwrap_or(defaults.edge_power),
                    light_reflection_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.surface.light_reflection_strength"),
                    )
                    .unwrap_or(defaults.light_reflection_strength),
                    foreground_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.perspective.foreground_strength"),
                    )
                    .unwrap_or(defaults.foreground_strength),
                    background_strength: metadata_f32(
                        metadata,
                        &format!("{prefix}.perspective.background_strength"),
                    )
                    .unwrap_or(defaults.background_strength),
                    horizon_y: metadata_f32(metadata, &format!("{prefix}.perspective.horizon_y"))
                        .unwrap_or(defaults.horizon_y),
                    noise_scale: metadata_f32(metadata, &format!("{prefix}.animation.noise_scale"))
                        .unwrap_or(defaults.noise_scale),
                    noise_speed: metadata_f32(metadata, &format!("{prefix}.animation.noise_speed"))
                        .unwrap_or(defaults.noise_speed),
                    ripple_speed: metadata_f32(
                        metadata,
                        &format!("{prefix}.animation.ripple_speed"),
                    )
                    .unwrap_or(defaults.ripple_speed),
                    debug_view: defaults.debug_view,
                }
                .normalized(),
            ))
        }
        _ => None,
    }
}

fn infer_indexed_count(metadata: &BTreeMap<String, String>, prefix: &str) -> usize {
    let mut max_index = None;

    for key in metadata.keys() {
        let Some(rest) = key.strip_prefix(&format!("{prefix}.")) else {
            continue;
        };
        let Some((raw_index, _field)) = rest.split_once('.') else {
            continue;
        };
        let Ok(index) = raw_index.parse::<usize>() else {
            continue;
        };
        max_index = Some(max_index.map_or(index, |current: usize| current.max(index)));
    }

    max_index.map_or(0, |index| index + 1)
}

fn metadata_string(metadata: &BTreeMap<String, String>, key: &str) -> Option<String> {
    metadata
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

fn metadata_f32(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    metadata.get(key)?.parse::<f32>().ok()
}

fn metadata_bool(metadata: &BTreeMap<String, String>, key: &str) -> Option<bool> {
    metadata.get(key)?.parse::<bool>().ok()
}

fn metadata_u32(metadata: &BTreeMap<String, String>, key: &str) -> Option<u32> {
    metadata.get(key)?.parse::<u32>().ok()
}

fn metadata_range_min(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    parse_range(metadata.get(key)?).map(|range| range.0)
}

fn metadata_range_max(metadata: &BTreeMap<String, String>, key: &str) -> Option<f32> {
    parse_range(metadata.get(key)?).map(|range| range.1)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn quantize_milli(value: f32) -> u32 {
    (finite_or(value, 0.0).max(0.0) * 1000.0).round() as u32
}

fn parse_emboss_mode(value: &str) -> PostFxEmbossMode2d {
    match value.trim().to_ascii_lowercase().as_str() {
        "light_aware_runtime" | "runtime" | "light_aware" => PostFxEmbossMode2d::LightAwareRuntime,
        _ => PostFxEmbossMode2d::PrebakedImage,
    }
}

fn parse_color_triplet(value: String) -> Option<[f32; 3]> {
    let mut parts = value.split(',').map(str::trim);
    let r = parts.next()?.parse::<f32>().ok()?;
    let g = parts.next()?.parse::<f32>().ok()?;
    let b = parts.next()?.parse::<f32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some([r, g, b])
}

fn parse_range(value: &str) -> Option<(f32, f32)> {
    let value = value.trim().trim_start_matches('[').trim_end_matches(']');
    let mut parts = value.split(',').map(str::trim);
    let min = parts.next()?.parse::<f32>().ok()?;
    let max = parts.next()?.parse::<f32>().ok()?;
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_blur_stack_effects() {
        let metadata = BTreeMap::from([
            (
                "layer.post_fx.effects.0.kind".to_owned(),
                "gaussian_blur".to_owned(),
            ),
            (
                "layer.post_fx.effects.0.radius".to_owned(),
                "24.0".to_owned(),
            ),
        ]);

        let stack = post_fx_stack_from_flat_metadata(&metadata, "layer.post_fx")
            .expect("stack should parse");

        assert_eq!(stack.effects.len(), 1);
        assert!(matches!(stack.effects[0], PostFx2d::Blur(_)));
    }

    #[test]
    fn parses_emboss_stack_effect() {
        let metadata = BTreeMap::from([
            ("layer.post_fx.kind".to_owned(), "embossed_edges".to_owned()),
            ("layer.post_fx.edge_strength".to_owned(), "1.6".to_owned()),
        ]);
        let stack = post_fx_stack_from_flat_metadata(&metadata, "layer.post_fx")
            .expect("stack should parse");
        assert_eq!(stack.effects.len(), 1);
        assert!(matches!(stack.effects[0], PostFx2d::EmbossEdges(_)));
    }

    #[test]
    fn parses_color_quantize_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "gif_dither".to_owned()),
            ("fx.colors".to_owned(), "32".to_owned()),
            ("fx.dither".to_owned(), "0.5".to_owned()),
            ("fx.opacity".to_owned(), "0.8".to_owned()),
            ("fx.highlight_bias".to_owned(), "0.45".to_owned()),
            ("fx.layered".to_owned(), "0.25".to_owned()),
            ("fx.shadow".to_owned(), "0.65".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::ColorQuantize(effect) = effect else {
            panic!("expected color quantize effect");
        };
        assert_eq!(effect.palette_size, 32);
        assert_eq!(effect.dither_strength, 0.5);
        assert_eq!(effect.opacity, 0.8);
        assert_eq!(effect.highlight_bias, 0.45);
        assert_eq!(effect.layered_dither, 0.25);
        assert_eq!(effect.shadow_bias, 0.65);
    }

    #[test]
    fn color_quantize_normalized_clamps_values() {
        let effect = ColorQuantize2d {
            palette_size: 999,
            dither_strength: -2.0,
            dither_scale: 99.0,
            layered_dither: 9.0,
            opacity: 4.0,
            luma_preserve: 9.0,
            highlight_bias: 9.0,
            shadow_bias: 9.0,
            contrast: 9.0,
            saturation: 9.0,
            gamma: 9.0,
            seed: 7,
        }
        .normalized();

        assert_eq!(effect.palette_size, 256);
        assert_eq!(effect.dither_strength, 0.0);
        assert_eq!(effect.dither_scale, 8.0);
        assert_eq!(effect.layered_dither, 1.0);
        assert_eq!(effect.opacity, 1.0);
        assert_eq!(effect.luma_preserve, 1.0);
        assert_eq!(effect.highlight_bias, 1.0);
        assert_eq!(effect.shadow_bias, 1.0);
        assert_eq!(effect.contrast, 2.0);
        assert_eq!(effect.saturation, 2.0);
        assert_eq!(effect.gamma, 3.0);
    }

    #[test]
    fn parses_downscale_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "downscale".to_owned()),
            ("fx.factor".to_owned(), "2".to_owned()),
            ("fx.opacity".to_owned(), "0.75".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::Downscale(effect) = effect else {
            panic!("expected downscale effect");
        };
        assert_eq!(effect.factor, 2.0);
        assert_eq!(effect.opacity, 0.75);
    }

    #[test]
    fn downscale_normalized_clamps_values() {
        let effect = Downscale2d {
            factor: 999.0,
            opacity: 9.0,
        }
        .normalized();

        assert_eq!(effect.factor, 16.0);
        assert_eq!(effect.opacity, 1.0);
    }

    #[test]
    fn normalizes_rain_glass_extended_parameters() {
        let effect = RainGlass2d {
            spawn_limit: 99_999,
            refract_scale: 99.0,
            trail_taper: 99.0,
            micro_droplets_per_second: 99_999.0,
            specular_shininess: 99_999.0,
            distortion_px: 999.0,
            normal_strength: 999.0,
            focus_blur_strength: 999.0,
            body_opacity: 999.0,
            trail_refract_scale: 999.0,
            trail_opacity: 999.0,
            scene_light_response: 999.0,
            rim_strength: 999.0,
            streak_boost: 999.0,
            streak_length: 999.0,
            mist_time: 999.0,
            mist_color_strength: 999.0,
            mist_blur_step: 999,
            background_blur_steps: 999,
            raindrop_eraser_size: [999.0, 999.0],
            ..RainGlass2d::default()
        }
        .normalized();

        assert_eq!(effect.spawn_limit, 3000);
        assert!(effect.refract_scale <= 4.0);
        assert!(effect.trail_taper <= 1.0);
        assert!(effect.micro_droplets_per_second <= 5000.0);
        assert!(effect.specular_shininess <= 1024.0);
        assert!(effect.distortion_px <= 128.0);
        assert!(effect.normal_strength <= 16.0);
        assert!(effect.focus_blur_strength <= 2.0);
        assert!(effect.body_opacity <= 1.0);
        assert!(effect.trail_refract_scale <= 2.0);
        assert!(effect.trail_opacity <= 1.0);
        assert!(effect.scene_light_response <= 5.0);
        assert!(effect.rim_strength <= 5.0);
        assert!(effect.streak_boost <= 2.0);
        assert!(effect.streak_length <= 4.0);
        assert!(effect.mist_time <= 120.0);
        assert!(effect.mist_color_strength <= 1.0);
        assert!(effect.mist_blur_step <= 8);
        assert!(effect.background_blur_steps <= 8);
        assert!(effect.raindrop_eraser_size[0] <= 4.0);
        assert!(effect.raindrop_eraser_size[1] <= 4.0);
    }

    #[test]
    fn normalizes_rain_glass_mist_blur() {
        let effect = RainGlass2d {
            mist_blur_px: 999.0,
            ..RainGlass2d::default()
        }
        .normalized();

        assert!(effect.mist_blur_px <= 32.0);
    }

    #[test]
    fn certifies_default_lens_droplets() {
        let report = PostFxLensDroplets2d::default().certify();
        assert!(report.accepted);
        assert!(report.cost_score > 0.0);
    }

    #[test]
    fn rejects_lens_droplets_affecting_debug_ui() {
        let report = PostFxLensDroplets2d {
            affects_debug_ui: true,
            ..PostFxLensDroplets2d::default()
        }
        .certify();

        assert!(!report.accepted);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.code == "lens_droplets_debug_ui_forbidden"));
    }

    #[test]
    fn parses_lens_droplets_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "lens_droplets".to_owned()),
            ("fx.droplets.max".to_owned(), "48".to_owned()),
            ("fx.surface.blur_samples".to_owned(), "4".to_owned()),
            ("fx.affects.debug_ui".to_owned(), "false".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        assert!(matches!(effect, PostFx2d::LensDroplets(_)));
    }

    #[test]
    fn shutter_blur_normalized_clamps_values() {
        let effect = ShutterBlur2d {
            fps: 999.0,
            shutter_angle: 999.0,
            opacity: 999.0,
            history_mix: 999.0,
            history_mix_2: 999.0,
            edge_rejection: 999.0,
            luma_threshold: 999.0,
            frame_hold: true,
        }
        .normalized();

        assert_eq!(effect.fps, 240.0);
        assert_eq!(effect.shutter_angle, 360.0);
        assert_eq!(effect.opacity, 1.0);
        assert_eq!(effect.edge_rejection, 1.0);
        assert_eq!(effect.luma_threshold, 1.0);
        assert!(effect.frame_hold);
    }

    #[test]
    fn parses_shutter_blur_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "shutter_blur".to_owned()),
            ("fx.fps".to_owned(), "24".to_owned()),
            ("fx.shutter_angle".to_owned(), "180".to_owned()),
            ("fx.opacity".to_owned(), "0.7".to_owned()),
            ("fx.frame_hold".to_owned(), "true".to_owned()),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        let PostFx2d::ShutterBlur(effect) = effect else {
            panic!("expected shutter_blur");
        };
        assert_eq!(effect.fps, 24.0);
        assert_eq!(effect.shutter_angle, 180.0);
        assert!(effect.frame_hold);
    }

    #[test]
    fn wet_reflections_kind_is_stable() {
        assert_eq!(
            PostFx2d::WetReflections(PostFxWetReflections2d::default()).kind(),
            "wet_reflections"
        );
    }

    #[test]
    fn wet_reflections_inactive_without_mask() {
        assert!(!PostFxWetReflections2d::default().is_active());
    }

    #[test]
    fn wet_reflections_normalized_clamps_values() {
        let effect = PostFxWetReflections2d {
            enabled: true,
            reflection_mask: "mask.png".to_owned(),
            reflection_mask_invert: true,
            edge_map: None,
            reflection_color: None,
            noise_normal: None,
            blur_px: 99.0,
            distortion_px: -3.0,
            shimmer_strength: 4.0,
            ripple_strength: 4.0,
            wet_darken: -2.0,
            specular_boost: 9.0,
            edge_power: 0.0,
            light_reflection_strength: 9.0,
            foreground_strength: 9.0,
            background_strength: -4.0,
            horizon_y: 9.0,
            noise_scale: 0.0,
            noise_speed: 9.0,
            ripple_speed: -9.0,
            debug_view: WetReflectionsDebugView::Final,
        }
        .normalized();

        assert_eq!(effect.blur_px, 12.0);
        assert_eq!(effect.distortion_px, 0.0);
        assert_eq!(effect.shimmer_strength, 1.0);
        assert_eq!(effect.ripple_strength, 1.0);
        assert_eq!(effect.wet_darken, 0.0);
        assert_eq!(effect.specular_boost, 4.0);
        assert_eq!(effect.edge_power, 0.25);
        assert_eq!(effect.light_reflection_strength, 4.0);
        assert_eq!(effect.foreground_strength, 4.0);
        assert_eq!(effect.background_strength, 0.0);
        assert_eq!(effect.horizon_y, 1.0);
        assert_eq!(effect.noise_scale, 0.01);
        assert_eq!(effect.noise_speed, 8.0);
        assert_eq!(effect.ripple_speed, -8.0);
    }

    #[test]
    fn parses_wet_reflections_from_flat_metadata() {
        let metadata = BTreeMap::from([
            ("fx.kind".to_owned(), "wet_reflections".to_owned()),
            (
                "fx.reflection_mask".to_owned(),
                "rotten-club/layered-images/neon-alley/reflection_mask.png".to_owned(),
            ),
            (
                "fx.edge_map".to_owned(),
                "rotten-club/layered-images/neon-alley/edge_map_2.png".to_owned(),
            ),
        ]);

        let effect = post_fx_from_flat_metadata(&metadata, "fx").expect("effect should parse");
        assert!(matches!(effect, PostFx2d::WetReflections(_)));
    }
}
