use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform3, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3dRenderSettings {
    pub fov_y_degrees: f32,
    pub near_clip: f32,
    pub far_clip: f32,
}

impl Default for Camera3dRenderSettings {
    fn default() -> Self {
        Self {
            fov_y_degrees: 55.0,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light3dRenderSettings {
    pub direction: Vec3,
    pub color: ColorRgba,
    pub intensity: f32,
    pub ambient: f32,
}

impl Default for Light3dRenderSettings {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.35, -0.8, -0.45),
            color: ColorRgba::WHITE,
            intensity: 0.85,
            ambient: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh3d {
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
    pub npr: Option<NprLineSettings3d>,
}

#[derive(Debug, Clone)]
pub struct MeshDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub mesh: Mesh3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprRenderStrategy3d {
    #[default]
    GpuRealtime,
    CpuReference,
}

impl NprRenderStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GpuRealtime => "gpu_realtime",
            Self::CpuReference => "cpu_reference",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprFillMode3d {
    Shaded,
    #[default]
    None,
    DepthOnly,
}

impl NprFillMode3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shaded => "shaded",
            Self::None => "none",
            Self::DepthOnly => "depth_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprCandidateStrategy3d {
    #[default]
    GeometryEdges,
    CharacterSemantic,
}

impl NprCandidateStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GeometryEdges => "geometry_edges",
            Self::CharacterSemantic => "character_semantic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprPathStrategy3d {
    StableStrokedPaths,
    #[default]
    DirectVisibleSegments,
}

impl NprPathStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StableStrokedPaths => "stable_stroked_paths",
            Self::DirectVisibleSegments => "direct_visible_segments",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprStrokeStrategy3d {
    #[default]
    ComicInk,
    AkiraInk,
    TechnicalInk,
    RoughPencil,
}

impl NprStrokeStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComicInk => "comic_ink",
            Self::AkiraInk => "akira_ink",
            Self::TechnicalInk => "technical_ink",
            Self::RoughPencil => "rough_pencil",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprInkFillStrategy3d {
    #[default]
    None,
    MaterialBlackMass,
    BinaryMangaShadow,
}

impl NprInkFillStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MaterialBlackMass => "material_black_mass",
            Self::BinaryMangaShadow => "binary_manga_shadow",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprHatchingStrategy3d {
    #[default]
    None,
    SparseCharacterHatching,
}

impl NprHatchingStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SparseCharacterHatching => "sparse_character_hatching",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprBudgetStrategy3d {
    #[default]
    EdgeVisibility,
    FaceAndSilhouettePriority,
    CharacterReadability,
}

impl NprBudgetStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EdgeVisibility => "edge_visibility",
            Self::FaceAndSilhouettePriority => "face_and_silhouette_priority",
            Self::CharacterReadability => "character_readability",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NprTemporalStrategy3d {
    #[default]
    PathHistory,
    StableArcLength,
}

impl NprTemporalStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PathHistory => "path_history",
            Self::StableArcLength => "stable_arc_length",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NprPipelineStrategies3d {
    pub candidate_strategy: NprCandidateStrategy3d,
    pub path_strategy: NprPathStrategy3d,
    pub stroke_strategy: NprStrokeStrategy3d,
    pub fill_strategy: NprInkFillStrategy3d,
    pub hatching_strategy: NprHatchingStrategy3d,
    pub budget_strategy: NprBudgetStrategy3d,
    pub temporal_strategy: NprTemporalStrategy3d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NprGpuDebugMode3d {
    Final,
    LineKinds,
    RawPaths,
    Dropout,
    WidthAlpha,
}

impl Default for NprGpuDebugMode3d {
    fn default() -> Self {
        Self::Final
    }
}

impl NprGpuDebugMode3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Final => "final",
            Self::LineKinds => "line_kinds",
            Self::RawPaths => "raw_paths",
            Self::Dropout => "dropout",
            Self::WidthAlpha => "width_alpha",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "final" | "camera.final" => Some(Self::Final),
            "line_kinds" | "npr.line_kinds" | "npr.kinds" => Some(Self::LineKinds),
            "raw_paths" | "npr.raw_paths" | "npr.paths" => Some(Self::RawPaths),
            "dropout" | "npr.dropout" | "npr.breakup" => Some(Self::Dropout),
            "width_alpha" | "npr.width_alpha" | "npr.pressure" => Some(Self::WidthAlpha),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprGpuRealtimeTuning3d {
    pub debug_mode: NprGpuDebugMode3d,
    pub max_render_length_px: f32,
    pub max_segment_length_px: f32,
    pub max_terminal_walk_edges: u32,
    pub max_chained_walk_edges: u32,
    pub max_chain_angle_degrees: f32,
    pub search_enabled: bool,
    pub search_max_render_length_px: f32,
    pub search_alpha_multiplier: f32,
    pub feature_min_length_multiplier: f32,
    pub feature_alpha_multiplier: f32,
    pub silhouette_min_length_multiplier: f32,
}

impl Default for NprGpuRealtimeTuning3d {
    fn default() -> Self {
        Self {
            debug_mode: NprGpuDebugMode3d::Final,
            max_render_length_px: 56.0,
            max_segment_length_px: 18.0,
            max_terminal_walk_edges: 0,
            max_chained_walk_edges: 0,
            max_chain_angle_degrees: 32.0,
            search_enabled: false,
            search_max_render_length_px: 24.0,
            search_alpha_multiplier: 0.35,
            feature_min_length_multiplier: 1.65,
            feature_alpha_multiplier: 0.72,
            silhouette_min_length_multiplier: 0.75,
        }
    }
}

impl NprGpuRealtimeTuning3d {
    pub fn rough_comic_experimental() -> Self {
        Self {
            debug_mode: NprGpuDebugMode3d::Final,
            max_render_length_px: 120.0,
            max_segment_length_px: 32.0,
            max_terminal_walk_edges: 2,
            max_chained_walk_edges: 4,
            max_chain_angle_degrees: 52.0,
            search_enabled: true,
            search_max_render_length_px: 48.0,
            search_alpha_multiplier: 0.55,
            feature_min_length_multiplier: 1.10,
            feature_alpha_multiplier: 0.86,
            silhouette_min_length_multiplier: 0.65,
        }
    }

    pub fn normalized(mut self) -> Self {
        if !self.max_render_length_px.is_finite() {
            self.max_render_length_px = 56.0;
        }
        if !self.max_segment_length_px.is_finite() {
            self.max_segment_length_px = 18.0;
        }
        if !self.max_chain_angle_degrees.is_finite() {
            self.max_chain_angle_degrees = 32.0;
        }
        if !self.search_max_render_length_px.is_finite() {
            self.search_max_render_length_px = 24.0;
        }
        if !self.search_alpha_multiplier.is_finite() {
            self.search_alpha_multiplier = 0.35;
        }
        if !self.feature_min_length_multiplier.is_finite() {
            self.feature_min_length_multiplier = 1.65;
        }
        if !self.feature_alpha_multiplier.is_finite() {
            self.feature_alpha_multiplier = 0.72;
        }
        if !self.silhouette_min_length_multiplier.is_finite() {
            self.silhouette_min_length_multiplier = 0.75;
        }

        self.max_render_length_px = self.max_render_length_px.clamp(8.0, 512.0);
        self.max_segment_length_px = self.max_segment_length_px.clamp(4.0, 128.0);
        self.max_terminal_walk_edges = self.max_terminal_walk_edges.min(16);
        self.max_chained_walk_edges = self.max_chained_walk_edges.min(64);
        self.max_chain_angle_degrees = self.max_chain_angle_degrees.clamp(1.0, 179.0);
        self.search_max_render_length_px = self.search_max_render_length_px.clamp(4.0, 256.0);
        self.search_alpha_multiplier = self.search_alpha_multiplier.clamp(0.0, 1.0);
        self.feature_min_length_multiplier = self.feature_min_length_multiplier.clamp(0.25, 8.0);
        self.feature_alpha_multiplier = self.feature_alpha_multiplier.clamp(0.0, 1.0);
        self.silhouette_min_length_multiplier =
            self.silhouette_min_length_multiplier.clamp(0.25, 4.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprLineSettings3d {
    pub style_preset: NprStylePreset3d,
    pub stroke_tool: NprStrokeTool3d,
    pub render_strategy: NprRenderStrategy3d,
    pub pipeline: NprPipelineStrategies3d,
    pub gpu_realtime_tuning: NprGpuRealtimeTuning3d,
    pub fill_mode: NprFillMode3d,
    pub boundary: bool,
    pub silhouette: bool,
    pub feature: bool,
    pub suggestive: bool,
    pub contact: bool,
    pub contact_ground_y: f32,
    pub contact_threshold: f32,
    pub black_mass_material_ids: Vec<u32>,
    pub ink_detail_material_ids: Vec<u32>,
    pub feature_angle_degrees: f32,
    pub min_screen_length_px: f32,
    pub ink_color: ColorRgba,
    pub humanization: f32,
    pub line_confidence: f32,
    pub temporal_stability: f32,
    pub temporal_path_smoothing: bool,
    pub visibility_hysteresis_frames: u8,
    pub visibility_max_dimension_px: f32,
    pub width_px: f32,
    pub tool_width_multiplier: f32,
    pub tool_alpha_multiplier: f32,
    pub tool_wobble_multiplier: f32,
    pub tool_pressure_jitter_multiplier: f32,
    pub tool_dropout_multiplier: f32,
    pub tool_search_multiplier: f32,
    pub silhouette_width_multiplier: f32,
    pub boundary_width_multiplier: f32,
    pub feature_width_multiplier: f32,
    pub distance_width_falloff: f32,
    pub depth_pressure: f32,
    pub depth_alpha: f32,
    pub width_pressure_curve: [f32; 4],
    pub alpha_pressure_curve: [f32; 4],
    pub endpoint_snap_px: f32,
    pub endpoint_lock_start_px: f32,
    pub endpoint_lock_end_px: f32,
    pub path_simplify_px: f32,
    pub straightness: f32,
    pub taper: f32,
    pub stroke_wobble_px: f32,
    pub stroke_wobble_frequency: f32,
    pub micro_wobble_px: f32,
    pub micro_wobble_frequency: f32,
    pub pressure_jitter: f32,
    pub local_angular_drift_degrees: f32,
    pub overshoot_px: f32,
    pub undershoot_px: f32,
    pub pass_offset_px: f32,
    pub dropout: f32,
    pub dropout_segment_min_px: f32,
    pub passes: u8,
    pub search_line_count: u8,
    pub search_line_alpha: f32,
    pub seed: u64,
    pub silhouette_override: Option<NprLineKindOverride3d>,
    pub boundary_override: Option<NprLineKindOverride3d>,
    pub feature_override: Option<NprLineKindOverride3d>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprStylePreset3d {
    GpuStableComic,
    RoughComicInk,
}

impl NprStylePreset3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GpuStableComic => "gpu_stable_comic",
            Self::RoughComicInk => "rough_comic_ink",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprStrokeTool3d {
    InkPen,
    Pencil,
    Brush,
    Marker,
    TechnicalPen,
}

impl NprStrokeTool3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InkPen => "ink_pen",
            Self::Pencil => "pencil",
            Self::Brush => "brush",
            Self::Marker => "marker",
            Self::TechnicalPen => "technical_pen",
        }
    }
}

impl Default for NprStrokeTool3d {
    fn default() -> Self {
        Self::InkPen
    }
}

impl Default for NprStylePreset3d {
    fn default() -> Self {
        Self::GpuStableComic
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprLineKindOverride3d {
    pub width_multiplier: Option<f32>,
    pub wobble_px: Option<f32>,
    pub dropout: Option<f32>,
    pub taper: Option<f32>,
    pub overshoot_px: Option<f32>,
    pub alpha_multiplier: Option<f32>,
}

impl Default for NprLineSettings3d {
    fn default() -> Self {
        Self::from_preset(NprStylePreset3d::default())
    }
}

impl NprLineSettings3d {
    pub fn from_preset(style_preset: NprStylePreset3d) -> Self {
        match style_preset {
            NprStylePreset3d::GpuStableComic => Self {
                style_preset,
                stroke_tool: NprStrokeTool3d::TechnicalPen,
                render_strategy: NprRenderStrategy3d::GpuRealtime,
                pipeline: NprPipelineStrategies3d {
                    stroke_strategy: NprStrokeStrategy3d::TechnicalInk,
                    ..NprPipelineStrategies3d::default()
                },
                fill_mode: NprFillMode3d::None,
                boundary: true,
                silhouette: true,
                feature: true,
                suggestive: false,
                contact: false,
                contact_ground_y: 0.0,
                contact_threshold: 0.08,
                black_mass_material_ids: Vec::new(),
                ink_detail_material_ids: Vec::new(),
                feature_angle_degrees: 42.0,
                min_screen_length_px: 2.4,
                ink_color: ColorRgba::new(0.070, 0.062, 0.050, 1.0),
                humanization: 0.16,
                line_confidence: 0.94,
                temporal_stability: 0.97,
                temporal_path_smoothing: true,
                visibility_hysteresis_frames: 3,
                visibility_max_dimension_px: 720.0,
                width_px: 2.0,
                tool_width_multiplier: 1.0,
                tool_alpha_multiplier: 1.0,
                tool_wobble_multiplier: 1.0,
                tool_pressure_jitter_multiplier: 1.0,
                tool_dropout_multiplier: 1.0,
                tool_search_multiplier: 1.0,
                silhouette_width_multiplier: 1.48,
                boundary_width_multiplier: 0.96,
                feature_width_multiplier: 0.52,
                distance_width_falloff: 0.08,
                depth_pressure: 0.08,
                depth_alpha: 0.04,
                width_pressure_curve: [0.88, 1.0, 0.98, 0.84],
                alpha_pressure_curve: [0.94, 1.0, 0.98, 0.90],
                endpoint_snap_px: 1.1,
                endpoint_lock_start_px: 6.0,
                endpoint_lock_end_px: 7.0,
                path_simplify_px: 1.25,
                straightness: 0.88,
                taper: 0.32,
                stroke_wobble_px: 0.10,
                stroke_wobble_frequency: 0.10,
                micro_wobble_px: 0.02,
                micro_wobble_frequency: 0.55,
                pressure_jitter: 0.02,
                local_angular_drift_degrees: 0.15,
                overshoot_px: 0.25,
                undershoot_px: 0.02,
                pass_offset_px: 0.0,
                dropout: 0.0,
                dropout_segment_min_px: 18.0,
                passes: 1,
                search_line_count: 0,
                search_line_alpha: 0.0,
                seed: 50060,
                gpu_realtime_tuning: NprGpuRealtimeTuning3d::default(),
                silhouette_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(1.18),
                    wobble_px: Some(0.06),
                    dropout: Some(0.0),
                    taper: Some(0.28),
                    overshoot_px: Some(0.18),
                    alpha_multiplier: Some(1.0),
                }),
                boundary_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(0.95),
                    wobble_px: Some(0.05),
                    dropout: Some(0.0),
                    taper: Some(0.26),
                    overshoot_px: Some(0.14),
                    alpha_multiplier: Some(0.92),
                }),
                feature_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(0.55),
                    wobble_px: Some(0.03),
                    dropout: Some(0.0),
                    taper: Some(0.18),
                    overshoot_px: Some(0.05),
                    alpha_multiplier: Some(0.72),
                }),
            },
            NprStylePreset3d::RoughComicInk => Self {
                style_preset,
                stroke_tool: NprStrokeTool3d::InkPen,
                render_strategy: NprRenderStrategy3d::GpuRealtime,
                pipeline: NprPipelineStrategies3d::default(),
                gpu_realtime_tuning: NprGpuRealtimeTuning3d::rough_comic_experimental(),
                fill_mode: NprFillMode3d::None,
                boundary: true,
                silhouette: true,
                feature: true,
                suggestive: false,
                contact: false,
                contact_ground_y: 0.0,
                contact_threshold: 0.08,
                black_mass_material_ids: Vec::new(),
                ink_detail_material_ids: Vec::new(),
                feature_angle_degrees: 32.0,
                min_screen_length_px: 2.0,
                ink_color: ColorRgba::new(0.07, 0.062, 0.05, 1.0),
                humanization: 0.55,
                line_confidence: 0.74,
                temporal_stability: 0.92,
                temporal_path_smoothing: true,
                visibility_hysteresis_frames: 2,
                visibility_max_dimension_px: 1024.0,
                width_px: 2.25,
                tool_width_multiplier: 1.0,
                tool_alpha_multiplier: 1.0,
                tool_wobble_multiplier: 1.0,
                tool_pressure_jitter_multiplier: 1.0,
                tool_dropout_multiplier: 1.0,
                tool_search_multiplier: 1.0,
                silhouette_width_multiplier: 1.42,
                boundary_width_multiplier: 1.0,
                feature_width_multiplier: 0.70,
                distance_width_falloff: 0.14,
                depth_pressure: 0.18,
                depth_alpha: 0.08,
                width_pressure_curve: [0.72, 1.03, 0.98, 0.68],
                alpha_pressure_curve: [0.78, 1.0, 0.92, 0.66],
                endpoint_snap_px: 1.8,
                endpoint_lock_start_px: 10.0,
                endpoint_lock_end_px: 12.0,
                path_simplify_px: 0.8,
                straightness: 0.58,
                taper: 0.65,
                stroke_wobble_px: 0.55,
                stroke_wobble_frequency: 0.22,
                micro_wobble_px: 0.10,
                micro_wobble_frequency: 1.15,
                pressure_jitter: 0.12,
                local_angular_drift_degrees: 1.1,
                overshoot_px: 1.4,
                undershoot_px: 0.15,
                pass_offset_px: 0.22,
                dropout: 0.035,
                dropout_segment_min_px: 8.0,
                passes: 2,
                search_line_count: 1,
                search_line_alpha: 0.16,
                seed: 41017,
                silhouette_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(1.48),
                    wobble_px: Some(0.48),
                    dropout: Some(0.012),
                    taper: None,
                    overshoot_px: Some(1.8),
                    alpha_multiplier: Some(1.0),
                }),
                boundary_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(1.0),
                    wobble_px: Some(0.55),
                    dropout: Some(0.030),
                    taper: None,
                    overshoot_px: Some(1.2),
                    alpha_multiplier: Some(0.92),
                }),
                feature_override: Some(NprLineKindOverride3d {
                    width_multiplier: Some(0.68),
                    wobble_px: Some(0.36),
                    dropout: Some(0.055),
                    taper: None,
                    overshoot_px: Some(0.6),
                    alpha_multiplier: Some(0.84),
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npr_line_settings_default_strategy_is_gpu_realtime() {
        assert_eq!(
            NprLineSettings3d::default().render_strategy,
            NprRenderStrategy3d::GpuRealtime
        );
    }

    #[test]
    fn npr_line_settings_default_style_is_gpu_stable_comic() {
        assert_eq!(
            NprLineSettings3d::default().style_preset,
            NprStylePreset3d::GpuStableComic
        );
    }

    #[test]
    fn npr_line_settings_default_fill_mode_is_none() {
        assert_eq!(NprLineSettings3d::default().fill_mode, NprFillMode3d::None);
    }

    #[test]
    fn npr_line_settings_default_pipeline_is_geometry_edge_technical_ink() {
        let settings = NprLineSettings3d::default();
        assert_eq!(
            settings.pipeline.candidate_strategy,
            NprCandidateStrategy3d::GeometryEdges
        );
        assert_eq!(
            settings.pipeline.path_strategy,
            NprPathStrategy3d::DirectVisibleSegments
        );
        assert_eq!(
            settings.pipeline.stroke_strategy,
            NprStrokeStrategy3d::TechnicalInk
        );
        assert_eq!(settings.pipeline.fill_strategy, NprInkFillStrategy3d::None);
        assert_eq!(
            settings.pipeline.hatching_strategy,
            NprHatchingStrategy3d::None
        );
        assert_eq!(
            settings.pipeline.budget_strategy,
            NprBudgetStrategy3d::EdgeVisibility
        );
        assert_eq!(
            settings.pipeline.temporal_strategy,
            NprTemporalStrategy3d::PathHistory
        );
    }

    #[test]
    fn npr_line_settings_default_search_is_disabled() {
        let settings = NprLineSettings3d::default();
        assert_eq!(settings.search_line_count, 0);
        assert!(!settings.gpu_realtime_tuning.search_enabled);
    }

    #[test]
    fn npr_gpu_debug_mode_parses_runtime_and_yaml_labels() {
        assert_eq!(
            NprGpuDebugMode3d::parse("line_kinds"),
            Some(NprGpuDebugMode3d::LineKinds)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("npr.raw_paths"),
            Some(NprGpuDebugMode3d::RawPaths)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("camera.final"),
            Some(NprGpuDebugMode3d::Final)
        );
        assert!(NprGpuDebugMode3d::parse("weird").is_none());
    }
}

#[derive(Debug, Clone)]
pub struct Material3d {
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
    pub render_order: i32,
    pub shading: Material3dShadingMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material3dShadingMode {
    Lit,
    Unlit,
}

impl Default for Material3dShadingMode {
    fn default() -> Self {
        Self::Lit
    }
}

#[derive(Debug, Clone)]
pub struct MaterialDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub material: Material3d,
}

#[derive(Debug, Clone)]
pub struct Text3d {
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

#[derive(Debug, Clone)]
pub struct Text3dDrawCommand {
    pub entity_id: u64,
    pub entity_name: String,
    pub text: Text3d,
}

pub trait Mesh3dRenderOutput {
    fn push_mesh3d_render_command(&mut self, command: MeshDrawCommand);
}

pub trait Material3dRenderOutput {
    fn push_material3d_render_command(&mut self, command: MaterialDrawCommand);
}

pub trait Text3dRenderOutput {
    fn push_text3d_render_command(&mut self, command: Text3dDrawCommand);
}
