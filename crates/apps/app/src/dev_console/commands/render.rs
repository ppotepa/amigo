use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;
use crate::render_runtime::RenderFrameStatsService;

pub(crate) struct RenderConsoleCommandHandler;

impl ConsoleCommandHandler for RenderConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "render-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![ConsoleCommandDescriptor {
            name: "render.stats",
            aliases: &["stats", "fps"],
            category: "render",
            help: "Show current render frame stats.",
            usage: "render.stats",
            examples: &["render.stats"],
            dev_only: true,
        }]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        matches!(
            command.name.as_str(),
            "render.stats" | "stats" | "fps" | "render.window"
        ) || command.name.starts_with("render.")
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        match command.name.as_str() {
            "render.stats" | "stats" | "fps" => {
                let stats = match ctx.required::<RenderFrameStatsService>() {
                    Ok(service) => service.snapshot(),
                    Err(error) => return ConsoleCommandResult::error(error.to_string()),
                };
                ConsoleCommandResult::ok(format!(
                    "frame={} window={}x{} tilemaps={} sprites={} layered={} layers={} routes={} global_lights={} lightmaps={} light_groups={} vectors={} text2d={} particles={} meshes3d={} materials3d={} text3d={} ui_overlays={}",
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
                    stats.world_2d_text,
                    stats.world_2d_particles,
                    stats.world_3d_meshes,
                    stats.world_3d_materials,
                    stats.world_3d_text,
                    stats.ui_overlays
                ))
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
