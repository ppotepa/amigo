use crate::renderer::*;

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        Self::new_with_device(&surface.device, surface.config.format)
    }

    pub fn new_for_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::new_with_device(&target.device, target.format)
    }

    fn new_with_device(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let common_layouts =
            crate::renderer::service::layout_builders::create_common_bind_group_layouts(device);
        let texture_bind_group_layout = common_layouts.texture;
        let camera_visual_source_bind_group_layout = common_layouts.camera_visual_source;
        let focus_blur_texture_bind_group_layout = common_layouts.focus_blur_texture;
        let shutter_blur_texture_bind_group_layout = common_layouts.shutter_blur_texture;

        let wet_reflections_texture_bind_group_layout = common_layouts.wet_reflections_texture;
        let wet_reflections_uniform_bind_group_layout = common_layouts.wet_reflections_uniform;
        let wet_reflections_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-wet-reflections-pipeline-layout"),
                bind_group_layouts: &[
                    Some(&wet_reflections_texture_bind_group_layout),
                    Some(&wet_reflections_uniform_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let focus_blur_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("amigo-render-focus-blur-pipeline-layout"),
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
                wet_reflections_uniform_bind_group_layout:
                    &wet_reflections_uniform_bind_group_layout,
                wet_reflections_pipeline_layout: &wet_reflections_pipeline_layout,
                focus_blur_pipeline_layout: &focus_blur_pipeline_layout,
            };
        let mut pipelines =
            crate::renderer::service::pipeline_registry::WgpuPipelineRegistry::default();
        let core_pipeline_ctx = WgpuCorePipelineCreateContext {
            device,
            surface_format: format,
            texture_bind_group_layout: &texture_bind_group_layout,
        };
        pipelines.extend(build_default_core_pipelines(&core_pipeline_ctx));
        pipelines.extend(
            crate::renderer::service::post_fx::pipelines::build_default_post_fx_pipelines(
                &post_fx_pipeline_ctx,
            ),
        );

        Self {
            texture_bind_group_layout,
            camera_visual_source_bind_group_layout,
            focus_blur_texture_bind_group_layout,
            shutter_blur_texture_bind_group_layout,
            wet_reflections_texture_bind_group_layout,
            wet_reflections_uniform_bind_group_layout,
            pipelines,
            npr_pipelines: crate::renderer::npr::NprPipelines::new(device, format),
            post_fx_executors:
                crate::renderer::service::post_fx::default_wgpu_screen_effect_executors(),
            shutter_blur_runtimes: BTreeMap::new(),
            rain_glass_runtimes: BTreeMap::new(),
            texture_cache: BTreeMap::new(),
            lightmap_2d_image_cache: BTreeMap::new(),
            font_atlas_cache: BTreeMap::new(),
            font_missing_glyph_warnings: BTreeSet::new(),
            frame_graph_executor: crate::renderer::graph::WgpuFrameGraphExecutor::default(),
            emergency_overlay_lines: Vec::new(),
            visual_source_targets_2d: crate::renderer::service::WgpuVisualSourceTargets2d::default(
            ),
            visual_source_previous_positions_2d: BTreeMap::new(),
            plate_relight_last_summary: "plate_relight: not run yet".to_owned(),
            render_materials_last_summary: "render.materials: not run yet".to_owned(),
            frame_diagnostics: Vec::new(),
        }
    }
}
