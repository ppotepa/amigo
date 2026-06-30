use std::collections::BTreeMap;

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
    ConfidentMangaInk,
    TechnicalInk,
    RoughPencil,
}

impl NprStrokeStrategy3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComicInk => "comic_ink",
            Self::AkiraInk => "akira_ink",
            Self::ConfidentMangaInk => "confident_manga_ink",
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
pub enum NprBlackToneHatchingSource3d {
    #[default]
    Auto,
    ExplicitMaterials,
}

impl NprBlackToneHatchingSource3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ExplicitMaterials => "explicit_materials",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprBlackToneHatching3d {
    pub enabled: bool,
    pub source: NprBlackToneHatchingSource3d,
    pub spacing_px: f32,
    pub length_px: f32,
    pub width_px: f32,
    pub alpha: f32,
    pub density: f32,
    pub tone_threshold: f32,
    pub tone_softness: f32,
    pub angle_degrees: f32,
    pub angle_jitter_degrees: f32,
    pub surface_clip_samples: u8,
    pub max_strokes: u32,
}

impl Default for NprBlackToneHatching3d {
    fn default() -> Self {
        Self {
            enabled: false,
            source: NprBlackToneHatchingSource3d::Auto,
            spacing_px: 7.0,
            length_px: 22.0,
            width_px: 0.65,
            alpha: 0.92,
            density: 0.65,
            tone_threshold: 0.58,
            tone_softness: 0.18,
            angle_degrees: 0.0,
            angle_jitter_degrees: 12.0,
            surface_clip_samples: 12,
            max_strokes: 1200,
        }
    }
}

