pub(crate) const RAIN_GLASS_OPTICAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const RAIN_GLASS_BLUR_PYRAMID_LEVELS: usize = 5;

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
    pub streak_map: RainGlassPingPongTarget,
    pub droplet_map: RainGlassPingPongTarget,
    pub mist_map: RainGlassPingPongTarget,
    pub blurred_scene_a: RainGlassRenderTarget,
    pub blurred_scene_b: RainGlassRenderTarget,
    pub blur_pyramid: Vec<RainGlassPingPongTarget>,
}

impl RainGlassResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scene_format: wgpu::TextureFormat,
        quality_scale: f32,
    ) -> Self {
        let quality_scale = quality_scale.clamp(0.35, 1.0);
        let width = ((width.max(1) as f32 * quality_scale).round() as u32).max(1);
        let height = ((height.max(1) as f32 * quality_scale).round() as u32).max(1);
        let blur_pyramid = (1..=RAIN_GLASS_BLUR_PYRAMID_LEVELS)
            .map(|level| {
                let scale = 1u32 << level;
                RainGlassPingPongTarget::new(
                    device,
                    (width / scale).max(1),
                    (height / scale).max(1),
                    scene_format,
                    &format!("amigo-rain-glass-blur-pyramid-{level}"),
                )
            })
            .collect();

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
            streak_map: RainGlassPingPongTarget::new(
                device,
                width,
                height,
                RAIN_GLASS_OPTICAL_FORMAT,
                "amigo-rain-glass-streak-map",
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
            blur_pyramid,
        }
    }

    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scene_format: wgpu::TextureFormat,
        quality_scale: f32,
    ) -> bool {
        let quality_scale = quality_scale.clamp(0.35, 1.0);
        let width = ((width.max(1) as f32 * quality_scale).round() as u32).max(1);
        let height = ((height.max(1) as f32 * quality_scale).round() as u32).max(1);
        if self.width == width
            && self.height == height
            && self.blurred_scene_a.format == scene_format
        {
            return false;
        }
        *self = Self::new(device, width, height, scene_format, 1.0);
        true
    }
}
