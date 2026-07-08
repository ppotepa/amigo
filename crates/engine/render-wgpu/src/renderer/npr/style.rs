#[derive(Debug, Clone, Copy)]
pub(crate) struct NprResolvedKindStyle {
    pub(crate) width_multiplier: f32,
    pub(crate) wobble_px: f32,
    pub(crate) dropout: f32,
    pub(crate) taper: f32,
    pub(crate) overshoot_px: f32,
    pub(crate) alpha_multiplier: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprGestureRoleProfile {
    pub(crate) hand_arc_multiplier: f32,
    pub(crate) tangent_drift_multiplier: f32,
    pub(crate) detail_crispness: f32,
    pub(crate) taper_multiplier: f32,
    pub(crate) overshoot_multiplier: f32,
    pub(crate) alpha_multiplier: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprResolvedBrushProfile3d {
    pub(crate) tip: amigo_render_api::NprBrushTip3d,
    pub(crate) ink_color: Option<amigo_math::ColorRgba>,
    pub(crate) width_multiplier: f32,
    pub(crate) alpha_multiplier: f32,
    pub(crate) pressure_jitter_multiplier: f32,
    pub(crate) dropout_multiplier: f32,
    pub(crate) search_multiplier: f32,
    pub(crate) path_wobble_multiplier: f32,
    pub(crate) micro_wobble_multiplier: f32,
    pub(crate) hand_arc_multiplier: f32,
    pub(crate) tangent_drift_multiplier: f32,
    pub(crate) detail_crispness_multiplier: f32,
    pub(crate) taper_multiplier: f32,
    pub(crate) width_curve: [f32; 4],
    pub(crate) alpha_curve: [f32; 4],
    pub(crate) overshoot_px: Option<f32>,
    pub(crate) angle_bias_radians: f32,
    pub(crate) angle_influence: f32,
    pub(crate) nib_width_base_scale: f32,
    pub(crate) nib_width_angle_scale: f32,
    pub(crate) path_adherence_multiplier: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NprLineCandidateTraits {
    pub(crate) technical_detail: bool,
    pub(crate) material_detail: bool,
    pub(crate) material_seam: bool,
}

pub(crate) fn resolve_npr_brush_profile(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprResolvedBrushProfile3d {
    resolve_npr_brush_profile_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn resolve_npr_brush_profile_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprResolvedBrushProfile3d {
    let family = resolve_npr_line_family_with_traits(kind, traits, settings);
    let family_brush = family
        .and_then(|family| family.brush.as_ref())
        .and_then(|id| settings.brush_profiles.get(id));
    let tool = family_brush
        .and_then(|brush| brush.tool)
        .unwrap_or(settings.stroke_tool);
    let tip = family_brush
        .and_then(|brush| brush.tip)
        .unwrap_or_else(|| default_npr_brush_tip(tool));
    let (
        width,
        alpha,
        pressure_jitter,
        dropout,
        search,
        path_wobble,
        micro_wobble,
        hand_arc,
        tangent_drift,
        detail_crispness,
    ): (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) = match tool {
        amigo_render_api::NprStrokeTool3d::InkPen => {
            (1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
        }
        amigo_render_api::NprStrokeTool3d::Pencil => {
            (0.84, 0.72, 1.65, 2.35, 1.65, 1.22, 1.55, 1.18, 1.08, 0.92)
        }
        amigo_render_api::NprStrokeTool3d::Brush => {
            (1.18, 0.96, 1.42, 1.65, 0.72, 1.08, 1.20, 1.26, 0.88, 0.86)
        }
        amigo_render_api::NprStrokeTool3d::Marker => {
            (1.08, 0.84, 0.58, 0.42, 0.35, 0.82, 0.65, 0.78, 0.72, 1.04)
        }
        amigo_render_api::NprStrokeTool3d::TechnicalPen => {
            (0.92, 1.0, 0.08, 0.0, 0.0, 0.22, 0.20, 0.45, 0.42, 1.14)
        }
    };
    let straightness = settings.straightness.clamp(0.0, 1.0);
    let path_adherence_multiplier = family_brush
        .map(|brush| brush.path_adherence_multiplier)
        .unwrap_or(1.0)
        .clamp(0.1, 2.0);
    let effective_straightness = (straightness * path_adherence_multiplier).clamp(0.0, 1.0);
    let family_path_wobble = family_brush
        .map(|brush| brush.path_wobble_multiplier)
        .unwrap_or(1.0)
        .max(0.0);
    let path_wobble_multiplier = ((1.0 - effective_straightness)
        * 1.55
        * path_wobble
        * family_path_wobble
        * settings.tool_wobble_multiplier.max(0.0))
    .clamp(0.0, 2.5);

    NprResolvedBrushProfile3d {
        tip,
        ink_color: family_brush.and_then(|brush| brush.ink_color.clone()),
        width_multiplier: (width
            * family_brush
                .map(|brush| brush.width_multiplier)
                .unwrap_or(1.0)
            * settings.tool_width_multiplier.max(0.0))
        .clamp(0.05, 4.0),
        alpha_multiplier: (alpha
            * family_brush
                .map(|brush| brush.alpha_multiplier)
                .unwrap_or(1.0)
            * settings.tool_alpha_multiplier.max(0.0))
        .clamp(0.0, 2.0),
        pressure_jitter_multiplier: (pressure_jitter
            * family_brush
                .map(|brush| brush.pressure_jitter_multiplier)
                .unwrap_or(1.0)
            * settings.tool_pressure_jitter_multiplier.max(0.0))
        .clamp(0.0, 4.0),
        dropout_multiplier: (dropout
            * family_brush
                .map(|brush| brush.dropout_multiplier)
                .unwrap_or(1.0)
            * settings.tool_dropout_multiplier.max(0.0))
        .clamp(0.0, 5.0),
        search_multiplier: (search
            * family_brush
                .map(|brush| brush.search_multiplier)
                .unwrap_or(1.0)
            * settings.tool_search_multiplier.max(0.0))
        .clamp(0.0, 5.0),
        path_wobble_multiplier,
        micro_wobble_multiplier: (micro_wobble
            * family_brush
                .map(|brush| brush.micro_wobble_multiplier)
                .unwrap_or(1.0)
            * settings.tool_wobble_multiplier.max(0.0))
        .clamp(0.0, 3.0),
        hand_arc_multiplier: (hand_arc
            * family_brush
                .map(|brush| brush.hand_arc_multiplier)
                .unwrap_or(1.0))
        .clamp(0.25f32, 2.0f32),
        tangent_drift_multiplier: (tangent_drift
            * family_brush
                .map(|brush| brush.tangent_drift_multiplier)
                .unwrap_or(1.0))
        .clamp(0.25f32, 2.0f32),
        detail_crispness_multiplier: (detail_crispness
            * family_brush
                .map(|brush| brush.detail_crispness_multiplier)
                .unwrap_or(1.0))
        .clamp(0.5f32, 1.5f32),
        taper_multiplier: family_brush
            .map(|brush| brush.taper_multiplier)
            .unwrap_or(1.0)
            .clamp(0.25, 2.0),
        width_curve: family_brush
            .map(|brush| brush.width_curve)
            .unwrap_or(settings.width_pressure_curve),
        alpha_curve: family_brush
            .map(|brush| brush.alpha_curve)
            .unwrap_or(settings.alpha_pressure_curve),
        overshoot_px: family_brush.and_then(|brush| brush.overshoot_px),
        angle_bias_radians: family_brush
            .map(|brush| brush.angle_bias_degrees.to_radians())
            .unwrap_or(0.0),
        angle_influence: family_brush
            .map(|brush| brush.angle_influence)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0),
        nib_width_base_scale: family_brush
            .map(|brush| brush.nib_width_base_scale)
            .unwrap_or(1.0)
            .clamp(0.0, 4.0),
        nib_width_angle_scale: family_brush
            .map(|brush| brush.nib_width_angle_scale)
            .unwrap_or(1.0)
            .clamp(0.0, 4.0),
        path_adherence_multiplier,
    }
}

pub(crate) fn npr_distance_width_multiplier(
    importance: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let depth_pressure = settings.depth_pressure.clamp(0.0, 1.0);
    let pressure_boost = 1.0 + depth_pressure * (importance - 1.0);
    (1.0 - settings.distance_width_falloff * (1.0 - importance))
        .mul_add(pressure_boost, 0.0)
        .clamp(0.62, 1.28)
}

pub(crate) fn npr_depth_alpha_multiplier(
    importance: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let near = importance.clamp(0.0, 1.35).powf(0.8);
    (1.0 + settings.depth_alpha.clamp(0.0, 1.0) * (near - 0.5)).clamp(0.35, 1.25)
}

pub(crate) fn npr_pressure_multiplier(
    t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
    brush: NprResolvedBrushProfile3d,
) -> f32 {
    let shaped = sample_4_point_curve(brush.width_curve, t.clamp(0.0, 1.0));
    shaped * (0.92 + settings.line_confidence.clamp(0.0, 1.0) * 0.12)
}

pub(crate) fn npr_alpha_pressure_multiplier(t: f32, brush: NprResolvedBrushProfile3d) -> f32 {
    sample_4_point_curve(brush.alpha_curve, t.clamp(0.0, 1.0)).clamp(0.0, 1.5)
}

pub(crate) fn sample_4_point_curve(points: [f32; 4], t: f32) -> f32 {
    let scaled = t * 3.0;
    let index = scaled.floor().clamp(0.0, 2.0) as usize;
    let local_t = (scaled - index as f32).clamp(0.0, 1.0);
    let a = points[index];
    let b = points[index + 1];
    a + (b - a) * local_t
}

pub(crate) fn resolve_npr_kind_style(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprResolvedKindStyle {
    resolve_npr_kind_style_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn resolve_npr_kind_style_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprResolvedKindStyle {
    let (mut width_multiplier, mut default_wobble) = match kind {
        crate::renderer::NprLineKind::Silhouette => (
            settings.silhouette_width_multiplier,
            settings.stroke_wobble_px * 0.92,
        ),
        crate::renderer::NprLineKind::Boundary => (
            settings.boundary_width_multiplier,
            settings.stroke_wobble_px,
        ),
        crate::renderer::NprLineKind::Crease => (
            settings.feature_width_multiplier,
            settings.stroke_wobble_px * 0.72,
        ),
        crate::renderer::NprLineKind::Seam => (
            settings.feature_width_multiplier * 0.85,
            settings.stroke_wobble_px * 0.58,
        ),
        crate::renderer::NprLineKind::Feature => (
            settings.feature_width_multiplier,
            settings.stroke_wobble_px * 0.72,
        ),
        crate::renderer::NprLineKind::Contact => (
            settings.feature_width_multiplier.max(1.0),
            settings.stroke_wobble_px * 0.45,
        ),
    };
    if npr_uses_character_roles(settings) {
        match kind {
            crate::renderer::NprLineKind::Silhouette => {
                width_multiplier *= 1.06;
                default_wobble *= 0.72;
            }
            crate::renderer::NprLineKind::Boundary => {
                width_multiplier *= 1.02;
                default_wobble *= 0.82;
            }
            crate::renderer::NprLineKind::Crease | crate::renderer::NprLineKind::Seam => {
                width_multiplier *= 0.88;
                default_wobble *= 0.62;
            }
            crate::renderer::NprLineKind::Feature => {
                width_multiplier *= 0.84;
                default_wobble *= 0.54;
            }
            crate::renderer::NprLineKind::Contact => {}
        }
    }
    if let Some(family) = resolve_npr_line_family_with_traits(kind, traits, settings) {
        width_multiplier *= family.width_multiplier.max(0.0);
    }
    let override_style = match kind {
        crate::renderer::NprLineKind::Silhouette => settings.silhouette_override,
        crate::renderer::NprLineKind::Boundary => settings.boundary_override,
        crate::renderer::NprLineKind::Crease => settings.feature_override,
        crate::renderer::NprLineKind::Seam => settings.feature_override,
        crate::renderer::NprLineKind::Feature => settings.feature_override,
        crate::renderer::NprLineKind::Contact => settings.feature_override,
    };
    let mut resolved = NprResolvedKindStyle {
        width_multiplier: override_style
            .and_then(|style| style.width_multiplier)
            .unwrap_or(width_multiplier),
        wobble_px: override_style
            .and_then(|style| style.wobble_px)
            .unwrap_or(default_wobble),
        dropout: override_style
            .and_then(|style| style.dropout)
            .unwrap_or(settings.dropout),
        taper: override_style
            .and_then(|style| style.taper)
            .unwrap_or(settings.taper),
        overshoot_px: override_style
            .and_then(|style| style.overshoot_px)
            .unwrap_or(settings.overshoot_px),
        alpha_multiplier: override_style
            .and_then(|style| style.alpha_multiplier)
            .unwrap_or(1.0),
    };
    if npr_uses_character_roles(settings) {
        match kind {
            crate::renderer::NprLineKind::Silhouette => {
                resolved.taper *= 0.85;
                resolved.overshoot_px *= 1.12;
            }
            crate::renderer::NprLineKind::Boundary => {
                resolved.taper *= 0.92;
                resolved.overshoot_px *= 1.06;
            }
            crate::renderer::NprLineKind::Crease
            | crate::renderer::NprLineKind::Seam
            | crate::renderer::NprLineKind::Feature => {
                resolved.taper *= 0.72;
                resolved.overshoot_px *= 0.42;
                resolved.alpha_multiplier *= 0.92;
            }
            crate::renderer::NprLineKind::Contact => {}
        }
    }
    if let Some(family) = resolve_npr_line_family_with_traits(kind, traits, settings) {
        resolved.alpha_multiplier *= family.alpha_multiplier.max(0.0);
        resolved.taper *= family.taper_multiplier.max(0.0);
        if let Some(overshoot_px) = family.overshoot_px {
            resolved.overshoot_px = overshoot_px;
        }
    }
    resolved
}

pub(crate) fn resolve_npr_line_family(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> Option<&amigo_render_api::NprLineFamily3d> {
    resolve_npr_line_family_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn resolve_npr_line_family_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> Option<&amigo_render_api::NprLineFamily3d> {
    if settings.line_families.is_empty() {
        return None;
    }
    let source = npr_line_source_for_kind(kind);
    settings
        .line_families
        .iter()
        .filter(|family| family.enabled && family.sources.contains(&source))
        .max_by(|left, right| {
            npr_line_family_score(left, traits)
                .partial_cmp(&npr_line_family_score(right, traits))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.priority.cmp(&right.priority))
        })
}

pub(crate) fn npr_line_kind_enabled(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> bool {
    settings.line_families.is_empty() || resolve_npr_line_family(kind, settings).is_some()
}

pub(crate) fn npr_preferred_stroke_length_px_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_preferred_stroke_length_px_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_preferred_stroke_length_px_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.preferred_stroke_length_px)
        .unwrap_or(settings.preferred_stroke_length_px)
}

pub(crate) fn npr_stroke_join_gap_px_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_stroke_join_gap_px_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_stroke_join_gap_px_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.stroke_join_gap_px)
        .unwrap_or(settings.stroke_join_gap_px)
}

pub(crate) fn npr_stroke_join_max_angle_degrees_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_stroke_join_max_angle_degrees_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_stroke_join_max_angle_degrees_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.stroke_join_max_angle_degrees)
        .unwrap_or(settings.stroke_join_max_angle_degrees)
}

pub(crate) fn npr_technical_detail_keep_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_technical_detail_keep_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_technical_detail_keep_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.technical_detail_keep)
        .unwrap_or(settings.technical_detail_keep)
}

pub(crate) fn npr_min_screen_length_px_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_min_screen_length_px_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_min_screen_length_px_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.min_screen_length_px)
        .unwrap_or(settings.min_screen_length_px)
}

