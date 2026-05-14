use amigo_scene::{SceneCommand, SceneCommandQueue, SceneKey, SceneService};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};

use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

pub(crate) struct SceneConsoleCommandHandler;

impl ConsoleCommandHandler for SceneConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "scene-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "scene.stats",
                aliases: &["stats"],
                category: "scene",
                help: "Show active scene stats.",
                usage: "scene.stats",
                examples: &["stats", "scene.stats", "scene stats"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scene.reload",
                aliases: &["reload"],
                category: "scene",
                help: "Reload the active scene.",
                usage: "scene.reload",
                examples: &["reload", "scene.reload", "scene reload"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scene.select",
                aliases: &[],
                category: "scene",
                help: "Select a scene by id.",
                usage: "scene.select <scene-id>",
                examples: &["scene.select main-menu", "scene select main-menu"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "scene.entities",
                aliases: &["entities"],
                category: "scene",
                help: "List, add, remove, or inspect scene entities.",
                usage: "scene.entities <list|add|remove|inspect> [entity-name]",
                examples: &[
                    "entities",
                    "scene.entities list",
                    "scene.entities add enemy",
                    "scene.entities inspect player",
                    "scene.entities remove enemy",
                ],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(
            command.name.as_str(),
            "stats" | "reload" | "entities" | "entity"
        ) || command.name == "scene"
            || command.name.starts_with("scene.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        mut command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        normalize_scene_command(&mut command);

        match command.name.as_str() {
            "scene.stats" => scene_stats(ctx),
            "scene.reload" => scene_reload(ctx),
            "scene.select" => scene_select(ctx, &command.args),
            "scene.info" => scene_info(ctx),
            "scene.entities" => scene_entities(ctx, &command.args),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn scene_stats(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let scene = match ctx.required::<SceneService>() {
        Ok(scene) => scene,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };

    ConsoleCommandResult::ok(format!(
        "scene.active={} scene.entities={}",
        scene
            .selected_scene()
            .map(|scene| scene.as_str().to_owned())
            .unwrap_or_else(|| "none".to_owned()),
        scene.entity_count()
    ))
}

fn scene_reload(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
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

fn scene_select(ctx: &ConsoleCommandContext<'_>, args: &[String]) -> ConsoleCommandResult {
    let Some(scene_id) = args.first() else {
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

fn scene_info(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
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

fn scene_entities(ctx: &ConsoleCommandContext<'_>, args: &[String]) -> ConsoleCommandResult {
    let scene = match ctx.required::<SceneService>() {
        Ok(scene) => scene,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };

    let verb = args.first().map(String::as_str).unwrap_or("list");

    match verb {
        "list" => {
            let names = scene.entity_names();
            ConsoleCommandResult::ok(if names.is_empty() {
                "scene.entities: none".to_owned()
            } else {
                format!("scene.entities:\n{}", names.join("\n"))
            })
        }
        "add" => {
            let Some(entity_name) = args.get(1) else {
                return ConsoleCommandResult::error("usage: scene.entities add <entity-name>");
            };
            let id = scene.spawn(entity_name.clone());
            ConsoleCommandResult::ok(format!(
                "scene.entities added `{entity_name}` id={}",
                id.raw()
            ))
        }
        "remove" => {
            let Some(entity_name) = args.get(1) else {
                return ConsoleCommandResult::error("usage: scene.entities remove <entity-name>");
            };
            let removed = scene.remove_entities_by_name(&[entity_name.clone()]);
            ConsoleCommandResult::ok(format!("scene.entities removed={removed}"))
        }
        "inspect" => {
            let Some(entity_name) = args.get(1) else {
                return ConsoleCommandResult::error("usage: scene.entities inspect <entity-name>");
            };
            let Some(entity) = scene.entity_by_name(entity_name) else {
                return ConsoleCommandResult::error(format!("entity `{entity_name}` not found"));
            };
            ConsoleCommandResult::ok(format!(
                "entity name={} id={} visible={} enabled={} collision={} tags={} groups={} properties={}",
                entity.name,
                entity.id.raw(),
                entity.lifecycle.visible,
                entity.lifecycle.simulation_enabled,
                entity.lifecycle.collision_enabled,
                entity.tags.len(),
                entity.groups.len(),
                entity.properties.len()
            ))
        }
        _ => ConsoleCommandResult::error(
            "usage: scene.entities <list|add|remove|inspect> [entity-name]",
        ),
    }
}

fn normalize_scene_command(command: &mut ParsedConsoleCommand) {
    match command.name.as_str() {
        "stats" => {
            command.name = "scene.stats".to_owned();
        }
        "reload" => {
            command.name = "scene.reload".to_owned();
        }
        "entities" => {
            command.name = "scene.entities".to_owned();
            if command.args.is_empty() {
                command.args.push("list".to_owned());
            }
        }
        "entity" => {
            command.name = "scene.entities".to_owned();
            command.args.insert(0, "inspect".to_owned());
        }
        "scene" => {
            let Some(verb) = command.args.first().cloned() else {
                command.name = "scene.info".to_owned();
                return;
            };
            command.name = format!("scene.{verb}");
            command.args.remove(0);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::SceneConsoleCommandHandler;
    use crate::{ParsedConsoleCommand, RuntimeConsoleCommandHandler};

    #[test]
    fn scene_claims_root_stats() {
        let handler = SceneConsoleCommandHandler;
        let command = ParsedConsoleCommand {
            raw: "stats".to_owned(),
            name: "stats".to_owned(),
            args: Vec::new(),
        };

        assert!(handler.can_handle(&command));
    }

    #[test]
    fn scene_claims_root_entities() {
        let handler = SceneConsoleCommandHandler;
        let command = ParsedConsoleCommand {
            raw: "entities".to_owned(),
            name: "entities".to_owned(),
            args: Vec::new(),
        };

        assert!(handler.can_handle(&command));
    }
}
