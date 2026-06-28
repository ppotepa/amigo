#[derive(Debug, Clone, Copy)]
pub(crate) struct NprResolvedKindStyle {
    pub(crate) width_multiplier: f32,
    pub(crate) wobble_px: f32,
    pub(crate) dropout: f32,
    pub(crate) taper: f32,
    pub(crate) overshoot_px: f32,
    pub(crate) alpha_multiplier: f32,
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
) -> f32 {
    let shaped = sample_4_point_curve(settings.width_pressure_curve, t.clamp(0.0, 1.0));
    shaped * (0.92 + settings.line_confidence.clamp(0.0, 1.0) * 0.12)
}

pub(crate) fn npr_alpha_pressure_multiplier(
    t: f32,
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    sample_4_point_curve(settings.alpha_pressure_curve, t.clamp(0.0, 1.0)).clamp(0.0, 1.5)
}

pub(crate) fn npr_straightness_wobble_multiplier(
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let straightness = settings.straightness.clamp(0.0, 1.0);
    let tool_multiplier = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 1.22,
        amigo_render_api::NprStrokeTool3d::Brush => 1.08,
        amigo_render_api::NprStrokeTool3d::Marker => 0.82,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 0.22,
    };
    ((1.0 - straightness)
        * 1.55
        * tool_multiplier
        * settings.tool_wobble_multiplier.max(0.0))
    .clamp(0.0, 2.5)
}

pub(crate) fn npr_tool_width_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    let base = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 0.84,
        amigo_render_api::NprStrokeTool3d::Brush => 1.18,
        amigo_render_api::NprStrokeTool3d::Marker => 1.08,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 0.92,
    };
    (base * settings.tool_width_multiplier.max(0.0)).clamp(0.05, 4.0)
}

pub(crate) fn npr_tool_alpha_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    let base = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 0.72,
        amigo_render_api::NprStrokeTool3d::Brush => 0.96,
        amigo_render_api::NprStrokeTool3d::Marker => 0.84,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 1.0,
    };
    (base * settings.tool_alpha_multiplier.max(0.0)).clamp(0.0, 2.0)
}

pub(crate) fn npr_tool_pressure_jitter_multiplier(
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let base = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 1.65,
        amigo_render_api::NprStrokeTool3d::Brush => 1.42,
        amigo_render_api::NprStrokeTool3d::Marker => 0.58,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 0.08,
    };
    (base * settings.tool_pressure_jitter_multiplier.max(0.0)).clamp(0.0, 4.0)
}

pub(crate) fn npr_tool_dropout_multiplier(
    settings: &amigo_render_api::NprLineSettings3d,
) -> f32 {
    let base = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 2.35,
        amigo_render_api::NprStrokeTool3d::Brush => 1.65,
        amigo_render_api::NprStrokeTool3d::Marker => 0.42,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 0.0,
    };
    (base * settings.tool_dropout_multiplier.max(0.0)).clamp(0.0, 5.0)
}

pub(crate) fn npr_tool_search_multiplier(settings: &amigo_render_api::NprLineSettings3d) -> f32 {
    let base = match settings.stroke_tool {
        amigo_render_api::NprStrokeTool3d::InkPen => 1.0,
        amigo_render_api::NprStrokeTool3d::Pencil => 1.65,
        amigo_render_api::NprStrokeTool3d::Brush => 0.72,
        amigo_render_api::NprStrokeTool3d::Marker => 0.35,
        amigo_render_api::NprStrokeTool3d::TechnicalPen => 0.0,
    };
    (base * settings.tool_search_multiplier.max(0.0)).clamp(0.0, 5.0)
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
    let (width_multiplier, default_wobble) = match kind {
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
    let override_style = match kind {
        crate::renderer::NprLineKind::Silhouette => settings.silhouette_override,
        crate::renderer::NprLineKind::Boundary => settings.boundary_override,
        crate::renderer::NprLineKind::Crease => settings.feature_override,
        crate::renderer::NprLineKind::Seam => settings.feature_override,
        crate::renderer::NprLineKind::Feature => settings.feature_override,
        crate::renderer::NprLineKind::Contact => settings.feature_override,
    };
    NprResolvedKindStyle {
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
    }
}
