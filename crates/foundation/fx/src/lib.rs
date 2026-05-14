//! Reusable effect primitives for authoring variation and animation.
//! This crate provides weighted picks, ranges, and color ramps used by particles, behavior, and tooling.

mod color_ramp;
mod range;
mod weighted;

pub use color_ramp::{ColorInterpolation, ColorRamp, ColorStop};
pub use range::{ScalarRange, Vec2Range};
pub use weighted::WeightedChoice;
