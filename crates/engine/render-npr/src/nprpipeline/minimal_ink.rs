use super::{
    contracts::{NprLogicalMark, NprPipelineContext, NprPipelineInput},
    pipeline::NprPipeline,
    strategies::{
        NprFeatureStrategy, NprGestureStrategy, NprHatchingStrategy, NprMarkStrategy, NprMediumStrategy, NprPaperStrategy,
        NprProjectionStrategy, NprSurfaceStrategy, NprValueStrategy,
        NprSalienceStrategy,
    },
};
use crate::{
    FeatureClass, GraphiteMedium, InkMedium, NprBrushLibrary, NprFillTriangle, NprMediumDefinition, NprPaperDefinition, NprRenderPacket, NprRenderStats, NprSurfaceDirectionField, PencilBrushLibrary, ProjectedPoint,
    TessellatedStroke, TopologyEdge, build_topology, classify_features, face_normal,
    tessellate_polyline_with_depth, tessellate_segment_with_depth, DEFAULT_NPR_PIPELINE_STAGES, NprSaliencePolicy,
    NprValueField, select_salient_features,
};
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// Composable implementation of the currently shipped flat-fill/ink pipeline.
/// Every decision is a strategy so later pencil, ink, or charcoal profiles can
/// replace stages without forking the renderer.
pub struct MinimalInkPipeline {
    surface: Arc<dyn NprSurfaceStrategy>,
    features: Arc<dyn NprFeatureStrategy>,
    salience: Arc<dyn NprSalienceStrategy>,
    projection: Arc<dyn NprProjectionStrategy>,
    values: Arc<dyn NprValueStrategy>,
    marks: Arc<dyn NprMarkStrategy>,
    hatching: Arc<dyn NprHatchingStrategy>,
    gesture: Arc<dyn NprGestureStrategy>,
    paper: Arc<dyn NprPaperStrategy>,
    medium: Arc<dyn NprMediumStrategy>,
}

impl Default for MinimalInkPipeline {
    fn default() -> Self {
        Self {
            surface: Arc::new(TopologySurfaceStrategy),
            features: Arc::new(ContourFeatureStrategy),
            salience: Arc::new(AllFeatureSalienceStrategy),
            projection: Arc::new(CameraProjectionStrategy),
            values: Arc::new(ThreeBandValueStrategy),
            marks: Arc::new(FeatureMarkStrategy),
            hatching: Arc::new(NoHatchingStrategy),
            gesture: Arc::new(FlatInkGestureStrategy),
            paper: Arc::new(FlatPaperStrategy),
            medium: Arc::new(InkMediumStrategy),
        }
    }
}

impl MinimalInkPipeline {
    pub fn with_strategies(
        surface: Arc<dyn NprSurfaceStrategy>,
        features: Arc<dyn NprFeatureStrategy>,
        salience: Arc<dyn NprSalienceStrategy>,
        projection: Arc<dyn NprProjectionStrategy>,
        values: Arc<dyn NprValueStrategy>,
        marks: Arc<dyn NprMarkStrategy>,
        hatching: Arc<dyn NprHatchingStrategy>,
        gesture: Arc<dyn NprGestureStrategy>,
        paper: Arc<dyn NprPaperStrategy>,
        medium: Arc<dyn NprMediumStrategy>,
    ) -> Self {
        Self { surface, features, salience, projection, values, marks, hatching, gesture, paper, medium }
    }
}

impl NprPipeline for MinimalInkPipeline {
    fn id(&self) -> &'static str { "minimal-ink" }
    fn stages(&self) -> &'static [crate::NprPipelineStage] { DEFAULT_NPR_PIPELINE_STAGES }

