use crate::{PostFx2d, PostFx2dService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostFxDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct PostFxDevConsoleCommandContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub fn can_handle_post_fx_dev_console_command(name: &str) -> bool {
    matches!(name, "postfx.cert" | "postfx.stats")
}

pub fn handle_post_fx_dev_console_command(
    ctx: PostFxDevConsoleCommandContext<'_>,
    name: &str,
    _args: &[String],
) -> PostFxDevConsoleCommandOutcome {
    match name {
        "postfx.cert" => {
            let reports = ctx.post_fx_service.lens_certification_reports();
            if reports.is_empty() {
                return PostFxDevConsoleCommandOutcome::Handled(
                    "postfx.cert: no LensDroplets2D reports".to_owned(),
                );
            }
            let lines = reports
                .into_iter()
                .map(|report| {
                    format!(
                        "LensDroplets2D accepted={} cost={:.1} issues={} renderer={}",
                        report.accepted,
                        report.cost_score,
                        report.issues.len(),
                        ctx.post_fx_service.renderer_mode()
                    )
                })
                .collect::<Vec<_>>();
            PostFxDevConsoleCommandOutcome::Handled(lines.join("\n"))
        }
        "postfx.stats" => {
            let stack = ctx.post_fx_service.scene_stack();
            let lens_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::LensDroplets(_)));
            let wet_active = stack
                .effects
                .iter()
                .any(|effect| matches!(effect, PostFx2d::WetReflections(_)));
            PostFxDevConsoleCommandOutcome::Handled(format!(
                "postfx.effects={} lens_droplets_active={} wet_reflections_active={} renderer_mode={} overlay_supported={} blur_supported={} world_offscreen_post_fx_supported={}",
                stack.effects.len(),
                lens_active,
                wet_active,
                ctx.post_fx_service.renderer_mode(),
                ctx.post_fx_service.supports_lens_droplets_overlay(),
                ctx.post_fx_service.supports_lens_droplets_blur(),
                ctx.post_fx_service.supports_world_offscreen_post_fx()
            ))
        }
        _ => PostFxDevConsoleCommandOutcome::Unhandled,
    }
}
