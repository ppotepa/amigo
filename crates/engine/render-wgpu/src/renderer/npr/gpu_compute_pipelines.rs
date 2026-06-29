use crate::renderer::shaders::{
    NPR_BUILD_ENDPOINT_BINS_SHADER, NPR_BUILD_STROKES_SHADER, NPR_CLAMP_INDIRECT_ARGS_SHADER,
    NPR_CLASSIFY_EDGES_SHADER, NPR_CLEAR_ENDPOINT_HEADS_SHADER, NPR_COMPACT_OWNERS_SHADER,
    NPR_CONNECT_PATHS_SHADER, NPR_EMIT_PATH_SEGMENTS_SHADER, NPR_PROJECT_VERTICES_SHADER,
    NPR_RELAX_PATH_OWNERS_SHADER,
};

use super::{
    create_compute_bind_group_layout, create_compute_pipeline, create_compute_pipeline_layout,
    storage_entry, texture_entry, uniform_entry,
};

#[derive(Debug)]
pub(crate) struct NprGpuComputePipelineSet3d {
    pub project_vertices_bind_group_layout: wgpu::BindGroupLayout,
    pub classify_edges_bind_group_layout: wgpu::BindGroupLayout,
    pub build_endpoint_bins_bind_group_layout: wgpu::BindGroupLayout,
    pub clear_endpoint_heads_bind_group_layout: wgpu::BindGroupLayout,
    pub compact_owners_bind_group_layout: wgpu::BindGroupLayout,
    pub connect_paths_bind_group_layout: wgpu::BindGroupLayout,
    pub relax_path_owners_bind_group_layout: wgpu::BindGroupLayout,
    pub emit_path_segments_bind_group_layout: wgpu::BindGroupLayout,
    pub build_strokes_bind_group_layout: wgpu::BindGroupLayout,
    pub clamp_indirect_args_bind_group_layout: wgpu::BindGroupLayout,
    pub project_vertices_pipeline: wgpu::ComputePipeline,
    pub classify_edges_pipeline: wgpu::ComputePipeline,
    pub build_endpoint_bins_pipeline: wgpu::ComputePipeline,
    pub clear_endpoint_heads_pipeline: wgpu::ComputePipeline,
    pub compact_owners_pipeline: wgpu::ComputePipeline,
    pub connect_paths_pipeline: wgpu::ComputePipeline,
    pub relax_path_owners_pipeline: wgpu::ComputePipeline,
    pub emit_path_segments_pipeline: wgpu::ComputePipeline,
    pub build_strokes_pipeline: wgpu::ComputePipeline,
    pub clamp_indirect_args_pipeline: wgpu::ComputePipeline,
}

