//! Scene hydration pipeline that turns authored documents into runtime work.
//! It expands document data into scene commands, events, and supporting runtime plans.

/// Shared hydration helpers used across domain-specific builders.
mod common;
/// Event expansion generated while hydrating authored content.
mod events;
/// Particle-specific hydration helpers and command emission.
mod particles;
/// Planning logic that turns documents into scene command batches.
mod plan;
mod post_fx;
/// Style conversion helpers used by UI and render hydration.
mod style;
/// UI hydration helpers for runtime document state.
mod ui;

pub use common::{entity_selector_from_document, scene_key_from_document};
pub use plan::{
    SceneHydrationPlan, build_scene_hydration_plan, build_scene_hydration_plan_for_runtime,
    build_scene_hydration_plan_with_component_hydrators,
};
pub use post_fx::component_post_fx_host_id;

use common::*;
use events::*;
use particles::*;
use ui::*;

#[cfg(test)]
mod tests;
