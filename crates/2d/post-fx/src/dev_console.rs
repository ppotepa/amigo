use crate::{
    Crt2d, DirtyBloom2d, PostFx2d, PostFx2dService, PostFx2dStack, PostFxBlur2d, RainGlass2d,
    RainGlassDebugView, RainGlassRaindropCompose, ShutterBlur2d,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostFxDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct PostFxDevConsoleCommandContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub fn can_handle_post_fx_dev_console_command(name: &str) -> bool {
    name == "postfx" || name == "postfx.items" || name.starts_with("postfx.")
}

pub fn handle_post_fx_dev_console_command(
    ctx: PostFxDevConsoleCommandContext<'_>,
    name: &str,
    args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    let (name, args) = normalize_postfx_command(name, args);

    match name.as_str() {
        "postfx.cert" => {
            let reports = ctx.post_fx_service.lens_certification_reports();
            if reports.is_empty() {
                return PostFxDevConsoleCommandOutcome::Handled(
                    "postfx.cert: no LensDroplets2D reports".to_owned(),
                );
            }
            let lines = reports
                .into_iter()
                .map(|report| {
                    format!(
                        "LensDroplets2D accepted={} cost={:.1} issues={} renderer={}",
                        report.accepted,
                        report.cost_score,
                        report.issues.len(),
                        ctx.post_fx_service.renderer_mode()
                    )
                })
                .collect::<Vec<_>>();
            PostFxDevConsoleCommandOutcome::Handled(lines.join("\n"))
        }
        "postfx.dirty_bloom" => handle_dirty_bloom(ctx.post_fx_service, &args),
        "postfx.crt" => handle_crt(ctx.post_fx_service, &args),
        "postfx.rain_glass" => handle_rain_glass(ctx.post_fx_service, &args),
        "postfx.shutter_blur" | "postfx.shutter" => handle_shutter_blur(ctx.post_fx_service, &args),
        "postfx.stats" => {
            let stack = ctx.post_fx_service.scene_stack();
            let dirty_bloom_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::DirtyBloom(_)));
            let crt_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::Crt(_)));
            let film_noise_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::FilmNoise(_)));
            let lens_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::LensDroplets(_)));
            let wet_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::WetReflections(_)));
            let rain_glass_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::RainGlass(_)));
            let shutter_blur_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::ShutterBlur(_)));
            PostFxDevConsoleCommandOutcome::Handled(format!(
                "postfx.effects={} dirty_bloom_active={} crt_active={} film_noise_active={} lens_droplets_active={} wet_reflections_active={} rain_glass_active={} shutter_blur_active={} renderer_mode={} overlay_supported={} blur_supported={} world_offscreen_post_fx_supported={}",
                stack.effects.len(),
                dirty_bloom_active,
                crt_active,
                film_noise_active,
                lens_active,
                wet_active,
                rain_glass_active,
                shutter_blur_active,
                ctx.post_fx_service.renderer_mode(),
                ctx.post_fx_service.supports_lens_droplets_overlay(),
                ctx.post_fx_service.supports_lens_droplets_blur(),
                ctx.post_fx_service.supports_world_offscreen_post_fx()
            ))
        }
        "postfx.items.list" | "postfx.list" => postfx_items_list(ctx),
        "postfx.items.count" | "postfx.count" => PostFxDevConsoleCommandOutcome::Handled(format!(
            "postfx.items={}",
            ctx.post_fx_service.scene_effect_count()
        )),
        "postfx.items.clear" | "postfx.clear" => {
            ctx.post_fx_service.clear_scene_stack();
            PostFxDevConsoleCommandOutcome::Handled("postfx.items cleared".to_owned())
        }
        "postfx.items.add" | "postfx.add" => postfx_items_add(ctx, &args),
        "postfx.items.inspect" | "postfx.inspect" => postfx_items_inspect(ctx, &args),
        _ => PostFxDevConsoleCommandOutcome::Unhandled,
    }
}

fn normalize_postfx_command(name: &str, args: &[String]) -> (String, Vec<String>) {
    let mut normalized_name = name.to_owned();
    let mut normalized_args = args.to_vec();

    if normalized_name == "postfx" {
        if let Some(verb) = normalized_args.first().cloned() {
            normalized_name = format!("postfx.{verb}");
            normalized_args.remove(0);
        } else {
            normalized_name = "postfx.stats".to_owned();
        }
    }

    if normalized_name == "postfx.items" {
        if let Some(verb) = normalized_args.first().cloned() {
            normalized_name = format!("postfx.items.{verb}");
            normalized_args.remove(0);
        } else {
            normalized_name = "postfx.items.list".to_owned();
        }
    }

    (normalized_name, normalized_args)
}

fn postfx_items_list(ctx: PostFxDevConsoleCommandContext<'_>) -> PostFxDevConsoleCommandOutcome {
    let effects = ctx.post_fx_service.scene_effects();

    if effects.is_empty() {
        return PostFxDevConsoleCommandOutcome::Handled("postfx.items=0".to_owned());
    }

    let lines = effects
        .into_iter()
        .enumerate()
        .map(|(index, effect)| describe_postfx_effect(index, &effect))
        .collect::<Vec<_>>();

    PostFxDevConsoleCommandOutcome::Handled(lines.join("\n"))
}

