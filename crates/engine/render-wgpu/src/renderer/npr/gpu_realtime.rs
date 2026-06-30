use amigo_assets::AssetKey;
use amigo_math::Transform3;
use std::sync::Arc;

use crate::renderer::{CachedMeshGeometry3d, NprDebugOverlay3d, Viewport};

use super::{
    NprGpuFrameStats3d, NprGpuMeshJob3d, NprGpuResources3d, build_npr_gpu_frame_plan,
    npr_gpu_path_segment_capacity_units, npr_gpu_trace_enabled, npr_gpu_trace_frame_plan,
    npr_gpu_trace_frame_start, npr_gpu_trace_line, render_npr_gpu_face_id_pass,
    run_npr_gpu_stroke_compute_passes, topology_cache_key, write_npr_gpu_indirect_args,
    write_npr_gpu_job_uniform_buffers,
};

#[derive(Debug, Default)]
pub(crate) struct GpuRealtimeNprRenderer3d {
    pub(crate) resources: NprGpuResources3d,
    pub(crate) pipelines: Option<super::NprGpuPipelines3d>,
    pub(crate) frame_jobs: Vec<NprGpuMeshJob3d>,
    pub(crate) job_uniform_buffers: Vec<wgpu::Buffer>,
    pub(crate) last_frame_has_draw_output: bool,
    pub(crate) debug_frame_index: u64,
}

impl GpuRealtimeNprRenderer3d {
    pub(crate) fn begin_frame(&mut self) {
        self.frame_jobs.clear();
        self.last_frame_has_draw_output = false;
    }

    pub(crate) fn enqueue_mesh(
        &mut self,
        entity_name: &str,
        mesh_key: &AssetKey,
        geometry: &Arc<CachedMeshGeometry3d>,
        transform: Transform3,
        settings: &amigo_render_api::NprLineSettings3d,
    ) -> Result<(), String> {
        self.frame_jobs.push(NprGpuMeshJob3d {
            entity_name: entity_name.to_owned(),
            mesh_key: mesh_key.as_str().to_owned(),
            geometry: Arc::clone(geometry),
            transform,
            settings: settings.clone(),
        });
        Ok(())
    }

