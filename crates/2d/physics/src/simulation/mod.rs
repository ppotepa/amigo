//! Low-level 2D collision simulation routines.
//! This module contains the geometry tests and broad helpers used by the higher-level physics service.

mod aabb;
mod circles;

pub use aabb::{move_and_collide, overlaps_trigger, overlaps_trigger_with_translation};
pub use circles::circle_colliders_overlap;
