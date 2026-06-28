use crate::renderer::shaders::{
    NPR_BUILD_ENDPOINT_BINS_SHADER, NPR_BUILD_STROKES_SHADER, NPR_CLASSIFY_EDGES_SHADER,
    NPR_CLAMP_INDIRECT_ARGS_SHADER, NPR_CLEAR_ENDPOINT_HEADS_SHADER, NPR_COMPACT_OWNERS_SHADER,
    NPR_CONNECT_PATHS_SHADER, NPR_EMIT_PATH_SEGMENTS_SHADER, NPR_FACE_ID_SHADER,
    NPR_PROJECT_VERTICES_SHADER, NPR_RELAX_PATH_OWNERS_SHADER,
};

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

impl NprGpuPipelines3d {
    pub(crate) fn create(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-npr-face-id-shader"),
            source: wgpu::ShaderSource::Wgsl(NPR_FACE_ID_SHADER.into()),
        });
        let face_id_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-npr-face-id-bind-group-layout"),
                entries: &[
                    vertex_storage_entry(0),
                    vertex_storage_entry(1),
                    vertex_uniform_entry(8),
                ],
            });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("amigo-npr-face-id-pipeline-layout"),
            bind_group_layouts: &[Some(&face_id_bind_group_layout)],
            immediate_size: 0,
        });
        let face_id_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("amigo-npr-face-id-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Uint,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
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
        Self {
            face_id_pipeline,
            face_id_bind_group_layout,
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
}

fn create_compute_bind_group_layout(
    device: &wgpu::Device,
    label: &'static str,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries,
    })
}

fn create_compute_pipeline_layout(
    device: &wgpu::Device,
    label: &'static str,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    })
}

fn create_compute_pipeline(
    device: &wgpu::Device,
    label: &'static str,
    source: &str,
    layout: &wgpu::PipelineLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn vertex_storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn vertex_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}
