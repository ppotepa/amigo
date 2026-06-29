use amigo_math::ColorRgba;

use crate::renderer::{
    NprCachedStrokePlan, NprDropoutInterval, NprDropoutMask, NprLineKind, NprStrokeGesture,
    NprStrokePassKind, NprStrokePassPlan, NprStrokePath, npr_stroke_plan_length_bucket,
    npr_stroke_plan_settings_signature,
};

use super::{deterministic_noise, deterministic_signed_noise, resolve_npr_brush_profile};

pub(crate) fn build_npr_stroke_pass_plan(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
) -> Vec<NprStrokePassPlan> {
    let primary_passes = settings.passes.min(8);
    let brush = resolve_npr_brush_profile(settings);
    let mut passes =
        Vec::with_capacity(primary_passes as usize + settings.search_line_count as usize + 1);

    for pass in 0..primary_passes {
        passes.push(NprStrokePassPlan {
            kind: NprStrokePassKind::Primary,
            pass_index: pass,
            active_t0: 0.0,
            active_t1: 1.0,
            wobble_px: gesture.dynamics.base_wobble_px
                * npr_pass_jitter_multiplier(primary_passes, pass),
            width_multiplier: npr_pass_width_multiplier(primary_passes, pass),
            color: npr_pass_color(
                settings.ink_color,
                primary_passes,
                pass,
                gesture.style.alpha_multiplier,
            ),
            overshoot_px: gesture.dynamics.effective_overshoot_px,
        });
    }

    let search_count = if path.kind == NprLineKind::Silhouette {
        0
    } else {
        ((settings.search_line_count as f32) * brush.search_multiplier)
            .round()
            .clamp(0.0, 8.0) as u8
    };
    for search_pass in 0..search_count {
        passes.push(NprStrokePassPlan {
            kind: NprStrokePassKind::Search,
            pass_index: primary_passes.saturating_add(search_pass),
            active_t0: 0.0,
            active_t1: 1.0,
            wobble_px: gesture.dynamics.base_wobble_px * 1.18,
            width_multiplier: 0.78,
            color: ColorRgba::new(
                settings.ink_color.r,
                settings.ink_color.g,
                settings.ink_color.b,
                (settings.ink_color.a * settings.search_line_alpha * brush.alpha_multiplier)
                    .clamp(0.0, 1.0),
            ),
            overshoot_px: gesture
                .dynamics
                .effective_overshoot_px
                .max(settings.undershoot_px),
        });
    }

    if let Some(hatch_pass) =
        build_npr_sparse_character_hatch_pass(path, settings, gesture, primary_passes, search_count)
    {
        passes.push(hatch_pass);
    }

    passes
}

fn build_npr_sparse_character_hatch_pass(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
    primary_passes: u8,
    search_count: u8,
) -> Option<NprStrokePassPlan> {
    if settings.pipeline.hatching_strategy
        != amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching
    {
        return None;
    }
    if !(settings.pipeline.candidate_strategy
        == amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
        || settings.pipeline.stroke_strategy == amigo_render_api::NprStrokeStrategy3d::AkiraInk
        || matches!(
            settings.pipeline.budget_strategy,
            amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority
                | amigo_render_api::NprBudgetStrategy3d::CharacterReadability
        ))
    {
        return None;
    }
    if !matches!(
        path.kind,
        NprLineKind::Crease | NprLineKind::Seam | NprLineKind::Feature
    ) {
        return None;
    }
    if !(8.0..=44.0).contains(&gesture.path_length_px) {
        return None;
    }

    let roll = deterministic_noise(settings.seed, gesture.path_seed, 37, 0);
    let chance =
        if settings.pipeline.stroke_strategy == amigo_render_api::NprStrokeStrategy3d::AkiraInk {
            0.30
        } else {
            0.18
        };
    if roll >= chance {
        return None;
    }

    let center = (0.42
        + deterministic_signed_noise(settings.seed, gesture.path_seed, 41, 0) * 0.18)
        .clamp(0.25, 0.75);
    let hatch_length_px = 7.0 + deterministic_noise(settings.seed, gesture.path_seed, 43, 0) * 9.0;
    let half_t = (hatch_length_px * 0.5 / gesture.path_length_px.max(1.0)).clamp(0.04, 0.28);
    let active_t0 = (center - half_t).clamp(0.0, 1.0);
    let active_t1 = (center + half_t).clamp(active_t0, 1.0);
    if active_t1 - active_t0 <= 0.02 {
        return None;
    }

    let pass_index = primary_passes.saturating_add(search_count);
    Some(NprStrokePassPlan {
        kind: NprStrokePassKind::Hatch,
        pass_index,
        active_t0,
        active_t1,
        wobble_px: gesture.dynamics.base_wobble_px * 0.55,
        width_multiplier: 0.24,
        color: ColorRgba::new(
            settings.ink_color.r,
            settings.ink_color.g,
            settings.ink_color.b,
            (settings.ink_color.a * gesture.style.alpha_multiplier * 0.38).clamp(0.0, 0.58),
        ),
        overshoot_px: 0.0,
    })
}

