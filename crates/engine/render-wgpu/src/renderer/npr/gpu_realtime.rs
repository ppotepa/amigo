use amigo_assets::AssetKey;
use amigo_math::{Transform3, Vec3};
use std::sync::Arc;

use crate::renderer::{
    CachedMeshGeometry3d, GpuNprFrameUniforms3d, NprDebugOverlay3d, NprLineKind, Viewport,
};

use super::NprGpuResources3d;

const NPR_GPU_SEGMENTS_PER_STROKE_PASS: usize = 3;
const NPR_GPU_PATH_SEGMENTS_PER_CHAIN: usize = 12;

#[derive(Debug, Clone)]
pub(crate) struct NprGpuMeshJob3d {
    pub entity_name: String,
    pub mesh_key: String,
    pub geometry: Arc<CachedMeshGeometry3d>,
    pub transform: Transform3,
    pub settings: amigo_render_api::NprLineSettings3d,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NprGpuFrameStats3d {
    pub meshes: usize,
    pub projected_vertices: usize,
    pub classified_edges: usize,
    pub built_stroke_capacity: usize,
    pub enqueued_triangles: usize,
    pub topology_uploads: usize,
    pub buffer_capacity_bytes: u64,
    pub frame_jobs: usize,
    pub projected_vertices_capacity: usize,
    pub visible_segments_capacity: usize,
    pub endpoint_heads_capacity: usize,
    pub endpoint_entries_capacity: usize,
    pub path_links_capacity: usize,
    pub path_segments_capacity: usize,
    pub path_states_capacity: usize,
    pub stroke_segments_capacity: usize,
    pub debug_mode: &'static str,
}

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
            npr_gpu_trace_header(debug_frame_index);
            npr_gpu_trace_line(
                debug_frame_index,
                "START",
                format!(
                    "begin jobs={} viewport=({:.0}x{:.0}) overlay={:?}",
                    self.frame_jobs.len(),
                    viewport.size().x,
                    viewport.size().y,
                    overlay
                ),
            );
            for (index, job) in self.frame_jobs.iter().enumerate() {
                npr_gpu_trace_line(
                    debug_frame_index,
                    "JOB",
                    format!(
                        "job[{index}] entity={} mesh={} vertices={} triangles={} edges={} strategy={} preset={} tool={} visibility_max={:.1} fill={:?}",
                        job.entity_name,
                        job.mesh_key,
                        job.geometry.vertex_count(),
                        job.geometry.triangle_count(),
                        job.geometry.edge_count(),
                        job.settings.render_strategy.as_str(),
                        job.settings.style_preset.as_str(),
                        job.settings.stroke_tool.as_str(),
                        job.settings.visibility_max_dimension_px,
                        job.settings.fill_mode,
                    ),
                );
            }
        }
        if trace {
            npr_gpu_trace_line(debug_frame_index, "STEP", "ensure pipelines");
        }
        if self.pipelines.is_none() {
            self.pipelines = Some(super::NprGpuPipelines3d::create(device));
        }
        let total_vertices = self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.vertex_count())
            .sum::<usize>();
        let total_edges = self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count())
            .sum::<usize>();
        let total_triangles = self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.triangle_count())
            .sum::<usize>();
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
        let max_visibility_dimension_px = self
            .frame_jobs
            .iter()
            .map(|job| job.settings.visibility_max_dimension_px.max(1.0))
            .fold(1.0, f32::max);
        let (target_width, target_height) =
            scaled_face_id_dimensions(viewport, max_visibility_dimension_px);
        let projected_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.vertex_count())
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprProjectedVertex3d>())
            as u64;
        let visible_segments_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count())
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprVisibleSegment3d>())
            as u64;
        let endpoint_head_count = npr_gpu_endpoint_head_count(total_edges.max(1));
        let endpoint_heads_capacity =
            endpoint_head_count as u64 * std::mem::size_of::<u32>() as u64;
        let endpoint_entries_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count() * 2)
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprEndpointEntry3d>())
            as u64;
        let path_links_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count())
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprPathLink3d>())
            as u64;
        let path_segments_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count() * NPR_GPU_PATH_SEGMENTS_PER_CHAIN)
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprPathSegment3d>())
            as u64;
        let path_states_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count())
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprPathState3d>())
            as u64;
        let stroke_segments_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| {
                npr_gpu_stroke_segment_capacity_units(
                    job.geometry.edge_count(),
                    npr_gpu_pass_count(&job.settings),
                    job.settings.pipeline.budget_strategy,
                )
            })
            .sum::<usize>()
            * std::mem::size_of::<crate::renderer::NprStrokeSegmentVertex>())
            as u64;
        if trace {
            npr_gpu_trace_line(
                debug_frame_index,
                "ALLOC",
                format!(
                    "capacities bytes projected={} visible={} endpoint_heads={} endpoint_entries={} path_links={} path_segments={} path_states={} stroke_segments={} uniform_size={}",
                    projected_capacity.max(64),
                    visible_segments_capacity.max(64),
                    endpoint_heads_capacity.max(64),
                    endpoint_entries_capacity.max(64),
                    path_links_capacity.max(64),
                    path_segments_capacity.max(64),
                    path_states_capacity.max(64),
                    stroke_segments_capacity.max(64),
                    std::mem::size_of::<GpuNprFrameUniforms3d>(),
                ),
            );
        }
        self.resources.ensure_frame_buffers(
            device,
            projected_capacity.max(64),
            visible_segments_capacity.max(64),
            endpoint_heads_capacity.max(64),
            endpoint_entries_capacity.max(64),
            path_links_capacity.max(64),
            path_segments_capacity.max(64),
            path_states_capacity.max(64),
            stroke_segments_capacity.max(64),
        );
        let (face_id_view, depth_view) = {
            let target = self
                .resources
                .ensure_face_id_target(device, target_width, target_height);
            if trace {
                npr_gpu_trace_line(
                    debug_frame_index,
                    "ALLOC",
                    format!("face-id target {}x{}", target.width, target.height),
                );
            }
            (target.face_id_view.clone(), target.depth_view.clone())
        };
        self.write_job_uniform_buffers(
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
        let stroke_segment_capacity_units = (stroke_segments_capacity
            / std::mem::size_of::<crate::renderer::NprStrokeSegmentVertex>() as u64)
            as u32;
        queue.write_buffer(
            &frame_buffers.indirect_args,
            0,
            slice_as_bytes(&[
                6u32,
                0u32,
                0u32,
                0u32,
                0u32,
                stroke_segment_capacity_units,
            ]),
        );

        let mut face_id_base = 0u32;
        let mut clear_face_id = true;
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
            let face_id_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-face-id-bind-group"),
                layout: &pipelines.face_id_bind_group_layout,
                entries: &[
                    storage_binding(0, &topology.vertices),
                    storage_binding(1, &topology.triangles),
                    uniform_binding(8, uniforms),
                ],
            });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("amigo-npr-face-id-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &face_id_view,
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
                        view: &depth_view,
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
            clear_face_id = false;
            face_id_base = face_id_base.saturating_add(topology.triangle_count);
        }

        face_id_base = 0u32;
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
            let project_vertices_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-npr-project-vertices-bind-group"),
                    layout: &pipelines.project_vertices_bind_group_layout,
                    entries: &[
                        storage_binding(0, &topology.vertices),
                        storage_binding(3, &frame_buffers.projected_vertices),
                        uniform_binding(8, uniforms),
                    ],
                });
            let classify_edges_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-classify-edges-bind-group"),
                layout: &pipelines.classify_edges_bind_group_layout,
                entries: &[
                    storage_binding(0, &topology.vertices),
                    storage_binding(1, &topology.triangles),
                    storage_binding(2, &topology.edges),
                    storage_binding(3, &frame_buffers.projected_vertices),
                    texture_binding(4, &face_id_view),
                    storage_binding(5, &frame_buffers.visible_segments),
                    uniform_binding(8, uniforms),
                ],
            });
            let build_endpoint_bins_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-npr-build-endpoint-bins-bind-group"),
                    layout: &pipelines.build_endpoint_bins_bind_group_layout,
                    entries: &[
                        storage_binding(5, &frame_buffers.visible_segments),
                        uniform_binding(8, uniforms),
                        storage_binding(11, &frame_buffers.endpoint_heads),
                        storage_binding(12, &frame_buffers.endpoint_entries),
                    ],
                });
            let clear_endpoint_heads_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-npr-clear-endpoint-heads-bind-group"),
                    layout: &pipelines.clear_endpoint_heads_bind_group_layout,
                    entries: &[storage_binding(11, &frame_buffers.endpoint_heads)],
                });
            let compact_owners_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            let connect_paths_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-connect-paths-bind-group"),
                layout: &pipelines.connect_paths_bind_group_layout,
                entries: &[
                    storage_binding(5, &frame_buffers.visible_segments),
                    storage_binding(10, &frame_buffers.path_links),
                    storage_binding(14, &frame_buffers.path_states),
                ],
            });
            let relax_path_owners_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-npr-relax-path-owners-bind-group"),
                    layout: &pipelines.relax_path_owners_bind_group_layout,
                    entries: &[
                        storage_binding(5, &frame_buffers.visible_segments),
                        storage_binding(10, &frame_buffers.path_links),
                        storage_binding(14, &frame_buffers.path_states),
                    ],
                });
            let emit_path_segments_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            let build_strokes_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-build-strokes-bind-group"),
                layout: &pipelines.build_strokes_bind_group_layout,
                entries: &[
                    storage_binding(2, &topology.edges),
                    storage_binding(5, &frame_buffers.visible_segments),
                    storage_binding(6, &frame_buffers.stroke_segments),
                    uniform_binding(8, uniforms),
                    storage_binding(9, &frame_buffers.indirect_args),
                    storage_binding(13, &frame_buffers.path_segments),
                ],
            });
            let clamp_indirect_args_bind_group =
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-npr-clamp-indirect-args-bind-group"),
                    layout: &pipelines.clamp_indirect_args_bind_group_layout,
                    entries: &[storage_binding(9, &frame_buffers.indirect_args)],
                });

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-project-vertices-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.project_vertices_pipeline);
                pass.set_bind_group(0, &project_vertices_bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(topology.vertex_count as usize), 1, 1);
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-classify-edges-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.classify_edges_pipeline);
                pass.set_bind_group(0, &classify_edges_bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
            }
            if job.settings.pipeline.path_strategy
                != amigo_render_api::NprPathStrategy3d::DirectVisibleSegments
            {
                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("amigo-npr-clear-endpoint-heads-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipelines.clear_endpoint_heads_pipeline);
                    pass.set_bind_group(0, &clear_endpoint_heads_bind_group, &[]);
                    pass.dispatch_workgroups(
                        workgroup_count(
                            (frame_buffers.endpoint_heads_capacity
                                / std::mem::size_of::<u32>() as u64)
                                as usize,
                        ),
                        1,
                        1,
                    );
                }
                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("amigo-npr-build-endpoint-bins-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipelines.build_endpoint_bins_pipeline);
                    pass.set_bind_group(0, &build_endpoint_bins_bind_group, &[]);
                    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
                }
                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("amigo-npr-compact-owners-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipelines.compact_owners_pipeline);
                    pass.set_bind_group(0, &compact_owners_bind_group, &[]);
                    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
                }
                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("amigo-npr-connect-paths-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipelines.connect_paths_pipeline);
                    pass.set_bind_group(0, &connect_paths_bind_group, &[]);
                    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
                }
                for _ in 0..2 {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("amigo-npr-relax-path-owners-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(&pipelines.relax_path_owners_pipeline);
                    pass.set_bind_group(0, &relax_path_owners_bind_group, &[]);
                    pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
                }
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-emit-path-segments-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.emit_path_segments_pipeline);
                pass.set_bind_group(0, &emit_path_segments_bind_group, &[]);
                pass.dispatch_workgroups(
                    workgroup_count(topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN),
                    1,
                    1,
                );
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-build-strokes-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.build_strokes_pipeline);
                pass.set_bind_group(0, &build_strokes_bind_group, &[]);
                pass.dispatch_workgroups(
                    workgroup_count(topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN),
                    1,
                    1,
                );
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-clamp-indirect-args-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.clamp_indirect_args_pipeline);
                pass.set_bind_group(0, &clamp_indirect_args_bind_group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
            }
            face_id_base = face_id_base.saturating_add(topology.triangle_count);
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
            projected_vertices: total_vertices,
            classified_edges: total_edges,
            built_stroke_capacity: (stroke_segments_capacity
                / std::mem::size_of::<crate::renderer::NprStrokeSegmentVertex>() as u64)
                as usize,
            enqueued_triangles: total_triangles,
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
            stroke_segments_capacity: buffer_capacities.stroke_segments,
            debug_mode,
        })
    }

    fn write_job_uniform_buffers(
        &mut self,
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
        while self.job_uniform_buffers.len() < self.frame_jobs.len() {
            let job_index = self.job_uniform_buffers.len();
            if trace {
                npr_gpu_trace_line(
                    debug_frame_index,
                    "ALLOC",
                    format!(
                        "job[{job_index}] create persistent uniform buffer bytes={uniform_size}"
                    ),
                );
            }
            self.job_uniform_buffers
                .push(create_job_uniform_buffer(device, uniform_size));
        }

        let mut face_id_base = 0u32;
        let mut path_segment_base = 0u32;
        for (job_index, job) in self.frame_jobs.iter().enumerate() {
            let edge_count = job.geometry.edge_count() as u32;
            let path_segment_slot_count =
                edge_count.saturating_mul(NPR_GPU_PATH_SEGMENTS_PER_CHAIN as u32);
            let uniforms = uniforms_for_job(
                viewport,
                camera,
                camera_settings,
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
                        "job[{job_index}] write persistent uniforms entity={} face_id_base={} triangles={} finite={} bytes={uniform_size}",
                        job.entity_name,
                        face_id_base,
                        job.geometry.triangle_count(),
                        uniforms_are_finite(&uniforms),
                    ),
                );
            }
            queue.write_buffer(
                &self.job_uniform_buffers[job_index],
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

fn scaled_face_id_dimensions(viewport: &Viewport, max_dimension_px: f32) -> (u32, u32) {
    let size = viewport.size();
    let scale = (max_dimension_px / size.x.max(size.y)).min(1.0);
    (
        (size.x * scale).round().max(1.0) as u32,
        (size.y * scale).round().max(1.0) as u32,
    )
}

fn slice_as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) }
}

