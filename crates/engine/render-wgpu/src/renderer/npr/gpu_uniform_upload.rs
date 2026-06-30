use amigo_math::Transform3;

use crate::renderer::{GpuNprFrameUniforms3d, NprDebugOverlay3d, Viewport};

use super::{
    NprGpuMeshJob3d, create_job_uniform_buffer, npr_gpu_path_segment_capacity_units,
    npr_gpu_trace_line, slice_as_bytes, uniforms_are_finite, uniforms_for_job,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_npr_gpu_job_uniform_buffers(
    job_uniform_buffers: &mut Vec<wgpu::Buffer>,
    frame_jobs: &[NprGpuMeshJob3d],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    overlay: Option<NprDebugOverlay3d>,
    debug_frame_index: u64,
    trace: bool,
) {
    let uniform_size = std::mem::size_of::<GpuNprFrameUniforms3d>() as u64;
    while job_uniform_buffers.len() < frame_jobs.len() {
        let job_index = job_uniform_buffers.len();
        if trace {
            npr_gpu_trace_line(
                debug_frame_index,
                "ALLOC",
                format!("job[{job_index}] create persistent uniform buffer bytes={uniform_size}"),
            );
        }
        job_uniform_buffers.push(create_job_uniform_buffer(device, uniform_size));
    }

    let mut face_id_base = 0u32;
    let mut path_segment_base = 0u32;
    let job_count = frame_jobs.len().max(1);
    for (job_index, job) in frame_jobs.iter().enumerate() {
        let edge_count = job.geometry.edge_count() as u32;
        let path_segment_slot_count =
            npr_gpu_path_segment_capacity_units(job.geometry.edge_count(), job_count) as u32;
        let uniforms = uniforms_for_job(
            viewport,
            camera,
            camera_settings,
            job.geometry.as_ref(),
            job.transform,
            &job.settings,
            face_id_base,
            path_segment_base,
            path_segment_slot_count,
            job.geometry.vertex_count() as u32,
            job.geometry.triangle_count() as u32,
            edge_count,
            overlay,
        );
        if trace {
            npr_gpu_trace_line(
                debug_frame_index,
                "WRITE",
                format!(
                    "job[{job_index}] write persistent uniforms entity={} face_id_base={} triangles={} path_segment_base={} path_segment_slots={} pipeline0={:?} pipeline1={:?} finite={} cam_near={:.3} cam_far={:.3} focus01={:.3} bytes={uniform_size}",
                    job.entity_name,
                    face_id_base,
                    job.geometry.triangle_count(),
                    path_segment_base,
                    path_segment_slot_count,
                    uniforms.pipeline0,
                    uniforms.pipeline1,
                    uniforms_are_finite(&uniforms),
                    uniforms.params20[1],
                    uniforms.params20[2],
                    uniforms.params20[3],
                ),
            );
        }
        queue.write_buffer(
            &job_uniform_buffers[job_index],
            0,
            slice_as_bytes(std::slice::from_ref(&uniforms)),
        );
        if trace {
            npr_gpu_trace_line(
                debug_frame_index,
                "OK",
                format!(
                    "job[{job_index}] wrote persistent uniforms entity={}",
                    job.entity_name
                ),
            );
        }
        face_id_base = face_id_base.saturating_add(job.geometry.triangle_count() as u32);
        path_segment_base = path_segment_base.saturating_add(path_segment_slot_count);
    }
}
