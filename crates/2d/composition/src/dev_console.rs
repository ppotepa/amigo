use crate::{LightRoute2dSceneService, RenderDepthMode2d, RenderLayer2dSceneService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Composition2dDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct Composition2dDevConsoleCommandContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
    pub light_route2d_scene_service: &'a LightRoute2dSceneService,
}

pub fn handle_composition2d_dev_console_command(
    ctx: Composition2dDevConsoleCommandContext<'_>,
    name: &str,
    args: &[String],
) -> Composition2dDevConsoleCommandOutcome {
    match name {
        "layers.list" => {
            let lines = ctx
                .render_layer2d_scene_service
                .commands()
                .into_iter()
                .map(|layer| {
                    format!(
                        "{} order={} visible={} opacity={} depth.mode={} depth.distance_m={} computed_z_depth={} depth.blur_scale={} optical_role={}",
                        layer.id,
                        layer.order,
                        layer.visible,
                        layer.opacity,
                        depth_mode_label(layer.depth.mode),
                        layer.depth.distance_m.map_or_else(|| "none".to_owned(), |value| value.to_string()),
                        layer.depth.z_depth,
                        layer.depth.blur_scale,
                        optical_role_label(layer.optical_role)
                    )
                })
                .collect::<Vec<_>>();
            Composition2dDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "render layers: none".to_owned()
            } else {
                format!("render layers:\n{}", lines.join("\n"))
            })
        }
        "layer.opacity" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.opacity <id> <value>".to_owned(),
                );
            };
            let Ok(opacity) = value.parse::<f32>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid opacity `{value}`"
                ));
            };
            if ctx.render_layer2d_scene_service.set_opacity(id, opacity) {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` opacity={opacity}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.visible" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.visible <id> true|false".to_owned(),
                );
            };
            let Ok(visible) = value.parse::<bool>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid visible value `{value}`"
                ));
            };
            if ctx.render_layer2d_scene_service.set_visible(id, visible) {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` visible={visible}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.depth.mode" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.depth.mode <id> depth_map|distance|z_depth|infinity|overlay"
                        .to_owned(),
                );
            };
            let mode = match value.as_str() {
                "depth_map" => RenderDepthMode2d::DepthMap,
                "distance" => RenderDepthMode2d::Distance,
                "z_depth" => RenderDepthMode2d::ZDepth,
                "infinity" => RenderDepthMode2d::Infinity,
                "overlay" => RenderDepthMode2d::Overlay,
                _ => {
                    return Composition2dDevConsoleCommandOutcome::Error(format!(
                        "invalid depth mode `{value}`"
                    ));
                }
            };
            if ctx.render_layer2d_scene_service.set_depth_mode(id, mode) {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` depth.mode={value}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.depth.distance_m" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.depth.distance_m <id> <meters>".to_owned(),
                );
            };
            let Ok(distance_m) = value.parse::<f32>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid distance_m `{value}`"
                ));
            };
            if ctx
                .render_layer2d_scene_service
                .set_distance_m_with_default_space(id, distance_m)
            {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` depth.distance_m={distance_m}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.depth.z_depth" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.depth.z_depth <id> <value>".to_owned(),
                );
            };
            let Ok(z_depth) = value.parse::<f32>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid z_depth `{value}`"
                ));
            };
            if ctx.render_layer2d_scene_service.set_z_depth(id, z_depth) {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` depth.z_depth={z_depth}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.depth.blur_scale" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.depth.blur_scale <id> <value>".to_owned(),
                );
            };
            let Ok(blur_scale) = value.parse::<f32>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid depth blur scale `{value}`"
                ));
            };
            if ctx
                .render_layer2d_scene_service
                .set_depth_blur_scale(id, blur_scale)
            {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` depth.blur_scale={blur_scale}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "layer.optical_role" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.optical_role <id> world_surface|scene_medium|foreground_medium|lens_surface|overlay|debug"
                        .to_owned(),
                );
            };
            let Some(optical_role) = parse_optical_role(value) else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid optical role `{value}`"
                ));
            };
            if ctx
                .render_layer2d_scene_service
                .set_optical_role(id, optical_role)
            {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` optical_role={value}"
                ))
            } else {
                Composition2dDevConsoleCommandOutcome::Error(format!("unknown render layer `{id}`"))
            }
        }
        "routes.list" => {
            let lines = ctx
                .light_route2d_scene_service
                .commands()
                .into_iter()
                .map(|route| format!("{} groups={}", route.receiver_layer, route.groups.join(",")))
                .collect::<Vec<_>>();
            Composition2dDevConsoleCommandOutcome::Handled(if lines.is_empty() {
                "light routes: none".to_owned()
            } else {
                format!("light routes:\n{}", lines.join("\n"))
            })
        }
        _ => Composition2dDevConsoleCommandOutcome::Unhandled,
    }
}

fn depth_mode_label(mode: RenderDepthMode2d) -> &'static str {
    match mode {
        RenderDepthMode2d::DepthMap => "depth_map",
        RenderDepthMode2d::Distance => "distance",
        RenderDepthMode2d::ZDepth => "z_depth",
        RenderDepthMode2d::Infinity => "infinity",
        RenderDepthMode2d::Overlay => "overlay",
    }
}

fn optical_role_label(role: amigo_2d_spatial::OpticalLayerRole2d) -> &'static str {
    match role {
        amigo_2d_spatial::OpticalLayerRole2d::WorldSurface => "world_surface",
        amigo_2d_spatial::OpticalLayerRole2d::SceneMedium => "scene_medium",
        amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium => "foreground_medium",
        amigo_2d_spatial::OpticalLayerRole2d::LensSurface => "lens_surface",
        amigo_2d_spatial::OpticalLayerRole2d::Overlay => "overlay",
        amigo_2d_spatial::OpticalLayerRole2d::Debug => "debug",
    }
}

fn parse_optical_role(value: &str) -> Option<amigo_2d_spatial::OpticalLayerRole2d> {
    match value {
        "world_surface" => Some(amigo_2d_spatial::OpticalLayerRole2d::WorldSurface),
        "scene_medium" => Some(amigo_2d_spatial::OpticalLayerRole2d::SceneMedium),
        "foreground_medium" => Some(amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium),
        "lens_surface" => Some(amigo_2d_spatial::OpticalLayerRole2d::LensSurface),
        "overlay" => Some(amigo_2d_spatial::OpticalLayerRole2d::Overlay),
        "debug" => Some(amigo_2d_spatial::OpticalLayerRole2d::Debug),
        _ => None,
    }
}