    fn build(&self, input: NprPipelineInput<'_>) -> NprRenderPacket {
        let mut context = NprPipelineContext::new(input);
        self.surface.prepare(&mut context);
        context.features = self.features.detect(&context, &context.topology);
        context.selected_features = self.salience.select(&context, &context.features);
        context.fills = self.values.plan_fills(&context, self.projection.as_ref());
        context.marks = self.marks.plan_marks(&context, self.projection.as_ref());
        context.marks.extend(self.hatching.plan_hatching(&context, self.projection.as_ref()));
        context.strokes = self.gesture.realize(&context, &context.marks);
        let silhouettes = context.features.iter().filter(|feature| feature.class == FeatureClass::Silhouette).count();
        let creases = context.features.iter().filter(|feature| feature.class == FeatureClass::Crease).count();
        let hatching_marks = context.marks.iter().filter(|mark| mark.class == FeatureClass::Hatching).count();
        let stats = NprRenderStats {
            geometry: 1,
            topology_edges: context.topology.len(),
            feature_segments: context.selected_features.len(),
            silhouettes,
            creases,
            hatching_marks,
            strokes: context.strokes.len(),
            stroke_vertices: context.strokes.iter().map(|stroke| stroke.vertices.len()).sum(),
            stroke_indices: context.strokes.iter().map(|stroke| stroke.indices.len()).sum(),
            viewport: context.input.viewport,
        };
        let paper = self.paper.paper(&context).normalized();
        let background = paper.color;
        let medium = self.medium.medium(&context);
        NprRenderPacket {
            fills: context.fills,
            strokes: context.strokes,
            background,
            paper,
            medium,
            material_epoch: context.input.temporal.material_epoch,
            debug_view: context.input.debug_view,
            stats,
        }
    }
}

/// Studio-style pencil line animation: paper plus selective live graphite
/// marks. Tone masses and hatching are intentionally disabled in this first
/// preset; they return later as sparse authored layers rather than a grid.
pub struct PencilAnimationPipeline {
    inner: MinimalInkPipeline,
}

impl Default for PencilAnimationPipeline {
    fn default() -> Self {
        Self {
            inner: MinimalInkPipeline::with_strategies(
                Arc::new(TopologySurfaceStrategy),
                Arc::new(ContourFeatureStrategy),
                Arc::new(SelectiveContourSalienceStrategy { minimum_crease_length_px: 34.0 }),
                Arc::new(CameraProjectionStrategy),
                Arc::new(NoValueStrategy),
                Arc::new(FeatureMarkStrategy),
                Arc::new(NoHatchingStrategy),
                Arc::new(GraphiteGestureStrategy::default()),
                Arc::new(GraphitePaperStrategy),
                Arc::new(GraphiteMediumStrategy),
            ),
        }
    }
}

