use amigo_math::Transform3;

use crate::renderer::{
    CachedMeshGeometry3d, GpuNprFrameUniforms3d, NprDebugOverlay3d, NprLineKind, Viewport,
};

use super::{
    camera_response_distances, gpu_budget_strategy, gpu_candidate_strategy, gpu_fill_strategy,
    gpu_hatching_strategy, gpu_material_id_mask, gpu_path_strategy, gpu_stroke_strategy,
    gpu_temporal_strategy, npr_ink_detail_material_preference_for_kind,
    npr_line_family_role_for_kind, npr_min_stroke_length_px_for_kind,
    npr_material_seam_preference_for_kind, npr_min_screen_length_px_for_kind,
    npr_preferred_stroke_length_px_for_kind, npr_stroke_join_gap_px_for_kind,
    npr_stroke_join_max_angle_degrees_for_kind, npr_technical_detail_keep_for_kind,
    npr_technical_detail_preference_for_kind, vec3_to_gpu4,
};

pub(crate) fn uniforms_are_finite(uniforms: &GpuNprFrameUniforms3d) -> bool {
    uniforms
        .model_translation
        .iter()
        .all(|value| value.is_finite())
        && uniforms
            .model_rotation
            .iter()
            .all(|value| value.is_finite())
        && uniforms.model_scale.iter().all(|value| value.is_finite())
        && uniforms
            .camera_translation
            .iter()
            .all(|value| value.is_finite())
        && uniforms
            .camera_rotation
            .iter()
            .all(|value| value.is_finite())
        && uniforms.viewport_half.iter().all(|value| value.is_finite())
        && uniforms.params0.iter().all(|value| value.is_finite())
        && uniforms.params1.iter().all(|value| value.is_finite())
        && uniforms.params2.iter().all(|value| value.is_finite())
        && uniforms.params3.iter().all(|value| value.is_finite())
        && uniforms.params4.iter().all(|value| value.is_finite())
        && uniforms.params5.iter().all(|value| value.is_finite())
        && uniforms.params6.iter().all(|value| value.is_finite())
        && uniforms.params7.iter().all(|value| value.is_finite())
        && uniforms.params8.iter().all(|value| value.is_finite())
        && uniforms.params9.iter().all(|value| value.is_finite())
        && uniforms.params10.iter().all(|value| value.is_finite())
        && uniforms.params11.iter().all(|value| value.is_finite())
        && uniforms.params12.iter().all(|value| value.is_finite())
        && uniforms.params13.iter().all(|value| value.is_finite())
        && uniforms.params14.iter().all(|value| value.is_finite())
        && uniforms.params15.iter().all(|value| value.is_finite())
        && uniforms.params16.iter().all(|value| value.is_finite())
        && uniforms.params17.iter().all(|value| value.is_finite())
        && uniforms.params18.iter().all(|value| value.is_finite())
        && uniforms.params19.iter().all(|value| value.is_finite())
        && uniforms.params20.iter().all(|value| value.is_finite())
        && uniforms.params21.iter().all(|value| value.is_finite())
        && uniforms.params22.iter().all(|value| value.is_finite())
        && uniforms.params23.iter().all(|value| value.is_finite())
        && uniforms.params24.iter().all(|value| value.is_finite())
        && uniforms.params25.iter().all(|value| value.is_finite())
        && uniforms.params26.iter().all(|value| value.is_finite())
        && uniforms.params27.iter().all(|value| value.is_finite())
        && uniforms.params28.iter().all(|value| value.is_finite())
        && uniforms.params29.iter().all(|value| value.is_finite())
        && uniforms.params30.iter().all(|value| value.is_finite())
        && uniforms.params31.iter().all(|value| value.is_finite())
        && uniforms.params32.iter().all(|value| value.is_finite())
        && uniforms.params33.iter().all(|value| value.is_finite())
        && uniforms.params34.iter().all(|value| value.is_finite())
        && uniforms.params35.iter().all(|value| value.is_finite())
        && uniforms.params36.iter().all(|value| value.is_finite())
        && uniforms.params37.iter().all(|value| value.is_finite())
        && uniforms.params38.iter().all(|value| value.is_finite())
        && uniforms.params39.iter().all(|value| value.is_finite())
        && uniforms.params40.iter().all(|value| value.is_finite())
        && uniforms.params41.iter().all(|value| value.is_finite())
        && uniforms.params42.iter().all(|value| value.is_finite())
        && uniforms.params43.iter().all(|value| value.is_finite())
        && uniforms.params44.iter().all(|value| value.is_finite())
        && uniforms.params45.iter().all(|value| value.is_finite())
        && uniforms.params46.iter().all(|value| value.is_finite())
        && uniforms.params47.iter().all(|value| value.is_finite())
        && uniforms.params48.iter().all(|value| value.is_finite())
        && uniforms.params49.iter().all(|value| value.is_finite())
        && uniforms.params50.iter().all(|value| value.is_finite())
        && uniforms.params51.iter().all(|value| value.is_finite())
        && uniforms.params52.iter().all(|value| value.is_finite())
        && uniforms.params53.iter().all(|value| value.is_finite())
        && uniforms.params54.iter().all(|value| value.is_finite())
        && uniforms.ink_color.iter().all(|value| value.is_finite())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn uniforms_for_job(
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
    face_id_base: u32,
    path_segment_base: u32,
    path_segment_slot_count: u32,
    _vertex_count: u32,
    _triangle_count: u32,
    edge_count: u32,
    overlay: Option<NprDebugOverlay3d>,
) -> GpuNprFrameUniforms3d {
    let silhouette_style = super::resolve_npr_kind_style(NprLineKind::Silhouette, settings);
    let boundary_style = super::resolve_npr_kind_style(NprLineKind::Boundary, settings);
    let feature_style = super::resolve_npr_kind_style(NprLineKind::Feature, settings);
    let seam_style = super::resolve_npr_kind_style(NprLineKind::Seam, settings);
    let gpu_tuning = settings.gpu_realtime_tuning.normalized();
    let camera_response = settings.camera_response.normalized();
    let pipeline_plan = settings.pipeline_plan();
    let (camera_near_distance, camera_far_distance, camera_focus_distance01) =
        camera_response_distances(camera_response, geometry, camera, transform);
    let silhouette_brush = super::resolve_npr_brush_profile(NprLineKind::Silhouette, settings);
    let boundary_brush = super::resolve_npr_brush_profile(NprLineKind::Boundary, settings);
    let feature_brush = super::resolve_npr_brush_profile(NprLineKind::Feature, settings);
    let seam_brush = super::resolve_npr_brush_profile(NprLineKind::Seam, settings);
    let primary_passes = settings.passes.min(8).max(1) as f32;
    let search_passes = if gpu_tuning.search_enabled {
        ((settings.search_line_count as f32) * feature_brush.search_multiplier)
            .round()
            .clamp(0.0, 8.0)
    } else {
        0.0
    };
    let micro_wobble = settings.micro_wobble_px
        * settings.humanization
        * feature_brush.path_wobble_multiplier
        * feature_brush.micro_wobble_multiplier;
    let overlay_mode = match settings.gpu_realtime_tuning.debug_mode {
        amigo_render_api::NprGpuDebugMode3d::Final => match overlay {
            Some(NprDebugOverlay3d::LineKinds) => 1.0,
            Some(NprDebugOverlay3d::RawPaths) => 2.0,
            Some(NprDebugOverlay3d::CandidateImportance) => 6.0,
            Some(NprDebugOverlay3d::TechnicalSelection) => 7.0,
            Some(NprDebugOverlay3d::StrokeLengthBucket) => 8.0,
            Some(NprDebugOverlay3d::SourceEdgeCount) => 9.0,
            Some(NprDebugOverlay3d::Dropout) => 3.0,
            Some(NprDebugOverlay3d::WidthAlpha) => 4.0,
            None => 0.0,
        },
        amigo_render_api::NprGpuDebugMode3d::LineKinds => 1.0,
        amigo_render_api::NprGpuDebugMode3d::RawPaths => 2.0,
        amigo_render_api::NprGpuDebugMode3d::Dropout => 3.0,
        amigo_render_api::NprGpuDebugMode3d::WidthAlpha => 4.0,
        amigo_render_api::NprGpuDebugMode3d::ChainHops => 5.0,
        amigo_render_api::NprGpuDebugMode3d::CandidateImportance => 6.0,
        amigo_render_api::NprGpuDebugMode3d::TechnicalSelection => 7.0,
        amigo_render_api::NprGpuDebugMode3d::StrokeLengthBucket => 8.0,
        amigo_render_api::NprGpuDebugMode3d::SourceEdgeCount => 9.0,
        amigo_render_api::NprGpuDebugMode3d::StrokeRoles => 10.0,
        amigo_render_api::NprGpuDebugMode3d::MaterialRoles => 11.0,
    };
    GpuNprFrameUniforms3d {
        model_translation: vec3_to_gpu4(transform.translation, 0.0),
        model_rotation: vec3_to_gpu4(transform.rotation_euler, 0.0),
        model_scale: vec3_to_gpu4(transform.scale, 0.0),
        camera_translation: vec3_to_gpu4(camera.translation, 0.0),
        camera_rotation: vec3_to_gpu4(camera.rotation_euler, 0.0),
        viewport_half: [
            viewport.size().x * 0.5,
            viewport.size().y * 0.5,
            viewport.size().x,
            viewport.size().y,
        ],
        params0: [
            camera_settings.fov_y_degrees.to_radians(),
            camera_settings.near_clip,
            camera_settings.far_clip,
            settings.min_screen_length_px,
        ],
        params1: [
            settings.width_px,
            settings.overshoot_px,
            if settings.boundary { 1.0 } else { 0.0 },
            if settings.silhouette { 1.0 } else { 0.0 },
        ],
        params2: [
            if settings.feature { 1.0 } else { 0.0 },
            if settings.contact { 1.0 } else { 0.0 },
            if settings.suggestive { 1.0 } else { 0.0 },
            settings.feature_angle_degrees.to_radians().cos(),
        ],
        params3: [
            silhouette_style.width_multiplier,
            boundary_style.width_multiplier,
            feature_style.width_multiplier,
            settings.pass_offset_px,
        ],
        params4: [
            silhouette_style.alpha_multiplier,
            boundary_style.alpha_multiplier,
            feature_style.alpha_multiplier,
            feature_style.overshoot_px,
        ],
        params5: [
            primary_passes,
            search_passes,
            settings.search_line_alpha,
            settings.taper,
        ],
        params6: [
            1.0,
            1.0,
            1.0,
            feature_brush.dropout_multiplier,
        ],
        params7: [
            1.0,
            settings.humanization,
            settings.distance_width_falloff,
            settings.depth_pressure,
        ],
        params8: settings.width_pressure_curve,
        params9: settings.alpha_pressure_curve,
        params10: [
            settings.pressure_jitter,
            settings.stroke_wobble_frequency.max(0.01),
            micro_wobble,
            settings.micro_wobble_frequency.max(0.01),
        ],
        params11: [
            settings.local_angular_drift_degrees.to_radians().sin() * settings.humanization,
            settings.line_confidence.clamp(0.0, 1.0),
            settings.depth_alpha.clamp(0.0, 1.0),
            settings.undershoot_px.max(0.0),
        ],
        params12: [
            silhouette_style.wobble_px,
            boundary_style.wobble_px,
            feature_style.wobble_px,
            settings.endpoint_snap_px.max(0.5),
        ],
        params13: [
            overlay_mode,
            settings.contact_ground_y,
            settings.contact_threshold.max(0.0),
            settings.dropout.max(0.0),
        ],
        params14: [
            gpu_tuning.max_render_length_px,
            gpu_tuning.max_segment_length_px,
            gpu_tuning.max_terminal_walk_edges as f32,
            gpu_tuning.max_chained_walk_edges as f32,
        ],
        params15: [
            gpu_tuning.max_chain_angle_degrees.to_radians().cos(),
            if gpu_tuning.search_enabled { 1.0 } else { 0.0 },
            gpu_tuning.search_max_render_length_px,
            gpu_tuning.search_alpha_multiplier,
        ],
        params16: [
            gpu_tuning.feature_min_length_multiplier,
            gpu_tuning.feature_alpha_multiplier,
            gpu_tuning.silhouette_min_length_multiplier,
            face_id_base as f32,
        ],
        params17: [
            gpu_tuning.artist_selection_amount,
            gpu_tuning.artist_trim_amount,
            gpu_tuning.artist_lift_amount,
            0.0,
        ],
        params18: [
            if camera_response.enabled { 1.0 } else { 0.0 },
            camera_response.near_width_boost,
            camera_response.near_detail_boost,
            camera_response.near_hatching_boost,
        ],
        params19: [
            camera_response.far_width_falloff,
            camera_response.far_alpha_falloff,
            camera_response.far_detail_suppression,
            camera_response.rim_silhouette_boost,
        ],
        params20: [
            camera_response.front_feature_suppression,
            camera_near_distance,
            camera_far_distance,
            camera_focus_distance01,
        ],
        params21: [
            silhouette_brush
                .overshoot_px
                .unwrap_or(silhouette_style.overshoot_px),
            boundary_brush
                .overshoot_px
                .unwrap_or(boundary_style.overshoot_px),
            feature_brush
                .overshoot_px
                .unwrap_or(feature_style.overshoot_px),
            0.0,
        ],
        params22: [
            silhouette_brush.width_multiplier,
            boundary_brush.width_multiplier,
            feature_brush.width_multiplier,
            seam_brush.width_multiplier,
        ],
        params23: [
            silhouette_brush.alpha_multiplier,
            boundary_brush.alpha_multiplier,
            feature_brush.alpha_multiplier,
            seam_brush.alpha_multiplier,
        ],
        params24: [
            silhouette_brush.path_wobble_multiplier,
            boundary_brush.path_wobble_multiplier,
            feature_brush.path_wobble_multiplier,
            seam_brush.path_wobble_multiplier,
        ],
        params25: [
            silhouette_brush.pressure_jitter_multiplier,
            boundary_brush.pressure_jitter_multiplier,
            feature_brush.pressure_jitter_multiplier,
            seam_brush.pressure_jitter_multiplier,
        ],
        params26: [
            (silhouette_style.taper * silhouette_brush.taper_multiplier).clamp(0.0, 1.5),
            (boundary_style.taper * boundary_brush.taper_multiplier).clamp(0.0, 1.5),
            (feature_style.taper * feature_brush.taper_multiplier).clamp(0.0, 1.5),
            (seam_style.taper * seam_brush.taper_multiplier).clamp(0.0, 1.5),
        ],
        params27: [
            silhouette_brush.angle_bias_radians,
            boundary_brush.angle_bias_radians,
            feature_brush.angle_bias_radians,
            seam_brush.angle_bias_radians,
        ],
        params28: silhouette_brush.width_curve,
        params29: boundary_brush.width_curve,
        params30: feature_brush.width_curve,
        params31: seam_brush.width_curve,
        params32: silhouette_brush.alpha_curve,
        params33: boundary_brush.alpha_curve,
        params34: feature_brush.alpha_curve,
        params35: seam_brush.alpha_curve,
        params36: [
            npr_min_screen_length_px_for_kind(NprLineKind::Silhouette, settings).max(0.0),
            npr_min_screen_length_px_for_kind(NprLineKind::Boundary, settings).max(0.0),
            npr_min_screen_length_px_for_kind(NprLineKind::Feature, settings).max(0.0),
            npr_min_screen_length_px_for_kind(NprLineKind::Crease, settings).max(0.0),
        ],
        params37: [
            npr_min_screen_length_px_for_kind(NprLineKind::Seam, settings).max(0.0),
            npr_min_screen_length_px_for_kind(NprLineKind::Contact, settings).max(0.0),
            npr_technical_detail_keep_for_kind(NprLineKind::Feature, settings).clamp(0.0, 1.0),
            npr_technical_detail_keep_for_kind(NprLineKind::Crease, settings).clamp(0.0, 1.0),
        ],
        params38: [
            npr_technical_detail_keep_for_kind(NprLineKind::Seam, settings).clamp(0.0, 1.0),
            npr_technical_detail_keep_for_kind(NprLineKind::Contact, settings).clamp(0.0, 1.0),
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Silhouette, settings).max(0.0),
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Boundary, settings).max(0.0),
        ],
        params39: [
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Feature, settings).max(0.0),
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Crease, settings).max(0.0),
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Seam, settings).max(0.0),
            npr_preferred_stroke_length_px_for_kind(NprLineKind::Contact, settings).max(0.0),
        ],
        params40: [
            npr_stroke_join_gap_px_for_kind(NprLineKind::Silhouette, settings).max(0.0),
            npr_stroke_join_gap_px_for_kind(NprLineKind::Boundary, settings).max(0.0),
            npr_stroke_join_gap_px_for_kind(NprLineKind::Feature, settings).max(0.0),
            npr_stroke_join_gap_px_for_kind(NprLineKind::Crease, settings).max(0.0),
        ],
        params41: [
            npr_stroke_join_gap_px_for_kind(NprLineKind::Seam, settings).max(0.0),
            npr_stroke_join_gap_px_for_kind(NprLineKind::Contact, settings).max(0.0),
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Silhouette, settings)
                .max(0.0)
                .to_radians()
                .cos(),
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Boundary, settings)
                .max(0.0)
                .to_radians()
                .cos(),
        ],
        params42: [
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Feature, settings)
                .max(0.0)
                .to_radians()
                .cos(),
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Crease, settings)
                .max(0.0)
                .to_radians()
                .cos(),
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Seam, settings)
                .max(0.0)
                .to_radians()
                .cos(),
            npr_stroke_join_max_angle_degrees_for_kind(NprLineKind::Contact, settings)
                .max(0.0)
                .to_radians()
                .cos(),
        ],
        params43: [
            silhouette_brush.path_adherence_multiplier,
            boundary_brush.path_adherence_multiplier,
            feature_brush.path_adherence_multiplier,
            seam_brush.path_adherence_multiplier,
        ],
        params44: [
            silhouette_brush.angle_influence,
            boundary_brush.angle_influence,
            feature_brush.angle_influence,
            seam_brush.angle_influence,
        ],
        params45: [
            super::npr_continuation_bias_for_kind(NprLineKind::Silhouette, settings),
            super::npr_continuation_bias_for_kind(NprLineKind::Boundary, settings),
            super::npr_continuation_bias_for_kind(NprLineKind::Feature, settings),
            super::npr_continuation_bias_for_kind(NprLineKind::Crease, settings),
        ],
        params46: [
            super::npr_continuation_bias_for_kind(NprLineKind::Seam, settings),
            super::npr_continuation_bias_for_kind(NprLineKind::Contact, settings),
            super::npr_breakup_bias_for_kind(NprLineKind::Feature, settings),
            super::npr_breakup_bias_for_kind(NprLineKind::Crease, settings),
        ],
        params47: [
            npr_technical_detail_preference_for_kind(NprLineKind::Feature, settings),
            npr_technical_detail_preference_for_kind(NprLineKind::Crease, settings),
            npr_technical_detail_preference_for_kind(NprLineKind::Seam, settings),
            npr_technical_detail_preference_for_kind(NprLineKind::Contact, settings),
        ],
        params48: [
            npr_ink_detail_material_preference_for_kind(NprLineKind::Feature, settings),
            npr_ink_detail_material_preference_for_kind(NprLineKind::Crease, settings),
            npr_ink_detail_material_preference_for_kind(NprLineKind::Seam, settings),
            npr_ink_detail_material_preference_for_kind(NprLineKind::Contact, settings),
        ],
        params49: [
            npr_material_seam_preference_for_kind(NprLineKind::Feature, settings),
            npr_material_seam_preference_for_kind(NprLineKind::Crease, settings),
            npr_material_seam_preference_for_kind(NprLineKind::Seam, settings),
            npr_material_seam_preference_for_kind(NprLineKind::Contact, settings),
        ],
        params50: [
            gpu_brush_tip(silhouette_brush.tip) as f32,
            gpu_brush_tip(boundary_brush.tip) as f32,
            gpu_brush_tip(feature_brush.tip) as f32,
            gpu_brush_tip(seam_brush.tip) as f32,
        ],
        params51: [
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Silhouette, settings))
                as f32,
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Boundary, settings))
                as f32,
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Feature, settings))
                as f32,
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Crease, settings))
                as f32,
        ],
        params52: [
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Seam, settings))
                as f32,
            gpu_line_family_role(npr_line_family_role_for_kind(NprLineKind::Contact, settings))
                as f32,
            0.0,
            0.0,
        ],
        params53: [
            npr_min_stroke_length_px_for_kind(NprLineKind::Silhouette, settings),
            npr_min_stroke_length_px_for_kind(NprLineKind::Boundary, settings),
            npr_min_stroke_length_px_for_kind(NprLineKind::Feature, settings),
            npr_min_stroke_length_px_for_kind(NprLineKind::Crease, settings),
        ],
        params54: [
            npr_min_stroke_length_px_for_kind(NprLineKind::Seam, settings),
            npr_min_stroke_length_px_for_kind(NprLineKind::Contact, settings),
            0.0,
            0.0,
        ],
        ink_color: [
            settings.ink_color.r,
            settings.ink_color.g,
            settings.ink_color.b,
            settings.ink_color.a,
        ],
        seed: [
            settings.seed as u32,
            (settings.seed >> 32) as u32,
            settings.endpoint_lock_start_px.to_bits(),
            settings.endpoint_lock_end_px.to_bits(),
        ],
        pipeline0: [
            gpu_candidate_strategy(pipeline_plan.candidate_strategy),
            gpu_path_strategy(pipeline_plan.path_strategy),
            gpu_stroke_strategy(pipeline_plan.stroke_strategy),
            gpu_fill_strategy(pipeline_plan.fill_strategy),
        ],
        pipeline1: [
            gpu_hatching_strategy(pipeline_plan.hatching_strategy),
            gpu_budget_strategy(pipeline_plan.budget_strategy),
            gpu_temporal_strategy(pipeline_plan.temporal_strategy),
            edge_count,
        ],
        material_roles0: [
            gpu_material_id_mask(&settings.black_mass_material_ids),
            gpu_material_id_mask(&settings.ink_detail_material_ids),
            path_segment_base,
            path_segment_slot_count,
        ],
    }
}

