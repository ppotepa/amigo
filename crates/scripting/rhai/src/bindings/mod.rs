//! Rhai world bindings exposed by the scripting backend.

pub(crate) mod actions;
pub(crate) mod arcade;
pub(crate) mod assets;
pub(crate) mod audio;
pub(crate) mod beacon2d;
pub(crate) mod camera;
pub(crate) mod commands;
pub(crate) mod common;
pub(crate) mod controls;
pub(crate) mod debug;
pub(crate) mod entities;
pub(crate) mod handle_refs;
pub(crate) mod input;
mod installers;
pub(crate) mod layered_image2d;
pub(crate) mod light2d;
pub(crate) mod material3d;
pub(crate) mod mesh3d;
pub(crate) mod mod_api;
pub(crate) mod motion;
pub(crate) mod panels;
pub(crate) mod particles;
pub(crate) mod physics;
pub(crate) mod physics3d;
pub(crate) mod pools;
pub(crate) mod postfx;
pub(crate) mod projectiles;
pub(crate) mod random;
pub(crate) mod registration;
pub(crate) mod render2d;
pub(crate) mod runtime;
pub(crate) mod scene;
pub(crate) mod session;
pub(crate) mod sprite2d;
pub(crate) mod state;
pub(crate) mod text2d;
pub(crate) mod text3d;
pub(crate) mod time;
pub(crate) mod timers;
pub(crate) mod trace;
pub(crate) mod ui;
pub(crate) mod vector2d;
pub(crate) mod world_root;

pub(crate) use postfx::PostFxItemRef;
pub(crate) use render2d::RenderLayer2dHandle;
pub(crate) use time::ScriptTimeState;
pub(crate) use world_root::WorldApi;

pub fn register_world_api(engine: &mut rhai::Engine, provider_namespaces: &[String]) {
    registration::register_all(engine, provider_namespaces);
}
