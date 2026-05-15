use std::collections::BTreeMap;

pub const POST_FX_2D_CAPABILITY: &str = "post_fx_2d";
pub const POST_FX_2D_PLUGIN_LABEL: &str = "amigo-2d-post-fx";

pub(crate) fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

pub(crate) fn quantize_milli(value: f32) -> u32 {
    (finite_or(value, 0.0).max(0.0) * 1000.0).round() as u32
}

mod blur;
mod cache_key;
mod color_quantize;
mod crt;
mod dirty_bloom;
mod downscale;
mod effect;
mod emboss_edges;
mod film_noise;
mod flat_metadata;
mod lens_droplets;
mod rain_glass;
mod shutter_blur;
mod stack;
mod wet_reflections;

pub use blur::*;
pub use cache_key::*;
pub use color_quantize::*;
pub use crt::*;
pub use dirty_bloom::*;
pub use downscale::*;
pub use effect::*;
pub use emboss_edges::*;
pub use film_noise::*;
pub use flat_metadata::*;
pub use lens_droplets::*;
pub use rain_glass::*;
pub use shutter_blur::*;
pub use stack::*;
pub use wet_reflections::*;
