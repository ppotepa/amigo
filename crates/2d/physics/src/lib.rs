//! 2D collision and kinematic simulation for the engine.
//! It owns collider state, overlap queries, scene commands, and movement helpers shared by scripts and systems.

/// Collision and trigger events produced by the physics domain.
mod events;
/// Shared physics data model for bodies, colliders, and rules.
mod model;
/// Runtime plugin wiring for the 2D physics crate.
mod plugin;
/// Registries that index active colliders, bodies, and triggers.
mod registry;
/// Scene command adapters for registering physics content from scenes.
mod scene_commands;
/// Selector helpers used by overlap and query APIs.
mod selectors;
/// High-level physics service API consumed by runtime systems and scripts.
mod service;
/// Low-level geometry and movement routines used by the service layer.
mod simulation;

pub use events::*;
pub use model::*;
pub use plugin::*;
pub use scene_commands::*;
pub use selectors::*;
pub use service::*;
pub use simulation::*;

pub use amigo_scene::{
    AabbCollider2dSceneCommand, CircleCollider2dSceneCommand, CollisionEventRule2dSceneCommand,
    KinematicBody2dSceneCommand, StaticCollider2dSceneCommand, Trigger2dSceneCommand,
};

#[cfg(test)]
mod tests;
