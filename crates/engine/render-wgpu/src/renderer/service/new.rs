use crate::renderer::*;

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        Self::new_with_device(&surface.device, surface.config.format)
    }

    pub fn new_for_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::new_with_device(&target.device, target.format)
    }

    fn new_with_device(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let color_shader = device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-color-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
            });
        let color_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-color-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let color_alpha_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[ColorVertex::layout()],
        );
        let color_additive_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-additive-pipeline",
            additive_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_multiply_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-multiply-pipeline",
            multiply_blend_state(),
            &[ColorVertex::layout()],
        );
        let color_screen_pipeline = create_color_pipeline(
            device,
            &color_shader,
            &color_pipeline_layout,
            format,
            "amigo-scene-color-screen-pipeline",
            screen_blend_state(),
            &[ColorVertex::layout()],
        );

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-scene-texture-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let texture_shader = device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-texture-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEXTURE_SHADER)),
            });
        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let texture_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("amigo-scene-texture-pipeline"),
                    layout: Some(&texture_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &texture_shader,
                        entry_point: Some("vs_main"),
                        buffers: &[TextureVertex::layout()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &texture_shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });

        Self {
            color_alpha_pipeline,
            color_additive_pipeline,
            color_multiply_pipeline,
            color_screen_pipeline,
            texture_pipeline,
            texture_bind_group_layout,
            texture_cache: BTreeMap::new(),
        }
    }
}
