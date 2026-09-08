use crate::{
    camera::PerspectiveCamera,
    feature::FeatureClass,
    geometry::NprGeometry,
    plan_graphite_tone, select_ranked, smooth_perspective_contours,
    style::{ComicInk, NprToneMode},
    suggestive_perspective_contours,
    tessellation::{tessellate_polyline, tessellate_polyline_variants, TessellatedStroke},
    topology::{build_topology, face_normal},
    trace_parallel_surface_lines, trace_surface_streamline, GraphiteTonePlan, NprConstructionMark,
    NprDebugView, NprSurfaceAnchorError, NprSurfaceMode, RankedCandidate, StrokeRole,
    SurfaceDirectionField,
};
use glam::{Vec2, Vec4};
use std::collections::BTreeMap;

const MAX_HATCHING_LINES_PER_PACKET: usize = 32_000;
/// CPU-side ceiling kept below the backend's global per-view upload ceiling.
/// Feature strokes are considered first, then tonal strokes in deterministic
/// planner order. This makes quality degradation explicit and repeatable.
const MAX_STROKE_DATA_BYTES_PER_PACKET: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
struct HatchSegment {
    points: [(Vec2, f32); 2],
    group: u32,
    lane: u8,
    line: i64,
    coverage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct HatchKey {
    group: u32,
    lane: u8,
    line: i64,
}

#[derive(Debug, Default)]
struct HatchingOutput {
    strokes: Vec<TessellatedStroke>,
    candidates: usize,
    rejected: usize,
    confidence_rejected: usize,
    corrections: usize,
}

#[derive(Debug, Default)]
struct SurfaceHatchLaneOutput {
    candidates: Vec<SurfaceHatchCandidate>,
    confidence_rejected: usize,
}

/// A projected path that has passed surface analysis but has not yet allocated
/// tessellation vertices. The shared budget ranks these first.
#[derive(Debug)]
struct SurfaceHatchCandidate {
    id: u32,
    points: Vec<(Vec2, f32)>,
    coverage: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprFillTriangle {
    pub positions: [Vec2; 3],
    pub color: Vec4,
    pub depths: [f32; 3],
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NprRenderStats {
    pub geometry: usize,
    /// Triangles of the authored/source surface before an optional smooth proxy.
    pub surface_source_triangles: usize,
    /// Triangles used by the drawing packet after the selected surface policy.
    pub surface_proxy_triangles: usize,
    pub topology_edges: usize,
    pub feature_segments: usize,
    /// Feature segments considered after surface policy but before chain ranking.
    pub feature_candidates: usize,
    /// Candidate segments omitted because their whole crease chain was too short.
    pub feature_rejected: usize,
    /// View-dependent contour spans generated from a smooth normal field,
    /// rather than from topology edges.
    pub smooth_contour_spans: usize,
    /// Optional interior form spans extracted from radial-curvature zero
    /// crossings, after confidence gating.
    pub suggestive_contour_spans: usize,
    pub silhouettes: usize,
    pub creases: usize,
    pub strokes: usize,
    pub stroke_vertices: usize,
    pub stroke_indices: usize,
    /// Number of tone strokes emitted after the per-packet hatch limit.
    pub hatching_strokes: usize,
    /// Restrained second tonal passes. They refine accepted paths rather than
    /// creating further planned hatching candidates.
    pub hatching_correction_strokes: usize,
    /// Sum of requested graphite material over eligible faces. This is a
    /// planning metric, not a physical pigment measurement.
    pub graphite_mass: f32,
    /// Paths considered by the hatching planner before its shared budget.
    pub hatching_candidates: usize,
    /// Candidates intentionally removed by the declared quality budget.
    pub hatching_rejected: usize,
    /// Tonal candidates rejected before tessellation because their local form
    /// direction was ambiguous or changed too abruptly across the path.
    pub hatching_confidence_rejected: usize,
    /// Anchored construction marks accepted after packet construction.
    pub construction_marks: usize,
    /// Construction marks omitted because they were clipped, underspecified or
    /// did not fit the remaining declared CPU packet budget.
    pub construction_rejected: usize,
    /// Stateful tonal detail tier chosen by the scene/domain owner.
    pub hatching_lod_tier: u8,
    /// The packet reached its declared hatch-line budget. This is a quality
    /// signal, not an error and must remain visible to diagnostics.
    pub hatching_budget_exhausted: bool,
    /// Strokes omitted by the packet payload budget before any WGPU upload.
    pub stroke_budget_rejected: usize,
    /// The packet reached its declared CPU stroke payload budget.
    pub stroke_budget_exhausted: bool,
    /// Strokes that retained an identity from the previous extracted frame.
    pub temporal_retained_strokes: usize,
    /// Newly appearing identities currently being eased into the drawing.
    pub temporal_entering_strokes: usize,
    /// Domain-selected seeded gesture variant for this packet. It changes only
    /// through an explicit motion policy, never as a side effect of FPS.
    pub gesture_variant_epoch: u32,
    /// CPU packet payload before backend vertex expansion or upload.
    pub stroke_data_bytes: usize,
    pub viewport: [u32; 2],
}
#[derive(Debug, Clone, PartialEq)]
pub struct NprRenderPacket {
    /// Geometry that participates in depth regardless of whether a profile
    /// chooses to paint a visible fill. This preserves hidden-line removal for
    /// pencil-on-paper profiles.
    pub occluders: Vec<NprFillTriangle>,
    pub fills: Vec<NprFillTriangle>,
    pub strokes: Vec<TessellatedStroke>,
    pub background: Vec4,
    pub debug_view: NprDebugView,
    pub ink: Vec4,
    pub stats: NprRenderStats,
}

impl NprRenderPacket {
    /// Explicit selection annotation, independent of ink styling and feature extraction.
    /// Two windings let culling backends draw one face without a special pipeline.
    pub fn mark_selection(
        &mut self,
        geometry: &NprGeometry,
        camera: PerspectiveCamera,
        color: Vec4,
    ) {
        let viewport = Vec2::new(self.stats.viewport[0] as f32, self.stats.viewport[1] as f32);
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for point in geometry
            .vertices
            .iter()
            .filter_map(|v| camera.project(v.position, viewport))
        {
            min = min.min(point.screen);
            max = max.max(point.screen);
        }
        let center = Vec2::new((min.x + max.x) * 0.5, min.y - 10.0);
        if !center.is_finite()
            || center.x < 7.0
            || center.x > viewport.x - 7.0
            || center.y < 7.0
            || center.y > viewport.y - 7.0
        {
            return;
        }
        let positions = [
            center + Vec2::new(-6.0, -4.0),
            center + Vec2::new(6.0, -4.0),
            center + Vec2::new(0.0, 4.0),
        ];
        for positions in [positions, [positions[2], positions[1], positions[0]]] {
            self.fills.push(NprFillTriangle {
                positions,
                color,
                depths: [0.0; 3],
            });
        }
    }
    pub fn stroke_color(&self, stroke: &TessellatedStroke) -> [f32; 4] {
        match self.debug_view {
            NprDebugView::Final => self.ink.to_array(),
            NprDebugView::FeatureClasses => match stroke.role {
                StrokeRole::Tone => [0.82, 0.48, 0.10, 1.0],
                StrokeRole::Construction => [0.62, 0.20, 0.78, 1.0],
                StrokeRole::Feature => match stroke.class {
                    FeatureClass::Boundary => [0.9, 0.15, 0.1, 1.0],
                    FeatureClass::Silhouette => [0.1, 0.75, 0.2, 1.0],
                    FeatureClass::Crease => [0.15, 0.3, 0.95, 1.0],
                },
            },
            NprDebugView::StrokeIds => {
                let hue = (stroke.id.wrapping_mul(97) % 255) as f32 / 255.0;
                [hue, 1.0 - hue, 0.8, 1.0]
            }
        }
    }
}

pub fn build_packet(
    geometry: &NprGeometry,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    let topology = build_topology(geometry);
    build_packet_with_topology(
        geometry, &topology, camera, viewport, style, seed, debug_view,
    )
}

pub fn build_packet_with_topology(
    geometry: &NprGeometry,
    topology: &[crate::TopologyEdge],
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    build_packet_with_identity(
        geometry, topology, camera, viewport, style, seed, debug_view, None,
    )
}

fn build_packet_with_identity(
    geometry: &NprGeometry,
    topology: &[crate::TopologyEdge],
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
    surface: Option<&crate::NprPreparedSurface>,
) -> NprRenderPacket {
    let vp = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let edge_features = crate::feature::classify_perspective_features(
        geometry,
        &topology,
        camera.position,
        style.crease_angle,
    );
    let smooth_contours = (style.surface_mode == NprSurfaceMode::Smooth)
        .then(|| smooth_perspective_contours(geometry, camera.position, style.smooth_crease_angle))
        .unwrap_or_default();
    let suggestive_contours = (style.surface_mode == NprSurfaceMode::Smooth
        && style.suggestive_contours)
        .then(|| {
            suggestive_perspective_contours(
                geometry,
                camera.position,
                style.smooth_crease_angle,
                style.suggestive_contour_confidence,
            )
        })
        .unwrap_or_default();
    // A smooth contour replaces the polygon-edge approximation only. Open
    // boundaries and authored sharp creases retain their explicit meaning.
    let features = if style.surface_mode == NprSurfaceMode::Smooth {
        edge_features
            .iter()
            .copied()
            .filter(|feature| match feature.class {
                FeatureClass::Silhouette => false,
                FeatureClass::Boundary => true,
                // `crease_angle` is the expressive threshold used by a
                // polygonal drawing. A smooth surface must not reinterpret
                // every coarse sampling edge as a form break; retain only a
                // dihedral which the smooth-normal policy itself treats as
                // discontinuous.
                FeatureClass::Crease => {
                    let [first, second] = feature.edge.faces;
                    second != u32::MAX
                        && face_normal(geometry, first).dot(face_normal(geometry, second))
                            < style.smooth_crease_angle.cos()
                }
            })
            .collect::<Vec<_>>()
    } else {
        edge_features
    };
    let mut occluders = Vec::new();
    let mut fills = Vec::new();
    let mut hatch_segments = Vec::new();
    let mut surface_hatch_tones = vec![GraphiteTonePlan::PAPER; geometry.triangles.len()];
    let mut hatching_budget = MAX_HATCHING_LINES_PER_PACKET;
    let light_direction = style.light_direction.normalize_or_zero();
    let direction_field = SurfaceDirectionField::build(geometry, topology, 0.85);
    for (face_index, tri) in geometry.triangles.iter().enumerate() {
        for clipped in camera.clip_triangle(tri.map(|i| geometry.vertices[i as usize].position)) {
            if let (Some(a), Some(b), Some(c)) = (
                camera.project(clipped[0], vp),
                camera.project(clipped[1], vp),
                camera.project(clipped[2], vp),
            ) {
                let shade = face_normal(geometry, face_index as u32).dot(light_direction);
                let color = if shade < -0.1 {
                    style.shadow
                } else if shade < 0.45 {
                    style.mid
                } else {
                    style.light
                };
                let positions = [a.screen, b.screen, c.screen];
                let depths = [a.depth, b.depth, c.depth].map(|d| camera.normalized_depth(d));
                let occluder = NprFillTriangle {
                    positions,
                    depths,
                    // The depth pass does not read colour. Retaining it makes
                    // the primitive self-contained for neutral consumers.
                    color,
                };
                occluders.push(occluder.clone());
                if style.tone_mode == NprToneMode::ThreeBand {
                    fills.push(occluder);
                }
                if style.tone_mode == NprToneMode::Hatching {
                    // Low-confidence local tangents (e.g. a cylinder cap for
                    // a longitudinal field) keep a light trace rather than a
                    // forced, arbitrary tonal direction.
                    let confidence = 0.25 + 0.75 * direction_field.face_confidence(face_index);
                    let mut tone = plan_graphite_tone(shade, style);
                    tone.mass *= confidence;
                    tone.primary_density *= confidence;
                    tone.cross_density *= confidence;
                    tone.primary_coverage *= confidence;
                    tone.cross_coverage *= confidence;
                    if tone.mass > surface_hatch_tones[face_index].mass {
                        surface_hatch_tones[face_index] = tone;
                    }
                } else {
                    collect_hatching_segments(
                        &mut hatch_segments,
                        positions,
                        depths,
                        shade,
                        style,
                        viewport,
                        &mut hatching_budget,
                        face_index as u32,
                        None,
                    );
                }
            }
        }
    }
    let feature_candidates = features.len();
    let (feature_chains, feature_rejected) = rank_feature_chains(
        crate::stroke::chain_features(&features),
        geometry,
        camera,
        vp,
        style.min_crease_length_pixels,
    );
    let mut strokes = Vec::new();
    for chain in feature_chains {
        let mut points = Vec::new();
        let flush = |points: &mut Vec<(Vec2, f32)>, strokes: &mut Vec<TessellatedStroke>| {
            if points.len() > 1 {
                let closed = points.first() == points.last();
                strokes.extend(tessellate_polyline_variants(
                    chain.id,
                    chain.class,
                    points,
                    closed,
                    style,
                    seed,
                ));
            }
            points.clear();
        };
        for edge in chain.vertices.windows(2) {
            if let Some((a, b)) = camera.project_segment(
                geometry.vertices[edge[0] as usize].position,
                geometry.vertices[edge[1] as usize].position,
                vp,
            ) {
                let a = (a.screen, camera.normalized_depth(a.depth));
                let b = (b.screen, camera.normalized_depth(b.depth));
                if points.last().is_some_and(|last| *last != a) {
                    flush(&mut points, &mut strokes);
                }
                if points.is_empty() {
                    points.push(a);
                }
                points.push(b);
            } else {
                flush(&mut points, &mut strokes);
            }
        }
        flush(&mut points, &mut strokes);
    }
    for contour in &smooth_contours {
        let points = contour
            .points
            .iter()
            .map(|point| {
                camera
                    .project(*point, vp)
                    .map(|projected| (projected.screen, camera.normalized_depth(projected.depth)))
            })
            .collect::<Option<Vec<_>>>();
        if let Some(points) = points.filter(|points| points.len() >= 2) {
            strokes.extend(tessellate_polyline_variants(
                contour.id,
                FeatureClass::Silhouette,
                &points,
                false,
                style,
                seed,
            ));
        }
    }
    for contour in &suggestive_contours {
        let points = contour
            .points
            .iter()
            .map(|point| {
                camera
                    .project(*point, vp)
                    .map(|projected| (projected.screen, camera.normalized_depth(projected.depth)))
            })
            .collect::<Option<Vec<_>>>();
        if let Some(points) = points.filter(|points| points.len() >= 2) {
            let suggestive_style = stroke_layer_style(style, style.suggestive_contour_width_scale);
            let mut contour_strokes = tessellate_polyline_variants(
                contour.id,
                FeatureClass::Crease,
                &points,
                false,
                suggestive_style,
                seed,
            );
            apply_stroke_opacity(&mut contour_strokes, style.suggestive_contour_opacity);
            strokes.extend(contour_strokes);
        }
    }
    let graphite_mass = if style.tone_mode == NprToneMode::Hatching {
        surface_hatch_tones.iter().map(|tone| tone.mass).sum()
    } else {
        0.0
    };
    let hatching = if style.tone_mode == NprToneMode::Hatching {
        emit_surface_hatching(
            geometry,
            topology,
            surface,
            camera,
            viewport,
            style,
            seed,
            &surface_hatch_tones,
            &mut hatching_budget,
        )
    } else {
        let mut strokes = emit_hatching_segments(
            hatch_segments,
            stroke_layer_style(style, style.form_line_width_scale),
            seed,
        );
        apply_stroke_opacity(&mut strokes, style.form_line_opacity);
        HatchingOutput {
            candidates: strokes.len(),
            strokes,
            rejected: 0,
            confidence_rejected: 0,
            corrections: 0,
        }
    };
    let hatching_strokes = hatching.strokes.len();
    let hatching_budget_exhausted = hatching_budget == 0;
    let mut candidates = strokes;
    candidates.extend(hatching.strokes);
    let (strokes, retained_stroke_data_bytes, stroke_budget_rejected, stroke_budget_rejected_tone) =
        retain_strokes_under_budget(candidates, MAX_STROKE_DATA_BYTES_PER_PACKET);
    let silhouettes = if style.surface_mode == NprSurfaceMode::Smooth {
        smooth_contours.len()
    } else {
        features
            .iter()
            .filter(|f| f.class == FeatureClass::Silhouette)
            .count()
    };
    let creases = features
        .iter()
        .filter(|f| f.class == FeatureClass::Crease)
        .count();
    let stats = NprRenderStats {
        geometry: 1,
        surface_source_triangles: geometry.triangles.len(),
        surface_proxy_triangles: geometry.triangles.len(),
        topology_edges: topology.len(),
        feature_segments: features.len(),
        feature_candidates,
        feature_rejected,
        smooth_contour_spans: smooth_contours.len(),
        suggestive_contour_spans: suggestive_contours.len(),
        silhouettes,
        creases,
        strokes: strokes.len(),
        stroke_vertices: strokes.iter().map(|s| s.vertices.len()).sum(),
        stroke_indices: strokes.iter().map(|s| s.indices.len()).sum(),
        hatching_strokes: hatching_strokes.saturating_sub(stroke_budget_rejected_tone),
        hatching_correction_strokes: hatching.corrections,
        graphite_mass,
        hatching_candidates: hatching.candidates,
        hatching_rejected: hatching.rejected + stroke_budget_rejected_tone,
        hatching_confidence_rejected: hatching.confidence_rejected,
        construction_marks: 0,
        construction_rejected: 0,
        hatching_lod_tier: 0,
        hatching_budget_exhausted,
        stroke_budget_rejected,
        stroke_budget_exhausted: stroke_budget_rejected > 0,
        temporal_retained_strokes: 0,
        temporal_entering_strokes: 0,
        gesture_variant_epoch: 0,
        stroke_data_bytes: retained_stroke_data_bytes,
        viewport,
    };
    NprRenderPacket {
        occluders,
        fills,
        strokes,
        background: style.paper,
        debug_view,
        ink: style.ink,
        stats,
    }
}

fn stroke_data_bytes(stroke: &TessellatedStroke) -> usize {
    stroke
        .vertices
        .len()
        .saturating_mul(std::mem::size_of::<crate::StrokeVertex>())
        + stroke
            .indices
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
}

fn stroke_layer_style(mut style: ComicInk, width_scale: f32) -> ComicInk {
    style.crease_width *= width_scale.clamp(0.0, 2.0);
    style
}

fn apply_stroke_opacity(strokes: &mut [TessellatedStroke], opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    for stroke in strokes {
        for vertex in &mut stroke.vertices {
            vertex.coverage *= opacity;
        }
    }
}

fn rank_feature_chains(
    chains: Vec<crate::FeatureStroke>,
    geometry: &NprGeometry,
    camera: PerspectiveCamera,
    viewport: Vec2,
    min_crease_length_pixels: f32,
) -> (Vec<crate::FeatureStroke>, usize) {
    let mut rejected = 0usize;
    let selected = chains
        .into_iter()
        .filter(|chain| {
            if chain.class != FeatureClass::Crease {
                return true;
            }
            let length = chain
                .vertices
                .windows(2)
                .filter_map(|edge| {
                    camera
                        .project_segment(
                            geometry.vertices[edge[0] as usize].position,
                            geometry.vertices[edge[1] as usize].position,
                            viewport,
                        )
                        .map(|(a, b)| a.screen.distance(b.screen))
                })
                .sum::<f32>();
            let keep = length >= min_crease_length_pixels.max(0.0);
            if !keep {
                rejected += chain.vertices.len().saturating_sub(1);
            }
            keep
        })
        .collect();
    (selected, rejected)
}

fn retain_strokes_under_budget(
    candidates: Vec<TessellatedStroke>,
    byte_budget: usize,
) -> (Vec<TessellatedStroke>, usize, usize, usize) {
    let mut retained_bytes = 0usize;
    let mut rejected = 0usize;
    let mut rejected_tone = 0usize;
    let mut retained = Vec::new();
    for stroke in candidates {
        let bytes = stroke_data_bytes(&stroke);
        if retained_bytes.saturating_add(bytes) <= byte_budget {
            retained_bytes += bytes;
            retained.push(stroke);
        } else {
            rejected += 1;
            if stroke.role == StrokeRole::Tone {
                rejected_tone += 1;
            }
        }
    }
    (retained, retained_bytes, rejected, rejected_tone)
}

/// Builds a packet from a surface whose revision-dependent topology was already
/// prepared. This is the preferred entry point for runtime asset caches.
pub fn build_packet_for_surface(
    surface: &crate::NprPreparedSurface,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    build_packet_with_identity(
        surface.geometry(),
        surface.topology(),
        camera,
        viewport,
        style,
        seed,
        debug_view,
        Some(surface),
    )
}

/// Appends author-owned construction marks after resolving every anchor against
/// the immutable source surface. The operation is atomic with respect to anchor
/// validity: a mismatched asset revision returns an error without mutating the
/// packet. Marks that cannot be projected or fit the remaining CPU payload
/// budget are reported in packet diagnostics instead of risking an upload.
pub fn append_construction_marks(
    packet: &mut NprRenderPacket,
    source_surface: &crate::NprPreparedSurface,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    marks: &[NprConstructionMark],
) -> Result<(), NprSurfaceAnchorError> {
    let viewport = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let mut candidates = Vec::new();
    let mut rejected = 0usize;
    for mark in marks {
        if mark.anchors.len() < 2 {
            rejected += 1;
            continue;
        }
        let points = mark
            .anchors
            .iter()
            .map(|anchor| {
                source_surface.sample(*anchor).map(|sample| {
                    camera.project(sample.position, viewport).map(|projected| {
                        (projected.screen, camera.normalized_depth(projected.depth))
                    })
                })
            })
            .collect::<Result<Option<Vec<_>>, _>>()?;
        let Some(points) = points else {
            rejected += 1;
            continue;
        };
        let style = stroke_layer_style(style, mark.width_scale);
        let mut stroke = tessellate_polyline(
            mark.id,
            FeatureClass::Crease,
            &points,
            mark.closed,
            style,
            seed ^ u64::from(mark.id),
        );
        if stroke.vertices.is_empty() {
            rejected += 1;
            continue;
        }
        stroke.role = StrokeRole::Construction;
        apply_stroke_opacity(std::slice::from_mut(&mut stroke), mark.opacity);
        candidates.push(stroke);
    }
    let mut bytes = packet.strokes.iter().map(stroke_data_bytes).sum::<usize>();
    for stroke in candidates {
        let stroke_bytes = stroke_data_bytes(&stroke);
        if bytes.saturating_add(stroke_bytes) <= MAX_STROKE_DATA_BYTES_PER_PACKET {
            bytes += stroke_bytes;
            packet.strokes.push(stroke);
            packet.stats.construction_marks += 1;
        } else {
            rejected += 1;
        }
    }
    packet.stats.construction_rejected += rejected;
    packet.stats.strokes = packet.strokes.len();
    packet.stats.stroke_vertices = packet
        .strokes
        .iter()
        .map(|stroke| stroke.vertices.len())
        .sum();
    packet.stats.stroke_indices = packet
        .strokes
        .iter()
        .map(|stroke| stroke.indices.len())
        .sum();
    packet.stats.stroke_data_bytes = bytes;
    packet.stats.stroke_budget_exhausted |=
        rejected > 0 && bytes >= MAX_STROKE_DATA_BYTES_PER_PACKET;
    Ok(())
}

#[cfg(test)]
fn append_hatching(
    output: &mut Vec<TessellatedStroke>,
    positions: [Vec2; 3],
    depths: [f32; 3],
    shade: f32,
    style: ComicInk,
    seed: u64,
    viewport: [u32; 2],
    budget: &mut usize,
    face_direction: Option<Vec2>,
) {
    let mut segments = Vec::new();
    collect_hatching_segments(
        &mut segments,
        positions,
        depths,
        shade,
        style,
        viewport,
        budget,
        0,
        face_direction,
    );
    output.extend(emit_hatching_segments(segments, style, seed));
}

/// Collect raw hatch segments before tessellation. This makes the shared edge
/// between coplanar triangles an internal path sample rather than a pair of
/// caps and a reset gesture.
fn collect_hatching_segments(
    output: &mut Vec<HatchSegment>,
    positions: [Vec2; 3],
    depths: [f32; 3],
    shade: f32,
    style: ComicInk,
    viewport: [u32; 2],
    budget: &mut usize,
    group: u32,
    face_direction: Option<Vec2>,
) {
    let density = hatching_density(shade, style);
    if density <= 0.001 || *budget == 0 || viewport.contains(&0) {
        return;
    }
    let spacing = (style.hatching_spacing / (0.35 + density * 0.95)).clamp(1.0, 40.0);
    let angle = style.hatching_angle.to_radians();
    let direction = face_direction
        .map(|direction| {
            let (sin, cos) = angle.sin_cos();
            Vec2::new(
                direction.x * cos - direction.y * sin,
                direction.x * sin + direction.y * cos,
            )
        })
        .unwrap_or_else(|| Vec2::new(angle.cos(), angle.sin()));
    let mut collect_direction = |direction: Vec2, lane: u8, coverage_scale: f32| {
        let normal = Vec2::new(-direction.y, direction.x);
        let coordinates = positions.map(|p| p.dot(normal));
        let min = coordinates.iter().copied().fold(f32::INFINITY, f32::min);
        let max = coordinates
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let viewport_corners = [
            Vec2::ZERO,
            Vec2::new(viewport[0] as f32, 0.0),
            Vec2::new(0.0, viewport[1] as f32),
            Vec2::new(viewport[0] as f32, viewport[1] as f32),
        ];
        let viewport_min = viewport_corners
            .iter()
            .map(|point| point.dot(normal))
            .fold(f32::INFINITY, f32::min);
        let viewport_max = viewport_corners
            .iter()
            .map(|point| point.dot(normal))
            .fold(f32::NEG_INFINITY, f32::max);
        let min = min.max(viewport_min);
        let max = max.min(viewport_max);
        if !min.is_finite() || !max.is_finite() || min > max {
            return;
        }
        let mut offset = (min / spacing).ceil() * spacing;
        while offset <= max && *budget > 0 {
            let intersections = triangle_line_intersections(positions, depths, normal, offset);
            if intersections.len() >= 2 {
                let mut line = intersections;
                line.sort_by(|a, b| a.0.dot(direction).total_cmp(&b.0.dot(direction)));
                if let Some(line) = clip_segment_to_viewport([line[0], line[1]], viewport) {
                    output.push(HatchSegment {
                        points: line,
                        group,
                        lane,
                        line: (offset / spacing).round() as i64,
                        coverage: coverage_scale,
                    });
                    *budget -= 1;
                }
            }
            offset += spacing;
        }
    };
    collect_direction(direction, 0, 1.0);
    if style.hatching_cross > 0.001 {
        collect_direction(
            Vec2::new(-direction.y, direction.x),
            1,
            style.hatching_cross.clamp(0.0, 1.0),
        );
    }
}

fn hatching_density(shade: f32, style: ComicInk) -> f32 {
    style.tone_density.clamp(0.0, 1.0) * ((0.5 - shade) / 1.25).clamp(0.0, 1.0)
}

/// Produces a plane family in surface space and projects completed paths only
/// after they have crossed all eligible smooth edges. It is deliberately a
/// separate representation from the screen-space ThreeBand hatch fallback.
fn emit_surface_hatching(
    geometry: &NprGeometry,
    topology: &[crate::TopologyEdge],
    surface: Option<&crate::NprPreparedSurface>,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    face_tones: &[GraphiteTonePlan],
    budget: &mut usize,
) -> HatchingOutput {
    let primary_faces = face_tones
        .iter()
        .map(|tone| tone.is_visible())
        .collect::<Vec<_>>();
    if *budget == 0 || !primary_faces.iter().any(|selected| *selected) {
        return HatchingOutput::default();
    }
    let primary_density = face_tones
        .iter()
        .map(|tone| tone.primary_density)
        .collect::<Vec<_>>();
    let primary_coverage = face_tones
        .iter()
        .map(|tone| tone.primary_coverage)
        .collect::<Vec<_>>();
    let direction_field = SurfaceDirectionField::build(geometry, topology, 0.85);
    let form_axis = direction_field.form_axis();
    // A tonal path is a mark on the object, not a new random screen pattern.
    // Keep its plane family in object space so orbiting the camera only
    // reprojects established paths. The selection/visibility of the path still
    // reacts to the current view later in the pipeline.
    let reference = if form_axis.y.abs() < 0.9 {
        glam::Vec3::Y
    } else {
        glam::Vec3::X
    };
    let transverse = (reference - form_axis * reference.dot(form_axis)).normalize_or_zero();
    let side = form_axis.cross(transverse).normalize_or_zero();
    let (sin, cos) = style.hatching_angle.to_radians().sin_cos();
    let plane_normal = (transverse * cos + side * sin).normalize_or_zero();
    if plane_normal.length_squared() <= 1e-8 {
        return HatchingOutput::default();
    }
    let available = *budget;
    let primary = emit_surface_hatching_lane(
        geometry,
        topology,
        surface,
        &direction_field,
        camera,
        viewport,
        style,
        &primary_faces,
        &primary_density,
        &primary_coverage,
        plane_normal,
        0,
        1.0,
        available,
    );
    let mut output = primary.candidates;
    let mut confidence_rejected = primary.confidence_rejected;
    let cross_faces = face_tones
        .iter()
        .map(|tone| tone.cross_density > 0.001)
        .collect::<Vec<_>>();
    let cross_density = face_tones
        .iter()
        .map(|tone| tone.cross_density)
        .collect::<Vec<_>>();
    let cross_coverage = face_tones
        .iter()
        .map(|tone| tone.cross_coverage)
        .collect::<Vec<_>>();
    if style.hatching_cross > 0.001 && cross_faces.iter().any(|selected| *selected) {
        // Give the second plane family a vertical component. On a vertical
        // surface this produces a true diagonal cross-hatch instead of another
        // parallel longitudinal set.
        let cross_normal = (side + form_axis * 0.72).normalize_or_zero();
        if cross_normal.length_squared() > 1e-8 {
            let cross = emit_surface_hatching_lane(
                geometry,
                topology,
                surface,
                &direction_field,
                camera,
                viewport,
                style,
                &cross_faces,
                &cross_density,
                &cross_coverage,
                cross_normal,
                1,
                style.hatching_cross.clamp(0.0, 1.0),
                available,
            );
            confidence_rejected += cross.confidence_rejected;
            output.extend(cross.candidates);
        }
    }
    let candidates = output
        .into_iter()
        .map(|candidate| {
            (
                RankedCandidate {
                    priority: candidate.coverage,
                    stable_id: u64::from(candidate.id),
                },
                candidate,
            )
        })
        .collect::<Vec<_>>();
    let candidate_count = candidates.len() + confidence_rejected;
    let (candidates, report) = select_ranked(available, candidates);
    let mut strokes = Vec::new();
    let mut corrections = 0;
    let form_style = stroke_layer_style(style, style.form_line_width_scale);
    for candidate in candidates {
        // A correction is a distinct, low-coverage gesture with a stable ID.
        // It refines an accepted hatch, so it never consumes another planned
        // line slot or changes LOD candidate accounting.
        for mut stroke in tessellate_polyline_variants(
            candidate.id,
            FeatureClass::Crease,
            &candidate.points,
            false,
            form_style,
            seed ^ u64::from(candidate.id),
        ) {
            stroke.role = StrokeRole::Tone;
            if candidate.coverage < 1.0 {
                for vertex in &mut stroke.vertices {
                    vertex.coverage *= candidate.coverage * style.form_line_opacity.clamp(0.0, 1.0);
                }
            } else if style.form_line_opacity < 1.0 {
                for vertex in &mut stroke.vertices {
                    vertex.coverage *= style.form_line_opacity.clamp(0.0, 1.0);
                }
            }
            if stroke.correction {
                corrections += 1;
            }
            if !stroke.vertices.is_empty() {
                strokes.push(stroke);
            }
        }
    }
    *budget = budget.saturating_sub(report.accepted);
    HatchingOutput {
        strokes,
        candidates: candidate_count,
        rejected: report.rejected + confidence_rejected,
        confidence_rejected,
        corrections,
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_surface_hatching_lane(
    geometry: &NprGeometry,
    topology: &[crate::TopologyEdge],
    surface: Option<&crate::NprPreparedSurface>,
    direction_field: &SurfaceDirectionField,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    selected_faces: &[bool],
    face_density: &[f32],
    face_coverage: &[f32],
    plane_normal: glam::Vec3,
    lane: u32,
    coverage_scale: f32,
    max_paths: usize,
) -> SurfaceHatchLaneOutput {
    let vp = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let (min, max) = geometry
        .vertices
        .iter()
        .map(|vertex| plane_normal.dot(vertex.position))
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        });
    if !min.is_finite() || !max.is_finite() || max - min <= 1e-6 {
        return SurfaceHatchLaneOutput::default();
    }
    let density = face_density.iter().copied().fold(0.0f32, f32::max);
    let pixel_spacing = (style.hatching_spacing / (0.35 + density * 0.95)).clamp(1.0, 40.0);
    // This is a surface-layout spacing, not the final pixel width of a mark.
    // Deriving it from the current camera projected an entirely new plane grid
    // whenever an object rotated. A fixed local reference scale preserves the
    // generated paths; view-dependent LOD may later select a nested subset.
    const LAYOUT_REFERENCE_PIXELS: f32 = 128.0;
    let world_spacing = ((max - min) * pixel_spacing / LAYOUT_REFERENCE_PIXELS).max(1e-4);
    let first = (min / world_spacing).ceil() * world_spacing;
    let offsets = std::iter::successors(Some(first), |offset| {
        Some(*offset + world_spacing).filter(|next| *next <= max + 1e-6)
    })
    .take(max_paths)
    .collect::<Vec<_>>();
    let paths = trace_parallel_surface_lines(
        geometry,
        topology,
        plane_normal,
        offsets,
        selected_faces,
        0.85,
        max_paths,
    );
    let mut output = SurfaceHatchLaneOutput::default();
    for seed_path in paths {
        let Some(start_face) = seed_path.faces.first().copied() else {
            continue;
        };
        let seed_id = stable_surface_path_id_for_surface(geometry, surface, lane, &seed_path);
        let curvature = seed_path
            .faces
            .iter()
            .map(|face| direction_field.face_curvature(*face as usize))
            .fold(0.0f32, f32::max);
        let max_steps = streamline_step_count(curvature, seed_id);
        // Plane intersections distribute the seeds evenly. The final line is
        // then integrated in the local surface field, which lets graphite flow
        // around smooth curvature instead of reading as a screen-space overlay.
        let traced = trace_surface_streamline(
            geometry,
            topology,
            direction_field,
            selected_faces,
            start_face,
            seed_path.points[seed_path.points.len() / 2],
            (world_spacing * 0.8).max(1e-4),
            max_steps,
            0.85,
        );
        let path = if traced.points.len() >= 2 {
            traced
        } else {
            seed_path
        };
        let form_confidence = surface_path_form_confidence(direction_field, &path);
        if form_confidence < style.min_form_line_confidence.clamp(0.0, 1.0) {
            output.confidence_rejected += 1;
            continue;
        }
        let points = path
            .points
            .iter()
            .map(|point| {
                camera
                    .project(*point, vp)
                    .map(|projected| (projected.screen, camera.normalized_depth(projected.depth)))
            })
            .collect::<Option<Vec<_>>>();
        let Some(points) = points else { continue };
        if points.len() < 2 {
            continue;
        }
        let id = stable_surface_path_id_for_surface(geometry, surface, lane, &path);
        let path_density = path
            .faces
            .iter()
            .filter_map(|face| face_density.get(*face as usize).copied())
            .fold(0.0f32, f32::max);
        let path_coverage = path
            .faces
            .iter()
            .filter_map(|face| face_coverage.get(*face as usize).copied())
            .fold(0.0f32, f32::max);
        // The primary family must retain a visible graphite base even in a
        // light form region; otherwise a continuous value map degenerates into
        // nearly invisible geometry. Cross-hatching still uses its authored
        // low lane coverage on top of this form response.
        let form_coverage = 0.42 + 0.58 * (path_density / density.max(1e-4)).clamp(0.0, 1.0);
        let coverage_scale =
            coverage_scale * form_coverage * path_coverage * form_confidence.sqrt();
        output.candidates.push(SurfaceHatchCandidate {
            id,
            points,
            coverage: coverage_scale,
        });
    }
    output
}

/// A reliable form line needs both an unambiguous tangent and a locally smooth
/// normal field. Curvature does not make a form invalid by itself; it simply
/// lowers trust when a short, irregular triangle fan would otherwise imprint
/// its sampling pattern into a graphite stroke.
fn surface_path_form_confidence(
    direction_field: &SurfaceDirectionField,
    path: &crate::SurfaceHatchPath,
) -> f32 {
    let mut count = 0usize;
    let mut tangent_confidence = 0.0f32;
    let mut curvature_penalty = 0.0f32;
    for face in &path.faces {
        let face = *face as usize;
        tangent_confidence += direction_field.face_confidence(face);
        curvature_penalty += direction_field.face_curvature(face).sqrt();
        count += 1;
    }
    if count == 0 {
        return 0.0;
    }
    let tangent_confidence = tangent_confidence / count as f32;
    let curvature_penalty = curvature_penalty / count as f32;
    // Square-rooted normal turn preserves broad, smooth bends but strongly
    // discounts abrupt one-ring changes caused by coarse or noisy sampling.
    tangent_confidence * (1.0 - curvature_penalty).clamp(0.0, 1.0)
}

/// Surface endpoints are a more durable identity than an emission index. They
/// are encoded as face-local barycentric anchors rather than world positions,
/// so rigid object transforms cannot reroll a path's gesture. The canonical
/// endpoint order makes tracing direction irrelevant.
#[cfg(test)]
fn stable_surface_path_id(
    geometry: &NprGeometry,
    lane: u32,
    path: &crate::SurfaceHatchPath,
) -> u32 {
    stable_surface_path_id_for_surface(geometry, None, lane, path)
}

fn stable_surface_path_id_for_surface(
    geometry: &NprGeometry,
    surface: Option<&crate::NprPreparedSurface>,
    lane: u32,
    path: &crate::SurfaceHatchPath,
) -> u32 {
    let (Some(first), Some(first_face)) = (path.points.first(), path.faces.first()) else {
        return 0x6000_0000 ^ lane;
    };
    let last = path.points.last().unwrap_or(first);
    let last_face = path.faces.last().unwrap_or(first_face);
    let first = surface_path_anchor_key(geometry, surface, *first_face, *first);
    let last = surface_path_anchor_key(geometry, surface, *last_face, *last);
    let (first, last) = if first > last {
        (last, first)
    } else {
        (first, last)
    };
    let mut hash = 0xcbf2_9ce4_8422_2325u64 ^ u64::from(lane);
    for value in first.into_iter().chain(last) {
        hash ^= u64::from(value);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    0x6000_0000 | ((hash ^ (hash >> 32)) as u32 & 0x1fff_ffff)
}

fn surface_path_anchor_key(
    geometry: &NprGeometry,
    surface: Option<&crate::NprPreparedSurface>,
    face: u32,
    point: glam::Vec3,
) -> Vec<u32> {
    if let Some(anchor) =
        surface.and_then(|surface| surface.source_anchor_at_point(face, point).ok())
    {
        return vec![
            anchor.content_id.0 as u32,
            (anchor.content_id.0 >> 32) as u32,
            anchor.triangle,
            quantize_surface_anchor(anchor.barycentric[0]),
            quantize_surface_anchor(anchor.barycentric[1]),
            quantize_surface_anchor(anchor.barycentric[2]),
        ];
    }
    let triangle = geometry.triangles[face as usize];
    let a = geometry.vertices[triangle[0] as usize].position;
    let b = geometry.vertices[triangle[1] as usize].position;
    let c = geometry.vertices[triangle[2] as usize].position;
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let dot_ab_ab = ab.dot(ab);
    let dot_ab_ac = ab.dot(ac);
    let dot_ac_ac = ac.dot(ac);
    let denominator = dot_ab_ab * dot_ac_ac - dot_ab_ac * dot_ab_ac;
    let barycentric = if denominator.abs() <= 1e-10 {
        [1.0, 0.0, 0.0]
    } else {
        let v = (dot_ac_ac * ap.dot(ab) - dot_ab_ac * ap.dot(ac)) / denominator;
        let w = (dot_ab_ab * ap.dot(ac) - dot_ab_ac * ap.dot(ab)) / denominator;
        [1.0 - v - w, v, w]
    };
    vec![
        face,
        quantize_surface_anchor(barycentric[0]),
        quantize_surface_anchor(barycentric[1]),
        quantize_surface_anchor(barycentric[2]),
    ]
}

fn quantize_surface_anchor(value: f32) -> u32 {
    (value.clamp(-1.0, 1.0) * 65_535.0).round() as i32 as u32
}

fn stable_hatch_noise(id: u32) -> f32 {
    let mut value = u64::from(id) ^ 0x9e37_79b9_7f4a_7c15;
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    ((value >> 32) as u32) as f32 / u32::MAX as f32
}

fn streamline_step_count(curvature: f32, id: u32) -> usize {
    // The one-ring normal turn is small for a well-tessellated cylinder, so
    // normalize its useful range before it influences stroke extent.
    let curve_weight = (curvature * 8.0).clamp(0.0, 1.0);
    let step_variation = 0.18 + stable_hatch_noise(id) * 0.28;
    (24.0 * (1.0 - curve_weight * step_variation))
        .round()
        .clamp(6.0, 24.0) as usize
}

fn emit_hatching_segments(
    segments: Vec<HatchSegment>,
    style: ComicInk,
    seed: u64,
) -> Vec<TessellatedStroke> {
    let mut buckets: BTreeMap<HatchKey, Vec<HatchSegment>> = BTreeMap::new();
    for segment in segments {
        buckets
            .entry(HatchKey {
                group: segment.group,
                lane: segment.lane,
                line: segment.line,
            })
            .or_default()
            .push(segment);
    }
    let mut output = Vec::new();
    for (key, mut segments) in buckets {
        while let Some(first) = segments.pop() {
            let mut path = vec![first.points[0], first.points[1]];
            let coverage = first.coverage;
            loop {
                let tail = *path.last().unwrap();
                let head = path[0];
                let mut next = None;
                for (index, segment) in segments.iter().enumerate() {
                    if tail.0.distance_squared(segment.points[0].0) <= 1e-4 {
                        next = Some((index, false, false));
                        break;
                    }
                    if tail.0.distance_squared(segment.points[1].0) <= 1e-4 {
                        next = Some((index, true, false));
                        break;
                    }
                    if head.0.distance_squared(segment.points[1].0) <= 1e-4 {
                        next = Some((index, false, true));
                        break;
                    }
                    if head.0.distance_squared(segment.points[0].0) <= 1e-4 {
                        next = Some((index, true, true));
                        break;
                    }
                }
                let Some((index, reverse, prepend)) = next else {
                    break;
                };
                let segment = segments.swap_remove(index);
                let (a, b) = if reverse {
                    (segment.points[1], segment.points[0])
                } else {
                    (segment.points[0], segment.points[1])
                };
                if prepend {
                    path.insert(0, a);
                } else {
                    path.push(b);
                }
            }
            let id = key.group.wrapping_mul(73_856_093)
                ^ (key.line as u32).wrapping_mul(19_349_663)
                ^ u32::from(key.lane).wrapping_mul(83_492_791);
            let mut stroke = tessellate_polyline(
                id,
                crate::feature::FeatureClass::Crease,
                &path,
                false,
                style,
                seed ^ u64::from(id),
            );
            stroke.role = StrokeRole::Tone;
            if coverage < 1.0 {
                for vertex in &mut stroke.vertices {
                    vertex.coverage *= coverage;
                }
            }
            if !stroke.vertices.is_empty() {
                output.push(stroke);
            }
        }
    }
    output
}

fn clip_segment_to_viewport(
    segment: [(Vec2, f32); 2],
    viewport: [u32; 2],
) -> Option<[(Vec2, f32); 2]> {
    let start = segment[0].0;
    let delta = segment[1].0 - start;
    let max = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let mut lower: f32 = 0.0;
    let mut upper: f32 = 1.0;
    for (origin, step, lower_bound, upper_bound) in [
        (start.x, delta.x, 0.0, max.x),
        (start.y, delta.y, 0.0, max.y),
    ] {
        if step.abs() <= f32::EPSILON {
            if origin < lower_bound || origin > upper_bound {
                return None;
            }
            continue;
        }
        let mut first = (lower_bound - origin) / step;
        let mut second = (upper_bound - origin) / step;
        if first > second {
            std::mem::swap(&mut first, &mut second);
        }
        lower = lower.max(first);
        upper = upper.min(second);
        if lower > upper {
            return None;
        }
    }
    let make = |t: f32| {
        (
            start + delta * t,
            segment[0].1 + (segment[1].1 - segment[0].1) * t,
        )
    };
    Some([make(lower), make(upper)])
}

fn triangle_line_intersections(
    positions: [Vec2; 3],
    depths: [f32; 3],
    normal: Vec2,
    offset: f32,
) -> Vec<(Vec2, f32)> {
    let mut intersections = Vec::with_capacity(3);
    for i in 0..3 {
        let j = (i + 1) % 3;
        let a = positions[i];
        let b = positions[j];
        let da = a.dot(normal) - offset;
        let db = b.dot(normal) - offset;
        if da.abs() <= 1e-4 {
            intersections.push((a, depths[i]));
        }
        if da * db < 0.0 {
            let t = da / (da - db);
            intersections.push((a.lerp(b, t), depths[i] + (depths[j] - depths[i]) * t));
        }
    }
    intersections.sort_by(|a, b| {
        a.0.x
            .total_cmp(&b.0.x)
            .then_with(|| a.0.y.total_cmp(&b.0.y))
    });
    intersections.dedup_by(|a, b| a.0.distance_squared(b.0) < 1e-4);
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify_features;

    #[test]
    fn surface_path_identity_is_independent_of_trace_orientation() {
        let geometry = NprGeometry::from_indexed(
            &[[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        )
        .unwrap();
        let forward = crate::SurfaceHatchPath {
            points: vec![
                glam::Vec3::new(-0.25, 0.0, 0.0),
                glam::Vec3::new(0.25, 0.0, 0.0),
            ],
            faces: vec![0, 0],
        };
        let backward = crate::SurfaceHatchPath {
            points: forward.points.iter().copied().rev().collect(),
            faces: vec![0, 0],
        };
        assert_eq!(
            stable_surface_path_id(&geometry, 1, &forward),
            stable_surface_path_id(&geometry, 1, &backward)
        );
        assert_ne!(
            stable_surface_path_id(&geometry, 0, &forward),
            stable_surface_path_id(&geometry, 1, &forward)
        );
    }

    #[test]
    fn surface_path_identity_uses_intrinsic_anchors_not_world_positions() {
        let geometry = NprGeometry::from_indexed(
            &[[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        )
        .unwrap();
        let path = crate::SurfaceHatchPath {
            points: vec![
                glam::Vec3::new(-0.25, 0.0, 0.0),
                glam::Vec3::new(0.25, 0.0, 0.0),
            ],
            faces: vec![0, 0],
        };
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(2.0),
            glam::Quat::from_rotation_y(0.8),
            glam::Vec3::new(3.0, -2.0, 1.0),
        );
        let transformed = crate::SurfaceHatchPath {
            points: path
                .points
                .iter()
                .map(|point| transform.transform_point3(*point))
                .collect(),
            faces: path.faces.clone(),
        };
        assert_eq!(
            stable_surface_path_id(&geometry, 0, &path),
            stable_surface_path_id(&geometry.transformed(transform), 0, &transformed)
        );
    }

    #[test]
    fn curvature_shortens_streamline_extent_without_affecting_flat_faces() {
        let id = 0x6000_0123;
        assert_eq!(streamline_step_count(0.0, id), 24);
        let curved = streamline_step_count(0.20, id);
        assert!((6..24).contains(&curved));
        assert_eq!(curved, streamline_step_count(0.20, id));
    }

    #[test]
    fn cube_has_18_edges() {
        assert_eq!(build_topology(&NprGeometry::canonical_cube()).len(), 18);
    }
    #[test]
    fn packet_is_deterministic() {
        let g = NprGeometry::canonical_cube();
        let c = PerspectiveCamera::cube_default(1.0);
        let s = ComicInk {
            wobble: 0.4,
            ..Default::default()
        };
        assert_eq!(
            build_packet(&g, c, [512, 512], s, 7, NprDebugView::Final),
            build_packet(&g, c, [512, 512], s, 7, NprDebugView::Final)
        );
    }
    #[test]
    fn near_clipping_keeps_crossing_segment() {
        let c = PerspectiveCamera::cube_default(1.0);
        assert!(c
            .project_segment(
                c.position + c.forward * 0.01,
                c.position + c.forward * 2.0,
                Vec2::splat(512.0)
            )
            .is_some());
    }

    #[test]
    fn triangulation_diagonals_do_not_become_feature_strokes() {
        let geometry = NprGeometry::canonical_cube();
        let topology = build_topology(&geometry);
        let features = classify_features(&geometry, &topology, glam::Vec3::Z, 0.35);
        assert!(features.len() <= 12);
    }

    #[test]
    fn stroke_payload_budget_preserves_feature_strokes_before_tone() {
        let vertex = crate::StrokeVertex {
            position: Vec2::ZERO,
            width: 1.0,
            id: 7,
            depth: 0.5,
            pressure: 1.0,
            coverage: 1.0,
            grain: 0.0,
            edge: 0.0,
            edge_softness: 0.0,
            paper_tooth: 0.0,
            dryness: 0.0,
        };
        let feature = TessellatedStroke {
            vertices: vec![vertex; 3],
            indices: vec![0, 1, 2],
            id: 1,
            class: FeatureClass::Silhouette,
            role: StrokeRole::Feature,
            correction: false,
        };
        let mut tone = feature.clone();
        tone.id = 2;
        tone.role = StrokeRole::Tone;
        let (retained, bytes, rejected, rejected_tone) =
            retain_strokes_under_budget(vec![feature.clone(), tone], stroke_data_bytes(&feature));
        assert_eq!(retained.len(), 1);
        assert_eq!(retained[0].role, StrokeRole::Feature);
        assert_eq!(bytes, stroke_data_bytes(&feature));
        assert_eq!((rejected, rejected_tone), (1, 1));
    }

    #[test]
    fn ranking_rejects_only_short_crease_chains() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera {
            position: glam::Vec3::new(3.0, 2.0, 4.0),
            forward: glam::Vec3::new(-3.0, -2.0, -4.0).normalize(),
            up: glam::Vec3::Y,
            vertical_fov: 0.9,
            near: 0.05,
            aspect: 1.0,
        };
        let chains = vec![
            crate::FeatureStroke {
                id: 1,
                class: FeatureClass::Crease,
                vertices: vec![0, 1],
            },
            crate::FeatureStroke {
                id: 2,
                class: FeatureClass::Boundary,
                vertices: vec![0, 1],
            },
        ];
        let (selected, rejected) =
            rank_feature_chains(chains, &geometry, camera, Vec2::splat(512.0), 10_000.0);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].class, FeatureClass::Boundary);
        assert_eq!(rejected, 1);
    }

    #[test]
    fn smooth_surface_replaces_polygon_silhouettes_with_field_contours() {
        let geometry = NprGeometry::icosphere();
        let camera = PerspectiveCamera::cube_default(1.0);
        let polygonal = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk::default(),
            42,
            NprDebugView::Final,
        );
        let smooth = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk {
                surface_mode: NprSurfaceMode::Smooth,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );

        assert!(smooth.stats.silhouettes > 0);
        assert_eq!(smooth.stats.smooth_contour_spans, smooth.stats.silhouettes);
        assert!(smooth.stats.feature_segments < polygonal.stats.feature_segments);
        assert_eq!(smooth.stats.creases, 0);
        assert_eq!(
            smooth,
            build_packet(
                &geometry,
                camera,
                [512, 512],
                ComicInk {
                    surface_mode: NprSurfaceMode::Smooth,
                    ..Default::default()
                },
                42,
                NprDebugView::Final,
            )
        );
    }

    #[test]
    fn suggestive_contours_are_opt_in_packet_features() {
        let geometry = NprGeometry::from_indexed(
            &[
                [-1.0, -1.0, 1.0],
                [0.0, -1.0, 0.0],
                [1.0, -1.0, -1.0],
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [-1.0, 1.0, -1.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            &[
                0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 3, 4, 7, 3, 7, 6, 4, 5, 8, 4, 8, 7,
            ],
        )
        .unwrap();
        let camera = PerspectiveCamera {
            position: glam::Vec3::new(0.2, 0.1, 4.0),
            forward: glam::Vec3::new(-0.2, -0.1, -4.0).normalize(),
            up: glam::Vec3::Y,
            vertical_fov: 0.9,
            near: 0.05,
            aspect: 1.0,
        };
        let base = ComicInk {
            surface_mode: NprSurfaceMode::Smooth,
            ..Default::default()
        };
        let without = build_packet(&geometry, camera, [512, 512], base, 7, NprDebugView::Final);
        let with = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk {
                suggestive_contours: true,
                suggestive_contour_confidence: 0.0,
                ..base
            },
            7,
            NprDebugView::Final,
        );
        assert_eq!(without.stats.suggestive_contour_spans, 0);
        assert!(with.stats.suggestive_contour_spans > 0);
        assert!(with.strokes.len() > without.strokes.len());

        let full = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk {
                suggestive_contours: true,
                suggestive_contour_confidence: 0.0,
                suggestive_contour_width_scale: 1.0,
                suggestive_contour_opacity: 1.0,
                ..base
            },
            7,
            NprDebugView::Final,
        );
        let maximum = |packet: &NprRenderPacket, field: fn(&crate::StrokeVertex) -> f32| {
            packet
                .strokes
                .iter()
                .filter(|stroke| stroke.id & 0xf000_0000 == 0x3000_0000)
                .flat_map(|stroke| stroke.vertices.iter())
                .map(field)
                .fold(0.0f32, f32::max)
        };
        assert!(maximum(&full, |vertex| vertex.width) > maximum(&with, |vertex| vertex.width));
        assert!(
            maximum(&full, |vertex| vertex.coverage) > maximum(&with, |vertex| vertex.coverage)
        );
    }

    #[test]
    fn form_line_layer_scales_only_crease_width() {
        let style = ComicInk::default();
        let layered = stroke_layer_style(style, 0.4);
        assert_eq!(layered.crease_width, style.crease_width * 0.4);
        assert_eq!(layered.outline_width, style.outline_width);
        assert_eq!(layered.boundary_width, style.boundary_width);
    }

    #[test]
    fn construction_marks_resolve_source_anchors_atomically() {
        let source = crate::NprPreparedSurface::new(NprGeometry::canonical_cube());
        let mark = NprConstructionMark {
            id: 0x4000_0001,
            anchors: vec![
                source.anchor(0, [0.70, 0.20, 0.10]).unwrap(),
                source.anchor(0, [0.10, 0.70, 0.20]).unwrap(),
            ],
            closed: false,
            width_scale: 0.5,
            opacity: 0.4,
        };
        let camera = PerspectiveCamera::cube_default(1.0);
        let mut first = build_packet_for_surface(
            &source,
            camera,
            [512, 512],
            ComicInk::default(),
            11,
            NprDebugView::Final,
        );
        append_construction_marks(
            &mut first,
            &source,
            camera,
            [512, 512],
            ComicInk::default(),
            11,
            std::slice::from_ref(&mark),
        )
        .unwrap();
        assert_eq!(first.stats.construction_marks, 1);
        assert!(first
            .strokes
            .iter()
            .any(|stroke| stroke.id == mark.id && stroke.role == StrokeRole::Construction));

        let mut second = build_packet_for_surface(
            &source,
            camera,
            [512, 512],
            ComicInk::default(),
            11,
            NprDebugView::Final,
        );
        append_construction_marks(
            &mut second,
            &source,
            camera,
            [512, 512],
            ComicInk::default(),
            11,
            std::slice::from_ref(&mark),
        )
        .unwrap();
        assert_eq!(first, second);

        let incompatible = crate::NprPreparedSurface::new(NprGeometry::wedge());
        let before = second.clone();
        assert!(append_construction_marks(
            &mut second,
            &incompatible,
            camera,
            [512, 512],
            ComicInk::default(),
            11,
            std::slice::from_ref(&mark),
        )
        .is_err());
        assert_eq!(second, before);
    }

    #[test]
    fn smooth_surface_keeps_explicitly_hard_cube_creases() {
        let packet = build_packet(
            &NprGeometry::canonical_cube(),
            PerspectiveCamera::cube_default(1.0),
            [512, 512],
            ComicInk {
                surface_mode: NprSurfaceMode::Smooth,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );
        assert!(packet.stats.creases > 0);
    }

    #[test]
    fn cube_packet_is_stable_for_thirty_six_rotations() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera::cube_default(1.0);
        for step in 0..36 {
            let angle = step as f32 * std::f32::consts::TAU / 36.0;
            let mut rotated = geometry.clone();
            let rotation = glam::Mat3::from_rotation_y(angle);
            for vertex in &mut rotated.vertices {
                vertex.position = rotation * vertex.position;
            }
            let first = build_packet(
                &rotated,
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            );
            let second = build_packet(
                &rotated,
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            );
            assert_eq!(first, second);
        }
    }

    #[test]
    fn tone_hatching_is_opt_in_and_stays_inside_face_bounds() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera::cube_default(1.0);
        let plain = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk::default(),
            42,
            NprDebugView::Final,
        );
        let hatched = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk {
                tone_density: 0.8,
                hatching_spacing: 5.0,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );
        assert!(hatched.strokes.len() > plain.strokes.len());
        assert!(hatched.strokes.iter().all(|stroke| {
            stroke
                .vertices
                .iter()
                .all(|vertex| vertex.position.is_finite())
        }));
    }

