use glam::{Vec2, Vec3};

pub type Point2 = Vec2;
pub type Point3 = Vec3;

pub fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a + (b - a) * t
}