fn postfx_items_add(
    ctx: PostFxDevConsoleCommandContext<'_>,
    args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    let Some(kind) = args.first().map(String::as_str) else {
        return PostFxDevConsoleCommandOutcome::Error("usage: postfx.items add <blur>".to_owned());
    };

    match kind {
        "blur" => {
            ctx.post_fx_service
                .push_scene_effect(PostFx2d::Blur(PostFxBlur2d::default()));
            PostFxDevConsoleCommandOutcome::Handled("postfx.items added blur".to_owned())
        }
        _ => PostFxDevConsoleCommandOutcome::Error(format!(
            "unsupported postfx kind `{kind}`; supported: blur"
        )),
    }
}

fn postfx_items_inspect(
    ctx: PostFxDevConsoleCommandContext<'_>,
    args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    let Some(index) = args.first().and_then(|value| value.parse::<usize>().ok()) else {
        return PostFxDevConsoleCommandOutcome::Error(
            "usage: postfx.items inspect <index>".to_owned(),
        );
    };

    let Some(effect) = ctx.post_fx_service.scene_effect(index) else {
        return PostFxDevConsoleCommandOutcome::Error(format!(
            "postfx item index {index} does not exist"
        ));
    };

    PostFxDevConsoleCommandOutcome::Handled(describe_postfx_effect(index, &effect))
}

fn describe_postfx_effect(index: usize, effect: &PostFx2d) -> String {
    format!(
        "{}: kind={} active={}",
        index,
        effect.clone().kind(),
        effect.is_active()
    )
}

fn handle_dirty_bloom(
    service: &PostFx2dService,
    args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    let mut stack = service.scene_stack();
    let index = ensure_dirty_bloom(&mut stack);
    let mut bloom = match stack.effects[index] {
        PostFx2d::DirtyBloom(bloom) => bloom,
        _ => DirtyBloom2d::default(),
    };

    let updates = match parse_updates(args) {
        Ok(updates) => updates,
        Err(outcome) => return outcome,
    };
    for (field, value) in updates {
        let value = match parse_f32(value) {
            Ok(value) => value,
            Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
        };
        match field {
            "threshold" => bloom.threshold = value,
            "strength" => bloom.strength = value,
            "small_radius_px" | "small" => bloom.small_radius_px = value,
            "medium_radius_px" | "medium" => bloom.medium_radius_px = value,
            "large_radius_px" | "large" => bloom.large_radius_px = value,
            "dirty_noise" | "noise" => bloom.dirty_noise = value,
            "halation_strength" | "halation" => bloom.halation_strength = value,
            "reflection_smear_x_px" | "smear_x" => bloom.reflection_smear_x_px = value,
            "reflection_smear_y_px" | "smear_y" => bloom.reflection_smear_y_px = value,
            "seed" => bloom.seed = value.max(0.0) as u32,
            other => {
                return PostFxDevConsoleCommandOutcome::Error(format!(
                    "unknown dirty_bloom field `{other}`"
                ));
            }
        }
    }

    bloom = bloom.normalized();
    stack.effects[index] = PostFx2d::DirtyBloom(bloom);
    service.set_scene_stack(stack.normalized());

    PostFxDevConsoleCommandOutcome::Handled(format!(
        "dirty_bloom threshold={:.2} strength={:.2} small={:.1} medium={:.1} large={:.1} dirty_noise={:.2} halation={:.2} smear_x={:.1} smear_y={:.1} seed={}",
        bloom.threshold,
        bloom.strength,
        bloom.small_radius_px,
        bloom.medium_radius_px,
        bloom.large_radius_px,
        bloom.dirty_noise,
        bloom.halation_strength,
        bloom.reflection_smear_x_px,
        bloom.reflection_smear_y_px,
        bloom.seed
    ))
}

fn handle_crt(service: &PostFx2dService, args: &[String]) -> PostFxDevConsoleCommandOutcome {
    let mut stack = service.scene_stack();
    let index = ensure_crt(&mut stack);
    let mut crt = match stack.effects[index] {
        PostFx2d::Crt(crt) => crt,
        _ => Crt2d::default(),
    };

    let updates = match parse_updates(args) {
        Ok(updates) => updates,
        Err(outcome) => return outcome,
    };
    for (field, value) in updates {
        let value = match parse_f32(value) {
            Ok(value) => value,
            Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
        };
        match field {
            "scanline_opacity" | "scanlines" => crt.scanline_opacity = value,
            "scanline_frequency_px" | "frequency" => crt.scanline_frequency_px = value,
            "rgb_split_px" | "rgb_split" => crt.rgb_split_px = value,
            "curvature" => crt.curvature = value,
            "vignette" => crt.vignette = value,
            "phosphor_mask" | "phosphor" => crt.phosphor_mask = value,
            "brightness_compensation" | "brightness" => crt.brightness_compensation = value,
            other => {
                return PostFxDevConsoleCommandOutcome::Error(format!(
                    "unknown crt field `{other}`"
                ));
            }
        }
    }

    crt = crt.normalized();
    stack.effects[index] = PostFx2d::Crt(crt);
    service.set_scene_stack(stack.normalized());

    PostFxDevConsoleCommandOutcome::Handled(format!(
        "crt scanline_opacity={:.2} frequency={:.1} rgb_split={:.1} curvature={:.3} vignette={:.2} phosphor={:.2} brightness={:.2}",
        crt.scanline_opacity,
        crt.scanline_frequency_px,
        crt.rgb_split_px,
        crt.curvature,
        crt.vignette,
        crt.phosphor_mask,
        crt.brightness_compensation
    ))
}

