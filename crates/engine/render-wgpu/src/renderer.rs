use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::mem::size_of;
use std::path::PathBuf;
use std::time::SystemTime;

use amigo_2d_particles::{Particle2dDrawCommand, ParticleBlendMode2d, ParticleShape2d};
use amigo_2d_sprite::{Sprite, SpriteSceneService, SpriteSheet};
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::{TileMap2d, TileMap2dSceneService};
use amigo_2d_vector::{VectorSceneService, VectorShape2d, VectorShapeKind2d, VectorStyle2d};
use amigo_3d_material::{MaterialDrawCommand, MaterialSceneService};
use amigo_3d_mesh::{MeshDrawCommand, MeshSceneService};
use amigo_3d_text::{Text3dDrawCommand, Text3dSceneService};
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_scene::SceneService;
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::WgpuSurfaceState;
use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiTextAnchor, UiViewportSize, build_ui_overlay_primitives,
};

const COLOR_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

const TEXTURE_SHADER: &str = r#"
struct VertexIn {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@vertex
fn vs_main(vertex: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.uv;
    out.color = vertex.color;
    return out;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(color_texture, color_sampler, input.uv) * input.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct ColorVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl ColorVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4
    ];

    fn new(position: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct TextureVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl TextureVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4
    ];

    fn new(position: Vec2, uv: Vec2, color: ColorRgba) -> Self {
        Self {
            position: [position.x, position.y],
            uv: [uv.x, uv.y],
            color: [color.r, color.g, color.b, color.a],
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy)]
struct Viewport {
    half_width: f32,
    half_height: f32,
    aspect: f32,
}

impl Viewport {
    fn from_surface(surface: &WgpuSurfaceState) -> Self {
        let width = surface.config.width.max(1) as f32;
        let height = surface.config.height.max(1) as f32;
        Self {
            half_width: width * 0.5,
            half_height: height * 0.5,
            aspect: width / height,
        }
    }
}

#[derive(Clone, Copy)]
struct ProjectedPoint {
    position: Vec2,
    depth: f32,
}

#[derive(Clone)]
struct ProjectedTriangle {
    points: [Vec2; 3],
    color: ColorRgba,
    depth: f32,
}

#[derive(Clone, Copy)]
struct TextureUvRect {
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

#[derive(Clone)]
struct TextureBatch {
    bind_group: wgpu::BindGroup,
    vertices: Vec<TextureVertex>,
}

#[derive(Clone)]
struct ColorBatch {
    blend_mode: ParticleBlendMode2d,
    vertices: Vec<ColorVertex>,
}

#[derive(Clone)]
enum World2dItem {
    TileMap(amigo_2d_tilemap::TileMap2dDrawCommand),
    Vector(amigo_2d_vector::VectorShape2dDrawCommand),
    Sprite(amigo_2d_sprite::SpriteDrawCommand),
    Particle(Particle2dDrawCommand),
}

struct CachedTextureResource {
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    image_path: PathBuf,
    modified_at: Option<SystemTime>,
    width: u32,
    height: u32,
}

impl CachedTextureResource {
    fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

fn create_color_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    label: &'static str,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[ColorVertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
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
    })
}

fn additive_blend_state() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

fn multiply_blend_state() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Dst,
            dst_factor: wgpu::BlendFactor::Zero,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

fn screen_blend_state() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
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

