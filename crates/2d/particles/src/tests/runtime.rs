use super::common::{TEST_EMITTER, TEST_SOURCE_EMITTER, test_emitter, test_input};
use super::*;
use amigo_math::Vec2;

#[test]
fn runtime_setter_updates_velocity_mode() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(false));

    assert!(service.set_velocity_mode(TEST_EMITTER, ParticleVelocityMode2d::SourceInertial));

    let emitter = service.emitter(TEST_EMITTER).expect("emitter should exist");
    assert_eq!(
        emitter.emitter.velocity_mode,
        ParticleVelocityMode2d::SourceInertial
    );
}

#[test]
fn set_wind_replaces_existing_wind_force() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.forces = vec![ParticleForce2d::Wind {
        velocity: Vec2::new(1.0, 0.0),
        strength: 1.0,
    }];
    service.queue_emitter(command);

    assert!(service.set_wind(TEST_EMITTER, 20.0, 5.0, 2.0));

    let emitter = service.emitter(TEST_EMITTER).expect("emitter should exist");
    assert_eq!(emitter.emitter.forces.len(), 1);
    assert!(matches!(
        emitter.emitter.forces[0],
        ParticleForce2d::Wind {
            velocity,
            strength
        } if velocity == Vec2::new(20.0, 5.0) && (strength - 2.0).abs() < 0.001
    ));
}

#[test]
fn runtime_setters_update_jitter_direction_and_inheritance() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(false));

    assert!(service.set_lifetime_jitter(TEST_EMITTER, 0.25));
    assert!(service.set_speed_jitter(TEST_EMITTER, 8.0));
    assert!(service.set_local_direction_radians(TEST_EMITTER, 1.5));
    assert!(service.set_inherit_parent_velocity(TEST_EMITTER, 0.35));

    let emitter = service.emitter(TEST_EMITTER).expect("emitter should exist");
    assert_eq!(emitter.emitter.lifetime_jitter, 0.25);
    assert_eq!(emitter.emitter.speed_jitter, 8.0);
    assert_eq!(emitter.emitter.local_direction_radians, 1.5);
    assert_eq!(emitter.emitter.inherit_parent_velocity, 0.35);
}

#[test]
fn copy_emitter_config_replaces_target_emitter_and_clears_live_particles() {
    use amigo_scene::SceneEntityId;

    let service = Particle2dSceneService::default();
    let mut source = test_emitter(false);
    source.entity_id = SceneEntityId::new(2);
    source.entity_name = TEST_SOURCE_EMITTER.to_owned();
    source.emitter.spawn_rate = 44.0;
    source.emitter.initial_speed = 33.0;
    source.emitter.shape = ParticleShape2d::Line { length: 18.0 };
    source.emitter.spawn_area = ParticleSpawnArea2d::Rect {
        size: Vec2::new(24.0, 8.0),
    };

    service.queue_emitter(test_emitter(true));
    service.queue_emitter(source);
    service.tick(&[test_input()], 0.2);
    assert!(service.particle_count(TEST_EMITTER) > 0);

    assert!(service.copy_emitter_config(TEST_SOURCE_EMITTER, TEST_EMITTER));

    let copied = service
        .emitter(TEST_EMITTER)
        .expect("target emitter should exist");
    assert_eq!(copied.emitter.spawn_rate, 44.0);
    assert_eq!(copied.emitter.initial_speed, 33.0);
    assert_eq!(copied.emitter.shape, ParticleShape2d::Line { length: 18.0 });
    assert_eq!(
        copied.emitter.spawn_area,
        ParticleSpawnArea2d::Rect {
            size: Vec2::new(24.0, 8.0)
        }
    );
    assert_eq!(service.particle_count(TEST_EMITTER), 0);
}
