use amigo_light_2d_plugin::{GlobalLight2dSceneService, LightMap2dSceneService};
use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    LightReceiver2dBindingSceneCommand, ParticleEmitter2dSceneCommand, SceneCommand, SceneEvent,
    SceneEventQueue, SceneService, format_scene_command,
};

use crate::{Particle2dSceneService, ParticleEmitter2d, ParticleEmitter2dCommand};

pub struct Particles2dSceneCommandHandler;

pub struct ParticlesSceneCommandContext<'a> {
    pub scene_service: &'a SceneService,
    pub particle2d_scene_service: &'a Particle2dSceneService,
    pub global_light2d_scene_service: &'a GlobalLight2dSceneService,
    pub lightmap2d_scene_service: &'a LightMap2dSceneService,
    pub scene_event_queue: &'a SceneEventQueue,
}

#[derive(Debug, Clone)]
pub struct ParticlesSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
    pub warnings: Vec<String>,
}

pub fn can_handle_particles_scene_command(command: &SceneCommand) -> bool {
    matches!(
        command,
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::PARTICLE_EMITTER_2D_PLUGIN_SCENE_COMMAND_TYPE
    )
}

pub fn handle_particles_scene_command(
    ctx: ParticlesSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<ParticlesSceneCommandOutcome> {
    match command {
        SceneCommand::Plugin { command }
            if command.command_type == amigo_scene::PARTICLE_EMITTER_2D_PLUGIN_SCENE_COMMAND_TYPE =>
        {
            let Some(command) = command
                .payload_as::<amigo_scene::ParticleEmitter2dSceneCommand>()
                .cloned()
            else {
                return Err(AmigoError::Message(
                    "particles-2d plugin command payload mismatch".to_owned(),
                ));
            };
            let warnings = collect_particle_lightmap_warnings(
                &command,
                ctx.lightmap2d_scene_service,
                ctx.global_light2d_scene_service,
            );
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
            Ok(ParticlesSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
                warnings,
            })
        }
        _ => Err(AmigoError::Message(format!(
            "particles-2d cannot handle command {}",
            format_scene_command(&command)
        ))),
    }
}

fn collect_particle_lightmap_warnings(
    command: &ParticleEmitter2dSceneCommand,
    lightmap2d_scene_service: &LightMap2dSceneService,
    global_light2d_scene_service: &GlobalLight2dSceneService,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(binding) = command.material.light_receiver.as_ref() else {
        return warnings;
    };

    if binding.source.trim().is_empty() {
        warnings.push(format!(
            "2d particle emitter `{}` has an empty lightmap source",
            command.entity_name
        ));
    }
    if binding.channel.trim().is_empty() {
        warnings.push(format!(
            "2d particle emitter `{}` has an empty lightmap channel",
            command.entity_name
        ));
    }
    collect_particle_lightmap_numeric_warnings(&mut warnings, command, binding);

    let lightmaps = lightmap2d_scene_service.commands();
    let Some(source) = lightmaps.iter().find(|source| source.id == binding.source) else {
        warnings.push(format!(
            "2d particle emitter `{}` references missing lightmap source `{}`",
            command.entity_name, binding.source
        ));
        return warnings;
    };

    if !source
        .channels
        .iter()
        .any(|channel| channel.id == binding.channel)
    {
        warnings.push(format!(
            "2d particle emitter `{}` references missing lightmap channel `{}` on source `{}`",
            command.entity_name, binding.channel, binding.source
        ));
    }

    let global_lights = global_light2d_scene_service.commands();
    for global_light in &binding.global_lights {
        if global_light.id.trim().is_empty() {
            warnings.push(format!(
                "2d particle emitter `{}` has a lightmap global light with an empty id",
                command.entity_name
            ));
            continue;
        }
        if !global_lights
            .iter()
            .any(|light| light.id == global_light.id)
        {
            warnings.push(format!(
                "2d particle emitter `{}` references missing global 2d light `{}`",
                command.entity_name, global_light.id
            ));
        }
    }

    warnings
}

fn collect_particle_lightmap_numeric_warnings(
    warnings: &mut Vec<String>,
    command: &ParticleEmitter2dSceneCommand,
    binding: &LightReceiver2dBindingSceneCommand,
) {
    if binding.sample_points == 0 {
        warnings.push(format!(
            "2d particle emitter `{}` lightmap sample_points should be at least 1",
            command.entity_name
        ));
    }
    if !binding.radius_px.is_finite() || binding.radius_px < 0.0 {
        warnings.push(format!(
            "2d particle emitter `{}` lightmap radius_px should be finite and non-negative",
            command.entity_name
        ));
    }
    if !binding.exposure.is_finite() || binding.exposure < 0.0 {
        warnings.push(format!(
            "2d particle emitter `{}` lightmap exposure should be finite and non-negative",
            command.entity_name
        ));
    }
    for global_light in &binding.global_lights {
        if !global_light.response.is_finite() || global_light.response < 0.0 {
            warnings.push(format!(
                "2d particle emitter `{}` global light `{}` response should be finite and non-negative",
                command.entity_name, global_light.id
            ));
        }
    }
}

impl amigo_scene::RuntimeSceneCommandHandler for Particles2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_particles_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let particle2d_scene_service = runtime.required::<Particle2dSceneService>()?;
        let global_light2d_scene_service = runtime.required::<GlobalLight2dSceneService>()?;
        let lightmap2d_scene_service = runtime.required::<LightMap2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        handle_particles_scene_command(
            ParticlesSceneCommandContext {
                scene_service: scene_service.as_ref(),
                particle2d_scene_service: particle2d_scene_service.as_ref(),
                global_light2d_scene_service: global_light2d_scene_service.as_ref(),
                lightmap2d_scene_service: lightmap2d_scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