pub struct WgpuSceneRenderer {
    color_alpha_pipeline: wgpu::RenderPipeline,
    color_additive_pipeline: wgpu::RenderPipeline,
    color_multiply_pipeline: wgpu::RenderPipeline,
    color_screen_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_cache: BTreeMap<String, CachedTextureResource>,
}

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        let color_shader = surface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-color-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(COLOR_SHADER)),
            });
        let color_pipeline_layout =
            surface
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("amigo-scene-color-pipeline-layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });
        let color_alpha_pipeline = create_color_pipeline(
            &surface.device,
            &color_shader,
            &color_pipeline_layout,
            surface.config.format,
            "amigo-scene-color-alpha-pipeline",
            wgpu::BlendState::ALPHA_BLENDING,
        );
        let color_additive_pipeline = create_color_pipeline(
            &surface.device,
            &color_shader,
            &color_pipeline_layout,
            surface.config.format,
            "amigo-scene-color-additive-pipeline",
            additive_blend_state(),
        );
        let color_multiply_pipeline = create_color_pipeline(
            &surface.device,
            &color_shader,
            &color_pipeline_layout,
            surface.config.format,
            "amigo-scene-color-multiply-pipeline",
            multiply_blend_state(),
        );
        let color_screen_pipeline = create_color_pipeline(
            &surface.device,
            &color_shader,
            &color_pipeline_layout,
            surface.config.format,
            "amigo-scene-color-screen-pipeline",
            screen_blend_state(),
        );

        let texture_bind_group_layout =
            surface
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let texture_shader = surface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-texture-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEXTURE_SHADER)),
            });
        let texture_pipeline_layout =
            surface
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("amigo-scene-texture-pipeline-layout"),
                    bind_group_layouts: &[Some(&texture_bind_group_layout)],
                    immediate_size: 0,
                });
        let texture_pipeline =
            surface
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                            format: surface.config.format,
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

    pub fn render_scene(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
    ) -> AmigoResult<()> {
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &[],
        )
    }

    pub fn render_scene_with_ui_documents(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_primitives(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let mesh_commands = meshes.commands();
        let material_commands = materials.commands();
        let text3d_commands = text3d.map(|service| service.commands());
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            &mesh_commands,
            &material_commands,
            text3d_commands.as_deref(),
            &[],
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_documents_and_3d_commands(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        self.render_scene_with_ui_primitives_and_3d_commands(
            surface,
            scene,
            assets,
            tilemaps,
            sprites,
            text2d,
            vectors,
            meshes,
            materials,
            text3d,
            particles,
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_primitives_and_3d_commands(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        assets: &AssetCatalog,
        tilemaps: &TileMap2dSceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        vectors: &VectorSceneService,
        meshes: &[MeshDrawCommand],
        materials: &[MaterialDrawCommand],
        text3d: Option<&[Text3dDrawCommand]>,
        particles: &[Particle2dDrawCommand],
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_surface(surface);
        let mut color_batches = Vec::new();
        let mut texture_batches = Vec::new();
        let camera2d = resolve_camera2d_transform(scene);
        let mut world2d_items = tilemaps
            .commands()
            .into_iter()
            .map(World2dItem::TileMap)
            .chain(vectors.commands().into_iter().map(World2dItem::Vector))
            .chain(sprites.commands().into_iter().map(World2dItem::Sprite))
            .chain(particles.iter().cloned().map(World2dItem::Particle))
            .collect::<Vec<_>>();
        world2d_items.sort_by(|left, right| {
            let (left_z, left_priority) = world2d_sort_key(left);
            let (right_z, right_priority) = world2d_sort_key(right);
            left_z
                .partial_cmp(&right_z)
                .unwrap_or(Ordering::Equal)
                .then(left_priority.cmp(&right_priority))
        });

        for item in world2d_items {
            match item {
                World2dItem::TileMap(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, Transform2::default());
                    if !self.append_tilemap_texture_batch(
                        &mut texture_batches,
                        surface,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.tilemap,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_tilemap_fallback_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.tilemap,
                        );
                    }
                }
                World2dItem::Sprite(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    if !self.append_sprite_texture_batch(
                        &mut texture_batches,
                        surface,
                        assets,
                        &viewport,
                        camera2d,
                        transform,
                        &command.sprite,
                    ) {
                        let vertices =
                            color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                        append_sprite_vertices(
                            vertices,
                            &viewport,
                            camera2d,
                            transform,
                            &command.sprite,
                            sprite_color(command.sprite.texture.as_str()),
                        );
                    }
                }
                World2dItem::Vector(command) => {
                    let transform =
                        resolve_transform2(scene, &command.entity_name, command.transform);
                    let vertices =
                        color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                    append_vector_shape_vertices(
                        vertices,
                        &viewport,
                        camera2d,
                        transform,
                        &command.shape,
                    );
                }
                World2dItem::Particle(command) => {
                    let vertices = color_batch_vertices(&mut color_batches, command.blend_mode);
                    append_particle_vertices(vertices, &viewport, camera2d, &command);
                }
            }
        }

        for command in text2d.commands() {
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_text_2d_vertices(
                vertices,
                &viewport,
                camera2d,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            );
        }

        let camera = resolve_camera_transform(scene);
        let material_lookup = material_lookup_from_commands(materials);
        let mut projected_triangles = Vec::new();

        for command in meshes {
            let transform = resolve_transform3(scene, &command.entity_name, command.mesh.transform);
            let color = material_lookup
                .get(&command.entity_name)
                .copied()
                .unwrap_or_else(|| mesh_color(command.mesh.mesh_asset.as_str()));
            append_mesh_triangles(
                &mut projected_triangles,
                &viewport,
                camera,
                transform,
                color,
            );
        }

        projected_triangles.sort_by(|left, right| {
            right
                .depth
                .partial_cmp(&left.depth)
                .unwrap_or(Ordering::Equal)
        });

        for triangle in projected_triangles {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            push_triangle(vertices, triangle.points, triangle.color);
        }

        if let Some(text3d) = text3d {
            for command in text3d {
                let transform =
                    resolve_transform3(scene, &command.entity_name, command.text.transform);
                let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
                append_text_3d_vertices(
                    vertices,
                    &viewport,
                    camera,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                );
            }
        }

        {
            let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
            append_ui_overlay_vertices(vertices, &viewport, ui_primitives);
        }

        let frame = match surface.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Occluded => {
                surface.surface.configure(&surface.device, &surface.config);
                return Ok(());
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = surface
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-scene-render-encoder"),
            });

        color_batches.retain(|batch| !batch.vertices.is_empty());
        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-color-vertices"),
                        contents: vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();
        let texture_vertex_buffers = texture_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-texture-vertices"),
                        contents: texture_vertices_as_bytes(&batch.vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            })
            .collect::<Vec<_>>();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-scene-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.09,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate() {
                pass.set_pipeline(&self.texture_pipeline);
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        surface.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    fn append_sprite_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        surface: &WgpuSurfaceState,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        sprite: &Sprite,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&sprite.texture) else {
            return false;
        };
        let Some(texture) = self.ensure_texture(surface, &prepared) else {
            return false;
        };
        let sheet = sprite
            .sheet
            .or_else(|| infer_sprite_sheet_from_asset(&prepared));
        let uv = sprite_uv_rect(texture.dimensions(), sheet, sprite.frame_index);
        let mut vertices = Vec::with_capacity(6);
        append_textured_sprite_vertices(
            &mut vertices,
            viewport,
            camera,
            transform,
            sprite.size,
            uv,
        );
        batches.push(TextureBatch {
            bind_group: texture.bind_group.clone(),
            vertices,
        });
        true
    }

    fn color_pipeline_for(&self, blend_mode: ParticleBlendMode2d) -> &wgpu::RenderPipeline {
        match blend_mode {
            ParticleBlendMode2d::Alpha => &self.color_alpha_pipeline,
            ParticleBlendMode2d::Additive => &self.color_additive_pipeline,
            ParticleBlendMode2d::Multiply => &self.color_multiply_pipeline,
            ParticleBlendMode2d::Screen => &self.color_screen_pipeline,
        }
    }

    fn append_tilemap_texture_batch(
        &mut self,
        batches: &mut Vec<TextureBatch>,
        surface: &WgpuSurfaceState,
        assets: &AssetCatalog,
        viewport: &Viewport,
        camera: Transform2,
        transform: Transform2,
        tilemap: &TileMap2d,
    ) -> bool {
        let Some(prepared) = assets.prepared_asset(&tilemap.tileset) else {
            return false;
        };
        let Some(texture) = self.ensure_texture(surface, &prepared) else {
            return false;
        };
        let Some(tileset) = infer_tileset_from_asset(&prepared, tilemap.tile_size) else {
            return false;
        };
        let mut vertices = Vec::new();
        append_textured_tilemap_vertices(
            &mut vertices,
            viewport,
            camera,
            transform,
            tilemap,
            texture.dimensions(),
            &tileset,
        );
        if vertices.is_empty() {
            return false;
        }
        batches.push(TextureBatch {
            bind_group: texture.bind_group.clone(),
            vertices,
        });
        true
    }

    fn ensure_texture(
        &mut self,
        surface: &WgpuSurfaceState,
        prepared: &PreparedAsset,
    ) -> Option<&CachedTextureResource> {
        let image_path = resolve_image_path(prepared)?;
        let modified_at = fs::metadata(&image_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        let key = prepared.key.as_str().to_owned();
        let should_reload = self
            .texture_cache
            .get(&key)
            .map(|cached| cached.image_path != image_path || cached.modified_at != modified_at)
            .unwrap_or(true);

        if should_reload {
            let image = image::open(&image_path).ok()?;
            let rgba = image.to_rgba8();
            let (width, height) = image.dimensions();
            if width == 0 || height == 0 {
                return None;
            }

            let texture = surface.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("amigo-scene-texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            surface.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let linear_sampling = metadata_bool(prepared, "sampling.linear")
                || prepared
                    .metadata
                    .get("sampling")
                    .map(|value| value.eq_ignore_ascii_case("linear"))
                    .unwrap_or(false);
            let (mag_filter, min_filter, mipmap_filter) = if linear_sampling {
                (
                    wgpu::FilterMode::Linear,
                    wgpu::FilterMode::Linear,
                    wgpu::MipmapFilterMode::Linear,
                )
            } else {
                (
                    wgpu::FilterMode::Nearest,
                    wgpu::FilterMode::Nearest,
                    wgpu::MipmapFilterMode::Nearest,
                )
            };
            let sampler = surface.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("amigo-scene-texture-sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter,
                min_filter,
                mipmap_filter,
                ..wgpu::SamplerDescriptor::default()
            });
            let bind_group = surface
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("amigo-scene-texture-bind-group"),
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });

            self.texture_cache.insert(
                key.clone(),
                CachedTextureResource {
                    _texture: texture,
                    _view: view,
                    _sampler: sampler,
                    bind_group,
                    image_path,
                    modified_at,
                    width,
                    height,
                },
            );
        }

        self.texture_cache.get(&key)
    }
}

fn resolve_transform2(scene: &SceneService, entity_name: &str, fallback: Transform2) -> Transform2 {
    scene
        .transform_of(entity_name)
        .map(transform2_from_transform3)
        .unwrap_or(fallback)
}

fn resolve_transform3(scene: &SceneService, entity_name: &str, fallback: Transform3) -> Transform3 {
    scene.transform_of(entity_name).unwrap_or(fallback)
}

fn resolve_camera_transform(scene: &SceneService) -> Transform3 {
    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("3d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() > 0.01)
        })
        .map(|entity| entity.transform)
        .unwrap_or(Transform3 {
            translation: Vec3::new(0.0, 0.0, 6.0),
            ..Transform3::default()
        })
}

fn resolve_camera2d_transform(scene: &SceneService) -> Transform2 {
    scene
        .entities()
        .into_iter()
        .find(|entity| {
            entity.name.contains("2d-camera")
                || (entity.name.contains("camera") && entity.transform.translation.z.abs() <= 0.01)
        })
        .map(|entity| transform2_from_transform3(entity.transform))
        .unwrap_or_default()
}

fn material_lookup_from_commands(materials: &[MaterialDrawCommand]) -> BTreeMap<String, ColorRgba> {
    materials
        .iter()
        .cloned()
        .map(|command| (command.entity_name, command.material.albedo))
        .collect()
}

#[derive(Clone)]
struct TileSetRenderInfo {
    tile_size: Vec2,
    columns: u32,
    ground_tile_id: u32,
    platform_tile_id: Option<u32>,
    derived_tiles: BTreeMap<u32, DerivedTileRenderInfo>,
}

#[derive(Clone, Copy)]
struct DerivedTileRenderInfo {
    source_tile_id: u32,
    crop: TileCropRect,
}

#[derive(Clone, Copy)]
struct TileCropRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

fn world2d_sort_key(item: &World2dItem) -> (f32, u8) {
    match item {
        World2dItem::TileMap(command) => (command.z_index, 0),
        World2dItem::Vector(command) => (command.z_index, 1),
        World2dItem::Particle(command) => (command.z_index, 2),
        World2dItem::Sprite(command) => (command.z_index, 3),
    }
}

fn color_batch_vertices(
    batches: &mut Vec<ColorBatch>,
    blend_mode: ParticleBlendMode2d,
) -> &mut Vec<ColorVertex> {
    let needs_new_batch = batches
        .last()
        .map(|batch| batch.blend_mode != blend_mode)
        .unwrap_or(true);
    if needs_new_batch {
        batches.push(ColorBatch {
            blend_mode,
            vertices: Vec::new(),
        });
    }
    &mut batches
        .last_mut()
        .expect("color batch should exist after push")
        .vertices
}

