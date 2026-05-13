use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;

pub(crate) struct LayeredImageConsoleCommandHandler;

impl ConsoleCommandHandler for LayeredImageConsoleCommandHandler {
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
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let layered = match ctx.required::<amigo_2d_layered_image::LayeredImageSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };
        match amigo_2d_layered_image::handle_layered_image_dev_console_command(
            amigo_2d_layered_image::LayeredImageDevConsoleCommandContext {
                layered_image_scene_service: layered.as_ref(),
            },
            &command.name,
            &command.args,
        ) {
            amigo_2d_layered_image::LayeredImageDevConsoleCommandOutcome::Handled(message) => {
                ConsoleCommandResult::ok(message)
            }
            amigo_2d_layered_image::LayeredImageDevConsoleCommandOutcome::Error(message) => {
                ConsoleCommandResult::error(message)
            }
            amigo_2d_layered_image::LayeredImageDevConsoleCommandOutcome::Unhandled => {
                ConsoleCommandResult::unknown(command.raw)
            }
        }
    }
}



