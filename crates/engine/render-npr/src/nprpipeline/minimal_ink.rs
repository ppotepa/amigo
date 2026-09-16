use super::{
    contracts::{NprLogicalMark, NprPipelineContext, NprPipelineInput},
    pipeline::NprPipeline,
    strategies::{
        NprFeatureStrategy, NprGestureStrategy, NprHatchingStrategy, NprMarkStrategy, NprMediumStrategy, NprPaperStrategy,
        NprProjectionStrategy, NprSurfaceStrategy, NprValueStrategy,
        NprSalienceStrategy, NprStrokeChainStrategy,
    },
};
use crate::{
    FeatureClass, GraphiteMedium, InkMedium, NprBrushLibrary, NprFillTriangle, NprMediumDefinition, NprPaperDefinition, NprRenderPacket, NprRenderStats, NprSurfaceDirectionField, PencilBrushLibrary, ProjectedPoint,
    TessellatedStroke, TopologyEdge, build_topology, classify_features, face_normal,
    tessellate_polyline_with_depth, DEFAULT_NPR_PIPELINE_STAGES, NprSaliencePolicy,
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
    chains: Arc<dyn NprStrokeChainStrategy>,
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
            chains: Arc::new(NoStrokeChainStrategy),
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
        chains: Arc<dyn NprStrokeChainStrategy>,
        gesture: Arc<dyn NprGestureStrategy>,
        paper: Arc<dyn NprPaperStrategy>,
        medium: Arc<dyn NprMediumStrategy>,
    ) -> Self {
        Self { surface, features, salience, projection, values, marks, hatching, chains, gesture, paper, medium }
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
        let marks = std::mem::take(&mut context.marks);
        context.marks = self.chains.chain(&context, marks);
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
                Arc::new(CompositeFeatureStrategy::new(vec![
                    Arc::new(PencilContourFeatureStrategy::default()),
                    Arc::new(SuggestiveContourFeatureStrategy::default()),
                ])),
                Arc::new(SelectiveContourSalienceStrategy { minimum_crease_length_px: 28.0 }),
                Arc::new(CameraProjectionStrategy),
                Arc::new(NoValueStrategy),
                Arc::new(FeatureMarkStrategy),
                Arc::new(NoHatchingStrategy),
                Arc::new(ScreenSpaceStrokeChainStrategy::default()),
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

/// Pencil animation starts from the drawn silhouette, not from the source
/// mesh's triangulation. It may add only genuinely structural creases; smooth
/// triangulation must never become an interior drawing grid.
pub struct PencilContourFeatureStrategy {
    pub structural_crease_angle: f32,
}

impl Default for PencilContourFeatureStrategy {
    fn default() -> Self { Self { structural_crease_angle: 1.10 } }
}

impl NprFeatureStrategy for PencilContourFeatureStrategy {
    fn detect(&self, context: &NprPipelineContext<'_>, topology: &[TopologyEdge]) -> Vec<crate::FeatureSegment> {
        let geometry = context.input.geometry;
        let view = -context.input.camera.forward();
        let mut features = classify_features(geometry, topology, view, 0.35)
            .into_iter()
            .filter(|feature| matches!(feature.class, FeatureClass::Silhouette | FeatureClass::Boundary))
            .collect::<Vec<_>>();
        features.extend(topology.iter().filter_map(|edge| {
            if edge.faces[1] == u32::MAX {
                return None;
            }
            let first = face_normal(geometry, edge.faces[0]);
            let second = face_normal(geometry, edge.faces[1]);
            // Silhouettes are already emitted above. Interior contours only
            // describe a deliberate hard structural change.
            if (first.dot(view) >= 0.0) != (second.dot(view) >= 0.0)
                || first.dot(second) >= self.structural_crease_angle.cos()
            {
                return None;
            }
            Some(crate::FeatureSegment {
                edge: *edge,
                class: FeatureClass::Crease,
                midpoint: (geometry.vertices[edge.a as usize].position
                    + geometry.vertices[edge.b as usize].position) * 0.5,
            })
        }));
        features
    }
}

/// Combines independent sources of drawing evidence. A preset can therefore
/// use topology for silhouettes, a curvature/view strategy for form lines,
/// and later authored semantic seams without creating a parallel pipeline.
pub struct CompositeFeatureStrategy {
    strategies: Vec<Arc<dyn NprFeatureStrategy>>,
}

impl CompositeFeatureStrategy {
    pub fn new(strategies: Vec<Arc<dyn NprFeatureStrategy>>) -> Self { Self { strategies } }
}

impl NprFeatureStrategy for CompositeFeatureStrategy {
    fn detect(&self, context: &NprPipelineContext<'_>, topology: &[TopologyEdge]) -> Vec<crate::FeatureSegment> {
        self.strategies
            .iter()
            .flat_map(|strategy| strategy.detect(context, topology))
            .collect()
    }
}

/// Conservative screen-relevant form-line candidates on smooth geometry.
/// Both adjacent faces must be visible, turn gradually (not a structural
/// crease), and sit close to the view-tangent band. This gives sparse hints
/// of turning form without promoting every triangle edge to a stroke.
pub struct SuggestiveContourFeatureStrategy {
    pub min_dihedral: f32,
    pub max_dihedral: f32,
    pub tangent_band: f32,
    pub min_view_delta: f32,
}

impl Default for SuggestiveContourFeatureStrategy {
    fn default() -> Self {
        Self {
            min_dihedral: 0.06,
            max_dihedral: 0.42,
            tangent_band: 0.24,
            min_view_delta: 0.035,
        }
    }
}

impl NprFeatureStrategy for SuggestiveContourFeatureStrategy {
    fn detect(&self, context: &NprPipelineContext<'_>, topology: &[TopologyEdge]) -> Vec<crate::FeatureSegment> {
        let geometry = context.input.geometry;
        let view = -context.input.camera.forward();
        topology.iter().filter_map(|edge| {
            if edge.faces[1] == u32::MAX {
                return None;
            }
            let first = face_normal(geometry, edge.faces[0]);
            let second = face_normal(geometry, edge.faces[1]);
            let first_view = first.dot(view);
            let second_view = second.dot(view);
            let dihedral = first.dot(second).clamp(-1.0, 1.0).acos();
            if first_view <= 0.0
                || second_view <= 0.0
                || dihedral < self.min_dihedral
                || dihedral > self.max_dihedral
                || first_view.min(second_view) > self.tangent_band
                || (first_view - second_view).abs() < self.min_view_delta
            {
                return None;
            }
            Some(crate::FeatureSegment {
                edge: *edge,
                class: FeatureClass::SuggestiveContour,
                midpoint: (geometry.vertices[edge.a as usize].position
                    + geometry.vertices[edge.b as usize].position) * 0.5,
            })
        }).collect()
    }
}

pub struct AllFeatureSalienceStrategy;
impl NprSalienceStrategy for AllFeatureSalienceStrategy {
    fn select(&self, _: &NprPipelineContext<'_>, features: &[crate::FeatureSegment]) -> Vec<crate::FeatureSegment> {
        features.to_vec()
    }
}

/// A drawing-oriented filter: preserve silhouettes/boundaries and keep only
/// interior contours with enough visible paper-space length to read as an
/// intentional gesture rather than mesh noise.
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
            ).map(|(a, b)| NprLogicalMark {
                id: id as u32,
                class: feature.class,
                points: vec![(a.screen, a.depth), (b.screen, b.depth)],
            })
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
                        points: intersections,
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
        marks.iter()
            .filter(|mark| mark.points.len() >= 2)
            .map(|mark| tessellate_polyline_with_depth(mark.id, mark.class, &mark.points, context.input.style))
            .collect()
    }
}

