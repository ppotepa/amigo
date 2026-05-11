use std::collections::BTreeMap;

pub const POST_FX_2D_CAPABILITY: &str = "post_fx_2d";
pub const POST_FX_2D_PLUGIN_LABEL: &str = "amigo-2d-post-fx";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostFx2dStack {
    pub effects: Vec<PostFx2d>,
}

impl PostFx2dStack {
    pub fn single(effect: PostFx2d) -> Self {
        Self {
            effects: vec![effect],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn normalized(self) -> Self {
        Self {
            effects: self
                .effects
                .into_iter()
                .map(PostFx2d::normalized)
                .filter(PostFx2d::is_active)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PostFx2d {
    Blur(PostFxBlur2d),
    EmbossEdges(PostFxEmbossEdges2d),
    LensDroplets(PostFxLensDroplets2d),
}

impl PostFx2d {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
            Self::EmbossEdges(_) => "embossed_edges",
            Self::LensDroplets(_) => "lens_droplets",
        }
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::Blur(blur) => Self::Blur(blur.normalized()),
            Self::EmbossEdges(emboss) => Self::EmbossEdges(emboss.normalized()),
            Self::LensDroplets(lens) => Self::LensDroplets(lens.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
            Self::EmbossEdges(emboss) => emboss.is_active(),
            Self::LensDroplets(lens) => lens.is_active(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensDroplets2dStage {
    AfterWorldBeforeUi,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxLensDroplets2d {
    pub enabled: bool,
    pub stage: LensDroplets2dStage,
    pub max_droplets: u32,
    pub spawn_rate: f32,
    pub min_radius_px: f32,
    pub max_radius_px: f32,
    pub min_opacity: f32,
    pub max_opacity: f32,
    pub min_lifetime: f32,
    pub max_lifetime: f32,
    pub dirt_opacity: f32,
    pub darken: f32,
    pub blur_px: f32,
    pub blur_samples: u32,
    pub distortion: f32,
    pub downsample: f32,
    pub streaks_enabled: bool,
    pub streak_chance: f32,
    pub gravity_px_per_sec: f32,
    pub max_streak_length: f32,
    pub wobble: f32,
    pub affects_world: bool,
    pub affects_game_ui: bool,
    pub affects_debug_ui: bool,
    pub strict_certification: bool,
}

impl Default for PostFxLensDroplets2d {
    fn default() -> Self {
        Self {
            enabled: true,
            stage: LensDroplets2dStage::AfterWorldBeforeUi,
            max_droplets: 48,
            spawn_rate: 0.25,
            min_radius_px: 10.0,
            max_radius_px: 42.0,
            min_opacity: 0.18,
            max_opacity: 0.52,
            min_lifetime: 4.0,
            max_lifetime: 12.0,
            dirt_opacity: 0.16,
            darken: 0.08,
            blur_px: 3.0,
            blur_samples: 4,
            distortion: 0.015,
            downsample: 1.0,
            streaks_enabled: true,
            streak_chance: 0.16,
            gravity_px_per_sec: 24.0,
            max_streak_length: 160.0,
            wobble: 0.35,
            affects_world: true,
            affects_game_ui: false,
            affects_debug_ui: false,
            strict_certification: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensDroplets2dCertificationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LensDroplets2dCertificationIssue {
    pub severity: LensDroplets2dCertificationSeverity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LensDroplets2dCertificationReport {
    pub accepted: bool,
    pub cost_score: f32,
    pub issues: Vec<LensDroplets2dCertificationIssue>,
    pub normalized: PostFxLensDroplets2d,
}

impl PostFxLensDroplets2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();

        let mut min_radius =
            finite_or(self.min_radius_px, defaults.min_radius_px).clamp(1.0, 256.0);
        let mut max_radius =
            finite_or(self.max_radius_px, defaults.max_radius_px).clamp(1.0, 256.0);
        if min_radius > max_radius {
            std::mem::swap(&mut min_radius, &mut max_radius);
        }

        let mut min_opacity = finite_or(self.min_opacity, defaults.min_opacity).clamp(0.0, 1.0);
        let mut max_opacity = finite_or(self.max_opacity, defaults.max_opacity).clamp(0.0, 1.0);
        if min_opacity > max_opacity {
            std::mem::swap(&mut min_opacity, &mut max_opacity);
        }

        let mut min_lifetime =
            finite_or(self.min_lifetime, defaults.min_lifetime).clamp(0.1, 120.0);
        let mut max_lifetime =
            finite_or(self.max_lifetime, defaults.max_lifetime).clamp(0.1, 120.0);
        if min_lifetime > max_lifetime {
            std::mem::swap(&mut min_lifetime, &mut max_lifetime);
        }

        Self {
            enabled: self.enabled,
            stage: self.stage,
            max_droplets: self.max_droplets.min(256),
            spawn_rate: finite_or(self.spawn_rate, defaults.spawn_rate).clamp(0.0, 32.0),
            min_radius_px: min_radius,
            max_radius_px: max_radius,
            min_opacity,
            max_opacity,
            min_lifetime,
            max_lifetime,
            dirt_opacity: finite_or(self.dirt_opacity, defaults.dirt_opacity).clamp(0.0, 1.0),
            darken: finite_or(self.darken, defaults.darken).clamp(0.0, 1.0),
            blur_px: finite_or(self.blur_px, defaults.blur_px).clamp(0.0, 32.0),
            blur_samples: self.blur_samples.min(16),
            distortion: finite_or(self.distortion, defaults.distortion).clamp(0.0, 0.1),
            downsample: finite_or(self.downsample, defaults.downsample).clamp(0.25, 1.0),
            streaks_enabled: self.streaks_enabled,
            streak_chance: finite_or(self.streak_chance, defaults.streak_chance).clamp(0.0, 1.0),
            gravity_px_per_sec: finite_or(self.gravity_px_per_sec, defaults.gravity_px_per_sec)
                .clamp(0.0, 512.0),
            max_streak_length: finite_or(self.max_streak_length, defaults.max_streak_length)
                .clamp(0.0, 1024.0),
            wobble: finite_or(self.wobble, defaults.wobble).clamp(0.0, 4.0),
            affects_world: self.affects_world,
            affects_game_ui: self.affects_game_ui,
            affects_debug_ui: self.affects_debug_ui,
            strict_certification: self.strict_certification,
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled
            && self.affects_world
            && (self.max_droplets > 0 || self.dirt_opacity > 0.0 || self.darken > 0.0)
    }

    pub fn certify(self) -> LensDroplets2dCertificationReport {
        let normalized = self.normalized();
        let mut issues = Vec::new();

        if normalized.affects_debug_ui {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Error,
                code: "lens_droplets_debug_ui_forbidden",
                message: "LensDroplets2D must not affect debug UI in the MVP renderer.".to_owned(),
            });
        }

        if normalized.max_droplets > 96 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_droplet_count",
                message: format!(
                    "max_droplets={} exceeds recommended budget 96.",
                    normalized.max_droplets
                ),
            });
        }

        if normalized.blur_samples > 8 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_blur_samples",
                message: format!(
                    "blur_samples={} exceeds recommended budget 8.",
                    normalized.blur_samples
                ),
            });
        }

        if normalized.blur_px > 12.0 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: LensDroplets2dCertificationSeverity::Warning,
                code: "lens_droplets_high_blur_radius",
                message: format!(
                    "blur_px={} exceeds recommended budget 12px.",
                    normalized.blur_px
                ),
            });
        }

        let blur_factor = normalized.blur_px.max(1.0) / 3.0;
        let downsample_factor = 1.0 / normalized.downsample.max(0.25);
        let cost_score = normalized.max_droplets as f32
            * normalized.blur_samples.max(1) as f32
            * blur_factor
            * downsample_factor;

        if cost_score > 1536.0 {
            issues.push(LensDroplets2dCertificationIssue {
                severity: if normalized.strict_certification {
                    LensDroplets2dCertificationSeverity::Error
                } else {
                    LensDroplets2dCertificationSeverity::Warning
                },
                code: "lens_droplets_cost_budget_exceeded",
                message: format!(
                    "estimated LensDroplets2D cost score {cost_score:.1} exceeds high budget."
                ),
            });
        }

        let accepted = !issues
            .iter()
            .any(|issue| issue.severity == LensDroplets2dCertificationSeverity::Error);

        LensDroplets2dCertificationReport {
            accepted,
            cost_score,
            issues,
            normalized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxBlur2d {
    pub radius: f32,
    pub downsample: f32,
    pub intensity: f32,
}

impl Default for PostFxBlur2d {
    fn default() -> Self {
        Self {
            radius: 12.0,
            downsample: 0.5,
            intensity: 1.0,
        }
    }
}

impl PostFxBlur2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            radius: finite_or(self.radius, defaults.radius).clamp(0.0, 128.0),
            downsample: finite_or(self.downsample, defaults.downsample).clamp(0.125, 1.0),
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 4.0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.radius > 0.0 && self.intensity > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostFxEmbossEdges2d {
    pub mode: PostFxEmbossMode2d,
    pub intensity: f32,
    pub edge_strength: f32,
    pub sample_offset_px: f32,
    pub luma_threshold: f32,
    pub luma_gamma: f32,
    pub specular_radius_px: f32,
    pub distance_falloff: f32,
    pub tint: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostFxEmbossMode2d {
    PrebakedImage,
    LightAwareRuntime,
}

impl Default for PostFxEmbossEdges2d {
    fn default() -> Self {
        Self {
            mode: PostFxEmbossMode2d::PrebakedImage,
            intensity: 0.35,
            edge_strength: 1.25,
            sample_offset_px: 1.0,
            luma_threshold: 0.22,
            luma_gamma: 2.2,
            specular_radius_px: 6.0,
            distance_falloff: 0.18,
            tint: [1.0, 1.0, 1.0],
        }
    }
}

impl PostFxEmbossEdges2d {
    pub fn normalized(self) -> Self {
        let defaults = Self::default();
        Self {
            mode: self.mode,
            intensity: finite_or(self.intensity, defaults.intensity).clamp(0.0, 2.0),
            edge_strength: finite_or(self.edge_strength, defaults.edge_strength).clamp(0.0, 4.0),
            sample_offset_px: finite_or(self.sample_offset_px, defaults.sample_offset_px)
                .clamp(1.0, 4.0),
            luma_threshold: finite_or(self.luma_threshold, defaults.luma_threshold).clamp(0.0, 1.0),
            luma_gamma: finite_or(self.luma_gamma, defaults.luma_gamma).clamp(0.5, 4.0),
            specular_radius_px: finite_or(self.specular_radius_px, defaults.specular_radius_px)
                .clamp(1.0, 24.0),
            distance_falloff: finite_or(self.distance_falloff, defaults.distance_falloff)
                .clamp(0.01, 2.0),
            tint: [
                finite_or(self.tint[0], defaults.tint[0]).clamp(0.0, 1.0),
                finite_or(self.tint[1], defaults.tint[1]).clamp(0.0, 1.0),
                finite_or(self.tint[2], defaults.tint[2]).clamp(0.0, 1.0),
            ],
        }
    }

    pub fn is_active(&self) -> bool {
        self.intensity > 0.0 && self.edge_strength > 0.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostFx2dCacheKey {
    pub source_id: String,
    pub effect_kind: &'static str,
    pub radius_milli: u32,
    pub downsample_milli: u32,
    pub intensity_milli: u32,
}

impl PostFx2dCacheKey {
    pub fn blur(source_id: impl Into<String>, blur: PostFxBlur2d) -> Self {
        let blur = blur.normalized();
        Self {
            source_id: source_id.into(),
            effect_kind: "blur",
            radius_milli: quantize_milli(blur.radius),
            downsample_milli: quantize_milli(blur.downsample),
            intensity_milli: quantize_milli(blur.intensity),
        }
    }

    pub fn embossed_edges(source_id: impl Into<String>, emboss: PostFxEmbossEdges2d) -> Self {
        let emboss = emboss.normalized();
        Self {
            source_id: source_id.into(),
            effect_kind: "embossed_edges",
            radius_milli: quantize_milli(emboss.sample_offset_px + emboss.specular_radius_px),
            downsample_milli: quantize_milli(emboss.edge_strength + emboss.distance_falloff),
            intensity_milli: quantize_milli(emboss.intensity),
        }
    }
}

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
    if value.is_finite() { value } else { fallback }
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
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "lens_droplets_debug_ui_forbidden")
        );
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
}
