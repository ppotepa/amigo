use amigo_math::Transform3;

use crate::renderer::{
    CachedMeshGeometry3d, GpuNprFrameUniforms3d, NprDebugOverlay3d, NprLineKind, Viewport,
};

use super::{
    camera_response_distances, gpu_budget_strategy, gpu_candidate_strategy, gpu_fill_strategy,
    gpu_hatching_strategy, gpu_material_id_mask, gpu_path_strategy, gpu_stroke_strategy,
    gpu_temporal_strategy, vec3_to_gpu4,
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
    let gpu_tuning = settings.gpu_realtime_tuning.normalized();
    let camera_response = settings.camera_response.normalized();
    let pipeline_plan = settings.pipeline_plan();
    let (camera_near_distance, camera_far_distance, camera_focus_distance01) =
        camera_response_distances(camera_response, geometry, camera, transform);
    let brush = super::resolve_npr_brush_profile(settings);
    let primary_passes = settings.passes.min(8).max(1) as f32;
    let search_passes = if gpu_tuning.search_enabled {
        ((settings.search_line_count as f32) * brush.search_multiplier)
            .round()
            .clamp(0.0, 8.0)
    } else {
        0.0
    };
    let micro_wobble = settings.micro_wobble_px
        * settings.humanization
        * brush.path_wobble_multiplier
        * brush.micro_wobble_multiplier;
    let overlay_mode = match settings.gpu_realtime_tuning.debug_mode {
        amigo_render_api::NprGpuDebugMode3d::Final => match overlay {
            Some(NprDebugOverlay3d::LineKinds) => 1.0,
            Some(NprDebugOverlay3d::RawPaths) => 2.0,
            Some(NprDebugOverlay3d::Dropout) => 3.0,
            Some(NprDebugOverlay3d::WidthAlpha) => 4.0,
            None => 0.0,
        },
        amigo_render_api::NprGpuDebugMode3d::LineKinds => 1.0,
        amigo_render_api::NprGpuDebugMode3d::RawPaths => 2.0,
        amigo_render_api::NprGpuDebugMode3d::Dropout => 3.0,
        amigo_render_api::NprGpuDebugMode3d::WidthAlpha => 4.0,
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
            brush.width_multiplier,
            brush.alpha_multiplier,
            brush.pressure_jitter_multiplier,
            brush.dropout_multiplier,
        ],
        params7: [
            brush.path_wobble_multiplier,
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

#[cfg(test)]
mod tests {
    use super::uniforms_for_job;
    use crate::renderer::{CachedMeshGeometry3d, Viewport};
    use amigo_math::Vec3;

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