/// Keeps independent marks unchanged. Useful for technical ink and for tests
/// that need the raw projected candidates.
pub struct NoStrokeChainStrategy;
impl NprStrokeChainStrategy for NoStrokeChainStrategy {
    fn chain(&self, _: &NprPipelineContext<'_>, marks: Vec<NprLogicalMark>) -> Vec<NprLogicalMark> { marks }
}

/// Chains projected contour segments that share a paper-space endpoint. This
/// intentionally works after projection: it respects clipping and visibility,
/// and does not need renderer-specific access to the source mesh.
pub struct ScreenSpaceStrokeChainStrategy {
    pub endpoint_tolerance_px: f32,
}

impl Default for ScreenSpaceStrokeChainStrategy {
    fn default() -> Self { Self { endpoint_tolerance_px: 0.75 } }
}

impl NprStrokeChainStrategy for ScreenSpaceStrokeChainStrategy {
    fn chain(&self, _: &NprPipelineContext<'_>, mut pending: Vec<NprLogicalMark>) -> Vec<NprLogicalMark> {
        let mut chained = Vec::new();
        while let Some(mut path) = pending.pop() {
            if path.class == FeatureClass::Hatching || path.points.len() < 2 {
                chained.push(path);
                continue;
            }
            loop {
                let Some(last) = path.points.last().copied() else { break };
                let Some(first) = path.points.first().copied() else { break };
                let next = pending.iter().position(|candidate| {
                    candidate.class == path.class
                        && candidate.points.len() >= 2
                        && [candidate.points[0], *candidate.points.last().expect("two points")]
                            .iter()
                            .any(|point| point.0.distance(last.0) <= self.endpoint_tolerance_px
                                || point.0.distance(first.0) <= self.endpoint_tolerance_px)
                });
                let Some(next) = next else { break };
                let mut candidate = pending.swap_remove(next);
                let candidate_first = candidate.points[0];
                let candidate_last = *candidate.points.last().expect("two points");
                if candidate_first.0.distance(last.0) <= self.endpoint_tolerance_px {
                    path.points.extend(candidate.points.drain(1..));
                } else if candidate_last.0.distance(last.0) <= self.endpoint_tolerance_px {
                    candidate.points.reverse();
                    path.points.extend(candidate.points.drain(1..));
                } else if candidate_last.0.distance(first.0) <= self.endpoint_tolerance_px {
                    candidate.points.pop();
                    candidate.points.extend(path.points);
                    path.points = candidate.points;
                } else {
                    candidate.points.reverse();
                    candidate.points.pop();
                    candidate.points.extend(path.points);
                    path.points = candidate.points;
                }
            }
            chained.push(path);
        }
        chained
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
        marks.iter().filter(|mark| mark.points.len() >= 2).map(|mark| {
            let seed = context.input.seed
                ^ context.input.temporal.path_epoch.wrapping_mul(0xA24B_AED4)
                ^ (mark.id as u64).wrapping_mul(0x9E37_79B9);
            let mut style = context.input.style;
            let brush = self.brushes.brush_for(mark.class);
            match mark.class {
                FeatureClass::Boundary | FeatureClass::Silhouette => style.boundary_width *= brush.width_scale,
                FeatureClass::Crease | FeatureClass::SuggestiveContour | FeatureClass::Hatching => style.crease_width *= brush.width_scale,
            }
            style.taper = style.taper.max(brush.taper);
            let source = resample_mark(mark, 28.0);
            let mut points = Vec::with_capacity(source.len());
            // A low-frequency, deterministic hand path. Consecutive points
            // share a slowly changing drift rather than independent white
            // noise, so a line looks hand-drawn but stays surface-stable.
            let mut carried_drift = 0.0;
            for knot in 0..source.len() {
                let t = knot as f32 / (source.len() - 1) as f32;
                let direction = if knot + 1 < source.len() {
                    (source[knot + 1].0 - source[knot].0).normalize_or_zero()
                } else {
                    (source[knot].0 - source[knot - 1].0).normalize_or_zero()
                };
                let normal = Vec2::new(-direction.y, direction.x);
                let hash = (seed ^ (knot as u64).wrapping_mul(0xD1B5_4A32))
                    .wrapping_mul(0x94D0_49BB);
                let random = ((hash ^ (hash >> 29)) >> 32) as f32 / u32::MAX as f32 - 0.5;
                carried_drift = carried_drift * 0.58 + random * self.drift_pixels;
                let envelope = (std::f32::consts::PI * t).sin().max(0.0).powf(0.45);
                let correction = direction * random * self.drift_pixels * 0.18;
                let overshoot = if knot == 0 { -self.overshoot_pixels } else if knot + 1 == source.len() { self.overshoot_pixels } else { 0.0 };
                let position = source[knot].0
                    + direction * overshoot
                    + normal * carried_drift * envelope
                    + correction * envelope;
                points.push((position, source[knot].1));
            }
            let mut stroke = tessellate_polyline_with_depth(mark.id, mark.class, &points, style);
            stroke.medium = Some(brush.medium);
            stroke
        }).collect()
    }
}

fn resample_mark(mark: &NprLogicalMark, spacing_px: f32) -> Vec<(Vec2, f32)> {
    let mut samples = vec![mark.points[0]];
    for pair in mark.points.windows(2) {
        let length = pair[0].0.distance(pair[1].0);
        let segments = (length / spacing_px).ceil().max(1.0) as usize;
        for step in 1..=segments {
            let t = step as f32 / segments as f32;
            samples.push((pair[0].0.lerp(pair[1].0, t), pair[0].1 + (pair[1].1 - pair[0].1) * t));
        }
    }
    samples
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
            Arc::new(NoStrokeChainStrategy),
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

    #[test]
    fn pencil_animation_emits_contours_without_hatching_or_fills() {
        let packet = PencilAnimationPipeline::default().build(NprPipelineInput {
            geometry: &NprGeometry::canonical_cube(),
            camera: NprCamera::Perspective(crate::PerspectiveCamera::cube_default(1.0)),
            viewport: [512, 512],
            style: ComicInk::default(),
            seed: 7,
            debug_view: NprDebugView::Final,
            temporal: crate::NprTemporalState::default(),
        });

        assert!(!packet.strokes.is_empty());
        assert!(packet.strokes.iter().all(|stroke| {
            matches!(stroke.class, FeatureClass::Silhouette | FeatureClass::Boundary | FeatureClass::Crease)
        }));
        assert!(!packet.strokes.iter().any(|stroke| stroke.class == FeatureClass::Hatching));
        assert!(packet.fills.is_empty());
    }

    #[test]
    fn pencil_contours_reject_low_dihedral_triangulation() {
        let geometry = NprGeometry::from_indexed(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.08],
            ],
            &[0, 1, 2, 1, 3, 2],
        ).unwrap();
        let input = NprPipelineInput {
            geometry: &geometry,
            camera: NprCamera::Perspective(crate::PerspectiveCamera::cube_default(1.0)),
            viewport: [512, 512],
            style: ComicInk::default(),
            seed: 7,
            debug_view: NprDebugView::Final,
            temporal: crate::NprTemporalState::default(),
        };
        let context = NprPipelineContext::new(input);
        let features = PencilContourFeatureStrategy::default().detect(&context, &build_topology(&geometry));

