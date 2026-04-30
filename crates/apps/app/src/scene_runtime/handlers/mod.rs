mod activation;
mod audio;
mod body2d;
mod camera2d;
mod collider2d;
mod lifecycle;
mod material3d;
mod mesh3d;
mod motion2d;
mod sprite2d;
mod text2d;
mod text3d;
mod tilemap2d;
mod trigger2d;
mod ui;
mod vector2d;

use super::dispatcher::{SceneCommandHandlerRegistry, register_scene_command_handler};

pub(super) use activation::SceneActivationCommandHandler;
pub(super) use audio::SceneAudioCommandHandler;
pub(super) use body2d::SceneBody2dCommandHandler;
pub(super) use camera2d::SceneCamera2dCommandHandler;
pub(super) use collider2d::SceneCollider2dCommandHandler;
pub(super) use lifecycle::SceneLifecycleCommandHandler;
pub(super) use material3d::SceneMaterial3dCommandHandler;
pub(super) use mesh3d::SceneMesh3dCommandHandler;
pub(super) use motion2d::SceneMotion2dCommandHandler;
pub(super) use sprite2d::SceneSprite2dCommandHandler;
pub(super) use text2d::SceneText2dCommandHandler;
pub(super) use text3d::SceneText3dCommandHandler;
pub(super) use tilemap2d::SceneTileMap2dCommandHandler;
pub(super) use trigger2d::SceneTrigger2dCommandHandler;
pub(super) use ui::SceneUiCommandHandler;
pub(super) use vector2d::SceneVector2dCommandHandler;

pub(super) fn register_builtin_scene_command_handlers(registry: &mut SceneCommandHandlerRegistry) {
    register_scene_command_handler(registry, SceneLifecycleCommandHandler);
    register_scene_command_handler(registry, SceneActivationCommandHandler);
    register_scene_command_handler(registry, SceneAudioCommandHandler);
    register_scene_command_handler(registry, SceneSprite2dCommandHandler);
    register_scene_command_handler(registry, SceneText2dCommandHandler);
    register_scene_command_handler(registry, SceneVector2dCommandHandler);
    register_scene_command_handler(registry, SceneTileMap2dCommandHandler);
    register_scene_command_handler(registry, SceneBody2dCommandHandler);
    register_scene_command_handler(registry, SceneCollider2dCommandHandler);
    register_scene_command_handler(registry, SceneTrigger2dCommandHandler);
    register_scene_command_handler(registry, SceneMotion2dCommandHandler);
    register_scene_command_handler(registry, SceneCamera2dCommandHandler);
    register_scene_command_handler(registry, SceneMesh3dCommandHandler);
    register_scene_command_handler(registry, SceneMaterial3dCommandHandler);
    register_scene_command_handler(registry, SceneText3dCommandHandler);
    register_scene_command_handler(registry, SceneUiCommandHandler);
}
