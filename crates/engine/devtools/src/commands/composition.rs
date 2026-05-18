use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

pub(crate) struct Composition2dConsoleCommandHandler;

impl ConsoleCommandHandler for Composition2dConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "composition-2d-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "layers.list",
                aliases: &[],
                category: "composition",
                help: "List 2D render layers.",
                usage: "layers.list",
                examples: &["layers.list"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "layer.depth.mode",
                aliases: &[],
                category: "composition",
                help: "Set a 2D render layer depth mode.",
                usage: "layer.depth.mode <layer-id> depth_map|distance|z_depth|infinity|overlay",
                examples: &["layer.depth.mode weather.rain.mid distance"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "layer.depth.distance_m",
                aliases: &[],
                category: "composition",
                help: "Set a 2D render layer authoring distance in meters.",
                usage: "layer.depth.distance_m <layer-id> <meters>",
                examples: &["layer.depth.distance_m weather.rain.mid 75"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "layer.depth.z_depth",
                aliases: &[],
                category: "composition",
                help: "Set a 2D render layer z-depth value.",
                usage: "layer.depth.z_depth <layer-id> <value>",
                examples: &["layer.depth.z_depth weather.rain.mid 0.52"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "layer.depth.blur_scale",
                aliases: &[],
                category: "composition",
                help: "Set a 2D render layer depth blur scale.",
                usage: "layer.depth.blur_scale <layer-id> <value>",
                examples: &["layer.depth.blur_scale weather.rain.mid 0.25"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(
            command.name.as_str(),
            "layers.list"
                | "layer.opacity"
                | "layer.visible"
                | "layer.depth.mode"
                | "layer.depth.distance_m"
                | "layer.depth.z_depth"
                | "layer.depth.blur_scale"
                | "routes.list"
        )
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let render_layers = match ctx.required::<amigo_2d_composition::RenderLayer2dSceneService>()
        {
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
