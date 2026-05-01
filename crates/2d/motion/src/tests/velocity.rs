use crate::{Velocity2d, step_velocity_2d};
use amigo_math::Vec2;

#[test]
fn step_velocity_applies_linear_displacement() {
    let next = step_velocity_2d(
        Vec2::new(2.0, -3.0),
        &Velocity2d::new(Vec2::new(4.0, 8.0)),
        0.5,
    );

    assert_eq!(next, Vec2::new(4.0, 1.0));
}
