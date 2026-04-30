use super::super::*;
use crate::runtime_context::RuntimeContext;
use amigo_2d_particles::Particle2dEmitterRuntimeInput;
use amigo_math::{Transform2, Vec2};

pub(crate) fn tick_particles_2d_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let motion_scene_service = ctx.required::<Motion2dSceneService>()?;
    let particle_scene_service = ctx.required::<Particle2dSceneService>()?;

    let inputs = particle_scene_service
        .emitters()
        .into_iter()
        .filter_map(|command| {
            let source_name = command
                .emitter
                .attached_to
                .as_deref()
                .unwrap_or(command.entity_name.as_str());
            let source_transform = scene_service.transform_of(source_name)?;
            Some(Particle2dEmitterRuntimeInput {
                emitter_entity_name: command.entity_name.clone(),
                source_entity_name: source_name.to_owned(),
                source_transform: Transform2 {
                    translation: Vec2::new(
                        source_transform.translation.x,
                        source_transform.translation.y,
                    ),
                    rotation_radians: source_transform.rotation_euler.z,
                    scale: Vec2::new(source_transform.scale.x, source_transform.scale.y),
                },
                source_velocity: motion_scene_service.current_velocity(source_name),
                source_visible: scene_service.is_visible(source_name),
                source_simulation_enabled: scene_service.is_simulation_enabled(source_name),
            })
        })
        .collect::<Vec<_>>();

    particle_scene_service.tick(&inputs, delta_seconds);

    Ok(())
}
