pub(crate) const RAIN_GLASS_OPTICAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

#[allow(dead_code)]
pub(crate) struct RainGlassRenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl RainGlassRenderTarget {
    fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            format,
            width,
            height,
        }
    }
}

pub(crate) struct RainGlassPingPongTarget {
    pub a: RainGlassRenderTarget,
    pub b: RainGlassRenderTarget,
    pub front_is_a: bool,
}

impl RainGlassPingPongTarget {
    fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        Self {
            a: RainGlassRenderTarget::new(device, width, height, format, &format!("{label}-a")),
            b: RainGlassRenderTarget::new(device, width, height, format, &format!("{label}-b")),
            front_is_a: true,
        }
    }

    pub(crate) fn front(&self) -> &RainGlassRenderTarget {
        if self.front_is_a { &self.a } else { &self.b }
    }

    pub(crate) fn back(&self) -> &RainGlassRenderTarget {
        if self.front_is_a { &self.b } else { &self.a }
    }

    pub(crate) fn swap(&mut self) {
        self.front_is_a = !self.front_is_a;
    }
}

pub(crate) struct RainGlassResources {
    pub width: u32,
    pub height: u32,
    pub raindrop_map: RainGlassRenderTarget,
    pub live_trail_map: RainGlassRenderTarget,
    pub droplet_map: RainGlassPingPongTarget,
    pub mist_map: RainGlassPingPongTarget,
    pub blurred_scene_a: RainGlassRenderTarget,
    pub blurred_scene_b: RainGlassRenderTarget,
}

impl RainGlassResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scene_format: wgpu::TextureFormat,
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            width,
            height,
            raindrop_map: RainGlassRenderTarget::new(
                device,
                width,
                height,
                RAIN_GLASS_OPTICAL_FORMAT,
                "amigo-rain-glass-raindrop-map",
            ),
            live_trail_map: RainGlassRenderTarget::new(
                device,
                width,
                height,
                RAIN_GLASS_OPTICAL_FORMAT,
                "amigo-rain-glass-live-trail-map",
            ),
            droplet_map: RainGlassPingPongTarget::new(
                device,
                width,
                height,
                RAIN_GLASS_OPTICAL_FORMAT,
                "amigo-rain-glass-droplet-map",
            ),
            mist_map: RainGlassPingPongTarget::new(
                device,
                width,
                height,
                RAIN_GLASS_OPTICAL_FORMAT,
                "amigo-rain-glass-mist-map",
            ),
            blurred_scene_a: RainGlassRenderTarget::new(
                device,
                width,
                height,
                scene_format,
                "amigo-rain-glass-blur-a",
            ),
            blurred_scene_b: RainGlassRenderTarget::new(
                device,
                width,
                height,
                scene_format,
                "amigo-rain-glass-blur-b",
            ),
        }
    }

    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scene_format: wgpu::TextureFormat,
    ) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width
            && self.height == height
            && self.blurred_scene_a.format == scene_format
        {
            return false;
        }
        *self = Self::new(device, width, height, scene_format);
        true
    }
}
