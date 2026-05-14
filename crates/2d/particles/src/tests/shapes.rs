use super::common::{TEST_EMITTER, test_emitter, test_input};
use super::*;

#[test]
fn set_shape_changes_draw_command_shape() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));

    assert!(service.set_shape(TEST_EMITTER, ParticleShape2d::Line { length: 12.0 }));
    service.tick(&[test_input()], 0.1);

    assert_eq!(
        service.draw_commands()[0].shape,
        ParticleShape2d::Line { length: 12.0 }
    );
}

#[test]
fn shape_choices_pick_particle_shape_at_spawn() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(false);
    command.emitter.shape_choices = vec![WeightedParticleShape2d {
        shape: ParticleShape2d::Line { length: 14.0 },
        weight: 1.0,
    }];
    service.queue_emitter(command);

    assert!(service.burst(TEST_EMITTER, 1));
    service.tick(&[test_input()], 0.016);

    let draw = service.draw_commands();
    assert_eq!(draw.len(), 1);
    assert_eq!(draw[0].shape, ParticleShape2d::Line { length: 14.0 });
}

#[test]
fn shape_over_lifetime_overrides_draw_shape_by_age() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.shape = ParticleShape2d::Circle { segments: 8 };
    command.emitter.shape_choices = vec![WeightedParticleShape2d {
        shape: ParticleShape2d::Line { length: 14.0 },
        weight: 1.0,
    }];
    command.emitter.shape_over_lifetime = vec![
        ParticleShapeKeyframe2d {
            t: 0.0,
            shape: ParticleShape2d::Quad,
        },
        ParticleShapeKeyframe2d {
            t: 0.5,
            shape: ParticleShape2d::Circle { segments: 12 },
        },
    ];
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);
    assert_eq!(service.draw_commands()[0].shape, ParticleShape2d::Quad);

    service.set_active(TEST_EMITTER, false);
    service.tick(&[test_input()], 0.5);

    assert_eq!(
        service.draw_commands()[0].shape,
        ParticleShape2d::Circle { segments: 12 }
    );
}

#[test]
fn draw_command_preserves_line_anchor() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.shape = ParticleShape2d::Line { length: 16.0 };
    command.emitter.line_anchor = ParticleLineAnchor2d::Start;
    service.queue_emitter(command);

    service.tick(&[test_input()], 0.1);

    assert_eq!(
        service.draw_commands()[0].line_anchor,
        ParticleLineAnchor2d::Start
    );
}
