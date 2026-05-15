use amigo_2d_post_fx::{
    ColorQuantize2d, Crt2d, DirtyBloom2d, Downscale2d, FilmNoise2d, LensDroplets2dStage,
    PostFx2d, PostFx2dId, PostFx2dInstance, PostFxHost2dId, PostFxLensDroplets2d,
    PostFxPipelineKind, PostFxScope2d, PostFxWetReflections2d, RainGlass2d, RainGlassDebugView,
    RainGlassRaindropCompose, ScopedPostFx2dStack, ShutterBlur2d, WetReflectionsDebugView,
};

use crate::{
    LensDroplets2dDocument, PostFx2dDocument, SceneDocumentError, SceneDocumentResult,
};

pub fn frame_post_fx_host_id(scene_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("scene:{scene_id}:visual2d"))
}

pub fn draw_layer_post_fx_host_id(draw_layer_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("draw_layer:{draw_layer_id}"))
}

pub fn scene_object_post_fx_host_id(scene_object_id: &str) -> PostFxHost2dId {
    PostFxHost2dId::new(format!("scene_object:{scene_object_id}"))
}

pub fn component_post_fx_host_id(
    scene_object_id: &str,
    component_index: usize,
    component_kind: &str,
) -> PostFxHost2dId {
    PostFxHost2dId::new(format!(
        "component:{scene_object_id}:{component_index}:{component_kind}"
    ))
}

pub fn image_part_post_fx_host_id(
    scene_object_id: &str,
    component_index: usize,
    part_id: &str,
) -> PostFxHost2dId {
    PostFxHost2dId::new(format!(
        "image_part:{scene_object_id}:{component_index}:{part_id}"
    ))
}

pub fn build_scoped_post_fx_stack(
    host_id: PostFxHost2dId,
    scope: PostFxScope2d,
    docs: &[PostFx2dDocument],
    scene_id: &str,
    owner_id: &str,
    owner_kind: &str,
) -> SceneDocumentResult<(
    Option<ScopedPostFx2dStack>,
    Vec<amigo_2d_post_fx::LensDroplets2dCertificationReport>,
)> {
    if docs.is_empty() {
        return Ok((None, Vec::new()));
    }

    let pipeline = scope.default_pipeline();
    let mut effects = Vec::with_capacity(docs.len());
    let mut lens_reports = Vec::new();

    for (index, document) in docs.iter().enumerate() {
        let (effect, report) = post_fx_from_document(document, scene_id, owner_id, owner_kind)?;
        if let Some(report) = report {
            lens_reports.push(report);
        }
        effects.push(PostFx2dInstance::new(
            PostFx2dId::new(format!("{}:{index}:{}", host_id.as_str(), document.type_name())),
            effect,
        ));
    }

    Ok((
        Some(
            ScopedPostFx2dStack {
                host_id,
                scope,
                pipeline: match pipeline {
                    PostFxPipelineKind::FrameGraph => PostFxPipelineKind::FrameGraph,
                    PostFxPipelineKind::CachedImage => PostFxPipelineKind::CachedImage,
                    PostFxPipelineKind::OffscreenObject => PostFxPipelineKind::OffscreenObject,
                    PostFxPipelineKind::OffscreenDrawLayer => {
                        PostFxPipelineKind::OffscreenDrawLayer
                    }
                    PostFxPipelineKind::OffscreenGroup => PostFxPipelineKind::OffscreenGroup,
                    PostFxPipelineKind::Unsupported => PostFxPipelineKind::Unsupported,
                },
                effects,
            }
            .normalized(),
        ),
        lens_reports,
    ))
}

