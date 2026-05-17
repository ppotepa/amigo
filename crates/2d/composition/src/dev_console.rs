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
                        "{} order={} visible={} opacity={} depth.mode={} depth.value={} depth.blur_scale={}",
                        layer.id,
                        layer.order,
                        layer.visible,
                        layer.opacity,
                        match layer.depth.mode {
                            RenderDepthMode2d::DepthMap => "depth_map",
                            RenderDepthMode2d::Plane => "plane",
                            RenderDepthMode2d::Overlay => "overlay",
                        },
                        layer.depth.value,
                        layer.depth.blur_scale
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
                    "usage: layer.depth.mode <id> depth_map|plane|overlay".to_owned(),
                );
            };
            let mode = match value.as_str() {
                "depth_map" => RenderDepthMode2d::DepthMap,
                "plane" => RenderDepthMode2d::Plane,
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
        "layer.depth.value" => {
            let [id, value] = args else {
                return Composition2dDevConsoleCommandOutcome::Error(
                    "usage: layer.depth.value <id> <value>".to_owned(),
                );
            };
            let Ok(depth_value) = value.parse::<f32>() else {
                return Composition2dDevConsoleCommandOutcome::Error(format!(
                    "invalid depth value `{value}`"
                ));
            };
            if ctx
                .render_layer2d_scene_service
                .set_depth_plane_value(id, depth_value)
            {
                Composition2dDevConsoleCommandOutcome::Handled(format!(
                    "render layer `{id}` depth.value={depth_value}"
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
