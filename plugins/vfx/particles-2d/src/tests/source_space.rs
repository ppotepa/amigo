use super::common::{TEST_EMITTER, test_emitter, test_input};
use super::*;
use amigo_math::Vec2;

#[test]
fn source_simulation_space_draws_particles_from_updated_source_anchor() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.simulation_space = ParticleSimulationSpace2d::Source;
    command.emitter.local_offset = Vec2::new(-15.0, 0.0);
    command.emitter.initial_speed = 0.0;
    command.emitter.spawn_rate = 10.0;
    service.queue_emitter(command);

    let mut input = test_input();
    input.source_transform.translation = Vec2::new(10.0, 20.0);
    input.source_transform.rotation_radians = std::f32::consts::FRAC_PI_2;
    service.tick(&[input.clone()], 0.1);

    let first_position = service.draw_commands()[0].position;
    assert!((first_position.x - 10.0).abs() < 0.001);
    assert!((first_position.y - 5.0).abs() < 0.001);

    service.set_active(TEST_EMITTER, false);
    input.source_transform.translation = Vec2::new(20.0, 20.0);
    service.tick(&[input], 0.1);

    let moved_position = service.draw_commands()[0].position;
    assert!((moved_position.x - 20.0).abs() < 0.001);
    assert!((moved_position.y - 5.0).abs() < 0.001);
}
