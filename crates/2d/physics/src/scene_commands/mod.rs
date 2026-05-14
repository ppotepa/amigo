//! Scene command types and handlers for the 2D physics domain.
//! It translates hydrated scene content into collider, trigger, and body registration updates.

mod aabb_collider;
mod circle_collider;
mod collision_rules;
mod kinematic_body;
mod static_collider;
mod trigger;

pub use aabb_collider::queue_aabb_collider_scene_command;
pub use circle_collider::queue_circle_collider_scene_command;
pub use collision_rules::queue_collision_event_rule_scene_command;
pub use kinematic_body::queue_kinematic_body_scene_command;
pub use static_collider::queue_static_collider_scene_command;
pub use trigger::queue_trigger_scene_command;