fn append_particle_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    particle: &Particle2dDrawCommand,
) {
    let size = particle.size.max(0.0);
    if size <= f32::EPSILON || particle.color.a <= 0.0 {
        return;
    }
    let shape = match particle.shape {
        ParticleShape2d::Circle { segments } => VectorShape2d {
            kind: VectorShapeKind2d::Circle {
                radius: size * 0.5,
                segments: segments.max(3),
            },
            style: VectorStyle2d {
                stroke_color: particle.color,
                stroke_width: 0.0,
                fill_color: Some(particle.color),
            },
        },
        ParticleShape2d::Quad => {
            let half = size * 0.5;
            VectorShape2d {
                kind: VectorShapeKind2d::Polygon {
                    points: vec![
                        Vec2::new(-half, -half),
                        Vec2::new(half, -half),
                        Vec2::new(half, half),
                        Vec2::new(-half, half),
                    ],
                },
                style: VectorStyle2d {
                    stroke_color: particle.color,
                    stroke_width: 0.0,
                    fill_color: Some(particle.color),
                },
            }
        }
        ParticleShape2d::Line { length } => VectorShape2d {
            kind: VectorShapeKind2d::Polyline {
                points: vec![Vec2::new(-length * 0.5, 0.0), Vec2::new(length * 0.5, 0.0)],
                closed: false,
            },
            style: VectorStyle2d {
                stroke_color: particle.color,
                stroke_width: size.max(1.0),
                fill_color: None,
            },
        },
    };
    append_vector_shape_vertices(
        vertices,
        viewport,
        camera,
        Transform2 {
            translation: particle.position,
            rotation_radians: particle.transform.rotation_radians,
            scale: particle.transform.scale,
        },
        &shape,
    );
}

fn resolve_image_path(prepared: &PreparedAsset) -> Option<PathBuf> {
    let extension = prepared
        .resolved_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());
    match extension.as_deref() {
        Some("png" | "jpg" | "jpeg" | "webp") => Some(prepared.resolved_path.clone()),
        _ => prepared.metadata.get("image").and_then(|image| {
            prepared
                .resolved_path
                .parent()
                .map(|parent| parent.join(image))
        }),
    }
}

fn infer_sprite_sheet_from_asset(prepared: &PreparedAsset) -> Option<SpriteSheet> {
    if !matches!(prepared.kind, PreparedAssetKind::SpriteSheet2d) {
        return None;
    }

    let columns = metadata_u32(prepared, "columns")?.max(1);
    let rows = metadata_u32(prepared, "rows")?.max(1);
    let frame_width = metadata_f32(prepared, "frame_size.x")?;
    let frame_height = metadata_f32(prepared, "frame_size.y")?;
    Some(SpriteSheet {
        columns,
        rows,
        frame_count: metadata_u32(prepared, "frame_count")
            .unwrap_or(columns.saturating_mul(rows))
            .max(1),
        frame_size: Vec2::new(frame_width, frame_height),
        fps: metadata_f32(prepared, "fps").unwrap_or(0.0),
        looping: prepared
            .metadata
            .get("looping")
            .and_then(|value| value.parse::<bool>().ok())
            .unwrap_or(true),
    })
}

fn infer_tileset_from_asset(
    prepared: &PreparedAsset,
    fallback_tile_size: Vec2,
) -> Option<TileSetRenderInfo> {
    if !matches!(prepared.kind, PreparedAssetKind::TileSet2d) {
        return None;
    }

    let columns = metadata_u32(prepared, "columns")?.max(1);
    let _rows = metadata_u32(prepared, "rows")?.max(1);
    let tile_size = Vec2::new(
        metadata_f32(prepared, "tile_size.x").unwrap_or(fallback_tile_size.x),
        metadata_f32(prepared, "tile_size.y").unwrap_or(fallback_tile_size.y),
    );
    let tile_ids = infer_tileset_tile_ids(prepared);
    Some(TileSetRenderInfo {
        tile_size,
        columns,
        ground_tile_id: metadata_u32(prepared, "tiles.ground.id")
            .or_else(|| metadata_u32(prepared, "tiles.ground_single.id"))
            .unwrap_or(1),
        platform_tile_id: metadata_u32(prepared, "tiles.platform.id")
            .or_else(|| metadata_u32(prepared, "tiles.ground_middle.id")),
        derived_tiles: infer_derived_tile_render_info(prepared, &tile_ids),
    })
}

fn metadata_u32(prepared: &PreparedAsset, key: &str) -> Option<u32> {
    prepared.metadata.get(key)?.parse().ok()
}

fn metadata_f32(prepared: &PreparedAsset, key: &str) -> Option<f32> {
    prepared.metadata.get(key)?.parse().ok()
}

fn metadata_bool(prepared: &PreparedAsset, key: &str) -> bool {
    prepared
        .metadata
        .get(key)
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false)
}

fn sprite_uv_rect(
    texture_size: Vec2,
    sheet: Option<SpriteSheet>,
    frame_index: u32,
) -> TextureUvRect {
    let Some(sheet) = sheet else {
        return TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        };
    };

    let columns = sheet.columns.max(1);
    let rows = sheet.rows.max(1);
    let frame = frame_index.min(sheet.visible_frame_count().saturating_sub(1));
    let column = frame % columns;
    let row = frame / columns;
    let frame_width = if sheet.frame_size.x > 0.0 {
        sheet.frame_size.x
    } else {
        texture_size.x / columns as f32
    };
    let frame_height = if sheet.frame_size.y > 0.0 {
        sheet.frame_size.y
    } else {
        texture_size.y / rows as f32
    };
    let u0 = (column as f32 * frame_width) / texture_size.x.max(1.0);
    let v0 = (row as f32 * frame_height) / texture_size.y.max(1.0);
    let u1 = ((column as f32 + 1.0) * frame_width) / texture_size.x.max(1.0);
    let v1 = ((row as f32 + 1.0) * frame_height) / texture_size.y.max(1.0);
    TextureUvRect { u0, v0, u1, v1 }
}

fn infer_tileset_tile_ids(prepared: &PreparedAsset) -> BTreeMap<String, u32> {
    prepared
        .metadata
        .iter()
        .filter_map(|(key, value)| {
            let tile_name = key.strip_prefix("tiles.")?.strip_suffix(".id")?;
            value
                .parse::<u32>()
                .ok()
                .map(|id| (tile_name.to_owned(), id))
        })
        .collect()
}

fn infer_derived_tile_render_info(
    prepared: &PreparedAsset,
    tile_ids: &BTreeMap<String, u32>,
) -> BTreeMap<u32, DerivedTileRenderInfo> {
    let mut variant_names = Vec::new();
    for key in prepared.metadata.keys() {
        if let Some(rest) = key.strip_prefix("derived_variants.") {
            if let Some((variant_name, _)) = rest.split_once('.') {
                if !variant_names
                    .iter()
                    .any(|name: &String| name == variant_name)
                {
                    variant_names.push(variant_name.to_owned());
                }
            }
        }
    }

    let mut derived_tiles = BTreeMap::new();
    for variant_name in variant_names {
        let target_id = tile_ids
            .get(&variant_name)
            .copied()
            .or_else(|| metadata_u32(prepared, &format!("derived_variants.{variant_name}.id")));
        let source_tile_id = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.from_tile"))
            .and_then(|source_name| tile_ids.get(source_name))
            .copied()
            .or_else(|| {
                metadata_u32(
                    prepared,
                    &format!("derived_variants.{variant_name}.from_id"),
                )
            });
        let mode = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.mode"))
            .map(String::as_str);
        let segment = prepared
            .metadata
            .get(&format!("derived_variants.{variant_name}.segment"))
            .or_else(|| {
                prepared
                    .metadata
                    .get(&format!("derived_variants.{variant_name}.side"))
            })
            .map(String::as_str);

        let crop = match (mode, segment) {
            (Some("split_x"), Some("left")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.0,
                x1: 0.5,
                y1: 1.0,
            }),
            (Some("split_x"), Some("right")) => Some(TileCropRect {
                x0: 0.5,
                y0: 0.0,
                x1: 1.0,
                y1: 1.0,
            }),
            (Some("split_y"), Some("top")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.0,
                x1: 1.0,
                y1: 0.5,
            }),
            (Some("split_y"), Some("bottom")) => Some(TileCropRect {
                x0: 0.0,
                y0: 0.5,
                x1: 1.0,
                y1: 1.0,
            }),
            _ => None,
        };

        if let (Some(target_id), Some(source_tile_id), Some(crop)) =
            (target_id, source_tile_id, crop)
        {
            derived_tiles.insert(
                target_id,
                DerivedTileRenderInfo {
                    source_tile_id,
                    crop,
                },
            );
        }
    }

    derived_tiles
}