    pub(crate) fn execute(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport: &Viewport,
        camera: Transform3,
        camera_settings: amigo_render_api::Camera3dRenderSettings,
        overlay: Option<NprDebugOverlay3d>,
    ) -> Result<NprGpuFrameStats3d, String> {
        if self.frame_jobs.is_empty() {
            return Ok(NprGpuFrameStats3d::default());
        }
        self.debug_frame_index = self.debug_frame_index.saturating_add(1);
        let debug_frame_index = self.debug_frame_index;
        let trace = npr_gpu_trace_enabled(debug_frame_index);
        if trace {
            npr_gpu_trace_frame_start(debug_frame_index, &self.frame_jobs, viewport, overlay);
        }
        if trace {
            npr_gpu_trace_line(debug_frame_index, "STEP", "ensure pipelines");
        }
        if self.pipelines.is_none() {
            self.pipelines = Some(super::NprGpuPipelines3d::create(device));
        }
        let frame_plan = build_npr_gpu_frame_plan(&self.frame_jobs, viewport);
        let mut topology_uploads = 0usize;
        for job in &self.frame_jobs {
            if self
                .resources
                .ensure_topology_uploaded(device, &job.mesh_key, &job.geometry)
            {
                topology_uploads += 1;
                if trace {
                    npr_gpu_trace_line(
                        debug_frame_index,
                        "ALLOC",
                        format!(
                            "topology uploaded mesh={} vertices={} triangles={} edges={}",
                            job.mesh_key,
                            job.geometry.vertex_count(),
                            job.geometry.triangle_count(),
                            job.geometry.edge_count()
                        ),
                    );
                }
            }
        }
        if trace {
            npr_gpu_trace_frame_plan(debug_frame_index, frame_plan);
        }
        self.resources.ensure_frame_buffers(
            device,
            frame_plan.allocated_projected_capacity(),
            frame_plan.allocated_visible_segments_capacity(),
            frame_plan.allocated_endpoint_heads_capacity(),
            frame_plan.allocated_endpoint_entries_capacity(),
            frame_plan.allocated_path_links_capacity(),
            frame_plan.allocated_path_segments_capacity(),
            frame_plan.allocated_path_states_capacity(),
            frame_plan.allocated_aggregated_paths_capacity(),
            frame_plan.allocated_stroke_segments_capacity(),
        );
        let (face_id_view, depth_view) = {
            let target = self.resources.ensure_face_id_target(
                device,
                frame_plan.target_width,
                frame_plan.target_height,
            );
            if trace {
                npr_gpu_trace_line(
                    debug_frame_index,
                    "ALLOC",
                    format!("face-id target {}x{}", target.width, target.height),
                );
            }
            (target.face_id_view.clone(), target.depth_view.clone())
        };
        write_npr_gpu_job_uniform_buffers(
            &mut self.job_uniform_buffers,
            &self.frame_jobs,
            device,
            queue,
            viewport,
            camera,
            camera_settings,
            overlay,
            debug_frame_index,
            trace,
        );
        let frame_buffers = self
            .resources
            .frame_buffers
            .as_ref()
            .expect("frame buffers should exist after ensure");
        let job_uniform_buffers = &self.job_uniform_buffers;
        let pipelines = self
            .pipelines
            .as_ref()
            .expect("npr gpu pipelines should exist after ensure");
        if trace {
            npr_gpu_trace_line(debug_frame_index, "WRITE", "write indirect args");
        }
        write_npr_gpu_indirect_args(queue, frame_buffers, frame_plan.stroke_segments_capacity);

        let mut clear_face_id = true;
        let job_count = self.frame_jobs.len().max(1);
        for (job_index, job) in self.frame_jobs.iter().enumerate() {
            let topology = self
                .resources
                .topology_cache
                .get(&topology_cache_key(&job.mesh_key, &job.geometry))
                .ok_or_else(|| {
                    format!(
                        "missing uploaded gpu topology cache for `{}`",
                        job.entity_name
                    )
                })?;
            let uniforms = &job_uniform_buffers[job_index];
            render_npr_gpu_face_id_pass(
                device,
                encoder,
                pipelines,
                topology,
                uniforms,
                &face_id_view,
                &depth_view,
                clear_face_id,
            );
            clear_face_id = false;
        }

        for (job_index, job) in self.frame_jobs.iter().enumerate() {
            let topology = self
                .resources
                .topology_cache
                .get(&topology_cache_key(&job.mesh_key, &job.geometry))
                .ok_or_else(|| {
                    format!(
                        "missing uploaded gpu topology cache for `{}`",
                        job.entity_name
                    )
                })?;
            let uniforms = &job_uniform_buffers[job_index];
            let path_segment_slot_count =
                npr_gpu_path_segment_capacity_units(job.geometry.edge_count(), job_count);
            run_npr_gpu_stroke_compute_passes(
                device,
                encoder,
                pipelines,
                topology,
                frame_buffers,
                uniforms,
                &face_id_view,
                job.settings.pipeline.path_strategy,
                path_segment_slot_count,
            );
        }

        self.last_frame_has_draw_output = !self.frame_jobs.is_empty();
        let buffer_capacities = self.resources.frame_buffer_capacities();
        let debug_mode = if self.frame_jobs.is_empty() {
            "none"
        } else {
            let first = self.frame_jobs[0]
                .settings
                .gpu_realtime_tuning
                .debug_mode
                .as_str();
            if self
                .frame_jobs
                .iter()
                .all(|job| job.settings.gpu_realtime_tuning.debug_mode.as_str() == first)
            {
                first
            } else {
                "mixed"
            }
        };
        Ok(NprGpuFrameStats3d {
            meshes: self.frame_jobs.len(),
            projected_vertices: frame_plan.total_vertices,
            classified_edges: frame_plan.total_edges,
            built_stroke_capacity: frame_plan.built_stroke_capacity_units(),
            enqueued_triangles: frame_plan.total_triangles,
            topology_uploads,
            buffer_capacity_bytes: self.resources.frame_buffer_capacity_bytes(),
            frame_jobs: self.frame_jobs.len(),
            projected_vertices_capacity: buffer_capacities.projected_vertices,
            visible_segments_capacity: buffer_capacities.visible_segments,
            endpoint_heads_capacity: buffer_capacities.endpoint_heads,
            endpoint_entries_capacity: buffer_capacities.endpoint_entries,
            path_links_capacity: buffer_capacities.path_links,
            path_segments_capacity: buffer_capacities.path_segments,
            path_states_capacity: buffer_capacities.path_states,
            aggregated_paths_capacity: buffer_capacities.aggregated_paths,
            stroke_segments_capacity: buffer_capacities.stroke_segments,
            debug_mode,
        })
    }

    pub(crate) fn has_draw_output(&self) -> bool {
        self.last_frame_has_draw_output
            && self
                .resources
                .frame_buffers
                .as_ref()
                .is_some_and(|buffers| buffers.stroke_segments_capacity > 0)
    }

    pub(crate) fn draw_to_offscreen_pass<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        pipeline: &'a wgpu::RenderPipeline,
    ) {
        let Some(frame_buffers) = self.resources.frame_buffers.as_ref() else {
            return;
        };
        pass.set_pipeline(pipeline);
        pass.set_vertex_buffer(0, frame_buffers.stroke_segments.slice(..));
        pass.draw_indirect(&frame_buffers.indirect_args, 0);
    }
}
