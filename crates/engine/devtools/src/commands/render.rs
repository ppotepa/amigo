use crate::DevConsoleCommandContext as ConsoleCommandContext;
use crate::RuntimeConsoleCommandHandler as ConsoleCommandHandler;
use crate::{ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand};

use amigo_render_api::{RenderCompositionDiagnosticsService, RenderFrameStatsService};

pub(crate) struct RenderConsoleCommandHandler;

impl ConsoleCommandHandler for RenderConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "render-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "render.stats",
                aliases: &["fps"],
                category: "render",
                help: "Show current render frame stats.",
                usage: "render.stats",
                examples: &["render.stats", "render stats", "fps"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "render.plan",
                aliases: &[],
                category: "render",
                help: "Show resolved frame composition plan.",
                usage: "render.plan",
                examples: &["render.plan"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "render.graph",
                aliases: &[],
                category: "render",
                help: "Show resolved frame graph nodes.",
                usage: "render.graph",
                examples: &["render.graph"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "camera.capture",
                aliases: &[],
                category: "camera",
                help: "Show resolved 2D camera capture input sources.",
                usage: "camera.capture [summary|diagnostics|sources]",
                examples: &[
                    "camera.capture",
                    "camera.capture summary",
                    "camera.capture diagnostics",
                    "camera.capture sources",
                ],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "camera.focus.plan",
                aliases: &["focus.plan"],
                category: "camera",
                help: "Show resolved 2D camera focus/depth layer plan.",
                usage: "camera.focus.plan",
                examples: &["camera.focus.plan", "focus.plan"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "render"
            || matches!(
                command.name.as_str(),
                "render.stats"
                    | "fps"
                    | "render.window"
                    | "camera.capture"
                    | "camera.focus.plan"
                    | "focus.plan"
            )
            || command.name.starts_with("render.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        mut command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        normalize_render_command(&mut command);

        match command.name.as_str() {
            "render.stats" | "fps" => {
                let stats = match ctx.required::<RenderFrameStatsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(format!(
                    "frame={} window={}x{} tilemaps={} sprites={} layered={} layers={} routes={} global_lights={} lightmaps={} light_groups={} vectors={} beacons={} text2d={} particles={} meshes3d={} materials3d={} text3d={} game_ui={} debug_ui={} ui_overlays={} post_fx={} graph_nodes={}",
                    stats.frame_index,
                    stats.window_width,
                    stats.window_height,
                    stats.world_2d_tilemaps,
                    stats.world_2d_sprites,
                    stats.world_2d_layered_images,
                    stats.world_2d_render_layers,
                    stats.world_2d_light_routes,
                    stats.world_2d_global_lights,
                    stats.world_2d_lightmaps,
                    stats.world_2d_light_groups,
                    stats.world_2d_vectors,
                    stats.world_2d_beacons,
                    stats.world_2d_text,
                    stats.world_2d_particles,
                    stats.world_3d_meshes,
                    stats.world_3d_materials,
                    stats.world_3d_text,
                    stats.game_ui_overlays,
                    stats.debug_overlays,
                    stats.ui_overlays,
                    stats.post_fx_effects,
                    stats.render_graph_nodes
                ))
            }
            "render.plan" => {
                let diagnostics = match ctx.required::<RenderCompositionDiagnosticsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(if diagnostics.composition_summary.is_empty() {
                    "render.plan: no composition captured yet".to_owned()
                } else {
                    diagnostics.composition_summary
                })
            }
            "render.graph" => {
                let diagnostics = match ctx.required::<RenderCompositionDiagnosticsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(if diagnostics.graph_summary.is_empty() {
                    "render.graph: no graph captured yet".to_owned()
                } else {
                    let mut output = Vec::new();
                    output.push(diagnostics.graph_summary);
                    output.push("".to_owned());
                    output.push("warnings:".to_owned());
                    if diagnostics.warnings.is_empty() {
                        output.push("none".to_owned());
                    } else {
                        for warning in diagnostics.warnings {
                            output.push(format!("- {warning}"));
                        }
                    }
                    output.join("\n")
                })
            }
            "camera.capture" => {
                let diagnostics = match ctx.required::<RenderCompositionDiagnosticsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                if diagnostics.camera_capture_summary.is_empty() {
                    return ConsoleCommandResult::ok(
                        "camera.capture: no capture input captured yet".to_owned(),
                    );
                }
                let mode = command
                    .args
                    .first()
                    .map(String::as_str)
                    .unwrap_or("summary");
                ConsoleCommandResult::ok(match mode {
                    "summary" => diagnostics.camera_capture_summary,
                    "diagnostics" => {
                        render_camera_capture_diagnostics(&diagnostics.camera_capture_summary)
                    }
                    "sources" => render_camera_capture_sources(&diagnostics.camera_capture_summary),
                    other => {
                        format!("usage: camera.capture [summary|diagnostics|sources], got {other}")
                    }
                })
            }
            "camera.focus.plan" | "focus.plan" => {
                let diagnostics = match ctx.required::<RenderCompositionDiagnosticsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(if diagnostics.camera_focus_plan_summary.is_empty() {
                    "camera.focus.plan: no focus plan captured yet".to_owned()
                } else {
                    diagnostics.camera_focus_plan_summary
                })
            }
            "render.window" => {
                let stats = match ctx.required::<RenderFrameStatsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(format!(
                    "window={}x{}",
                    stats.window_width, stats.window_height
                ))
            }
            "render.scale" => ConsoleCommandResult::ok(
                "render.scale is reserved; add RenderResolutionPolicyService before enabling it",
            ),
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}

fn render_camera_capture_sources(summary: &str) -> String {
    let lines = summary
        .lines()
        .filter(|line| line.trim_start().starts_with("source "))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        "camera.capture sources: none".to_owned()
    } else {
        lines.join("\n")
    }
}

fn render_camera_capture_diagnostics(summary: &str) -> String {
    let mut lines = summary
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            line.starts_with("diagnostic ")
                || line.starts_with("WARNING ")
                || line.contains("visual_source_not_produced")
                || line.contains("visual_source_asset_backed")
                || line.contains("visual_source_derived")
                || line.contains("visual_source_missing")
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if lines.is_empty() {
        lines.extend(summary.lines().filter_map(|line| {
            let line = line.trim_start();
            if line.starts_with("source ") && !line.contains("availability=produced") {
                Some(format!("WARNING visual_source_not_produced {line}"))
            } else {
                None
            }
        }));
    }

    if lines.is_empty() {
        "camera.capture diagnostics: none".to_owned()
    } else {
        lines.join("\n")
    }
}

fn normalize_render_command(command: &mut ParsedConsoleCommand) {
    if command.name != "render" {
        return;
    }

    let Some(verb) = command.args.first().cloned() else {
        command.name = "render.stats".to_owned();
        return;
    };

    command.name = format!("render.{verb}");
    command.args.remove(0);
}

#[cfg(test)]
mod tests {
    use super::RenderConsoleCommandHandler;
    use crate::{ParsedConsoleCommand, RuntimeConsoleCommandHandler};

    #[test]
    fn render_does_not_claim_root_stats() {
        let handler = RenderConsoleCommandHandler;
        let command = ParsedConsoleCommand {
            raw: "stats".to_owned(),
            name: "stats".to_owned(),
            args: Vec::new(),
        };

        assert!(!handler.can_handle(&command));
    }
}