fn atlas_tile_uv_rect(
    texture_size: Vec2,
    tileset: &TileSetRenderInfo,
    tile_id: u32,
) -> TextureUvRect {
    let column = tile_id % tileset.columns;
    let row = tile_id / tileset.columns;
    let tile_width = tileset.tile_size.x.max(1.0);
    let tile_height = tileset.tile_size.y.max(1.0);
    let u0 = (column as f32 * tile_width) / texture_size.x.max(1.0);
    let v0 = (row as f32 * tile_height) / texture_size.y.max(1.0);
    let u1 = ((column as f32 + 1.0) * tile_width) / texture_size.x.max(1.0);
    let v1 = ((row as f32 + 1.0) * tile_height) / texture_size.y.max(1.0);
    TextureUvRect { u0, v0, u1, v1 }
}

fn inset_uv_rect(texture_size: Vec2, uv: TextureUvRect, inset_pixels: f32) -> TextureUvRect {
    let width = (uv.u1 - uv.u0).max(0.0);
    let height = (uv.v1 - uv.v0).max(0.0);
    let inset_u = (inset_pixels / texture_size.x.max(1.0)).min(width * 0.25);
    let inset_v = (inset_pixels / texture_size.y.max(1.0)).min(height * 0.25);
    TextureUvRect {
        u0: uv.u0 + inset_u,
        v0: uv.v0 + inset_v,
        u1: uv.u1 - inset_u,
        v1: uv.v1 - inset_v,
    }
}

fn tile_uv_rect(texture_size: Vec2, tileset: &TileSetRenderInfo, tile_id: u32) -> TextureUvRect {
    let uv = if let Some(derived) = tileset.derived_tiles.get(&tile_id).copied() {
        let base = atlas_tile_uv_rect(texture_size, tileset, derived.source_tile_id);
        let du = base.u1 - base.u0;
        let dv = base.v1 - base.v0;
        TextureUvRect {
            u0: base.u0 + du * derived.crop.x0,
            v0: base.v0 + dv * derived.crop.y0,
            u1: base.u0 + du * derived.crop.x1,
            v1: base.v0 + dv * derived.crop.y1,
        }
    } else {
        atlas_tile_uv_rect(texture_size, tileset, tile_id)
    };

    inset_uv_rect(texture_size, uv, 0.5)
}

fn tile_id_for_symbol(symbol: char, tileset: &TileSetRenderInfo) -> Option<u32> {
    match symbol {
        '#' => Some(tileset.ground_tile_id),
        '=' => tileset.platform_tile_id.or(Some(tileset.ground_tile_id)),
        _ => None,
    }
}

fn append_ui_overlay_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    primitives: &[UiDrawPrimitive],
) {
    for primitive in primitives {
        match primitive {
            UiDrawPrimitive::Quad { rect, color } => {
                append_ui_quad_vertices(vertices, viewport, *rect, *color);
            }
            UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: _font,
                anchor,
                word_wrap,
                fit_to_width,
            } => append_text_screen_space_vertices(
                vertices,
                viewport,
                content,
                *rect,
                *font_size,
                *color,
                *anchor,
                *word_wrap,
                *fit_to_width,
            ),
            UiDrawPrimitive::ProgressBar {
                rect,
                value,
                background,
                foreground,
            } => append_progress_bar_vertices(
                vertices,
                viewport,
                *rect,
                *value,
                *background,
                *foreground,
            ),
        }
    }
}

fn append_ui_quad_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    rect: crate::ui_overlay::UiRect,
    color: ColorRgba,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let points = [
        ndc_from_ui_screen(Vec2::new(rect.x, rect.y), viewport),
        ndc_from_ui_screen(Vec2::new(rect.x + rect.width, rect.y), viewport),
        ndc_from_ui_screen(
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
            viewport,
        ),
        ndc_from_ui_screen(Vec2::new(rect.x, rect.y + rect.height), viewport),
    ];
    push_quad(vertices, points[0], points[1], points[2], points[3], color);
}

fn append_progress_bar_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    rect: crate::ui_overlay::UiRect,
    value: f32,
    background: ColorRgba,
    foreground: ColorRgba,
) {
    append_ui_quad_vertices(vertices, viewport, rect, background);

    let inset = rect.height.min(4.0).max(2.0);
    let inner = rect.inset(inset);
    let clamped = value.clamp(0.0, 1.0);
    let fill_rect =
        crate::ui_overlay::UiRect::new(inner.x, inner.y, inner.width * clamped, inner.height);
    append_ui_quad_vertices(vertices, viewport, fill_rect, foreground);
}

fn append_textured_sprite_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    size: Vec2,
    uv: TextureUvRect,
) {
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let points = [
        transform_point_2d(Vec2::new(-half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, half.y), transform),
        transform_point_2d(Vec2::new(-half.x, half.y), transform),
    ];
    push_textured_quad(
        vertices,
        ndc_from_world_2d(points[0], camera, viewport),
        ndc_from_world_2d(points[1], camera, viewport),
        ndc_from_world_2d(points[2], camera, viewport),
        ndc_from_world_2d(points[3], camera, viewport),
        uv,
        ColorRgba::WHITE,
    );
}

fn append_textured_tilemap_vertices(
    vertices: &mut Vec<TextureVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    tilemap: &TileMap2d,
    texture_size: Vec2,
    tileset: &TileSetRenderInfo,
) {
    let row_count = tilemap.grid.len();
    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        for (column_index, symbol) in row.chars().enumerate() {
            let tile_id = tilemap
                .resolved
                .as_ref()
                .and_then(|resolved| {
                    resolved
                        .rows
                        .get(row_index)
                        .and_then(|row| row.get(column_index))
                        .and_then(|tile| tile.tile_id)
                })
                .or_else(|| tile_id_for_symbol(symbol, tileset));
            let Some(tile_id) = tile_id else {
                continue;
            };
            let uv = tile_uv_rect(texture_size, tileset, tile_id);
            let min = Vec2::new(
                column_index as f32 * tilemap.tile_size.x,
                row_from_bottom as f32 * tilemap.tile_size.y,
            );
            let max = Vec2::new(min.x + tilemap.tile_size.x, min.y + tilemap.tile_size.y);
            let points = [
                transform_point_2d(min, transform),
                transform_point_2d(Vec2::new(max.x, min.y), transform),
                transform_point_2d(max, transform),
                transform_point_2d(Vec2::new(min.x, max.y), transform),
            ];
            push_textured_quad(
                vertices,
                ndc_from_world_2d_snapped(points[0], camera, viewport),
                ndc_from_world_2d_snapped(points[1], camera, viewport),
                ndc_from_world_2d_snapped(points[2], camera, viewport),
                ndc_from_world_2d_snapped(points[3], camera, viewport),
                uv,
                ColorRgba::WHITE,
            );
        }
    }
}

fn append_tilemap_fallback_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    tilemap: &TileMap2d,
) {
    let row_count = tilemap.grid.len();
    for (row_index, row) in tilemap.grid.iter().enumerate() {
        let row_from_bottom = row_count.saturating_sub(row_index + 1);
        for (column_index, symbol) in row.chars().enumerate() {
            if symbol != '#' && symbol != '=' {
                continue;
            }
            let min = Vec2::new(
                column_index as f32 * tilemap.tile_size.x,
                row_from_bottom as f32 * tilemap.tile_size.y,
            );
            let max = Vec2::new(min.x + tilemap.tile_size.x, min.y + tilemap.tile_size.y);
            let points = [
                transform_point_2d(min, transform),
                transform_point_2d(Vec2::new(max.x, min.y), transform),
                transform_point_2d(max, transform),
                transform_point_2d(Vec2::new(min.x, max.y), transform),
            ];
            push_quad(
                vertices,
                ndc_from_world_2d_snapped(points[0], camera, viewport),
                ndc_from_world_2d_snapped(points[1], camera, viewport),
                ndc_from_world_2d_snapped(points[2], camera, viewport),
                ndc_from_world_2d_snapped(points[3], camera, viewport),
                ColorRgba::new(0.28, 0.31, 0.38, 1.0),
            );
        }
    }
}

