use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneParticles2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneParticles2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-particles-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_particles::can_handle_particles_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_particles::handle_particles_scene_command(
            amigo_2d_particles::ParticlesSceneCommandContext {
                scene_service: ctx.scene_service,
                particle2d_scene_service: ctx.particle2d_scene_service,
                global_light2d_scene_service: ctx.global_light2d_scene_service,
                lightmap2d_scene_service: ctx.lightmap2d_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        for warning in outcome.warnings {
            ctx.dev_console_state.write_line(warning);
        }

        ctx.dev_console_state.write_line(format!(
            "queued 2d particle emitter `{}` from mod `{}`",
            outcome.entity_name, outcome.source_mod
        ));

        Ok(())
    }
}


