use super::common::{TEST_EMITTER, test_emitter};
use super::*;
use amigo_fx::ColorRamp;
use amigo_math::ColorRgba;

#[test]
fn exports_particle_emitter_yaml_from_runtime_config() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(false);
    command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(0.0, 1.0, 0.0, 1.0)));
    command.emitter.spawn_area = ParticleSpawnArea2d::Ring {
        inner_radius: 4.0,
        outer_radius: 12.0,
    };
    command.emitter.shape = ParticleShape2d::Line { length: 14.0 };
    command.emitter.shape_choices = vec![WeightedParticleShape2d {
        shape: ParticleShape2d::Quad,
        weight: 1.0,
    }];
    command.emitter.shape_over_lifetime = vec![ParticleShapeKeyframe2d {
        t: 0.5,
        shape: ParticleShape2d::Circle { segments: 12 },
    }];
    command.emitter.align = ParticleAlignMode2d::Emitter;
    command.emitter.forces = vec![ParticleForce2d::Drag { coefficient: 0.5 }];
    service.queue_emitter(command);

    let yaml = service
        .emitter_yaml(TEST_EMITTER)
        .expect("emitter yaml should exist");

    assert!(yaml.contains("type: ParticleEmitter2D"));
    assert!(yaml.contains("color_ramp:"));
    assert!(yaml.contains("kind: ring"));
    assert!(yaml.contains("kind: line"));
    assert!(yaml.contains("shape_choices:"));
    assert!(yaml.contains("shape_over_lifetime:"));
    assert!(yaml.contains("kind: quad"));
    assert!(yaml.contains("segments: 12"));
    assert!(yaml.contains("align: emitter"));
    assert!(yaml.contains("kind: drag"));
}
