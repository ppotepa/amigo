use crate::{Crt2d, DirtyBloom2d, PostFx2d, PostFx2dService, PostFx2dStack, PostFxBlur2d};

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
            PostFxDevConsoleCommandOutcome::Handled(format!(
                "postfx.effects={} dirty_bloom_active={} crt_active={} film_noise_active={} lens_droplets_active={} wet_reflections_active={} renderer_mode={} overlay_supported={} blur_supported={} world_offscreen_post_fx_supported={}",
                stack.effects.len(),
                dirty_bloom_active,
                crt_active,
                film_noise_active,
                lens_active,
                wet_active,
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