fn append_sprite_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    sprite: &Sprite,
    color: ColorRgba,
) {
    let asset_key = sprite.texture.as_str();
    let size = sprite.size;
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let points = [
        transform_point_2d(Vec2::new(-half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, half.y), transform),
        transform_point_2d(Vec2::new(-half.x, half.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(points[0], camera, viewport),
        ndc_from_world_2d(points[1], camera, viewport),
        ndc_from_world_2d(points[2], camera, viewport),
        ndc_from_world_2d(points[3], camera, viewport),
        if sprite.sheet.is_some() {
            modulate_color(color, 0.18)
        } else {
            color
        },
    );

    if let Some(sheet) = sprite.sheet {
        append_sprite_sheet_overlay(
            vertices,
            viewport,
            camera,
            transform,
            size,
            sheet,
            sprite.frame_index,
            color,
        );
    } else if asset_key.contains("square") || asset_key.contains("sprite") {
        let marker_half = Vec2::new(size.x * 0.12, size.y * 0.12);
        let marker_center = Vec2::new(size.x * 0.18, size.y * 0.18);
        let marker_points = [
            transform_point_2d(
                Vec2::new(
                    marker_center.x - marker_half.x,
                    marker_center.y - marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x + marker_half.x,
                    marker_center.y - marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x + marker_half.x,
                    marker_center.y + marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x - marker_half.x,
                    marker_center.y + marker_half.y,
                ),
                transform,
            ),
        ];

        push_quad(
            vertices,
            ndc_from_world_2d(marker_points[0], camera, viewport),
            ndc_from_world_2d(marker_points[1], camera, viewport),
            ndc_from_world_2d(marker_points[2], camera, viewport),
            ndc_from_world_2d(marker_points[3], camera, viewport),
            ColorRgba::new(0.98, 0.98, 0.98, 1.0),
        );
    }
}

fn append_sprite_sheet_overlay(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    size: Vec2,
    sheet: SpriteSheet,
    frame_index: u32,
    base_color: ColorRgba,
) {
    let columns = sheet.columns.max(1);
    let rows = sheet.rows.max(1);
    let visible_frames = sheet.visible_frame_count();
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let sheet_width = size.x * 0.64;
    let preview_width = size.x * 0.24;
    let gap_width = (size.x - sheet_width - preview_width).max(12.0);
    let sheet_left = -half.x;
    let sheet_right = sheet_left + sheet_width;
    let preview_left = sheet_right + gap_width;
    let preview_right = half.x;
    let cell_size = Vec2::new(sheet_width / columns as f32, size.y / rows as f32);
    let pad = Vec2::new((cell_size.x * 0.08).max(3.0), (cell_size.y * 0.08).max(3.0));

    for frame in 0..visible_frames {
        let column = frame % columns;
        let row = frame / columns;
        let left = sheet_left + column as f32 * cell_size.x;
        let right = left + cell_size.x;
        let top = half.y - row as f32 * cell_size.y;
        let bottom = top - cell_size.y;
        let min = Vec2::new(left + pad.x, bottom + pad.y);
        let max = Vec2::new(right - pad.x, top - pad.y);
        let frame_color = if frame == frame_index.min(visible_frames.saturating_sub(1)) {
            ColorRgba::new(0.99, 0.97, 0.88, 1.0)
        } else {
            modulate_color(
                blend_colors(base_color, spritesheet_frame_color(frame)),
                0.42,
            )
        };
        let quad = [
            transform_point_2d(min, transform),
            transform_point_2d(Vec2::new(max.x, min.y), transform),
            transform_point_2d(max, transform),
            transform_point_2d(Vec2::new(min.x, max.y), transform),
        ];
        push_quad(
            vertices,
            ndc_from_world_2d(quad[0], camera, viewport),
            ndc_from_world_2d(quad[1], camera, viewport),
            ndc_from_world_2d(quad[2], camera, viewport),
            ndc_from_world_2d(quad[3], camera, viewport),
            frame_color,
        );
    }

    let preview_frame = frame_index.min(visible_frames.saturating_sub(1));
    let preview_color = blend_colors(base_color, spritesheet_frame_color(preview_frame));
    let preview_pad = Vec2::new(
        ((preview_right - preview_left) * 0.16).max(4.0),
        (size.y * 0.16).max(4.0),
    );
    let preview_min = Vec2::new(preview_left + preview_pad.x, -half.y + preview_pad.y);
    let preview_max = Vec2::new(preview_right - preview_pad.x, half.y - preview_pad.y);
    let preview_quad = [
        transform_point_2d(preview_min, transform),
        transform_point_2d(Vec2::new(preview_max.x, preview_min.y), transform),
        transform_point_2d(preview_max, transform),
        transform_point_2d(Vec2::new(preview_min.x, preview_max.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(preview_quad[0], camera, viewport),
        ndc_from_world_2d(preview_quad[1], camera, viewport),
        ndc_from_world_2d(preview_quad[2], camera, viewport),
        ndc_from_world_2d(preview_quad[3], camera, viewport),
        preview_color,
    );

    let marker_size = Vec2::new(
        (preview_max.x - preview_min.x) * 0.28,
        (preview_max.y - preview_min.y) * 0.28,
    );
    let marker_center = Vec2::new(
        (preview_min.x + preview_max.x) * 0.5,
        (preview_min.y + preview_max.y) * 0.5,
    );
    let marker_min = Vec2::new(
        marker_center.x - marker_size.x * 0.5,
        marker_center.y - marker_size.y * 0.5,
    );
    let marker_max = Vec2::new(
        marker_center.x + marker_size.x * 0.5,
        marker_center.y + marker_size.y * 0.5,
    );
    let marker_quad = [
        transform_point_2d(marker_min, transform),
        transform_point_2d(Vec2::new(marker_max.x, marker_min.y), transform),
        transform_point_2d(marker_max, transform),
        transform_point_2d(Vec2::new(marker_min.x, marker_max.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(marker_quad[0], camera, viewport),
        ndc_from_world_2d(marker_quad[1], camera, viewport),
        ndc_from_world_2d(marker_quad[2], camera, viewport),
        ndc_from_world_2d(marker_quad[3], camera, viewport),
        ColorRgba::new(0.98, 0.98, 0.98, 1.0),
    );
}

fn append_text_2d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    content: &str,
    transform: Transform2,
    bounds: Vec2,
    color: ColorRgba,
) {
    let pixel_size = (bounds.y / 7.0).clamp(4.0, 18.0);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec2::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                );
                let max = Vec2::new(min.x + pixel_size, min.y + pixel_size);
                let quad = [
                    transform_point_2d(min, transform),
                    transform_point_2d(Vec2::new(max.x, min.y), transform),
                    transform_point_2d(max, transform),
                    transform_point_2d(Vec2::new(min.x, max.y), transform),
                ];
                push_quad(
                    vertices,
                    ndc_from_world_2d(quad[0], camera, viewport),
                    ndc_from_world_2d(quad[1], camera, viewport),
                    ndc_from_world_2d(quad[2], camera, viewport),
                    ndc_from_world_2d(quad[3], camera, viewport),
                    color,
                );
            }
        }
    }
}

fn append_vector_shape_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    shape: &VectorShape2d,
) {
    let local_points = vector_shape_points(shape);
    if local_points.is_empty() {
        return;
    }

    let world_points = local_points
        .into_iter()
        .map(|point| transform_point_2d(point, transform))
        .collect::<Vec<_>>();
    let (closed, can_fill) = match &shape.kind {
        VectorShapeKind2d::Polyline { closed, .. } => (*closed, *closed),
        VectorShapeKind2d::Polygon { .. } | VectorShapeKind2d::Circle { .. } => (true, true),
    };

    if can_fill {
        if let Some(fill_color) = shape.style.fill_color {
            append_filled_polygon_vertices(vertices, viewport, camera, &world_points, fill_color);
        }
    }

    if shape.style.stroke_width > 0.0 {
        append_polyline_stroke_vertices(
            vertices,
            viewport,
            camera,
            &world_points,
            closed,
            shape.style.stroke_width,
            shape.style.stroke_color,
        );
    }
}

fn vector_shape_points(shape: &VectorShape2d) -> Vec<Vec2> {
    match &shape.kind {
        VectorShapeKind2d::Polyline { points, .. } | VectorShapeKind2d::Polygon { points } => {
            points.clone()
        }
        VectorShapeKind2d::Circle { radius, segments } => {
            let segment_count = (*segments).max(3) as usize;
            let mut points = Vec::with_capacity(segment_count);
            for index in 0..segment_count {
                let angle = (index as f32 / segment_count as f32) * std::f32::consts::TAU;
                points.push(Vec2::new(angle.cos() * *radius, angle.sin() * *radius));
            }
            points
        }
    }
}

