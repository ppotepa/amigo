//! Scene document, hydration, and command pipeline for the engine.
//! It turns authored scene files into runtime commands and services that other domains consume.

/// Shared scene command types and queue-facing helpers.
mod commands;
/// Human-readable formatting for scene commands and diagnostics.
mod command_format;
/// Authored scene document structures and loading entry points.
mod document;
/// Service contracts that other engine domains expose to the scene layer.
mod domain_services;
/// Runtime entity identifiers and entity-facing metadata.
mod entity;
/// Scene-specific error types returned during loading and hydration.
mod error;
/// Hydration pipeline that expands documents into runtime work.
mod hydration;
/// Commands for the 2D motion domain.
mod motion_commands;
/// Commands for the particle domain.
mod particle_commands;
/// Runtime plugin wiring for the scene crate.
mod plugin;
/// Commands for rendering-oriented domains.
mod render_commands;
/// Services used while activating a newly loaded scene.
mod service_activation;
/// Helpers that queue and apply hydrated scene state.
mod service_hydration;
/// Shared queues used by scene loading and runtime execution.
mod service_queues;
/// Core scene services shared across runtime systems.
mod services;
/// Scene transition planning and active transition state.
mod transition;
/// Commands for UI and audio content described in scene documents.
mod ui_audio_commands;

pub use command_format::*;
pub use commands::*;
pub use document::*;
pub use domain_services::*;
pub use entity::*;
pub use error::*;
pub use hydration::*;
pub use motion_commands::*;
pub use particle_commands::*;
pub use plugin::*;
pub use render_commands::*;
pub use service_activation::*;
pub use service_hydration::*;
pub use service_queues::*;
pub use services::*;
pub use transition::*;
pub use ui_audio_commands::*;

#[cfg(test)]
mod tests;