impl NprBlackToneHatching3d {
    pub fn normalized(mut self) -> Self {
        if !self.spacing_px.is_finite() {
            self.spacing_px = 7.0;
        }
        if !self.length_px.is_finite() {
            self.length_px = 22.0;
        }
        if !self.width_px.is_finite() {
            self.width_px = 0.65;
        }
        if !self.alpha.is_finite() {
            self.alpha = 0.92;
        }
        if !self.density.is_finite() {
            self.density = 0.65;
        }
        if !self.tone_threshold.is_finite() {
            self.tone_threshold = 0.58;
        }
        if !self.tone_softness.is_finite() {
            self.tone_softness = 0.18;
        }
        if !self.angle_degrees.is_finite() {
            self.angle_degrees = 0.0;
        }
        if !self.angle_jitter_degrees.is_finite() {
            self.angle_jitter_degrees = 12.0;
        }
        self.spacing_px = self.spacing_px.clamp(2.0, 64.0);
        self.length_px = self.length_px.clamp(2.0, 240.0);
        self.width_px = self.width_px.clamp(0.1, 8.0);
        self.alpha = self.alpha.clamp(0.0, 1.0);
        self.density = self.density.clamp(0.0, 1.0);
        self.tone_threshold = self.tone_threshold.clamp(0.0, 1.0);
        self.tone_softness = self.tone_softness.clamp(0.001, 1.0);
        self.angle_degrees = self.angle_degrees.clamp(-180.0, 180.0);
        self.angle_jitter_degrees = self.angle_jitter_degrees.clamp(0.0, 90.0);
        self.surface_clip_samples = self.surface_clip_samples.clamp(2, 64);
        self.max_strokes = self.max_strokes.clamp(0, 20_000);
        self
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
pub struct NprLineSelectionProfile3d {
    pub feature_importance: f32,
    pub crease_importance: f32,
    pub seam_importance: f32,
    pub cloth_fold_importance: f32,
    pub detail_ink_importance: f32,
    pub material_detail_bonus: f32,
    pub material_seam_penalty: f32,
    pub length_weight: f32,
    pub angle_weight: f32,
    pub view_weight: f32,
    pub depth_weight: f32,
    pub feature_face_bonus: f32,
    pub feature_torso_bonus: f32,
    pub feature_hand_bonus: f32,
    pub crease_face_bonus: f32,
    pub crease_torso_bonus: f32,
    pub crease_hand_bonus: f32,
    pub seam_torso_bonus: f32,
    pub seam_hand_bonus: f32,
    pub readable_face_start_y: f32,
    pub readable_face_height: f32,
    pub readable_face_half_width: f32,
    pub readable_torso_center_y: f32,
    pub readable_torso_half_height: f32,
    pub readable_torso_half_width: f32,
    pub readable_hand_start_x: f32,
    pub readable_hand_width: f32,
    pub readable_hand_start_y: f32,
    pub readable_hand_height: f32,
    pub short_feature_penalty: f32,
    pub short_crease_penalty: f32,
    pub short_seam_penalty: f32,
    pub readable_region_penalty_relief: f32,
    pub material_detail_penalty_scale: f32,
    pub material_detail_min_screen_length_multiplier: f32,
    pub candidate_length_span_min_screen_multiplier: f32,
    pub candidate_depth_weight: f32,
    pub candidate_depth_min_score: f32,
    pub cloth_fold_length_weight: f32,
    pub detail_ink_material_base: f32,
    pub detail_ink_length_weight: f32,
    pub material_cut_seam_base: f32,
    pub material_cut_length_weight: f32,
    pub short_crease_base_penalty: f32,
    pub short_seam_base_penalty: f32,
    pub short_feature_base_penalty: f32,
    pub readable_region_relief_scale: f32,
    pub detail_keep_importance_weight: f32,
    pub cloth_fold_keep_floor: f32,
    pub detail_ink_keep_floor: f32,
    pub material_cut_keep_floor: f32,
    pub shadow_hatch_keep_floor: f32,
    pub contact_shadow_keep_floor: f32,
    pub generic_feature_keep_floor: f32,
    pub generic_crease_keep_floor: f32,
    pub material_detail_keep_floor_relief: f32,
    pub keep_floor_max: f32,
    pub dense_edge_start_per_10k_px: f32,
    pub dense_edge_full_per_10k_px: f32,
    pub dense_material_seam_start_ratio: f32,
    pub dense_material_seam_full_ratio: f32,
    pub dense_boundary_start_ratio: f32,
    pub dense_boundary_full_ratio: f32,
    pub dense_technical_min_length_boost: f32,
    pub dense_boundary_min_length_boost: f32,
    pub dense_technical_keep_scale_drop: f32,
    pub dense_keep_floor_boost: f32,
    pub dense_material_detail_keep_floor_boost_scale: f32,
    pub dense_material_detail_keep_scale_retention: f32,
    pub dense_boundary_outer_contour_threshold: f32,
    pub dense_pressure_outer_contour_threshold: f32,
    pub dense_seam_pressure_weight: f32,
    pub dense_boundary_pressure_weight: f32,
    pub dense_material_detail_protection: f32,
    pub dense_material_detail_min_length_multiplier: f32,
    pub dense_quality_relief_start: f32,
    pub dense_quality_relief_span: f32,
    pub dense_quality_relief_scale: f32,
    pub dense_quality_relief_penalty_scale: f32,
    pub dense_seam_quality_relief_scale: f32,
    pub dense_seam_penalty_min: f32,
    pub dense_seam_penalty: f32,
    pub dense_feature_penalty: f32,
    pub dense_crease_penalty: f32,
}

impl Default for NprLineSelectionProfile3d {
    fn default() -> Self {
        Self {
            feature_importance: 0.0,
            crease_importance: 0.0,
            seam_importance: 0.16,
            cloth_fold_importance: 0.0,
            detail_ink_importance: 0.0,
            material_detail_bonus: 0.16,
            material_seam_penalty: 0.0,
            length_weight: 0.42,
            angle_weight: 0.28,
            view_weight: 0.06,
            depth_weight: 0.04,
            feature_face_bonus: 0.0,
            feature_torso_bonus: 0.0,
            feature_hand_bonus: 0.0,
            crease_face_bonus: 0.0,
            crease_torso_bonus: 0.0,
            crease_hand_bonus: 0.0,
            seam_torso_bonus: 0.05,
            seam_hand_bonus: 0.05,
            readable_face_start_y: 0.10,
            readable_face_height: 0.55,
            readable_face_half_width: 0.34,
            readable_torso_center_y: 0.02,
            readable_torso_half_height: 0.52,
            readable_torso_half_width: 0.42,
            readable_hand_start_x: 0.48,
            readable_hand_width: 0.34,
            readable_hand_start_y: -0.08,
            readable_hand_height: 0.72,
            short_feature_penalty: 0.14,
            short_crease_penalty: 0.10,
            short_seam_penalty: 0.12,
            readable_region_penalty_relief: 0.0,
            material_detail_penalty_scale: 0.55,
            material_detail_min_screen_length_multiplier: 0.55,
            candidate_length_span_min_screen_multiplier: 6.0,
            candidate_depth_weight: 0.12,
            candidate_depth_min_score: 0.35,
            cloth_fold_length_weight: 0.16,
            detail_ink_material_base: 0.14,
            detail_ink_length_weight: 0.08,
            material_cut_seam_base: 0.10,
            material_cut_length_weight: 0.04,
            short_crease_base_penalty: 0.05,
            short_seam_base_penalty: 0.06,
            short_feature_base_penalty: 0.08,
            readable_region_relief_scale: 1.8,
            detail_keep_importance_weight: 0.26,
            cloth_fold_keep_floor: 0.24,
            detail_ink_keep_floor: 0.30,
            material_cut_keep_floor: 0.32,
            shadow_hatch_keep_floor: 0.40,
            contact_shadow_keep_floor: 0.40,
            generic_feature_keep_floor: 0.44,
            generic_crease_keep_floor: 0.38,
            material_detail_keep_floor_relief: 0.12,
            keep_floor_max: 0.92,
            dense_edge_start_per_10k_px: 260.0,
            dense_edge_full_per_10k_px: 780.0,
            dense_material_seam_start_ratio: 0.06,
            dense_material_seam_full_ratio: 0.24,
            dense_boundary_start_ratio: 0.08,
            dense_boundary_full_ratio: 0.28,
            dense_technical_min_length_boost: 2.15,
            dense_boundary_min_length_boost: 1.45,
            dense_technical_keep_scale_drop: 0.62,
            dense_keep_floor_boost: 0.24,
            dense_material_detail_keep_floor_boost_scale: 0.35,
            dense_material_detail_keep_scale_retention: 0.35,
            dense_boundary_outer_contour_threshold: 0.28,
            dense_pressure_outer_contour_threshold: 0.72,
            dense_seam_pressure_weight: 0.90,
            dense_boundary_pressure_weight: 0.72,
            dense_material_detail_protection: 0.45,
            dense_material_detail_min_length_multiplier: 0.45,
            dense_quality_relief_start: 0.55,
            dense_quality_relief_span: 0.45,
            dense_quality_relief_scale: 0.32,
            dense_quality_relief_penalty_scale: 0.45,
            dense_seam_quality_relief_scale: 0.08,
            dense_seam_penalty_min: 0.07,
            dense_seam_penalty: 0.18,
            dense_feature_penalty: 0.09,
            dense_crease_penalty: 0.07,
        }
    }
}

impl NprLineSelectionProfile3d {
    pub fn toriyama_readability() -> Self {
        Self {
            feature_importance: 0.18,
            crease_importance: 0.24,
            seam_importance: 0.16,
            cloth_fold_importance: 0.10,
            detail_ink_importance: 0.06,
            readable_region_penalty_relief: 0.42,
            feature_face_bonus: 0.18,
            feature_torso_bonus: 0.10,
            feature_hand_bonus: 0.08,
            crease_face_bonus: 0.06,
            crease_torso_bonus: 0.11,
            crease_hand_bonus: 0.05,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprPathJoiningProfile3d {
    pub readable_detail_relax_multiplier: f32,
    pub readable_detail_importance_relax: f32,
    pub readable_detail_relax_max: f32,
    pub continuation_bias_scale: f32,
    pub readable_continuation_bonus: f32,
    pub readable_region_join_bonus: f32,
    pub preferred_length_bias_base: f32,
    pub gap_weight_base: f32,
    pub gap_weight_breakup_scale: f32,
    pub gap_weight_continuation_scale: f32,
    pub gap_weight_readable_relax_scale: f32,
    pub gap_weight_min: f32,
    pub tangent_weight_base: f32,
    pub tangent_weight_breakup_scale: f32,
    pub tangent_weight_continuation_scale: f32,
    pub tangent_weight_readable_relax_scale: f32,
    pub tangent_weight_min: f32,
    pub readability_join_region_scale: f32,
    pub readability_join_importance_scale: f32,
    pub readability_join_continuation_base: f32,
    pub readability_join_continuation_scale: f32,
    pub feature_arc_target_degrees: f32,
    pub feature_arc_window_degrees: f32,
    pub feature_arc_bonus: f32,
    pub crease_arc_target_degrees: f32,
    pub crease_arc_window_degrees: f32,
    pub crease_arc_bonus: f32,
    pub seam_arc_target_degrees: f32,
    pub seam_arc_window_degrees: f32,
    pub seam_arc_bonus: f32,
    pub feature_dead_straight_penalty: f32,
    pub crease_dead_straight_penalty: f32,
    pub path_importance_chain_bonus_per_edge: f32,
    pub path_importance_chain_bonus_max: f32,
    pub path_importance_candidate_base: f32,
    pub path_importance_candidate_scale: f32,
    pub path_importance_min: f32,
    pub path_importance_max: f32,
    pub path_importance_depth_base: f32,
    pub path_importance_depth_weight: f32,
    pub path_importance_depth_min: f32,
    pub path_importance_depth_max: f32,
    pub path_importance_silhouette_multiplier: f32,
    pub path_importance_boundary_multiplier: f32,
    pub path_importance_crease_multiplier: f32,
    pub path_importance_seam_multiplier: f32,
    pub path_importance_feature_multiplier: f32,
    pub path_importance_contact_multiplier: f32,
    pub region_feature_face_bonus: f32,
    pub region_feature_torso_bonus: f32,
    pub region_feature_hand_bonus: f32,
    pub region_crease_face_bonus: f32,
    pub region_crease_torso_bonus: f32,
    pub region_crease_hand_bonus: f32,
    pub region_seam_torso_bonus: f32,
    pub region_seam_hand_bonus: f32,
    pub survival_trait_keep_weight: f32,
    pub survival_base_keep: f32,
    pub survival_length_weight: f32,
    pub survival_confidence_weight: f32,
    pub survival_chain_bonus_per_edge: f32,
    pub survival_chain_bonus_max: f32,
    pub survival_cloth_fold_base_bonus: f32,
    pub survival_cloth_fold_chain_bonus_per_edge: f32,
    pub survival_cloth_fold_chain_bonus_max: f32,
    pub survival_detail_material_bonus: f32,
    pub survival_detail_plain_bonus: f32,
    pub survival_material_cut_seam_bonus: f32,
    pub survival_material_cut_plain_bonus: f32,
    pub survival_long_form_length_weight: f32,
    pub survival_long_form_chain_bonus_per_edge: f32,
    pub survival_long_form_chain_bonus_max: f32,
    pub survival_continuation_weight: f32,
    pub survival_breakup_penalty: f32,
    pub isolated_detail_short_ratio: f32,
    pub isolated_cloth_fold_short_ratio: f32,
    pub isolated_material_cut_short_ratio: f32,
    pub min_length_character_readability_multiplier: f32,
    pub min_length_silhouette_multiplier: f32,
    pub min_length_boundary_multiplier: f32,
    pub min_length_contact_multiplier: f32,
    pub min_length_crease_multiplier: f32,
    pub min_length_seam_multiplier: f32,
    pub min_length_feature_multiplier: f32,
}

impl Default for NprPathJoiningProfile3d {
    fn default() -> Self {
        Self {
            readable_detail_relax_multiplier: 0.0,
            readable_detail_importance_relax: 0.0,
            readable_detail_relax_max: 0.0,
            continuation_bias_scale: 1.0,
            readable_continuation_bonus: 0.0,
            readable_region_join_bonus: 0.0,
            preferred_length_bias_base: 1.6,
            gap_weight_base: 0.8,
            gap_weight_breakup_scale: 0.35,
            gap_weight_continuation_scale: 0.18,
            gap_weight_readable_relax_scale: 0.22,
            gap_weight_min: 0.35,
            tangent_weight_base: 8.0,
            tangent_weight_breakup_scale: 8.0,
            tangent_weight_continuation_scale: 2.3,
            tangent_weight_readable_relax_scale: 3.2,
            tangent_weight_min: 2.0,
            readability_join_region_scale: 1.45,
            readability_join_importance_scale: 0.18,
            readability_join_continuation_base: 0.7,
            readability_join_continuation_scale: 0.5,
            feature_arc_target_degrees: 17.0,
            feature_arc_window_degrees: 20.0,
            feature_arc_bonus: 0.0,
            crease_arc_target_degrees: 15.0,
            crease_arc_window_degrees: 18.0,
            crease_arc_bonus: 0.0,
            seam_arc_target_degrees: 10.0,
            seam_arc_window_degrees: 12.0,
            seam_arc_bonus: 0.14,
            feature_dead_straight_penalty: 0.0,
            crease_dead_straight_penalty: 0.0,
            path_importance_chain_bonus_per_edge: 0.04,
            path_importance_chain_bonus_max: 0.22,
            path_importance_candidate_base: 0.62,
            path_importance_candidate_scale: 0.38,
            path_importance_min: 0.56,
            path_importance_max: 1.10,
            path_importance_depth_base: 1.18,
            path_importance_depth_weight: 0.08,
            path_importance_depth_min: 0.72,
            path_importance_depth_max: 1.18,
            path_importance_silhouette_multiplier: 1.08,
            path_importance_boundary_multiplier: 0.96,
            path_importance_crease_multiplier: 0.92,
            path_importance_seam_multiplier: 0.84,
            path_importance_feature_multiplier: 0.94,
            path_importance_contact_multiplier: 0.92,
            region_feature_face_bonus: 0.12,
            region_feature_torso_bonus: 0.08,
            region_feature_hand_bonus: 0.06,
            region_crease_face_bonus: 0.05,
            region_crease_torso_bonus: 0.08,
            region_crease_hand_bonus: 0.04,
            region_seam_torso_bonus: 0.04,
            region_seam_hand_bonus: 0.04,
            survival_trait_keep_weight: 0.12,
            survival_base_keep: 0.10,
            survival_length_weight: 0.52,
            survival_confidence_weight: 0.10,
            survival_chain_bonus_per_edge: 0.04,
            survival_chain_bonus_max: 0.18,
            survival_cloth_fold_base_bonus: 0.06,
            survival_cloth_fold_chain_bonus_per_edge: 0.03,
            survival_cloth_fold_chain_bonus_max: 0.12,
            survival_detail_material_bonus: 0.12,
            survival_detail_plain_bonus: 0.05,
            survival_material_cut_seam_bonus: 0.10,
            survival_material_cut_plain_bonus: -0.04,
            survival_long_form_length_weight: 0.08,
            survival_long_form_chain_bonus_per_edge: 0.02,
            survival_long_form_chain_bonus_max: 0.08,
            survival_continuation_weight: 0.12,
            survival_breakup_penalty: 0.08,
            isolated_detail_short_ratio: 0.22,
            isolated_cloth_fold_short_ratio: 0.24,
            isolated_material_cut_short_ratio: 0.28,
            min_length_character_readability_multiplier: 1.18,
            min_length_silhouette_multiplier: 0.38,
            min_length_boundary_multiplier: 0.55,
            min_length_contact_multiplier: 0.75,
            min_length_crease_multiplier: 1.0,
            min_length_seam_multiplier: 1.0,
            min_length_feature_multiplier: 1.0,
        }
    }
}

impl NprPathJoiningProfile3d {
    pub fn expressive_ink_paths() -> Self {
        Self {
            readable_detail_relax_multiplier: 3.2,
            readable_detail_importance_relax: 0.20,
            readable_detail_relax_max: 0.38,
            continuation_bias_scale: 2.4,
            readable_continuation_bonus: 1.7,
            readable_region_join_bonus: 0.28,
            feature_arc_bonus: 0.52,
            crease_arc_bonus: 0.34,
            seam_arc_bonus: 0.12,
            feature_dead_straight_penalty: 0.18,
            crease_dead_straight_penalty: 0.12,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprBreakPolicyProfile3d {
    pub allow_seeded_long_feature_breaks: bool,
    pub important_feature_break_threshold: f32,
    pub long_feature_break_min_length_px: f32,
    pub long_feature_break_min_complexity: f32,
    pub long_feature_break_chance: f32,
    pub long_feature_break_center_t: f32,
    pub long_feature_break_center_jitter: f32,
    pub long_feature_break_center_min_t: f32,
    pub long_feature_break_center_max_t: f32,
    pub long_feature_break_min_gap_px: f32,
    pub long_feature_break_gap_jitter_px: f32,
    pub long_feature_break_half_t_min: f32,
    pub long_feature_break_half_t_max: f32,
    pub long_feature_break_t0_min: f32,
    pub long_feature_break_t0_max: f32,
    pub long_feature_break_t1_min: f32,
    pub long_feature_break_t1_max: f32,
    pub dropout_complexity_edge_limit: f32,
    pub dropout_complexity_drop_per_edge: f32,
    pub dropout_effective_max: f32,
    pub dropout_interval_length_px: f32,
    pub dropout_max_intervals: u32,
    pub dropout_min_gap_t: f32,
    pub dropout_max_gap_t: f32,
    pub dropout_edge_margin_t: f32,
}

impl Default for NprBreakPolicyProfile3d {
    fn default() -> Self {
        Self {
            allow_seeded_long_feature_breaks: true,
            important_feature_break_threshold: 1.1,
            long_feature_break_min_length_px: 96.0,
            long_feature_break_min_complexity: 0.0,
            long_feature_break_chance: 0.38,
            long_feature_break_center_t: 0.38,
            long_feature_break_center_jitter: 0.20,
            long_feature_break_center_min_t: 0.22,
            long_feature_break_center_max_t: 0.78,
            long_feature_break_min_gap_px: 7.0,
            long_feature_break_gap_jitter_px: 10.0,
            long_feature_break_half_t_min: 0.018,
            long_feature_break_half_t_max: 0.070,
            long_feature_break_t0_min: 0.08,
            long_feature_break_t0_max: 0.90,
            long_feature_break_t1_min: 0.10,
            long_feature_break_t1_max: 0.92,
            dropout_complexity_edge_limit: 12.0,
            dropout_complexity_drop_per_edge: 0.01,
            dropout_effective_max: 0.85,
            dropout_interval_length_px: 64.0,
            dropout_max_intervals: 8,
            dropout_min_gap_t: 0.01,
            dropout_max_gap_t: 0.25,
            dropout_edge_margin_t: 0.08,
        }
    }
}

impl NprBreakPolicyProfile3d {
    pub fn preserve_readable_features() -> Self {
        Self {
            allow_seeded_long_feature_breaks: true,
            important_feature_break_threshold: 0.70,
            long_feature_break_min_length_px: 160.0,
            long_feature_break_min_complexity: 4.0,
            long_feature_break_chance: 0.22,
            long_feature_break_center_t: 0.42,
            long_feature_break_center_jitter: 0.14,
            long_feature_break_center_min_t: 0.22,
            long_feature_break_center_max_t: 0.78,
            long_feature_break_min_gap_px: 5.0,
            long_feature_break_gap_jitter_px: 7.0,
            long_feature_break_half_t_min: 0.018,
            long_feature_break_half_t_max: 0.070,
            long_feature_break_t0_min: 0.08,
            long_feature_break_t0_max: 0.90,
            long_feature_break_t1_min: 0.10,
            long_feature_break_t1_max: 0.92,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprStrokeSynthesisProfile3d {
    pub silhouette_pressure: f32,
    pub boundary_pressure: f32,
    pub feature_pressure: f32,
    pub crease_pressure: f32,
    pub seam_pressure: f32,
    pub contact_pressure: f32,
    pub technical_importance_base: f32,
    pub technical_candidate_weight: f32,
    pub technical_importance_min: f32,
    pub technical_importance_max: f32,
    pub expressive_importance_min: f32,
    pub expressive_importance_max: f32,
    pub protected_silhouette_importance_threshold: f32,
    pub single_pass_jitter_multiplier: f32,
    pub single_pass_width_multiplier: f32,
    pub single_pass_alpha: f32,
    pub dual_primary_jitter_multiplier: f32,
    pub dual_secondary_jitter_multiplier: f32,
    pub dual_primary_width_multiplier: f32,
    pub dual_secondary_width_multiplier: f32,
    pub dual_primary_alpha: f32,
    pub dual_secondary_alpha: f32,
    pub multi_pass_jitter_base: f32,
    pub multi_pass_jitter_step: f32,
    pub multi_pass_width_multiplier: f32,
    pub multi_pass_alpha: f32,
    pub search_wobble_multiplier: f32,
    pub search_width_multiplier: f32,
    pub hatch_chance_akira: f32,
    pub hatch_chance_confident_manga: f32,
    pub hatch_chance_generic: f32,
    pub hatch_path_length_min_px: f32,
    pub hatch_path_length_max_px: f32,
    pub hatch_center_t: f32,
    pub hatch_center_jitter: f32,
    pub hatch_length_min_px: f32,
    pub hatch_length_jitter_px: f32,
    pub hatch_half_t_min: f32,
    pub hatch_half_t_max: f32,
    pub hatch_wobble_multiplier: f32,
    pub hatch_width_multiplier: f32,
    pub hatch_alpha_multiplier: f32,
    pub hatch_alpha_max: f32,
    pub short_detail_boost: f32,
    pub short_detail_threshold_px: f32,
    pub medium_detail_boost: f32,
    pub medium_detail_threshold_px: f32,
}

impl Default for NprStrokeSynthesisProfile3d {
    fn default() -> Self {
        Self {
            silhouette_pressure: 1.0,
            boundary_pressure: 1.0,
            feature_pressure: 1.0,
            crease_pressure: 1.0,
            seam_pressure: 1.0,
            contact_pressure: 1.0,
            technical_importance_base: 0.82,
            technical_candidate_weight: 0.18,
            technical_importance_min: 0.45,
            technical_importance_max: 1.2,
            expressive_importance_min: 0.55,
            expressive_importance_max: 1.35,
            protected_silhouette_importance_threshold: 0.90,
            single_pass_jitter_multiplier: 0.35,
            single_pass_width_multiplier: 0.75,
            single_pass_alpha: 0.92,
            dual_primary_jitter_multiplier: 1.1,
            dual_secondary_jitter_multiplier: 0.35,
            dual_primary_width_multiplier: 1.6,
            dual_secondary_width_multiplier: 0.85,
            dual_primary_alpha: 0.28,
            dual_secondary_alpha: 0.75,
            multi_pass_jitter_base: 1.0,
            multi_pass_jitter_step: 0.55,
            multi_pass_width_multiplier: 0.9,
            multi_pass_alpha: 0.18,
            search_wobble_multiplier: 1.18,
            search_width_multiplier: 0.78,
            hatch_chance_akira: 0.30,
            hatch_chance_confident_manga: 0.20,
            hatch_chance_generic: 0.18,
            hatch_path_length_min_px: 8.0,
            hatch_path_length_max_px: 44.0,
            hatch_center_t: 0.42,
            hatch_center_jitter: 0.18,
            hatch_length_min_px: 7.0,
            hatch_length_jitter_px: 9.0,
            hatch_half_t_min: 0.04,
            hatch_half_t_max: 0.28,
            hatch_wobble_multiplier: 0.55,
            hatch_width_multiplier: 0.24,
            hatch_alpha_multiplier: 0.38,
            hatch_alpha_max: 0.58,
            short_detail_boost: 1.0,
            short_detail_threshold_px: 0.0,
            medium_detail_boost: 1.0,
            medium_detail_threshold_px: 0.0,
        }
    }
}

impl NprStrokeSynthesisProfile3d {
    pub fn manga_ink() -> Self {
        Self {
            silhouette_pressure: 1.14,
            boundary_pressure: 1.06,
            feature_pressure: 1.08,
            crease_pressure: 1.03,
            seam_pressure: 0.92,
            contact_pressure: 0.96,
            technical_importance_base: 0.82,
            technical_candidate_weight: 0.18,
            technical_importance_min: 0.45,
            technical_importance_max: 1.2,
            expressive_importance_min: 0.55,
            expressive_importance_max: 1.35,
            protected_silhouette_importance_threshold: 0.90,
            single_pass_jitter_multiplier: 0.35,
            single_pass_width_multiplier: 0.75,
            single_pass_alpha: 0.92,
            dual_primary_jitter_multiplier: 1.1,
            dual_secondary_jitter_multiplier: 0.35,
            dual_primary_width_multiplier: 1.6,
            dual_secondary_width_multiplier: 0.85,
            dual_primary_alpha: 0.28,
            dual_secondary_alpha: 0.75,
            multi_pass_jitter_base: 1.0,
            multi_pass_jitter_step: 0.55,
            multi_pass_width_multiplier: 0.9,
            multi_pass_alpha: 0.18,
            search_wobble_multiplier: 1.18,
            search_width_multiplier: 0.78,
            hatch_chance_akira: 0.30,
            hatch_chance_confident_manga: 0.20,
            hatch_chance_generic: 0.18,
            hatch_path_length_min_px: 8.0,
            hatch_path_length_max_px: 44.0,
            hatch_center_t: 0.42,
            hatch_center_jitter: 0.18,
            hatch_length_min_px: 7.0,
            hatch_length_jitter_px: 9.0,
            hatch_half_t_min: 0.04,
            hatch_half_t_max: 0.28,
            hatch_wobble_multiplier: 0.55,
            hatch_width_multiplier: 0.24,
            hatch_alpha_multiplier: 0.38,
            hatch_alpha_max: 0.58,
            short_detail_boost: 1.14,
            short_detail_threshold_px: 22.0,
            medium_detail_boost: 1.06,
            medium_detail_threshold_px: 44.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprTessellationProfile3d {
    pub rail_tangent_smoothing: bool,
    pub kink_fallback_dot: f32,
    pub resample_spacing_px: f32,
    pub endpoint_lock_max_t: f32,
    pub taper_endpoint_floor: f32,
    pub pass_wobble_max_px: f32,
    pub angle_alpha_influence: f32,
    pub min_sample_width_px: f32,
    pub long_stroke_detail_crispness: f32,
    pub hand_arc_length_min: f32,
    pub hand_arc_length_max: f32,
    pub hand_arc_scale: f32,
    pub preferred_length_floor_px: f32,
    pub primary_noise_frequency_scale: f32,
    pub hand_arc_noise_frequency_scale: f32,
    pub hand_arc_noise_phase: f32,
    pub tangent_drift_noise_frequency_scale: f32,
    pub tangent_drift_noise_phase: f32,
    pub micro_noise_frequency_scale: f32,
    pub micro_noise_phase: f32,
    pub width_noise_frequency_scale: f32,
    pub width_noise_phase: f32,
    pub bow_min_length_px: f32,
    pub bow_preferred_min_px: f32,
    pub bow_length_min: f32,
    pub bow_length_max: f32,
    pub bow_wobble_floor_px: f32,
    pub bow_scale: f32,
    pub bow_non_feature_factor: f32,
    pub bow_max_px: f32,
}

impl Default for NprTessellationProfile3d {
    fn default() -> Self {
        Self {
            rail_tangent_smoothing: false,
            kink_fallback_dot: -0.35,
            resample_spacing_px: 2.5,
            endpoint_lock_max_t: 0.45,
            taper_endpoint_floor: 0.35,
            pass_wobble_max_px: 1.5,
            angle_alpha_influence: 0.22,
            min_sample_width_px: 0.25,
            long_stroke_detail_crispness: 0.82,
            hand_arc_length_min: 0.35,
            hand_arc_length_max: 1.15,
            hand_arc_scale: 0.65,
            preferred_length_floor_px: 24.0,
            primary_noise_frequency_scale: 100.0,
            hand_arc_noise_frequency_scale: 18.0,
            hand_arc_noise_phase: 1.7,
            tangent_drift_noise_frequency_scale: 37.0,
            tangent_drift_noise_phase: 3.7,
            micro_noise_frequency_scale: 100.0,
            micro_noise_phase: 13.0,
            width_noise_frequency_scale: 100.0,
            width_noise_phase: 7.0,
            bow_min_length_px: 18.0,
            bow_preferred_min_px: 24.0,
            bow_length_min: 0.35,
            bow_length_max: 1.35,
            bow_wobble_floor_px: 0.18,
            bow_scale: 1.15,
            bow_non_feature_factor: 0.72,
            bow_max_px: 2.4,
        }
    }
}

impl NprTessellationProfile3d {
    pub fn smoothed_strip() -> Self {
        Self {
            rail_tangent_smoothing: true,
            kink_fallback_dot: -0.35,
            resample_spacing_px: 2.0,
            endpoint_lock_max_t: 0.45,
            taper_endpoint_floor: 0.35,
            pass_wobble_max_px: 1.5,
            angle_alpha_influence: 0.22,
            min_sample_width_px: 0.25,
            long_stroke_detail_crispness: 0.82,
            hand_arc_length_min: 0.35,
            hand_arc_length_max: 1.15,
            hand_arc_scale: 0.78,
            preferred_length_floor_px: 24.0,
            primary_noise_frequency_scale: 100.0,
            hand_arc_noise_frequency_scale: 18.0,
            hand_arc_noise_phase: 1.7,
            tangent_drift_noise_frequency_scale: 37.0,
            tangent_drift_noise_phase: 3.7,
            micro_noise_frequency_scale: 100.0,
            micro_noise_phase: 13.0,
            width_noise_frequency_scale: 100.0,
            width_noise_phase: 7.0,
            bow_min_length_px: 16.0,
            bow_preferred_min_px: 22.0,
            bow_length_min: 0.42,
            bow_length_max: 1.45,
            bow_wobble_floor_px: 0.16,
            bow_scale: 1.28,
            bow_non_feature_factor: 0.78,
            bow_max_px: 2.8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NprCpuStrategyProfile3d {
    pub line_selection: NprLineSelectionProfile3d,
    pub path_joining: NprPathJoiningProfile3d,
    pub break_policy: NprBreakPolicyProfile3d,
    pub stroke_synthesis: NprStrokeSynthesisProfile3d,
    pub tessellation: NprTessellationProfile3d,
}

impl NprCpuStrategyProfile3d {
    pub fn toriyama_manga_ink() -> Self {
        Self {
            line_selection: NprLineSelectionProfile3d::toriyama_readability(),
            path_joining: NprPathJoiningProfile3d::expressive_ink_paths(),
            break_policy: NprBreakPolicyProfile3d::preserve_readable_features(),
            stroke_synthesis: NprStrokeSynthesisProfile3d::manga_ink(),
            tessellation: NprTessellationProfile3d::smoothed_strip(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprPipelinePlanWarning3d {
    AkiraInkWithoutCharacterSemantic,
    AkiraInkWithoutStableStrokedPaths,
    AkiraInkWithSearchLines,
    ConfidentMangaInkWithoutCharacterSemantic,
    ConfidentMangaInkWithoutStableStrokedPaths,
    ConfidentMangaInkWithSearchLines,
    SparseHatchingWithoutCharacterSemantic,
    SparseHatchingWithoutCameraResponse,
    GpuRealtimeWithoutStableArcLength,
    CpuReferenceWithGpuDebugMode,
}

impl NprPipelinePlanWarning3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AkiraInkWithoutCharacterSemantic => "akira_ink_without_character_semantic",
            Self::AkiraInkWithoutStableStrokedPaths => "akira_ink_without_stable_stroked_paths",
            Self::AkiraInkWithSearchLines => "akira_ink_with_search_lines",
            Self::ConfidentMangaInkWithoutCharacterSemantic => {
                "confident_manga_ink_without_character_semantic"
            }
            Self::ConfidentMangaInkWithoutStableStrokedPaths => {
                "confident_manga_ink_without_stable_stroked_paths"
            }
            Self::ConfidentMangaInkWithSearchLines => "confident_manga_ink_with_search_lines",
            Self::SparseHatchingWithoutCharacterSemantic => {
                "sparse_hatching_without_character_semantic"
            }
            Self::SparseHatchingWithoutCameraResponse => "sparse_hatching_without_camera_response",
            Self::GpuRealtimeWithoutStableArcLength => "gpu_realtime_without_stable_arc_length",
            Self::CpuReferenceWithGpuDebugMode => "cpu_reference_with_gpu_debug_mode",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NprPipelinePlan3d {
    pub render_strategy: NprRenderStrategy3d,
    pub style_preset: NprStylePreset3d,
    pub stroke_tool: NprStrokeTool3d,
    pub candidate_strategy: NprCandidateStrategy3d,
    pub path_strategy: NprPathStrategy3d,
    pub stroke_strategy: NprStrokeStrategy3d,
    pub fill_strategy: NprInkFillStrategy3d,
    pub hatching_strategy: NprHatchingStrategy3d,
    pub budget_strategy: NprBudgetStrategy3d,
    pub temporal_strategy: NprTemporalStrategy3d,
    pub fill_mode: NprFillMode3d,
    pub camera_response_enabled: bool,
    pub camera_response_auto_focus: bool,
    pub gpu_debug_mode: NprGpuDebugMode3d,
    pub warnings: Vec<NprPipelinePlanWarning3d>,
}

impl NprPipelinePlan3d {
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn warning_labels(&self) -> Vec<&'static str> {
        self.warnings
            .iter()
            .map(|warning| warning.as_str())
            .collect()
    }

    pub fn summary_label(&self) -> String {
        let warnings = if self.warnings.is_empty() {
            "none".to_owned()
        } else {
            self.warning_labels().join("|")
        };
        format!(
            "strategy={} preset={} tool={} candidate={} path={} stroke={} fill={} hatch={} budget={} temporal={} camera={} warnings={}",
            self.render_strategy.as_str(),
            self.style_preset.as_str(),
            self.stroke_tool.as_str(),
            self.candidate_strategy.as_str(),
            self.path_strategy.as_str(),
            self.stroke_strategy.as_str(),
            self.fill_strategy.as_str(),
            self.hatching_strategy.as_str(),
            self.budget_strategy.as_str(),
            self.temporal_strategy.as_str(),
            if self.camera_response_auto_focus {
                "auto_focus"
            } else if self.camera_response_enabled {
                "enabled"
            } else {
                "off"
            },
            warnings
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprGpuDebugMode3d {
    Final,
    LineKinds,
    RawPaths,
    Dropout,
    WidthAlpha,
    ChainHops,
    CandidateImportance,
    TechnicalSelection,
    StrokeLengthBucket,
    SourceEdgeCount,
    StrokeRoles,
    MaterialRoles,
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
            Self::ChainHops => "chain_hops",
            Self::CandidateImportance => "candidate_importance",
            Self::TechnicalSelection => "technical_selection",
            Self::StrokeLengthBucket => "stroke_length_bucket",
            Self::SourceEdgeCount => "source_edge_count",
            Self::StrokeRoles => "stroke_roles",
            Self::MaterialRoles => "material_roles",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "final" | "camera.final" => Some(Self::Final),
            "line_kinds" | "npr.line_kinds" | "npr.kinds" => Some(Self::LineKinds),
            "raw_paths" | "npr.raw_paths" | "npr.paths" => Some(Self::RawPaths),
            "dropout" | "npr.dropout" | "npr.breakup" => Some(Self::Dropout),
            "width_alpha" | "npr.width_alpha" | "npr.pressure" => Some(Self::WidthAlpha),
            "chain_hops" | "npr.chain_hops" | "npr.hops" => Some(Self::ChainHops),
            "candidate_importance"
            | "importance"
            | "npr.candidate_importance"
            | "npr.importance" => Some(Self::CandidateImportance),
            "technical_selection" | "technical" | "npr.technical_selection" => {
                Some(Self::TechnicalSelection)
            }
            "stroke_length_bucket" | "length" | "npr.stroke_length_bucket" => {
                Some(Self::StrokeLengthBucket)
            }
            "source_edge_count" | "source_edges" | "npr.source_edge_count" => {
                Some(Self::SourceEdgeCount)
            }
            "stroke_roles" | "roles" | "npr.stroke_roles" => Some(Self::StrokeRoles),
            "material_roles" | "npr.material_roles" => Some(Self::MaterialRoles),
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
    pub artist_selection_amount: f32,
    pub artist_trim_amount: f32,
    pub artist_lift_amount: f32,
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
            artist_selection_amount: 1.0,
            artist_trim_amount: 1.0,
            artist_lift_amount: 1.0,
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
            artist_selection_amount: 1.0,
            artist_trim_amount: 1.0,
            artist_lift_amount: 1.0,
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
        if !self.artist_selection_amount.is_finite() {
            self.artist_selection_amount = 1.0;
        }
        if !self.artist_trim_amount.is_finite() {
            self.artist_trim_amount = 1.0;
        }
        if !self.artist_lift_amount.is_finite() {
            self.artist_lift_amount = 1.0;
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
        self.artist_selection_amount = self.artist_selection_amount.clamp(0.0, 3.0);
        self.artist_trim_amount = self.artist_trim_amount.clamp(0.0, 3.0);
        self.artist_lift_amount = self.artist_lift_amount.clamp(0.0, 3.0);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprCameraResponse3d {
    pub enabled: bool,
    pub auto_focus: bool,
    pub near_distance: f32,
    pub far_distance: f32,
    pub focus_near_band: f32,
    pub focus_far_band: f32,
    pub near_width_boost: f32,
    pub near_detail_boost: f32,
    pub near_hatching_boost: f32,
    pub far_width_falloff: f32,
    pub far_alpha_falloff: f32,
    pub far_detail_suppression: f32,
    pub rim_silhouette_boost: f32,
    pub front_feature_suppression: f32,
}

impl Default for NprCameraResponse3d {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_focus: false,
            near_distance: 2.0,
            far_distance: 14.0,
            focus_near_band: 0.75,
            focus_far_band: 1.85,
            near_width_boost: 0.0,
            near_detail_boost: 0.0,
            near_hatching_boost: 0.0,
            far_width_falloff: 0.0,
            far_alpha_falloff: 0.0,
            far_detail_suppression: 0.0,
            rim_silhouette_boost: 0.0,
            front_feature_suppression: 0.0,
        }
    }
}

impl NprCameraResponse3d {
    pub fn normalized(mut self) -> Self {
        if !self.near_distance.is_finite() {
            self.near_distance = 2.0;
        }
        if !self.far_distance.is_finite() {
            self.far_distance = 14.0;
        }
        if !self.focus_near_band.is_finite() {
            self.focus_near_band = 0.75;
        }
        if !self.focus_far_band.is_finite() {
            self.focus_far_band = 1.85;
        }
        if !self.near_width_boost.is_finite() {
            self.near_width_boost = 0.0;
        }
        if !self.near_detail_boost.is_finite() {
            self.near_detail_boost = 0.0;
        }
        if !self.near_hatching_boost.is_finite() {
            self.near_hatching_boost = 0.0;
        }
        if !self.far_width_falloff.is_finite() {
            self.far_width_falloff = 0.0;
        }
        if !self.far_alpha_falloff.is_finite() {
            self.far_alpha_falloff = 0.0;
        }
        if !self.far_detail_suppression.is_finite() {
            self.far_detail_suppression = 0.0;
        }
        if !self.rim_silhouette_boost.is_finite() {
            self.rim_silhouette_boost = 0.0;
        }
        if !self.front_feature_suppression.is_finite() {
            self.front_feature_suppression = 0.0;
        }

        self.near_distance = self.near_distance.clamp(0.05, 10_000.0);
        self.far_distance = self.far_distance.clamp(self.near_distance + 0.05, 10_000.0);
        self.focus_near_band = self.focus_near_band.clamp(0.05, 1000.0);
        self.focus_far_band = self
            .focus_far_band
            .clamp(self.focus_near_band + 0.05, 1000.0);
        self.near_width_boost = self.near_width_boost.clamp(0.0, 2.0);
        self.near_detail_boost = self.near_detail_boost.clamp(0.0, 2.0);
        self.near_hatching_boost = self.near_hatching_boost.clamp(0.0, 3.0);
        self.far_width_falloff = self.far_width_falloff.clamp(0.0, 2.0);
        self.far_alpha_falloff = self.far_alpha_falloff.clamp(0.0, 2.0);
        self.far_detail_suppression = self.far_detail_suppression.clamp(0.0, 3.0);
        self.rim_silhouette_boost = self.rim_silhouette_boost.clamp(0.0, 2.0);
        self.front_feature_suppression = self.front_feature_suppression.clamp(0.0, 2.0);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprLineSettings3d {
    pub style_preset: NprStylePreset3d,
    pub stroke_tool: NprStrokeTool3d,
    pub render_strategy: NprRenderStrategy3d,
    pub pipeline: NprPipelineStrategies3d,
    pub cpu_strategy_profile: NprCpuStrategyProfile3d,
    pub gpu_realtime_tuning: NprGpuRealtimeTuning3d,
    pub camera_response: NprCameraResponse3d,
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
    pub black_tone_hatching: NprBlackToneHatching3d,
    pub brush_profiles: BTreeMap<String, NprBrushProfile3d>,
    pub line_families: Vec<NprLineFamily3d>,
    pub feature_angle_degrees: f32,
    pub min_screen_length_px: f32,
    pub min_stroke_length_px: f32,
    pub preferred_stroke_length_px: f32,
    pub stroke_join_gap_px: f32,
    pub stroke_join_max_angle_degrees: f32,
    pub technical_detail_keep: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprLineSource3d {
    Silhouette,
    Boundary,
    Feature,
    Crease,
    Seam,
    Contact,
}

impl NprLineSource3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Silhouette => "silhouette",
            Self::Boundary => "boundary",
            Self::Feature => "feature",
            Self::Crease => "crease",
            Self::Seam => "seam",
            Self::Contact => "contact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprBrushTip3d {
    Round,
    Flat,
    GPen,
    MaruPen,
    DryBrush,
}

impl NprBrushTip3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Round => "round",
            Self::Flat => "flat",
            Self::GPen => "g_pen",
            Self::MaruPen => "maru_pen",
            Self::DryBrush => "dry_brush",
        }
    }
}

impl Default for NprBrushTip3d {
    fn default() -> Self {
        Self::Round
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprLineFamilyRole3d {
    Generic,
    OuterContour,
    DetailInk,
    ClothFold,
    MaterialCut,
    ShadowHatch,
    ContactShadow,
}

impl NprLineFamilyRole3d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::OuterContour => "outer_contour",
            Self::DetailInk => "detail_ink",
            Self::ClothFold => "cloth_fold",
            Self::MaterialCut => "material_cut",
            Self::ShadowHatch => "shadow_hatch",
            Self::ContactShadow => "contact_shadow",
        }
    }
}

impl Default for NprLineFamilyRole3d {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprBrushProfile3d {
    pub tool: Option<NprStrokeTool3d>,
    pub tip: Option<NprBrushTip3d>,
    pub width_multiplier: f32,
    pub alpha_multiplier: f32,
    pub pressure_jitter_multiplier: f32,
    pub dropout_multiplier: f32,
    pub search_multiplier: f32,
    pub path_wobble_multiplier: f32,
    pub micro_wobble_multiplier: f32,
    pub hand_arc_multiplier: f32,
    pub tangent_drift_multiplier: f32,
    pub detail_crispness_multiplier: f32,
    pub taper_multiplier: f32,
    pub overshoot_px: Option<f32>,
    pub width_curve: [f32; 4],
    pub alpha_curve: [f32; 4],
    pub angle_bias_degrees: f32,
    pub angle_influence: f32,
    pub nib_width_base_scale: f32,
    pub nib_width_angle_scale: f32,
    pub path_adherence_multiplier: f32,
}

impl Default for NprBrushProfile3d {
    fn default() -> Self {
        Self {
            tool: None,
            tip: None,
            width_multiplier: 1.0,
            alpha_multiplier: 1.0,
            pressure_jitter_multiplier: 1.0,
            dropout_multiplier: 1.0,
            search_multiplier: 1.0,
            path_wobble_multiplier: 1.0,
            micro_wobble_multiplier: 1.0,
            hand_arc_multiplier: 1.0,
            tangent_drift_multiplier: 1.0,
            detail_crispness_multiplier: 1.0,
            taper_multiplier: 1.0,
            overshoot_px: None,
            width_curve: [1.0, 1.0, 1.0, 1.0],
            alpha_curve: [1.0, 1.0, 1.0, 1.0],
            angle_bias_degrees: 0.0,
            angle_influence: 0.0,
            nib_width_base_scale: 1.0,
            nib_width_angle_scale: 1.0,
            path_adherence_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprLineFamily3d {
    pub id: String,
    pub enabled: bool,
    pub role: Option<NprLineFamilyRole3d>,
    pub priority: i32,
    pub sources: Vec<NprLineSource3d>,
    pub brush: Option<String>,
    pub preferred_stroke_length_px: Option<f32>,
    pub stroke_join_gap_px: Option<f32>,
    pub stroke_join_max_angle_degrees: Option<f32>,
    pub technical_detail_keep: Option<f32>,
    pub min_screen_length_px: Option<f32>,
    pub min_stroke_length_px: Option<f32>,
    pub technical_detail_preference: Option<f32>,
    pub ink_detail_material_preference: Option<f32>,
    pub material_seam_preference: Option<f32>,
    pub continuation_bias: Option<f32>,
    pub breakup_bias: Option<f32>,
    pub width_multiplier: f32,
    pub alpha_multiplier: f32,
    pub taper_multiplier: f32,
    pub overshoot_px: Option<f32>,
}

impl Default for NprLineFamily3d {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            role: None,
            priority: 0,
            sources: Vec::new(),
            brush: None,
            preferred_stroke_length_px: None,
            stroke_join_gap_px: None,
            stroke_join_max_angle_degrees: None,
            technical_detail_keep: None,
            min_screen_length_px: None,
            min_stroke_length_px: None,
            technical_detail_preference: None,
            ink_detail_material_preference: None,
            material_seam_preference: None,
            continuation_bias: None,
            breakup_bias: None,
            width_multiplier: 1.0,
            alpha_multiplier: 1.0,
            taper_multiplier: 1.0,
            overshoot_px: None,
        }
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
                cpu_strategy_profile: NprCpuStrategyProfile3d::default(),
                camera_response: NprCameraResponse3d::default(),
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
                black_tone_hatching: NprBlackToneHatching3d::default(),
                brush_profiles: BTreeMap::new(),
                line_families: Vec::new(),
                feature_angle_degrees: 42.0,
                min_screen_length_px: 2.4,
                min_stroke_length_px: 0.0,
                preferred_stroke_length_px: 56.0,
                stroke_join_gap_px: 2.25,
                stroke_join_max_angle_degrees: 22.0,
                technical_detail_keep: 0.72,
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
                cpu_strategy_profile: NprCpuStrategyProfile3d::default(),
                gpu_realtime_tuning: NprGpuRealtimeTuning3d::rough_comic_experimental(),
                camera_response: NprCameraResponse3d::default(),
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
                black_tone_hatching: NprBlackToneHatching3d::default(),
                brush_profiles: BTreeMap::new(),
                line_families: Vec::new(),
                feature_angle_degrees: 32.0,
                min_screen_length_px: 2.0,
                min_stroke_length_px: 0.0,
                preferred_stroke_length_px: 48.0,
                stroke_join_gap_px: 3.0,
                stroke_join_max_angle_degrees: 28.0,
                technical_detail_keep: 0.88,
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

impl NprLineSettings3d {
    pub fn pipeline_plan(&self) -> NprPipelinePlan3d {
        let mut warnings = Vec::new();

        if self.pipeline.stroke_strategy == NprStrokeStrategy3d::AkiraInk {
            if self.pipeline.candidate_strategy != NprCandidateStrategy3d::CharacterSemantic {
                warnings.push(NprPipelinePlanWarning3d::AkiraInkWithoutCharacterSemantic);
            }
            if self.pipeline.path_strategy != NprPathStrategy3d::StableStrokedPaths {
                warnings.push(NprPipelinePlanWarning3d::AkiraInkWithoutStableStrokedPaths);
            }
            if self.search_line_count > 0 || self.gpu_realtime_tuning.search_enabled {
                warnings.push(NprPipelinePlanWarning3d::AkiraInkWithSearchLines);
            }
        }

        if self.pipeline.stroke_strategy == NprStrokeStrategy3d::ConfidentMangaInk {
            if self.pipeline.candidate_strategy != NprCandidateStrategy3d::CharacterSemantic {
                warnings.push(NprPipelinePlanWarning3d::ConfidentMangaInkWithoutCharacterSemantic);
            }
            if self.pipeline.path_strategy != NprPathStrategy3d::StableStrokedPaths {
                warnings.push(NprPipelinePlanWarning3d::ConfidentMangaInkWithoutStableStrokedPaths);
            }
            if self.search_line_count > 0 || self.gpu_realtime_tuning.search_enabled {
                warnings.push(NprPipelinePlanWarning3d::ConfidentMangaInkWithSearchLines);
            }
        }

        if self.pipeline.hatching_strategy == NprHatchingStrategy3d::SparseCharacterHatching {
            if self.pipeline.candidate_strategy != NprCandidateStrategy3d::CharacterSemantic {
                warnings.push(NprPipelinePlanWarning3d::SparseHatchingWithoutCharacterSemantic);
            }
            if !self.camera_response.enabled {
                warnings.push(NprPipelinePlanWarning3d::SparseHatchingWithoutCameraResponse);
            }
        }

        if self.render_strategy == NprRenderStrategy3d::GpuRealtime
            && self.pipeline.temporal_strategy != NprTemporalStrategy3d::StableArcLength
            && matches!(
                self.pipeline.stroke_strategy,
                NprStrokeStrategy3d::AkiraInk | NprStrokeStrategy3d::ConfidentMangaInk
            )
        {
            warnings.push(NprPipelinePlanWarning3d::GpuRealtimeWithoutStableArcLength);
        }

        if self.render_strategy == NprRenderStrategy3d::CpuReference
            && self.gpu_realtime_tuning.debug_mode != NprGpuDebugMode3d::Final
        {
            warnings.push(NprPipelinePlanWarning3d::CpuReferenceWithGpuDebugMode);
        }

        NprPipelinePlan3d {
            render_strategy: self.render_strategy,
            style_preset: self.style_preset,
            stroke_tool: self.stroke_tool,
            candidate_strategy: self.pipeline.candidate_strategy,
            path_strategy: self.pipeline.path_strategy,
            stroke_strategy: self.pipeline.stroke_strategy,
            fill_strategy: self.pipeline.fill_strategy,
            hatching_strategy: self.pipeline.hatching_strategy,
            budget_strategy: self.pipeline.budget_strategy,
            temporal_strategy: self.pipeline.temporal_strategy,
            fill_mode: self.fill_mode,
            camera_response_enabled: self.camera_response.enabled,
            camera_response_auto_focus: self.camera_response.auto_focus,
            gpu_debug_mode: self.gpu_realtime_tuning.debug_mode,
            warnings,
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
    fn npr_line_settings_default_cpu_strategy_profile_is_neutral() {
        let profile = NprLineSettings3d::default().cpu_strategy_profile;

        assert_eq!(profile.line_selection.feature_importance, 0.0);
        assert_eq!(profile.line_selection.readable_face_start_y, 0.10);
        assert_eq!(profile.line_selection.readable_face_height, 0.55);
        assert_eq!(profile.line_selection.readable_face_half_width, 0.34);
        assert_eq!(profile.line_selection.readable_torso_center_y, 0.02);
        assert_eq!(profile.line_selection.readable_torso_half_height, 0.52);
        assert_eq!(profile.line_selection.readable_torso_half_width, 0.42);
        assert_eq!(profile.line_selection.readable_hand_start_x, 0.48);
        assert_eq!(profile.line_selection.readable_hand_width, 0.34);
        assert_eq!(profile.line_selection.readable_hand_start_y, -0.08);
        assert_eq!(profile.line_selection.readable_hand_height, 0.72);
        assert_eq!(profile.path_joining.feature_arc_bonus, 0.0);
        assert_eq!(profile.path_joining.isolated_detail_short_ratio, 0.22);
        assert_eq!(profile.path_joining.isolated_cloth_fold_short_ratio, 0.24);
        assert_eq!(profile.path_joining.isolated_material_cut_short_ratio, 0.28);
        assert_eq!(profile.path_joining.preferred_length_bias_base, 1.6);
        assert_eq!(profile.path_joining.gap_weight_base, 0.8);
        assert_eq!(profile.path_joining.tangent_weight_base, 8.0);
        assert_eq!(profile.path_joining.readability_join_region_scale, 1.45);
        assert!(profile.break_policy.allow_seeded_long_feature_breaks);
        assert_eq!(profile.break_policy.dropout_effective_max, 0.85);
        assert_eq!(profile.break_policy.dropout_interval_length_px, 64.0);
        assert_eq!(profile.break_policy.dropout_max_intervals, 8);
        assert_eq!(profile.break_policy.dropout_edge_margin_t, 0.08);
        assert_eq!(profile.break_policy.long_feature_break_center_min_t, 0.22);
        assert_eq!(profile.break_policy.long_feature_break_center_max_t, 0.78);
        assert_eq!(profile.break_policy.long_feature_break_half_t_min, 0.018);
        assert_eq!(profile.break_policy.long_feature_break_half_t_max, 0.070);
        assert_eq!(profile.break_policy.long_feature_break_t0_min, 0.08);
        assert_eq!(profile.break_policy.long_feature_break_t0_max, 0.90);
        assert_eq!(profile.break_policy.long_feature_break_t1_min, 0.10);
        assert_eq!(profile.break_policy.long_feature_break_t1_max, 0.92);
        assert_eq!(profile.stroke_synthesis.feature_pressure, 1.0);
        assert_eq!(profile.stroke_synthesis.technical_importance_base, 0.82);
        assert_eq!(profile.stroke_synthesis.technical_candidate_weight, 0.18);
        assert_eq!(profile.stroke_synthesis.technical_importance_min, 0.45);
        assert_eq!(profile.stroke_synthesis.technical_importance_max, 1.2);
        assert_eq!(profile.stroke_synthesis.expressive_importance_min, 0.55);
        assert_eq!(profile.stroke_synthesis.expressive_importance_max, 1.35);
        assert_eq!(profile.stroke_synthesis.single_pass_jitter_multiplier, 0.35);
        assert_eq!(profile.stroke_synthesis.single_pass_width_multiplier, 0.75);
        assert_eq!(profile.stroke_synthesis.single_pass_alpha, 0.92);
        assert_eq!(profile.stroke_synthesis.search_wobble_multiplier, 1.18);
        assert_eq!(profile.stroke_synthesis.search_width_multiplier, 0.78);
        assert_eq!(profile.stroke_synthesis.hatch_chance_akira, 0.30);
        assert_eq!(profile.stroke_synthesis.hatch_chance_confident_manga, 0.20);
        assert_eq!(profile.stroke_synthesis.hatch_chance_generic, 0.18);
        assert_eq!(profile.stroke_synthesis.hatch_path_length_min_px, 8.0);
        assert_eq!(profile.stroke_synthesis.hatch_path_length_max_px, 44.0);
        assert_eq!(profile.stroke_synthesis.hatch_width_multiplier, 0.24);
        assert_eq!(profile.stroke_synthesis.hatch_alpha_multiplier, 0.38);
        assert_eq!(profile.stroke_synthesis.hatch_alpha_max, 0.58);
        assert!(!profile.tessellation.rail_tangent_smoothing);
        assert_eq!(profile.tessellation.endpoint_lock_max_t, 0.45);
        assert_eq!(profile.tessellation.taper_endpoint_floor, 0.35);
        assert_eq!(profile.tessellation.pass_wobble_max_px, 1.5);
        assert_eq!(profile.tessellation.angle_alpha_influence, 0.22);
        assert_eq!(profile.tessellation.min_sample_width_px, 0.25);
        assert_eq!(profile.tessellation.long_stroke_detail_crispness, 0.82);
    }

    #[test]
    fn npr_cpu_strategy_profile_toriyama_enables_readability_and_manga_ink() {
        let profile = NprCpuStrategyProfile3d::toriyama_manga_ink();

        assert!(profile.line_selection.feature_face_bonus > 0.0);
        assert!(profile.path_joining.feature_arc_bonus > 0.0);
        assert!(profile.break_policy.important_feature_break_threshold < 1.0);
        assert!(profile.stroke_synthesis.feature_pressure > 1.0);
        assert!(profile.tessellation.rail_tangent_smoothing);
    }

    #[test]
    fn npr_pipeline_plan_resolves_default_framework_layers() {
        let plan = NprLineSettings3d::default().pipeline_plan();

        assert_eq!(plan.render_strategy, NprRenderStrategy3d::GpuRealtime);
        assert_eq!(plan.style_preset, NprStylePreset3d::GpuStableComic);
        assert_eq!(plan.stroke_tool, NprStrokeTool3d::TechnicalPen);
        assert_eq!(plan.stroke_strategy, NprStrokeStrategy3d::TechnicalInk);
        assert_eq!(
            plan.candidate_strategy,
            NprCandidateStrategy3d::GeometryEdges
        );
        assert_eq!(plan.path_strategy, NprPathStrategy3d::DirectVisibleSegments);
        assert_eq!(plan.fill_strategy, NprInkFillStrategy3d::None);
        assert_eq!(plan.hatching_strategy, NprHatchingStrategy3d::None);
        assert_eq!(plan.budget_strategy, NprBudgetStrategy3d::EdgeVisibility);
        assert_eq!(plan.temporal_strategy, NprTemporalStrategy3d::PathHistory);
        assert!(!plan.has_warnings());
    }

    #[test]
    fn npr_pipeline_plan_accepts_akira_framework_stack() {
        let mut settings = NprLineSettings3d::default();
        settings.stroke_tool = NprStrokeTool3d::InkPen;
        settings.pipeline = NprPipelineStrategies3d {
            candidate_strategy: NprCandidateStrategy3d::CharacterSemantic,
            path_strategy: NprPathStrategy3d::StableStrokedPaths,
            stroke_strategy: NprStrokeStrategy3d::AkiraInk,
            fill_strategy: NprInkFillStrategy3d::MaterialBlackMass,
            hatching_strategy: NprHatchingStrategy3d::SparseCharacterHatching,
            budget_strategy: NprBudgetStrategy3d::FaceAndSilhouettePriority,
            temporal_strategy: NprTemporalStrategy3d::StableArcLength,
        };
        settings.camera_response.enabled = true;
        settings.camera_response.auto_focus = true;
        settings.search_line_count = 0;
        settings.gpu_realtime_tuning.search_enabled = false;

        let plan = settings.pipeline_plan();

        assert_eq!(plan.stroke_tool, NprStrokeTool3d::InkPen);
        assert_eq!(plan.stroke_strategy, NprStrokeStrategy3d::AkiraInk);
        assert_eq!(plan.fill_strategy, NprInkFillStrategy3d::MaterialBlackMass);
        assert_eq!(
            plan.hatching_strategy,
            NprHatchingStrategy3d::SparseCharacterHatching
        );
        assert!(plan.camera_response_enabled);
        assert!(plan.camera_response_auto_focus);
        assert_eq!(
            plan.summary_label(),
            "strategy=gpu_realtime preset=gpu_stable_comic tool=ink_pen candidate=character_semantic path=stable_stroked_paths stroke=akira_ink fill=material_black_mass hatch=sparse_character_hatching budget=face_and_silhouette_priority temporal=stable_arc_length camera=auto_focus warnings=none"
        );
        assert!(!plan.has_warnings(), "{:?}", plan.warning_labels());
    }

    #[test]
    fn npr_pipeline_plan_accepts_confident_manga_framework_stack() {
        let mut settings = NprLineSettings3d::default();
        settings.stroke_tool = NprStrokeTool3d::InkPen;
        settings.pipeline = NprPipelineStrategies3d {
            candidate_strategy: NprCandidateStrategy3d::CharacterSemantic,
            path_strategy: NprPathStrategy3d::StableStrokedPaths,
            stroke_strategy: NprStrokeStrategy3d::ConfidentMangaInk,
            fill_strategy: NprInkFillStrategy3d::MaterialBlackMass,
            hatching_strategy: NprHatchingStrategy3d::None,
            budget_strategy: NprBudgetStrategy3d::CharacterReadability,
            temporal_strategy: NprTemporalStrategy3d::StableArcLength,
        };
        settings.camera_response.enabled = true;
        settings.camera_response.auto_focus = true;
        settings.search_line_count = 0;
        settings.gpu_realtime_tuning.search_enabled = false;

        let plan = settings.pipeline_plan();

        assert_eq!(plan.stroke_strategy, NprStrokeStrategy3d::ConfidentMangaInk);
        assert_eq!(plan.fill_strategy, NprInkFillStrategy3d::MaterialBlackMass);
        assert_eq!(
            plan.budget_strategy,
            NprBudgetStrategy3d::CharacterReadability
        );
        assert_eq!(
            plan.summary_label(),
            "strategy=gpu_realtime preset=gpu_stable_comic tool=ink_pen candidate=character_semantic path=stable_stroked_paths stroke=confident_manga_ink fill=material_black_mass hatch=none budget=character_readability temporal=stable_arc_length camera=auto_focus warnings=none"
        );
        assert!(!plan.has_warnings(), "{:?}", plan.warning_labels());
    }

    #[test]
    fn npr_pipeline_plan_warns_about_incoherent_akira_stack() {
        let mut settings = NprLineSettings3d::default();
        settings.pipeline.stroke_strategy = NprStrokeStrategy3d::AkiraInk;
        settings.pipeline.hatching_strategy = NprHatchingStrategy3d::SparseCharacterHatching;
        settings.search_line_count = 1;

        let plan = settings.pipeline_plan();

        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::AkiraInkWithoutCharacterSemantic)
        );
        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::AkiraInkWithoutStableStrokedPaths)
        );
        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::AkiraInkWithSearchLines)
        );
        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::SparseHatchingWithoutCharacterSemantic)
        );
        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::SparseHatchingWithoutCameraResponse)
        );
        assert!(
            plan.warnings
                .contains(&NprPipelinePlanWarning3d::GpuRealtimeWithoutStableArcLength)
        );
    }

    #[test]
    fn npr_line_settings_default_search_is_disabled() {
        let settings = NprLineSettings3d::default();
        assert_eq!(settings.search_line_count, 0);
        assert!(!settings.gpu_realtime_tuning.search_enabled);
    }

    #[test]
    fn npr_brush_tip_and_line_family_role_have_stable_strings() {
        assert_eq!(NprBrushTip3d::GPen.as_str(), "g_pen");
        assert_eq!(NprBrushTip3d::MaruPen.as_str(), "maru_pen");
        assert_eq!(NprLineFamilyRole3d::OuterContour.as_str(), "outer_contour");
        assert_eq!(NprLineFamilyRole3d::DetailInk.as_str(), "detail_ink");
    }

    #[test]
    fn npr_gpu_realtime_artist_tuning_defaults_and_clamps() {
        let default_tuning = NprGpuRealtimeTuning3d::default();
        assert_eq!(default_tuning.artist_selection_amount, 1.0);
        assert_eq!(default_tuning.artist_trim_amount, 1.0);
        assert_eq!(default_tuning.artist_lift_amount, 1.0);

        let normalized = NprGpuRealtimeTuning3d {
            artist_selection_amount: f32::NAN,
            artist_trim_amount: -2.0,
            artist_lift_amount: 9.0,
            ..NprGpuRealtimeTuning3d::default()
        }
        .normalized();

        assert_eq!(normalized.artist_selection_amount, 1.0);
        assert_eq!(normalized.artist_trim_amount, 0.0);
        assert_eq!(normalized.artist_lift_amount, 3.0);
    }

    #[test]
    fn npr_camera_response_defaults_disabled_and_clamps() {
        let default_response = NprCameraResponse3d::default();
        assert!(!default_response.enabled);
        assert_eq!(default_response.near_hatching_boost, 0.0);

        let normalized = NprCameraResponse3d {
            enabled: true,
            auto_focus: true,
            near_distance: f32::NAN,
            far_distance: 0.01,
            focus_near_band: f32::NAN,
            focus_far_band: 0.02,
            near_width_boost: f32::NAN,
            near_detail_boost: 3.5,
            near_hatching_boost: 9.0,
            far_width_falloff: -2.0,
            far_alpha_falloff: 4.0,
            far_detail_suppression: 5.0,
            rim_silhouette_boost: 4.0,
            front_feature_suppression: 4.0,
        }
        .normalized();

        assert!(normalized.enabled);
        assert!(normalized.auto_focus);
        assert_eq!(normalized.near_distance, 2.0);
        assert_eq!(normalized.far_distance, 2.05);
        assert_eq!(normalized.focus_near_band, 0.75);
        assert_eq!(normalized.focus_far_band, 0.8);
        assert_eq!(normalized.near_width_boost, 0.0);
        assert_eq!(normalized.near_detail_boost, 2.0);
        assert_eq!(normalized.near_hatching_boost, 3.0);
        assert_eq!(normalized.far_width_falloff, 0.0);
        assert_eq!(normalized.far_alpha_falloff, 2.0);
        assert_eq!(normalized.far_detail_suppression, 3.0);
        assert_eq!(normalized.rim_silhouette_boost, 2.0);
        assert_eq!(normalized.front_feature_suppression, 2.0);
    }

    #[test]
    fn npr_black_tone_hatching_defaults_are_surface_safe_and_clamp() {
        let default_hatching = NprBlackToneHatching3d::default();

        assert!(!default_hatching.enabled);
        assert_eq!(default_hatching.tone_threshold, 0.58);
        assert_eq!(default_hatching.surface_clip_samples, 12);

        let normalized = NprBlackToneHatching3d {
            tone_threshold: f32::NAN,
            tone_softness: -1.0,
            surface_clip_samples: 1,
            ..NprBlackToneHatching3d::default()
        }
        .normalized();

        assert_eq!(normalized.tone_threshold, 0.58);
        assert_eq!(normalized.tone_softness, 0.001);
        assert_eq!(normalized.surface_clip_samples, 2);
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
        assert_eq!(
            NprGpuDebugMode3d::parse("npr.chain_hops"),
            Some(NprGpuDebugMode3d::ChainHops)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("npr.candidate_importance"),
            Some(NprGpuDebugMode3d::CandidateImportance)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("technical_selection"),
            Some(NprGpuDebugMode3d::TechnicalSelection)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("npr.stroke_length_bucket"),
            Some(NprGpuDebugMode3d::StrokeLengthBucket)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("source_edges"),
            Some(NprGpuDebugMode3d::SourceEdgeCount)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("npr.stroke_roles"),
            Some(NprGpuDebugMode3d::StrokeRoles)
        );
        assert_eq!(
            NprGpuDebugMode3d::parse("material_roles"),
            Some(NprGpuDebugMode3d::MaterialRoles)
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