fn append_filled_polygon_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    points: &[Vec2],
    color: ColorRgba,
) {
    if points.len() < 3 {
        return;
    }

    let origin = ndc_from_world_2d(points[0], camera, viewport);
    for index in 1..points.len() - 1 {
        push_triangle(
            vertices,
            [
                origin,
                ndc_from_world_2d(points[index], camera, viewport),
                ndc_from_world_2d(points[index + 1], camera, viewport),
            ],
            color,
        );
    }
}

fn append_polyline_stroke_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    points: &[Vec2],
    closed: bool,
    stroke_width: f32,
    color: ColorRgba,
) {
    if points.len() < 2 {
        return;
    }

    for index in 0..points.len() - 1 {
        append_line_segment_vertices(
            vertices,
            viewport,
            camera,
            points[index],
            points[index + 1],
            stroke_width,
            color,
        );
    }

    if closed {
        append_line_segment_vertices(
            vertices,
            viewport,
            camera,
            *points
                .last()
                .expect("closed vector shape should have a last point"),
            points[0],
            stroke_width,
            color,
        );
    }
}

fn append_line_segment_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    start: Vec2,
    end: Vec2,
    stroke_width: f32,
    color: ColorRgba,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let half_width = stroke_width * 0.5;
    let normal = Vec2::new(-dy / length * half_width, dx / length * half_width);
    let a = Vec2::new(start.x + normal.x, start.y + normal.y);
    let b = Vec2::new(end.x + normal.x, end.y + normal.y);
    let c = Vec2::new(end.x - normal.x, end.y - normal.y);
    let d = Vec2::new(start.x - normal.x, start.y - normal.y);
    push_quad(
        vertices,
        ndc_from_world_2d(a, camera, viewport),
        ndc_from_world_2d(b, camera, viewport),
        ndc_from_world_2d(c, camera, viewport),
        ndc_from_world_2d(d, camera, viewport),
        color,
    );
}

fn append_text_screen_space_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    content: &str,
    rect: crate::ui_overlay::UiRect,
    font_size: f32,
    color: ColorRgba,
    anchor: UiTextAnchor,
    word_wrap: bool,
    fit_to_width: bool,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let (effective_font_size, lines) =
        layout_ui_text_lines(content, rect.width, font_size, word_wrap, fit_to_width);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    let line_height = effective_font_size * 1.2;
    let line_widths = lines
        .iter()
        .map(|line| line.chars().count() as f32 * advance)
        .collect::<Vec<_>>();
    let text_width = line_widths.iter().copied().fold(0.0, f32::max);
    let text_height = line_height * lines.len().max(1) as f32;

    let origin_x = match anchor {
        UiTextAnchor::TopLeft => rect.x,
        UiTextAnchor::Center => rect.x + (rect.width - text_width).max(0.0) * 0.5,
    };
    let origin_y = match anchor {
        UiTextAnchor::TopLeft => rect.y,
        UiTextAnchor::Center => rect.y + (rect.height - text_height).max(0.0) * 0.5,
    };

    for (line_index, line) in lines.iter().enumerate() {
        let line_origin_y = origin_y + line_index as f32 * line_height;
        let line_origin_x = match anchor {
            UiTextAnchor::TopLeft => origin_x,
            UiTextAnchor::Center => rect.x + (rect.width - line_widths[line_index]).max(0.0) * 0.5,
        };
        for (index, ch) in line.chars().enumerate() {
            let rows = glyph_rows(ch);
            let glyph_origin_x = line_origin_x + index as f32 * advance;
            for (row_index, row_bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if row_bits & (1 << (4 - column)) == 0 {
                        continue;
                    }

                    let min = Vec2::new(
                        glyph_origin_x + column as f32 * pixel_size,
                        line_origin_y + row_index as f32 * pixel_size,
                    );
                    let max = Vec2::new(min.x + pixel_size, min.y + pixel_size);
                    let quad = [
                        ndc_from_ui_screen(min, viewport),
                        ndc_from_ui_screen(Vec2::new(max.x, min.y), viewport),
                        ndc_from_ui_screen(max, viewport),
                        ndc_from_ui_screen(Vec2::new(min.x, max.y), viewport),
                    ];
                    push_quad(vertices, quad[0], quad[1], quad[2], quad[3], color);
                }
            }
        }
    }
}

fn layout_ui_text_lines(
    content: &str,
    max_width: f32,
    font_size: f32,
    word_wrap: bool,
    fit_to_width: bool,
) -> (f32, Vec<String>) {
    let mut effective_font_size = font_size.max(8.0);
    if fit_to_width && !word_wrap && max_width > 0.0 {
        let width = measure_ui_text_line_width(content, effective_font_size);
        if width > max_width {
            effective_font_size = (effective_font_size * (max_width / width))
                .max(8.0)
                .min(effective_font_size);
        }
    }

    let lines = if word_wrap && max_width > 0.0 {
        wrap_ui_text_lines(content, effective_font_size, max_width)
    } else {
        content.split('\n').map(|line| line.to_owned()).collect()
    };

    (
        effective_font_size,
        if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        },
    )
}

fn wrap_ui_text_lines(content: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in content.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if measure_ui_text_line_width(&candidate, font_size) <= max_width {
                current = candidate;
                continue;
            }

            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }

            if measure_ui_text_line_width(word, font_size) <= max_width {
                current = word.to_owned();
                continue;
            }

            let mut fragment = String::new();
            for ch in word.chars() {
                let candidate = format!("{fragment}{ch}");
                if !fragment.is_empty()
                    && measure_ui_text_line_width(&candidate, font_size) > max_width
                {
                    lines.push(fragment.clone());
                    fragment.clear();
                }
                fragment.push(ch);
            }
            current = fragment;
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

fn measure_ui_text_line_width(content: &str, font_size: f32) -> f32 {
    let effective_font_size = font_size.max(8.0);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    content.chars().count() as f32 * advance
}

fn append_mesh_triangles(
    triangles: &mut Vec<ProjectedTriangle>,
    viewport: &Viewport,
    camera: Transform3,
    transform: Transform3,
    base_color: ColorRgba,
) {
    let corners = [
        Vec3::new(-0.75, -0.75, -0.75),
        Vec3::new(0.75, -0.75, -0.75),
        Vec3::new(0.75, 0.75, -0.75),
        Vec3::new(-0.75, 0.75, -0.75),
        Vec3::new(-0.75, -0.75, 0.75),
        Vec3::new(0.75, -0.75, 0.75),
        Vec3::new(0.75, 0.75, 0.75),
        Vec3::new(-0.75, 0.75, 0.75),
    ]
    .map(|point| transform_point_3d(point, transform));
    let faces = [
        (
            [[0usize, 1usize, 2usize], [0usize, 2usize, 3usize]],
            ColorRgba::new(0.88, 0.34, 0.22, 1.0),
        ),
        (
            [[4usize, 5usize, 6usize], [4usize, 6usize, 7usize]],
            ColorRgba::new(0.22, 0.72, 0.96, 1.0),
        ),
        (
            [[0usize, 1usize, 5usize], [0usize, 5usize, 4usize]],
            ColorRgba::new(0.94, 0.84, 0.28, 1.0),
        ),
        (
            [[2usize, 3usize, 7usize], [2usize, 7usize, 6usize]],
            ColorRgba::new(0.32, 0.82, 0.54, 1.0),
        ),
        (
            [[1usize, 2usize, 6usize], [1usize, 6usize, 5usize]],
            ColorRgba::new(0.82, 0.42, 0.94, 1.0),
        ),
        (
            [[3usize, 0usize, 4usize], [3usize, 4usize, 7usize]],
            ColorRgba::new(0.96, 0.58, 0.18, 1.0),
        ),
    ];

    for (face_triangles, face_tint) in faces {
        for [a, b, c] in face_triangles {
            let world = [corners[a], corners[b], corners[c]];
            let projected = [
                project_point(world[0], camera, *viewport),
                project_point(world[1], camera, *viewport),
                project_point(world[2], camera, *viewport),
            ];
            let [Some(a), Some(b), Some(c)] = projected else {
                continue;
            };
            let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
            let light_dir = normalize(Vec3::new(0.35, 0.7, 0.6));
            let brightness = (0.25 + 0.75 * dot(normal, light_dir).max(0.0)).clamp(0.0, 1.0);
            triangles.push(ProjectedTriangle {
                points: [a.position, b.position, c.position],
                color: modulate_color(blend_colors(base_color, face_tint), brightness),
                depth: (a.depth + b.depth + c.depth) / 3.0,
            });
        }
    }
}

