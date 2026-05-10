use amigo_scene::{SceneCommand, SceneCommandQueue, SceneKey, SceneService};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};

use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

pub(crate) struct SceneConsoleCommandHandler;

impl ConsoleCommandHandler for SceneConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "scene-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "scene.reload",
                aliases: &[],
                category: "scene",
                help: "Reload the active scene.",
                usage: "scene.reload",
                examples: &["scene.reload", "scene reload"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scene.select",
                aliases: &[],
                category: "scene",
                help: "Select a scene by id.",
                usage: "scene.select <scene-id>",
                examples: &["scene.select main-menu"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "scene" || command.name.starts_with("scene.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        mut command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        normalize_legacy_scene_command(&mut command);
        match command.name.as_str() {
            "scene.reload" => {
                let queue = match ctx.required::<SceneCommandQueue>() {
                    Ok(queue) => queue,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let events = match ctx.required::<ScriptEventQueue>() {
                    Ok(events) => events,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                queue.submit(SceneCommand::ReloadActiveScene);
                events.publish(ScriptEvent::new(
                    "dev-console.scene-reload-requested",
                    Vec::<String>::new(),
                ));
                ConsoleCommandResult::ok("queued reload for the active scene")
            }
            "scene.select" => {
                let Some(scene_id) = command.args.first() else {
                    return ConsoleCommandResult::error("usage: scene.select <scene-id>");
                };
                let queue = match ctx.required::<SceneCommandQueue>() {
                    Ok(queue) => queue,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let events = match ctx.required::<ScriptEventQueue>() {
                    Ok(events) => events,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                queue.submit(SceneCommand::SelectScene {
                    scene: SceneKey::new(scene_id.clone()),
                });
                events.publish(ScriptEvent::new(
                    "dev-console.scene-select-requested",
                    vec![scene_id.clone()],
                ));
                ConsoleCommandResult::ok(format!("queued scene selection `{scene_id}`"))
            }
            "scene.info" => {
                let scene = match ctx.required::<SceneService>() {
                    Ok(scene) => scene,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(format!(
                    "active_scene={} entities={}",
                    scene
                        .selected_scene()
                        .map(|scene| scene.as_str().to_owned())
                        .unwrap_or_else(|| "none".to_owned()),
                    scene.entities().len()
                ))
            }
            "scene.entities" => {
                let scene = match ctx.required::<SceneService>() {
                    Ok(scene) => scene,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let names = scene
                    .entities()
                    .into_iter()
                    .map(|entity| entity.name)
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(if names.is_empty() {
                    "scene entities: none".to_owned()
                } else {
                    format!("scene entities:\n{}", names.join("\n"))
                })
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn normalize_legacy_scene_command(command: &mut ParsedConsoleCommand) {
    if command.name != "scene" {
        return;
    }
    let Some(verb) = command.args.first().cloned() else {
        return;
    };
    command.name = format!("scene.{verb}");
    command.args.remove(0);
}
