use std::collections::{BTreeMap, BTreeSet};

use amigo_core::AmigoResult;
use amigo_render_api::{FrameResourceId, RenderInitializationReport};

use crate::WgpuOffscreenTarget;

pub(crate) struct WgpuTransientTexture {
    pub(crate) target: WgpuOffscreenTarget,
}

#[derive(Default)]
pub(crate) struct WgpuFrameResourceAllocator {
    textures: BTreeMap<FrameResourceId, WgpuTransientTexture>,
}

impl WgpuFrameResourceAllocator {
    pub(crate) fn retain_ids(&mut self, used: &BTreeSet<FrameResourceId>) {
        self.textures.retain(|id, _| used.contains(id));
    }

    pub(crate) fn create_color_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: FrameResourceId,
        label: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> AmigoResult<()> {
        let width = width.max(1);
        let height = height.max(1);
        if let Some(existing) = self.textures.get(&id) {
            if existing.target.width == width
                && existing.target.height == height
                && existing.target.format == format
            {
                return Ok(());
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let report = RenderInitializationReport {
            backend_name: "offscreen",
            adapter_name: String::new(),
            adapter_backend: "offscreen".to_owned(),
            device_type: String::new(),
            shader_language: "wgsl",
            queue_ready: true,
        };

        let target = WgpuOffscreenTarget {
            report,
            device: device.clone(),
            queue: queue.clone(),
            width,
            height,
            format,
            texture,
            view,
            _depth_texture: depth_texture,
            depth_view,
        };

        self.textures.insert(id, WgpuTransientTexture { target });
        Ok(())
    }

    pub(crate) fn target(&self, id: FrameResourceId) -> Option<&WgpuOffscreenTarget> {
        self.textures.get(&id).map(|entry| &entry.target)
    }

    pub(crate) fn target_mut(&mut self, id: FrameResourceId) -> Option<&mut WgpuOffscreenTarget> {
        self.textures.get_mut(&id).map(|entry| &mut entry.target)
    }
}
