use super::super::super::*;
use super::super::AppScriptCommandContext;
use amigo_session::ScriptCommandHandler;

pub(super) struct RenderScriptCommandHandler;

impl<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
    for RenderScriptCommandHandler
{
    fn name(&self) -> &'static str {
        "render"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(
            (
                command.namespace.as_str(),
                command.name.as_str(),
                command.arguments.len(),
            ),
            ("2d.sprite", "spawn", 4)
                | ("2d.text", "spawn", 5)
                | ("3d.mesh", "spawn", 2)
                | ("3d.material", "bind", 3)
                | ("3d.text", "spawn", 4)
        )
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
        match (
            command.namespace.as_str(),
            command.name.as_str(),
            command.arguments.as_slice(),
        ) {
            ("2d.sprite", "spawn", [entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.text", "spawn", [entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("3d.mesh", _, _) => match amigo_3d_mesh::handle_mesh3d_script_command(
                amigo_3d_mesh::Mesh3dScriptCommandContext {
                    selected_mod: &ctx.launch_selection.selected_mod(),
                },
                command.clone(),
            ) {
                amigo_3d_mesh::Mesh3dScriptCommandOutcome::Submit(scene_command) => {
                    ctx.scene_command_queue.submit(scene_command)
                }
                amigo_3d_mesh::Mesh3dScriptCommandOutcome::Unhandled => ctx
                    .dev_console_state
                    .write_line(format!(
                        "{} could not handle command: {}",
                        self.name(),
                        crate::app_helpers::format_script_command(&command)
                    )),
            },
            ("3d.material", _, _) => {
                match amigo_3d_material::handle_material3d_script_command(
                    amigo_3d_material::Material3dScriptCommandContext {
                        selected_mod: &ctx.launch_selection.selected_mod(),
                    },
                    command.clone(),
                ) {
                    amigo_3d_material::Material3dScriptCommandOutcome::Submit(scene_command) => {
                        ctx.scene_command_queue.submit(scene_command)
                    }
                    amigo_3d_material::Material3dScriptCommandOutcome::Unhandled => ctx
                        .dev_console_state
                        .write_line(format!(
                            "{} could not handle command: {}",
                            self.name(),
                            crate::app_helpers::format_script_command(&command)
                        )),
                }
            }
            ("3d.text", _, _) => match amigo_3d_text::handle_text3d_script_command(
                amigo_3d_text::Text3dScriptCommandContext {
                    selected_mod: &ctx.launch_selection.selected_mod(),
                },
                command.clone(),
            ) {
                amigo_3d_text::Text3dScriptCommandOutcome::Submit(scene_command) => {
                    ctx.scene_command_queue.submit(scene_command)
                }
                amigo_3d_text::Text3dScriptCommandOutcome::ParseError(message) => {
                    ctx.dev_console_state.write_line(message)
                }
                amigo_3d_text::Text3dScriptCommandOutcome::Unhandled => ctx
                    .dev_console_state
                    .write_line(format!(
                        "{} could not handle command: {}",
                        self.name(),
                        crate::app_helpers::format_script_command(&command)
                    )),
            },
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}



