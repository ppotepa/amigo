use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity2d {
    pub linear: Vec2,
}

impl Default for Velocity2d {
    fn default() -> Self {
        Self { linear: Vec2::ZERO }
    }
}

impl Velocity2d {
    pub const fn new(linear: Vec2) -> Self {
        Self { linear }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Velocity2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub velocity: Velocity2d,
}

pub fn step_velocity_2d(translation: Vec2, velocity: &Velocity2d, delta_seconds: f32) -> Vec2 {
    Vec2::new(
        translation.x + velocity.linear.x * delta_seconds,
        translation.y + velocity.linear.y * delta_seconds,
    )
}
