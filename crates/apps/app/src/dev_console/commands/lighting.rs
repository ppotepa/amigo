use amigo_2d_lighting::{
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
};
use amigo_math::ColorRgba;

use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

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
        match command.name.as_str() {
            "light2d.list" => list_lights(ctx),
            "light2d.set" => set_global_light(ctx, &command),
            "light2d.group" => set_light_group(ctx, &command),
            "lightmaps.list" => list_lightmaps(ctx),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn list_lights(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let lights = match ctx.required::<GlobalLight2dSceneService>() {
        Ok(service) => service,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    let lines = lights
        .commands()
        .into_iter()
        .map(|light| {
            format!(
                "{} intensity={} color={:?}",
                light.id, light.intensity, light.color
            )
        })
        .collect::<Vec<_>>();
    ConsoleCommandResult::ok(if lines.is_empty() {
        "global lights: none".to_owned()
    } else {
        format!("global lights:\n{}", lines.join("\n"))
    })
}

fn set_global_light(
    ctx: &ConsoleCommandContext<'_>,
    command: &ParsedConsoleCommand,
) -> ConsoleCommandResult {
    let [id, field, value] = command.args.as_slice() else {
        return ConsoleCommandResult::error("usage: light2d.set <id> intensity|color <value>");
    };
    let lights = match ctx.required::<GlobalLight2dSceneService>() {
        Ok(service) => service,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    match field.as_str() {
        "intensity" => {
            let Ok(intensity) = value.parse::<f32>() else {
                return ConsoleCommandResult::error(format!("invalid intensity `{value}`"));
            };
            if lights.set_intensity(id, intensity) {
                ConsoleCommandResult::ok(format!("global light `{id}` intensity={intensity}"))
            } else {
                ConsoleCommandResult::error(format!("unknown global light `{id}`"))
            }
        }
        "color" => {
            let Some(color) = parse_hex_rgba(value) else {
                return ConsoleCommandResult::error(format!("invalid color `{value}`"));
            };
            if lights.set_color(id, color) {
                ConsoleCommandResult::ok(format!("global light `{id}` color={value}"))
            } else {
                ConsoleCommandResult::error(format!("unknown global light `{id}`"))
            }
        }
        _ => ConsoleCommandResult::error("usage: light2d.set <id> intensity|color <value>"),
    }
}

fn set_light_group(
    ctx: &ConsoleCommandContext<'_>,
    command: &ParsedConsoleCommand,
) -> ConsoleCommandResult {
    let [id, field, value] = command.args.as_slice() else {
        return ConsoleCommandResult::error("usage: light2d.group <id> intensity|color <value>");
    };
    let groups = match ctx.required::<LightGroup2dSceneService>() {
        Ok(service) => service,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    match field.as_str() {
        "intensity" => {
            let Ok(intensity) = value.parse::<f32>() else {
                return ConsoleCommandResult::error(format!("invalid intensity `{value}`"));
            };
            if groups.set_intensity(id, intensity) {
                ConsoleCommandResult::ok(format!("light group `{id}` intensity={intensity}"))
            } else {
                ConsoleCommandResult::error(format!("unknown light group `{id}`"))
            }
        }
        "color" => {
            let Some(color) = parse_hex_rgba(value) else {
                return ConsoleCommandResult::error(format!("invalid color `{value}`"));
            };
            if groups.set_color(id, color) {
                ConsoleCommandResult::ok(format!("light group `{id}` color={value}"))
            } else {
                ConsoleCommandResult::error(format!("unknown light group `{id}`"))
            }
        }
        _ => ConsoleCommandResult::error("usage: light2d.group <id> intensity|color <value>"),
    }
}

fn list_lightmaps(ctx: &ConsoleCommandContext<'_>) -> ConsoleCommandResult {
    let lightmaps = match ctx.required::<LightMap2dSceneService>() {
        Ok(service) => service,
        Err(error) => return ConsoleCommandResult::error(error.to_string()),
    };
    let lines = lightmaps
        .commands()
        .into_iter()
        .map(|lightmap| {
            format!(
                "{} entity={} source={} channels={}",
                lightmap.id,
                lightmap.entity_name,
                lightmap.source.entity_name,
                lightmap
                    .channels
                    .iter()
                    .map(|channel| channel.id.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>();
    ConsoleCommandResult::ok(if lines.is_empty() {
        "lightmaps: none".to_owned()
    } else {
        format!("lightmaps:\n{}", lines.join("\n"))
    })
}

fn parse_hex_rgba(value: &str) -> Option<ColorRgba> {
    let hex = value.trim().trim_start_matches('#');
    let parse = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16)
            .ok()
            .map(|value| value as f32 / 255.0)
    };
    match hex.len() {
        6 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            1.0,
        )),
        8 => Some(ColorRgba::new(
            parse(0..2)?,
            parse(2..4)?,
            parse(4..6)?,
            parse(6..8)?,
        )),
        _ => None,
    }
}