fn npr_gpu_trace_enabled(frame_index: u64) -> bool {
    frame_index <= 4
        || std::env::var("AMIGO_NPR_GPU_TRACE")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
}

fn npr_gpu_trace_header(frame_index: u64) {
    if frame_index != 1 {
        return;
    }
    if npr_gpu_trace_clear_enabled() {
        print!("\x1b[2J\x1b[H");
    }
    npr_gpu_trace_line(frame_index, "START", "GPU NPR realtime trace");
    npr_gpu_trace_line(
        frame_index,
        "INFO",
        "env: AMIGO_NPR_GPU_TRACE=1 keeps logging, AMIGO_NPR_GPU_TRACE_CLEAR=0 disables clear, AMIGO_NPR_GPU_TRACE_COLOR=0 disables colors",
    );
}

fn npr_gpu_trace_line(frame_index: u64, level: &str, message: impl AsRef<str>) {
    let message = message.as_ref();
    if npr_gpu_trace_color_enabled() {
        let color = npr_gpu_trace_level_color(level);
        println!(
            "{color}[npr-gpu]\x1b[0m \x1b[2mframe={frame_index:04}\x1b[0m {color}{level:<5}\x1b[0m {message}"
        );
    } else {
        println!("[npr-gpu] frame={frame_index:04} {level:<5} {message}");
    }
}

