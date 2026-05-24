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

        let common_layouts =
            crate::renderer::service::layout_builders::create_common_bind_group_layouts(device);
        let texture_bind_group_layout = common_layouts.texture;
        let camera_visual_source_bind_group_layout = common_layouts.camera_visual_source;
        let focus_blur_texture_bind_group_layout = common_layouts.focus_blur_texture;
        let shutter_blur_texture_bind_group_layout = common_layouts.shutter_blur_texture;
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
        let texture_opaque_pipeline = create_color_pipeline(
            device,
            &texture_shader,
            &texture_pipeline_layout,
            format,
            "amigo-scene-texture-opaque-pipeline",
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            },
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

        let wet_reflections_texture_bind_group_layout = common_layouts.wet_reflections_texture;
        let wet_reflections_uniform_bind_group_layout = common_layouts.wet_reflections_uniform;
        let wet_reflections_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-wet-reflections-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&wet_reflections_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let focus_blur_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-scene-focus-blur-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&focus_blur_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let post_fx_pipeline_ctx =
            crate::renderer::service::post_fx::pipelines::WgpuPostFxPipelineCreateContext {
                device,
                format,
                texture_bind_group_layout: &texture_bind_group_layout,
                camera_visual_source_bind_group_layout: &camera_visual_source_bind_group_layout,
                focus_blur_texture_bind_group_layout: &focus_blur_texture_bind_group_layout,
                shutter_blur_texture_bind_group_layout: &shutter_blur_texture_bind_group_layout,
                wet_reflections_uniform_bind_group_layout: &wet_reflections_uniform_bind_group_layout,
                wet_reflections_pipeline_layout: &wet_reflections_pipeline_layout,
                focus_blur_pipeline_layout: &focus_blur_pipeline_layout,
            };
        let mut pipelines = crate::renderer::service::pipeline_registry::WgpuPipelineRegistry::default();
        pipelines.extend(
            crate::renderer::service::post_fx::pipelines::build_default_post_fx_pipelines(
                &post_fx_pipeline_ctx,
            ),
        );

        Self {
            color_alpha_pipeline,
            color_additive_pipeline,
            color_multiply_pipeline,
            color_screen_pipeline,
            texture_alpha_pipeline,
            texture_opaque_pipeline,
            texture_additive_pipeline,
            texture_multiply_pipeline,
            texture_screen_pipeline,
            texture_lighten_pipeline,
            texture_bind_group_layout,
            camera_visual_source_bind_group_layout,
            focus_blur_texture_bind_group_layout,
            shutter_blur_texture_bind_group_layout,
            wet_reflections_texture_bind_group_layout,
            wet_reflections_uniform_bind_group_layout,
            pipelines,
            post_fx_executors: crate::renderer::service::post_fx::default_post_fx_executor_registry(),
            shutter_blur_runtimes: BTreeMap::new(),
            rain_glass_runtimes: BTreeMap::new(),
            texture_cache: BTreeMap::new(),
            lightmap_2d_image_cache: BTreeMap::new(),
            font_atlas_cache: BTreeMap::new(),
            font_fallback_warnings: BTreeSet::new(),
            frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor::default(),
            emergency_overlay_lines: Vec::new(),
            visual_source_targets_2d: crate::renderer::service::WgpuVisualSourceTargets2d::default(
            ),
            visual_source_previous_positions_2d: BTreeMap::new(),
            plate_relight_last_summary: "plate_relight: not run yet".to_owned(),
            render_materials_last_summary: "render.materials: not run yet".to_owned(),
        }
    }
}

