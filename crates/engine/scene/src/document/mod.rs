//! Authored scene document schema and loading helpers.
//! This module owns the YAML-facing data model that scene hydration consumes.

/// Behavior-related authored document fragments.
mod behavior;
/// Component schemas shared by authored scene entities.
mod components;
/// Core scene document types and top-level metadata.
mod core;
/// Default values used while decoding authored scene content.
mod defaults;
/// Scene document loading and parsing entry points.
mod loader;
/// Particle-specific authored document structures.
mod particles;
/// Render-oriented scalar and color value decoding.
mod render_values;
/// Authored UI document fragments embedded in scenes.
mod ui;

pub use behavior::*;
pub use components::*;
pub use core::*;
pub use loader::*;
pub use particles::*;
pub use render_values::*;
pub use ui::*;

#[cfg(test)]
mod tests;