fn npr_gpu_trace_clear_enabled() -> bool {
    !npr_gpu_trace_env_is_false("AMIGO_NPR_GPU_TRACE_CLEAR")
}

fn npr_gpu_trace_color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && !npr_gpu_trace_env_is_false("AMIGO_NPR_GPU_TRACE_COLOR")
}

fn npr_gpu_trace_env_is_false(name: &str) -> bool {
    std::env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
        .unwrap_or(false)
}

fn npr_gpu_trace_level_color(level: &str) -> &'static str {
    match level {
        "START" => "\x1b[1;36m",
        "INFO" => "\x1b[36m",
        "JOB" => "\x1b[32m",
        "STEP" => "\x1b[33m",
        "ALLOC" => "\x1b[35m",
        "WRITE" => "\x1b[34m",
        "OK" => "\x1b[32m",
        _ => "\x1b[37m",
    }
}

fn uniforms_are_finite(uniforms: &GpuNprFrameUniforms3d) -> bool {
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
        && uniforms.ink_color.iter().all(|value| value.is_finite())
}

fn create_job_uniform_buffer(device: &wgpu::Device, uniform_size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("amigo-npr-job-uniforms"),
        size: uniform_size.max(16),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn workgroup_count(items: usize) -> u32 {
    ((items.max(1) as u32).saturating_add(63)) / 64
}

fn npr_gpu_endpoint_head_count(edge_count: usize) -> usize {
    let target = (edge_count.max(1) * 4).next_power_of_two();
    target.max(64)
}

fn topology_cache_key(mesh_key: &str, geometry: &CachedMeshGeometry3d) -> String {
    format!(
        "{mesh_key}:{}:{}:{}",
        geometry.vertex_count(),
        geometry.triangle_count(),
        geometry.edge_count()
    )
}

fn uniforms_for_job(
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
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
    let primary_passes = settings.passes.min(8).max(1) as f32;
    let search_passes = if gpu_tuning.search_enabled {
        ((settings.search_line_count as f32) * super::npr_tool_search_multiplier(settings))
            .round()
            .clamp(0.0, 8.0)
    } else {
        0.0
    };
    let tool_width = super::npr_tool_width_multiplier(settings);
    let tool_alpha = super::npr_tool_alpha_multiplier(settings);
    let tool_pressure_jitter = super::npr_tool_pressure_jitter_multiplier(settings);
    let tool_dropout = super::npr_tool_dropout_multiplier(settings);
    let straightness_wobble = super::npr_straightness_wobble_multiplier(settings);
    let micro_wobble = settings.micro_wobble_px * settings.humanization * straightness_wobble;
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
        params6: [tool_width, tool_alpha, tool_pressure_jitter, tool_dropout],
        params7: [
            straightness_wobble,
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
            gpu_candidate_strategy(settings.pipeline.candidate_strategy),
            gpu_path_strategy(settings.pipeline.path_strategy),
            gpu_stroke_strategy(settings.pipeline.stroke_strategy),
            gpu_fill_strategy(settings.pipeline.fill_strategy),
        ],
        pipeline1: [
            gpu_hatching_strategy(settings.pipeline.hatching_strategy),
            gpu_budget_strategy(settings.pipeline.budget_strategy),
            gpu_temporal_strategy(settings.pipeline.temporal_strategy),
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

fn gpu_candidate_strategy(value: amigo_render_api::NprCandidateStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprCandidateStrategy3d::GeometryEdges => 0,
        amigo_render_api::NprCandidateStrategy3d::CharacterSemantic => 1,
    }
}

fn gpu_path_strategy(value: amigo_render_api::NprPathStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprPathStrategy3d::StableStrokedPaths => 0,
        amigo_render_api::NprPathStrategy3d::DirectVisibleSegments => 1,
    }
}

fn gpu_stroke_strategy(value: amigo_render_api::NprStrokeStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprStrokeStrategy3d::ComicInk => 0,
        amigo_render_api::NprStrokeStrategy3d::AkiraInk => 1,
        amigo_render_api::NprStrokeStrategy3d::TechnicalInk => 2,
        amigo_render_api::NprStrokeStrategy3d::RoughPencil => 3,
    }
}

