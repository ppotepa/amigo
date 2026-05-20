use amigo_devtools::{
    ConsoleCommandDescriptor, ConsoleCommandResult, DevConsoleCommandContext,
    ParsedConsoleCommand, RuntimeConsoleCommandHandler,
};

pub struct LayeredImageConsoleCommandHandler;

impl RuntimeConsoleCommandHandler for LayeredImageConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "layered-image-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "layered.opacity",
            aliases: &[],
            category: "layered-image",
            help: "Set runtime opacity for one layered image layer.",
            usage: "layered.opacity <entity> <layer-id> <value>",
            examples: &["layered.opacity main-menu-background club_sign 0.5"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name.starts_with("layered.")
    }

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let layered = match ctx.required::<crate::LayeredImageSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match (command.name.as_str(), command.args.as_slice()) {
            ("layered.opacity", [entity_name, layer_id, opacity]) => match opacity.parse::<f32>() {
                Ok(opacity) => {
                    if layered.set_layer_opacity(entity_name, layer_id, opacity) {
                        ConsoleCommandResult::ok(format!(
                            "updated layered image `{entity_name}` layer `{layer_id}` opacity"
                        ))
                    } else {
                        ConsoleCommandResult::error(format!(
                            "layered image `{entity_name}` layer `{layer_id}` was not found"
                        ))
                    }
                }
                Err(error) => ConsoleCommandResult::error(format!(
                    "invalid layered image opacity `{opacity}`: {error}"
                )),
            },
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}
