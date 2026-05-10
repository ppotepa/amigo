use amigo_2d_layered_image::LayeredImageSceneService;

use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

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
        let layered = match ctx.required::<LayeredImageSceneService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "layered.list" => {
                let names = layered.entity_names();
                ConsoleCommandResult::ok(if names.is_empty() {
                    "layered images: none".to_owned()
                } else {
                    format!("layered images:\n{}", names.join("\n"))
                })
            }
            "layered.info" => {
                let Some(entity) = command.args.first() else {
                    return ConsoleCommandResult::error("usage: layered.info <entity>");
                };
                let Some(command) = layered
                    .commands()
                    .into_iter()
                    .find(|command| command.entity_name == *entity)
                else {
                    return ConsoleCommandResult::error(format!(
                        "unknown layered image `{entity}`"
                    ));
                };
                let overrides = command
                    .image
                    .layer_overrides
                    .iter()
                    .map(|override_| {
                        format!(
                            "{} opacity={:?} enabled={:?} blend={:?}",
                            override_.id,
                            override_.opacity,
                            override_.enabled,
                            override_.blend_mode
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(format!(
                    "{} asset={} base_opacity={} render_layer={} z={} overrides={}",
                    command.entity_name,
                    command.image.asset.as_str(),
                    command.image.base_opacity,
                    command.render_layer,
                    command.z_index,
                    if overrides.is_empty() {
                        "none".to_owned()
                    } else {
                        overrides.join("; ")
                    }
                ))
            }
            "layered.base" => {
                let [entity, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error("usage: layered.base <entity> <opacity>");
                };
                let Ok(opacity) = value.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid opacity `{value}`"));
                };
                if layered.set_base_opacity(entity, opacity) {
                    ConsoleCommandResult::ok(format!(
                        "layered image `{entity}` base opacity={opacity}"
                    ))
                } else {
                    ConsoleCommandResult::error(format!("unknown layered image `{entity}`"))
                }
            }
            "layered.opacity" => {
                let [entity, layer, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error(
                        "usage: layered.opacity <entity> <layer-id> <value>",
                    );
                };
                let Ok(opacity) = value.parse::<f32>() else {
                    return ConsoleCommandResult::error(format!("invalid opacity `{value}`"));
                };
                if layered.set_layer_opacity(entity, layer, opacity) {
                    ConsoleCommandResult::ok(format!(
                        "layered image `{entity}` layer `{layer}` opacity={opacity}"
                    ))
                } else {
                    ConsoleCommandResult::error(format!(
                        "layered image `{entity}` layer `{layer}` not found"
                    ))
                }
            }
            "layered.enable" => {
                let [entity, layer, value] = command.args.as_slice() else {
                    return ConsoleCommandResult::error(
                        "usage: layered.enable <entity> <layer-id> true|false",
                    );
                };
                let Ok(enabled) = value.parse::<bool>() else {
                    return ConsoleCommandResult::error(format!("invalid enabled value `{value}`"));
                };
                if layered.set_layer_enabled(entity, layer, enabled) {
                    ConsoleCommandResult::ok(format!(
                        "layered image `{entity}` layer `{layer}` enabled={enabled}"
                    ))
                } else {
                    ConsoleCommandResult::error(format!(
                        "layered image `{entity}` layer `{layer}` not found"
                    ))
                }
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}
