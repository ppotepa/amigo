use crate::{ProjectileEmitter2d, projectile_launch_2d};
use amigo_math::{Transform3, Vec2, Vec3};

#[test]
fn projectile_launch_uses_transform_facing_offset_and_inherited_velocity() {
    let launch = projectile_launch_2d(
        Transform3 {
            translation: Vec3::new(10.0, 20.0, 0.0),
            rotation_euler: Vec3::new(0.0, 0.0, std::f32::consts::FRAC_PI_2),
            ..Transform3::default()
        },
        Vec2::new(4.0, -2.0),
        &ProjectileEmitter2d {
            pool: "projectiles".to_owned(),
            speed: 100.0,
            spawn_offset: Vec2::new(8.0, 2.0),
            inherit_velocity_scale: 0.5,
        },
    );

    assert!((launch.transform.translation.x - 8.0).abs() < 0.001);
    assert!((launch.transform.translation.y - 28.0).abs() < 0.001);
    assert!((launch.velocity.x - 2.0).abs() < 0.001);
    assert!((launch.velocity.y - 99.0).abs() < 0.001);
}
