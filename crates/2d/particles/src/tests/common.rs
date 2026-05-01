use crate::{
    Particle2dEmitterRuntimeInput, ParticleAlignMode2d, ParticleBlendMode2d, ParticleEmitter2d,
    ParticleEmitter2dCommand, ParticleLineAnchor2d, ParticleMaterial2d, ParticleShape2d,
    ParticleSimulationSpace2d, ParticleSpawnArea2d, ParticleVelocityMode2d,
};
use amigo_math::{ColorRgba, Curve1d, Transform2, Vec2};
use amigo_scene::SceneEntityId;

pub const TEST_EMITTER: &str = "test-emitter";
pub const TEST_SOURCE: &str = "test-ship";
pub const TEST_SOURCE_EMITTER: &str = "source-emitter";

pub fn test_emitter(active: bool) -> ParticleEmitter2dCommand {
    ParticleEmitter2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: TEST_EMITTER.to_owned(),
        emitter: ParticleEmitter2d {
            attached_to: Some(TEST_SOURCE.to_owned()),
            local_offset: Vec2::ZERO,
            local_direction_radians: 0.0,
            spawn_area: ParticleSpawnArea2d::Point,
            active,
            spawn_rate: 10.0,
            max_particles: 10,
            particle_lifetime: 1.0,
            lifetime_jitter: 0.0,
            initial_speed: 10.0,
            speed_jitter: 0.0,
            spread_radians: 0.0,
            inherit_parent_velocity: 0.0,
            velocity_mode: ParticleVelocityMode2d::Free,
            simulation_space: ParticleSimulationSpace2d::World,
            initial_size: 2.0,
            final_size: 6.0,
            color: ColorRgba::WHITE,
            color_ramp: None,
            z_index: 1.0,
            shape: ParticleShape2d::Circle { segments: 8 },
            shape_choices: Vec::new(),
            shape_over_lifetime: Vec::new(),
            line_anchor: ParticleLineAnchor2d::Center,
            align: ParticleAlignMode2d::Velocity,
            blend_mode: ParticleBlendMode2d::Alpha,
            motion_stretch: None,
            material: ParticleMaterial2d {
                receives_light: false,
                light_response: 1.0,
            },
            light: None,
            emission_rate_curve: Curve1d::Constant(1.0),
            size_curve: Curve1d::Linear,
            alpha_curve: Curve1d::Constant(1.0),
            speed_curve: Curve1d::Constant(1.0),
            forces: Vec::new(),
        },
    }
}

pub fn test_input() -> Particle2dEmitterRuntimeInput {
    Particle2dEmitterRuntimeInput {
        emitter_entity_name: TEST_EMITTER.to_owned(),
        source_entity_name: TEST_SOURCE.to_owned(),
        source_transform: Transform2::default(),
        source_velocity: Vec2::ZERO,
        source_visible: true,
        source_simulation_enabled: true,
    }
}
