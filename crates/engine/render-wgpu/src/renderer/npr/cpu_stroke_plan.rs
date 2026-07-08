use amigo_math::ColorRgba;

use crate::renderer::{
    NprCachedStrokePlan, NprDropoutInterval, NprDropoutMask, NprLineKind, NprStrokeGesture,
    NprStrokePassKind, NprStrokePassPlan, NprStrokePath, npr_stroke_plan_length_bucket,
    npr_stroke_plan_settings_signature,
};

use super::{
    NprLineCandidateTraits, deterministic_noise, deterministic_signed_noise,
    npr_cpu_break_policy_profile, npr_cpu_stroke_synthesis_profile,
    resolve_npr_brush_profile_with_traits,
};

pub(crate) fn build_npr_stroke_pass_plan(
    path: &NprStrokePath,
    settings: &amigo_render_api::NprLineSettings3d,
    gesture: NprStrokeGesture,
) -> Vec<NprStrokePassPlan> {
    let primary_passes = settings.passes.min(8);
    let brush = resolve_npr_brush_profile_with_traits(
        path.kind,
        NprLineCandidateTraits {
            technical_detail: path.technical_detail,
            material_detail: path.material_detail,
            material_seam: path.material_seam,
        },
        settings,
    );
    let mut passes =
        Vec::with_capacity(primary_passes as usize + settings.search_line_count as usize + 1);
    let synthesis = npr_cpu_stroke_synthesis_profile(settings);
    let brush_ink_color = brush.ink_color.unwrap_or(settings.ink_color);

    for pass in 0..primary_passes {
        passes.push(NprStrokePassPlan {
            kind: NprStrokePassKind::Primary,
            pass_index: pass,
            active_t0: 0.0,
            active_t1: 1.0,
            wobble_px: gesture.dynamics.base_wobble_px
                * npr_pass_jitter_multiplier(primary_passes, pass, synthesis),
            width_multiplier: npr_pass_width_multiplier(primary_passes, pass, synthesis),
            color: npr_pass_color(
                brush_ink_color,
                primary_passes,
                pass,
                gesture.style.alpha_multiplier * gesture.role.alpha_multiplier,
                synthesis,
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
            wobble_px: gesture.dynamics.base_wobble_px * synthesis.search_wobble_multiplier,
            width_multiplier: synthesis.search_width_multiplier,
            color: ColorRgba::new(
                brush_ink_color.r,
                brush_ink_color.g,
                brush_ink_color.b,
                (brush_ink_color.a * settings.search_line_alpha * brush.alpha_multiplier)
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
        || matches!(
            settings.pipeline.stroke_strategy,
            amigo_render_api::NprStrokeStrategy3d::AkiraInk
                | amigo_render_api::NprStrokeStrategy3d::ConfidentMangaInk
        )
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
    let synthesis = npr_cpu_stroke_synthesis_profile(settings);
    if gesture.path_length_px < synthesis.hatch_path_length_min_px.max(0.0)
        || gesture.path_length_px
            > synthesis
                .hatch_path_length_max_px
                .max(synthesis.hatch_path_length_min_px)
    {
        return None;
    }

    let roll = deterministic_noise(settings.seed, gesture.path_seed, 37, 0);
    let chance = match settings.pipeline.stroke_strategy {
        amigo_render_api::NprStrokeStrategy3d::AkiraInk => synthesis.hatch_chance_akira,
        amigo_render_api::NprStrokeStrategy3d::ConfidentMangaInk => {
            synthesis.hatch_chance_confident_manga
        }
        _ => synthesis.hatch_chance_generic,
    };
    if roll >= chance.clamp(0.0, 1.0) {
        return None;
    }

    let center = (synthesis.hatch_center_t
        + deterministic_signed_noise(settings.seed, gesture.path_seed, 41, 0)
            * synthesis.hatch_center_jitter)
        .clamp(0.25, 0.75);
    let hatch_length_px = synthesis.hatch_length_min_px.max(0.0)
        + deterministic_noise(settings.seed, gesture.path_seed, 43, 0)
            * synthesis.hatch_length_jitter_px.max(0.0);
    let half_t = (hatch_length_px * 0.5 / gesture.path_length_px.max(1.0)).clamp(
        synthesis.hatch_half_t_min.clamp(0.0, 1.0),
        synthesis
            .hatch_half_t_max
            .max(synthesis.hatch_half_t_min)
            .clamp(0.0, 1.0),
    );
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
        wobble_px: gesture.dynamics.base_wobble_px * synthesis.hatch_wobble_multiplier,
        width_multiplier: synthesis.hatch_width_multiplier,
        color: ColorRgba::new(
            settings.ink_color.r,
            settings.ink_color.g,
            settings.ink_color.b,
            (settings.ink_color.a
                * gesture.style.alpha_multiplier
                * gesture.role.alpha_multiplier
                * synthesis.hatch_alpha_multiplier)
                .clamp(0.0, synthesis.hatch_alpha_max.clamp(0.0, 1.0)),
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
        let brush = resolve_npr_brush_profile_with_traits(
            gesture.kind,
            NprLineCandidateTraits {
                technical_detail: gesture.technical_detail,
                material_detail: gesture.material_detail,
                material_seam: gesture.material_seam,
            },
            settings,
        );
        let break_policy = npr_cpu_break_policy_profile(settings);
        let complexity_multiplier = (1.0
            - (gesture
                .dynamics
                .edge_complexity
                .min(break_policy.dropout_complexity_edge_limit.max(1.0))
                - 1.0)
                * break_policy.dropout_complexity_drop_per_edge.max(0.0))
        .max(0.0);
        let effective_dropout =
            (gesture.style.dropout * brush.dropout_multiplier * complexity_multiplier)
                .clamp(0.0, break_policy.dropout_effective_max.clamp(0.0, 1.0));
        let path_length = gesture.path_length_px.max(1.0);
        let interval_count = (effective_dropout * path_length
            / break_policy.dropout_interval_length_px.max(1.0))
        .ceil() as usize;
        let interval_count = interval_count.min(break_policy.dropout_max_intervals as usize);
        let min_gap_t = (settings.dropout_segment_min_px.max(1.0) / path_length).clamp(
            break_policy.dropout_min_gap_t.clamp(0.0, 1.0),
            break_policy.dropout_max_gap_t.clamp(0.0, 1.0),
        );
        let edge_margin_t = break_policy.dropout_edge_margin_t.clamp(0.0, 0.45);

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
                    .clamp(
                        break_policy.dropout_min_gap_t.clamp(0.0, 1.0),
                        break_policy.dropout_max_gap_t.clamp(0.0, 1.0),
                    );
                let t0 = (center - width * 0.5).clamp(edge_margin_t, 1.0 - edge_margin_t);
                let t1 = (center + width * 0.5).clamp(edge_margin_t, 1.0 - edge_margin_t);
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

    if npr_allows_seeded_long_feature_break(gesture, settings) {
        let break_policy = npr_cpu_break_policy_profile(settings);
        for pass in passes
            .iter()
            .copied()
            .filter(|pass| pass.kind == NprStrokePassKind::Primary)
        {
            let roll = deterministic_noise(
                settings.seed,
                gesture.path_seed,
                pass.pass_index as u64,
                1217,
            );
            if roll > break_policy.long_feature_break_chance.clamp(0.0, 1.0) {
                continue;
            }
            let path_length = gesture.path_length_px.max(1.0);
            let center = (break_policy.long_feature_break_center_t
                + deterministic_signed_noise(
                    settings.seed,
                    gesture.path_seed,
                    pass.pass_index as u64,
                    1223,
                ) * break_policy.long_feature_break_center_jitter)
                .clamp(
                    break_policy.long_feature_break_center_min_t.clamp(0.0, 1.0),
                    break_policy
                        .long_feature_break_center_max_t
                        .max(break_policy.long_feature_break_center_min_t)
                        .clamp(0.0, 1.0),
                );
            let gap_px = break_policy.long_feature_break_min_gap_px
                + deterministic_noise(
                    settings.seed,
                    gesture.path_seed,
                    pass.pass_index as u64,
                    1229,
                ) * break_policy.long_feature_break_gap_jitter_px;
            let half_t = (gap_px * 0.5 / path_length).clamp(
                break_policy.long_feature_break_half_t_min.clamp(0.0, 1.0),
                break_policy
                    .long_feature_break_half_t_max
                    .max(break_policy.long_feature_break_half_t_min)
                    .clamp(0.0, 1.0),
            );
            let t0 = (center - half_t).clamp(
                break_policy.long_feature_break_t0_min.clamp(0.0, 1.0),
                break_policy
                    .long_feature_break_t0_max
                    .max(break_policy.long_feature_break_t0_min)
                    .clamp(0.0, 1.0),
            );
            let t1 = (center + half_t).clamp(
                break_policy.long_feature_break_t1_min.clamp(0.0, 1.0),
                break_policy
                    .long_feature_break_t1_max
                    .max(break_policy.long_feature_break_t1_min)
                    .clamp(0.0, 1.0),
            );
            if t1 > t0 {
                intervals.push(NprDropoutInterval {
                    pass_index: pass.pass_index,
                    t0,
                    t1,
                });
            }
        }
    }

    NprDropoutMask { intervals }
}

fn npr_allows_seeded_long_feature_break(
    gesture: NprStrokeGesture,
    settings: &amigo_render_api::NprLineSettings3d,
) -> bool {
    if !matches!(
        settings.pipeline.stroke_strategy,
        amigo_render_api::NprStrokeStrategy3d::ConfidentMangaInk
            | amigo_render_api::NprStrokeStrategy3d::AkiraInk
    ) {
        return false;
    }
    if !matches!(
        gesture.kind,
        NprLineKind::Feature | NprLineKind::Crease | NprLineKind::Seam
    ) {
        return false;
    }
    let profile = npr_cpu_break_policy_profile(settings);
    if !profile.allow_seeded_long_feature_breaks {
        return false;
    }
    if matches!(gesture.kind, NprLineKind::Feature | NprLineKind::Crease)
        && gesture.importance >= profile.important_feature_break_threshold
    {
        return false;
    }
    if gesture.path_length_px < profile.long_feature_break_min_length_px {
        return false;
    }
    if gesture.dynamics.edge_complexity < profile.long_feature_break_min_complexity {
        return false;
    }
    !gesture.material_detail
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

fn npr_pass_jitter_multiplier(
    passes: u8,
    pass: u8,
    profile: amigo_render_api::NprStrokeSynthesisProfile3d,
) -> f32 {
    if passes >= 3 {
        profile.multi_pass_jitter_base + pass as f32 * profile.multi_pass_jitter_step
    } else if passes == 2 {
        if pass == 0 {
            profile.dual_primary_jitter_multiplier
        } else {
            profile.dual_secondary_jitter_multiplier
        }
    } else {
        profile.single_pass_jitter_multiplier
    }
}

fn npr_pass_width_multiplier(
    passes: u8,
    pass: u8,
    profile: amigo_render_api::NprStrokeSynthesisProfile3d,
) -> f32 {
    if passes >= 3 {
        profile.multi_pass_width_multiplier
    } else if passes == 2 {
        if pass == 0 {
            profile.dual_primary_width_multiplier
        } else {
            profile.dual_secondary_width_multiplier
        }
    } else {
        profile.single_pass_width_multiplier
    }
}

fn npr_pass_color(
    color: ColorRgba,
    passes: u8,
    pass: u8,
    alpha_multiplier: f32,
    profile: amigo_render_api::NprStrokeSynthesisProfile3d,
) -> ColorRgba {
    let alpha = if passes >= 3 {
        profile.multi_pass_alpha
    } else if passes == 2 {
        if pass == 0 {
            profile.dual_primary_alpha
        } else {
            profile.dual_secondary_alpha
        }
    } else {
        profile.single_pass_alpha
    };
    ColorRgba::new(
        color.r,
        color.g,
        color.b,
        (color.a * alpha * alpha_multiplier).clamp(0.0, 1.0),
    )
}
