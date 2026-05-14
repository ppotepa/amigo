use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

pub(crate) struct Lighting2dConsoleCommandHandler;

impl ConsoleCommandHandler for Lighting2dConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "lighting-2d-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "light2d.list",
            aliases: &[],
            category: "lighting",
            help: "List 2D global lights.",
            usage: "light2d.list",
            examples: &["light2d.list"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name.starts_with("light2d.") || command.name == "lightmaps.list"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let global_lights = match ctx.required::<amigo_2d_lighting::GlobalLight2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let light_groups = match ctx.required::<amigo_2d_lighting::LightGroup2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let lightmaps = match ctx.required::<amigo_2d_lighting::LightMap2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match amigo_2d_lighting::handle_lighting2d_dev_console_command(
            amigo_2d_lighting::Lighting2dDevConsoleCommandContext {
                global_light2d_scene_service: global_lights.as_ref(),
                light_group2d_scene_service: light_groups.as_ref(),
                light_map2d_scene_service: lightmaps.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            amigo_2d_lighting::Lighting2dDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            amigo_2d_lighting::Lighting2dDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            amigo_2d_lighting::Lighting2dDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}
