//! Backend-independent, deterministic non-photorealistic rendering contracts.
//!
//! This crate deliberately contains no WGPU types.  It turns authored geometry and
//! a camera into a stable packet of flat triangles and screen-space ink strokes.

pub mod camera;
pub mod brush;
pub mod debug;
pub mod feature;
pub mod field;
pub mod frame;
pub mod geometry;
pub mod math;
pub mod medium;
pub mod nprpipeline;
pub mod paper;
pub mod salience;
pub mod style;
pub mod tessellation;
pub mod temporal;
pub mod topology;
pub mod value;

pub use camera::*;
pub use brush::*;
pub use debug::*;
pub use feature::*;
pub use field::*;
pub use frame::*;
pub use geometry::*;
pub use math::*;
pub use medium::*;
pub use nprpipeline::*;
pub use paper::*;
pub use salience::*;
pub use style::*;
pub use tessellation::*;
pub use temporal::*;
pub use topology::*;
pub use value::*;
