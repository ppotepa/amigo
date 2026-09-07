use crate::{feature::FeatureClass, style::ComicInk};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeVertex {
    pub position: Vec2,
    pub width: f32,
    pub id: u32,
    pub depth: f32,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TessellatedStroke {
    pub vertices: Vec<StrokeVertex>,
    pub indices: Vec<u32>,
    pub id: u32,
    pub class: FeatureClass,
}

pub fn tessellate_segment(
    id: u32,
    class: FeatureClass,
    segment: (Vec2, Vec2),
    style: ComicInk,
    seed: u64,
) -> TessellatedStroke {
    tessellate_segment_with_depth(id, class, segment, (0.5, 0.5), style, seed)
}

pub fn tessellate_segment_with_depth(
    id: u32,
    class: FeatureClass,
    segment: (Vec2, Vec2),
    depths: (f32, f32),
    style: ComicInk,
    seed: u64,
) -> TessellatedStroke {
    let (a, b) = segment;
    let d = b - a;
    let len = d.length().max(1e-5);
    let n = Vec2::new(-d.y, d.x) / len;
    let wobble = ((seed
        .wrapping_add(id as u64)
        .wrapping_mul(6364136223846793005)
        >> 32) as f32
        / u32::MAX as f32
        - 0.5)
        * style.wobble;
    let a = a + n * wobble;
    let b = b - n * wobble;
    let width = style.width(class);
    let cap = width * 0.5;
    let start = a - d.normalize_or_zero() * cap;
    let end = b + d.normalize_or_zero() * cap;
    let vertices = vec![
        StrokeVertex {
            position: start + n * width * 0.5,
            width,
            id,
            depth: depths.0,
        },
        StrokeVertex {
            position: start - n * width * 0.5,
            width,
            id,
            depth: depths.0,
        },
        StrokeVertex {
            position: end + n * width * 0.5,
            width: width * (1.0 - style.taper).max(0.0),
            id,
            depth: depths.1,
        },
        StrokeVertex {
            position: end - n * width * 0.5,
            width: width * (1.0 - style.taper).max(0.0),
            id,
            depth: depths.1,
        },
    ];
    TessellatedStroke {
        vertices,
        indices: vec![0, 1, 2, 2, 1, 3],
        id,
        class,
    }
}
