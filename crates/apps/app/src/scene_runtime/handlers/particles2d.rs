use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_2d_particles::{ParticleEmitter2d, ParticleEmitter2dCommand};
use amigo_scene::{LightReceiver2dBindingSceneCommand, ParticleEmitter2dSceneCommand};

pub(crate) struct SceneParticles2dCommandHandler;

impl SceneCommandHandler for SceneParticles2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-particles-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueParticleEmitter2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueParticleEmitter2d { command } => {
                warn_particle_lightmap_issues(ctx, &command);
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.particle2d_scene_service
                    .queue_emitter(ParticleEmitter2dCommand {
                        entity_id: entity,
                        entity_name: command.entity_name.clone(),
                        emitter: ParticleEmitter2d::from_scene_command(&command),
                    });
                ctx.scene_event_queue
                    .publish(SceneEvent::ParticleEmitterQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d particle emitter `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}

fn warn_particle_lightmap_issues(
    ctx: &AppSceneCommandContext<'_>,
    command: &ParticleEmitter2dSceneCommand,
) {
    let Some(binding) = command.material.lightmap.as_ref() else {
        return;
    };

    if binding.source.trim().is_empty() {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` has an empty lightmap source",
            command.entity_name
        ));
    }
    if binding.channel.trim().is_empty() {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` has an empty lightmap channel",
            command.entity_name
        ));
    }
    warn_particle_lightmap_numeric_issues(ctx, command, binding);

    let lightmaps = ctx.lightmap2d_scene_service.commands();
    let Some(source) = lightmaps.iter().find(|source| source.id == binding.source) else {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` references missing lightmap source `{}`",
            command.entity_name, binding.source
        ));
        return;
    };

    if !source
        .channels
        .iter()
        .any(|channel| channel.id == binding.channel)
    {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` references missing lightmap channel `{}` on source `{}`",
            command.entity_name, binding.channel, binding.source
        ));
    }

    let global_lights = ctx.global_light2d_scene_service.commands();
    for global_light in &binding.global_lights {
        if global_light.id.trim().is_empty() {
            ctx.dev_console_state.write_line(format!(
                "2d particle emitter `{}` has a lightmap global light with an empty id",
                command.entity_name
            ));
            continue;
        }
        if !global_lights
            .iter()
            .any(|light| light.id == global_light.id)
        {
            ctx.dev_console_state.write_line(format!(
                "2d particle emitter `{}` references missing global 2d light `{}`",
                command.entity_name, global_light.id
            ));
        }
    }
}

fn warn_particle_lightmap_numeric_issues(
    ctx: &AppSceneCommandContext<'_>,
    command: &ParticleEmitter2dSceneCommand,
    binding: &LightReceiver2dBindingSceneCommand,
) {
    if binding.sample_points == 0 {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` lightmap sample_points should be at least 1",
            command.entity_name
        ));
    }
    if !binding.radius_px.is_finite() || binding.radius_px < 0.0 {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` lightmap radius_px should be finite and non-negative",
            command.entity_name
        ));
    }
    if !binding.exposure.is_finite() || binding.exposure < 0.0 {
        ctx.dev_console_state.write_line(format!(
            "2d particle emitter `{}` lightmap exposure should be finite and non-negative",
            command.entity_name
        ));
    }
    for global_light in &binding.global_lights {
        if !global_light.response.is_finite() || global_light.response < 0.0 {
            ctx.dev_console_state.write_line(format!(
                "2d particle emitter `{}` global light `{}` response should be finite and non-negative",
                command.entity_name, global_light.id
            ));
        }
    }
}
