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
/// Prefab document schema for reusable entity hierarchies.
mod prefab;
/// Render-oriented scalar and color value decoding.
mod render_values;
/// Authored UI document fragments embedded in scenes.
mod ui;
/// Authored 2D visual composition document fragments.
mod visual2d;

pub use behavior::*;
pub use components::*;
pub use core::*;
pub use loader::*;
pub use particles::*;
pub use prefab::*;
pub use render_values::*;
pub use ui::*;
pub use visual2d::*;

#[cfg(test)]
mod tests;
