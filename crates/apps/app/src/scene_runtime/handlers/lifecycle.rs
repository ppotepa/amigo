use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use super::super::{
    clear_runtime_scene_content_with_runtime, load_scene_document_for_mod,
    queue_scene_document_hydration, record_loaded_scene_document_for_runtime,
    record_scene_hydration_queued_for_runtime, record_scene_lifecycle_error_for_runtime,
};

pub(crate) struct SceneLifecycleCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneLifecycleCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-lifecycle"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_scene::can_handle_scene_lifecycle_scene_command(command)
            || amigo_scene::can_handle_scene_selection_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        if amigo_scene::can_handle_scene_lifecycle_scene_command(&command) {
            amigo_scene::handle_scene_lifecycle_scene_command(
                amigo_scene::SceneLifecycleCommandContext {
                    scene_service: ctx.scene_service,
                    scene_event_queue: ctx.scene_event_queue,
                },
                command,
            )?;
            return Ok(());
        }

        match command {
            SceneCommand::SelectScene { scene } => {
                let scene_id = scene.as_str().to_owned();
                let loaded_scene_document =
                    if let Some(root_mod) = ctx.launch_selection.startup_mod.as_deref() {
                        match load_scene_document_for_mod(ctx.runtime, root_mod, &scene_id) {
                            Ok(document) => document,
                            Err(error) => {
                                record_scene_lifecycle_error_for_runtime(ctx.runtime, &error);
                                ctx.dev_console_state.write_line(error.to_string());
                                return Ok(());
                            }
                        }
                    } else {
                        None
                    };

                clear_runtime_scene_content_with_runtime(ctx.runtime)?;

                amigo_scene::handle_scene_selection_scene_command(
                    amigo_scene::SceneSelectionCommandContext {
                        scene_service: ctx.scene_service,
                        scene_event_queue: ctx.scene_event_queue,
                        scene_command_queue: ctx.scene_command_queue,
                    },
                    SceneCommand::SelectScene {
                        scene: scene.clone(),
                    },
                )?;

                if let Some(loaded_scene_document) = loaded_scene_document {
                    queue_scene_document_hydration(
                        ctx.scene_command_queue,
                        ctx.dev_console_state,
                        ctx.hydrated_scene_state,
                        ctx.scene_transition_service,
                        &loaded_scene_document,
                    );
                    record_loaded_scene_document_for_runtime(ctx.runtime, &loaded_scene_document);
                    record_scene_hydration_queued_for_runtime(ctx.runtime);
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
                let outcome = amigo_scene::handle_scene_selection_scene_command(
                    amigo_scene::SceneSelectionCommandContext {
                        scene_service: ctx.scene_service,
                        scene_event_queue: ctx.scene_event_queue,
                        scene_command_queue: ctx.scene_command_queue,
                    },
                    SceneCommand::ReloadActiveScene,
                )?;

                match outcome {
                    amigo_scene::SceneSelectionCommandOutcome::ReloadQueued { scene } => {
                        ctx.dev_console_state.write_line(format!(
                            "reloading active scene `{}` through queue-driven scene selection",
                            scene.as_str()
                        ));
                        Ok(())
                    }
                    amigo_scene::SceneSelectionCommandOutcome::ReloadSkippedNoActiveScene => {
                        ctx.dev_console_state
                            .write_line("cannot reload scene because no active scene is selected");
                        Ok(())
                    }
                    amigo_scene::SceneSelectionCommandOutcome::Selected { .. } => {
                        Err(AmigoError::Message(format!(
                            "{} received wrong scene selection outcome",
                            self.name()
                        )))
                    }
                }
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}


