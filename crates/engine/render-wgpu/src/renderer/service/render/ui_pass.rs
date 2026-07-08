use crate::renderer::*;
use amigo_render_api::RenderAssetSource;

impl WgpuSceneRenderer {
    pub(super) fn surface_ui_batches(
        &mut self,
        surface: &WgpuSurfaceState,
        assets: &dyn RenderAssetSource,
        ui_documents: &[UiOverlayDocument],
    ) -> (Vec<ColorBatch>, Vec<TextureBatch>) {
        if ui_documents.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let viewport = Viewport::from_surface(surface);
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(surface.config.width as f32, surface.config.height as f32),
            ui_documents,
        );
        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());

        for primitive in &ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if self.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if self.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &surface.device,
                    &surface.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());
        (color_batches, ui_texture_batches)
    }

    pub(super) fn render_ui_documents_to_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        assets: &dyn RenderAssetSource,
        ui_documents: &[UiOverlayDocument],
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> AmigoResult<()> {
        if ui_documents.is_empty() {
            return Ok(());
        }

        let viewport = Viewport::from_offscreen(target);
        let ui_primitives = build_ui_overlay_primitives(
            UiViewportSize::new(target.width as f32, target.height as f32),
            ui_documents,
        );
        let mut color_batches = Vec::new();
        let mut ui_texture_batches = Vec::new();
        let mut ui_color_primitives = Vec::with_capacity(ui_primitives.len());

        for primitive in &ui_primitives {
            if let UiDrawPrimitive::Text {
                rect,
                content,
                color,
                font_size,
                font: Some(font),
                anchor,
                word_wrap,
                fit_to_width,
            } = primitive
            {
                if self.append_ui_ttf_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }

                if self.append_ui_bitmap_font_texture_batch(
                    &mut ui_texture_batches,
                    &target.device,
                    &target.queue,
                    assets,
                    &viewport,
                    font,
                    content,
                    *rect,
                    *font_size,
                    *color,
                    *anchor,
                    *word_wrap,
                    *fit_to_width,
                ) {
                    continue;
                }
            }
            ui_color_primitives.push(primitive.clone());
        }

        let vertices = color_batch_vertices(&mut color_batches, ParticleBlendMode2d::Alpha);
        append_ui_overlay_vertices(vertices, &viewport, &ui_color_primitives);

        color_batches.retain(|batch| !batch.vertices.is_empty());
        ui_texture_batches.retain(|batch| !batch.vertices.is_empty());

        let color_vertex_buffers = color_batches
            .iter()
            .map(|batch| {
                create_uploaded_vertex_buffer(
                    &target.device,
                    &target.queue,
                    "amigo-offscreen-ui-color-vertices",
                    vertices_as_bytes(&batch.vertices),
                )
            })
            .collect::<Vec<_>>();
        let ui_texture_vertex_buffers = ui_texture_batches
            .iter()
            .map(|batch| {
                create_uploaded_vertex_buffer(
                    &target.device,
                    &target.queue,
                    "amigo-offscreen-ui-texture-vertices",
                    texture_vertices_as_bytes(&batch.vertices),
                )
            })
            .collect::<Vec<_>>();

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-ui-render-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-ui-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
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

        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

fn create_uploaded_vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    contents: &[u8],
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (contents.len() as u64).max(4),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    if !contents.is_empty() {
        queue.write_buffer(&buffer, 0, contents);
    }
    buffer
}
