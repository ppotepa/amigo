use super::common::{TEST_EMITTER, test_emitter, test_input};
use super::*;
use amigo_math::Curve1d;
use amigo_math::Vec2;

#[test]
fn active_emitter_spawns_particles() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));

    service.tick(&[test_input()], 0.2);

    assert_eq!(service.particle_count(TEST_EMITTER), 2);
}

#[test]
fn inactive_emitter_does_not_spawn() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(false));

    service.tick(&[test_input()], 0.5);

    assert_eq!(service.particle_count(TEST_EMITTER), 0);
}

#[test]
fn existing_particles_age_and_expire() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));
    service.tick(&[test_input()], 0.2);
    service.set_active(TEST_EMITTER, false);

    service.tick(&[test_input()], 1.0);

    assert_eq!(service.particle_count(TEST_EMITTER), 0);
}

#[test]
fn max_particles_caps_runtime_particles() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.max_particles = 3;
    command.emitter.spawn_rate = 100.0;
    service.queue_emitter(command);

    service.tick(&[test_input()], 1.0);

    assert_eq!(service.particle_count(TEST_EMITTER), 3);
}

#[test]
fn size_curve_grows_particle_over_lifetime() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);
    service.tick(&[test_input()], 0.4);

    let draw = service.draw_commands();

    assert!(draw[0].size > 2.0);
}

#[test]
fn emission_rate_curve_modulates_spawn_count() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.emission_rate_curve = Curve1d::Constant(0.5);
    service.queue_emitter(command);

    service.tick(&[test_input()], 1.0);

    assert_eq!(service.particle_count(TEST_EMITTER), 5);
}

#[test]
fn burst_spawns_particles_when_emitter_inactive() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(false));

    assert!(service.burst(TEST_EMITTER, 4));
    service.tick(&[test_input()], 0.1);

    assert_eq!(service.particle_count(TEST_EMITTER), 4);
}

#[test]
fn burst_respects_max_particles() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(false);
    command.emitter.max_particles = 3;
    service.queue_emitter(command);

    assert!(service.burst(TEST_EMITTER, 8));
    service.tick(&[test_input()], 0.1);

    assert_eq!(service.particle_count(TEST_EMITTER), 3);
}

#[test]
fn burst_missing_emitter_returns_false() {
    let service = Particle2dSceneService::default();

    assert!(!service.burst("missing", 1));
}

#[test]
fn burst_at_spawns_particles_at_requested_position() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(false);
    command.emitter.initial_speed = 0.0;
    command.emitter.spawn_area = ParticleSpawnArea2d::Rect {
        size: Vec2::new(100.0, 100.0),
    };
    service.queue_emitter(command);

    assert!(service.burst_at(TEST_EMITTER, Vec2::new(42.0, -24.0), 3));
    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();
    assert_eq!(draw.len(), 3);
    assert!(
        draw.iter()
            .all(|command| command.position == Vec2::new(42.0, -24.0))
    );
}

#[test]
fn burst_at_rejects_invalid_position() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(false));

    assert!(!service.burst_at(TEST_EMITTER, Vec2::new(f32::NAN, 0.0), 1));
    assert!(!service.burst_at("missing", Vec2::ZERO, 1));
}

#[test]
fn set_max_particles_caps_future_particles() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.spawn_rate = 100.0;
    service.queue_emitter(command);

    assert!(service.set_max_particles(TEST_EMITTER, 2));
    service.tick(&[test_input()], 1.0);

    assert_eq!(service.particle_count(TEST_EMITTER), 2);
}