    #[test]
    fn extreme_hatching_is_clipped_and_budgeted() {
        let mut strokes = Vec::new();
        let mut budget = MAX_HATCHING_LINES_PER_PACKET;
        append_hatching(
            &mut strokes,
            [
                Vec2::new(-1.0e9, -1.0e9),
                Vec2::new(1.0e9, -1.0e9),
                Vec2::new(0.0, 1.0e9),
            ],
            [0.5; 3],
            -1.0,
            ComicInk {
                tone_density: 1.0,
                hatching_spacing: 1.0,
                hatching_cross: 1.0,
                ..Default::default()
            },
            42,
            [512, 512],
            &mut budget,
            None,
        );
        assert!(strokes.len() <= MAX_HATCHING_LINES_PER_PACKET);
        assert!(strokes
            .iter()
            .flat_map(|stroke| stroke.vertices.iter())
            .all(|vertex| vertex.position.is_finite()));
    }

    #[test]
    fn hatching_tone_keeps_occlusion_without_a_colored_fill() {
        let geometry = NprGeometry::canonical_cube();
        let packet = build_packet(
            &geometry,
            PerspectiveCamera::cube_default(1.0),
            [512, 512],
            ComicInk {
                tone_mode: NprToneMode::Hatching,
                tone_density: 0.8,
                hatching_spacing: 6.0,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );
        assert!(!packet.occluders.is_empty());
        assert!(packet.fills.is_empty());
        assert!(packet.stats.hatching_strokes > 0);
        assert!(packet.stats.graphite_mass > 0.0);
        assert!(packet.stats.hatching_candidates >= packet.stats.hatching_strokes);
        assert_eq!(
            packet.stats.hatching_candidates + packet.stats.hatching_correction_strokes,
            packet.stats.hatching_strokes + packet.stats.hatching_rejected
        );
        assert_eq!(
            packet
                .strokes
                .iter()
                .filter(|stroke| stroke.role == StrokeRole::Tone)
                .count(),
            packet.stats.hatching_strokes
        );
        let crossed = build_packet(
            &geometry,
            PerspectiveCamera::cube_default(1.0),
            [512, 512],
            ComicInk {
                tone_mode: NprToneMode::Hatching,
                tone_density: 0.8,
                hatching_spacing: 6.0,
                hatching_cross: 0.5,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );
        assert!(crossed.stats.hatching_strokes > packet.stats.hatching_strokes);
        assert_eq!(
            build_packet(
                &geometry,
                PerspectiveCamera::cube_default(1.0),
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            )
            .stats
            .graphite_mass,
            0.0
        );
    }

    #[test]
    fn tonal_corrections_are_separate_from_hatching_candidates() {
        let packet = build_packet(
            &NprGeometry::canonical_cube(),
            PerspectiveCamera::cube_default(1.0),
            [512, 512],
            ComicInk {
                tool: crate::StrokeTool::Pencil,
                tone_mode: NprToneMode::Hatching,
                tone_density: 0.9,
                hatching_spacing: 5.0,
                gesture_correction: 0.7,
                gesture_overstroke: 0.35,
                wobble: 1.0,
                ..Default::default()
            },
            71,
            NprDebugView::Final,
        );
        assert!(packet.stats.hatching_correction_strokes > 0);
        assert_eq!(
            packet.stats.hatching_correction_strokes,
            packet
                .strokes
                .iter()
                .filter(|stroke| stroke.role == StrokeRole::Tone && stroke.correction)
                .count()
        );
        assert_eq!(
            packet.stats.hatching_candidates + packet.stats.hatching_correction_strokes,
            packet.stats.hatching_strokes + packet.stats.hatching_rejected
        );
    }

    #[test]
    fn form_line_confidence_rejects_ambiguous_tonal_paths_before_tessellation() {
        let mut style = ComicInk::default();
        style.tone_mode = NprToneMode::Hatching;
        style.tone_density = 1.0;
        style.min_form_line_confidence = 1.0;
        let packet = build_packet(
            &NprGeometry::cylinder(16),
            PerspectiveCamera::cube_default(1.0),
            [512, 512],
            style,
            7,
            NprDebugView::Final,
        );
        assert!(packet.stats.hatching_candidates > 0);
        assert!(packet.stats.hatching_confidence_rejected > 0);
        assert_eq!(packet.stats.hatching_strokes, 0);
        assert_eq!(
            packet.stats.hatching_candidates,
            packet.stats.hatching_rejected
        );
    }
}
