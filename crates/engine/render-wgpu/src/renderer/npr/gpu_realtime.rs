use amigo_assets::AssetKey;
use amigo_math::{Transform3, Vec3};
use std::sync::Arc;

use crate::renderer::{
    CachedMeshGeometry3d, GpuNprFrameUniforms3d, NprDebugOverlay3d, NprLineKind, Viewport,
};

use super::NprGpuResources3d;

const NPR_GPU_SEGMENTS_PER_STROKE_PASS: usize = 2;
const NPR_GPU_PATH_SEGMENTS_PER_CHAIN: usize = 8;

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
}

#[derive(Debug, Default)]
pub(crate) struct GpuRealtimeNprRenderer3d {
    pub(crate) resources: NprGpuResources3d,
    pub(crate) pipelines: Option<super::NprGpuPipelines3d>,
    pub(crate) frame_jobs: Vec<NprGpuMeshJob3d>,
    pub(crate) last_frame_has_draw_output: bool,
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
        let pipelines = self
            .pipelines
            .get_or_insert_with(|| super::NprGpuPipelines3d::create(device));
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
            }
        }
        let max_visibility_dimension_px = self
            .frame_jobs
            .iter()
            .map(|job| job.settings.visibility_max_dimension_px.max(1.0))
            .fold(1.0, f32::max);
        let (target_width, target_height) =
            scaled_face_id_dimensions(viewport, max_visibility_dimension_px);
        let projected_capacity =
            (self.frame_jobs.iter().map(|job| job.geometry.vertex_count()).sum::<usize>()
                * std::mem::size_of::<super::GpuNprProjectedVertex3d>()) as u64;
        let visible_segments_capacity =
            (self.frame_jobs.iter().map(|job| job.geometry.edge_count()).sum::<usize>()
                * std::mem::size_of::<super::GpuNprVisibleSegment3d>()) as u64;
        let endpoint_head_count = npr_gpu_endpoint_head_count(total_edges.max(1));
        let endpoint_heads_capacity =
            endpoint_head_count as u64 * std::mem::size_of::<u32>() as u64;
        let endpoint_entries_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| job.geometry.edge_count() * 2)
            .sum::<usize>()
            * std::mem::size_of::<super::GpuNprEndpointEntry3d>()) as u64;
        let path_links_capacity =
            (self.frame_jobs.iter().map(|job| job.geometry.edge_count()).sum::<usize>()
                * std::mem::size_of::<super::GpuNprPathLink3d>()) as u64;
        let path_segments_capacity =
            (self
                .frame_jobs
                .iter()
                .map(|job| job.geometry.edge_count() * NPR_GPU_PATH_SEGMENTS_PER_CHAIN)
                .sum::<usize>()
                * std::mem::size_of::<super::GpuNprPathSegment3d>()) as u64;
        let stroke_segments_capacity = (self
            .frame_jobs
            .iter()
            .map(|job| {
                job.geometry.edge_count()
                    * npr_gpu_pass_count(&job.settings) as usize
                    * NPR_GPU_SEGMENTS_PER_STROKE_PASS
            })
            .sum::<usize>()
            * std::mem::size_of::<crate::renderer::NprStrokeSegmentVertex>())
            as u64;
        self.resources.ensure_frame_buffers(
            device,
            projected_capacity.max(64),
            visible_segments_capacity.max(64),
            endpoint_heads_capacity.max(64),
            endpoint_entries_capacity.max(64),
            path_links_capacity.max(64),
            path_segments_capacity.max(64),
            stroke_segments_capacity.max(64),
        );
        let (face_id_view, depth_view) = {
            let target = self
                .resources
                .ensure_face_id_target(device, target_width, target_height);
            (target.face_id_view.clone(), target.depth_view.clone())
        };
        let frame_buffers = self
            .resources
            .frame_buffers
            .as_ref()
            .expect("frame buffers should exist after ensure");
        queue.write_buffer(
            &frame_buffers.indirect_args,
            0,
            slice_as_bytes(&[6u32, 0u32, 0u32, 0u32]),
        );

        let mut face_id_base = 0u32;
        let mut clear_face_id = true;
        for job in &self.frame_jobs {
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
            let uniforms = uniforms_for_job(
                viewport,
                camera,
                camera_settings,
                job.transform,
                &job.settings,
                face_id_base,
                overlay,
            );
            queue.write_buffer(&frame_buffers.uniforms, 0, slice_as_bytes(&[uniforms]));
            let face_id_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-face-id-bind-group"),
                layout: &pipelines.face_id_bind_group_layout,
                entries: &[
                    storage_binding(0, &topology.vertices),
                    storage_binding(1, &topology.triangles),
                    uniform_binding(8, &frame_buffers.uniforms),
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
        for job in &self.frame_jobs {
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
            let uniforms = uniforms_for_job(
                viewport,
                camera,
                camera_settings,
                job.transform,
                &job.settings,
                face_id_base,
                overlay,
            );
            queue.write_buffer(&frame_buffers.uniforms, 0, slice_as_bytes(&[uniforms]));
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("amigo-npr-compute-bind-group"),
                layout: &pipelines.compute_bind_group_layout,
                entries: &[
                    storage_binding(0, &topology.vertices),
                    storage_binding(1, &topology.triangles),
                    storage_binding(2, &topology.edges),
                    storage_binding(3, &frame_buffers.projected_vertices),
                    texture_binding(4, &face_id_view),
                    storage_binding(5, &frame_buffers.visible_segments),
                    storage_binding(6, &frame_buffers.stroke_segments),
                    uniform_binding(8, &frame_buffers.uniforms),
                    storage_binding(9, &frame_buffers.indirect_args),
                    storage_binding(10, &frame_buffers.path_links),
                    storage_binding(11, &frame_buffers.endpoint_heads),
                    storage_binding(12, &frame_buffers.endpoint_entries),
                    storage_binding(13, &frame_buffers.path_segments),
                ],
            });

            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-project-vertices-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.project_vertices_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(topology.vertex_count as usize), 1, 1);
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-classify-edges-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.classify_edges_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
            }
            {
                let zero_heads = vec![0u32; endpoint_head_count];
                queue.write_buffer(&frame_buffers.endpoint_heads, 0, slice_as_bytes(&zero_heads));
            }
            {
                let endpoint_items = topology.edge_count as usize * 2;
                let work_items = endpoint_head_count
                    .max(endpoint_items)
                    .max(topology.edge_count as usize);
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-build-endpoint-bins-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.build_endpoint_bins_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(work_items), 1, 1);
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-compact-owners-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.compact_owners_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroup_count(topology.edge_count as usize), 1, 1);
            }
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("amigo-npr-emit-path-segments-pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&pipelines.emit_path_segments_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(
                    workgroup_count(
                        topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN,
                    ),
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
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(
                    workgroup_count(
                        topology.edge_count as usize * NPR_GPU_PATH_SEGMENTS_PER_CHAIN,
                    ),
                    1,
                    1,
                );
            }
            face_id_base = face_id_base.saturating_add(topology.triangle_count);
        }

        self.last_frame_has_draw_output = !self.frame_jobs.is_empty();
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

fn scaled_face_id_dimensions(viewport: &Viewport, max_dimension_px: f32) -> (u32, u32) {
    let size = viewport.size();
    let scale = (max_dimension_px / size.x.max(size.y)).min(1.0);
    (
        (size.x * scale).round().max(1.0) as u32,
        (size.y * scale).round().max(1.0) as u32,
    )
}

fn slice_as_bytes<T>(data: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            data.as_ptr() as *const u8,
            std::mem::size_of_val(data),
        )
    }
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
        params5: [primary_passes, search_passes, settings.search_line_alpha, settings.taper],
        params6: [
            tool_width,
            tool_alpha,
            tool_pressure_jitter,
            tool_dropout,
        ],
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
    }
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
    primary_passes + search_passes
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

fn texture_binding<'a>(
    binding: u32,
    view: &'a wgpu::TextureView,
) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

#[cfg(test)]
mod tests {
    use super::npr_gpu_pass_count;

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
}
