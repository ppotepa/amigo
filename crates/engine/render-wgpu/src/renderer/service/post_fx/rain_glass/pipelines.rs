use super::resources::RAIN_GLASS_OPTICAL_FORMAT;
use super::shaders::{
    RAIN_GLASS_BLUR_SHADER, RAIN_GLASS_COMPOSE_SHADER, RAIN_GLASS_ERASE_SHADER,
    RAIN_GLASS_FADE_SHADER, RAIN_GLASS_MIST_SHADER, RAIN_GLASS_STAMP_SHADER,
};
use super::types::RainGlassInstance;

pub(crate) struct RainGlassPipelines {
    pub stamp_smoother_pipeline: wgpu::RenderPipeline,
    pub stamp_harder_pipeline: wgpu::RenderPipeline,
    pub fade_pipeline: wgpu::RenderPipeline,
    pub erase_pipeline: wgpu::RenderPipeline,
    pub mist_pipeline: wgpu::RenderPipeline,
    pub blur_pipeline: wgpu::RenderPipeline,
    pub compose_pipeline: wgpu::RenderPipeline,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub map_bind_group_layout: wgpu::BindGroupLayout,
    pub erase_bind_group_layout: wgpu::BindGroupLayout,
    pub mist_bind_group_layout: wgpu::BindGroupLayout,
    pub compose_bind_group_layout: wgpu::BindGroupLayout,
    pub blur_direction_bind_group_layout: wgpu::BindGroupLayout,
}

impl RainGlassPipelines {
    pub(crate) fn new(device: &wgpu::Device, scene_format: wgpu::TextureFormat) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-uniform-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let map_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-map-layout"),
                entries: &[texture_entry(0), sampler_entry(1)],
            });
        let erase_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-erase-layout"),
                entries: &[texture_entry(0), texture_entry(1), sampler_entry(2)],
            });
        let mist_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-mist-layout"),
                entries: &[
                    texture_entry(0),
                    texture_entry(1),
                    texture_entry(2),
                    texture_entry(3),
                    sampler_entry(4),
                ],
            });
        let compose_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-compose-layout"),
                entries: &[
                    texture_entry(0),
                    texture_entry(1),
                    texture_entry(2),
                    texture_entry(3),
                    texture_entry(4),
                    texture_entry(5),
                    sampler_entry(6),
                ],
            });
        let blur_direction_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("amigo-rain-glass-blur-direction-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let stamp_smoother_pipeline = pipeline(
            device,
            "amigo-rain-glass-stamp-smoother",
            RAIN_GLASS_STAMP_SHADER,
            &[&uniform_bind_group_layout],
            RAIN_GLASS_OPTICAL_FORMAT,
            &[instance_layout()],
            Some(reference_smoother_blend()),
        );
        let stamp_harder_pipeline = pipeline(
            device,
            "amigo-rain-glass-stamp-harder",
            RAIN_GLASS_STAMP_SHADER,
            &[&uniform_bind_group_layout],
            RAIN_GLASS_OPTICAL_FORMAT,
            &[instance_layout()],
            Some(optical_premultiplied_blend()),
        );
        let fade_pipeline = pipeline(
            device,
            "amigo-rain-glass-fade",
            RAIN_GLASS_FADE_SHADER,
            &[&map_bind_group_layout, &uniform_bind_group_layout],
            RAIN_GLASS_OPTICAL_FORMAT,
            &[],
            None,
        );
        let erase_pipeline = pipeline(
            device,
            "amigo-rain-glass-erase",
            RAIN_GLASS_ERASE_SHADER,
            &[&erase_bind_group_layout, &uniform_bind_group_layout],
            RAIN_GLASS_OPTICAL_FORMAT,
            &[],
            None,
        );
        let mist_pipeline = pipeline(
            device,
            "amigo-rain-glass-mist",
            RAIN_GLASS_MIST_SHADER,
            &[&mist_bind_group_layout, &uniform_bind_group_layout],
            RAIN_GLASS_OPTICAL_FORMAT,
            &[],
            None,
        );
        let blur_pipeline = pipeline(
            device,
            "amigo-rain-glass-blur",
            RAIN_GLASS_BLUR_SHADER,
            &[
                &map_bind_group_layout,
                &uniform_bind_group_layout,
                &blur_direction_bind_group_layout,
            ],
            scene_format,
            &[],
            None,
        );
        let compose_pipeline = pipeline(
            device,
            "amigo-rain-glass-compose",
            RAIN_GLASS_COMPOSE_SHADER,
            &[&compose_bind_group_layout, &uniform_bind_group_layout],
            scene_format,
            &[],
            None,
        );

        Self {
            stamp_smoother_pipeline,
            stamp_harder_pipeline,
            fade_pipeline,
            erase_pipeline,
            mist_pipeline,
            blur_pipeline,
            compose_pipeline,
            uniform_bind_group_layout,
            map_bind_group_layout,
            erase_bind_group_layout,
            mist_bind_group_layout,
            compose_bind_group_layout,
            blur_direction_bind_group_layout,
        }
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<RainGlassInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    }
}

fn pipeline(
    device: &wgpu::Device,
    label: &str,
    shader_src: &str,
    layouts: &[&wgpu::BindGroupLayout],
    format: wgpu::TextureFormat,
    vertex_buffers: &[wgpu::VertexBufferLayout<'_>],
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{label}-shader")),
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });
    let bind_group_layouts = layouts
        .iter()
        .map(|layout| Some(*layout))
        .collect::<Vec<_>>();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{label}-layout")),
        bind_group_layouts: &bind_group_layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: vertex_buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn optical_premultiplied_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

fn reference_smoother_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::OneMinusDst,
            dst_factor: wgpu::BlendFactor::OneMinusSrc,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}