fn append_text_3d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    content: &str,
    transform: Transform3,
    size: f32,
    color: ColorRgba,
) {
    let pixel_size = (size * 0.18).max(0.05);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec3::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                    0.0,
                );
                let max = Vec3::new(min.x + pixel_size, min.y + pixel_size, 0.0);
                let quad = [
                    transform_point_3d(min, transform),
                    transform_point_3d(Vec3::new(max.x, min.y, 0.0), transform),
                    transform_point_3d(max, transform),
                    transform_point_3d(Vec3::new(min.x, max.y, 0.0), transform),
                ];
                let [Some(a), Some(b), Some(c), Some(d)] = quad.map(|point| {
                    project_point(point, camera, *viewport).map(|projected| projected.position)
                }) else {
                    continue;
                };
                push_quad(vertices, a, b, c, d, color);
            }
        }
    }
}

fn project_point(point: Vec3, camera: Transform3, viewport: Viewport) -> Option<ProjectedPoint> {
    let relative = sub(point, camera.translation);
    let camera_space = rotate_inverse(relative, camera.rotation_euler);
    let depth = -camera_space.z;

    if depth <= 0.05 {
        return None;
    }

    let focal = 1.0 / (60.0_f32.to_radians() * 0.5).tan();
    let x = (camera_space.x * focal / viewport.aspect) / depth;
    let y = (camera_space.y * focal) / depth;

    Some(ProjectedPoint {
        position: Vec2::new(x, y),
        depth,
    })
}

fn ndc_from_screen(point: Vec2, viewport: &Viewport) -> Vec2 {
    Vec2::new(
        point.x / viewport.half_width,
        point.y / viewport.half_height,
    )
}

fn ndc_from_world_2d(point: Vec2, camera: Transform2, viewport: &Viewport) -> Vec2 {
    let relative = Vec2::new(
        point.x - camera.translation.x,
        point.y - camera.translation.y,
    );
    ndc_from_screen(relative, viewport)
}

fn ndc_from_world_2d_snapped(point: Vec2, camera: Transform2, viewport: &Viewport) -> Vec2 {
    let relative = Vec2::new(
        (point.x - camera.translation.x).round(),
        (point.y - camera.translation.y).round(),
    );
    ndc_from_screen(relative, viewport)
}

fn ndc_from_ui_screen(point: Vec2, viewport: &Viewport) -> Vec2 {
    Vec2::new(
        point.x / viewport.half_width - 1.0,
        1.0 - point.y / viewport.half_height,
    )
}

fn push_quad(
    vertices: &mut Vec<ColorVertex>,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    color: ColorRgba,
) {
    push_triangle(vertices, [a, b, c], color);
    push_triangle(vertices, [a, c, d], color);
}

fn push_textured_quad(
    vertices: &mut Vec<TextureVertex>,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    uv: TextureUvRect,
    color: ColorRgba,
) {
    let bottom_left = Vec2::new(uv.u0, uv.v1);
    let bottom_right = Vec2::new(uv.u1, uv.v1);
    let top_right = Vec2::new(uv.u1, uv.v0);
    let top_left = Vec2::new(uv.u0, uv.v0);
    vertices.push(TextureVertex::new(a, bottom_left, color));
    vertices.push(TextureVertex::new(b, bottom_right, color));
    vertices.push(TextureVertex::new(c, top_right, color));
    vertices.push(TextureVertex::new(a, bottom_left, color));
    vertices.push(TextureVertex::new(c, top_right, color));
    vertices.push(TextureVertex::new(d, top_left, color));
}

fn push_triangle(vertices: &mut Vec<ColorVertex>, points: [Vec2; 3], color: ColorRgba) {
    vertices.push(ColorVertex::new(points[0], color));
    vertices.push(ColorVertex::new(points[1], color));
    vertices.push(ColorVertex::new(points[2], color));
}

fn transform_point_2d(point: Vec2, transform: Transform2) -> Vec2 {
    let scaled = Vec2::new(point.x * transform.scale.x, point.y * transform.scale.y);
    let sin = transform.rotation_radians.sin();
    let cos = transform.rotation_radians.cos();
    let rotated = Vec2::new(
        scaled.x * cos - scaled.y * sin,
        scaled.x * sin + scaled.y * cos,
    );
    Vec2::new(
        rotated.x + transform.translation.x,
        rotated.y + transform.translation.y,
    )
}

fn transform_point_3d(point: Vec3, transform: Transform3) -> Vec3 {
    let scaled = Vec3::new(
        point.x * transform.scale.x,
        point.y * transform.scale.y,
        point.z * transform.scale.z,
    );
    let rotated_x = rotate_x(scaled, transform.rotation_euler.x);
    let rotated_y = rotate_y(rotated_x, transform.rotation_euler.y);
    let rotated_z = rotate_z(rotated_y, transform.rotation_euler.z);
    Vec3::new(
        rotated_z.x + transform.translation.x,
        rotated_z.y + transform.translation.y,
        rotated_z.z + transform.translation.z,
    )
}

fn rotate_inverse(point: Vec3, rotation: Vec3) -> Vec3 {
    let around_z = rotate_z(point, -rotation.z);
    let around_y = rotate_y(around_z, -rotation.y);
    rotate_x(around_y, -rotation.x)
}

fn rotate_x(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x,
        point.y * cos - point.z * sin,
        point.y * sin + point.z * cos,
    )
}

fn rotate_y(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x * cos + point.z * sin,
        point.y,
        -point.x * sin + point.z * cos,
    )
}

fn rotate_z(point: Vec3, angle: f32) -> Vec3 {
    let sin = angle.sin();
    let cos = angle.cos();
    Vec3::new(
        point.x * cos - point.y * sin,
        point.x * sin + point.y * cos,
        point.z,
    )
}

fn transform2_from_transform3(transform: Transform3) -> Transform2 {
    Transform2 {
        translation: Vec2::new(transform.translation.x, transform.translation.y),
        rotation_radians: transform.rotation_euler.z,
        scale: Vec2::new(transform.scale.x, transform.scale.y),
    }
}

fn sprite_color(asset_key: &str) -> ColorRgba {
    if asset_key.contains("square") {
        ColorRgba::new(0.18, 0.74, 1.0, 1.0)
    } else {
        ColorRgba::new(0.46, 0.78, 0.54, 1.0)
    }
}

fn mesh_color(asset_key: &str) -> ColorRgba {
    if asset_key.contains("cube") {
        ColorRgba::new(0.92, 0.46, 0.18, 1.0)
    } else {
        ColorRgba::new(0.68, 0.7, 0.92, 1.0)
    }
}

fn modulate_color(color: ColorRgba, factor: f32) -> ColorRgba {
    ColorRgba::new(
        color.r * factor,
        color.g * factor,
        color.b * factor,
        color.a,
    )
}

fn blend_colors(base: ColorRgba, accent: ColorRgba) -> ColorRgba {
    ColorRgba::new(
        (base.r * 0.45 + accent.r * 0.55).clamp(0.0, 1.0),
        (base.g * 0.45 + accent.g * 0.55).clamp(0.0, 1.0),
        (base.b * 0.45 + accent.b * 0.55).clamp(0.0, 1.0),
        base.a,
    )
}

fn spritesheet_frame_color(frame: u32) -> ColorRgba {
    match frame % 8 {
        0 => ColorRgba::new(0.95, 0.36, 0.28, 1.0),
        1 => ColorRgba::new(0.95, 0.6, 0.22, 1.0),
        2 => ColorRgba::new(0.93, 0.82, 0.24, 1.0),
        3 => ColorRgba::new(0.36, 0.82, 0.42, 1.0),
        4 => ColorRgba::new(0.22, 0.72, 0.92, 1.0),
        5 => ColorRgba::new(0.34, 0.48, 0.95, 1.0),
        6 => ColorRgba::new(0.66, 0.34, 0.95, 1.0),
        _ => ColorRgba::new(0.92, 0.3, 0.72, 1.0),
    }
}

fn sub(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x - right.x, left.y - right.y, left.z - right.z)
}

