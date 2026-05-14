use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_math::Vec2;
use amigo_runtime::Runtime;
use amigo_scene::{SceneCommand, SceneCommandQueue, Text2dSceneCommand};
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

pub struct Text2dScriptCommandHandler;

impl RuntimeScriptCommandHandler for Text2dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "2d.text"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "2d.text" && command.name == "spawn" && command.arguments.len() == 6
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = runtime.required::<SceneCommandQueue>()?;
        if let Some(scene_command) = parse_text_spawn(command) {
            scene_command_queue.submit(scene_command);
        }
        Ok(())
    }
}

fn parse_text_spawn(command: ScriptCommand) -> Option<SceneCommand> {
    let [source_mod, entity_name, content, font_key, width, height] = command.arguments.as_slice()
    else {
        return None;
    };
    let bounds = parse_vec2(width, height)?;
    Some(SceneCommand::QueueText2d {
        command: Text2dSceneCommand::new(
            source_mod.clone(),
            entity_name.clone(),
            content.clone(),
            AssetKey::new(font_key.clone()),
            bounds,
        ),
    })
}

fn parse_vec2(x: &str, y: &str) -> Option<Vec2> {
    let x = x.parse::<f32>().ok()?;
    let y = y.parse::<f32>().ok()?;
    Some(Vec2::new(x, y))
}