fn gpu_brush_tip(value: amigo_render_api::NprBrushTip3d) -> u32 {
    match value {
        amigo_render_api::NprBrushTip3d::Round => 0,
        amigo_render_api::NprBrushTip3d::Flat => 1,
        amigo_render_api::NprBrushTip3d::GPen => 2,
        amigo_render_api::NprBrushTip3d::MaruPen => 3,
        amigo_render_api::NprBrushTip3d::DryBrush => 4,
    }
}

fn gpu_line_family_role(value: amigo_render_api::NprLineFamilyRole3d) -> u32 {
    match value {
        amigo_render_api::NprLineFamilyRole3d::Generic => 0,
        amigo_render_api::NprLineFamilyRole3d::OuterContour => 1,
        amigo_render_api::NprLineFamilyRole3d::DetailInk => 2,
        amigo_render_api::NprLineFamilyRole3d::ClothFold => 3,
        amigo_render_api::NprLineFamilyRole3d::MaterialCut => 4,
        amigo_render_api::NprLineFamilyRole3d::ShadowHatch => 5,
        amigo_render_api::NprLineFamilyRole3d::ContactShadow => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::uniforms_for_job;
    use crate::renderer::{CachedMeshGeometry3d, Viewport};
    use amigo_math::Vec3;
    use std::collections::BTreeMap;

    #[test]
    fn gpu_uniforms_encode_pipeline_strategies() {
        let settings = amigo_render_api::NprLineSettings3d {
            pipeline: amigo_render_api::NprPipelineStrategies3d {
                candidate_strategy: amigo_render_api::NprCandidateStrategy3d::CharacterSemantic,
                path_strategy: amigo_render_api::NprPathStrategy3d::StableStrokedPaths,
                stroke_strategy: amigo_render_api::NprStrokeStrategy3d::AkiraInk,
                fill_strategy: amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass,
                hatching_strategy: amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching,
                budget_strategy: amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority,
                temporal_strategy: amigo_render_api::NprTemporalStrategy3d::StableArcLength,
            },
            black_mass_material_ids: vec![4, 5, 7, 11, 12, 13, 64],
            ink_detail_material_ids: vec![6, 7, 11, 12, 13],
            gpu_realtime_tuning: amigo_render_api::NprGpuRealtimeTuning3d {
                artist_selection_amount: 1.25,
                artist_trim_amount: 1.5,
                artist_lift_amount: 0.75,
                ..amigo_render_api::NprGpuRealtimeTuning3d::default()
            },
            camera_response: amigo_render_api::NprCameraResponse3d {
                enabled: true,
                auto_focus: false,
                near_distance: 1.6,
                far_distance: 9.5,
                focus_near_band: 0.8,
                focus_far_band: 1.9,
                near_width_boost: 0.36,
                near_detail_boost: 0.8,
                near_hatching_boost: 1.35,
                far_width_falloff: 0.48,
                far_alpha_falloff: 0.72,
                far_detail_suppression: 1.05,
                rim_silhouette_boost: 0.28,
                front_feature_suppression: 0.42,
            },
            brush_profiles: BTreeMap::from([
                (
                    "silhouette_brush".to_string(),
                    amigo_render_api::NprBrushProfile3d {
                        tip: Some(amigo_render_api::NprBrushTip3d::GPen),
                        width_curve: [0.41, 0.52, 0.63, 0.74],
                        alpha_curve: [0.81, 0.82, 0.83, 0.84],
                        path_adherence_multiplier: 0.86,
                        angle_influence: 0.62,
                        ..amigo_render_api::NprBrushProfile3d::default()
                    },
                ),
                (
                    "feature_brush".to_string(),
                    amigo_render_api::NprBrushProfile3d {
                        tip: Some(amigo_render_api::NprBrushTip3d::MaruPen),
                        width_curve: [0.11, 0.22, 0.33, 0.44],
                        alpha_curve: [0.55, 0.66, 0.77, 0.88],
                        path_adherence_multiplier: 0.94,
                        angle_influence: 0.28,
                        ..amigo_render_api::NprBrushProfile3d::default()
                    },
                ),
            ]),
            line_families: vec![
                amigo_render_api::NprLineFamily3d {
                    id: "silhouette_family".to_string(),
                    role: Some(amigo_render_api::NprLineFamilyRole3d::OuterContour),
                    sources: vec![amigo_render_api::NprLineSource3d::Silhouette],
                    brush: Some("silhouette_brush".to_string()),
                    min_screen_length_px: Some(1.25),
                    min_stroke_length_px: Some(11.0),
                    preferred_stroke_length_px: Some(44.0),
                    stroke_join_gap_px: Some(7.0),
                    stroke_join_max_angle_degrees: Some(24.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
                amigo_render_api::NprLineFamily3d {
                    id: "feature_family".to_string(),
                    role: Some(amigo_render_api::NprLineFamilyRole3d::DetailInk),
                    sources: vec![amigo_render_api::NprLineSource3d::Feature],
                    brush: Some("feature_brush".to_string()),
                    min_screen_length_px: Some(3.5),
                    min_stroke_length_px: Some(13.0),
                    preferred_stroke_length_px: Some(18.0),
                    technical_detail_keep: Some(0.61),
                    technical_detail_preference: Some(0.77),
                    ink_detail_material_preference: Some(0.88),
                    material_seam_preference: Some(0.12),
                    continuation_bias: Some(0.22),
                    breakup_bias: Some(0.84),
                    stroke_join_gap_px: Some(2.5),
                    stroke_join_max_angle_degrees: Some(13.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
                amigo_render_api::NprLineFamily3d {
                    id: "crease_family".to_string(),
                    role: Some(amigo_render_api::NprLineFamilyRole3d::ClothFold),
                    sources: vec![amigo_render_api::NprLineSource3d::Crease],
                    min_screen_length_px: Some(4.5),
                    min_stroke_length_px: Some(17.0),
                    preferred_stroke_length_px: Some(22.0),
                    technical_detail_keep: Some(0.52),
                    technical_detail_preference: Some(0.66),
                    ink_detail_material_preference: Some(0.34),
                    material_seam_preference: Some(0.21),
                    continuation_bias: Some(0.28),
                    breakup_bias: Some(0.72),
                    stroke_join_gap_px: Some(3.5),
                    stroke_join_max_angle_degrees: Some(17.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
                amigo_render_api::NprLineFamily3d {
                    id: "seam_family".to_string(),
                    role: Some(amigo_render_api::NprLineFamilyRole3d::MaterialCut),
                    sources: vec![amigo_render_api::NprLineSource3d::Seam],
                    min_screen_length_px: Some(5.5),
                    min_stroke_length_px: Some(19.0),
                    preferred_stroke_length_px: Some(26.0),
                    technical_detail_keep: Some(0.73),
                    technical_detail_preference: Some(0.42),
                    ink_detail_material_preference: Some(0.18),
                    material_seam_preference: Some(0.93),
                    continuation_bias: Some(0.34),
                    stroke_join_gap_px: Some(4.5),
                    stroke_join_max_angle_degrees: Some(21.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
                amigo_render_api::NprLineFamily3d {
                    id: "contact_family".to_string(),
                    sources: vec![amigo_render_api::NprLineSource3d::Contact],
                    min_screen_length_px: Some(6.5),
                    min_stroke_length_px: Some(23.0),
                    preferred_stroke_length_px: Some(30.0),
                    technical_detail_keep: Some(0.47),
                    technical_detail_preference: Some(0.28),
                    ink_detail_material_preference: Some(0.08),
                    material_seam_preference: Some(0.16),
                    continuation_bias: Some(0.64),
                    stroke_join_gap_px: Some(5.5),
                    stroke_join_max_angle_degrees: Some(26.0),
                    ..amigo_render_api::NprLineFamily3d::default()
                },
            ],
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let geometry = CachedMeshGeometry3d::from_test_vertices(vec![Vec3::ZERO]);

        let uniforms = uniforms_for_job(
            &Viewport::from_dimensions(1280.0, 720.0),
            amigo_math::Transform3::default(),
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            amigo_math::Transform3::default(),
            &settings,
            0,
            321,
            654,
            123,
            456,
            789,
            None,
        );

        assert_eq!(uniforms.pipeline0, [1, 0, 1, 1]);
        assert_eq!(uniforms.pipeline1, [1, 1, 1, 789]);
        assert_eq!(uniforms.params17, [1.25, 1.5, 0.75, 0.0]);
        assert_eq!(uniforms.params18, [1.0, 0.36, 0.8, 1.35]);
        assert_eq!(uniforms.params19, [0.48, 0.72, 1.05, 0.28]);
        assert_eq!(uniforms.params20, [0.42, 1.6, 9.5, 0.0]);
        assert_eq!(uniforms.params21[3], 0.0);
        assert_eq!(uniforms.params28, [0.41, 0.52, 0.63, 0.74]);
        assert_eq!(uniforms.params34, [0.55, 0.66, 0.77, 0.88]);
        assert_eq!(uniforms.params36, [1.25, 2.4, 3.5, 4.5]);
        assert_eq!(uniforms.params37, [5.5, 6.5, 0.61, 0.52]);
        assert_eq!(uniforms.params38, [0.73, 0.47, 44.0, 56.0]);
        assert_eq!(uniforms.params39, [18.0, 22.0, 26.0, 30.0]);
        assert_eq!(uniforms.params40, [7.0, 2.25, 2.5, 3.5]);
        assert_eq!(uniforms.params41, [4.5, 5.5, 24.0f32.to_radians().cos(), 22.0f32.to_radians().cos()]);
        assert_eq!(uniforms.params42, [
            13.0f32.to_radians().cos(),
            17.0f32.to_radians().cos(),
            21.0f32.to_radians().cos(),
            26.0f32.to_radians().cos(),
        ]);
        assert_eq!(uniforms.params43, [0.86, 1.0, 0.94, 1.0]);
        assert_eq!(uniforms.params44, [0.62, 0.0, 0.28, 0.0]);
        assert_eq!(uniforms.params45, [0.5, 0.5, 0.22, 0.28]);
        assert_eq!(uniforms.params46, [0.34, 0.64, 0.84, 0.72]);
        assert_eq!(uniforms.params47, [0.77, 0.66, 0.42, 0.28]);
        assert_eq!(uniforms.params48, [0.88, 0.34, 0.18, 0.08]);
        assert_eq!(uniforms.params49, [0.12, 0.21, 0.93, 0.16]);
        assert_eq!(uniforms.params50, [2.0, 3.0, 3.0, 3.0]);
        assert_eq!(uniforms.params51, [1.0, 1.0, 2.0, 3.0]);
        assert_eq!(uniforms.params52, [4.0, 6.0, 0.0, 0.0]);
        assert_eq!(uniforms.params53, [11.0, settings.min_stroke_length_px, 13.0, 17.0]);
        assert_eq!(uniforms.params54, [19.0, 23.0, 0.0, 0.0]);
        assert_eq!(
            uniforms.material_roles0,
            [
                (1 << 4) | (1 << 5) | (1 << 7) | (1 << 11) | (1 << 12) | (1 << 13),
                (1 << 6) | (1 << 7) | (1 << 11) | (1 << 12) | (1 << 13),
                321,
                654
            ]
        );
    }

    #[test]
    fn gpu_uniforms_auto_focus_rebases_camera_response_distances() {
        let settings = amigo_render_api::NprLineSettings3d {
            camera_response: amigo_render_api::NprCameraResponse3d {
                enabled: true,
                auto_focus: true,
                near_distance: 2.0,
                far_distance: 10.0,
                focus_near_band: 0.8,
                focus_far_band: 1.9,
                ..amigo_render_api::NprCameraResponse3d::default()
            },
            ..amigo_render_api::NprLineSettings3d::default()
        };
        let geometry = CachedMeshGeometry3d::from_test_vertices(vec![
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ]);
        let camera = amigo_math::Transform3 {
            translation: Vec3::new(0.0, 0.0, 6.0),
            ..amigo_math::Transform3::default()
        };

        let uniforms = uniforms_for_job(
            &Viewport::from_dimensions(1280.0, 720.0),
            camera,
            amigo_render_api::Camera3dRenderSettings::default(),
            &geometry,
            amigo_math::Transform3::default(),
            &settings,
            0,
            0,
            0,
            2,
            0,
            0,
            None,
        );

        assert!((uniforms.params20[1] - 5.2).abs() < 0.001);
        assert!((uniforms.params20[2] - 7.9).abs() < 0.001);
        assert!((uniforms.params20[3] - 0.5).abs() < 0.001);
    }

}
