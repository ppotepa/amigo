use super::common::{TEST_EMITTER, test_emitter, test_input};
use super::*;
use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d};

#[test]
fn draw_command_tracks_previous_position_for_motion_stretch() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);
    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();

    assert!(draw[0].position.x > draw[0].previous_position.x);
    assert_eq!(draw[0].previous_position.y, draw[0].position.y);
}

#[test]
fn alpha_curve_fades_particle_over_lifetime() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.alpha_curve = Curve1d::Custom {
        points: vec![
            amigo_math::CurvePoint1d { t: 0.0, value: 1.0 },
            amigo_math::CurvePoint1d { t: 1.0, value: 0.0 },
        ],
    };
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);
    service.tick(&[test_input()], 0.4);

    let draw = service.draw_commands();

    assert!(draw[0].color.a < 1.0);
}

#[test]
fn particle_color_ramp_changes_rgb_over_lifetime() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.color = ColorRgba::new(1.0, 1.0, 1.0, 1.0);
    command.emitter.color_ramp = Some(ColorRamp {
        interpolation: amigo_fx::ColorInterpolation::LinearRgb,
        stops: vec![
            amigo_fx::ColorStop {
                t: 0.0,
                color: ColorRgba::new(1.0, 0.0, 0.0, 1.0),
            },
            amigo_fx::ColorStop {
                t: 1.0,
                color: ColorRgba::new(0.0, 0.0, 1.0, 1.0),
            },
        ],
    });
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);
    service.set_active(TEST_EMITTER, false);
    service.tick(&[test_input()], 0.4);

    let color = service.draw_commands()[0].color;

    assert!(color.r < 1.0);
    assert!(color.b > 0.0);
}

#[test]
fn particle_color_ramp_alpha_multiplies_alpha_curve() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 0.5)));
    command.emitter.alpha_curve = Curve1d::Constant(0.5);
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);

    let color = service.draw_commands()[0].color;

    assert!((color.a - 0.25).abs() < 0.001);
}

#[test]
fn particle_missing_color_ramp_preserves_legacy_color() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.color = ColorRgba::new(0.25, 0.5, 0.75, 1.0);
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);

    let color = service.draw_commands()[0].color;

    assert_eq!(color, ColorRgba::new(0.25, 0.5, 0.75, 1.0));
}

#[test]
fn draw_command_carries_particle_light_module() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.light = Some(ParticleLight2d {
        radius: 24.0,
        intensity: 0.35,
        mode: ParticleLightMode2d::Source,
        glow: false,
    });
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();

    assert_eq!(
        draw[0].light,
        Some(ParticleLight2d {
            radius: 24.0,
            intensity: 0.35,
            mode: ParticleLightMode2d::Source,
            glow: false,
        })
    );
}

#[test]
fn draw_command_carries_particle_material() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.material = ParticleMaterial2d {
        receives_light: true,
        light_response: 0.5,
    };
    service.queue_emitter(command);
    service.tick(&[test_input()], 0.1);

    let draw = service.draw_commands();

    assert_eq!(
        draw[0].material,
        ParticleMaterial2d {
            receives_light: true,
            light_response: 0.5,
        }
    );
}

#[test]
fn set_color_clears_color_ramp_override() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 1.0)));
    service.queue_emitter(command);

    assert!(service.set_color(TEST_EMITTER, ColorRgba::new(0.0, 1.0, 0.0, 1.0)));

    let emitter = service.emitter(TEST_EMITTER).expect("emitter should exist");
    assert_eq!(emitter.emitter.color, ColorRgba::new(0.0, 1.0, 0.0, 1.0));
    assert!(emitter.emitter.color_ramp.is_none());
}

#[test]
fn set_color_ramp_updates_draw_color() {
    let service = Particle2dSceneService::default();
    service.queue_emitter(test_emitter(true));

    assert!(service.set_color_ramp(
        TEST_EMITTER,
        ColorRamp::constant(ColorRgba::new(0.0, 1.0, 0.0, 1.0))
    ));
    service.tick(&[test_input()], 0.1);

    assert_eq!(
        service.draw_commands()[0].color,
        ColorRgba::new(0.0, 1.0, 0.0, 1.0)
    );
}

#[test]
fn clear_color_ramp_restores_legacy_color() {
    let service = Particle2dSceneService::default();
    let mut command = test_emitter(true);
    command.emitter.color = ColorRgba::new(0.2, 0.3, 0.4, 1.0);
    command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 1.0)));
    service.queue_emitter(command);

    assert!(service.clear_color_ramp(TEST_EMITTER));
    service.tick(&[test_input()], 0.1);

    assert_eq!(
        service.draw_commands()[0].color,
        ColorRgba::new(0.2, 0.3, 0.4, 1.0)
    );
}
