use glam::{Vec2, Vec3};

pub const EPS: f32 = 1.0e-6;

pub fn deg(v: f32) -> f32 {
    v.to_radians()
}

pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn hash01(v: f32) -> f32 {
    ((v.sin() * 43_758.547).fract()).abs()
}

pub fn noise(seed: f32, i: f32) -> f32 {
    hash01(seed * 12.9898 + i * 78.233) * 2.0 - 1.0
}

pub fn rot2(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

pub fn norm2(v: Vec2) -> Vec2 {
    let len = v.length();
    if len <= EPS { Vec2::X } else { v / len }
}

pub fn tri_area2(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() * 0.5
}

pub fn bary2(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> Option<Vec3> {
    let den = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if den.abs() <= EPS {
        return None;
    }
    let u = ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / den;
    let v = ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / den;
    Some(Vec3::new(u, v, 1.0 - u - v))
}

pub fn bary_inside(bary: Option<Vec3>, eps: f32) -> bool {
    bary.is_some_and(|b| b.x >= -eps && b.y >= -eps && b.z >= -eps)
}

pub fn parse_hex_rgb(hex: &str, fallback: [f32; 4]) -> [f32; 4] {
    let value = hex.trim().trim_start_matches('#');
    if value.len() != 6 {
        return fallback;
    }
    let Ok(rgb) = u32::from_str_radix(value, 16) else {
        return fallback;
    };
    [
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
        fallback[3],
    ]
}
