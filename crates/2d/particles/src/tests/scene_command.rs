use amigo_math::{ColorRgba, Curve1d, Vec2};
use amigo_scene::{
    Material2dLightingModeSceneCommand, ParticleAlignMode2dSceneCommand,
    ParticleBlendMode2dSceneCommand, ParticleEmitter2dSceneCommand,
    ParticleLineAnchor2dSceneCommand, ParticleMaterial2dSceneCommand, ParticleShape2dSceneCommand,
    ParticleSimulationSpace2dSceneCommand, ParticleSpawnArea2dSceneCommand,
    ParticleVelocityMode2dSceneCommand, SceneCommand, SceneEvent, SceneEventQueue, SceneService,
};

use super::*;

#[test]
fn particles_scene_command_handler_queues_emitter_and_event() {
    let scene = SceneService::default();
    let particles = Particle2dSceneService::default();
    let global_lights = amigo_2d_lighting::GlobalLight2dSceneService::default();
    let lightmaps = amigo_2d_lighting::LightMap2dSceneService::default();
    let events = SceneEventQueue::default();
    let command = particle_emitter_command();

    assert!(can_handle_particles_scene_command(
        &SceneCommand::QueueParticleEmitter2d {
            command: command.clone()
        }
    ));

    let outcome = handle_particles_scene_command(
        ParticlesSceneCommandContext {
            scene_service: &scene,
            particle2d_scene_service: &particles,
            global_light2d_scene_service: &global_lights,
            lightmap2d_scene_service: &lightmaps,
            scene_event_queue: &events,
        },
        SceneCommand::QueueParticleEmitter2d { command },
    )
    .expect("particle scene command should be handled");

    assert_eq!(outcome.entity_name, "embers");
    assert_eq!(outcome.source_mod, "test-mod");
    assert!(outcome.warnings.is_empty());
    assert!(particles.emitter("embers").is_some());

    let entity = scene
        .entity_by_name("embers")
        .expect("particle entity should be spawned");
    assert_eq!(
        events.pending(),
        [SceneEvent::ParticleEmitterQueued {
            entity_id: entity.id.raw(),
            entity_name: "embers".to_owned()
        }]
    );
}

fn particle_emitter_command() -> ParticleEmitter2dSceneCommand {
    ParticleEmitter2dSceneCommand {
        source_mod: "test-mod".to_owned(),
        entity_name: "embers".to_owned(),
        render_layer: "effects".to_owned(),
        attached_to: None,
        local_offset: Vec2::ZERO,
        local_direction_radians: 0.0,
        spawn_area: ParticleSpawnArea2dSceneCommand::Point,
        active: true,
        spawn_rate: 12.0,
        max_particles: 64,
        particle_lifetime: 1.0,
        lifetime_jitter: 0.0,
        initial_speed: 10.0,
        speed_jitter: 0.0,
        spread_radians: 0.0,
        inherit_parent_velocity: 0.0,
        velocity_mode: ParticleVelocityMode2dSceneCommand::Free,
        simulation_space: ParticleSimulationSpace2dSceneCommand::World,
        initial_size: 2.0,
        final_size: 0.5,
        size_jitter: 0.0,
        color: ColorRgba::WHITE,
        color_ramp: None,
        z_index: 0.0,
        shape: ParticleShape2dSceneCommand::Quad,
        shape_choices: Vec::new(),
        shape_over_lifetime: Vec::new(),
        line_anchor: ParticleLineAnchor2dSceneCommand::Center,
        align: ParticleAlignMode2dSceneCommand::None,
        blend_mode: ParticleBlendMode2dSceneCommand::Alpha,
        motion_stretch: None,
        material: ParticleMaterial2dSceneCommand {
            lighting_mode: Material2dLightingModeSceneCommand::Unlit,
            receives_light: false,
            light_response: 1.0,
            light_receiver: None,
        },
        light: None,
        emission_rate_curve: Curve1d::Linear,
        size_curve: Curve1d::Linear,
        alpha_curve: Curve1d::Linear,
        speed_curve: Curve1d::Linear,
        forces: Vec::new(),
    }
}
