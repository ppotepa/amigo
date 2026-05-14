use amigo_math::{Transform3, Vec2, Vec3};
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileEmitter2d {
    pub pool: String,
    pub speed: f32,
    pub spawn_offset: Vec2,
    pub inherit_velocity_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileEmitter2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub emitter: ProjectileEmitter2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileLaunch2d {
    pub transform: Transform3,
    pub velocity: Vec2,
}

pub fn projectile_launch_2d(
    source_transform: Transform3,
    source_velocity: Vec2,
    emitter: &ProjectileEmitter2d,
) -> ProjectileLaunch2d {
    let rotation = source_transform.rotation_euler.z;
    let forward = Vec2::new(rotation.cos(), rotation.sin());
    let right = Vec2::new(-rotation.sin(), rotation.cos());
    let offset = Vec2::new(
        forward.x * emitter.spawn_offset.x + right.x * emitter.spawn_offset.y,
        forward.y * emitter.spawn_offset.x + right.y * emitter.spawn_offset.y,
    );
    let mut transform = source_transform;
    transform.translation = Vec3::new(
        source_transform.translation.x + offset.x,
        source_transform.translation.y + offset.y,
        source_transform.translation.z,
    );

    ProjectileLaunch2d {
        transform,
        velocity: Vec2::new(
            forward.x * emitter.speed + source_velocity.x * emitter.inherit_velocity_scale,
            forward.y * emitter.speed + source_velocity.y * emitter.inherit_velocity_scale,
        ),
    }
}
