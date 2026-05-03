use amigo_core::{AmigoError, AmigoResult};
use amigo_render_api::RenderInitializationReport;
use amigo_window_api::{WindowSize, WindowSurfaceHandles};
use crate::backend::helpers;
use crate::backend::types::{
    WgpuHeadlessContext, WgpuOffscreenTarget, WgpuRenderBackend, WgpuSurfaceState,
};

impl WgpuRenderBackend {
    pub fn initialize_headless(&self) -> AmigoResult<WgpuHeadlessContext> {
        let instance = wgpu::Instance::default();
        let adapter = helpers::request_adapter(&instance, None)?;
        let adapter_info = adapter.get_info();
        let descriptor = helpers::create_device_descriptor();
        let (device, queue) = helpers::block_on(adapter.request_device(&descriptor))
            .map_err(|error| AmigoError::Message(error.to_string()))?;

        Ok(WgpuHeadlessContext {
            report: RenderInitializationReport {
                backend_name: "wgpu",
                adapter_name: adapter_info.name,
                adapter_backend: format!("{:?}", adapter_info.backend),
                device_type: format!("{:?}", adapter_info.device_type),
                shader_language: "wgsl",
                queue_ready: true,
            },
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn initialize_for_window(
        &self,
        handles: WindowSurfaceHandles,
    ) -> AmigoResult<WgpuSurfaceState> {
        let instance = wgpu::Instance::default();
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: handles.raw_display_handle,
                raw_window_handle: handles.raw_window_handle,
            })
        }
        .map_err(|error| AmigoError::Message(error.to_string()))?;

        let adapter = helpers::request_adapter(&instance, Some(&surface))?;
        let adapter_info = adapter.get_info();
        let descriptor = helpers::create_device_descriptor();
        let (device, queue) = helpers::block_on(adapter.request_device(&descriptor))
            .map_err(|error| AmigoError::Message(error.to_string()))?;

        let config = surface
            .get_default_config(
                &adapter,
                handles.size.width.max(1),
                handles.size.height.max(1),
            )
            .ok_or_else(|| {
                AmigoError::Message("failed to derive a default surface configuration".to_owned())
            })?;
        surface.configure(&device, &config);

        Ok(WgpuSurfaceState {
            report: RenderInitializationReport {
                backend_name: "wgpu",
                adapter_name: adapter_info.name,
                adapter_backend: format!("{:?}", adapter_info.backend),
                device_type: format!("{:?}", adapter_info.device_type),
                shader_language: "wgsl",
                queue_ready: true,
            },
            surface,
            device,
            queue,
            config,
        })
    }

    pub fn initialize_offscreen(
        &self,
        width: u32,
        height: u32,
    ) -> AmigoResult<WgpuOffscreenTarget> {
        let context = self.initialize_headless()?;
        let width = width.max(1);
        let height = height.max(1);
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("amigo-offscreen-render-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(WgpuOffscreenTarget {
            report: context.report,
            device: context.device,
            queue: context.queue,
            width,
            height,
            format,
            texture,
            view,
        })
    }
}

impl WgpuSurfaceState {
    pub fn size(&self) -> WindowSize {
        WindowSize {
            width: self.config.width,
            height: self.config.height,
        }
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render_default_frame(&mut self) -> AmigoResult<()> {
        self.render_clear_rgba(0.08, 0.09, 0.12, 1.0)
    }

    pub fn render_clear_rgba(&mut self, r: f64, g: f64, b: f64, a: f64) -> AmigoResult<()> {
        self.render_clear(wgpu::Color { r, g, b, a })
    }

    pub fn render_clear(&mut self, color: wgpu::Color) -> AmigoResult<()> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Occluded => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("amigo-clear-frame"),
            });

        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("amigo-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}

impl WgpuOffscreenTarget {
    pub fn read_rgba8(&self) -> AmigoResult<Vec<u8>> {
        let unpadded_bytes_per_row = self.width as u64 * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let output_size = padded_bytes_per_row * self.height as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("amigo-offscreen-readback-buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("amigo-offscreen-readback-encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        let submission = self.queue.submit(Some(encoder.finish()));
        let slice = output_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .map_err(|error| AmigoError::Message(error.to_string()))?;
        receiver
            .recv()
            .map_err(|error| AmigoError::Message(error.to_string()))?
            .map_err(|error| AmigoError::Message(error.to_string()))?;

        let mapped = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in mapped.chunks(padded_bytes_per_row as usize).take(self.height as usize) {
            pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        output_buffer.unmap();

        Ok(pixels)
    }
}