fn cross(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(
        left.y * right.z - left.z * right.y,
        left.z * right.x - left.x * right.z,
        left.x * right.y - left.y * right.x,
    )
}

fn dot(left: Vec3, right: Vec3) -> f32 {
    left.x * right.x + left.y * right.y + left.z * right.z
}

fn normalize(value: Vec3) -> Vec3 {
    let length = dot(value, value).sqrt();
    if length <= f32::EPSILON {
        Vec3::ZERO
    } else {
        Vec3::new(value.x / length, value.y / length, value.z / length)
    }
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b11100,
        ],
        '.' => [0, 0, 0, 0, 0, 0b00110, 0b00110],
        ',' => [0, 0, 0, 0, 0, 0b00110, 0b00100],
        ':' => [0, 0b00110, 0b00110, 0, 0b00110, 0b00110, 0],
        ';' => [0, 0b00110, 0b00110, 0, 0b00110, 0b00100, 0],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '=' => [0, 0, 0b11111, 0, 0b11111, 0, 0],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01000, 0b10011, 0b10110, 0b00110,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '<' => [0, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0],
        '>' => [0, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0],
        '/' => [0b00001, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        _ => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
    }
}

fn vertices_as_bytes(vertices: &[ColorVertex]) -> &[u8] {
    let byte_len = std::mem::size_of_val(vertices);
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast::<u8>(), byte_len) }
}

fn texture_vertices_as_bytes(vertices: &[TextureVertex]) -> &[u8] {
    let byte_len = std::mem::size_of_val(vertices);
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast::<u8>(), byte_len) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use amigo_assets::{AssetKey, AssetSourceKind, PreparedAsset, PreparedAssetKind};

    use super::{
        glyph_rows, infer_sprite_sheet_from_asset, infer_tileset_from_asset, resolve_image_path,
        tile_uv_rect,
    };
    use amigo_math::Vec2;

    #[test]
    fn glyph_rows_cover_hello_world_letters() {
        for ch in ['H', 'E', 'L', 'O', 'W', 'R', 'D', ' '] {
            assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
        }
    }

    #[test]
    fn glyph_rows_cover_basic_scripting_demo_characters() {
        for ch in
            "BASIC SCRIPTING DEMO LEFT / RIGHT rotate square via EntityRef.rotate_2d()".chars()
        {
            assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
        }
    }

    #[test]
    fn glyph_rows_cover_ui_showcase_punctuation() {
        for ch in
            "Theme: space_dark volume=65% F1 dark | F2 clean | T cycle [-] [+] UI; <START>".chars()
        {
            assert!(glyph_rows(ch).iter().any(|row| *row != 0) || ch == ' ');
        }
    }

    #[test]
    fn resolves_image_path_relative_to_metadata_file() {
        let prepared = PreparedAsset {
            key: AssetKey::new("test/textures/player"),
            source: AssetSourceKind::Mod("test".to_owned()),
            resolved_path: PathBuf::from("mods/test/textures/player.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::SpriteSheet2d,
            label: None,
            format: None,
            metadata: BTreeMap::from([("image".to_owned(), "player.png".to_owned())]),
        };

        assert_eq!(
            resolve_image_path(&prepared),
            Some(PathBuf::from("mods/test/textures/player.png"))
        );
    }

    #[test]
    fn infers_sprite_sheet_from_prepared_metadata() {
        let prepared = PreparedAsset {
            key: AssetKey::new("test/textures/player"),
            source: AssetSourceKind::Mod("test".to_owned()),
            resolved_path: PathBuf::from("mods/test/textures/player.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::SpriteSheet2d,
            label: None,
            format: None,
            metadata: BTreeMap::from([
                ("columns".to_owned(), "8".to_owned()),
                ("rows".to_owned(), "4".to_owned()),
                ("frame_size.x".to_owned(), "32".to_owned()),
                ("frame_size.y".to_owned(), "32".to_owned()),
                ("fps".to_owned(), "10".to_owned()),
                ("looping".to_owned(), "true".to_owned()),
            ]),
        };

        let sheet = infer_sprite_sheet_from_asset(&prepared).expect("sheet metadata should parse");
        assert_eq!(sheet.columns, 8);
        assert_eq!(sheet.rows, 4);
        assert_eq!(sheet.frame_count, 32);
        assert_eq!(sheet.frame_size.x, 32.0);
        assert_eq!(sheet.frame_size.y, 32.0);
        assert_eq!(sheet.fps, 10.0);
        assert!(sheet.looping);
    }

    #[test]
    fn infers_tileset_with_derived_variants_from_prepared_metadata() {
        let prepared = PreparedAsset {
            key: AssetKey::new("test/tilesets/platformer"),
            source: AssetSourceKind::Mod("test".to_owned()),
            resolved_path: PathBuf::from("mods/test/tilesets/platformer.yml"),
            byte_len: 0,
            kind: PreparedAssetKind::TileSet2d,
            label: None,
            format: None,
            metadata: BTreeMap::from([
                ("columns".to_owned(), "1".to_owned()),
                ("rows".to_owned(), "1".to_owned()),
                ("tile_size.x".to_owned(), "16".to_owned()),
                ("tile_size.y".to_owned(), "16".to_owned()),
                ("tiles.ground_single.id".to_owned(), "0".to_owned()),
                ("tiles.ground_left_cap.id".to_owned(), "1".to_owned()),
                ("tiles.ground_right_cap.id".to_owned(), "2".to_owned()),
                ("tiles.ground_top_cap.id".to_owned(), "3".to_owned()),
                ("tiles.ground_bottom_cap.id".to_owned(), "4".to_owned()),
                (
                    "derived_variants.ground_left_cap.from_tile".to_owned(),
                    "ground_single".to_owned(),
                ),
                (
                    "derived_variants.ground_left_cap.mode".to_owned(),
                    "split_x".to_owned(),
                ),
                (
                    "derived_variants.ground_left_cap.segment".to_owned(),
                    "left".to_owned(),
                ),
                (
                    "derived_variants.ground_right_cap.from_tile".to_owned(),
                    "ground_single".to_owned(),
                ),
                (
                    "derived_variants.ground_right_cap.mode".to_owned(),
                    "split_x".to_owned(),
                ),
                (
                    "derived_variants.ground_right_cap.segment".to_owned(),
                    "right".to_owned(),
                ),
                (
                    "derived_variants.ground_top_cap.from_tile".to_owned(),
                    "ground_single".to_owned(),
                ),
                (
                    "derived_variants.ground_top_cap.mode".to_owned(),
                    "split_y".to_owned(),
                ),
                (
                    "derived_variants.ground_top_cap.segment".to_owned(),
                    "top".to_owned(),
                ),
                (
                    "derived_variants.ground_bottom_cap.from_tile".to_owned(),
                    "ground_single".to_owned(),
                ),
                (
                    "derived_variants.ground_bottom_cap.mode".to_owned(),
                    "split_y".to_owned(),
                ),
                (
                    "derived_variants.ground_bottom_cap.segment".to_owned(),
                    "bottom".to_owned(),
                ),
            ]),
        };

        let tileset = infer_tileset_from_asset(&prepared, Vec2::new(16.0, 16.0))
            .expect("tileset should parse");

        let left = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 1);
        assert!(left.u0 > 0.0 && left.u0 < 0.1);
        assert!(left.u1 > 0.4 && left.u1 < 0.5);
        assert!(left.v0 > 0.0 && left.v0 < 0.1);
        assert!(left.v1 > 0.9 && left.v1 < 1.0);

        let right = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 2);
        assert!(right.u0 > 0.5 && right.u0 < 0.6);
        assert!(right.u1 > 0.9 && right.u1 < 1.0);
        assert!(right.v0 > 0.0 && right.v0 < 0.1);
        assert!(right.v1 > 0.9 && right.v1 < 1.0);

        let top = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 3);
        assert!(top.u0 > 0.0 && top.u0 < 0.1);
        assert!(top.u1 > 0.9 && top.u1 < 1.0);
        assert!(top.v0 > 0.0 && top.v0 < 0.1);
        assert!(top.v1 > 0.4 && top.v1 < 0.5);

        let bottom = tile_uv_rect(Vec2::new(16.0, 16.0), &tileset, 4);
        assert!(bottom.u0 > 0.0 && bottom.u0 < 0.1);
        assert!(bottom.u1 > 0.9 && bottom.u1 < 1.0);
        assert!(bottom.v0 > 0.5 && bottom.v0 < 0.6);
        assert!(bottom.v1 > 0.9 && bottom.v1 < 1.0);
    }
}