pub(crate) fn build_npr_dropout_mask(
    gesture: NprStrokeGesture,
    settings: &amigo_render_api::NprLineSettings3d,
    passes: &[NprStrokePassPlan],
) -> NprDropoutMask {
    let mut intervals = Vec::new();
    if !gesture.dynamics.protected_silhouette && gesture.style.dropout > 0.0 {
        let brush = resolve_npr_brush_profile(settings);
        let complexity_multiplier =
            (1.0 - (gesture.dynamics.edge_complexity.min(12.0) - 1.0) * 0.01).max(0.0);
        let effective_dropout =
            (gesture.style.dropout * brush.dropout_multiplier * complexity_multiplier)
                .clamp(0.0, 0.85);
        let path_length = gesture.path_length_px.max(1.0);
        let interval_count = (effective_dropout * path_length / 64.0).ceil() as usize;
        let interval_count = interval_count.min(8);
        let min_gap_t = (settings.dropout_segment_min_px.max(1.0) / path_length).clamp(0.01, 0.25);

        for pass in passes
            .iter()
            .copied()
            .filter(|pass| pass.kind == NprStrokePassKind::Primary)
        {
            for interval_index in 0..interval_count {
                let center = deterministic_noise(
                    settings.seed,
                    gesture.path_seed,
                    pass.pass_index as u64,
                    751 + interval_index as u64,
                );
                let width = (min_gap_t
                    + deterministic_noise(
                        settings.seed,
                        gesture.path_seed,
                        pass.pass_index as u64,
                        811 + interval_index as u64,
                    ) * min_gap_t)
                    .clamp(0.01, 0.25);
                let t0 = (center - width * 0.5).clamp(0.08, 0.92);
                let t1 = (center + width * 0.5).clamp(0.08, 0.92);
                if t1 > t0 {
                    intervals.push(NprDropoutInterval {
                        pass_index: pass.pass_index,
                        t0,
                        t1,
                    });
                }
            }
        }
    }

    NprDropoutMask { intervals }
}

pub(crate) fn build_npr_cached_stroke_plan(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
) -> NprCachedStrokePlan {
    let passes = build_npr_stroke_pass_plan(path, settings, gesture);
    let dropout = build_npr_dropout_mask(gesture, settings, &passes);
    NprCachedStrokePlan {
        settings_signature: npr_stroke_plan_settings_signature(settings),
        length_bucket_px: npr_stroke_plan_length_bucket(gesture.path_length_px),
        passes,
        dropout,
    }
}

pub(crate) fn build_empty_npr_cached_stroke_plan(
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprCachedStrokePlan {
    NprCachedStrokePlan {
        settings_signature: npr_stroke_plan_settings_signature(settings),
        length_bucket_px: 0,
        passes: Vec::new(),
        dropout: NprDropoutMask {
            intervals: Vec::new(),
        },
    }
}

fn npr_pass_jitter_multiplier(passes: u8, pass: u8) -> f32 {
    if passes >= 3 {
        1.0 + pass as f32 * 0.55
    } else if passes == 2 {
        if pass == 0 { 1.1 } else { 0.35 }
    } else {
        0.35
    }
}

fn npr_pass_width_multiplier(passes: u8, pass: u8) -> f32 {
    if passes >= 3 {
        0.9
    } else if passes == 2 {
        if pass == 0 { 1.6 } else { 0.85 }
    } else {
        0.75
    }
}

fn npr_pass_color(color: ColorRgba, passes: u8, pass: u8, alpha_multiplier: f32) -> ColorRgba {
    let alpha = if passes >= 3 {
        0.18
    } else if passes == 2 {
        if pass == 0 { 0.28 } else { 0.75 }
    } else {
        0.92
    };
    ColorRgba::new(
        color.r,
        color.g,
        color.b,
        (color.a * alpha * alpha_multiplier).clamp(0.0, 1.0),
    )
}
