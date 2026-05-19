use super::common::{TEST_EMITTER, test_emitter, test_input};
use super::*;
use amigo_math::Vec2;

#[test]
fn parent_velocity_is_inherited() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.inherit_parent_velocity = 0.5;
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);
    let mut input = test_input();
    input.source_velocity = Vec2::new(20.0, 0.0);

    service.tick(&[input.clone()], 0.1);
    service.tick(&[input], 0.1);
    let first = service.draw_commands().remove(0);

    assert!((first.position.x - 1.0).abs() < 0.001);
}

#[test]
fn source_inertial_velocity_mode_inherits_full_source_velocity() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.velocity_mode = ParticleVelocityMode2d::SourceInertial;
    command.emitter.inherit_parent_velocity = 0.0;
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);
    let mut input = test_input();
    input.source_velocity = Vec2::new(20.0, -10.0);

    service.tick(&[input.clone()], 0.1);
    service.tick(&[input], 0.1);
    let first = service.draw_commands().remove(0);

    assert!((first.position.x - 2.0).abs() < 0.001);
    assert!((first.position.y + 1.0).abs() < 0.001);
}

#[test]
fn gravity_changes_particle_velocity() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.initial_speed = 0.0;
    command.emitter.forces = vec![ParticleForce2d::Gravity {
        acceleration: Vec2::new(0.0, -10.0),
    }];
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);

    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();
    assert!(draw[0].position.y < 0.0);
}

#[test]
fn drag_reduces_particle_velocity() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.forces = vec![ParticleForce2d::Drag { coefficient: 5.0 }];
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);

    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();
    assert!(draw[0].position.x < 2.0);
}

#[test]
fn wind_moves_velocity_toward_wind_velocity() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.initial_speed = 0.0;
    command.emitter.forces = vec![ParticleForce2d::Wind {
        velocity: Vec2::new(20.0, 0.0),
        strength: 10.0,
    }];
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);

    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();
    assert!(draw[0].position.x > 0.0);
}
