use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct RenderScriptCommandHandler;

impl ScriptCommandHandler for RenderScriptCommandHandler {
    fn name(&self) -> &'static str {
        "render"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(
            command.namespace.as_str(),
            "2d.sprite" | "2d.text" | "3d.mesh" | "3d.material" | "3d.text"
        )
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (
            command.namespace.as_str(),
            command.name.as_str(),
            command.arguments.as_slice(),
        ) {
            ("2d.sprite", "spawn", [source_mod, entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
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
            ("2d.text", "spawn", [source_mod, entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
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
            ("3d.mesh", "spawn", [source_mod, entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.mesh", "spawn", [entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        ctx.launch_selection.selected_mod(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.material", "bind", [source_mod, entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.material", "bind", [entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.text", "spawn", [source_mod, entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            ("3d.text", "spawn", [entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
