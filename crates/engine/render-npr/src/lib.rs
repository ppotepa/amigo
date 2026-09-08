//! Backend-independent, deterministic non-photorealistic rendering contracts.
//!
//! This crate deliberately contains no WGPU types.  It turns authored geometry and
//! a camera into a stable packet of flat triangles and screen-space ink strokes.

pub mod budget;
pub mod camera;
pub mod contour;
pub mod debug;
pub mod feature;
pub mod field;
pub mod fingerprint;
pub mod frame;
pub mod geometry;
pub mod gesture;
pub mod hatching;
pub mod lod;
pub mod math;
pub mod stroke;
pub mod style;
pub mod surface;
pub mod temporal;
pub mod tessellation;
pub mod tone;
pub mod tool;
pub mod topology;

pub use budget::*;
pub use camera::*;
pub use contour::*;
pub use debug::*;
pub use feature::*;
pub use field::*;
pub use fingerprint::*;
pub use frame::*;
pub use geometry::*;
pub use hatching::*;
pub use lod::*;
pub use math::*;
pub use stroke::*;
pub use style::*;
pub use surface::*;
pub use temporal::*;
pub use tessellation::*;
pub use tone::*;
pub use tool::*;
pub use topology::*;