fn handle_rain_glass(service: &PostFx2dService, args: &[String]) -> PostFxDevConsoleCommandOutcome {
    let mut stack = service.scene_stack();
    let index = ensure_rain_glass(&mut stack);
    let mut rain = match stack.effects[index] {
        PostFx2d::RainGlass(rain) => rain,
        _ => RainGlass2d::default(),
    };

    let updates = match parse_updates(args) {
        Ok(updates) => updates,
        Err(outcome) => return outcome,
    };
    for (field, value) in updates {
        match field {
            "enabled" => rain.enabled = parse_bool(value).unwrap_or(rain.enabled),
            "trails_enabled" | "trails" => {
                rain.trails_enabled = parse_bool(value).unwrap_or(rain.trails_enabled)
            }
            "micro_droplets_enabled" | "micro_enabled" => {
                rain.micro_droplets_enabled =
                    parse_bool(value).unwrap_or(rain.micro_droplets_enabled)
            }
            "mist_enabled" | "mist" => {
                rain.mist_enabled = parse_bool(value).unwrap_or(rain.mist_enabled)
            }
            "receives_scene_light" | "scene_lighting" | "light_react" => {
                rain.receives_scene_light =
                    parse_bool(value).unwrap_or(rain.receives_scene_light)
            }
            "reference_mode" | "reference" => {
                rain.reference_mode = parse_bool(value).unwrap_or(rain.reference_mode)
            }
            "debug" | "debug_view" => rain.debug_view = parse_rain_glass_debug_view(value),
            "raindrop_compose" | "compose" => {
                rain.raindrop_compose = parse_rain_glass_compose(value)
            }
            "preset" => {
                if let Err(message) = apply_rain_glass_preset(&mut rain, value) {
                    return PostFxDevConsoleCommandOutcome::Error(message);
                }
            }
            "spawn_limit" => {
                let value = match parse_f32(value) {
                    Ok(value) => value,
                    Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
                };
                rain.spawn_limit = value.max(0.0) as u32;
            }
            "seed" => {
                let value = match parse_f32(value) {
                    Ok(value) => value,
                    Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
                };
                rain.seed = value.max(0.0) as u32;
            }
            _ => {
                let value = match parse_f32(value) {
                    Ok(value) => value,
                    Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
                };
                match field {
                    "spawn_rate" => rain.spawn_rate = value,
                    "min_radius_px" | "radius_min" => rain.min_radius_px = value,
                    "max_radius_px" | "radius_max" => rain.max_radius_px = value,
                    "gravity" | "gravity_px_per_sec2" => rain.gravity_px_per_sec2 = value,
                    "slip_rate" => rain.slip_rate = value,
                    "motion_interval_min" | "motion_min" => rain.motion_interval_min = value,
                    "motion_interval_max" | "motion_max" => rain.motion_interval_max = value,
                    "x_shift_min" => rain.x_shift_min = value,
                    "x_shift_max" => rain.x_shift_max = value,
                    "collider_scale" | "collider" => rain.collider_scale = value,
                    "initial_spread" | "impact_spread" => rain.initial_spread = value,
                    "shrink_rate" | "spread_shrink" => rain.shrink_rate = value,
                    "velocity_spread" | "velocity_stretch" => rain.velocity_spread = value,
                    "evaporate" => rain.evaporate = value,
                    "refract_base" => rain.refract_base = value,
                    "refract_scale" => rain.refract_scale = value,
                    "opacity" => rain.opacity = value,
                    "background_blur_px" | "blur" => rain.background_blur_px = value,
                    "chromatic_aberration" | "chroma" => rain.chromatic_aberration = value,
                    "distortion_px" | "distortion" => rain.distortion_px = value,
                    "normal_strength" | "normal" => rain.normal_strength = value,
                    "focus_blur_strength" | "focus_blur" => rain.focus_blur_strength = value,
                    "body_opacity" | "body" => rain.body_opacity = value,
                    "scene_blend" | "blend" => rain.scene_blend = value,
                    "drop_plane_blur_px" | "drop_blur" | "plane_blur" => {
                        rain.drop_plane_blur_px = value
                    }
                    "scene_light_tint_strength" | "light_tint" | "tint" => {
                        rain.scene_light_tint_strength = value
                    }
                    "scene_shadow_floor" | "shadow_floor" => rain.scene_shadow_floor = value,
                    "trail_refract_scale" | "trail_refract" => rain.trail_refract_scale = value,
                    "trail_opacity" => rain.trail_opacity = value,
                    "scene_light_response" | "scene_light" => rain.scene_light_response = value,
                    "rim_strength" | "rim" => rain.rim_strength = value,
                    "smooth_edge_min" => rain.smooth_edge_min = value,
                    "smooth_edge_max" => rain.smooth_edge_max = value,
                    "trail_taper" => rain.trail_taper = value,
                    "trail_spread" => rain.trail_spread = value,
                    "trail_drop_density" | "trail_density" => rain.trail_drop_density = value,
                    "trail_evaporate" => rain.trail_evaporate = value,
                    "trail_shrink_rate" => rain.trail_shrink_rate = value,
                    "streak_boost" | "streak" => rain.streak_boost = value,
                    "streak_length" | "streak_len" => rain.streak_length = value,
                    "trail_distance_min_px" | "trail_dist_min" => {
                        rain.trail_distance_min_px = value
                    }
                    "trail_distance_max_px" | "trail_dist_max" => {
                        rain.trail_distance_max_px = value
                    }
                    "trail_drop_size_min" | "trail_size_min" => rain.trail_drop_size_min = value,
                    "trail_drop_size_max" | "trail_size_max" => rain.trail_drop_size_max = value,
                    "mist_accumulation" => rain.mist_accumulation = value,
                    "mist_opacity" => rain.mist_opacity = value,
                    "mist_time" => rain.mist_time = value,
                    "mist_color_strength" | "mist_strength" => {
                        rain.mist_color_strength = value;
                    }
                    "mist_blur_px" | "mist_blur" | "mist_blur_radius" => {
                        rain.mist_blur_px = value;
                    }
                    "mist_blur_step" => rain.mist_blur_step = value.max(0.0) as u32,
                    "background_blur_steps" | "blur_steps" => {
                        rain.background_blur_steps = value.max(0.0) as u32;
                    }
                    "raindrop_eraser_size" | "eraser" => {
                        rain.raindrop_eraser_size = [value, value];
                    }
                    "raindrop_eraser_min" | "eraser_min" => rain.raindrop_eraser_size[0] = value,
                    "raindrop_eraser_max" | "eraser_max" => rain.raindrop_eraser_size[1] = value,
                    "micro_droplets_per_second" | "micro" => {
                        rain.micro_droplets_per_second = value;
                    }
                    "micro_droplet_min_px" | "micro_min" => rain.micro_droplet_min_px = value,
                    "micro_droplet_max_px" | "micro_max" => rain.micro_droplet_max_px = value,
                    "light_bump" => rain.light_bump = value,
                    "diffuse" | "diffuse_light" => rain.diffuse_light = [value, value, value],
                    "specular" | "specular_light" | "specular_color" => {
                        rain.specular_light = [value, value, value]
                    }
                    "shininess" | "specular_shininess" => rain.specular_shininess = value,
                    "shadow_offset" => rain.shadow_offset = value,
                    other => {
                        return PostFxDevConsoleCommandOutcome::Error(format!(
                            "unknown rain_glass field `{other}`"
                        ));
                    }
                }
            }
        }
    }

    rain = rain.normalized();
    stack.effects[index] = PostFx2d::RainGlass(rain);
    service.set_scene_stack(stack.normalized());

    PostFxDevConsoleCommandOutcome::Handled(format!(
        "rain_glass enabled={} spawn_rate={:.2} spawn_limit={} radius=[{:.1},{:.1}] gravity={:.1} slip={:.2} refract=[{:.2},{:.2}] opacity={:.2} blur={:.1}/steps={} chroma={:.2} optics(dist_px={:.1} normal={:.2} focus_blur={:.2} body={:.2} blend={:.2} drop_blur={:.2} scene_light={:.2} react={} tint={:.2} floor={:.2} rim={:.2} trail_refract={:.2} trail_opacity={:.2} compose={:?} eraser=[{:.2},{:.2}]) trail(taper={:.2} spread={:.2} streak=[{:.2},{:.2}] evap={:.1} shrink={:.3} dist=[{:.1},{:.1}] size=[{:.2},{:.2}]) mist(opacity={:.2} blur={:.1} blur_step={} time={:.1} color={:.3} acc={:.2}) micro={:.1} spec={:.1} shadow={:.2} debug={:?} seed={}",
        rain.enabled,
        rain.spawn_rate,
        rain.spawn_limit,
        rain.min_radius_px,
        rain.max_radius_px,
        rain.gravity_px_per_sec2,
        rain.slip_rate,
        rain.refract_base,
        rain.refract_scale,
        rain.opacity,
        rain.background_blur_px,
        rain.background_blur_steps,
        rain.chromatic_aberration,
        rain.distortion_px,
        rain.normal_strength,
        rain.focus_blur_strength,
        rain.body_opacity,
        rain.scene_blend,
        rain.drop_plane_blur_px,
        rain.scene_light_response,
        rain.receives_scene_light,
        rain.scene_light_tint_strength,
        rain.scene_shadow_floor,
        rain.rim_strength,
        rain.trail_refract_scale,
        rain.trail_opacity,
        rain.raindrop_compose,
        rain.raindrop_eraser_size[0],
        rain.raindrop_eraser_size[1],
        rain.trail_taper,
        rain.trail_spread,
        rain.streak_boost,
        rain.streak_length,
        rain.trail_evaporate,
        rain.trail_shrink_rate,
        rain.trail_distance_min_px,
        rain.trail_distance_max_px,
        rain.trail_drop_size_min,
        rain.trail_drop_size_max,
        rain.mist_opacity,
        rain.mist_blur_px,
        rain.mist_blur_step,
        rain.mist_time,
        rain.mist_color_strength,
        rain.mist_accumulation,
        rain.micro_droplets_per_second,
        rain.specular_shininess,
        rain.shadow_offset,
        rain.debug_view,
        rain.seed
    ))
}

