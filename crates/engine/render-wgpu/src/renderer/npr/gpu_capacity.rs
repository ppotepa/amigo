pub(crate) const NPR_GPU_SEGMENTS_PER_STROKE_PASS: usize = 3;
pub(crate) const NPR_GPU_PATH_SEGMENTS_PER_CHAIN: usize = 3;
pub(crate) const NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES: usize = 120 * 1024 * 1024;

pub(crate) fn npr_gpu_pass_count(settings: &amigo_render_api::NprLineSettings3d) -> u32 {
    let primary_passes = settings.passes.min(8).max(1) as u32;
    let search_passes = if settings.gpu_realtime_tuning.search_enabled {
        ((settings.search_line_count as f32)
            * super::resolve_npr_brush_profile(
                crate::renderer::NprLineKind::Feature,
                settings,
            )
            .search_multiplier)
            .round()
            .clamp(0.0, 8.0) as u32
    } else {
        0
    };
    let hatching_passes = match settings.pipeline.hatching_strategy {
        amigo_render_api::NprHatchingStrategy3d::None => 0,
        amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching => 1,
    };
    primary_passes + search_passes + hatching_passes
}

pub(crate) fn npr_gpu_stroke_segment_capacity_units(
    edge_count: usize,
    pass_count: u32,
    budget_strategy: amigo_render_api::NprBudgetStrategy3d,
) -> usize {
    let raw_capacity = edge_count
        .saturating_mul(NPR_GPU_PATH_SEGMENTS_PER_CHAIN)
        .saturating_mul(pass_count as usize)
        .saturating_mul(NPR_GPU_SEGMENTS_PER_STROKE_PASS);
    let per_edge_budget = match budget_strategy {
        amigo_render_api::NprBudgetStrategy3d::EdgeVisibility => 4,
        amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority => 3,
        amigo_render_api::NprBudgetStrategy3d::CharacterReadability => 2,
    };
    let draw_budget = edge_count
        .saturating_mul(per_edge_budget)
        .saturating_mul(pass_count.max(1) as usize);
    raw_capacity.min(draw_budget.max(edge_count.max(1)))
}

pub(crate) fn npr_gpu_max_path_segment_capacity_units() -> usize {
    NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES / std::mem::size_of::<super::GpuNprPathSegment3d>()
}

pub(crate) fn npr_gpu_path_segment_capacity_units(edge_count: usize, job_count: usize) -> usize {
    let raw_capacity = edge_count.saturating_mul(NPR_GPU_PATH_SEGMENTS_PER_CHAIN);
    let per_job_limit = (npr_gpu_max_path_segment_capacity_units() / job_count.max(1)).max(1);
    raw_capacity.min(per_job_limit).max(1)
}

#[cfg(test)]
mod tests {
    use super::{
        NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES, NPR_GPU_PATH_SEGMENTS_PER_CHAIN,
        npr_gpu_max_path_segment_capacity_units, npr_gpu_pass_count,
        npr_gpu_path_segment_capacity_units, npr_gpu_stroke_segment_capacity_units,
    };

    #[test]
    fn gpu_pass_count_matches_tool_scaled_search_passes() {
        let settings = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            passes: 2,
            search_line_count: 2,
            gpu_realtime_tuning: amigo_render_api::NprGpuRealtimeTuning3d {
                search_enabled: true,
                ..amigo_render_api::NprGpuRealtimeTuning3d::default()
            },
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(npr_gpu_pass_count(&settings), 5);
    }

    #[test]
    fn gpu_pass_count_ignores_search_lines_when_gpu_search_disabled() {
        let settings = amigo_render_api::NprLineSettings3d {
            stroke_tool: amigo_render_api::NprStrokeTool3d::Pencil,
            passes: 2,
            search_line_count: 2,
            gpu_realtime_tuning: amigo_render_api::NprGpuRealtimeTuning3d {
                search_enabled: false,
                ..amigo_render_api::NprGpuRealtimeTuning3d::default()
            },
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(npr_gpu_pass_count(&settings), 2);
    }

    #[test]
    fn gpu_pass_count_reserves_sparse_hatching_pass() {
        let settings = amigo_render_api::NprLineSettings3d {
            passes: 1,
            pipeline: amigo_render_api::NprPipelineStrategies3d {
                hatching_strategy: amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching,
                ..amigo_render_api::NprPipelineStrategies3d::default()
            },
            ..amigo_render_api::NprLineSettings3d::default()
        };

        assert_eq!(npr_gpu_pass_count(&settings), 2);
    }

    #[test]
    fn gpu_path_segment_chain_budget_matches_shader_slots() {
        assert_eq!(NPR_GPU_PATH_SEGMENTS_PER_CHAIN, 3);
    }

    #[test]
    fn gpu_stroke_segment_capacity_scales_with_path_segment_slots() {
        assert_eq!(
            npr_gpu_stroke_segment_capacity_units(
                10,
                2,
                amigo_render_api::NprBudgetStrategy3d::EdgeVisibility,
            ),
            10 * 4 * 2
        );
    }

    #[test]
    fn gpu_path_segment_capacity_stays_below_safe_binding_budget() {
        let bytes = npr_gpu_max_path_segment_capacity_units()
            * std::mem::size_of::<super::super::GpuNprPathSegment3d>();

        assert!(bytes <= NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES);
    }

    #[test]
    fn gpu_path_segment_capacity_caps_large_riders_mesh() {
        let riders_edge_count = 703_028usize;
        let capacity = npr_gpu_path_segment_capacity_units(riders_edge_count, 1);
        let bytes = capacity * std::mem::size_of::<super::super::GpuNprPathSegment3d>();

        assert!(capacity < riders_edge_count * NPR_GPU_PATH_SEGMENTS_PER_CHAIN);
        assert!(bytes <= NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES);
    }

    #[test]
    fn gpu_path_segment_capacity_splits_binding_budget_between_jobs() {
        let riders_edge_count = 703_028usize;
        let capacity = npr_gpu_path_segment_capacity_units(riders_edge_count, 2);
        let total_bytes = capacity * 2 * std::mem::size_of::<super::super::GpuNprPathSegment3d>();

        assert!(total_bytes <= NPR_GPU_MAX_PATH_SEGMENT_BINDING_BYTES);
    }
}