impl NprPipeline for PencilAnimationPipeline {
    fn id(&self) -> &'static str { "pencil-animation" }
    fn stages(&self) -> &'static [crate::NprPipelineStage] { DEFAULT_NPR_PIPELINE_STAGES }
    fn build(&self, input: NprPipelineInput<'_>) -> NprRenderPacket { self.inner.build(input) }
}

pub struct TopologySurfaceStrategy;
impl NprSurfaceStrategy for TopologySurfaceStrategy {
    fn prepare(&self, context: &mut NprPipelineContext<'_>) {
        context.topology = build_topology(context.input.geometry);
    }
}

pub struct ContourFeatureStrategy;
impl NprFeatureStrategy for ContourFeatureStrategy {
    fn detect(&self, context: &NprPipelineContext<'_>, topology: &[TopologyEdge]) -> Vec<crate::FeatureSegment> {
        classify_features(context.input.geometry, topology, -context.input.camera.forward(), 0.35)
    }
}

pub struct AllFeatureSalienceStrategy;
impl NprSalienceStrategy for AllFeatureSalienceStrategy {
    fn select(&self, _: &NprPipelineContext<'_>, features: &[crate::FeatureSegment]) -> Vec<crate::FeatureSegment> {
        features.to_vec()
    }
}

/// A first drawing-oriented filter: preserve silhouettes/boundaries and keep
/// only creases with enough visible paper-space length to read as intentional.
pub struct SelectiveContourSalienceStrategy {
    pub minimum_crease_length_px: f32,
}

impl Default for SelectiveContourSalienceStrategy {
    fn default() -> Self { Self { minimum_crease_length_px: 12.0 } }
}

impl NprSalienceStrategy for SelectiveContourSalienceStrategy {
    fn select(&self, context: &NprPipelineContext<'_>, features: &[crate::FeatureSegment]) -> Vec<crate::FeatureSegment> {
        let viewport = Vec2::new(context.input.viewport[0] as f32, context.input.viewport[1] as f32);
        select_salient_features(
            context.input.geometry,
            context.input.camera,
            viewport,
            features,
            NprSaliencePolicy {
                crease_length_px: self.minimum_crease_length_px,
                ..NprSaliencePolicy::default()
            },
        )
    }
}

pub struct CameraProjectionStrategy;
impl NprProjectionStrategy for CameraProjectionStrategy {
    fn project_point(&self, camera: crate::NprCamera, point: Vec3, viewport: Vec2) -> Option<ProjectedPoint> {
        camera.project(point, viewport)
    }
    fn project_segment(&self, camera: crate::NprCamera, a: Vec3, b: Vec3, viewport: Vec2) -> Option<(ProjectedPoint, ProjectedPoint)> {
        camera.project_segment(a, b, viewport)
    }
}

pub struct ThreeBandValueStrategy;
impl NprValueStrategy for ThreeBandValueStrategy {
    fn plan_fills(&self, context: &NprPipelineContext<'_>, projection: &dyn NprProjectionStrategy) -> Vec<NprFillTriangle> {
        let viewport = Vec2::new(context.input.viewport[0] as f32, context.input.viewport[1] as f32);
        let light_direction = Vec3::new(-0.4, 0.7, 1.0).normalize();
        context.input.geometry.triangles.iter().enumerate().filter_map(|(face_index, triangle)| {
            let geometry = context.input.geometry;
            let (a, b, c) = (
                projection.project_point(context.input.camera, geometry.vertices[triangle[0] as usize].position, viewport)?,
                projection.project_point(context.input.camera, geometry.vertices[triangle[1] as usize].position, viewport)?,
                projection.project_point(context.input.camera, geometry.vertices[triangle[2] as usize].position, viewport)?,
            );
            let shade = face_normal(geometry, face_index as u32).dot(light_direction);
            let color = if shade < -0.1 { context.input.style.shadow } else if shade < 0.45 { context.input.style.mid } else { context.input.style.light };
            Some(NprFillTriangle { positions: [a.screen, b.screen, c.screen], color, depth: (a.depth + b.depth + c.depth) / 3.0 })
        }).collect()
    }
}

/// The animation-line preset starts from untouched paper. Broad graphite tone
/// is a later explicit layer, never an accidental cel-shaded fill.
pub struct NoValueStrategy;
impl NprValueStrategy for NoValueStrategy {
    fn plan_fills(&self, _: &NprPipelineContext<'_>, _: &dyn NprProjectionStrategy) -> Vec<NprFillTriangle> {
        Vec::new()
    }
}

/// Smooths values only across similarly oriented neighbouring faces before
/// quantization. This prevents a low-poly mesh from becoming a field of tiny
/// Lambert bands while retaining hard creases as drawing boundaries.
pub struct GroupedValueStrategy {
    pub iterations: u8,
    pub normal_similarity: f32,
}

impl Default for GroupedValueStrategy {
    fn default() -> Self { Self { iterations: 4, normal_similarity: 0.7 } }
}

impl NprValueStrategy for GroupedValueStrategy {
    fn plan_fills(&self, context: &NprPipelineContext<'_>, projection: &dyn NprProjectionStrategy) -> Vec<NprFillTriangle> {
        let geometry = context.input.geometry;
        let mut value_field = NprValueField::directional(geometry, Vec3::new(-0.4, 0.7, 1.0));
        value_field.group_by_surface(geometry, &context.topology, self.iterations, self.normal_similarity);
        let viewport = Vec2::new(context.input.viewport[0] as f32, context.input.viewport[1] as f32);
        geometry.triangles.iter().enumerate().filter_map(|(face, triangle)| {
            let (a, b, c) = (
                projection.project_point(context.input.camera, geometry.vertices[triangle[0] as usize].position, viewport)?,
                projection.project_point(context.input.camera, geometry.vertices[triangle[1] as usize].position, viewport)?,
                projection.project_point(context.input.camera, geometry.vertices[triangle[2] as usize].position, viewport)?,
            );
            let color = if value_field.values[face] < 0.20 { context.input.style.shadow } else if value_field.values[face] < 0.58 { context.input.style.mid } else { context.input.style.light };
            Some(NprFillTriangle { positions: [a.screen, b.screen, c.screen], color, depth: (a.depth + b.depth + c.depth) / 3.0 })
        }).collect()
    }
}

pub struct FeatureMarkStrategy;
impl NprMarkStrategy for FeatureMarkStrategy {
    fn plan_marks(&self, context: &NprPipelineContext<'_>, projection: &dyn NprProjectionStrategy) -> Vec<NprLogicalMark> {
        let viewport = Vec2::new(context.input.viewport[0] as f32, context.input.viewport[1] as f32);
        context.selected_features.iter().enumerate().filter_map(|(id, feature)| {
            let geometry = context.input.geometry;
            projection.project_segment(
                context.input.camera,
                geometry.vertices[feature.edge.a as usize].position,
                geometry.vertices[feature.edge.b as usize].position,
                viewport,
            ).map(|(a, b)| NprLogicalMark { id: id as u32, class: feature.class, segment: (a.screen, b.screen), depths: (a.depth, b.depth) })
        }).collect()
    }
}

/// Plans value-driven marks directly on projected surface triangles.  The
/// offsets are tied to a face index and seed, so the same surface retains its
/// hatch rhythm while the camera moves.  This is deliberately a mark planner,
/// not a WGPU filter: later surface-direction fields can replace this strategy
/// without changing gesture, medium or packet code.
pub struct NoHatchingStrategy;
impl NprHatchingStrategy for NoHatchingStrategy {
    fn plan_hatching(&self, _: &NprPipelineContext<'_>, _: &dyn NprProjectionStrategy) -> Vec<NprLogicalMark> {
        Vec::new()
    }
}

pub struct TriangleHatchingStrategy {
    pub base_spacing_px: f32,
    pub max_marks_per_face: u8,
}

impl Default for TriangleHatchingStrategy {
    fn default() -> Self { Self { base_spacing_px: 18.0, max_marks_per_face: 5 } }
}

impl NprHatchingStrategy for TriangleHatchingStrategy {
    fn plan_hatching(&self, context: &NprPipelineContext<'_>, projection: &dyn NprProjectionStrategy) -> Vec<NprLogicalMark> {
        let mut marks = Vec::new();
        let geometry = context.input.geometry;
        let viewport = Vec2::new(context.input.viewport[0] as f32, context.input.viewport[1] as f32);
        let mut values = NprValueField::directional(geometry, Vec3::new(-0.4, 0.7, 1.0));
        values.group_by_surface(geometry, &context.topology, 4, 0.7);
        let field = NprSurfaceDirectionField::from_geometry(geometry);
        for (face, triangle) in geometry.triangles.iter().enumerate() {
            let darkness = 1.0 - values.values[face];
            if darkness < 0.24 { continue; }
            let Some(a) = projection.project_point(context.input.camera, geometry.vertices[triangle[0] as usize].position, viewport) else { continue };
            let Some(b) = projection.project_point(context.input.camera, geometry.vertices[triangle[1] as usize].position, viewport) else { continue };
            let Some(c) = projection.project_point(context.input.camera, geometry.vertices[triangle[2] as usize].position, viewport) else { continue };
            let centroid = (geometry.vertices[triangle[0] as usize].position
                + geometry.vertices[triangle[1] as usize].position
                + geometry.vertices[triangle[2] as usize].position) / 3.0;
            let Some(surface_direction) = field.direction(face) else { continue };
            let Some(projected_centroid) = projection.project_point(context.input.camera, centroid, viewport) else { continue };
            let Some(projected_tangent) = projection.project_point(
                context.input.camera,
                centroid + surface_direction.tangent * 0.1,
                viewport,
            ) else { continue };
            let direction = (projected_tangent.screen - projected_centroid.screen).normalize_or_zero();
            if direction.length_squared() < 0.001 { continue; }
            let normal = Vec2::new(-direction.y, direction.x);
            let points = [(a.screen, a.depth), (b.screen, b.depth), (c.screen, c.depth)];
            let min_offset = points.iter().map(|(point, _)| normal.dot(*point)).fold(f32::INFINITY, f32::min);
            let max_offset = points.iter().map(|(point, _)| normal.dot(*point)).fold(f32::NEG_INFINITY, f32::max);
            let spacing = self.base_spacing_px / (0.45 + darkness * 0.85);
            let seed = context.input.seed ^ (face as u64).wrapping_mul(0x9E37_79B9);
            let jitter = ((seed ^ (seed >> 17)).wrapping_mul(0x85EB_CA6B) >> 32) as f32 / u32::MAX as f32 - 0.5;
            let first = min_offset + spacing * (0.5 + jitter * 0.22);
            let planned = (((max_offset - first) / spacing).floor() as i32 + 1)
                .clamp(0, self.max_marks_per_face as i32);
            for lane in 0..planned {
                let offset = first + lane as f32 * spacing;
                let mut intersections = Vec::new();
                for ((p0, d0), (p1, d1)) in [(points[0], points[1]), (points[1], points[2]), (points[2], points[0])] {
                    let start = normal.dot(p0) - offset;
                    let end = normal.dot(p1) - offset;
                    let denominator = start - end;
                    if denominator.abs() < 0.0001 || start.signum() == end.signum() { continue; }
                    let t = (start / denominator).clamp(0.0, 1.0);
                    let point = p0.lerp(p1, t);
                    if intersections.iter().all(|(other, _): &(Vec2, f32)| other.distance(point) > 0.01) {
                        intersections.push((point, d0 + (d1 - d0) * t));
                    }
                }
                if intersections.len() == 2 {
                    let id = 0x8000_0000u32.wrapping_add((face as u32).wrapping_mul(16)).wrapping_add(lane as u32);
                    marks.push(NprLogicalMark {
                        id,
                        class: FeatureClass::Hatching,
                        segment: (intersections[0].0, intersections[1].0),
                        depths: (intersections[0].1, intersections[1].1),
                    });
                }
            }
        }
        marks
    }
}

pub struct FlatInkGestureStrategy;
impl NprGestureStrategy for FlatInkGestureStrategy {
    fn realize(&self, context: &NprPipelineContext<'_>, marks: &[NprLogicalMark]) -> Vec<TessellatedStroke> {
        marks.iter().map(|mark| tessellate_segment_with_depth(mark.id, mark.class, mark.segment, mark.depths, context.input.style, context.input.seed)).collect()
    }
}

/// Produces a stable, low-frequency offset for a whole mark. Unlike white noise
/// per vertex, both ends share a hand-like drift and stay deterministic for the
/// same scene/object/stroke seed.
pub struct GraphiteGestureStrategy {
    pub drift_pixels: f32,
    pub overshoot_pixels: f32,
    brushes: Arc<dyn NprBrushLibrary>,
}

impl Default for GraphiteGestureStrategy {
    fn default() -> Self { Self { drift_pixels: 0.82, overshoot_pixels: 1.35, brushes: Arc::new(PencilBrushLibrary) } }
}

impl NprGestureStrategy for GraphiteGestureStrategy {
    fn realize(&self, context: &NprPipelineContext<'_>, marks: &[NprLogicalMark]) -> Vec<TessellatedStroke> {
        marks.iter().map(|mark| {
            let direction = (mark.segment.1 - mark.segment.0).normalize_or_zero();
            let normal = Vec2::new(-direction.y, direction.x);
            let seed = context.input.seed
                ^ context.input.temporal.path_epoch.wrapping_mul(0xA24B_AED4)
                ^ (mark.id as u64).wrapping_mul(0x9E37_79B9);
            let mut style = context.input.style;
            let brush = self.brushes.brush_for(mark.class);
            match mark.class {
                FeatureClass::Boundary | FeatureClass::Silhouette => style.boundary_width *= brush.width_scale,
                FeatureClass::Crease | FeatureClass::Hatching => style.crease_width *= brush.width_scale,
            }
            style.taper = style.taper.max(brush.taper);
            let length = mark.segment.0.distance(mark.segment.1);
            let knots = (length / 28.0).ceil().clamp(3.0, 7.0) as usize;
            let mut points = Vec::with_capacity(knots);
            // A low-frequency, deterministic hand path. Consecutive points
            // share a slowly changing drift rather than independent white
            // noise, so a line looks hand-drawn but stays surface-stable.
            let mut carried_drift = 0.0;
            for knot in 0..knots {
                let t = knot as f32 / (knots - 1) as f32;
                let hash = (seed ^ (knot as u64).wrapping_mul(0xD1B5_4A32))
                    .wrapping_mul(0x94D0_49BB);
                let random = ((hash ^ (hash >> 29)) >> 32) as f32 / u32::MAX as f32 - 0.5;
                carried_drift = carried_drift * 0.58 + random * self.drift_pixels;
                let envelope = (std::f32::consts::PI * t).sin().max(0.0).powf(0.45);
                let correction = direction * random * self.drift_pixels * 0.18;
                let overshoot = if knot == 0 { -self.overshoot_pixels } else if knot + 1 == knots { self.overshoot_pixels } else { 0.0 };
                let position = mark.segment.0.lerp(mark.segment.1, t)
                    + direction * overshoot
                    + normal * carried_drift * envelope
                    + correction * envelope;
                points.push((position, mark.depths.0 + (mark.depths.1 - mark.depths.0) * t));
            }
            let mut stroke = tessellate_polyline_with_depth(mark.id, mark.class, &points, style);
            stroke.medium = Some(brush.medium);
            stroke
        }).collect()
    }
}

pub struct FlatPaperStrategy;
impl NprPaperStrategy for FlatPaperStrategy {
    fn paper(&self, context: &NprPipelineContext<'_>) -> NprPaperDefinition {
        NprPaperDefinition { color: context.input.style.paper, ..Default::default() }
    }
}

pub struct InkMediumStrategy;
impl NprMediumStrategy for InkMediumStrategy {
    fn medium(&self, _: &NprPipelineContext<'_>) -> NprMediumDefinition { NprMediumDefinition::Ink(InkMedium::default()) }
}

pub struct GraphiteMediumStrategy;
impl NprMediumStrategy for GraphiteMediumStrategy {
    fn medium(&self, _: &NprPipelineContext<'_>) -> NprMediumDefinition { NprMediumDefinition::Graphite(GraphiteMedium::default()) }
}

pub struct GraphitePaperStrategy;
impl NprPaperStrategy for GraphitePaperStrategy {
    fn paper(&self, context: &NprPipelineContext<'_>) -> NprPaperDefinition {
        let paper = context.input.style.paper;
        NprPaperDefinition {
            color: Vec4::new(paper.x * 1.03, paper.y * 1.02, paper.z * 0.98, paper.w),
            tooth_scale: 0.78,
            fibre_strength: 0.58,
            absorption: 0.70,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComicInk, NprCamera, NprDebugView, NprGeometry, OrthographicCamera};

    struct BlackPaper;
    impl NprPaperStrategy for BlackPaper {
        fn paper(&self, _: &NprPipelineContext<'_>) -> NprPaperDefinition {
            NprPaperDefinition { color: Vec4::ZERO, ..Default::default() }
        }
    }

    #[test]
    fn paper_strategy_can_be_replaced_without_rewriting_the_pipeline() {
        let pipeline = MinimalInkPipeline::with_strategies(
            Arc::new(TopologySurfaceStrategy),
            Arc::new(ContourFeatureStrategy),
            Arc::new(AllFeatureSalienceStrategy),
            Arc::new(CameraProjectionStrategy),
            Arc::new(ThreeBandValueStrategy),
            Arc::new(FeatureMarkStrategy),
            Arc::new(NoHatchingStrategy),
            Arc::new(FlatInkGestureStrategy),
            Arc::new(BlackPaper),
            Arc::new(InkMediumStrategy),
        );
        let packet = pipeline.build(NprPipelineInput {
            geometry: &NprGeometry::canonical_cube(),
            camera: NprCamera::Orthographic(OrthographicCamera::default_for_viewport(1.0)),
            viewport: [256, 256],
            style: ComicInk::default(),
            seed: 4,
            debug_view: NprDebugView::Final,
            temporal: crate::NprTemporalState::default(),
        });
        assert_eq!(packet.background, Vec4::ZERO);
        assert!(!packet.strokes.is_empty());
    }

    #[test]
    fn profiles_expose_the_same_composable_stage_contract() {
        assert_eq!(MinimalInkPipeline::default().stages(), PencilAnimationPipeline::default().stages());
        assert_eq!(MinimalInkPipeline::default().stages(), DEFAULT_NPR_PIPELINE_STAGES);
    }
}