fn handle_shutter_blur(
    service: &PostFx2dService,
    args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    let mut stack = service.scene_stack();
    let index = ensure_shutter_blur(&mut stack);
    let mut effect = match stack.effects[index] {
        PostFx2d::ShutterBlur(effect) => effect,
        _ => ShutterBlur2d::default(),
    };

    let updates = match parse_updates(args) {
        Ok(updates) => updates,
        Err(outcome) => return outcome,
    };

    for (field, value) in updates {
        match field {
            "frame_hold" | "hold" => {
                effect.frame_hold = parse_bool(value).unwrap_or(effect.frame_hold);
            }
            _ => {
                let value = match parse_f32(value) {
                    Ok(value) => value,
                    Err(message) => return PostFxDevConsoleCommandOutcome::Error(message),
                };
                match field {
                    "fps" => effect.fps = value,
                    "shutter_angle" | "angle" => effect.shutter_angle = value,
                    "opacity" | "strength" => effect.opacity = value,
                    "history_mix" | "previous_mix" | "mix" => effect.history_mix = value,
                    "history_mix_2" | "previous_mix_2" | "older_mix" | "mix2" => effect.history_mix_2 = value,
                    "edge_rejection" | "edge_reject" => effect.edge_rejection = value,
                    "luma_threshold" | "luma" => effect.luma_threshold = value,
                    other => {
                        return PostFxDevConsoleCommandOutcome::Error(format!(
                            "unknown shutter_blur field `{other}`"
                        ));
                    }
                }
            }
        }
    }

    effect = effect.normalized();
    stack.effects[index] = PostFx2d::ShutterBlur(effect);
    service.set_scene_stack(stack.normalized());

    PostFxDevConsoleCommandOutcome::Handled(format!(
        "shutter_blur fps={:.1} angle={:.1} opacity={:.2} history_mix={:.2} history_mix_2={:.2} edge_rejection={:.2} luma_threshold={:.3} frame_hold={}",
        effect.fps,
        effect.shutter_angle,
        effect.opacity,
        effect.history_mix,
        effect.history_mix_2,
        effect.edge_rejection,
        effect.luma_threshold,
        effect.frame_hold
    ))
}

