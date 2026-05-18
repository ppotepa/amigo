use super::offscreen_ops::append_fullscreen_texture_vertices;
use crate::renderer::*;

fn append_surface_texture_rect_vertices(
    vertices: &mut Vec<TextureVertex>,
    surface_width: f32,
    surface_height: f32,
    placement: WgpuGameViewportPlacement,
) {
    let rect = placement.surface_rect;
    let surface_width = surface_width.max(1.0);
    let surface_height = surface_height.max(1.0);
    let logical_width = placement.logical_width.max(1) as f32;
    let logical_height = placement.logical_height.max(1) as f32;
    let base_scale = (rect.width / logical_width)
        .min(rect.height / logical_height)
        .max(0.0001);
    let zoom = placement.zoom.max(0.01);
    let draw_scale = base_scale * zoom;

    let visible_left = (-placement.pan_x / draw_scale).clamp(0.0, logical_width);
    let visible_top = (-placement.pan_y / draw_scale).clamp(0.0, logical_height);
    let visible_right = ((rect.width - placement.pan_x) / draw_scale).clamp(0.0, logical_width);
    let visible_bottom = ((rect.height - placement.pan_y) / draw_scale).clamp(0.0, logical_height);

    if visible_right <= visible_left || visible_bottom <= visible_top {
        return;
    }

    let x0 = rect.x + placement.pan_x + visible_left * draw_scale;
    let y0 = rect.y + placement.pan_y + visible_top * draw_scale;
    let x1 = rect.x + placement.pan_x + visible_right * draw_scale;
    let y1 = rect.y + placement.pan_y + visible_bottom * draw_scale;

    let left = x0 / surface_width * 2.0 - 1.0;
    let right = x1 / surface_width * 2.0 - 1.0;
    let top = 1.0 - y0 / surface_height * 2.0;
    let bottom = 1.0 - y1 / surface_height * 2.0;

    push_textured_quad(
        vertices,
        Vec2::new(left, bottom),
        Vec2::new(right, bottom),
        Vec2::new(right, top),
        Vec2::new(left, top),
        TextureUvRect {
            u0: visible_left / logical_width,
            v0: visible_top / logical_height,
            u1: visible_right / logical_width,
            v1: visible_bottom / logical_height,
        },
        ColorRgba::new(1.0, 1.0, 1.0, 1.0),
    );
}

impl WgpuSceneRenderer {
    pub(super) fn render_texture_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        source_view: &wgpu::TextureView,
        assets: &AssetCatalog,
        surface_overlay_ui: &[UiOverlayDocument],
        game_viewport: Option<WgpuGameViewportPlacement>,
        emergency_overlay: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        let mut world_batch = self.create_fullscreen_texture_batch(
            &surface.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        if let Some(placement) = game_viewport {
            append_surface_texture_rect_vertices(
                &mut world_batch.vertices,
                surface.config.width as f32,
                surface.config.height as f32,
                placement,
            );
        } else {
            append_fullscreen_texture_vertices(&mut world_batch.vertices);
        }

        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();

        if !surface_overlay_ui.is_empty() {
            let (mut ui_color_batches, mut ui_textures) =
                self.surface_ui_batches(surface, assets, surface_overlay_ui);
            color_batches.append(&mut ui_color_batches);
            ui_texture_batches.append(&mut ui_textures);
        }

        let mut emergency_batch =
            self.emergency_overlay_color_batch_for_surface(surface, emergency_overlay);
        color_batches.append(&mut emergency_batch);

        self.render_surface_batches(
            surface,
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
            &[world_batch],
            color_batches.as_slice(),
            ui_texture_batches.as_slice(),
        )
    }

    pub(super) fn render_surface_batches(
        &self,
        surface: &mut WgpuSurfaceState,
        load_op: wgpu::LoadOp<wgpu::Color>,
        texture_batches: &[TextureBatch],
        color_batches: &[ColorBatch],
        ui_texture_batches: &[TextureBatch],
    ) -> AmigoResult<()> {
        let texture_batches = texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let color_batches = color_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();
        let ui_texture_batches = ui_texture_batches
            .iter()
            .filter(|batch| !batch.vertices.is_empty())
            .collect::<Vec<_>>();

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
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                surface
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("amigo-scene-ui-texture-vertices"),
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
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            for (index, batch) in texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in color_batches.iter().enumerate() {
                pass.set_pipeline(self.color_pipeline_for(batch.blend_mode));
                pass.set_vertex_buffer(0, color_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }

            for (index, batch) in ui_texture_batches.iter().enumerate() {
                pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
                pass.set_bind_group(0, &batch.bind_group, &[]);
                pass.set_vertex_buffer(0, ui_texture_vertex_buffers[index].slice(..));
                pass.draw(0..batch.vertices.len() as u32, 0..1);
            }
        }

        surface.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
