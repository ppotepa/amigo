use super::{
    NprGpuFrameBuffers3d, NprGpuMeshTopologyBuffers3d, NprGpuPipelines3d, storage_binding,
    texture_binding, uniform_binding,
};

#[derive(Debug)]
pub(crate) struct NprGpuStrokeComputeBindGroups3d {
    pub project_vertices: wgpu::BindGroup,
    pub classify_edges: wgpu::BindGroup,
    pub build_endpoint_bins: wgpu::BindGroup,
    pub clear_endpoint_heads: wgpu::BindGroup,
    pub compact_owners: wgpu::BindGroup,
    pub connect_paths: wgpu::BindGroup,
    pub relax_path_owners: wgpu::BindGroup,
    pub aggregate_paths: wgpu::BindGroup,
    pub emit_path_segments: wgpu::BindGroup,
    pub build_strokes: wgpu::BindGroup,
    pub clamp_indirect_args: wgpu::BindGroup,
}

pub(crate) fn create_npr_gpu_face_id_bind_group(
    device: &wgpu::Device,
    pipelines: &NprGpuPipelines3d,
    topology: &NprGpuMeshTopologyBuffers3d,
    uniforms: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-face-id-bind-group"),
        layout: &pipelines.face_id_bind_group_layout,
        entries: &[
            storage_binding(0, &topology.vertices),
            storage_binding(1, &topology.triangles),
            uniform_binding(8, uniforms),
        ],
    })
}

pub(crate) fn create_npr_gpu_stroke_compute_bind_groups(
    device: &wgpu::Device,
    pipelines: &NprGpuPipelines3d,
    topology: &NprGpuMeshTopologyBuffers3d,
    frame_buffers: &NprGpuFrameBuffers3d,
    uniforms: &wgpu::Buffer,
    face_id_view: &wgpu::TextureView,
) -> NprGpuStrokeComputeBindGroups3d {
    let project_vertices = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-project-vertices-bind-group"),
        layout: &pipelines.project_vertices_bind_group_layout,
        entries: &[
            storage_binding(0, &topology.vertices),
            storage_binding(3, &frame_buffers.projected_vertices),
            uniform_binding(8, uniforms),
        ],
    });
    let classify_edges = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-classify-edges-bind-group"),
        layout: &pipelines.classify_edges_bind_group_layout,
        entries: &[
            storage_binding(0, &topology.vertices),
            storage_binding(1, &topology.triangles),
            storage_binding(2, &topology.edges),
            storage_binding(3, &frame_buffers.projected_vertices),
            texture_binding(4, face_id_view),
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
        ],
    });
    let build_endpoint_bins = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-build-endpoint-bins-bind-group"),
        layout: &pipelines.build_endpoint_bins_bind_group_layout,
        entries: &[
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(11, &frame_buffers.endpoint_heads),
            storage_binding(12, &frame_buffers.endpoint_entries),
        ],
    });
    let clear_endpoint_heads = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-clear-endpoint-heads-bind-group"),
        layout: &pipelines.clear_endpoint_heads_bind_group_layout,
        entries: &[storage_binding(11, &frame_buffers.endpoint_heads)],
    });
    let compact_owners = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-compact-owners-bind-group"),
        layout: &pipelines.compact_owners_bind_group_layout,
        entries: &[
            storage_binding(2, &topology.edges),
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(10, &frame_buffers.path_links),
            storage_binding(11, &frame_buffers.endpoint_heads),
            storage_binding(12, &frame_buffers.endpoint_entries),
        ],
    });
    let connect_paths = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-connect-paths-bind-group"),
        layout: &pipelines.connect_paths_bind_group_layout,
        entries: &[
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(10, &frame_buffers.path_links),
            storage_binding(14, &frame_buffers.path_states),
        ],
    });
    let relax_path_owners = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-relax-path-owners-bind-group"),
        layout: &pipelines.relax_path_owners_bind_group_layout,
        entries: &[
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(10, &frame_buffers.path_links),
            storage_binding(14, &frame_buffers.path_states),
        ],
    });
    let aggregate_paths = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-aggregate-paths-bind-group"),
        layout: &pipelines.aggregate_paths_bind_group_layout,
        entries: &[
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(10, &frame_buffers.path_links),
            storage_binding(14, &frame_buffers.path_states),
            storage_binding(15, &frame_buffers.aggregated_paths),
        ],
    });
    let emit_path_segments = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-emit-path-segments-bind-group"),
        layout: &pipelines.emit_path_segments_bind_group_layout,
        entries: &[
            storage_binding(5, &frame_buffers.visible_segments),
            uniform_binding(8, uniforms),
            storage_binding(9, &frame_buffers.indirect_args),
            storage_binding(10, &frame_buffers.path_links),
            storage_binding(13, &frame_buffers.path_segments),
            storage_binding(14, &frame_buffers.path_states),
        ],
    });
    let build_strokes = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-build-strokes-bind-group"),
        layout: &pipelines.build_strokes_bind_group_layout,
        entries: &[
            storage_binding(2, &topology.edges),
            storage_binding(5, &frame_buffers.visible_segments),
            storage_binding(6, &frame_buffers.stroke_segments),
            uniform_binding(8, uniforms),
            storage_binding(9, &frame_buffers.indirect_args),
            storage_binding(13, &frame_buffers.path_segments),
            storage_binding(15, &frame_buffers.aggregated_paths),
        ],
    });
    let clamp_indirect_args = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("amigo-npr-clamp-indirect-args-bind-group"),
        layout: &pipelines.clamp_indirect_args_bind_group_layout,
        entries: &[storage_binding(9, &frame_buffers.indirect_args)],
    });

    NprGpuStrokeComputeBindGroups3d {
        project_vertices,
        classify_edges,
        build_endpoint_bins,
        clear_endpoint_heads,
        compact_owners,
        connect_paths,
        relax_path_owners,
        aggregate_paths,
        emit_path_segments,
        build_strokes,
        clamp_indirect_args,
    }
}
