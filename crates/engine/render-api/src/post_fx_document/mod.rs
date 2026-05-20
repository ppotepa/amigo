use serde::{Deserialize, Serialize};

mod post_fx;
mod post_fx_defaults;
mod post_fx_lens_droplets;
mod post_fx_rain_glass;
mod post_fx_wet_reflections;

pub use post_fx::*;
use post_fx_defaults::*;
pub use post_fx_lens_droplets::*;
pub use post_fx_rain_glass::*;
pub use post_fx_wet_reflections::*;

fn default_true() -> bool {
    true
}

fn default_one() -> f32 {
    1.0
}
