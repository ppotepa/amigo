use serde::{Deserialize, Serialize};

mod core;
mod draw_layer;
mod lighting;
mod spatial;

pub use core::*;
pub use draw_layer::*;
pub use lighting::*;
pub use amigo_composite_plugin::scene::document::{
    ColorQuantize2dDocument, ColorRamp2dDocument, Crt2dDocument, DirtyBloom2dDocument,
    Downscale2dDocument, FilmNoise2dDocument, LensDroplets2dDocument, PostFx2dDocument,
    RainGlass2dDocument, ShutterBlur2dDocument, WetReflections2dDocument,
};
pub use spatial::*;
