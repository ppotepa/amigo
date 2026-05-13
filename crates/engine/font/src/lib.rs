//! Font asset domain for Amigo.
//!
//! This crate owns the engine-level Font2d descriptor model.
//! It does not own GPU resources and does not depend on WGPU.

mod descriptor;
mod glyph_set;
mod model;

pub use descriptor::*;
pub use glyph_set::*;
pub use model::*;

#[cfg(test)]
mod tests;