pub fn post_fx_from_document(
    document: &PostFx2dDocument,
    scene_id: &str,
    owner_id: &str,
    owner_kind: &str,
) -> SceneDocumentResult<(
    PostFx2d,
    Option<amigo_2d_post_fx::LensDroplets2dCertificationReport>,
)> {
    let output = match document {
        PostFx2dDocument::ColorQuantize(effect) => (
            PostFx2d::ColorQuantize(
                ColorQuantize2d {
                    palette_size: effect.palette_size,
                    dither_strength: effect.dither_strength,
                    opacity: effect.opacity,
                    luma_preserve: effect.luma_preserve,
                    highlight_bias: effect.highlight_bias,
                    gamma: effect.gamma,
                    seed: effect.seed,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::Downscale(effect) => (
            PostFx2d::Downscale(
                Downscale2d {
                    factor: effect.factor,
                    opacity: effect.opacity,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::ShutterBlur(effect) => (
            PostFx2d::ShutterBlur(
                ShutterBlur2d {
                    fps: effect.fps,
                    shutter_angle: effect.shutter_angle,
                    opacity: effect.opacity,
                    history_mix: effect.history_mix,
                    history_mix_2: effect.history_mix_2,
                    edge_rejection: effect.edge_rejection,
                    luma_threshold: effect.luma_threshold,
                    frame_hold: effect.frame_hold,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::DirtyBloom(bloom) => (
            PostFx2d::DirtyBloom(
                DirtyBloom2d {
                    threshold: bloom.threshold,
                    strength: bloom.strength,
                    small_radius_px: bloom.small_radius_px,
                    medium_radius_px: bloom.medium_radius_px,
                    large_radius_px: bloom.large_radius_px,
                    dirty_noise: bloom.dirty_noise,
                    halation_strength: bloom.halation_strength,
                    reflection_smear_x_px: bloom.reflection_smear_x_px,
                    reflection_smear_y_px: bloom.reflection_smear_y_px,
                    seed: bloom.seed,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::Crt(crt) => (
            PostFx2d::Crt(
                Crt2d {
                    scanline_opacity: crt.scanline_opacity,
                    scanline_frequency_px: crt.scanline_frequency_px,
                    rgb_split_px: crt.rgb_split_px,
                    curvature: crt.curvature,
                    vignette: crt.vignette,
                    phosphor_mask: crt.phosphor_mask,
                    brightness_compensation: crt.brightness_compensation,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::FilmNoise(noise) => (
            PostFx2d::FilmNoise(
                FilmNoise2d {
                    iso: noise.iso,
                    grain_size: noise.grain_size,
                    chroma_noise: noise.chroma_noise,
                    color_shift: noise.color_shift,
                    contrast: noise.contrast,
                    saturation: noise.saturation,
                    flicker: noise.flicker,
                    vignette: noise.vignette,
                    opacity: noise.opacity,
                    seed: noise.seed,
                }
                .normalized(),
            ),
            None,
        ),
        PostFx2dDocument::LensDroplets(lens) => {
            let runtime = lens_droplets_from_document(lens);
            let report = runtime.certify();
            if !report.accepted && lens.certification.strict {
                return Err(SceneDocumentError::Hydration {
                    scene_id: scene_id.to_owned(),
                    entity_id: owner_id.to_owned(),
                    component_kind: owner_kind.to_owned(),
                    message: format!("LensDroplets2D `{}` failed certification", lens.id),
                });
            }
            (PostFx2d::LensDroplets(report.normalized.clone()), Some(report))
        }
        PostFx2dDocument::RainGlass(rain) => (
            PostFx2d::RainGlass(rain_glass_from_document(rain)),
            None,
        ),
        PostFx2dDocument::WetReflections(wet) => {
            let reflection_mask = wet.masks.reflection.clone().unwrap_or_default();
            if reflection_mask.trim().is_empty() {
                eprintln!(
                    "warning: wet_reflections `{}` has no reflection mask and will be inactive",
                    wet.id
                );
            }
            (
                PostFx2d::WetReflections(
                    PostFxWetReflections2d {
                        enabled: wet.enabled,
                        reflection_mask,
                        reflection_mask_invert: wet.masks.reflection_invert.unwrap_or(true),
                        edge_map: wet.masks.edges.clone(),
                        reflection_color: wet.masks.reflection_color.clone(),
                        noise_normal: wet.masks.noise_normal.clone(),
                        blur_px: wet.surface.blur_px,
                        distortion_px: wet.surface.distortion_px,
                        shimmer_strength: wet.surface.shimmer_strength,
                        ripple_strength: wet.surface.ripple_strength,
                        wet_darken: wet.surface.wet_darken,
                        specular_boost: wet.surface.specular_boost,
                        edge_power: wet
                            .light_response
                            .edge_power
                            .unwrap_or(wet.surface.edge_power),
                        light_reflection_strength: wet
                            .light_response
                            .strength
                            .unwrap_or(wet.surface.light_reflection_strength),
                        foreground_strength: wet.perspective.foreground_strength,
                        background_strength: wet.perspective.background_strength,
                        horizon_y: wet.perspective.horizon_y,
                        noise_scale: wet.animation.noise_scale,
                        noise_speed: wet.animation.noise_speed,
                        ripple_speed: wet.animation.ripple_speed,
                        debug_view: WetReflectionsDebugView::Final,
                    }
                    .normalized(),
                ),
                None,
            )
        }
    };
    Ok(output)
}

fn lens_droplets_from_document(lens: &LensDroplets2dDocument) -> PostFxLensDroplets2d {
    let stage = match lens.stage.as_deref() {
        Some("after_world_before_ui") | None => LensDroplets2dStage::AfterWorldBeforeUi,
        Some(_) => LensDroplets2dStage::AfterWorldBeforeUi,
    };

    PostFxLensDroplets2d {
        enabled: lens.enabled,
        stage,
        max_droplets: lens.droplets.max,
        spawn_rate: lens.droplets.spawn_rate,
        min_radius_px: lens.droplets.radius_range[0],
        max_radius_px: lens.droplets.radius_range[1],
        min_opacity: lens.droplets.opacity_range[0],
        max_opacity: lens.droplets.opacity_range[1],
        min_lifetime: lens.droplets.lifetime_range[0],
        max_lifetime: lens.droplets.lifetime_range[1],
        dirt_opacity: lens.surface.dirt_opacity,
        darken: lens.surface.darken,
        blur_px: lens.surface.blur_px,
        blur_samples: lens.surface.blur_samples,
        distortion: lens.surface.distortion,
        downsample: lens.surface.downsample,
        streaks_enabled: lens.streaks.enabled,
        streak_chance: lens.streaks.chance,
        gravity_px_per_sec: lens.streaks.gravity_px_per_sec,
        max_streak_length: lens.streaks.max_length,
        wobble: lens.streaks.wobble,
        affects_world: lens.affects.world,
        affects_game_ui: lens.affects.game_ui,
        affects_debug_ui: lens.affects.debug_ui,
        strict_certification: lens.certification.strict,
    }
}

fn rain_glass_from_document(rain: &crate::RainGlass2dDocument) -> RainGlass2d {
    RainGlass2d {
        enabled: rain.enabled,
        spawn_rate: rain.spawn_rate,
        spawn_limit: rain.spawn_limit,
        min_radius_px: rain.spawn_size[0],
        max_radius_px: rain.spawn_size[1],
        seed: rain.seed,
        gravity_px_per_sec2: rain.simulation.gravity_px_per_sec2,
        slip_rate: rain.simulation.slip_rate,
        motion_interval_min: rain.simulation.motion_interval[0],
        motion_interval_max: rain.simulation.motion_interval[1],
        x_shift_min: rain.simulation.x_shifting[0],
        x_shift_max: rain.simulation.x_shifting[1],
        collider_scale: rain.simulation.collider_scale,
        initial_spread: rain.simulation.initial_spread,
        shrink_rate: rain.simulation.shrink_rate,
        velocity_spread: rain.simulation.velocity_spread,
        evaporate: rain.simulation.evaporate,
        trails_enabled: rain.trails.enabled,
        trail_drop_density: rain.trails.density,
        trail_drop_size_min: rain.trails.size[0],
        trail_drop_size_max: rain.trails.size[1],
        trail_distance_min_px: rain.trails.distance_px[0],
        trail_distance_max_px: rain.trails.distance_px[1],
        trail_spread: rain.trails.spread,
        trail_shrink_rate: rain.trails.shrink_rate,
        trail_evaporate: rain.trails.evaporate,
        trail_taper: rain.trails.taper,
        streak_boost: rain.trails.streak_boost,
        streak_length: rain.trails.streak_length,
        micro_droplets_enabled: rain.micro_droplets.enabled,
        micro_droplets_per_second: rain.micro_droplets.per_second,
        micro_droplet_min_px: rain.micro_droplets.size[0],
        micro_droplet_max_px: rain.micro_droplets.size[1],
        mist_enabled: rain.mist.enabled,
        mist_opacity: rain.mist.opacity,
        mist_blur_px: rain.mist.blur_px,
        mist_accumulation: rain.mist.accumulation,
        mist_time: rain.mist.time,
        mist_color_strength: rain.mist.color_strength,
        mist_blur_step: rain.mist.blur_step,
        background_blur_px: rain.render.background_blur_px,
        background_blur_steps: rain.render.background_blur_steps,
        smooth_edge_min: rain.render.smooth_edge[0],
        smooth_edge_max: rain.render.smooth_edge[1],
        refract_base: rain.refract_base,
        refract_scale: rain.refract_scale,
        opacity: rain.opacity,
        chromatic_aberration: rain.render.chromatic_aberration,
        distortion_px: rain.render.distortion_px,
        normal_strength: rain.render.normal_strength,
        focus_blur_strength: rain.render.focus_blur_strength,
        body_opacity: rain.render.body_opacity,
        scene_blend: rain.render.scene_blend,
        drop_plane_blur_px: rain.render.drop_plane_blur_px,
        receives_scene_light: rain.render.receives_scene_light,
        scene_light_tint_strength: rain.render.scene_light_tint_strength,
        scene_shadow_floor: rain.render.scene_shadow_floor,
        blood_tint: rain.render.blood_tint,
        blood_amount: rain.render.blood_amount,
        scene_darken: rain.render.scene_darken,
        trail_refract_scale: rain.render.trail_refract_scale,
        trail_opacity: rain.render.trail_opacity,
        reference_mode: rain.render.reference_mode,
        raindrop_compose: parse_rain_glass_raindrop_compose(&rain.render.raindrop_compose),
        raindrop_eraser_size: rain.render.raindrop_eraser_size,
        scene_light_response: rain.lighting.scene_light_response,
        rim_strength: rain.lighting.rim_strength,
        light_pos: rain.lighting.light_pos,
        diffuse_light: rain.lighting.diffuse,
        shadow_offset: rain.lighting.shadow_offset,
        specular_light: rain.lighting.specular,
        specular_shininess: rain.lighting.specular_shininess,
        light_bump: rain.light_bump,
        debug_view: parse_rain_glass_debug_view(rain.debug.view.as_deref()),
    }
    .normalized()
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
