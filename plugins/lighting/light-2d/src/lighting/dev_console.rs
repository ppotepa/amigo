use amigo_devtools::{
    ConsoleCommandDescriptor, ConsoleCommandResult, DevConsoleCommandContext,
    ParsedConsoleCommand, RuntimeConsoleCommandHandler,
};

pub struct Lighting2dConsoleCommandHandler;

impl RuntimeConsoleCommandHandler for Lighting2dConsoleCommandHandler {
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
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let global_lights = match ctx.required::<crate::GlobalLight2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let light_groups = match ctx.required::<crate::LightGroup2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let lightmaps = match ctx.required::<crate::LightMap2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match command.name.as_str() {
            "light2d.list" => {
                let global_count = global_lights.commands().len();
                let group_count = light_groups.commands().len();
                ConsoleCommandResult::ok(format!(
                    "light2d: {global_count} global lights, {group_count} light groups"
                ))
            }
            "lightmaps.list" => {
                let lightmap_count = lightmaps.commands().len();
                ConsoleCommandResult::ok(format!("lightmaps: {lightmap_count} sources"))
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}
