use amigo_core::{AmigoError, AmigoResult};

use crate::{
    ActivationSetSceneCommand, ActivationSetSceneService, CameraFollow2dSceneCommand,
    CameraFollow2dSceneService, EntityPoolSceneService, EntitySelector, Parallax2dSceneCommand,
    Parallax2dSceneService, SceneCommand, SceneCommandQueue, SceneEvent, SceneEventQueue, SceneKey,
    SceneService, format_scene_command,
};

pub struct SceneCamera2dCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub camera_follow_scene_service: &'a CameraFollow2dSceneService,
    pub parallax_scene_service: &'a Parallax2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

pub enum SceneCamera2dCommandOutcome {
    CameraFollow {
        entity_name: String,
        target: String,
        source_mod: String,
    },
    Parallax {
        entity_name: String,
        camera: String,
        source_mod: String,
    },
}

pub fn can_handle_scene_camera2d_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueCameraFollow2d { .. } | SceneCommand::QueueParallax2d { .. }
    )
}

pub fn handle_scene_camera2d_scene_command(
    ctx: SceneCamera2dCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<SceneCamera2dCommandOutcome> {
    match command {
        SceneCommand::QueueCameraFollow2d { command } => {
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.camera_follow_scene_service
                .queue(CameraFollow2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    target: command.target.clone(),
                    offset: command.offset,
                    lerp: command.lerp,
                    lookahead_velocity_scale: command.lookahead_velocity_scale,
                    lookahead_max_distance: command.lookahead_max_distance,
                    sway_amount: command.sway_amount,
                    sway_frequency: command.sway_frequency,
                });
            ctx.scene_event_queue.publish(SceneEvent::CameraFollowQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                target: command.target.clone(),
            });
            Ok(SceneCamera2dCommandOutcome::CameraFollow {
                entity_name: command.entity_name,
                target: command.target,
                source_mod: command.source_mod,
            })
        }
        SceneCommand::QueueParallax2d { command } => {
            let entity = ctx
                .scene_service
                .find_or_spawn_named_entity(command.entity_name.clone());
            ctx.parallax_scene_service.queue(Parallax2dSceneCommand {
                source_mod: command.source_mod.clone(),
                entity_name: command.entity_name.clone(),
                camera: command.camera.clone(),
                factor: command.factor,
                anchor: command.anchor,
                camera_origin: None,
            });
            ctx.scene_event_queue.publish(SceneEvent::ParallaxQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                camera: command.camera.clone(),
            });
            Ok(SceneCamera2dCommandOutcome::Parallax {
                entity_name: command.entity_name,
                camera: command.camera,
                source_mod: command.source_mod,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "scene camera2d handler cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

pub struct SceneActivationCommandContext<'a> {
    pub activation_set_scene_service: &'a ActivationSetSceneService,
}

pub enum SceneActivationCommandOutcome {
    Queued {
        id: String,
        source_mod: String,
        entry_count: usize,
    },
    Activate {
        set: Option<ActivationSetSceneCommand>,
    },
}

pub fn can_handle_scene_activation_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::QueueActivationSet { .. } | SceneCommand::ActivateSet { .. }
    )
}

pub fn handle_scene_activation_scene_command(
    ctx: SceneActivationCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<SceneActivationCommandOutcome> {
    match command {
        SceneCommand::QueueActivationSet { command } => {
            ctx.activation_set_scene_service.queue(command.clone());
            Ok(SceneActivationCommandOutcome::Queued {
                id: command.id,
                source_mod: command.source_mod,
                entry_count: command.entries.len(),
            })
        }
        SceneCommand::ActivateSet { id } => Ok(SceneActivationCommandOutcome::Activate {
            set: ctx.activation_set_scene_service.activation_set(&id),
        }),
        _ => Err(AmigoError::Message(format!(
            "scene activation handler cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

pub struct ActivationTargetResolveContext<'a> {
    pub scene_service: &'a SceneService,
    pub entity_pool_scene_service: &'a EntityPoolSceneService,
}

pub fn resolve_activation_targets(
    ctx: ActivationTargetResolveContext<'_>,
    selector: &EntitySelector,
) -> Vec<String> {
    match selector {
        EntitySelector::Entity(entity_name) => {
            if ctx.scene_service.entity_by_name(entity_name).is_some() {
                vec![entity_name.clone()]
            } else {
                Vec::new()
            }
        }
        EntitySelector::Tag(tag) => ctx.scene_service.entities_by_tag(tag),
        EntitySelector::Group(group) => ctx.scene_service.entities_by_group(group),
        EntitySelector::Pool(pool) => ctx.entity_pool_scene_service.members(pool),
    }
}

pub struct SceneLifecycleCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

pub fn can_handle_scene_lifecycle_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::SpawnNamedEntity { .. }
            | SceneCommand::ConfigureEntity { .. }
            | SceneCommand::ClearEntities
    )
}

pub fn handle_scene_lifecycle_scene_command(
    ctx: SceneLifecycleCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<()> {
    match command {
        SceneCommand::SpawnNamedEntity { name, transform } => {
            let entity = transform
                .map(|transform| ctx.scene_service.spawn_with_transform(name.clone(), transform))
                .unwrap_or_else(|| ctx.scene_service.spawn(name.clone()));
            ctx.scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name,
            });
            Ok(())
        }
        SceneCommand::ConfigureEntity {
            entity_name,
            lifecycle,
            tags,
            groups,
            properties,
        } => {
            ctx.scene_service.configure_entity_metadata(
                &entity_name,
                lifecycle,
                tags,
                groups,
                properties,
            );
            Ok(())
        }
        SceneCommand::ClearEntities => {
            ctx.scene_service.clear_entities();
            ctx.scene_event_queue.publish(SceneEvent::EntitiesCleared);
            Ok(())
        }
        _ => Err(AmigoError::Message(format!(
            "scene lifecycle handler cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

pub struct SceneSelectionCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub scene_event_queue: &'a SceneEventQueue,
    pub scene_command_queue: &'a SceneCommandQueue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSelectionCommandOutcome {
    Selected { scene: SceneKey },
    ReloadQueued { scene: SceneKey },
    ReloadSkippedNoActiveScene,
}

pub fn can_handle_scene_selection_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::SelectScene { .. } | SceneCommand::ReloadActiveScene
    )
}

pub fn handle_scene_selection_scene_command(
    ctx: SceneSelectionCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<SceneSelectionCommandOutcome> {
    match command {
        SceneCommand::SelectScene { scene } => {
            ctx.scene_service.select_scene(scene.clone());
            ctx.scene_event_queue.publish(SceneEvent::SceneSelected {
                scene: scene.clone(),
            });
            Ok(SceneSelectionCommandOutcome::Selected { scene })
        }
        SceneCommand::ReloadActiveScene => {
            let Some(active_scene) = ctx.scene_service.selected_scene() else {
                return Ok(SceneSelectionCommandOutcome::ReloadSkippedNoActiveScene);
            };

            ctx.scene_event_queue
                .publish(SceneEvent::SceneReloadRequested {
                    scene: active_scene.clone(),
                });
            ctx.scene_command_queue.submit(SceneCommand::SelectScene {
                scene: active_scene.clone(),
            });

            Ok(SceneSelectionCommandOutcome::ReloadQueued {
                scene: active_scene,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "scene selection handler cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}
