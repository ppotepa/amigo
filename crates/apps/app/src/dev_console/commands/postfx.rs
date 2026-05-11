use crate::dev_console::dispatcher::ConsoleCommandContext;
use crate::dev_console::model::{
    ConsoleCommandDescriptor, ConsoleCommandResult, ParsedConsoleCommand,
};
use crate::dev_console::registry::ConsoleCommandHandler;

pub(crate) struct PostFxConsoleCommandHandler;

impl ConsoleCommandHandler for PostFxConsoleCommandHandler {
    fn name(&self) -> &'static str {
        "postfx-console"
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        vec![
            ConsoleCommandDescriptor {
                name: "postfx.cert",
                aliases: &[],
                category: "render",
                help: "Show LensDroplets2D certification reports.",
                usage: "postfx.cert",
                examples: &["postfx.cert"],
                dev_only: true,
            },
            ConsoleCommandDescriptor {
                name: "postfx.stats",
                aliases: &[],
                category: "render",
                help: "Show active 2D post-fx stack stats.",
                usage: "postfx.stats",
                examples: &["postfx.stats"],
                dev_only: true,
            },
        ]
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        command.name == "postfx.cert" || command.name == "postfx.stats"
    }

    fn handle(
        &self,
        ctx: &ConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult {
        let post_fx = match ctx.required::<amigo_2d_post_fx::PostFx2dService>() {
            Ok(service) => service,
            Err(error) => return ConsoleCommandResult::error(error.to_string()),
        };

        match command.name.as_str() {
            "postfx.cert" => {
                let reports = post_fx.lens_certification_reports();
                if reports.is_empty() {
                    return ConsoleCommandResult::ok("postfx.cert: no LensDroplets2D reports");
                }

                let lines = reports
                    .into_iter()
                    .map(|report| {
                        format!(
                            "LensDroplets2D accepted={} cost={:.1} issues={} renderer={}",
                            report.accepted,
                            report.cost_score,
                            report.issues.len(),
                            post_fx.renderer_mode()
                        )
                    })
                    .collect::<Vec<_>>();
                ConsoleCommandResult::ok(lines.join("\n"))
            }
            "postfx.stats" => {
                let stack = post_fx.scene_stack();
                let lens_active = stack
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, amigo_2d_post_fx::PostFx2d::LensDroplets(_)));
                let wet_active = stack.effects.iter().any(|effect| {
                    matches!(effect, amigo_2d_post_fx::PostFx2d::WetReflections(_))
                });
                ConsoleCommandResult::ok(format!(
                    "postfx.effects={} lens_droplets_active={} wet_reflections_active={} renderer_mode={} overlay_supported={} blur_supported={} world_offscreen_post_fx_supported={}",
                    stack.effects.len(),
                    lens_active,
                    wet_active,
                    post_fx.renderer_mode(),
                    post_fx.supports_lens_droplets_overlay(),
                    post_fx.supports_lens_droplets_blur(),
                    post_fx.supports_world_offscreen_post_fx()
                ))
            }
            _ => ConsoleCommandResult::unknown(command.raw),
        }
    }
}
