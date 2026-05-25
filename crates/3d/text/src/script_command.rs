use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::{SceneCommand, Text3dSceneCommand};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Text3dScriptCommandContext<'a> {
    pub selected_mod: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Text3dScriptCommandOutcome {
    Submit(SceneCommand),
    ParseError(String),
    Unhandled,
}

pub fn handle_text3d_script_command(
    ctx: Text3dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Text3dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("spawn", [source_mod, entity_name, content, font_key, size]) => {
            match size.parse::<f32>() {
                Ok(size) => Text3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
                    command: amigo_scene::text_3d_plugin_scene_command(Text3dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        content.clone(),
                        AssetKey::new(font_key.clone()),
                        size,
                    )),
                }),
                Err(error) => Text3dScriptCommandOutcome::ParseError(format!(
                    "failed to parse 3d text size `{size}` as f32: {error}"
                )),
            }
        }
        ("spawn", [entity_name, content, font_key, size]) => match size.parse::<f32>() {
            Ok(size) => Text3dScriptCommandOutcome::Submit(SceneCommand::Plugin {
                command: amigo_scene::text_3d_plugin_scene_command(Text3dSceneCommand::new(
                    ctx.selected_mod.to_owned(),
                    entity_name.clone(),
                    content.clone(),
                    AssetKey::new(font_key.clone()),
                    size,
                )),
            }),
            Err(error) => Text3dScriptCommandOutcome::ParseError(format!(
                "failed to parse 3d text size `{size}` as f32: {error}"
            )),
        },
        _ => Text3dScriptCommandOutcome::Unhandled,
    }
}

pub struct Text3dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Text3dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "3d.text"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "3d.text" && command.name == "spawn" && command.arguments.len() == 5
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<amigo_scene::SceneCommandQueue>()?;
        match handle_text3d_script_command(Text3dScriptCommandContext { selected_mod: "" }, command)
        {
            Text3dScriptCommandOutcome::Submit(scene_command) => {
                scene_command_queue.submit(scene_command);
            }
            Text3dScriptCommandOutcome::ParseError(_) | Text3dScriptCommandOutcome::Unhandled => {}
        }
        Ok(())
    }
}
