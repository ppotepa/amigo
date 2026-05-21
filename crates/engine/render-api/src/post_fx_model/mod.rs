pub fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

pub fn quantize_milli(value: f32) -> u32 {
    (finite_or(value, 0.0).max(0.0) * 1000.0).round() as u32
}

mod blur;
mod cache_key;
mod camera_exposure;
mod camera_optics;
mod color_quantize;
mod color_ramp;
mod crt;
mod dirty_bloom;
mod downscale;
mod effect;
mod emboss_edges;
mod film_emulsion;
mod film_noise;
mod focus_blur;
mod lens_droplets;
mod rain_glass;
mod rain_glass_patch;
mod render_descriptor;
mod scan_output;
mod shutter_blur;
mod stack;
mod wet_reflections;

pub use blur::*;
pub use cache_key::*;
pub use camera_exposure::*;
pub use camera_optics::*;
pub use color_quantize::*;
pub use color_ramp::*;
pub use crt::*;
pub use dirty_bloom::*;
pub use downscale::*;
pub use effect::*;
pub use emboss_edges::*;
pub use film_emulsion::*;
pub use film_noise::*;
pub use focus_blur::*;
pub use lens_droplets::*;
pub use rain_glass::*;
pub use rain_glass_patch::*;
pub use render_descriptor::*;
pub use scan_output::*;
pub use shutter_blur::*;
pub use stack::*;
pub use wet_reflections::*;
