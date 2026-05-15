use super::*;

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
