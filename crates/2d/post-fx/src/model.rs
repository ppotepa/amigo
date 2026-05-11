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
}

impl PostFx2d {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
            Self::EmbossEdges(_) => "embossed_edges",
        }
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::Blur(blur) => Self::Blur(blur.normalized()),
            Self::EmbossEdges(emboss) => Self::EmbossEdges(emboss.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
            Self::EmbossEdges(emboss) => emboss.is_active(),
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
            (
                "layer.post_fx.kind".to_owned(),
                "embossed_edges".to_owned(),
            ),
            (
                "layer.post_fx.edge_strength".to_owned(),
                "1.6".to_owned(),
            ),
        ]);
        let stack =
            post_fx_stack_from_flat_metadata(&metadata, "layer.post_fx").expect("stack should parse");
        assert_eq!(stack.effects.len(), 1);
        assert!(matches!(stack.effects[0], PostFx2d::EmbossEdges(_)));
    }
}
