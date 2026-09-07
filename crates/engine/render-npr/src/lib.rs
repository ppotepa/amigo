//! Backend-independent, deterministic non-photorealistic rendering contracts.
//!
//! This crate deliberately contains no WGPU types.  It turns authored geometry and
//! a camera into a stable packet of flat triangles and screen-space ink strokes.

pub mod camera;
pub mod debug;
pub mod feature;
pub mod frame;
pub mod geometry;
pub mod math;
pub mod style;
pub mod tessellation;
pub mod topology;

pub use camera::*;
pub use debug::*;
pub use feature::*;
pub use frame::*;
pub use geometry::*;
pub use math::*;
pub use style::*;
pub use tessellation::*;
pub use topology::*;
