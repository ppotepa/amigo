use crate::{NprMediumDefinition, feature::FeatureClass, style::ComicInk};
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
    /// Per-stroke medium overrides the packet default, enabling multiple
    /// brushes (for example 2B contours and HB hatching) in one profile.
    pub medium: Option<NprMediumDefinition>,
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
        medium: None,
    }
}

/// Tessellates a pressure-ready centreline.  Planning/gesture own the path;
/// this function owns only the neutral ribbon topology consumed by backends.
pub fn tessellate_polyline_with_depth(
    id: u32,
    class: FeatureClass,
    points: &[(Vec2, f32)],
    style: ComicInk,
) -> TessellatedStroke {
    if points.len() < 2 {
        return TessellatedStroke { id, class, ..Default::default() };
    }
    let mut vertices = Vec::with_capacity(points.len() * 2);
    for (index, &(position, depth)) in points.iter().enumerate() {
        let previous = points[index.saturating_sub(1)].0;
        let next = points[(index + 1).min(points.len() - 1)].0;
        let tangent = (next - previous).normalize_or_zero();
        let normal = Vec2::new(-tangent.y, tangent.x);
        let envelope = if index == 0 || index + 1 == points.len() {
            1.0 - style.taper
        } else {
            1.0
        };
        let t = index as f32 / (points.len() - 1) as f32;
        // One continuous pressure gesture, with an ID-stable low-frequency
        // correction. Width changes belong to the planned physical mark,
        // while the graphite shader separately controls pigment deposition.
        let base_pressure = 0.78 + 0.22 * (std::f32::consts::PI * t).sin();
        let phase = (id.wrapping_mul(2_654_435_761) % 10_007) as f32 / 10_007.0;
        let hand_correction = 0.92 + 0.08 * (t * 8.3 + phase * 6.2831853).sin();
        let width = style.width(class) * envelope.max(0.1) * base_pressure * hand_correction;
        vertices.push(StrokeVertex { position: position + normal * width * 0.5, width, id, depth });
        vertices.push(StrokeVertex { position: position - normal * width * 0.5, width, id, depth });
    }
    let mut indices = Vec::with_capacity((points.len() - 1) * 6);
    for segment in 0..points.len() - 1 {
        let start = (segment * 2) as u32;
        indices.extend_from_slice(&[start, start + 1, start + 2, start + 2, start + 1, start + 3]);
    }
    TessellatedStroke { vertices, indices, id, class, medium: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polyline_generates_two_triangles_per_segment() {
        let stroke = tessellate_polyline_with_depth(
            1,
            FeatureClass::Hatching,
            &[(Vec2::ZERO, 0.2), (Vec2::X, 0.3), (Vec2::ONE, 0.4)],
            ComicInk::default(),
        );
        assert_eq!(stroke.vertices.len(), 6);
        assert_eq!(stroke.indices.len(), 12);
    }

    #[test]
    fn graphite_ready_polyline_has_a_continuous_pressure_width_profile() {
        let stroke = tessellate_polyline_with_depth(
            19,
            FeatureClass::Silhouette,
            &[
                (Vec2::ZERO, 0.2),
                (Vec2::X, 0.2),
                (Vec2::new(2.0, 0.2), 0.2),
                (Vec2::new(3.0, 0.2), 0.2),
            ],
            ComicInk::default(),
        );
        assert!(stroke.vertices[2].width > stroke.vertices[0].width);
        assert_ne!(stroke.vertices[2].width, stroke.vertices[4].width);
    }
}
