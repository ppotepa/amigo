use crate::renderer::NprStrokeSegmentVertex;

use super::{
    NPR_GPU_PATH_SEGMENTS_PER_CHAIN, NprGpuFrameBuffers3d, NprGpuMeshTopologyBuffers3d,
    NprGpuPipelines3d, create_npr_gpu_face_id_bind_group,
    create_npr_gpu_stroke_compute_bind_groups, slice_as_bytes, workgroup_count,
};

pub(crate) fn write_npr_gpu_indirect_args(
    queue: &wgpu::Queue,
    frame_buffers: &NprGpuFrameBuffers3d,
    stroke_segments_capacity: u64,
) {
    let stroke_segment_capacity_units =
        (stroke_segments_capacity / std::mem::size_of::<NprStrokeSegmentVertex>() as u64) as u32;
    queue.write_buffer(
        &frame_buffers.indirect_args,
        0,
        slice_as_bytes(&[6u32, 0u32, 0u32, 0u32, 0u32, stroke_segment_capacity_units]),
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_npr_gpu_face_id_pass(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipelines: &NprGpuPipelines3d,
    topology: &NprGpuMeshTopologyBuffers3d,
    uniforms: &wgpu::Buffer,
    face_id_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    clear_face_id: bool,
) {
    let face_id_bind_group =
        create_npr_gpu_face_id_bind_group(device, pipelines, topology, uniforms);

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("amigo-npr-face-id-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: face_id_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: if clear_face_id {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: if clear_face_id {
                    wgpu::LoadOp::Clear(1.0)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(&pipelines.face_id_pipeline);
    pass.set_bind_group(0, &face_id_bind_group, &[]);
    pass.draw(0..topology.triangle_count.saturating_mul(3), 0..1);
}

pub(crate) fn run_npr_gpu_stroke_compute_passes(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    pipelines: &NprGpuPipelines3d,
    topology: &NprGpuMeshTopologyBuffers3d,
    frame_buffers: &NprGpuFrameBuffers3d,
    uniforms: &wgpu::Buffer,
    face_id_view: &wgpu::TextureView,
    path_strategy: amigo_render_api::NprPathStrategy3d,
) {
    let bind_groups = create_npr_gpu_stroke_compute_bind_groups(
        device,
        pipelines,
        topology,
        frame_buffers,
        uniforms,
        face_id_view,
    );

    run_compute_pass(
        encoder,
        "amigo-npr-project-vertices-pass",
        &pipelines.project_vertices_pipeline,
        &bind_groups.project_vertices,
        workgroup_count(topology.vertex_count as usize),
    );
    run_compute_pass(
        encoder,
        "amigo-npr-classify-edges-pass",
        &pipelines.classify_edges_pipeline,
        &bind_groups.classify_edges,
        workgroup_count(topology.edge_count as usize),
    );
    if path_strategy != amigo_render_api::NprPathStrategy3d::DirectVisibleSegments {
        run_compute_pass(
            encoder,
            "amigo-npr-clear-endpoint-heads-pass",
            &pipelines.clear_endpoint_heads_pipeline,
            &bind_groups.clear_endpoint_heads,
            workgroup_count(
                (frame_buffers.endpoint_heads_capacity / std::mem::size_of::<u32>() as u64)
                    as usize,
            ),
        );
        run_compute_pass(
            encoder,
            "amigo-npr-build-endpoint-bins-pass",
            &pipelines.build_endpoint_bins_pipeline,
            &bind_groups.build_endpoint_bins,
            workgroup_count(topology.edge_count as usize),
        );
        run_compute_pass(
            encoder,
            "amigo-npr-compact-owners-pass",
            &pipelines.compact_owners_pipeline,
            &bind_groups.compact_owners,
            workgroup_count(topology.edge_count as usize),
        );
        run_compute_pass(
            encoder,
            "amigo-npr-connect-paths-pass",
            &pipelines.connect_paths_pipeline,
            &bind_groups.connect_paths,
            workgroup_count(topology.edge_count as usize),
        );
        for _ in 0..2 {
            run_compute_pass(
                encoder,
                "amigo-npr-relax-path-owners-pass",
                &pipelines.relax_path_owners_pipeline,
                &bind_groups.relax_path_owners,
                workgroup_count(topology.edge_count as usize),
            );
        }
    }
    run_compute_pass(
        encoder,
        "amigo-npr-emit-path-segments-pass",
        &pipelines.emit_path_segments_pipeline,
        &bind_groups.emit_path_segments,
        workgroup_count(topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN),
    );
    run_compute_pass(
        encoder,
        "amigo-npr-build-strokes-pass",
        &pipelines.build_strokes_pipeline,
        &bind_groups.build_strokes,
        workgroup_count(topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN),
    );
    run_compute_pass(
        encoder,
        "amigo-npr-clamp-indirect-args-pass",
        &pipelines.clamp_indirect_args_pipeline,
        &bind_groups.clamp_indirect_args,
        1,
    );
}

fn run_compute_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    workgroups: u32,
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(workgroups, 1, 1);
}
