use crate::{StrokeRole, feature::FeatureClass, gesture, style::ComicInk, tool::StrokeTool};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeVertex {
    pub position: Vec2,
    pub width: f32,
    pub id: u32,
    pub depth: f32,
    pub pressure: f32,
    pub coverage: f32,
    pub grain: f32,
    /// Normalized distance from the gesture centreline. The material shader uses
    /// it for analytic edge coverage; 1.0 is the geometric envelope.
    pub edge: f32,
    /// Width of the analytically softened edge, normalized to the stroke radius.
    /// This is material data; it is not geometric jitter.
    pub edge_softness: f32,
    /// Shared-paper parameters carried to the backend material pass.
    pub paper_tooth: f32,
    pub dryness: f32,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TessellatedStroke {
    pub vertices: Vec<StrokeVertex>,
    pub indices: Vec<u32>,
    pub id: u32,
    pub class: FeatureClass,
    pub role: StrokeRole,
    pub correction: bool,
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
    tessellate_polyline(
        id,
        class,
        &[(segment.0, depths.0), (segment.1, depths.1)],
        false,
        style,
        seed,
    )
}

/// Width and wobble live solely in pixel space. The strip shares vertices across
/// adjoining segments, so a single gesture does not deposit a dark disk at every
/// sample point. Round caps are emitted only at real open ends.
pub fn tessellate_polyline(
    id: u32,
    class: FeatureClass,
    points: &[(Vec2, f32)],
    closed: bool,
    style: ComicInk,
    seed: u64,
) -> TessellatedStroke {
    tessellate_polyline_variant(id, class, points, closed, style, seed, 0, false)
}

/// Creates the primary gesture and, when requested, one restrained correction
/// pass. The correction is intentionally a separate stroke so its coverage and
/// depth can be inspected and disabled independently by a backend or debug view.
pub fn tessellate_polyline_variants(
    id: u32,
    class: FeatureClass,
    points: &[(Vec2, f32)],
    closed: bool,
    style: ComicInk,
    seed: u64,
) -> Vec<TessellatedStroke> {
    let mut strokes = vec![tessellate_polyline_variant(
        id, class, points, closed, style, seed, 0, false,
    )];
    if !closed && style.gesture_overstroke > 0.0 && points.len() > 2 {
        let correction = tessellate_polyline_variant(
            id.wrapping_add(0x4000_0000),
            class,
            points,
            false,
            style,
            seed,
            1,
            true,
        );
        if !correction.vertices.is_empty() {
            strokes.push(correction);
        }
    }
    strokes
}

fn tessellate_polyline_variant(
    id: u32,
    class: FeatureClass,
    points: &[(Vec2, f32)],
    closed: bool,
    style: ComicInk,
    seed: u64,
    variant: u32,
    correction: bool,
) -> TessellatedStroke {
    let mut out = TessellatedStroke {
        id,
        class,
        correction,
        ..Default::default()
    };
    if points.len() < 2 || style.width(class) <= 0.0 {
        return out;
    }
    let mut expanded = Vec::with_capacity(points.len() * 2 - 1);
    for pair in points.windows(2) {
        expanded.push(pair[0]);
        expanded.push(((pair[0].0 + pair[1].0) * 0.5, (pair[0].1 + pair[1].1) * 0.5));
    }
    expanded.push(*points.last().unwrap());
    let expanded = gesture::simplify(
        &expanded,
        style.gesture_simplification.clamp(0.0, 1.0) * 2.5,
    );
    let total: f32 = expanded.windows(2).map(|p| p[0].0.distance(p[1].0)).sum();
    if total < 1e-5 {
        return out;
    }
    let mut distance = 0.0;
    let mut shaped = Vec::new();
    for (i, &(point, depth)) in expanded.iter().enumerate() {
        if i > 0 {
            distance += point.distance(expanded[i - 1].0);
        }
        let t = distance / total;
        let sample = gesture::sample(
            seed,
            id,
            t,
            style.gesture_confidence,
            style.gesture_correction,
            style.wobble,
            variant,
        );
        let previous = expanded[i.saturating_sub(1)].0;
        let next = expanded[(i + 1).min(expanded.len() - 1)].0;
        let tangent = next - previous;
        let direction = tangent.normalize_or_zero();
        let normal = Vec2::new(-direction.y, direction.x);
        let tangent_angle = direction.y.atan2(direction.x) - style.nib_angle.to_radians();
        let pressure = (sample.pressure * style.tool_pressure.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let tool = style
            .tool
            .response(pressure, tangent_angle, style.tool_hardness);
        // The tool response owns directional width. `nib_aspect` expands the
        // contact ellipse once here rather than applying another angle heuristic
        // later in the tessellator.
        let aspect = if matches!(style.tool, StrokeTool::Nib) {
            1.0 + style.nib_aspect.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let width = style.width(class)
            * tool.width_scale
            * tool.pressure_width
            * aspect
            * if closed {
                1.0
            } else {
                1.0 - style.taper.clamp(0.0, 1.0) * (2.0 * t - 1.0).abs()
            };
        let hash = seed
            .wrapping_add(id as u64)
            .wrapping_add(i as u64)
            .wrapping_mul(6364136223846793005);
        // Keep original chain vertices fixed. Midpoint jitter cannot open joins.
        let jitter = if i % 2 == 1 {
            ((hash >> 32) as f32 / u32::MAX as f32 - 0.5) * style.wobble
        } else {
            0.0
        };
        let correction_offset = if correction {
            sample.correction * style.wobble * 0.42
        } else {
            0.0
        };
        let position = point + normal * (jitter + sample.offset + correction_offset);
        let grain_strength =
            style.paper_grain.clamp(0.0, 1.0) * (0.45 + style.ink_dryness.clamp(0.0, 1.0) * 0.55);
        let grain = (sample.grain - 0.5) * tool.grain * grain_strength;
        let dryness = style.ink_dryness.clamp(0.0, 1.0);
        let mut coverage = tool.pressure_alpha
            * (1.0 - grain.abs() * (0.35 + dryness * 0.25))
            * (1.0 - dryness * 0.08);
        // The hand supplies a stable deposit field; the tool decides how much
        // it shows. A fineliner deliberately remains the uniform compatibility
        // baseline, while soft pencil can leave genuine small gaps.
        let breakup = match style.tool {
            StrokeTool::Pencil => {
                0.16 + (1.0 - style.tool_hardness.clamp(0.0, 1.0)) * 0.46
                    + dryness * 0.20
                    + style.paper_tooth.clamp(0.0, 1.0) * 0.18
            }
            StrokeTool::Brush => 0.10 + (1.0 - style.tool_hardness.clamp(0.0, 1.0)) * 0.26,
            StrokeTool::Nib => 0.04 + dryness * 0.08,
            StrokeTool::Fineliner => 0.0,
        }
        .clamp(0.0, 0.92);
        coverage *= 1.0 - breakup * (1.0 - sample.deposit);
        if matches!(style.tool, StrokeTool::Pencil) {
            coverage *= 0.88 + pressure * 0.12;
        }
        if correction {
            coverage *= style.gesture_overstroke.clamp(0.0, 1.0) * 0.45;
        }
        shaped.push((
            position,
            depth,
            width,
            pressure,
            coverage.clamp(if breakup > 0.0 { 0.003 } else { 0.02 }, 1.0),
            sample.grain,
            tool.edge_softness,
            style.paper_tooth.clamp(0.0, 1.0),
            dryness,
        ));
    }
    let sample_count = shaped.len();
    for index in 0..sample_count {
        let previous = shaped[index.saturating_sub(1)].0;
        let next = shaped[(index + 1).min(sample_count - 1)].0;
        let direction = (next - previous).normalize_or_zero();
        if direction.length_squared() <= 1e-8 {
            continue;
        }
        let normal = Vec2::new(-direction.y, direction.x);
        let (point, depth, width, pressure, coverage, grain, edge_softness, paper_tooth, dryness) =
            shaped[index];
        for side in [-1.0, 1.0] {
            out.vertices.push(StrokeVertex {
                position: point + normal * width * 0.5 * side,
                width,
                id,
                depth,
                pressure,
                coverage,
                grain,
                edge: side,
                edge_softness,
                paper_tooth,
                dryness,
            });
        }
    }
    // Every shaped sample produced exactly two vertices above. Invalid zero
    // direction input was rejected before this point by the non-zero total test.
    for index in 0..sample_count - 1 {
        let base = (index * 2) as u32;
        out.indices
            .extend([base, base + 1, base + 2, base + 2, base + 1, base + 3]);
    }
    if !closed {
        for &sample_index in &[0, sample_count - 1] {
            let (
                center,
                depth,
                width,
                pressure,
                coverage,
                grain,
                edge_softness,
                paper_tooth,
                dryness,
            ) = shaped[sample_index];
            let base = out.vertices.len() as u32;
            out.vertices.push(StrokeVertex {
                position: center,
                width,
                id,
                depth,
                pressure,
                coverage,
                grain,
                edge: 0.0,
                edge_softness,
                paper_tooth,
                dryness,
            });
            for i in 0..=12 {
                let a = i as f32 * std::f32::consts::TAU / 12.0;
                out.vertices.push(StrokeVertex {
                    position: center + Vec2::new(a.cos(), a.sin()) * width * 0.5,
                    width,
                    id,
                    depth,
                    pressure,
                    coverage,
                    grain,
                    edge: 1.0,
                    edge_softness,
                    paper_tooth,
                    dryness,
                });
            }
            for i in 0..12 {
                out.indices.extend([base, base + i + 1, base + i + 2]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taper_changes_geometry_and_round_caps_have_valid_indices() {
        let style = ComicInk {
            outline_width: 10.0,
            taper: 0.8,
            wobble: 0.0,
            ..Default::default()
        };
        let s = tessellate_segment(
            7,
            FeatureClass::Silhouette,
            (Vec2::ZERO, Vec2::new(100.0, 0.0)),
            style,
            1,
        );
        assert!((s.vertices[0].position.y.abs() - 1.0).abs() < 1e-4);
        assert!(s.vertices.iter().any(|v| v.position.y.abs() > 4.9));
        assert!(s.vertices.iter().any(|v| v.position.x < 0.0));
        assert!(s.indices.iter().all(|i| (*i as usize) < s.vertices.len()));
    }
    #[test]
    fn wobble_is_seeded_and_keeps_endpoints() {
        let style = ComicInk {
            wobble: 5.0,
            ..Default::default()
        };
        let make = |seed| {
            tessellate_segment(
                7,
                FeatureClass::Silhouette,
                (Vec2::ZERO, Vec2::splat(100.0)),
                style,
                seed,
            )
        };
        assert_eq!(make(1), make(1));
        assert_ne!(make(1), make(2));
    }

    #[test]
    fn pencil_response_changes_pressure_coverage_and_correction_is_restrained() {
        let style = ComicInk {
            tool: StrokeTool::Pencil,
            gesture_confidence: 0.35,
            gesture_correction: 0.8,
            gesture_overstroke: 0.35,
            wobble: 2.0,
            paper_grain: 0.8,
            ..Default::default()
        };
        let strokes = tessellate_polyline_variants(
            7,
            FeatureClass::Silhouette,
            &[
                (Vec2::ZERO, 0.2),
                (Vec2::new(40.0, 10.0), 0.3),
                (Vec2::new(100.0, 0.0), 0.4),
            ],
            false,
            style,
            42,
        );
        assert_eq!(strokes.len(), 2);
        assert!(
            strokes[0]
                .vertices
                .iter()
                .map(|v| v.pressure.to_bits())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        );
        assert!(strokes[1].correction);
        assert!(strokes[1].vertices.iter().all(|v| v.coverage < 0.3));
    }

    #[test]
    fn open_strip_uses_shared_vertices_and_caps_only_the_real_ends() {
        let stroke = tessellate_polyline(
            11,
            FeatureClass::Silhouette,
            &[
                (Vec2::new(0.0, 0.0), 0.2),
                (Vec2::new(30.0, 8.0), 0.3),
                (Vec2::new(60.0, 0.0), 0.4),
            ],
            false,
            ComicInk {
                paper_tooth: 0.7,
                ..Default::default()
            },
            99,
        );
        // Three authored points expand to five shaped samples. The strip is two
        // shared vertices per sample plus two 12-sided end caps, not one disk
        // for every sample.
        assert_eq!(stroke.vertices.len(), 5 * 2 + 2 * 14);
        assert_eq!(stroke.indices.len(), 4 * 6 + 2 * 12 * 3);
        assert!(
            stroke
                .vertices
                .iter()
                .all(|vertex| vertex.paper_tooth == 0.7)
        );
        assert!(
            stroke
                .indices
                .iter()
                .all(|index| (*index as usize) < stroke.vertices.len())
        );
    }

    #[test]
    fn soft_pencil_exposes_deposit_breaks_but_fineliner_remains_uniform() {
        let points = &[
            (Vec2::new(0.0, 0.0), 0.2),
            (Vec2::new(60.0, 4.0), 0.2),
            (Vec2::new(120.0, -2.0), 0.2),
        ];
        let pencil = tessellate_polyline(
            8,
            FeatureClass::Crease,
            points,
            false,
            ComicInk {
                tool: StrokeTool::Pencil,
                tool_hardness: 0.05,
                paper_tooth: 1.0,
                ink_dryness: 0.7,
                ..Default::default()
            },
            123,
        );
        let fineliner = tessellate_polyline(
            8,
            FeatureClass::Crease,
            points,
            false,
            ComicInk::default(),
            123,
        );
        let pencil_range = pencil
            .vertices
            .iter()
            .map(|vertex| vertex.coverage)
            .fold((1.0f32, 0.0f32), |(min, max), value| {
                (min.min(value), max.max(value))
            });
        assert!(pencil_range.1 - pencil_range.0 > 0.08);
        assert!(
            fineliner
                .vertices
                .windows(2)
                .all(|pair| (pair[0].coverage - pair[1].coverage).abs() < 1e-6)
        );
    }
}
