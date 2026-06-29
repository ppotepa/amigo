use std::collections::BTreeMap;

use amigo_math::{ColorRgba, Vec2};

use super::NprResolvedKindStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NprLineKind {
    Boundary,
    Silhouette,
    Crease,
    Seam,
    Feature,
    Contact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NprDebugOverlay3d {
    LineKinds,
    RawPaths,
    Dropout,
    WidthAlpha,
}

impl NprDebugOverlay3d {
    pub(crate) fn from_camera_debug_view(
        view: &amigo_render_api::CameraDebugView2d,
    ) -> Option<Self> {
        match view.as_str() {
            "npr.line_kinds" | "npr.kinds" => Some(Self::LineKinds),
            "npr.raw_paths" | "npr.paths" => Some(Self::RawPaths),
            "npr.dropout" | "npr.breakup" => Some(Self::Dropout),
            "npr.width_alpha" | "npr.pressure" => Some(Self::WidthAlpha),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprLineFragment {
    pub(crate) source_edge_id: u64,
    pub(crate) kind: NprLineKind,
    pub(crate) p0: Vec2,
    pub(crate) p1: Vec2,
    pub(crate) t0: f32,
    pub(crate) t1: f32,
    pub(crate) tangent0: Vec2,
    pub(crate) tangent1: Vec2,
    pub(crate) avg_depth: f32,
}

pub(crate) struct NprEdgeSampleResult3d {
    pub(crate) fragments: Vec<NprLineFragment>,
    pub(crate) visible_edges: usize,
}

pub(crate) struct NprPathBuildResult3d {
    pub(crate) paths: Vec<NprStrokePath>,
    pub(crate) stats: NprPathBuildStats3d,
}

#[derive(Debug, Clone)]
pub(crate) struct NprStrokePath {
    pub(crate) path_id: u64,
    pub(crate) kind: NprLineKind,
    pub(crate) points: Vec<Vec2>,
    pub(crate) source_edges: Vec<u64>,
    pub(crate) sorted_source_edges: Vec<u64>,
    pub(crate) arc_lengths_px: Vec<f32>,
    pub(crate) importance: f32,
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprBrushSample {
    pub(crate) point: Vec2,
    pub(crate) arc_length_px: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct NprStableBrushPath {
    pub(crate) path_id: u64,
    pub(crate) samples: Vec<NprBrushSample>,
    pub(crate) length_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NprStrokePassKind {
    Primary,
    Search,
    Hatch,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprToolDynamics {
    pub(crate) base_width_px: f32,
    pub(crate) base_wobble_px: f32,
    pub(crate) effective_overshoot_px: f32,
    pub(crate) edge_complexity: f32,
    pub(crate) protected_silhouette: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprStrokeGesture {
    pub(crate) path_seed: u64,
    pub(crate) path_length_px: f32,
    pub(crate) importance: f32,
    pub(crate) dynamics: NprToolDynamics,
    pub(crate) style: NprResolvedKindStyle,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprStrokePassPlan {
    pub(crate) kind: NprStrokePassKind,
    pub(crate) pass_index: u8,
    pub(crate) active_t0: f32,
    pub(crate) active_t1: f32,
    pub(crate) wobble_px: f32,
    pub(crate) width_multiplier: f32,
    pub(crate) color: ColorRgba,
    pub(crate) overshoot_px: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprDropoutInterval {
    pub(crate) pass_index: u8,
    pub(crate) t0: f32,
    pub(crate) t1: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct NprDropoutMask {
    pub(crate) intervals: Vec<NprDropoutInterval>,
}

impl NprDropoutMask {
    pub(crate) fn keeps_segment(
        &self,
        pass: NprStrokePassPlan,
        segment_t0: f32,
        segment_t1: f32,
        segment_length_px: f32,
    ) -> bool {
        if pass.kind == NprStrokePassKind::Search
            || pass.kind == NprStrokePassKind::Hatch
            || segment_length_px <= f32::EPSILON
        {
            return true;
        }
        !self.intervals.iter().any(|interval| {
            interval.pass_index == pass.pass_index
                && segment_t1 >= interval.t0
                && segment_t0 <= interval.t1
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NprCachedStrokePlan {
    pub(crate) settings_signature: u64,
    pub(crate) length_bucket_px: u32,
    pub(crate) passes: Vec<NprStrokePassPlan>,
    pub(crate) dropout: NprDropoutMask,
}

impl NprCachedStrokePlan {
    pub(crate) fn is_compatible(
        &self,
        settings: &amigo_render_api::NprLineSettings3d,
        gesture: NprStrokeGesture,
    ) -> bool {
        self.settings_signature == npr_stroke_plan_settings_signature(settings)
            && self.length_bucket_px == npr_stroke_plan_length_bucket(gesture.path_length_px)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprStrokeStripSample {
    pub(crate) point: Vec2,
    pub(crate) width_px: f32,
    pub(crate) offset_px: f32,
    pub(crate) overshoot_px: f32,
    pub(crate) color: ColorRgba,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NprStrokeRail {
    pub(crate) left: Vec2,
    pub(crate) right: Vec2,
    pub(crate) color: ColorRgba,
}

#[derive(Debug, Clone)]
pub(crate) struct NprTemporalPathState3d {
    pub(crate) path: NprStrokePath,
    pub(crate) cached_plan: Option<NprCachedStrokePlan>,
    pub(crate) missing_frames: u8,
    pub(crate) last_seen_frame: u64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NprEntityPathHistory3d {
    pub(crate) paths: BTreeMap<u64, NprTemporalPathState3d>,
}

#[derive(Debug, Clone, Default)]
pub struct NprStrokeFrameStats3d {
    pub meshes: usize,
    pub gpu_realtime_meshes: usize,
    pub cpu_reference_meshes: usize,
    pub gpu_realtime_enqueued_edges: usize,
    pub gpu_realtime_enqueued_triangles: usize,
    pub gpu_realtime_topology_uploads: usize,
    pub gpu_realtime_buffer_capacity_bytes: u64,
    pub gpu_realtime_frame_jobs: usize,
    pub gpu_realtime_projected_vertices_capacity: usize,
    pub gpu_realtime_visible_segments_capacity: usize,
    pub gpu_realtime_endpoint_heads_capacity: usize,
    pub gpu_realtime_endpoint_entries_capacity: usize,
    pub gpu_realtime_path_links_capacity: usize,
    pub gpu_realtime_path_states_capacity: usize,
    pub gpu_realtime_path_segments_capacity: usize,
    pub gpu_realtime_stroke_segments_capacity: usize,
    pub gpu_realtime_debug_mode: String,
    pub paths: usize,
    pub boundary_paths: usize,
    pub silhouette_paths: usize,
    pub crease_paths: usize,
    pub seam_paths: usize,
    pub feature_paths: usize,
    pub contact_paths: usize,
    pub brush_samples: usize,
    pub strip_vertices: usize,
    pub primary_passes: usize,
    pub search_passes: usize,
    pub dropout_intervals: usize,
    pub cached_plan_hits: usize,
    pub cached_plan_misses: usize,
    pub path_build_us: f64,
    pub stabilize_us: f64,
    pub stroke_vertices_us: f64,
    pub path_project_us: f64,
    pub path_visibility_us: f64,
    pub path_edge_sample_us: f64,
    pub path_stitch_us: f64,
    pub path_visible_edges: usize,
    pub path_fragments: usize,
}

impl NprStrokeFrameStats3d {
    pub(crate) fn record_strategy(&mut self, strategy: amigo_render_api::NprRenderStrategy3d) {
        match strategy {
            amigo_render_api::NprRenderStrategy3d::GpuRealtime => self.gpu_realtime_meshes += 1,
            amigo_render_api::NprRenderStrategy3d::CpuReference => self.cpu_reference_meshes += 1,
        }
    }

    pub(crate) fn record_path_kind(&mut self, kind: NprLineKind) {
        match kind {
            NprLineKind::Boundary => self.boundary_paths += 1,
            NprLineKind::Silhouette => self.silhouette_paths += 1,
            NprLineKind::Crease => self.crease_paths += 1,
            NprLineKind::Seam => self.seam_paths += 1,
            NprLineKind::Feature => self.feature_paths += 1,
            NprLineKind::Contact => self.contact_paths += 1,
        }
    }

    pub(crate) fn record_pass(&mut self, pass: NprStrokePassPlan) {
        match pass.kind {
            NprStrokePassKind::Primary => self.primary_passes += 1,
            NprStrokePassKind::Search => self.search_passes += 1,
            NprStrokePassKind::Hatch => {}
        }
    }

    pub(crate) fn add(&mut self, other: Self) {
        self.meshes += other.meshes;
        self.gpu_realtime_meshes += other.gpu_realtime_meshes;
        self.cpu_reference_meshes += other.cpu_reference_meshes;
        self.gpu_realtime_enqueued_edges += other.gpu_realtime_enqueued_edges;
        self.gpu_realtime_enqueued_triangles += other.gpu_realtime_enqueued_triangles;
        self.gpu_realtime_topology_uploads += other.gpu_realtime_topology_uploads;
        self.gpu_realtime_buffer_capacity_bytes += other.gpu_realtime_buffer_capacity_bytes;
        self.gpu_realtime_frame_jobs += other.gpu_realtime_frame_jobs;
        self.gpu_realtime_projected_vertices_capacity +=
            other.gpu_realtime_projected_vertices_capacity;
        self.gpu_realtime_visible_segments_capacity += other.gpu_realtime_visible_segments_capacity;
        self.gpu_realtime_endpoint_heads_capacity += other.gpu_realtime_endpoint_heads_capacity;
        self.gpu_realtime_endpoint_entries_capacity += other.gpu_realtime_endpoint_entries_capacity;
        self.gpu_realtime_path_links_capacity += other.gpu_realtime_path_links_capacity;
        self.gpu_realtime_path_states_capacity += other.gpu_realtime_path_states_capacity;
        self.gpu_realtime_path_segments_capacity += other.gpu_realtime_path_segments_capacity;
        self.gpu_realtime_stroke_segments_capacity += other.gpu_realtime_stroke_segments_capacity;
        if self.gpu_realtime_debug_mode.is_empty() {
            self.gpu_realtime_debug_mode = other.gpu_realtime_debug_mode.clone();
        } else if !other.gpu_realtime_debug_mode.is_empty()
            && self.gpu_realtime_debug_mode != other.gpu_realtime_debug_mode
        {
            self.gpu_realtime_debug_mode = "mixed".to_owned();
        }
        self.paths += other.paths;
        self.boundary_paths += other.boundary_paths;
        self.silhouette_paths += other.silhouette_paths;
        self.crease_paths += other.crease_paths;
        self.seam_paths += other.seam_paths;
        self.feature_paths += other.feature_paths;
        self.contact_paths += other.contact_paths;
        self.brush_samples += other.brush_samples;
        self.strip_vertices += other.strip_vertices;
        self.primary_passes += other.primary_passes;
        self.search_passes += other.search_passes;
        self.dropout_intervals += other.dropout_intervals;
        self.cached_plan_hits += other.cached_plan_hits;
        self.cached_plan_misses += other.cached_plan_misses;
        self.path_build_us += other.path_build_us;
        self.stabilize_us += other.stabilize_us;
        self.stroke_vertices_us += other.stroke_vertices_us;
        self.path_project_us += other.path_project_us;
        self.path_visibility_us += other.path_visibility_us;
        self.path_edge_sample_us += other.path_edge_sample_us;
        self.path_stitch_us += other.path_stitch_us;
        self.path_visible_edges += other.path_visible_edges;
        self.path_fragments += other.path_fragments;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NprPathBuildStats3d {
    pub(crate) project_us: f64,
    pub(crate) visibility_us: f64,
    pub(crate) edge_sample_us: f64,
    pub(crate) stitch_us: f64,
    pub(crate) visible_edges: usize,
    pub(crate) fragments: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct NprFaceVisibilityBuffer {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) face_id: Vec<usize>,
    pub(crate) face_visible: Vec<bool>,
}

pub(crate) fn npr_stroke_plan_length_bucket(length_px: f32) -> u32 {
    (length_px.max(0.0) / 8.0).round() as u32
}

pub(crate) fn npr_stroke_plan_settings_signature(
    settings: &amigo_render_api::NprLineSettings3d,
) -> u64 {
    let mut hash = 0x9E37_79B9_7F4A_7C15_u64;
    hash = mix_u64(hash, settings.stroke_tool as u64);
    hash = mix_u64(hash, settings.suggestive as u64);
    hash = mix_u64(hash, settings.contact as u64);
    hash = mix_u64(hash, settings.contact_ground_y.to_bits() as u64);
    hash = mix_u64(hash, settings.contact_threshold.to_bits() as u64);
    hash = mix_u64(hash, settings.passes as u64);
    hash = mix_u64(hash, settings.search_line_count as u64);
    hash = mix_u64(hash, settings.search_line_alpha.to_bits() as u64);
    hash = mix_u64(hash, settings.pipeline.candidate_strategy as u64);
    hash = mix_u64(hash, settings.pipeline.stroke_strategy as u64);
    hash = mix_u64(hash, settings.pipeline.hatching_strategy as u64);
    hash = mix_u64(hash, settings.pipeline.budget_strategy as u64);
    hash = mix_u64(hash, settings.dropout.to_bits() as u64);
    hash = mix_u64(hash, settings.dropout_segment_min_px.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_dropout_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_search_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.tool_alpha_multiplier.to_bits() as u64);
    hash = mix_u64(hash, settings.ink_color.a.to_bits() as u64);
    hash = mix_u64(hash, settings.seed);
    hash
}

fn mix_u64(current: u64, value: u64) -> u64 {
    current
        .wrapping_mul(0x100_0000_01B3)
        .wrapping_add(value ^ 0xA53A_9E37_1337_5EED)
}
