use std::collections::BTreeMap;

use amigo_render_api::FrameResourceId;

#[allow(dead_code)]
pub(crate) struct WgpuTransientTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: wgpu::TextureFormat,
}

#[derive(Default)]
pub(crate) struct WgpuFrameResourceAllocator {
    textures: BTreeMap<FrameResourceId, WgpuTransientTexture>,
}

impl WgpuFrameResourceAllocator {
    pub(crate) fn clear(&mut self) {
        self.textures.clear();
    }

    pub(crate) fn create_color_texture(
        &mut self,
        device: &wgpu::Device,
        id: FrameResourceId,
        label: &str,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
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

        self.textures.insert(
            id,
            WgpuTransientTexture {
                texture,
                view,
                width,
                height,
                format,
            },
        );
    }

    #[allow(dead_code)]
    pub(crate) fn texture(&self, id: FrameResourceId) -> Option<&WgpuTransientTexture> {
        self.textures.get(&id)
    }
}