        assert!(!features.iter().any(|feature| feature.class == FeatureClass::Crease));
    }

    #[test]
    fn suggestive_contours_select_a_smooth_view_tangent_turn() {
        let geometry = NprGeometry::from_indexed(
            &[
                [0.0, -1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-0.20, 0.0, 0.98],
                [0.10, 0.0, -0.995],
            ],
            &[0, 1, 2, 1, 0, 3],
        ).unwrap();
        let input = NprPipelineInput {
            geometry: &geometry,
            camera: NprCamera::Perspective(crate::PerspectiveCamera {
                position: Vec3::new(0.0, 0.0, 5.0),
                forward: -Vec3::Z,
                up: Vec3::Y,
                vertical_fov: 45.0_f32.to_radians(),
                near: 0.05,
                aspect: 1.0,
            }),
            viewport: [512, 512],
            style: ComicInk::default(),
            seed: 7,
            debug_view: NprDebugView::Final,
            temporal: crate::NprTemporalState::default(),
        };
        let context = NprPipelineContext::new(input);
        let features = SuggestiveContourFeatureStrategy::default()
            .detect(&context, &build_topology(&geometry));

        assert_eq!(features.len(), 1);
        assert_eq!(features[0].class, FeatureClass::SuggestiveContour);
    }

    #[test]
    fn screen_space_chain_turns_adjacent_contours_into_one_path() {
        let input = NprPipelineInput {
            geometry: &NprGeometry::canonical_cube(),
            camera: NprCamera::Perspective(crate::PerspectiveCamera::cube_default(1.0)),
            viewport: [512, 512],
            style: ComicInk::default(),
            seed: 7,
            debug_view: NprDebugView::Final,
            temporal: crate::NprTemporalState::default(),
        };
        let context = NprPipelineContext::new(input);
        let marks = vec![
            NprLogicalMark {
                id: 1,
                class: FeatureClass::Silhouette,
                points: vec![(Vec2::new(10.0, 10.0), 1.0), (Vec2::new(20.0, 10.0), 1.0)],
            },
            NprLogicalMark {
                id: 2,
                class: FeatureClass::Silhouette,
                points: vec![(Vec2::new(20.4, 10.1), 1.0), (Vec2::new(30.0, 15.0), 1.0)],
            },
        ];

        let chained = ScreenSpaceStrokeChainStrategy::default().chain(&context, marks);
        assert_eq!(chained.len(), 1);
        assert_eq!(chained[0].points.len(), 3);
    }
}
