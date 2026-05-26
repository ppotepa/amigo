//! Rhai world bindings exposed by the scripting backend.
//! This module registers gameplay-facing APIs and scalar helpers into each script engine instance.

/// Input-action helpers exposed to scripts.
pub(crate) mod actions;
/// Arcade-style movement helpers that combine input and motion state.
pub(crate) mod arcade;
/// Asset catalog access exposed to scripts.
pub(crate) mod assets;
/// Audio playback and mixer controls exposed to scripts.
pub(crate) mod audio;
/// 2D beacon-light runtime bindings.
pub(crate) mod beacon2d;
/// Camera-owned lens-surface runtime bindings.
pub(crate) mod camera;
/// Shared script command helpers.
pub(crate) mod commands;
/// Common scalar conversion helpers used by bindings.
pub(crate) mod common;
/// Handle reference bindings shared across APIs.
pub(crate) mod handle_refs;
/// Debug and developer-console bindings.
pub(crate) mod debug;
/// Entity lookup and mutation bindings.
pub(crate) mod entities;
/// Raw input bindings exposed to scripts.
pub(crate) mod input;
/// 2D layered-image runtime bindings.
pub(crate) mod layered_image2d;
/// 2D lighting runtime bindings.
pub(crate) mod light2d;
/// 3D material bindings.
pub(crate) mod material3d;
/// 3D mesh bindings.
pub(crate) mod mesh3d;
/// Mod and content-pack metadata bindings.
pub(crate) mod mod_api;
/// Motion control and state bindings.
pub(crate) mod motion;
/// Particle emitter and preset bindings.
pub(crate) mod particles;
/// Physics query and collider bindings.
pub(crate) mod physics;
/// Entity-pool bindings used for reuse-oriented gameplay patterns.
pub(crate) mod pools;
/// Post-fx stack inspection bindings.
pub(crate) mod postfx;
/// Projectile helpers built on top of motion and pools.
pub(crate) mod projectiles;
/// Random value helpers for lightweight script effects.
pub(crate) mod random;
/// 2D render-composition runtime bindings.
pub(crate) mod render2d;
/// Runtime diagnostics and backend metadata bindings.
pub(crate) mod runtime;
/// Rhai binding registration dispatcher.
pub(crate) mod registration;
/// Scene selection and reload bindings.
pub(crate) mod scene;
/// Session-state bindings that live longer than a single scene.
pub(crate) mod session;
/// 2D sprite bindings.
pub(crate) mod sprite2d;
/// Per-scene state bindings.
pub(crate) mod state;
/// 2D text bindings.
pub(crate) mod text2d;
/// 3D text bindings.
pub(crate) mod text3d;
/// Frame time and elapsed-time bindings.
pub(crate) mod time;
/// Timer utility bindings for script-managed schedules.
pub(crate) mod timers;
/// Trace bindings for structured script diagnostics.
pub(crate) mod trace;
/// Runtime UI bindings for live document updates.
pub(crate) mod ui;
/// 2D vector-shape bindings.
pub(crate) mod vector2d;
/// Root world object that groups domain APIs for scripts.
pub(crate) mod world_root;

pub(crate) use postfx::PostFxItemRef;
pub(crate) use render2d::RenderLayer2dHandle;
pub(crate) use time::ScriptTimeState;
pub(crate) use world_root::WorldApi;

pub fn register_world_api(engine: &mut rhai::Engine) {
    registration::register_all(engine);
}
