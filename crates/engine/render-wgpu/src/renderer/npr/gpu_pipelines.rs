use super::{create_npr_gpu_compute_pipeline_set, create_npr_gpu_face_id_pipeline_set};

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct NprGpuPipelines3d {
    pub face_id_pipeline: wgpu::RenderPipeline,
    pub face_id_bind_group_layout: wgpu::BindGroupLayout,
    pub project_vertices_bind_group_layout: wgpu::BindGroupLayout,
    pub classify_edges_bind_group_layout: wgpu::BindGroupLayout,
    pub build_endpoint_bins_bind_group_layout: wgpu::BindGroupLayout,
    pub clear_endpoint_heads_bind_group_layout: wgpu::BindGroupLayout,
    pub compact_owners_bind_group_layout: wgpu::BindGroupLayout,
    pub connect_paths_bind_group_layout: wgpu::BindGroupLayout,
    pub relax_path_owners_bind_group_layout: wgpu::BindGroupLayout,
    pub aggregate_paths_bind_group_layout: wgpu::BindGroupLayout,
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
    pub aggregate_paths_pipeline: wgpu::ComputePipeline,
    pub emit_path_segments_pipeline: wgpu::ComputePipeline,
    pub build_strokes_pipeline: wgpu::ComputePipeline,
    pub clamp_indirect_args_pipeline: wgpu::ComputePipeline,
}

impl NprGpuPipelines3d {
    pub(crate) fn create(device: &wgpu::Device) -> Self {
        let face_id = create_npr_gpu_face_id_pipeline_set(device);
        let compute = create_npr_gpu_compute_pipeline_set(device);

        Self {
            face_id_pipeline: face_id.pipeline,
            face_id_bind_group_layout: face_id.bind_group_layout,
            project_vertices_bind_group_layout: compute.project_vertices_bind_group_layout,
            classify_edges_bind_group_layout: compute.classify_edges_bind_group_layout,
            build_endpoint_bins_bind_group_layout: compute.build_endpoint_bins_bind_group_layout,
            clear_endpoint_heads_bind_group_layout: compute.clear_endpoint_heads_bind_group_layout,
            compact_owners_bind_group_layout: compute.compact_owners_bind_group_layout,
            connect_paths_bind_group_layout: compute.connect_paths_bind_group_layout,
            relax_path_owners_bind_group_layout: compute.relax_path_owners_bind_group_layout,
            aggregate_paths_bind_group_layout: compute.aggregate_paths_bind_group_layout,
            emit_path_segments_bind_group_layout: compute.emit_path_segments_bind_group_layout,
            build_strokes_bind_group_layout: compute.build_strokes_bind_group_layout,
            clamp_indirect_args_bind_group_layout: compute.clamp_indirect_args_bind_group_layout,
            project_vertices_pipeline: compute.project_vertices_pipeline,
            classify_edges_pipeline: compute.classify_edges_pipeline,
            build_endpoint_bins_pipeline: compute.build_endpoint_bins_pipeline,
            clear_endpoint_heads_pipeline: compute.clear_endpoint_heads_pipeline,
            compact_owners_pipeline: compute.compact_owners_pipeline,
            connect_paths_pipeline: compute.connect_paths_pipeline,
            relax_path_owners_pipeline: compute.relax_path_owners_pipeline,
            aggregate_paths_pipeline: compute.aggregate_paths_pipeline,
            emit_path_segments_pipeline: compute.emit_path_segments_pipeline,
            build_strokes_pipeline: compute.build_strokes_pipeline,
            clamp_indirect_args_pipeline: compute.clamp_indirect_args_pipeline,
        }
    }
}