fn ensure_dirty_bloom(stack: &mut PostFx2dStack) -> usize {
    if let Some(index) = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::DirtyBloom(_)))
    {
        return index;
    }

    let insert_at = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::Crt(_) | PostFx2d::FilmNoise(_)))
        .unwrap_or(stack.effects.len());
    stack
        .effects
        .insert(insert_at, PostFx2d::DirtyBloom(DirtyBloom2d::default()));
    insert_at
}

fn ensure_crt(stack: &mut PostFx2dStack) -> usize {
    if let Some(index) = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::Crt(_)))
    {
        return index;
    }

    stack.effects.push(PostFx2d::Crt(Crt2d::default()));
    stack.effects.len() - 1
}

fn ensure_rain_glass(stack: &mut PostFx2dStack) -> usize {
    if let Some(index) = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::RainGlass(_)))
    {
        return index;
    }

    stack
        .effects
        .push(PostFx2d::RainGlass(RainGlass2d::default()));
    stack.effects.len() - 1
}

fn ensure_shutter_blur(stack: &mut PostFx2dStack) -> usize {
    if let Some(index) = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::ShutterBlur(_)))
    {
        return index;
    }

    let insert_at = stack
        .effects
        .iter()
        .position(|effect| matches!(effect, PostFx2d::Crt(_) | PostFx2d::Downscale(_)))
        .unwrap_or(stack.effects.len());
    stack
        .effects
        .insert(insert_at, PostFx2d::ShutterBlur(ShutterBlur2d::default()));
    insert_at
}

