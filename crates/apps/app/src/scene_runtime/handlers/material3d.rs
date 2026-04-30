use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneMaterial3dCommandHandler;

impl SceneCommandHandler for SceneMaterial3dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-material-3d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueMaterial3d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueMaterial3d { command } => {
                let entity = amigo_3d_material::queue_material_scene_command(
                    ctx.scene_service,
                    ctx.material_scene_service,
                    &command,
                );

                if let Some(source) = command.source.as_ref() {
                    crate::app_helpers::register_mod_asset_reference(
                        ctx.asset_catalog,
                        &command.source_mod,
                        source,
                        "3d",
                        "material",
                    );
                }

                ctx.scene_event_queue.publish(SceneEvent::MaterialQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                    material_label: command.label.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 3d material `{}` for entity `{}` from mod `{}`",
                    command.label, command.entity_name, command.source_mod
                ));
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
