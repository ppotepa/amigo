use crate::renderer::*;

pub(super) fn compatible_offscreen_target(
    template: &WgpuOffscreenTarget,
    label: &'static str,
) -> WgpuOffscreenTarget {
    let texture = template.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: template.width.max(1),
            height: template.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: template.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    WgpuOffscreenTarget {
        report: template.report.clone(),
        device: template.device.clone(),
        queue: template.queue.clone(),
        width: template.width,
        height: template.height,
        format: template.format,
        texture,
        view,
    }
}

pub(super) fn append_fullscreen_texture_vertices(vertices: &mut Vec<TextureVertex>) {
    push_textured_quad(
        vertices,
        Vec2::new(-1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(-1.0, 1.0),
        TextureUvRect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        },
        ColorRgba::new(1.0, 1.0, 1.0, 1.0),
    );
}

impl WgpuSceneRenderer {
    pub(crate) fn copy_offscreen_to_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        source_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let mut world_batch = self.create_fullscreen_texture_batch(
            &target.device,
            source_view,
            TextureBlendMode::Opaque,
        );
        append_fullscreen_texture_vertices(&mut world_batch.vertices);
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-copy-vertex-buffer"),
                contents: texture_vertices_as_bytes(&world_batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-copy-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-copy-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
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

            pass.set_pipeline(self.texture_pipeline_for(world_batch.blend_mode));
            pass.set_bind_group(0, &world_batch.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..world_batch.vertices.len() as u32, 0..1);
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub(crate) fn composite_offscreen_over_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        source_view: &wgpu::TextureView,
    ) -> AmigoResult<()> {
        let mut batch = self.create_fullscreen_texture_batch(
            &target.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        append_fullscreen_texture_vertices(&mut batch.vertices);
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-composite-vertex-buffer"),
                contents: texture_vertices_as_bytes(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-composite-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-composite-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
            pass.set_bind_group(0, &batch.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..batch.vertices.len() as u32, 0..1);
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub(crate) fn composite_tinted_offscreen_over_offscreen(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        source_view: &wgpu::TextureView,
        tint: ColorRgba,
    ) -> AmigoResult<()> {
        let mut batch = self.create_fullscreen_texture_batch(
            &target.device,
            source_view,
            TextureBlendMode::Alpha,
        );
        append_fullscreen_texture_vertices(&mut batch.vertices);
        for vertex in &mut batch.vertices {
            vertex.color = [
                tint.r.clamp(0.0, 1.0),
                tint.g.clamp(0.0, 1.0),
                tint.b.clamp(0.0, 1.0),
                tint.a.clamp(0.0, 1.0),
            ];
        }
        let vertex_buffer = target
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("amigo-offscreen-composite-tinted-vertex-buffer"),
                contents: texture_vertices_as_bytes(&batch.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-composite-tinted-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-composite-tinted-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(self.texture_pipeline_for(batch.blend_mode));
            pass.set_bind_group(0, &batch.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..batch.vertices.len() as u32, 0..1);
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub(crate) fn clear_offscreen_to_color(
        &mut self,
        target: &mut WgpuOffscreenTarget,
        color: wgpu::Color,
    ) -> AmigoResult<()> {
        let mut encoder = target
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-offscreen-clear-encoder"),
            });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-offscreen-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }
        target.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    pub(super) fn create_fullscreen_texture_batch(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        blend_mode: TextureBlendMode,
    ) -> TextureBatch {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("amigo-post-fx-fullscreen-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("amigo-post-fx-fullscreen-bind-group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        TextureBatch {
            blend_mode,
            bind_group,
            _owned_sampler: Some(sampler),
            vertices: Vec::new(),
        }
    }
}
