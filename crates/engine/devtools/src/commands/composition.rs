use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

pub(crate) struct Composition2dConsoleCommandHandler;

impl ConsoleCommandHandler for Composition2dConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "composition-2d-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "layers.list",
            aliases: &[],
            category: "composition",
            help: "List 2D render layers.",
            usage: "layers.list",
            examples: &["layers.list"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(
            command.name.as_str(),
            "layers.list" | "layer.opacity" | "layer.visible" | "routes.list"
        )
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let render_layers = match ctx.required::<amigo_2d_composition::RenderLayer2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        let light_routes = match ctx.required::<amigo_2d_composition::LightRoute2dSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match amigo_2d_composition::handle_composition2d_dev_console_command(
            amigo_2d_composition::Composition2dDevConsoleCommandContext {
                render_layer2d_scene_service: render_layers.as_ref(),
                light_route2d_scene_service: light_routes.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            amigo_2d_composition::Composition2dDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            amigo_2d_composition::Composition2dDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            amigo_2d_composition::Composition2dDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}



