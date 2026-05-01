use super::common::{test_emitter, test_input};
use super::*;
use amigo_math::Vec2;

#[test]
fn rect_spawn_area_offsets_particles_within_bounds() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.spawn_area = ParticleSpawnArea2d::Rect {
        size: Vec2::new(10.0, 20.0),
    };
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);

    let position = service.draw_commands()[0].position;
    assert!(position.x.abs() <= 5.0);
    assert!(position.y.abs() <= 10.0);
}

#[test]
fn circle_spawn_area_offsets_particles_within_radius() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.spawn_area = ParticleSpawnArea2d::Circle { radius: 12.0 };
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);

    let position = service.draw_commands()[0].position;
    assert!((position.x * position.x + position.y * position.y).sqrt() <= 12.0);
}

#[test]
fn line_spawn_area_offsets_particles_within_length() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.spawn_area = ParticleSpawnArea2d::Line { length: 20.0 };
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);

    let position = service.draw_commands()[0].position;
    assert!(position.x.abs() <= 10.0);
}

#[test]
fn ring_spawn_area_offsets_particles_between_radii() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.spawn_area = ParticleSpawnArea2d::Ring {
        inner_radius: 8.0,
        outer_radius: 16.0,
    };
    command.emitter.initial_speed = 0.0;
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);

    let position = service.draw_commands()[0].position;
    let radius = (position.x * position.x + position.y * position.y).sqrt();
    assert!((8.0..=16.0).contains(&radius));
}