pub(crate) fn npr_min_stroke_length_px_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_min_stroke_length_px_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_min_stroke_length_px_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.min_stroke_length_px)
        .unwrap_or(settings.min_stroke_length_px)
}

pub(crate) fn npr_continuation_bias_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_continuation_bias_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_continuation_bias_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.continuation_bias)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

pub(crate) fn npr_breakup_bias_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_breakup_bias_with_traits(kind, NprLineCandidateTraits::default(), settings)
}

pub(crate) fn npr_breakup_bias_with_traits(
    kind: crate::renderer::NprLineKind,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    resolve_npr_line_family_with_traits(kind, traits, settings)
        .and_then(|family| family.breakup_bias)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

pub(crate) fn npr_technical_detail_preference_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_family_preference_for_kind(kind, settings, |family| family.technical_detail_preference)
}

pub(crate) fn npr_ink_detail_material_preference_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_family_preference_for_kind(kind, settings, |family| {
        family.ink_detail_material_preference
    })
}

pub(crate) fn npr_material_seam_preference_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    npr_family_preference_for_kind(kind, settings, |family| family.material_seam_preference)
}

pub(crate) fn npr_line_family_role_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
) -> amigo_render_api::NprLineFamilyRole3d {
    resolve_npr_line_family(kind, settings)
        .and_then(|family| family.role)
        .unwrap_or_else(|| default_npr_line_family_role(kind))
}

