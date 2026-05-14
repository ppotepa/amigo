use super::common::test_bounds;
use crate::{BoundsBehavior2d, BoundsContact2d, BoundsOutcome2d, apply_bounds_2d};
use amigo_math::Vec2;

#[test]
fn bounds_bounce_reflects_velocity_and_reports_contact() {
    let result = apply_bounds_2d(
        Vec2::new(12.0, 0.0),
        Vec2::new(8.0, 1.0),
        &test_bounds(BoundsBehavior2d::Bounce { restitution: 0.5 }),
    );

    assert_eq!(result.translation, Vec2::new(10.0, 0.0));
    assert_eq!(result.velocity, Vec2::new(-4.0, 1.0));
    assert_eq!(
        result.outcome,
        BoundsOutcome2d::Bounced {
            contact: BoundsContact2d {
                max_x: true,
                ..Default::default()
            }
        }
    );
}

#[test]
fn bounds_wrap_moves_to_opposite_edge() {
    let result = apply_bounds_2d(
        Vec2::new(-12.0, 6.0),
        Vec2::new(-2.0, 3.0),
        &test_bounds(BoundsBehavior2d::Wrap),
    );

    assert_eq!(result.translation, Vec2::new(10.0, -5.0));
    assert_eq!(result.velocity, Vec2::new(-2.0, 3.0));
    assert_eq!(
        result.outcome,
        BoundsOutcome2d::Wrapped {
            contact: BoundsContact2d {
                min_x: true,
                max_y: true,
                ..Default::default()
            }
        }
    );
}

#[test]
fn bounds_clamp_limits_translation_without_lifecycle_outcome() {
    let result = apply_bounds_2d(
        Vec2::new(2.0, -8.0),
        Vec2::new(1.0, -9.0),
        &test_bounds(BoundsBehavior2d::Clamp),
    );

    assert_eq!(result.translation, Vec2::new(2.0, -5.0));
    assert_eq!(result.velocity, Vec2::new(1.0, -9.0));
    assert_eq!(
        result.outcome,
        BoundsOutcome2d::Clamped {
            contact: BoundsContact2d {
                min_y: true,
                ..Default::default()
            }
        }
    );
}

#[test]
fn bounds_hide_reports_generic_hidden_outcome() {
    let result = apply_bounds_2d(
        Vec2::new(11.0, 0.0),
        Vec2::new(1.0, 0.0),
        &test_bounds(BoundsBehavior2d::Hide),
    );

    assert_eq!(
        result.outcome,
        BoundsOutcome2d::Hidden {
            contact: BoundsContact2d {
                max_x: true,
                ..Default::default()
            }
        }
    );
}

#[test]
fn bounds_despawn_reports_generic_despawn_outcome() {
    let result = apply_bounds_2d(
        Vec2::new(0.0, -6.0),
        Vec2::new(0.0, -1.0),
        &test_bounds(BoundsBehavior2d::Despawn),
    );

    assert_eq!(
        result.outcome,
        BoundsOutcome2d::Despawned {
            contact: BoundsContact2d {
                min_y: true,
                ..Default::default()
            }
        }
    );
}
