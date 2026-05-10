use amigo_2d_composition::{LightRoute2dSceneService, RenderLayer2dSceneService};

use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

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
        match command.name.as_str() {
            "layers.list" => {
                let layers = match ctx.required::<RenderLayer2dSceneService>() {
                    Ok(service) => service,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let lines = layers
                    .commands()
                    .into_iter()
                    .map(|layer| {
                        format!(
                            "{} order={} visible={} opacity={}",
                            layer.id, layer.order, layer.visible, layer.opacity
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(if lines.is_empty() {
                    "render layers: none".to_owned()
                } else {
                    format!("render layers:\n{}", lines.join("\n"))
                })
            }
            "layer.opacity" => {
                let [id, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error("usage: layer.opacity <id> <value>");
                };
                let Ok(opacity) = value.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid opacity `{value}`"));
                };
                let layers = match ctx.required::<RenderLayer2dSceneService>() {
                    Ok(service) => service,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                if layers.set_opacity(id, opacity) {
                    ConsoleCommandResult::ok(format!("render layer `{id}` opacity={opacity}"))
                } else {
                    ConsoleCommandResult::error(format!("unknown render layer `{id}`"))
                }
            }
            "layer.visible" => {
                let [id, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error("usage: layer.visible <id> true|false");
                };
                let Ok(visible) = value.parse::<bool>() else {
                    return ConsoleCommandResult::error(format!("invalid visible value `{value}`"));
                };
                let layers = match ctx.required::<RenderLayer2dSceneService>() {
                    Ok(service) => service,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                if layers.set_visible(id, visible) {
                    ConsoleCommandResult::ok(format!("render layer `{id}` visible={visible}"))
                } else {
                    ConsoleCommandResult::error(format!("unknown render layer `{id}`"))
                }
            }
            "routes.list" => {
                let routes = match ctx.required::<LightRoute2dSceneService>() {
                    Ok(service) => service,
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                let lines = routes
                    .commands()
                    .into_iter()
                    .map(|route| {
                        format!("{} groups={}", route.receiver_layer, route.groups.join(","))
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(if lines.is_empty() {
                    "light routes: none".to_owned()
                } else {
                    format!("light routes:\n{}", lines.join("\n"))
                })
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}