pub(crate) fn create_npr_gpu_compute_pipeline_set(
    device: &wgpu::Device,
) -> NprGpuComputePipelineSet3d {
    let project_vertices_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-project-vertices-bind-group-layout",
        &[
            storage_entry(0, true),
            storage_entry(3, false),
            uniform_entry(8),
        ],
    );
    let classify_edges_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-classify-edges-bind-group-layout",
        &[
            storage_entry(0, true),
            storage_entry(1, true),
            storage_entry(2, true),
            storage_entry(3, true),
            texture_entry(4),
            storage_entry(5, false),
            uniform_entry(8),
        ],
    );
    let build_endpoint_bins_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-build-endpoint-bins-bind-group-layout",
        &[
            storage_entry(5, true),
            uniform_entry(8),
            storage_entry(11, false),
            storage_entry(12, false),
        ],
    );
    let clear_endpoint_heads_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-clear-endpoint-heads-bind-group-layout",
        &[storage_entry(11, false)],
    );
    let compact_owners_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-compact-owners-bind-group-layout",
        &[
            storage_entry(2, true),
            storage_entry(5, false),
            uniform_entry(8),
            storage_entry(10, false),
            storage_entry(11, false),
            storage_entry(12, false),
        ],
    );
    let connect_paths_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-connect-paths-bind-group-layout",
        &[
            storage_entry(5, true),
            storage_entry(10, true),
            storage_entry(14, false),
        ],
    );
    let relax_path_owners_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-relax-path-owners-bind-group-layout",
        &[
            storage_entry(5, true),
            storage_entry(10, true),
            storage_entry(14, false),
        ],
    );
    let emit_path_segments_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-emit-path-segments-bind-group-layout",
        &[
            storage_entry(5, true),
            uniform_entry(8),
            storage_entry(9, false),
            storage_entry(10, true),
            storage_entry(13, false),
            storage_entry(14, true),
        ],
    );
    let build_strokes_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-build-strokes-bind-group-layout",
        &[
            storage_entry(2, true),
            storage_entry(5, true),
            storage_entry(6, false),
            uniform_entry(8),
            storage_entry(9, false),
            storage_entry(13, true),
        ],
    );
    let clamp_indirect_args_bind_group_layout = create_compute_bind_group_layout(
        device,
        "amigo-npr-clamp-indirect-args-bind-group-layout",
        &[storage_entry(9, false)],
    );
    let project_vertices_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-project-vertices-pipeline",
        NPR_PROJECT_VERTICES_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-project-vertices-pipeline-layout",
            &project_vertices_bind_group_layout,
        ),
    );
    let classify_edges_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-classify-edges-pipeline",
        NPR_CLASSIFY_EDGES_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-classify-edges-pipeline-layout",
            &classify_edges_bind_group_layout,
        ),
    );
    let build_strokes_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-build-strokes-pipeline",
        NPR_BUILD_STROKES_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-build-strokes-pipeline-layout",
            &build_strokes_bind_group_layout,
        ),
    );
    let build_endpoint_bins_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-build-endpoint-bins-pipeline",
        NPR_BUILD_ENDPOINT_BINS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-build-endpoint-bins-pipeline-layout",
            &build_endpoint_bins_bind_group_layout,
        ),
    );
    let clear_endpoint_heads_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-clear-endpoint-heads-pipeline",
        NPR_CLEAR_ENDPOINT_HEADS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-clear-endpoint-heads-pipeline-layout",
            &clear_endpoint_heads_bind_group_layout,
        ),
    );
    let compact_owners_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-compact-owners-pipeline",
        NPR_COMPACT_OWNERS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-compact-owners-pipeline-layout",
            &compact_owners_bind_group_layout,
        ),
    );
    let emit_path_segments_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-emit-path-segments-pipeline",
        NPR_EMIT_PATH_SEGMENTS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-emit-path-segments-pipeline-layout",
            &emit_path_segments_bind_group_layout,
        ),
    );
    let connect_paths_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-connect-paths-pipeline",
        NPR_CONNECT_PATHS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-connect-paths-pipeline-layout",
            &connect_paths_bind_group_layout,
        ),
    );
    let relax_path_owners_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-relax-path-owners-pipeline",
        NPR_RELAX_PATH_OWNERS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-relax-path-owners-pipeline-layout",
            &relax_path_owners_bind_group_layout,
        ),
    );
    let clamp_indirect_args_pipeline = create_compute_pipeline(
        device,
        "amigo-npr-clamp-indirect-args-pipeline",
        NPR_CLAMP_INDIRECT_ARGS_SHADER,
        &create_compute_pipeline_layout(
            device,
            "amigo-npr-clamp-indirect-args-pipeline-layout",
            &clamp_indirect_args_bind_group_layout,
        ),
    );
    NprGpuComputePipelineSet3d {
        project_vertices_bind_group_layout,
        classify_edges_bind_group_layout,
        build_endpoint_bins_bind_group_layout,
        clear_endpoint_heads_bind_group_layout,
        compact_owners_bind_group_layout,
        connect_paths_bind_group_layout,
        relax_path_owners_bind_group_layout,
        emit_path_segments_bind_group_layout,
        build_strokes_bind_group_layout,
        clamp_indirect_args_bind_group_layout,
        project_vertices_pipeline,
        classify_edges_pipeline,
        build_endpoint_bins_pipeline,
        clear_endpoint_heads_pipeline,
        compact_owners_pipeline,
        connect_paths_pipeline,
        relax_path_owners_pipeline,
        emit_path_segments_pipeline,
        build_strokes_pipeline,
        clamp_indirect_args_pipeline,
    }
}
