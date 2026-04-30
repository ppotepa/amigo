use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::{
    clear_runtime_scene_content_with_runtime, load_scene_document_for_mod,
    queue_scene_document_hydration,
};

pub(crate) struct SceneLifecycleCommandHandler;

impl SceneCommandHandler for SceneLifecycleCommandHandler {
    fn name(&self) -> &'static str {
        "scene-lifecycle"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::SpawnNamedEntity { .. }
                | SceneCommand::ConfigureEntity { .. }
                | SceneCommand::SelectScene { .. }
                | SceneCommand::ReloadActiveScene
                | SceneCommand::ClearEntities
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::SpawnNamedEntity { name, transform } => {
                let entity = transform
                    .map(|transform| {
                        ctx.scene_service
                            .spawn_with_transform(name.clone(), transform)
                    })
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
            SceneCommand::SelectScene { scene } => {
                let scene_id = scene.as_str().to_owned();
                let loaded_scene_document =
                    if let Some(root_mod) = ctx.launch_selection.startup_mod.as_deref() {
                        match load_scene_document_for_mod(ctx.runtime, root_mod, &scene_id) {
                            Ok(document) => document,
                            Err(error) => {
                                ctx.dev_console_state.write_line(error.to_string());
                                return Ok(());
                            }
                        }
                    } else {
                        None
                    };

                clear_runtime_scene_content_with_runtime(ctx.runtime)?;
                ctx.scene_service.select_scene(scene.clone());
                ctx.scene_event_queue.publish(SceneEvent::SceneSelected {
                    scene: scene.clone(),
                });
                if let Some(loaded_scene_document) = loaded_scene_document {
                    queue_scene_document_hydration(
                        ctx.scene_command_queue,
                        ctx.dev_console_state,
                        ctx.hydrated_scene_state,
                        ctx.scene_transition_service,
                        &loaded_scene_document,
                    );
                } else {
                    ctx.scene_transition_service.clear();
                    ctx.dev_console_state.write_line(format!(
                        "active placeholder scene set to `{}` without scene document hydration",
                        scene.as_str()
                    ));
                }
                Ok(())
            }
            SceneCommand::ReloadActiveScene => {
                let Some(active_scene) = ctx.scene_service.selected_scene() else {
                    ctx.dev_console_state
                        .write_line("cannot reload scene because no active scene is selected");
                    return Ok(());
                };
                ctx.scene_event_queue
                    .publish(SceneEvent::SceneReloadRequested {
                        scene: active_scene.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "reloading active scene `{}` through queue-driven scene selection",
                    active_scene.as_str()
                ));
                ctx.scene_command_queue.submit(SceneCommand::SelectScene {
                    scene: active_scene,
                });
                Ok(())
            }
            SceneCommand::ClearEntities => {
                ctx.scene_service.clear_entities();
                ctx.scene_event_queue.publish(SceneEvent::EntitiesCleared);
                ctx.dev_console_state
                    .write_line("cleared placeholder scene entities");
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
