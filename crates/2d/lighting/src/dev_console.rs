use amigo_math::ColorRgba;

use crate::{GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lighting2dDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct Lighting2dDevConsoleCommandContext<'a> {
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub light_group2d_scene_service: &'a LightGroup2dSceneService,
    pub light_map2d_scene_service: &'a LightMap2dSceneService,
}

pub fn handle_lighting2d_dev_console_command(
    ctx: Lighting2dDevConsoleCommandContext<'_>,
    name: &str,
    args: &[String],
) -> Lighting2dDevConsoleCommandOutcome {
    match name {
        "light2d.list" => {
            let lines = ctx
                .global_light2d_scene_service
                .commands()
                .into_iter()
                .map(|light| format!("{} intensity={} color={:?}", light.id, light.intensity, light.color))
                .collect::<Vec<_>>();
            Lighting2dDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "global lights: none".to_owned()
            } else {
                format!("global lights:\n{}", lines.join("\n"))
            })
        }
        "light2d.set" => {
            let [id, field, value] = args else {
                return Lighting2dDevConsoleCommandOutcome::Error(
                    "usage: light2d.set <id> intensity|color <value>".to_owned(),
                );
            };
            match field.as_str() {
                "intensity" => {
                    let Ok(intensity) = value.parse::<f32>() else {
                        return Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "invalid intensity `{value}`"
                        ));
                    };
                    if ctx.global_light2d_scene_service.set_intensity(id, intensity) {
                        Lighting2dDevConsoleCommandOutcome::Handled(format!(
                            "global light `{id}` intensity={intensity}"
                        ))
                    } else {
                        Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "unknown global light `{id}`"
                        ))
                    }
                }
                "color" => {
                    let Some(color) = parse_hex_rgba(value) else {
                        return Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "invalid color `{value}`"
                        ));
                    };
                    if ctx.global_light2d_scene_service.set_color(id, color) {
                        Lighting2dDevConsoleCommandOutcome::Handled(format!(
                            "global light `{id}` color={value}"
                        ))
                    } else {
                        Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "unknown global light `{id}`"
                        ))
                    }
                }
                _ => Lighting2dDevConsoleCommandOutcome::Error(
                    "usage: light2d.set <id> intensity|color <value>".to_owned(),
                ),
            }
        }
        "light2d.group" => {
            let [id, field, value] = args else {
                return Lighting2dDevConsoleCommandOutcome::Error(
                    "usage: light2d.group <id> intensity|color <value>".to_owned(),
                );
            };
            match field.as_str() {
                "intensity" => {
                    let Ok(intensity) = value.parse::<f32>() else {
                        return Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "invalid intensity `{value}`"
                        ));
                    };
                    if ctx.light_group2d_scene_service.set_intensity(id, intensity) {
                        Lighting2dDevConsoleCommandOutcome::Handled(format!(
                            "light group `{id}` intensity={intensity}"
                        ))
                    } else {
                        Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "unknown light group `{id}`"
                        ))
                    }
                }
                "color" => {
                    let Some(color) = parse_hex_rgba(value) else {
                        return Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "invalid color `{value}`"
                        ));
                    };
                    if ctx.light_group2d_scene_service.set_color(id, color) {
                        Lighting2dDevConsoleCommandOutcome::Handled(format!(
                            "light group `{id}` color={value}"
                        ))
                    } else {
                        Lighting2dDevConsoleCommandOutcome::Error(format!(
                            "unknown light group `{id}`"
                        ))
                    }
                }
                _ => Lighting2dDevConsoleCommandOutcome::Error(
                    "usage: light2d.group <id> intensity|color <value>".to_owned(),
                ),
            }
        }
        "lightmaps.list" => {
            let lines = ctx
                .light_map2d_scene_service
                .commands()
                .into_iter()
                .map(|lightmap| {
                    format!(
                        "{} entity={} source={} channels={}",
                        lightmap.id,
                        lightmap.entity_name,
                        lightmap.source.entity_name,
                        lightmap.channels.iter().map(|channel| channel.id.clone()).collect::<Vec<_>>().join(",")
                    )
                })
                .collect::<Vec<_>>();
            Lighting2dDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "lightmaps: none".to_owned()
            } else {
                format!("lightmaps:\n{}", lines.join("\n"))
            })
        }
        _ => Lighting2dDevConsoleCommandOutcome::Unhandled,
    }
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