fn gpu_fill_strategy(value: amigo_render_api::NprInkFillStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprInkFillStrategy3d::None => 0,
        amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass => 1,
        amigo_render_api::NprInkFillStrategy3d::BinaryMangaShadow => 2,
    }
}

fn gpu_hatching_strategy(value: amigo_render_api::NprHatchingStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprHatchingStrategy3d::None => 0,
        amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching => 1,
    }
}

fn gpu_budget_strategy(value: amigo_render_api::NprBudgetStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprBudgetStrategy3d::EdgeVisibility => 0,
        amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority => 1,
        amigo_render_api::NprBudgetStrategy3d::CharacterReadability => 2,
    }
}

fn gpu_temporal_strategy(value: amigo_render_api::NprTemporalStrategy3d) -> u32 {
    match value {
        amigo_render_api::NprTemporalStrategy3d::PathHistory => 0,
        amigo_render_api::NprTemporalStrategy3d::StableArcLength => 1,
    }
}

fn gpu_material_id_mask(material_ids: &[u32]) -> u32 {
    material_ids
        .iter()
        .filter(|id| **id < 32)
        .fold(0u32, |mask, id| mask | (1u32 << *id))
}

fn npr_gpu_pass_count(settings: &amigo_render_api::NprLineSettings3d) -> u32 {
    let primary_passes = settings.passes.min(8).max(1) as u32;
    let search_passes = if settings.gpu_realtime_tuning.search_enabled {
        ((settings.search_line_count as f32) * super::npr_tool_search_multiplier(settings))
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

fn npr_gpu_stroke_segment_capacity_units(
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

fn vec3_to_gpu4(value: Vec3, w: f32) -> [f32; 4] {
    [value.x, value.y, value.z, w]
}

fn storage_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn uniform_binding(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn texture_binding<'a>(binding: u32, view: &'a wgpu::TextureView) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

#[cfg(test)]
mod tests {
    use super::{npr_gpu_pass_count, uniforms_for_job, NPR_GPU_PATH_SEGMENTS_PER_CHAIN};
    use crate::renderer::Viewport;

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
            ..amigo_render_api::NprLineSettings3d::default()
        };

        let uniforms = uniforms_for_job(
            &Viewport::from_dimensions(1280.0, 720.0),
            amigo_math::Transform3::default(),
            amigo_render_api::Camera3dRenderSettings::default(),
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
    fn gpu_path_segment_chain_budget_matches_shader_slots() {
        assert_eq!(NPR_GPU_PATH_SEGMENTS_PER_CHAIN, 12);
    }

    #[test]
    fn gpu_stroke_segment_capacity_scales_with_path_segment_slots() {
        assert_eq!(
            super::npr_gpu_stroke_segment_capacity_units(
                10,
                2,
                amigo_render_api::NprBudgetStrategy3d::EdgeVisibility,
            ),
            10 * 4 * 2
        );
    }
}
