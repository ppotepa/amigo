use crate::renderer::*;

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        Self::new_with_device(&surface.device, surface.config.format)
    }

    pub fn new_for_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::new_with_device(&target.device, target.format)
    }

    fn new_with_device(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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
        let texture_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("amigo-scene-texture-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEXTURE_SHADER)),
        });
        let texture_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-texture-pipeline-layout"),
                bind_group_layouts: &[Some(&texture_bind_group_layout)],
                immediate_size: 0,
            });
        let texture_alpha_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
            &[TextureVertex::layout()],
        );
        let texture_additive_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-additive-pipeline",
            additive_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_multiply_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-multiply-pipeline",
            multiply_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_screen_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-screen-pipeline",
            screen_blend_state(),
            &[TextureVertex::layout()],
        );
        let texture_lighten_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-lighten-pipeline",
            lighten_blend_state(),
            &[TextureVertex::layout()],
        );

        Self {
            color_alpha_pipeline,
            color_additive_pipeline,
            color_multiply_pipeline,
            color_screen_pipeline,
            texture_alpha_pipeline,
            texture_additive_pipeline,
            texture_multiply_pipeline,
            texture_screen_pipeline,
            texture_lighten_pipeline,
            texture_bind_group_layout,
            texture_cache: BTreeMap::new(),
        }
    }
}
