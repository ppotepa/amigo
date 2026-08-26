//! Scene document, hydration, and command pipeline for the engine.
//! It turns authored scene files into runtime commands and services that other domains consume.

/// Human-readable formatting for scene commands and diagnostics.
mod command_format;
/// Shared scene command types and queue-facing helpers.
mod commands;
mod component_graph_provider;
/// Plugin-owned component hydration extension point.
mod component_hydrator_registry;
/// Component metadata and editor-facing capability descriptors.
mod component_metadata;
/// Transitional engine-owned metadata grouped behind domain providers.
mod component_metadata_domains;
/// Plugin-owned component metadata provider extension point.
mod component_metadata_provider;
/// Plugin-owned component schema descriptor registry.
mod component_schema_registry;
/// Authored scene document structures and loading entry points.
mod document;
/// Service contracts that other engine domains expose to the scene layer.
mod domain_services;
/// Runtime entity identifiers and entity-facing metadata.
mod entity;
/// Effective editor-property access semantics.
mod editor_property_semantics;
/// Scene-specific error types returned during loading and hydration.
mod error;
/// Semantic scene graph, typed references, and scene-object projections.
mod graph;
/// Hydration pipeline that expands documents into runtime work.
mod hydration;
/// Declarative metadata traits used by engine, backend DTOs, editor UI, and tools.
mod metadata_traits;
/// Commands for the 2D motion domain.
mod motion_commands;
/// Commands for the particle domain.
mod particle_commands;
/// Runtime plugin wiring for the scene crate.
mod plugin;
/// Plugin-owned scene command envelopes.
mod plugin_command;
/// Plugin-owned scene component descriptor registry contracts.
mod plugin_registry;
/// Compile-time contracts for plugin-owned scene component registrations.
mod plugin_specs;
/// Commands for rendering-oriented domains.
mod render_commands;
/// Runtime scene reset handler registry.
mod reset;
/// Runtime contribution descriptors for scene-owned handlers and systems.
mod runtime_capabilities;
/// Scene-owned command handlers that are shared by app hosts and future editors.
mod scene_command;
/// Registry for plugin-owned scene command handlers.
mod scene_command_registry;
mod script_command;
/// Services used while activating a newly loaded scene.
mod service_activation;
/// Helpers that queue and apply hydrated scene state.
mod service_hydration;
/// Shared queues used by scene loading and runtime execution.
mod service_queues;
/// Core scene services shared across runtime systems.
mod services;
mod systems;
/// Scene transition planning and active transition state.
mod transition;
/// Commands for UI and audio content described in scene documents.
mod ui_audio_commands;

pub use command_format::*;
pub use commands::{RuntimeSceneCommandHandler, *};
pub use component_graph_provider::*;
pub use component_hydrator_registry::*;
pub use component_metadata::*;
pub use component_metadata_domains::*;
pub use component_metadata_provider::*;
pub use component_schema_registry::*;
pub use document::*;
pub use domain_services::*;
pub use entity::*;
pub use editor_property_semantics::*;
pub use error::*;
pub use graph::*;
pub use hydration::*;
pub use metadata_traits::*;
pub use motion_commands::*;
pub use particle_commands::*;
pub use plugin::*;
pub use plugin_command::*;
pub use plugin_registry::*;
pub use plugin_specs::*;
pub use render_commands::*;
pub use reset::*;
pub use runtime_capabilities::*;
pub use scene_command::*;
pub use scene_command_registry::*;
pub use script_command::*;
pub use service_activation::*;
pub use service_hydration::*;
pub use service_queues::*;
pub use services::*;
pub use systems::*;
pub use transition::*;
pub use ui_audio_commands::*;

#[cfg(test)]
mod tests;
