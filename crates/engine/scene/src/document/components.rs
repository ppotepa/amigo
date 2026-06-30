use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::behavior::*;
use super::core::*;
use super::defaults::*;
use super::render_values::{SceneVec2Document, SceneVec3Document};
use super::ui::*;
use super::visual2d::PostFx2dDocument;

impl SceneEntityDocument {
    pub fn display_name(&self) -> String {
        if self.name.trim().is_empty() {
            self.id.clone()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneComponentSemanticClass {
    Sprite2d,
    LayeredImage2d,
    TileMap2d,
    Text2d,
    VectorShape2d,
    ParticleEmitter2d,
    BeaconLight2d,
    Camera2d,
    Motion2d,
    Physics2d,
    Physics3d,
    Script,
    Plugin,
    Generic2d,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NprLine3dDocument {
    Enabled(bool),
    Settings(NprLine3dSettingsDocument),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dSettingsDocument {
    #[serde(default = "default_bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub fill_mode: Option<String>,
    #[serde(default)]
    pub style_preset: Option<String>,
    #[serde(default)]
    pub stroke_tool: Option<String>,
    #[serde(default)]
    pub pipeline: Option<NprPipelineStrategiesDocument>,
    #[serde(default)]
    pub cpu_strategy_profile: Option<NprCpuStrategyProfileDocument>,
    #[serde(default)]
    pub boundary: Option<bool>,
    #[serde(default)]
    pub silhouette: Option<bool>,
    #[serde(default)]
    pub feature: Option<bool>,
    #[serde(default)]
    pub suggestive: Option<bool>,
    #[serde(default)]
    pub contact: Option<bool>,
    #[serde(default)]
    pub contact_ground_y: Option<f32>,
    #[serde(default)]
    pub contact_threshold: Option<f32>,
    #[serde(default)]
    pub black_mass_material_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub ink_detail_material_ids: Option<Vec<u32>>,
    #[serde(default)]
    pub black_tone_hatching: Option<NprBlackToneHatchingDocument>,
    #[serde(default)]
    pub brushes: Option<BTreeMap<String, NprBrushProfileDocument>>,
    #[serde(default)]
    pub families: Option<Vec<NprLineFamilyDocument>>,
    #[serde(default)]
    pub feature_angle_degrees: Option<f32>,
    #[serde(default)]
    pub min_screen_length_px: Option<f32>,
    #[serde(default)]
    pub min_stroke_length_px: Option<f32>,
    #[serde(default)]
    pub preferred_stroke_length_px: Option<f32>,
    #[serde(default)]
    pub stroke_join_gap_px: Option<f32>,
    #[serde(default)]
    pub stroke_join_max_angle_degrees: Option<f32>,
    #[serde(default)]
    pub technical_detail_keep: Option<f32>,
    #[serde(default)]
    pub ink_color: Option<String>,
    #[serde(default)]
    pub humanization: Option<f32>,
    #[serde(default)]
    pub line_confidence: Option<f32>,
    #[serde(default)]
    pub temporal_stability: Option<f32>,
    #[serde(default)]
    pub temporal_path_smoothing: Option<bool>,
    #[serde(default)]
    pub visibility_hysteresis_frames: Option<u8>,
    #[serde(default)]
    pub visibility_max_dimension_px: Option<f32>,
    #[serde(default)]
    pub width_px: Option<f32>,
    #[serde(default)]
    pub tool_width_multiplier: Option<f32>,
    #[serde(default)]
    pub tool_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub tool_wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub tool_pressure_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub tool_dropout_multiplier: Option<f32>,
    #[serde(default)]
    pub tool_search_multiplier: Option<f32>,
    #[serde(default)]
    pub silhouette_width_multiplier: Option<f32>,
    #[serde(default)]
    pub boundary_width_multiplier: Option<f32>,
    #[serde(default)]
    pub feature_width_multiplier: Option<f32>,
    #[serde(default)]
    pub distance_width_falloff: Option<f32>,
    #[serde(default)]
    pub depth_pressure: Option<f32>,
    #[serde(default)]
    pub depth_alpha: Option<f32>,
    #[serde(default)]
    pub width_pressure_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub alpha_pressure_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub endpoint_snap_px: Option<f32>,
    #[serde(default)]
    pub endpoint_lock_start_px: Option<f32>,
    #[serde(default)]
    pub endpoint_lock_end_px: Option<f32>,
    #[serde(default)]
    pub path_simplify_px: Option<f32>,
    #[serde(default)]
    pub straightness: Option<f32>,
    #[serde(default)]
    pub taper: Option<f32>,
    #[serde(default)]
    pub stroke_wobble_px: Option<f32>,
    #[serde(default)]
    pub stroke_wobble_frequency: Option<f32>,
    #[serde(default)]
    pub micro_wobble_px: Option<f32>,
    #[serde(default)]
    pub micro_wobble_frequency: Option<f32>,
    #[serde(default)]
    pub pressure_jitter: Option<f32>,
    #[serde(default)]
    pub local_angular_drift_degrees: Option<f32>,
    #[serde(default)]
    pub overshoot_px: Option<f32>,
    #[serde(default)]
    pub undershoot_px: Option<f32>,
    #[serde(default)]
    pub pass_offset_px: Option<f32>,
    #[serde(default)]
    pub dropout: Option<f32>,
    #[serde(default)]
    pub dropout_segment_min_px: Option<f32>,
    #[serde(default)]
    pub passes: Option<NprLine3dPassesFieldDocument>,
    #[serde(default)]
    pub search_line_count: Option<u8>,
    #[serde(default)]
    pub search_line_alpha: Option<f32>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub gpu_realtime_tuning: Option<NprGpuRealtimeTuningDocument>,
    #[serde(default)]
    pub camera_response: Option<NprCameraResponseDocument>,
    #[serde(default)]
    pub silhouette_override: Option<NprLine3dKindOverrideDocument>,
    #[serde(default)]
    pub boundary_override: Option<NprLine3dKindOverrideDocument>,
    #[serde(default)]
    pub feature_override: Option<NprLine3dKindOverrideDocument>,
    #[serde(default)]
    pub tool: Option<NprLine3dToolDocument>,
    #[serde(default)]
    pub trajectory: Option<NprLine3dTrajectoryDocument>,
    #[serde(default)]
    pub pressure: Option<NprLine3dPressureDocument>,
    #[serde(default)]
    pub opacity: Option<NprLine3dOpacityDocument>,
    #[serde(default)]
    pub endpoints: Option<NprLine3dEndpointsDocument>,
    #[serde(default)]
    pub breakup: Option<NprLine3dBreakupDocument>,
    #[serde(default)]
    pub depth: Option<NprLine3dDepthDocument>,
    #[serde(default)]
    pub confidence: Option<NprLine3dConfidenceDocument>,
    #[serde(default)]
    pub class_overrides: Option<NprLine3dClassOverridesDocument>,
    #[serde(default)]
    pub performance: Option<NprLine3dPerformanceDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprGpuRealtimeTuningDocument {
    #[serde(default)]
    pub debug_mode: Option<String>,
    #[serde(default)]
    pub max_render_length_px: Option<f32>,
    #[serde(default)]
    pub max_segment_length_px: Option<f32>,
    #[serde(default)]
    pub max_terminal_walk_edges: Option<u32>,
    #[serde(default)]
    pub max_chained_walk_edges: Option<u32>,
    #[serde(default)]
    pub max_chain_angle_degrees: Option<f32>,
    #[serde(default)]
    pub search_enabled: Option<bool>,
    #[serde(default)]
    pub search_max_render_length_px: Option<f32>,
    #[serde(default)]
    pub search_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub feature_min_length_multiplier: Option<f32>,
    #[serde(default)]
    pub feature_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub silhouette_min_length_multiplier: Option<f32>,
    #[serde(default)]
    pub artist_selection_amount: Option<f32>,
    #[serde(default)]
    pub artist_trim_amount: Option<f32>,
    #[serde(default)]
    pub artist_lift_amount: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprCameraResponseDocument {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub auto_focus: Option<bool>,
    #[serde(default)]
    pub near_distance: Option<f32>,
    #[serde(default)]
    pub far_distance: Option<f32>,
    #[serde(default)]
    pub focus_near_band: Option<f32>,
    #[serde(default)]
    pub focus_far_band: Option<f32>,
    #[serde(default)]
    pub near_width_boost: Option<f32>,
    #[serde(default)]
    pub near_detail_boost: Option<f32>,
    #[serde(default)]
    pub near_hatching_boost: Option<f32>,
    #[serde(default)]
    pub far_width_falloff: Option<f32>,
    #[serde(default)]
    pub far_alpha_falloff: Option<f32>,
    #[serde(default)]
    pub far_detail_suppression: Option<f32>,
    #[serde(default)]
    pub rim_silhouette_boost: Option<f32>,
    #[serde(default)]
    pub front_feature_suppression: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprBlackToneHatchingDocument {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub spacing_px: Option<f32>,
    #[serde(default)]
    pub length_px: Option<f32>,
    #[serde(default)]
    pub width_px: Option<f32>,
    #[serde(default)]
    pub alpha: Option<f32>,
    #[serde(default)]
    pub density: Option<f32>,
    #[serde(default)]
    pub tone_threshold: Option<f32>,
    #[serde(default)]
    pub tone_softness: Option<f32>,
    #[serde(default)]
    pub angle_degrees: Option<f32>,
    #[serde(default)]
    pub angle_jitter_degrees: Option<f32>,
    #[serde(default)]
    pub surface_clip_samples: Option<u8>,
    #[serde(default)]
    pub max_strokes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprPipelineStrategiesDocument {
    #[serde(default)]
    pub candidate_strategy: Option<String>,
    #[serde(default)]
    pub path_strategy: Option<String>,
    #[serde(default)]
    pub stroke_strategy: Option<String>,
    #[serde(default)]
    pub fill_strategy: Option<String>,
    #[serde(default)]
    pub hatching_strategy: Option<String>,
    #[serde(default)]
    pub budget_strategy: Option<String>,
    #[serde(default)]
    pub temporal_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprCpuStrategyProfileDocument {
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub line_selection: Option<NprLineSelectionProfileDocument>,
    #[serde(default)]
    pub path_joining: Option<NprPathJoiningProfileDocument>,
    #[serde(default)]
    pub break_policy: Option<NprBreakPolicyProfileDocument>,
    #[serde(default)]
    pub stroke_synthesis: Option<NprStrokeSynthesisProfileDocument>,
    #[serde(default)]
    pub tessellation: Option<NprTessellationProfileDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLineSelectionProfileDocument {
    #[serde(default)]
    pub feature_importance: Option<f32>,
    #[serde(default)]
    pub crease_importance: Option<f32>,
    #[serde(default)]
    pub seam_importance: Option<f32>,
    #[serde(default)]
    pub cloth_fold_importance: Option<f32>,
    #[serde(default)]
    pub detail_ink_importance: Option<f32>,
    #[serde(default)]
    pub material_detail_bonus: Option<f32>,
    #[serde(default)]
    pub material_seam_penalty: Option<f32>,
    #[serde(default)]
    pub length_weight: Option<f32>,
    #[serde(default)]
    pub angle_weight: Option<f32>,
    #[serde(default)]
    pub view_weight: Option<f32>,
    #[serde(default)]
    pub depth_weight: Option<f32>,
    #[serde(default)]
    pub feature_face_bonus: Option<f32>,
    #[serde(default)]
    pub feature_torso_bonus: Option<f32>,
    #[serde(default)]
    pub feature_hand_bonus: Option<f32>,
    #[serde(default)]
    pub crease_face_bonus: Option<f32>,
    #[serde(default)]
    pub crease_torso_bonus: Option<f32>,
    #[serde(default)]
    pub crease_hand_bonus: Option<f32>,
    #[serde(default)]
    pub seam_torso_bonus: Option<f32>,
    #[serde(default)]
    pub seam_hand_bonus: Option<f32>,
    #[serde(default)]
    pub readable_face_start_y: Option<f32>,
    #[serde(default)]
    pub readable_face_height: Option<f32>,
    #[serde(default)]
    pub readable_face_half_width: Option<f32>,
    #[serde(default)]
    pub readable_torso_center_y: Option<f32>,
    #[serde(default)]
    pub readable_torso_half_height: Option<f32>,
    #[serde(default)]
    pub readable_torso_half_width: Option<f32>,
    #[serde(default)]
    pub readable_hand_start_x: Option<f32>,
    #[serde(default)]
    pub readable_hand_width: Option<f32>,
    #[serde(default)]
    pub readable_hand_start_y: Option<f32>,
    #[serde(default)]
    pub readable_hand_height: Option<f32>,
    #[serde(default)]
    pub short_feature_penalty: Option<f32>,
    #[serde(default)]
    pub short_crease_penalty: Option<f32>,
    #[serde(default)]
    pub short_seam_penalty: Option<f32>,
    #[serde(default)]
    pub readable_region_penalty_relief: Option<f32>,
    #[serde(default)]
    pub material_detail_penalty_scale: Option<f32>,
    #[serde(default)]
    pub material_detail_min_screen_length_multiplier: Option<f32>,
    #[serde(default)]
    pub candidate_length_span_min_screen_multiplier: Option<f32>,
    #[serde(default)]
    pub candidate_depth_weight: Option<f32>,
    #[serde(default)]
    pub candidate_depth_min_score: Option<f32>,
    #[serde(default)]
    pub cloth_fold_length_weight: Option<f32>,
    #[serde(default)]
    pub detail_ink_material_base: Option<f32>,
    #[serde(default)]
    pub detail_ink_length_weight: Option<f32>,
    #[serde(default)]
    pub material_cut_seam_base: Option<f32>,
    #[serde(default)]
    pub material_cut_length_weight: Option<f32>,
    #[serde(default)]
    pub short_crease_base_penalty: Option<f32>,
    #[serde(default)]
    pub short_seam_base_penalty: Option<f32>,
    #[serde(default)]
    pub short_feature_base_penalty: Option<f32>,
    #[serde(default)]
    pub readable_region_relief_scale: Option<f32>,
    #[serde(default)]
    pub detail_keep_importance_weight: Option<f32>,
    #[serde(default)]
    pub cloth_fold_keep_floor: Option<f32>,
    #[serde(default)]
    pub detail_ink_keep_floor: Option<f32>,
    #[serde(default)]
    pub material_cut_keep_floor: Option<f32>,
    #[serde(default)]
    pub shadow_hatch_keep_floor: Option<f32>,
    #[serde(default)]
    pub contact_shadow_keep_floor: Option<f32>,
    #[serde(default)]
    pub generic_feature_keep_floor: Option<f32>,
    #[serde(default)]
    pub generic_crease_keep_floor: Option<f32>,
    #[serde(default)]
    pub material_detail_keep_floor_relief: Option<f32>,
    #[serde(default)]
    pub keep_floor_max: Option<f32>,
    #[serde(default)]
    pub dense_edge_start_per_10k_px: Option<f32>,
    #[serde(default)]
    pub dense_edge_full_per_10k_px: Option<f32>,
    #[serde(default)]
    pub dense_material_seam_start_ratio: Option<f32>,
    #[serde(default)]
    pub dense_material_seam_full_ratio: Option<f32>,
    #[serde(default)]
    pub dense_boundary_start_ratio: Option<f32>,
    #[serde(default)]
    pub dense_boundary_full_ratio: Option<f32>,
    #[serde(default)]
    pub dense_technical_min_length_boost: Option<f32>,
    #[serde(default)]
    pub dense_boundary_min_length_boost: Option<f32>,
    #[serde(default)]
    pub dense_technical_keep_scale_drop: Option<f32>,
    #[serde(default)]
    pub dense_keep_floor_boost: Option<f32>,
    #[serde(default)]
    pub dense_material_detail_keep_floor_boost_scale: Option<f32>,
    #[serde(default)]
    pub dense_material_detail_keep_scale_retention: Option<f32>,
    #[serde(default)]
    pub dense_boundary_outer_contour_threshold: Option<f32>,
    #[serde(default)]
    pub dense_pressure_outer_contour_threshold: Option<f32>,
    #[serde(default)]
    pub dense_seam_pressure_weight: Option<f32>,
    #[serde(default)]
    pub dense_boundary_pressure_weight: Option<f32>,
    #[serde(default)]
    pub dense_material_detail_protection: Option<f32>,
    #[serde(default)]
    pub dense_material_detail_min_length_multiplier: Option<f32>,
    #[serde(default)]
    pub dense_quality_relief_start: Option<f32>,
    #[serde(default)]
    pub dense_quality_relief_span: Option<f32>,
    #[serde(default)]
    pub dense_quality_relief_scale: Option<f32>,
    #[serde(default)]
    pub dense_quality_relief_penalty_scale: Option<f32>,
    #[serde(default)]
    pub dense_seam_quality_relief_scale: Option<f32>,
    #[serde(default)]
    pub dense_seam_penalty_min: Option<f32>,
    #[serde(default)]
    pub dense_seam_penalty: Option<f32>,
    #[serde(default)]
    pub dense_feature_penalty: Option<f32>,
    #[serde(default)]
    pub dense_crease_penalty: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprPathJoiningProfileDocument {
    #[serde(default)]
    pub readable_detail_relax_multiplier: Option<f32>,
    #[serde(default)]
    pub readable_detail_importance_relax: Option<f32>,
    #[serde(default)]
    pub readable_detail_relax_max: Option<f32>,
    #[serde(default)]
    pub continuation_bias_scale: Option<f32>,
    #[serde(default)]
    pub readable_continuation_bonus: Option<f32>,
    #[serde(default)]
    pub readable_region_join_bonus: Option<f32>,
    #[serde(default)]
    pub preferred_length_bias_base: Option<f32>,
    #[serde(default)]
    pub gap_weight_base: Option<f32>,
    #[serde(default)]
    pub gap_weight_breakup_scale: Option<f32>,
    #[serde(default)]
    pub gap_weight_continuation_scale: Option<f32>,
    #[serde(default)]
    pub gap_weight_readable_relax_scale: Option<f32>,
    #[serde(default)]
    pub gap_weight_min: Option<f32>,
    #[serde(default)]
    pub tangent_weight_base: Option<f32>,
    #[serde(default)]
    pub tangent_weight_breakup_scale: Option<f32>,
    #[serde(default)]
    pub tangent_weight_continuation_scale: Option<f32>,
    #[serde(default)]
    pub tangent_weight_readable_relax_scale: Option<f32>,
    #[serde(default)]
    pub tangent_weight_min: Option<f32>,
    #[serde(default)]
    pub readability_join_region_scale: Option<f32>,
    #[serde(default)]
    pub readability_join_importance_scale: Option<f32>,
    #[serde(default)]
    pub readability_join_continuation_base: Option<f32>,
    #[serde(default)]
    pub readability_join_continuation_scale: Option<f32>,
    #[serde(default)]
    pub feature_arc_target_degrees: Option<f32>,
    #[serde(default)]
    pub feature_arc_window_degrees: Option<f32>,
    #[serde(default)]
    pub feature_arc_bonus: Option<f32>,
    #[serde(default)]
    pub crease_arc_target_degrees: Option<f32>,
    #[serde(default)]
    pub crease_arc_window_degrees: Option<f32>,
    #[serde(default)]
    pub crease_arc_bonus: Option<f32>,
    #[serde(default)]
    pub seam_arc_target_degrees: Option<f32>,
    #[serde(default)]
    pub seam_arc_window_degrees: Option<f32>,
    #[serde(default)]
    pub seam_arc_bonus: Option<f32>,
    #[serde(default)]
    pub feature_dead_straight_penalty: Option<f32>,
    #[serde(default)]
    pub crease_dead_straight_penalty: Option<f32>,
    #[serde(default)]
    pub path_importance_chain_bonus_per_edge: Option<f32>,
    #[serde(default)]
    pub path_importance_chain_bonus_max: Option<f32>,
    #[serde(default)]
    pub path_importance_candidate_base: Option<f32>,
    #[serde(default)]
    pub path_importance_candidate_scale: Option<f32>,
    #[serde(default)]
    pub path_importance_min: Option<f32>,
    #[serde(default)]
    pub path_importance_max: Option<f32>,
    #[serde(default)]
    pub path_importance_depth_base: Option<f32>,
    #[serde(default)]
    pub path_importance_depth_weight: Option<f32>,
    #[serde(default)]
    pub path_importance_depth_min: Option<f32>,
    #[serde(default)]
    pub path_importance_depth_max: Option<f32>,
    #[serde(default)]
    pub path_importance_silhouette_multiplier: Option<f32>,
    #[serde(default)]
    pub path_importance_boundary_multiplier: Option<f32>,
    #[serde(default)]
    pub path_importance_crease_multiplier: Option<f32>,
    #[serde(default)]
    pub path_importance_seam_multiplier: Option<f32>,
    #[serde(default)]
    pub path_importance_feature_multiplier: Option<f32>,
    #[serde(default)]
    pub path_importance_contact_multiplier: Option<f32>,
    #[serde(default)]
    pub region_feature_face_bonus: Option<f32>,
    #[serde(default)]
    pub region_feature_torso_bonus: Option<f32>,
    #[serde(default)]
    pub region_feature_hand_bonus: Option<f32>,
    #[serde(default)]
    pub region_crease_face_bonus: Option<f32>,
    #[serde(default)]
    pub region_crease_torso_bonus: Option<f32>,
    #[serde(default)]
    pub region_crease_hand_bonus: Option<f32>,
    #[serde(default)]
    pub region_seam_torso_bonus: Option<f32>,
    #[serde(default)]
    pub region_seam_hand_bonus: Option<f32>,
    #[serde(default)]
    pub survival_trait_keep_weight: Option<f32>,
    #[serde(default)]
    pub survival_base_keep: Option<f32>,
    #[serde(default)]
    pub survival_length_weight: Option<f32>,
    #[serde(default)]
    pub survival_confidence_weight: Option<f32>,
    #[serde(default)]
    pub survival_chain_bonus_per_edge: Option<f32>,
    #[serde(default)]
    pub survival_chain_bonus_max: Option<f32>,
    #[serde(default)]
    pub survival_cloth_fold_base_bonus: Option<f32>,
    #[serde(default)]
    pub survival_cloth_fold_chain_bonus_per_edge: Option<f32>,
    #[serde(default)]
    pub survival_cloth_fold_chain_bonus_max: Option<f32>,
    #[serde(default)]
    pub survival_detail_material_bonus: Option<f32>,
    #[serde(default)]
    pub survival_detail_plain_bonus: Option<f32>,
    #[serde(default)]
    pub survival_material_cut_seam_bonus: Option<f32>,
    #[serde(default)]
    pub survival_material_cut_plain_bonus: Option<f32>,
    #[serde(default)]
    pub survival_long_form_length_weight: Option<f32>,
    #[serde(default)]
    pub survival_long_form_chain_bonus_per_edge: Option<f32>,
    #[serde(default)]
    pub survival_long_form_chain_bonus_max: Option<f32>,
    #[serde(default)]
    pub survival_continuation_weight: Option<f32>,
    #[serde(default)]
    pub survival_breakup_penalty: Option<f32>,
    #[serde(default)]
    pub isolated_detail_short_ratio: Option<f32>,
    #[serde(default)]
    pub isolated_cloth_fold_short_ratio: Option<f32>,
    #[serde(default)]
    pub isolated_material_cut_short_ratio: Option<f32>,
    #[serde(default)]
    pub min_length_character_readability_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_silhouette_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_boundary_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_contact_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_crease_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_seam_multiplier: Option<f32>,
    #[serde(default)]
    pub min_length_feature_multiplier: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprBreakPolicyProfileDocument {
    #[serde(default)]
    pub allow_seeded_long_feature_breaks: Option<bool>,
    #[serde(default)]
    pub important_feature_break_threshold: Option<f32>,
    #[serde(default)]
    pub long_feature_break_min_length_px: Option<f32>,
    #[serde(default)]
    pub long_feature_break_min_complexity: Option<f32>,
    #[serde(default)]
    pub long_feature_break_chance: Option<f32>,
    #[serde(default)]
    pub long_feature_break_center_t: Option<f32>,
    #[serde(default)]
    pub long_feature_break_center_jitter: Option<f32>,
    #[serde(default)]
    pub long_feature_break_center_min_t: Option<f32>,
    #[serde(default)]
    pub long_feature_break_center_max_t: Option<f32>,
    #[serde(default)]
    pub long_feature_break_min_gap_px: Option<f32>,
    #[serde(default)]
    pub long_feature_break_gap_jitter_px: Option<f32>,
    #[serde(default)]
    pub long_feature_break_half_t_min: Option<f32>,
    #[serde(default)]
    pub long_feature_break_half_t_max: Option<f32>,
    #[serde(default)]
    pub long_feature_break_t0_min: Option<f32>,
    #[serde(default)]
    pub long_feature_break_t0_max: Option<f32>,
    #[serde(default)]
    pub long_feature_break_t1_min: Option<f32>,
    #[serde(default)]
    pub long_feature_break_t1_max: Option<f32>,
    #[serde(default)]
    pub dropout_complexity_edge_limit: Option<f32>,
    #[serde(default)]
    pub dropout_complexity_drop_per_edge: Option<f32>,
    #[serde(default)]
    pub dropout_effective_max: Option<f32>,
    #[serde(default)]
    pub dropout_interval_length_px: Option<f32>,
    #[serde(default)]
    pub dropout_max_intervals: Option<u32>,
    #[serde(default)]
    pub dropout_min_gap_t: Option<f32>,
    #[serde(default)]
    pub dropout_max_gap_t: Option<f32>,
    #[serde(default)]
    pub dropout_edge_margin_t: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprStrokeSynthesisProfileDocument {
    #[serde(default)]
    pub silhouette_pressure: Option<f32>,
    #[serde(default)]
    pub boundary_pressure: Option<f32>,
    #[serde(default)]
    pub feature_pressure: Option<f32>,
    #[serde(default)]
    pub crease_pressure: Option<f32>,
    #[serde(default)]
    pub seam_pressure: Option<f32>,
    #[serde(default)]
    pub contact_pressure: Option<f32>,
    #[serde(default)]
    pub technical_importance_base: Option<f32>,
    #[serde(default)]
    pub technical_candidate_weight: Option<f32>,
    #[serde(default)]
    pub technical_importance_min: Option<f32>,
    #[serde(default)]
    pub technical_importance_max: Option<f32>,
    #[serde(default)]
    pub expressive_importance_min: Option<f32>,
    #[serde(default)]
    pub expressive_importance_max: Option<f32>,
    #[serde(default)]
    pub protected_silhouette_importance_threshold: Option<f32>,
    #[serde(default)]
    pub single_pass_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub single_pass_width_multiplier: Option<f32>,
    #[serde(default)]
    pub single_pass_alpha: Option<f32>,
    #[serde(default)]
    pub dual_primary_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub dual_secondary_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub dual_primary_width_multiplier: Option<f32>,
    #[serde(default)]
    pub dual_secondary_width_multiplier: Option<f32>,
    #[serde(default)]
    pub dual_primary_alpha: Option<f32>,
    #[serde(default)]
    pub dual_secondary_alpha: Option<f32>,
    #[serde(default)]
    pub multi_pass_jitter_base: Option<f32>,
    #[serde(default)]
    pub multi_pass_jitter_step: Option<f32>,
    #[serde(default)]
    pub multi_pass_width_multiplier: Option<f32>,
    #[serde(default)]
    pub multi_pass_alpha: Option<f32>,
    #[serde(default)]
    pub search_wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub search_width_multiplier: Option<f32>,
    #[serde(default)]
    pub hatch_chance_akira: Option<f32>,
    #[serde(default)]
    pub hatch_chance_confident_manga: Option<f32>,
    #[serde(default)]
    pub hatch_chance_generic: Option<f32>,
    #[serde(default)]
    pub hatch_path_length_min_px: Option<f32>,
    #[serde(default)]
    pub hatch_path_length_max_px: Option<f32>,
    #[serde(default)]
    pub hatch_center_t: Option<f32>,
    #[serde(default)]
    pub hatch_center_jitter: Option<f32>,
    #[serde(default)]
    pub hatch_length_min_px: Option<f32>,
    #[serde(default)]
    pub hatch_length_jitter_px: Option<f32>,
    #[serde(default)]
    pub hatch_half_t_min: Option<f32>,
    #[serde(default)]
    pub hatch_half_t_max: Option<f32>,
    #[serde(default)]
    pub hatch_wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub hatch_width_multiplier: Option<f32>,
    #[serde(default)]
    pub hatch_alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub hatch_alpha_max: Option<f32>,
    #[serde(default)]
    pub short_detail_boost: Option<f32>,
    #[serde(default)]
    pub short_detail_threshold_px: Option<f32>,
    #[serde(default)]
    pub medium_detail_boost: Option<f32>,
    #[serde(default)]
    pub medium_detail_threshold_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprTessellationProfileDocument {
    #[serde(default)]
    pub rail_tangent_smoothing: Option<bool>,
    #[serde(default)]
    pub kink_fallback_dot: Option<f32>,
    #[serde(default)]
    pub resample_spacing_px: Option<f32>,
    #[serde(default)]
    pub endpoint_lock_max_t: Option<f32>,
    #[serde(default)]
    pub taper_endpoint_floor: Option<f32>,
    #[serde(default)]
    pub pass_wobble_max_px: Option<f32>,
    #[serde(default)]
    pub angle_alpha_influence: Option<f32>,
    #[serde(default)]
    pub min_sample_width_px: Option<f32>,
    #[serde(default)]
    pub long_stroke_detail_crispness: Option<f32>,
    #[serde(default)]
    pub hand_arc_length_min: Option<f32>,
    #[serde(default)]
    pub hand_arc_length_max: Option<f32>,
    #[serde(default)]
    pub hand_arc_scale: Option<f32>,
    #[serde(default)]
    pub preferred_length_floor_px: Option<f32>,
    #[serde(default)]
    pub primary_noise_frequency_scale: Option<f32>,
    #[serde(default)]
    pub hand_arc_noise_frequency_scale: Option<f32>,
    #[serde(default)]
    pub hand_arc_noise_phase: Option<f32>,
    #[serde(default)]
    pub tangent_drift_noise_frequency_scale: Option<f32>,
    #[serde(default)]
    pub tangent_drift_noise_phase: Option<f32>,
    #[serde(default)]
    pub micro_noise_frequency_scale: Option<f32>,
    #[serde(default)]
    pub micro_noise_phase: Option<f32>,
    #[serde(default)]
    pub width_noise_frequency_scale: Option<f32>,
    #[serde(default)]
    pub width_noise_phase: Option<f32>,
    #[serde(default)]
    pub bow_min_length_px: Option<f32>,
    #[serde(default)]
    pub bow_preferred_min_px: Option<f32>,
    #[serde(default)]
    pub bow_length_min: Option<f32>,
    #[serde(default)]
    pub bow_length_max: Option<f32>,
    #[serde(default)]
    pub bow_wobble_floor_px: Option<f32>,
    #[serde(default)]
    pub bow_scale: Option<f32>,
    #[serde(default)]
    pub bow_non_feature_factor: Option<f32>,
    #[serde(default)]
    pub bow_max_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprBrushProfileDocument {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tip: Option<String>,
    #[serde(default)]
    pub ink_color: Option<String>,
    #[serde(default)]
    pub width_multiplier: Option<f32>,
    #[serde(default)]
    pub alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub pressure_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub dropout_multiplier: Option<f32>,
    #[serde(default)]
    pub search_multiplier: Option<f32>,
    #[serde(default)]
    pub path_wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub micro_wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub hand_arc_multiplier: Option<f32>,
    #[serde(default)]
    pub tangent_drift_multiplier: Option<f32>,
    #[serde(default)]
    pub detail_crispness_multiplier: Option<f32>,
    #[serde(default)]
    pub taper_multiplier: Option<f32>,
    #[serde(default)]
    pub overshoot_px: Option<f32>,
    #[serde(default)]
    pub width_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub alpha_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub angle_bias_degrees: Option<f32>,
    #[serde(default)]
    pub angle_influence: Option<f32>,
    #[serde(default)]
    pub nib_width_base_scale: Option<f32>,
    #[serde(default)]
    pub nib_width_angle_scale: Option<f32>,
    #[serde(default)]
    pub path_adherence_multiplier: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLineFamilyDocument {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default = "default_bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub brush: Option<String>,
    #[serde(default)]
    pub preferred_stroke_length_px: Option<f32>,
    #[serde(default)]
    pub stroke_join_gap_px: Option<f32>,
    #[serde(default)]
    pub stroke_join_max_angle_degrees: Option<f32>,
    #[serde(default)]
    pub technical_detail_keep: Option<f32>,
    #[serde(default)]
    pub min_screen_length_px: Option<f32>,
    #[serde(default)]
    pub min_stroke_length_px: Option<f32>,
    #[serde(default)]
    pub technical_detail_preference: Option<f32>,
    #[serde(default)]
    pub ink_detail_material_preference: Option<f32>,
    #[serde(default)]
    pub material_seam_preference: Option<f32>,
    #[serde(default)]
    pub continuation_bias: Option<f32>,
    #[serde(default)]
    pub breakup_bias: Option<f32>,
    #[serde(default)]
    pub width_multiplier: Option<f32>,
    #[serde(default)]
    pub alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub taper_multiplier: Option<f32>,
    #[serde(default)]
    pub overshoot_px: Option<f32>,
}

fn default_bool_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NprPreset3dDocument {
    Ref(String),
    Source { source: String },
    Definition(NprPreset3dDefinitionDocument),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NprPreset3dDefinitionDocument {
    pub id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub settings: NprLine3dSettingsDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dKindOverrideDocument {
    #[serde(default)]
    pub width_multiplier: Option<f32>,
    #[serde(default)]
    pub wobble_px: Option<f32>,
    #[serde(default)]
    pub dropout: Option<f32>,
    #[serde(default)]
    pub taper: Option<f32>,
    #[serde(default)]
    pub overshoot_px: Option<f32>,
    #[serde(default)]
    pub alpha_multiplier: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dToolDocument {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub base_width_px: Option<f32>,
    #[serde(default)]
    pub base_alpha: Option<f32>,
    #[serde(default)]
    pub width_multiplier: Option<f32>,
    #[serde(default)]
    pub alpha_multiplier: Option<f32>,
    #[serde(default)]
    pub wobble_multiplier: Option<f32>,
    #[serde(default)]
    pub pressure_jitter_multiplier: Option<f32>,
    #[serde(default)]
    pub dropout_multiplier: Option<f32>,
    #[serde(default)]
    pub search_multiplier: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dTrajectoryDocument {
    #[serde(default)]
    pub path_adherence: Option<f32>,
    #[serde(default)]
    pub straightness: Option<f32>,
    #[serde(default)]
    pub humanization: Option<f32>,
    #[serde(default)]
    pub gesture_offset_px: Option<f32>,
    #[serde(default)]
    pub gesture_frequency_per_100px: Option<f32>,
    #[serde(default)]
    pub micro_offset_px: Option<f32>,
    #[serde(default)]
    pub micro_frequency_per_100px: Option<f32>,
    #[serde(default)]
    pub angular_drift_degrees: Option<f32>,
    #[serde(default)]
    pub endpoint_snap_px: Option<f32>,
    #[serde(default)]
    pub path_simplify_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dPressureDocument {
    #[serde(default)]
    pub width_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub jitter: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dOpacityDocument {
    #[serde(default)]
    pub alpha_curve: Option<[f32; 4]>,
    #[serde(default)]
    pub base_alpha: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dEndpointsDocument {
    #[serde(default)]
    pub taper: Option<f32>,
    #[serde(default)]
    pub lock_start_px: Option<f32>,
    #[serde(default)]
    pub lock_end_px: Option<f32>,
    #[serde(default)]
    pub overshoot_start_px: Option<f32>,
    #[serde(default)]
    pub overshoot_end_px: Option<f32>,
    #[serde(default)]
    pub overshoot_px: Option<f32>,
    #[serde(default)]
    pub undershoot_end_px: Option<f32>,
    #[serde(default)]
    pub undershoot_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dBreakupDocument {
    #[serde(default)]
    pub amount: Option<f32>,
    #[serde(default)]
    pub dropout: Option<f32>,
    #[serde(default)]
    pub min_gap_px: Option<f32>,
    #[serde(default)]
    pub min_visible_segment_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dPerformanceDocument {
    #[serde(default)]
    pub visibility_max_dimension_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dDepthDocument {
    #[serde(default)]
    pub width_influence: Option<f32>,
    #[serde(default)]
    pub alpha_influence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dConfidenceDocument {
    #[serde(default)]
    pub line_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dPassesDocument {
    #[serde(default)]
    pub primary_count: Option<u8>,
    #[serde(default)]
    pub search_count: Option<u8>,
    #[serde(default)]
    pub search_alpha: Option<f32>,
    #[serde(default)]
    pub search_offset_px: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NprLine3dPassesFieldDocument {
    Count(u8),
    Plan(NprLine3dPassesDocument),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NprLine3dClassOverridesDocument {
    #[serde(default)]
    pub silhouette: Option<NprLine3dKindOverrideDocument>,
    #[serde(default)]
    pub boundary: Option<NprLine3dKindOverrideDocument>,
    #[serde(default)]
    pub feature: Option<NprLine3dKindOverrideDocument>,
}

fn default_camera_controller_3d_mode() -> String {
    "orbit".to_owned()
}

fn default_camera_controller_3d_distance() -> f32 {
    6.5
}

fn default_camera_controller_3d_min_distance() -> f32 {
    1.2
}

fn default_camera_controller_3d_max_distance() -> f32 {
    18.0
}

fn default_camera_controller_3d_pitch() -> f32 {
    0.12
}

fn default_camera_controller_3d_sensitivity() -> f32 {
    0.006
}

fn default_camera_controller_3d_pan_sensitivity() -> f32 {
    0.0012
}

fn default_camera_controller_3d_zoom_speed() -> f32 {
    0.035
}

fn default_camera_controller_3d_freelook_speed() -> f32 {
    3.8
}

fn default_camera_controller_3d_freelook_sensitivity() -> f32 {
    0.004
}

fn default_camera_controller_3d_freelook_fast_multiplier() -> f32 {
    3.0
}

fn default_camera_controller_3d_move_forward_action() -> String {
    "npr.fly_forward".to_owned()
}

fn default_camera_controller_3d_move_strafe_action() -> String {
    "npr.fly_strafe".to_owned()
}

fn default_camera_controller_3d_move_lift_action() -> String {
    "npr.fly_lift".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SceneComponentDocumentModel {
    #[serde(rename = "Camera3D")]
    Camera3d {
        #[serde(default = "default_camera3d_fov_y_degrees")]
        fov_y_degrees: f32,
        #[serde(default = "default_camera3d_near_clip")]
        near_clip: f32,
        #[serde(default = "default_camera3d_far_clip")]
        far_clip: f32,
        #[serde(default)]
        background_color: Option<String>,
    },
    #[serde(rename = "CameraController3D")]
    CameraController3d {
        camera: String,
        #[serde(default = "default_camera_controller_3d_mode")]
        mode: String,
        #[serde(default)]
        switch_action: Option<String>,
        #[serde(default)]
        orbit_target: Option<String>,
        #[serde(default = "default_camera_controller_3d_distance")]
        orbit_distance: f32,
        #[serde(default = "default_camera_controller_3d_min_distance")]
        orbit_min_distance: f32,
        #[serde(default = "default_camera_controller_3d_max_distance")]
        orbit_max_distance: f32,
        #[serde(default)]
        orbit_yaw: f32,
        #[serde(default = "default_camera_controller_3d_pitch")]
        orbit_pitch: f32,
        #[serde(default = "default_camera_controller_3d_sensitivity")]
        orbit_sensitivity: f32,
        #[serde(default = "default_camera_controller_3d_pan_sensitivity")]
        orbit_pan_sensitivity: f32,
        #[serde(default = "default_camera_controller_3d_zoom_speed")]
        orbit_zoom_speed: f32,
        #[serde(default = "default_camera_controller_3d_freelook_speed")]
        freelook_speed: f32,
        #[serde(default = "default_camera_controller_3d_freelook_sensitivity")]
        freelook_sensitivity: f32,
        #[serde(default = "default_camera_controller_3d_freelook_fast_multiplier")]
        freelook_fast_multiplier: f32,
        #[serde(default = "default_camera_controller_3d_move_forward_action")]
        move_forward_action: String,
        #[serde(default = "default_camera_controller_3d_move_strafe_action")]
        move_strafe_action: String,
        #[serde(default = "default_camera_controller_3d_move_lift_action")]
        move_lift_action: String,
    },
    #[serde(rename = "Light3D")]
    Light3d {
        #[serde(default)]
        kind: String,
        #[serde(default = "default_light3d_direction")]
        direction: SceneVec3Document,
        #[serde(default)]
        color: Option<String>,
        #[serde(default = "default_light3d_intensity")]
        intensity: f32,
        #[serde(default = "default_light3d_ambient")]
        ambient: f32,
    },
    #[serde(rename = "LightMap2DSource")]
    LightMap2dSource {
        id: String,
        source: LightMap2dSourceRefDocument,
        #[serde(default)]
        channels: Vec<LightMap2dChannelDocument>,
    },
    #[serde(rename = "EntityPool")]
    EntityPool {
        #[serde(default)]
        pool: Option<String>,
        members: Vec<String>,
    },
    #[serde(rename = "Lifetime")]
    Lifetime {
        seconds: f32,
        outcome: SceneLifetimeExpirationOutcomeDocument,
        #[serde(default)]
        pool: Option<String>,
    },
    #[serde(rename = "ProjectileEmitter2D")]
    ProjectileEmitter2d {
        pool: String,
        speed: f32,
        #[serde(default = "default_vec2_zero")]
        spawn_offset: SceneVec2Document,
        #[serde(default)]
        inherit_velocity_scale: f32,
    },
    #[serde(rename = "InputActionMap")]
    InputActionMap {
        id: String,
        #[serde(default)]
        active: bool,
        #[serde(default)]
        actions: BTreeMap<String, SceneInputActionBindingDocument>,
    },
    #[serde(rename = "Behavior")]
    Behavior {
        #[serde(default)]
        enabled_when: Option<SceneBehaviorConditionDocument>,
        #[serde(flatten)]
        behavior: SceneBehaviorDocument,
    },
    #[serde(rename = "EventPipeline")]
    EventPipeline {
        id: String,
        topic: String,
        #[serde(default)]
        steps: Vec<SceneEventPipelineStepDocument>,
    },
    #[serde(rename = "UiModelBindings")]
    UiModelBindings {
        #[serde(default)]
        bindings: Vec<SceneUiModelBindingDocument>,
    },
    #[serde(rename = "ScriptComponent")]
    ScriptComponent {
        script: String,
        #[serde(default)]
        params: BTreeMap<String, ScenePropertyValueDocument>,
    },
    #[serde(rename = "Velocity2D")]
    Velocity2d {
        #[serde(default = "default_vec2_zero")]
        velocity: SceneVec2Document,
    },
    #[serde(rename = "Bounds2D")]
    Bounds2d {
        min: SceneVec2Document,
        max: SceneVec2Document,
        behavior: SceneBoundsBehavior2dDocument,
        #[serde(default = "default_bounds_restitution")]
        restitution: f32,
    },
    #[serde(rename = "FreeflightMotion2D")]
    FreeflightMotion2d {
        thrust_acceleration: f32,
        reverse_acceleration: f32,
        strafe_acceleration: f32,
        turn_acceleration: f32,
        linear_damping: f32,
        turn_damping: f32,
        max_speed: f32,
        max_angular_speed: f32,
        #[serde(default = "default_vec2_zero")]
        initial_velocity: SceneVec2Document,
        #[serde(default)]
        initial_angular_velocity: f32,
        #[serde(default)]
        thrust_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        reverse_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        strafe_response_curve: Option<Curve1dSceneDocument>,
        #[serde(default)]
        turn_response_curve: Option<Curve1dSceneDocument>,
    },
    #[serde(rename = "KinematicBody2D")]
    KinematicBody2d {
        #[serde(default = "default_vec2_zero")]
        velocity: SceneVec2Document,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        terminal_velocity: f32,
    },
    #[serde(rename = "AabbCollider2D")]
    AabbCollider2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
        #[serde(default)]
        mask: Vec<String>,
    },
    #[serde(rename = "StaticCollider2D")]
    StaticCollider2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
    },
    #[serde(rename = "CircleCollider2D")]
    CircleCollider2d {
        radius: f32,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
    },
    #[serde(rename = "Trigger2D")]
    Trigger2d {
        size: SceneVec2Document,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        layer: String,
        #[serde(default)]
        mask: Vec<String>,
        #[serde(default)]
        event: Option<String>,
    },
    #[serde(rename = "MotionController2D")]
    MotionController2d {
        max_speed: f32,
        acceleration: f32,
        deceleration: f32,
        air_acceleration: f32,
        gravity: f32,
        jump_velocity: f32,
        terminal_velocity: f32,
    },
    #[serde(rename = "CameraFollow2D")]
    CameraFollow2d {
        target: String,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
        #[serde(default = "default_camera_follow_lerp")]
        lerp: f32,
        #[serde(default)]
        lookahead_velocity_scale: f32,
        #[serde(default)]
        lookahead_max_distance: f32,
        #[serde(default)]
        sway_amount: f32,
        #[serde(default)]
        sway_frequency: f32,
    },
    #[serde(rename = "Parallax2D")]
    Parallax2d {
        camera: String,
        factor: SceneVec2Document,
    },
    #[serde(rename = "TileMapMarker2D")]
    TileMapMarker2d {
        symbol: String,
        #[serde(default)]
        tilemap_entity: Option<String>,
        #[serde(default)]
        index: usize,
        #[serde(default = "default_vec2_zero")]
        offset: SceneVec2Document,
    },
    #[serde(rename = "Mesh3D")]
    Mesh3d {
        mesh: String,
        #[serde(default)]
        npr: Option<NprLine3dDocument>,
    },
    #[serde(rename = "Material3D")]
    Material3d {
        label: String,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        albedo: Option<String>,
        #[serde(default)]
        render_order: i32,
        #[serde(default)]
        shading: Option<String>,
    },
    #[serde(rename = "Text3D")]
    Text3d {
        content: String,
        font: String,
        size: f32,
    },
    #[serde(rename = "PhysicsWorld3D")]
    PhysicsWorld3d {
        #[serde(default = "default_physics3d_gravity")]
        gravity: SceneVec3Document,
        #[serde(default = "default_physics3d_substeps")]
        substeps: u32,
        #[serde(default = "default_physics3d_solver_iterations")]
        solver_iterations: u32,
        #[serde(default = "default_physics3d_ccd_substeps")]
        ccd_substeps: u32,
    },
    #[serde(rename = "RigidBody3D")]
    RigidBody3d {
        #[serde(default = "default_vec3_zero")]
        velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity: SceneVec3Document,
        #[serde(default = "default_rigid_body_mass_3d")]
        mass: f32,
        #[serde(default = "default_linear_damping_3d")]
        linear_damping: f32,
        #[serde(default = "default_angular_damping_3d")]
        angular_damping: f32,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        restitution: f32,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        ccd: bool,
    },
    #[serde(rename = "BoxCollider3D")]
    BoxCollider3d {
        size: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        offset: SceneVec3Document,
    },
    #[serde(rename = "StaticBoxCollider3D")]
    StaticBoxCollider3d {
        size: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        offset: SceneVec3Document,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        restitution: f32,
    },
    #[serde(rename = "PhysicsSpawner3D")]
    PhysicsSpawner3d {
        entity_prefix: String,
        mesh: String,
        material: String,
        #[serde(default)]
        material_label: Option<String>,
        #[serde(default = "default_spawn_interval_seconds")]
        interval_seconds: f32,
        #[serde(default = "default_vec3_zero")]
        origin: SceneVec3Document,
        #[serde(default = "default_vec3_one")]
        spawn_scale: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        grid_spacing: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        initial_velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        spawn_position_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        spawn_rotation_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        initial_velocity_jitter: SceneVec3Document,
        #[serde(default = "default_vec3_zero")]
        angular_velocity_jitter: SceneVec3Document,
        #[serde(default = "default_rigid_body_mass_3d")]
        mass: f32,
        #[serde(default = "default_linear_damping_3d")]
        linear_damping: f32,
        #[serde(default = "default_angular_damping_3d")]
        angular_damping: f32,
        #[serde(default = "default_gravity_scale")]
        gravity_scale: f32,
        #[serde(default)]
        restitution: f32,
        #[serde(default = "default_rigid_body_friction_3d")]
        friction: f32,
        #[serde(default)]
        ccd: bool,
        #[serde(default = "default_vec3_one")]
        collider_size: SceneVec3Document,
        #[serde(default)]
        max_alive: u32,
        #[serde(default)]
        counter_entity: Option<String>,
        #[serde(default = "default_physics3d_counter_prefix")]
        counter_prefix: String,
        #[serde(default)]
        counter_font: Option<String>,
        #[serde(default = "default_physics3d_counter_size")]
        counter_size: f32,
        #[serde(default = "default_vec3_zero")]
        counter_position: SceneVec3Document,
    },
    #[serde(rename = "UiDocument")]
    UiDocument {
        target: SceneUiTargetComponentDocument,
        root: SceneUiNodeComponentDocument,
    },
    #[serde(rename = "UiThemeSet")]
    UiThemeSet {
        #[serde(default)]
        active: Option<String>,
        themes: Vec<SceneUiThemeComponentDocument>,
    },
    Plugin {
        component_type: String,
        payload: Value,
    },
}

pub type SceneComponentDocument = SceneComponentDocumentModel;

pub fn is_builtin_component_type(kind: &str) -> bool {
    matches!(
        kind,
        "Camera3D"
            | "CameraController3D"
            | "Light3D"
            | "LightMap2DSource"
            | "EntityPool"
            | "Lifetime"
            | "ProjectileEmitter2D"
            | "InputActionMap"
            | "Behavior"
            | "EventPipeline"
            | "UiModelBindings"
            | "ScriptComponent"
            | "Velocity2D"
            | "Bounds2D"
            | "FreeflightMotion2D"
            | "KinematicBody2D"
            | "AabbCollider2D"
            | "StaticCollider2D"
            | "CircleCollider2D"
            | "Trigger2D"
            | "MotionController2D"
            | "CameraFollow2D"
            | "Parallax2D"
            | "TileMapMarker2D"
            | "Mesh3D"
            | "Material3D"
            | "Text3D"
            | "PhysicsWorld3D"
            | "RigidBody3D"
            | "BoxCollider3D"
            | "StaticBoxCollider3D"
            | "PhysicsSpawner3D"
            | "UiDocument"
            | "UiThemeSet"
    )
}

pub fn is_rejected_retired_component_type(kind: &str) -> bool {
    matches!(kind, "PlatformerController2D")
}

pub fn plugin_component_document(component_type: String, payload: Value) -> SceneComponentDocument {
    type ComponentDocument = SceneComponentDocument;

    ComponentDocument::Plugin {
        component_type,
        payload,
    }
}

impl SceneComponentDocument {
    pub fn kind(&self) -> &str {
        match self {
            Self::Camera3d { .. } => "Camera3D",
            Self::CameraController3d { .. } => "CameraController3D",
            Self::Light3d { .. } => "Light3D",
            Self::LightMap2dSource { .. } => "LightMap2DSource",
            Self::EntityPool { .. } => "EntityPool",
            Self::Lifetime { .. } => "Lifetime",
            Self::ProjectileEmitter2d { .. } => "ProjectileEmitter2D",
            Self::InputActionMap { .. } => "InputActionMap",
            Self::Behavior { .. } => "Behavior",
            Self::EventPipeline { .. } => "EventPipeline",
            Self::UiModelBindings { .. } => "UiModelBindings",
            Self::ScriptComponent { .. } => "ScriptComponent",
            Self::Velocity2d { .. } => "Velocity2D",
            Self::Bounds2d { .. } => "Bounds2D",
            Self::FreeflightMotion2d { .. } => "FreeflightMotion2D",
            Self::KinematicBody2d { .. } => "KinematicBody2D",
            Self::AabbCollider2d { .. } => "AabbCollider2D",
            Self::StaticCollider2d { .. } => "StaticCollider2D",
            Self::CircleCollider2d { .. } => "CircleCollider2D",
            Self::Trigger2d { .. } => "Trigger2D",
            Self::MotionController2d { .. } => "MotionController2D",
            Self::CameraFollow2d { .. } => "CameraFollow2D",
            Self::Parallax2d { .. } => "Parallax2D",
            Self::TileMapMarker2d { .. } => "TileMapMarker2D",
            Self::Mesh3d { .. } => "Mesh3D",
            Self::Material3d { .. } => "Material3D",
            Self::Text3d { .. } => "Text3D",
            Self::PhysicsWorld3d { .. } => "PhysicsWorld3D",
            Self::RigidBody3d { .. } => "RigidBody3D",
            Self::BoxCollider3d { .. } => "BoxCollider3D",
            Self::StaticBoxCollider3d { .. } => "StaticBoxCollider3D",
            Self::PhysicsSpawner3d { .. } => "PhysicsSpawner3D",
            Self::UiDocument { .. } => "UiDocument",
            Self::UiThemeSet { .. } => "UiThemeSet",
            Self::Plugin { component_type, .. } => component_type.as_str(),
        }
    }

    pub fn primary_render_layer(&self) -> Option<&str> {
        None
    }

    pub fn plugin_payload(&self) -> Option<(&str, &Value)> {
        match self {
            Self::Plugin {
                component_type,
                payload,
            } => Some((component_type.as_str(), payload)),
            _ => None,
        }
    }

    pub fn post_fx_documents(&self) -> Option<&[PostFx2dDocument]> {
        None
    }

    pub fn layered_image_part_post_fx_documents(&self) -> Option<Vec<(&str, &[PostFx2dDocument])>> {
        None
    }

    pub fn is_particle_emitter_2d(&self) -> bool {
        false
    }

    pub fn semantic_class(&self) -> SceneComponentSemanticClass {
        match self {
            Self::CameraFollow2d { .. } => SceneComponentSemanticClass::Camera2d,
            Self::Velocity2d { .. }
            | Self::FreeflightMotion2d { .. }
            | Self::MotionController2d { .. } => SceneComponentSemanticClass::Motion2d,
            Self::KinematicBody2d { .. }
            | Self::AabbCollider2d { .. }
            | Self::StaticCollider2d { .. }
            | Self::CircleCollider2d { .. }
            | Self::Trigger2d { .. } => SceneComponentSemanticClass::Physics2d,
            Self::PhysicsWorld3d { .. }
            | Self::RigidBody3d { .. }
            | Self::BoxCollider3d { .. }
            | Self::StaticBoxCollider3d { .. }
            | Self::PhysicsSpawner3d { .. } => SceneComponentSemanticClass::Physics3d,
            Self::ScriptComponent { .. } => SceneComponentSemanticClass::Script,
            Self::Plugin { .. } => SceneComponentSemanticClass::Plugin,
            _ => SceneComponentSemanticClass::Generic2d,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TileMap2dEditorDocument {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub lock_size: bool,
    #[serde(default)]
    pub show_grid: bool,
    #[serde(default)]
    pub default_brush: Option<String>,
    #[serde(default)]
    pub snap: Option<String>,
    #[serde(default)]
    pub palette: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayeredImageBlendMode2dDocument {
    Alpha,
    Additive,
    Screen,
    Multiply,
    Lighten,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LayeredImageViewportFit2dDocument {
    #[default]
    Fixed,
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepthAuxMap2dChannelsDocument {
    #[serde(default = "default_depth_aux_r_channel")]
    pub r: String,
    #[serde(default = "default_depth_aux_g_channel")]
    pub g: String,
    #[serde(default = "default_depth_aux_b_channel")]
    pub b: String,
    #[serde(default = "default_depth_aux_a_channel")]
    pub a: String,
}

impl Default for DepthAuxMap2dChannelsDocument {
    fn default() -> Self {
        Self {
            r: default_depth_aux_r_channel(),
            g: default_depth_aux_g_channel(),
            b: default_depth_aux_b_channel(),
            a: default_depth_aux_a_channel(),
        }
    }
}

fn default_depth_aux_r_channel() -> String {
    "auxiliary_depth".to_owned()
}

fn default_depth_aux_g_channel() -> String {
    "local_height".to_owned()
}

fn default_depth_aux_b_channel() -> String {
    "occluder_strength".to_owned()
}

fn default_depth_aux_a_channel() -> String {
    "valid_mask".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayeredImageLayerOverrideDocument {
    pub id: String,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub blend: Option<LayeredImageBlendMode2dDocument>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_maps: Option<VisualMaps2dDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_fx: Vec<PostFx2dDocument>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VisualMaps2dDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wetness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissive: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LightMap2dSourceRefDocument {
    #[serde(rename = "layered_image_2d", alias = "layered_image2d")]
    LayeredImage2d { entity: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightMap2dChannelDocument {
    pub id: String,
    #[serde(default)]
    pub layers: Vec<String>,
}
use std::collections::BTreeMap;
