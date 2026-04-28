use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::mem::size_of;

use amigo_2d_sprite::{Sprite, SpriteSceneService, SpriteSheet};
use amigo_2d_text::Text2dSceneService;
use amigo_3d_material::MaterialSceneService;
use amigo_3d_mesh::MeshSceneService;
use amigo_3d_text::Text3dSceneService;
use amigo_core::AmigoResult;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
use amigo_scene::SceneService;
use wgpu::util::DeviceExt;

use crate::WgpuSurfaceState;
use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiTextAnchor, UiViewportSize, build_ui_overlay_primitives,
};

const SCENE_SHADER: &str = r#"
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

pub struct WgpuSceneRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl WgpuSceneRenderer {
    pub fn new(surface: &WgpuSurfaceState) -> Self {
        let shader = surface
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("amigo-scene-shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SCENE_SHADER)),
            });
        let pipeline_layout =
            surface
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("amigo-scene-pipeline-layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });
        let pipeline = surface
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("amigo-scene-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[ColorVertex::layout()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
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

        Self { pipeline }
    }

    pub fn render_scene(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
    ) -> AmigoResult<()> {
        self.render_scene_with_ui_primitives(
            surface,
            scene,
            sprites,
            text2d,
            meshes,
            materials,
            text3d,
            &[],
        )
    }

    pub fn render_scene_with_ui_documents(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_documents: &[UiOverlayDocument],
    ) -> AmigoResult<()> {
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        self.render_scene_with_ui_primitives(
            surface,
            scene,
            sprites,
            text2d,
            meshes,
            materials,
            text3d,
            &ui_primitives,
        )
    }

    pub fn render_scene_with_ui_primitives(
        &mut self,
        surface: &mut WgpuSurfaceState,
        scene: &SceneService,
        sprites: &SpriteSceneService,
        text2d: &Text2dSceneService,
        meshes: &MeshSceneService,
        materials: &MaterialSceneService,
        text3d: Option<&Text3dSceneService>,
        ui_primitives: &[UiDrawPrimitive],
    ) -> AmigoResult<()> {
        let viewport = Viewport::from_surface(surface);
        let mut vertices = Vec::new();

        for command in sprites.commands() {
            let transform = resolve_transform2(scene, &command.entity_name, command.transform);
            append_sprite_vertices(
                &mut vertices,
                &viewport,
                transform,
                &command.sprite,
                sprite_color(command.sprite.texture.as_str()),
            );
        }

        for command in text2d.commands() {
            let transform = resolve_transform2(scene, &command.entity_name, command.text.transform);
            append_text_2d_vertices(
                &mut vertices,
                &viewport,
                &command.text.content,
                transform,
                command.text.bounds,
                ColorRgba::new(1.0, 0.96, 0.82, 1.0),
            );
        }

        let camera = resolve_camera_transform(scene);
        let material_lookup = material_lookup(materials);
        let mut projected_triangles = Vec::new();

        for command in meshes.commands() {
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
            push_triangle(&mut vertices, triangle.points, triangle.color);
        }

        if let Some(text3d) = text3d {
            for command in text3d.commands() {
                let transform =
                    resolve_transform3(scene, &command.entity_name, command.text.transform);
                append_text_3d_vertices(
                    &mut vertices,
                    &viewport,
                    camera,
                    &command.text.content,
                    transform,
                    command.text.size,
                    ColorRgba::new(0.94, 0.98, 1.0, 1.0),
                );
            }
        }

        append_ui_overlay_vertices(&mut vertices, &viewport, ui_primitives);

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

        let vertex_buffer = if vertices.is_empty() {
            None
        } else {
            Some(
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-vertices"),
                        contents: vertices_as_bytes(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        };

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

            if let Some(vertex_buffer) = vertex_buffer.as_ref() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..vertices.len() as u32, 0..1);
            }
        }

        surface.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
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

fn material_lookup(materials: &MaterialSceneService) -> BTreeMap<String, ColorRgba> {
    materials
        .commands()
        .into_iter()
        .map(|command| (command.entity_name, command.material.albedo))
        .collect()
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
            } => append_text_screen_space_vertices(
                vertices, viewport, content, *rect, *font_size, *color, *anchor,
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

fn append_sprite_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
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
        ndc_from_screen(points[0], viewport),
        ndc_from_screen(points[1], viewport),
        ndc_from_screen(points[2], viewport),
        ndc_from_screen(points[3], viewport),
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
            ndc_from_screen(marker_points[0], viewport),
            ndc_from_screen(marker_points[1], viewport),
            ndc_from_screen(marker_points[2], viewport),
            ndc_from_screen(marker_points[3], viewport),
            ColorRgba::new(0.98, 0.98, 0.98, 1.0),
        );
    }
}

fn append_sprite_sheet_overlay(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
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
            ndc_from_screen(quad[0], viewport),
            ndc_from_screen(quad[1], viewport),
            ndc_from_screen(quad[2], viewport),
            ndc_from_screen(quad[3], viewport),
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
        ndc_from_screen(preview_quad[0], viewport),
        ndc_from_screen(preview_quad[1], viewport),
        ndc_from_screen(preview_quad[2], viewport),
        ndc_from_screen(preview_quad[3], viewport),
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
        ndc_from_screen(marker_quad[0], viewport),
        ndc_from_screen(marker_quad[1], viewport),
        ndc_from_screen(marker_quad[2], viewport),
        ndc_from_screen(marker_quad[3], viewport),
        ColorRgba::new(0.98, 0.98, 0.98, 1.0),
    );
}

fn append_text_2d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
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
                    ndc_from_screen(quad[0], viewport),
                    ndc_from_screen(quad[1], viewport),
                    ndc_from_screen(quad[2], viewport),
                    ndc_from_screen(quad[3], viewport),
                    color,
                );
            }
        }
    }
}

fn append_text_screen_space_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    content: &str,
    rect: crate::ui_overlay::UiRect,
    font_size: f32,
    color: ColorRgba,
    anchor: UiTextAnchor,
) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }

    let effective_font_size = font_size.max(8.0);
    let pixel_size = effective_font_size / 7.0;
    let advance = 6.0 * pixel_size;
    let line_height = effective_font_size * 1.2;
    let lines: Vec<&str> = content.split('\n').collect();
    let text_width = lines
        .iter()
        .map(|line| line.chars().count() as f32 * advance)
        .fold(0.0, f32::max);
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
        for (index, ch) in line.chars().enumerate() {
            let rows = glyph_rows(ch);
            let glyph_origin_x = origin_x + index as f32 * advance;
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
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111],
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

#[cfg(test)]
mod tests {
    use super::glyph_rows;

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
}
