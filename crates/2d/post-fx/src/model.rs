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
}

impl PostFx2d {
    pub fn kind(self) -> &'static str {
        match self {
            Self::Blur(_) => "blur",
        }
    }

    pub fn normalized(self) -> Self {
        match self {
            Self::Blur(blur) => Self::Blur(blur.normalized()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Blur(blur) => blur.is_active(),
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
}