fn npr_family_preference_for_kind(
    kind: crate::renderer::NprLineKind,
    settings: &amigo_render_api::NprLineSettings3d,
    preference: impl Fn(&amigo_render_api::NprLineFamily3d) -> Option<f32>,
) -> f32 {
    let source = npr_line_source_for_kind(kind);
    settings
        .line_families
        .iter()
        .filter(|family| family.enabled && family.sources.contains(&source))
        .filter_map(preference)
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

fn npr_line_source_for_kind(
    kind: crate::renderer::NprLineKind,
) -> amigo_render_api::NprLineSource3d {
    match kind {
        crate::renderer::NprLineKind::Silhouette => amigo_render_api::NprLineSource3d::Silhouette,
        crate::renderer::NprLineKind::Boundary => amigo_render_api::NprLineSource3d::Boundary,
        crate::renderer::NprLineKind::Feature => amigo_render_api::NprLineSource3d::Feature,
        crate::renderer::NprLineKind::Crease => amigo_render_api::NprLineSource3d::Crease,
        crate::renderer::NprLineKind::Seam => amigo_render_api::NprLineSource3d::Seam,
        crate::renderer::NprLineKind::Contact => amigo_render_api::NprLineSource3d::Contact,
    }
}

fn npr_line_family_score(
    family: &amigo_render_api::NprLineFamily3d,
    traits: NprLineCandidateTraits,
) -> f32 {
    let trait_score = |pref: Option<f32>, active: bool| -> f32 {
        let value = pref.unwrap_or(0.0);
        if active {
            value
        } else if value > 0.0 {
            -value * 0.35
        } else if value < 0.0 {
            value.abs() * 0.15
        } else {
            0.0
        }
    };
    family.priority as f32 * 100.0
        + trait_score(family.technical_detail_preference, traits.technical_detail)
        + trait_score(
            family.ink_detail_material_preference,
            traits.material_detail,
        )
        + trait_score(family.material_seam_preference, traits.material_seam)
}

fn npr_uses_character_roles(settings: &amigo_render_api::NprLineSettings3d) -> bool {
    settings.pipeline.candidate_strategy
        == amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
        || matches!(
            settings.pipeline.budget_strategy,
            amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority
                | amigo_render_api::NprBudgetStrategy3d::CharacterReadability
        )
}

pub(crate) fn resolve_npr_gesture_role_profile_with_traits(
    kind: crate::renderer::NprLineKind,
    path_length_px: f32,
    traits: NprLineCandidateTraits,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprGestureRoleProfile {
    if !npr_uses_character_roles(settings) {
        return NprGestureRoleProfile {
            hand_arc_multiplier: 1.0,
            tangent_drift_multiplier: 1.0,
            detail_crispness: 1.0,
            taper_multiplier: 1.0,
            overshoot_multiplier: 1.0,
            alpha_multiplier: 1.0,
        };
    }

    let role =
        resolve_npr_line_family_with_traits(kind, traits, settings).and_then(|family| family.role);
    match role.unwrap_or_else(|| default_npr_line_family_role(kind)) {
        amigo_render_api::NprLineFamilyRole3d::OuterContour => NprGestureRoleProfile {
            hand_arc_multiplier: 1.18,
            tangent_drift_multiplier: 0.82,
            detail_crispness: 0.84,
            taper_multiplier: 0.80,
            overshoot_multiplier: 1.16,
            alpha_multiplier: 1.0,
        },
        amigo_render_api::NprLineFamilyRole3d::DetailInk => NprGestureRoleProfile {
            hand_arc_multiplier: 0.44,
            tangent_drift_multiplier: 0.50,
            detail_crispness: 1.14,
            taper_multiplier: 0.62,
            overshoot_multiplier: 0.24,
            alpha_multiplier: 0.84,
        },
        amigo_render_api::NprLineFamilyRole3d::ClothFold => {
            let short_detail = path_length_px
                <= npr_preferred_stroke_length_px_for_kind(kind, settings).max(24.0) * 0.45;
            if short_detail {
                NprGestureRoleProfile {
                    hand_arc_multiplier: 0.58,
                    tangent_drift_multiplier: 0.62,
                    detail_crispness: 1.06,
                    taper_multiplier: 0.72,
                    overshoot_multiplier: 0.38,
                    alpha_multiplier: 0.88,
                }
            } else {
                NprGestureRoleProfile {
                    hand_arc_multiplier: 0.74,
                    tangent_drift_multiplier: 0.72,
                    detail_crispness: 0.98,
                    taper_multiplier: 0.78,
                    overshoot_multiplier: 0.52,
                    alpha_multiplier: 0.90,
                }
            }
        }
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => NprGestureRoleProfile {
            hand_arc_multiplier: 0.42,
            tangent_drift_multiplier: 0.46,
            detail_crispness: 1.18,
            taper_multiplier: 0.70,
            overshoot_multiplier: 0.20,
            alpha_multiplier: 0.78,
        },
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch => NprGestureRoleProfile {
            hand_arc_multiplier: 0.32,
            tangent_drift_multiplier: 0.40,
            detail_crispness: 1.08,
            taper_multiplier: 0.58,
            overshoot_multiplier: 0.0,
            alpha_multiplier: 0.68,
        },
        amigo_render_api::NprLineFamilyRole3d::ContactShadow => NprGestureRoleProfile {
            hand_arc_multiplier: 0.78,
            tangent_drift_multiplier: 0.68,
            detail_crispness: 1.0,
            taper_multiplier: 0.90,
            overshoot_multiplier: 0.62,
            alpha_multiplier: 0.90,
        },
        amigo_render_api::NprLineFamilyRole3d::Generic => match kind {
            crate::renderer::NprLineKind::Silhouette => NprGestureRoleProfile {
                hand_arc_multiplier: 1.14,
                tangent_drift_multiplier: 0.88,
                detail_crispness: 0.86,
                taper_multiplier: 0.82,
                overshoot_multiplier: 1.12,
                alpha_multiplier: 1.0,
            },
            crate::renderer::NprLineKind::Boundary => NprGestureRoleProfile {
                hand_arc_multiplier: 0.96,
                tangent_drift_multiplier: 0.82,
                detail_crispness: 0.92,
                taper_multiplier: 0.88,
                overshoot_multiplier: 1.04,
                alpha_multiplier: 0.96,
            },
            crate::renderer::NprLineKind::Crease | crate::renderer::NprLineKind::Seam => {
                let short_detail = path_length_px
                    <= npr_preferred_stroke_length_px_for_kind(kind, settings).max(24.0) * 0.45;
                if short_detail {
                    NprGestureRoleProfile {
                        hand_arc_multiplier: 0.52,
                        tangent_drift_multiplier: 0.58,
                        detail_crispness: 1.08,
                        taper_multiplier: 0.70,
                        overshoot_multiplier: 0.36,
                        alpha_multiplier: 0.88,
                    }
                } else {
                    NprGestureRoleProfile {
                        hand_arc_multiplier: 0.72,
                        tangent_drift_multiplier: 0.68,
                        detail_crispness: 0.96,
                        taper_multiplier: 0.76,
                        overshoot_multiplier: 0.48,
                        alpha_multiplier: 0.90,
                    }
                }
            }
            crate::renderer::NprLineKind::Feature => NprGestureRoleProfile {
                hand_arc_multiplier: 0.46,
                tangent_drift_multiplier: 0.54,
                detail_crispness: 1.12,
                taper_multiplier: 0.66,
                overshoot_multiplier: 0.28,
                alpha_multiplier: 0.84,
            },
            crate::renderer::NprLineKind::Contact => NprGestureRoleProfile {
                hand_arc_multiplier: 0.85,
                tangent_drift_multiplier: 0.72,
                detail_crispness: 0.98,
                taper_multiplier: 0.92,
                overshoot_multiplier: 0.72,
                alpha_multiplier: 0.92,
            },
        },
    }
}

fn default_npr_brush_tip(
    tool: amigo_render_api::NprStrokeTool3d,
) -> amigo_render_api::NprBrushTip3d {
    match tool {
        amigo_render_api::NprStrokeTool3d::InkPen => amigo_render_api::NprBrushTip3d::GPen,
        amigo_render_api::NprStrokeTool3d::Pencil => amigo_render_api::NprBrushTip3d::Round,
        amigo_render_api::NprStrokeTool3d::Brush => amigo_render_api::NprBrushTip3d::Flat,
        amigo_render_api::NprStrokeTool3d::Marker => amigo_render_api::NprBrushTip3d::Flat,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => amigo_render_api::NprBrushTip3d::MaruPen,
    }
}

fn default_npr_line_family_role(
    kind: crate::renderer::NprLineKind,
) -> amigo_render_api::NprLineFamilyRole3d {
    match kind {
        crate::renderer::NprLineKind::Silhouette | crate::renderer::NprLineKind::Boundary => {
            amigo_render_api::NprLineFamilyRole3d::OuterContour
        }
        crate::renderer::NprLineKind::Feature => amigo_render_api::NprLineFamilyRole3d::DetailInk,
        crate::renderer::NprLineKind::Crease => amigo_render_api::NprLineFamilyRole3d::ClothFold,
        crate::renderer::NprLineKind::Seam => amigo_render_api::NprLineFamilyRole3d::MaterialCut,
        crate::renderer::NprLineKind::Contact => {
            amigo_render_api::NprLineFamilyRole3d::ContactShadow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_npr_brush_profile;

    #[test]
    fn brush_profile_makes_pencil_more_exploratory_than_ink() {
        let ink = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let pencil = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let ink_profile = resolve_npr_brush_profile(crate::renderer::NprLineKind::Silhouette, &ink);
        let pencil_profile =
            resolve_npr_brush_profile(crate::renderer::NprLineKind::Silhouette, &pencil);

        assert!(pencil_profile.search_multiplier > ink_profile.search_multiplier);
        assert!(pencil_profile.pressure_jitter_multiplier > ink_profile.pressure_jitter_multiplier);
        assert!(pencil_profile.dropout_multiplier > ink_profile.dropout_multiplier);
    }

    #[test]
    fn brush_profile_makes_technical_pen_stable_and_single_pass() {
        let technical = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::TechnicalPen,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let profile =
            resolve_npr_brush_profile(crate::renderer::NprLineKind::Silhouette, &technical);

        assert_eq!(profile.search_multiplier, 0.0);
        assert_eq!(profile.dropout_multiplier, 0.0);
        assert!(profile.path_wobble_multiplier < 0.2);
        assert!(profile.pressure_jitter_multiplier < 0.2);
    }

    #[test]
    fn brush_profile_applies_author_tool_scalars() {
        let settings = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::InkPen,
            tool_width_multiplier: 1.5,
            tool_alpha_multiplier: 0.5,
            tool_search_multiplier: 0.25,
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let profile =
            resolve_npr_brush_profile(crate::renderer::NprLineKind::Silhouette, &settings);

        assert_eq!(profile.width_multiplier, 1.5);
        assert_eq!(profile.alpha_multiplier, 0.5);
        assert_eq!(profile.search_multiplier, 0.25);
    }
}
