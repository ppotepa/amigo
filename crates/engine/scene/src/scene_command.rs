use amigo_core::{AmigoError, AmigoResult};

use crate::{
    ActivationSetSceneCommand, ActivationSetSceneService, EntityPoolSceneService, EntitySelector, RuntimeSceneCommandHandler, SceneCommand, SceneCommandQueue,
    SceneEvent, SceneEventQueue, SceneKey, SceneService, format_scene_command,
};

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

pub struct SceneLifecycleRuntimeSceneCommandHandler;

impl RuntimeSceneCommandHandler for SceneLifecycleRuntimeSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_scene_lifecycle_scene_command(command)
            || can_handle_scene_selection_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        if can_handle_scene_lifecycle_scene_command(&command) {
            let scene_service = runtime.required::<SceneService>()?;
            let scene_event_queue = runtime.required::<SceneEventQueue>()?;
            handle_scene_lifecycle_scene_command(
                SceneLifecycleCommandContext {
                    scene_service: scene_service.as_ref(),
                    scene_event_queue: scene_event_queue.as_ref(),
                },
                command,
            )?;
            return Ok(());
        }

        if can_handle_scene_selection_scene_command(&command) {
            let scene_service = runtime.required::<SceneService>()?;
            let scene_event_queue = runtime.required::<SceneEventQueue>()?;
            let scene_command_queue = runtime.required::<SceneCommandQueue>()?;
            handle_scene_selection_scene_command(
                SceneSelectionCommandContext {
                    scene_service: scene_service.as_ref(),
                    scene_event_queue: scene_event_queue.as_ref(),
                    scene_command_queue: scene_command_queue.as_ref(),
                },
                command,
            )?;
            return Ok(());
        }

        Err(AmigoError::Message(format!(
            "scene-lifecycle runtime handler cannot handle command {}",
            format_scene_command(&command)
        )))
    }
}

pub struct SceneActivationRuntimeSceneCommandHandler;

impl RuntimeSceneCommandHandler for SceneActivationRuntimeSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_scene_activation_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let activation_set_scene_service = runtime.required::<ActivationSetSceneService>()?;
        handle_scene_activation_scene_command(
            SceneActivationCommandContext {
                activation_set_scene_service: activation_set_scene_service.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}

pub struct ScenePostFx2dRuntimeSceneCommandHandler;

impl RuntimeSceneCommandHandler for ScenePostFx2dRuntimeSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::SetPostFx2dStack { .. })
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let post_fx = runtime.required::<amigo_2d_post_fx::PostFx2dService>()?;
        let SceneCommand::SetPostFx2dStack {
            stack,
            lens_certification_reports,
        } = command
        else {
            return Err(AmigoError::Message(format!(
                "scene-post-fx-2d runtime handler cannot handle command {}",
                format_scene_command(&command)
            )));
        };

        amigo_2d_post_fx::handle_post_fx_scene_stack(
            amigo_2d_post_fx::PostFxSceneCommandContext {
                post_fx2d_service: post_fx.as_ref(),
            },
            stack,
            lens_certification_reports,
        )?;
        Ok(())
    }
}