fn parse_updates(args: &[String]) -> Result<Vec<(&str, &str)>, PostFxDevConsoleCommandOutcome> {
    if args.is_empty() {
        return Ok(Vec::new());
    }
    if args.len() == 2 && !args[0].contains('=') {
        return Ok(vec![(args[0].as_str(), args[1].as_str())]);
    }

    let mut updates = Vec::new();
    for arg in args {
        let Some((field, value)) = arg.split_once('=') else {
            return Err(PostFxDevConsoleCommandOutcome::Error(
                "expected `<field> <value>` or `field=value`".to_owned(),
            ));
        };
        updates.push((field, value));
    }
    Ok(updates)
}

fn parse_f32(value: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .map_err(|_| format!("expected numeric value, got `{value}`"))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_rain_glass_debug_view(value: &str) -> RainGlassDebugView {
    match value.trim().to_ascii_lowercase().as_str() {
        "scene" | "scene_input" => RainGlassDebugView::SceneInput,
        "blur" | "blurred" | "blurred_scene" => RainGlassDebugView::BlurredScene,
        "raindrop_map" | "raindrops" => RainGlassDebugView::RaindropMap,
        "droplet_map" | "droplets" => RainGlassDebugView::DropletMap,
        "trail_map" | "streak_map" | "trails" | "streaks" => RainGlassDebugView::TrailMap,
        "drop_normals" | "normals" => RainGlassDebugView::DropNormals,
        "drop_mask" | "mask" => RainGlassDebugView::DropMask,
        "mist" => RainGlassDebugView::Mist,
        "refraction" => RainGlassDebugView::Refraction,
        _ => RainGlassDebugView::Final,
    }
}

fn parse_rain_glass_compose(value: &str) -> RainGlassRaindropCompose {
    match value.trim().to_ascii_lowercase().as_str() {
        "hard" | "harder" => RainGlassRaindropCompose::Harder,
        _ => RainGlassRaindropCompose::Smoother,
    }
}

#[derive(Debug, Clone, Copy)]
struct RainGlassReferenceControls {
    spawn_rate: f32,
    spawn_limit: u32,
    spawn_min: f32,
    spawn_max: f32,
    slip_rate: f32,
    gravity: f32,
    evaporate: f32,
    initial_spread: f32,
    velocity_spread: f32,
    shrink_rate: f32,
    trail_density: f32,
    trail_size: f32,
    trail_spread: f32,
    streak_boost: f32,
    streak_length: f32,
    streak_taper: f32,
    streak_evap: f32,
    micro_drops: f32,
    micro_min: f32,
    micro_max: f32,
    background_blur_steps: u32,
    mist_enabled: bool,
    mist_blur_step: u32,
    mist_time: f32,
    mist_strength: f32,
    edge_soft_a: f32,
    edge_soft_b: f32,
    refract_base: f32,
    refract_scale: f32,
    shadow_offset: f32,
    diffuse: f32,
    specular: f32,
    shininess: f32,
    light_x: f32,
    light_y: f32,
    light_z: f32,
    light_bump: f32,
}

impl RainGlassReferenceControls {
    fn html_current_controls() -> Self {
        Self {
            spawn_rate: 10.0,
            spawn_limit: 850,
            spawn_min: 45.0,
            spawn_max: 118.0,
            slip_rate: 0.34,
            gravity: 2400.0,
            evaporate: 11.0,
            initial_spread: 0.52,
            velocity_spread: 0.34,
            shrink_rate: 0.014,
            trail_density: 0.20,
            trail_size: 0.42,
            trail_spread: 0.58,
            streak_boost: 0.72,
            streak_length: 1.15,
            streak_taper: 0.68,
            streak_evap: 18.0,
            micro_drops: 620.0,
            micro_min: 8.0,
            micro_max: 27.0,
            background_blur_steps: 2,
            mist_enabled: true,
            mist_blur_step: 4,
            mist_time: 16.0,
            mist_strength: 0.012,
            edge_soft_a: 0.945,
            edge_soft_b: 0.992,
            refract_base: 0.34,
            refract_scale: 0.76,
            shadow_offset: 0.76,
            diffuse: 0.22,
            specular: 0.025,
            shininess: 300.0,
            light_x: -1.0,
            light_y: 1.0,
            light_z: 2.0,
            light_bump: 0.78,
        }
    }

    fn html_cinematic_button() -> Self {
        Self {
            spawn_rate: 6.0,
            spawn_limit: 650,
            spawn_min: 52.0,
            spawn_max: 130.0,
            slip_rate: 0.18,
            gravity: 2400.0,
            evaporate: 10.0,
            initial_spread: 0.50,
            velocity_spread: 0.28,
            shrink_rate: 0.01,
            trail_density: 0.18,
            trail_size: 0.38,
            trail_spread: 0.56,
            streak_boost: 0.55,
            streak_length: 1.05,
            streak_taper: 0.72,
            streak_evap: 19.0,
            micro_drops: 420.0,
            micro_min: 8.0,
            micro_max: 24.0,
            background_blur_steps: 2,
            mist_enabled: true,
            mist_blur_step: 4,
            mist_time: 10.0,
            mist_strength: 0.010,
            edge_soft_a: 0.96,
            edge_soft_b: 0.99,
            refract_base: 0.32,
            refract_scale: 0.70,
            shadow_offset: 0.80,
            diffuse: 0.20,
            specular: 0.0,
            shininess: 256.0,
            light_x: -1.0,
            light_y: 1.0,
            light_z: 2.0,
            light_bump: 1.0,
        }
    }

    fn html_storm_button() -> Self {
        Self {
            spawn_rate: 28.0,
            spawn_limit: 1500,
            spawn_min: 34.0,
            spawn_max: 155.0,
            slip_rate: 0.78,
            gravity: 3150.0,
            evaporate: 6.0,
            initial_spread: 0.72,
            velocity_spread: 0.48,
            shrink_rate: 0.010,
            trail_density: 0.30,
            trail_size: 0.54,
            trail_spread: 0.98,
            streak_boost: 0.96,
            streak_length: 1.78,
            streak_taper: 0.86,
            streak_evap: 12.0,
            micro_drops: 1250.0,
            micro_min: 7.0,
            micro_max: 34.0,
            background_blur_steps: 3,
            mist_enabled: true,
            mist_blur_step: 5,
            mist_time: 12.0,
            mist_strength: 0.018,
            edge_soft_a: 0.945,
            edge_soft_b: 0.992,
            refract_base: 0.48,
            refract_scale: 0.94,
            shadow_offset: 0.82,
            diffuse: 0.28,
            specular: 0.030,
            shininess: 300.0,
            light_x: -1.0,
            light_y: 1.0,
            light_z: 2.0,
            light_bump: 0.95,
        }
    }
}

fn apply_reference_controls(rain: &mut RainGlass2d, controls: RainGlassReferenceControls) {
    rain.enabled = true;
    rain.reference_mode = true;
    rain.spawn_rate = controls.spawn_rate;
    rain.spawn_limit = controls.spawn_limit;
    rain.min_radius_px = controls.spawn_min;
    rain.max_radius_px = controls.spawn_max;
    rain.slip_rate = controls.slip_rate;
    rain.gravity_px_per_sec2 = controls.gravity;
    rain.evaporate = controls.evaporate;
    rain.initial_spread = controls.initial_spread;
    rain.velocity_spread = controls.velocity_spread;
    rain.shrink_rate = controls.shrink_rate;
    rain.motion_interval_min = 0.08;
    rain.motion_interval_max = 0.32;
    rain.x_shift_min = 0.0;
    rain.x_shift_max = 0.12;
    rain.collider_scale = 1.0;

    rain.trails_enabled = true;
    rain.streak_boost = controls.streak_boost;
    rain.streak_length = controls.streak_length;
    rain.trail_drop_density = controls.trail_density * (1.0 + controls.streak_boost * 0.85);
    rain.trail_drop_size_min = (controls.trail_size * 0.58).max(0.05);
    rain.trail_drop_size_max = (controls.trail_size * 1.22).max(0.06);
    rain.trail_spread =
        controls.trail_spread * (1.0 + controls.streak_boost * controls.streak_length);
    rain.trail_distance_min_px = (19.0 - controls.streak_boost * 12.0).max(5.0);
    rain.trail_distance_max_px = (34.0 - controls.streak_boost * 17.0).max(9.0);
    rain.trail_taper = controls.streak_taper;
    rain.trail_evaporate = controls.streak_evap;
    rain.trail_shrink_rate = (0.35 + controls.shrink_rate).clamp(0.001, 1.0);
    rain.trail_refract_scale = 1.0;
    rain.trail_opacity = 1.0;

    rain.micro_droplets_enabled = true;
    rain.micro_droplets_per_second = controls.micro_drops;
    rain.micro_droplet_min_px = controls.micro_min;
    rain.micro_droplet_max_px = controls.micro_max;

    rain.mist_enabled = controls.mist_enabled;
    rain.mist_opacity = if controls.mist_enabled { 1.0 } else { 0.0 };
    rain.mist_blur_px = controls.mist_blur_step as f32;
    rain.mist_blur_step = controls.mist_blur_step;
    rain.mist_time = controls.mist_time;
    rain.mist_color_strength = controls.mist_strength;
    rain.mist_accumulation = controls.mist_strength;

    rain.background_blur_px = controls.background_blur_steps as f32;
    rain.background_blur_steps = controls.background_blur_steps;
    rain.smooth_edge_min = controls.edge_soft_a;
    rain.smooth_edge_max = controls.edge_soft_b;
    rain.refract_base = controls.refract_base;
    rain.refract_scale = controls.refract_scale;
    rain.opacity = 1.0;
    rain.body_opacity = 1.0;
    rain.scene_blend = 1.0;
    rain.chromatic_aberration = 0.0;
    rain.distortion_px = 28.0;
    rain.normal_strength = 6.0;
    rain.focus_blur_strength = 0.85;
    rain.raindrop_compose = RainGlassRaindropCompose::Smoother;
    rain.raindrop_eraser_size = [0.93, 1.0];

    rain.shadow_offset = controls.shadow_offset;
    rain.diffuse_light = [controls.diffuse; 3];
    rain.specular_light = [controls.specular; 3];
    rain.specular_shininess = controls.shininess;
    rain.light_pos = [controls.light_x, controls.light_y, controls.light_z, 0.0];
    rain.light_bump = controls.light_bump;
    rain.scene_light_response = 0.0;
    rain.rim_strength = 0.0;
    rain.debug_view = RainGlassDebugView::Final;
}

fn apply_rain_glass_preset(rain: &mut RainGlass2d, value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" => {
            rain.enabled = true;
            rain.reference_mode = false;
            rain.spawn_rate = 2.0;
            rain.spawn_limit = 40;
            rain.min_radius_px = 32.0;
            rain.max_radius_px = 96.0;
            rain.micro_droplets_enabled = false;
            rain.micro_droplets_per_second = 0.0;
            rain.mist_enabled = false;
            rain.mist_opacity = 0.0;
            rain.opacity = 0.75;
            rain.refract_scale = 1.6;
            rain.background_blur_px = 8.0;
            rain.debug_view = RainGlassDebugView::Final;
        }
        "cinematic" | "html_current_controls" => {
            apply_reference_controls(rain, RainGlassReferenceControls::html_current_controls());
        }
        "storm" | "html_storm" | "html_storm_button" => {
            apply_reference_controls(rain, RainGlassReferenceControls::html_storm_button());
        }
        "lens_streaks" => {
            rain.enabled = true;
            rain.reference_mode = false;
            rain.spawn_rate = 5.0;
            rain.spawn_limit = 360;
            rain.min_radius_px = 24.0;
            rain.max_radius_px = 88.0;
            rain.trails_enabled = true;
            rain.trail_distance_min_px = 8.0;
            rain.trail_distance_max_px = 18.0;
            rain.trail_taper = 0.72;
            rain.trail_spread = 1.15;
            rain.trail_evaporate = 26.0;
            rain.trail_shrink_rate = 0.94;
            rain.distortion_px = 36.0;
            rain.normal_strength = 7.2;
            rain.focus_blur_strength = 0.85;
            rain.body_opacity = 0.58;
            rain.trail_refract_scale = 0.62;
            rain.trail_opacity = 0.78;
            rain.scene_light_response = 1.55;
            rain.rim_strength = 1.18;
            rain.mist_enabled = false;
            rain.mist_opacity = 0.0;
            rain.debug_view = RainGlassDebugView::Final;
        }
        "subtle" => {
            rain.enabled = true;
            rain.reference_mode = false;
            rain.spawn_rate = 3.0;
            rain.spawn_limit = 130;
            rain.min_radius_px = 10.0;
            rain.max_radius_px = 44.0;
            rain.micro_droplets_enabled = true;
            rain.micro_droplets_per_second = 90.0;
            rain.mist_enabled = true;
            rain.mist_opacity = 0.08;
            rain.opacity = 0.58;
            rain.refract_scale = 0.62;
            rain.background_blur_px = 2.0;
            rain.debug_view = RainGlassDebugView::Final;
        }
        "optics_debug" => {
            rain.enabled = true;
            rain.reference_mode = false;
            rain.spawn_rate = 1.0;
            rain.spawn_limit = 12;
            rain.min_radius_px = 72.0;
            rain.max_radius_px = 140.0;
            rain.micro_droplets_enabled = false;
            rain.micro_droplets_per_second = 0.0;
            rain.mist_enabled = false;
            rain.mist_opacity = 0.0;
            rain.mist_accumulation = 0.0;
            rain.trails_enabled = true;
            rain.trail_distance_min_px = 8.0;
            rain.trail_distance_max_px = 16.0;
            rain.trail_spread = 1.35;
            rain.trail_taper = 0.78;
            rain.trail_shrink_rate = 0.92;
            rain.trail_evaporate = 28.0;
            rain.trail_opacity = 0.85;
            rain.trail_refract_scale = 0.72;
            rain.opacity = 1.0;
            rain.refract_base = 0.80;
            rain.refract_scale = 3.0;
            rain.distortion_px = 48.0;
            rain.normal_strength = 8.0;
            rain.background_blur_px = 12.0;
            rain.focus_blur_strength = 1.0;
            rain.body_opacity = 0.95;
            rain.scene_light_response = 2.2;
            rain.rim_strength = 1.6;
            rain.diffuse_light = [0.01, 0.012, 0.015];
            rain.specular_light = [0.02, 0.025, 0.03];
            rain.shadow_offset = 0.58;
            rain.light_bump = 1.2;
            rain.chromatic_aberration = 0.22;
            rain.debug_view = RainGlassDebugView::Final;
        }
        "reference_cinematic" | "html_cinematic" | "html_cinematic_button" => {
            apply_reference_controls(rain, RainGlassReferenceControls::html_cinematic_button());
        }
        other => {
            return Err(format!(
                "unknown rain_glass preset `{other}` (expected: debug/cinematic/html_current_controls/storm/html_storm_button/lens_streaks/subtle/optics_debug/reference_cinematic/html_cinematic/html_cinematic_button)"
            ));
        }
    }
    Ok(())
}
