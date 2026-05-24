use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use amigo_math::{ColorRgba, Transform2, Vec2};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) struct ParticleRenderLight {
    pub(crate) position: Vec2,
    pub(crate) color: ColorRgba,
    pub(crate) radius: f32,
    pub(crate) intensity: f32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct LightMap2dImageData {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Arc<Vec<[f32; 4]>>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct LightMap2dLayer {
    pub(crate) image: LightMap2dImageData,
    pub(crate) opacity: f32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct LightMap2dSampler {
    pub(crate) id: String,
    pub(crate) transform: Transform2,
    pub(crate) size: Vec2,
    pub(crate) channels: BTreeMap<String, Vec<LightMap2dLayer>>,
}

pub(crate) struct CachedTextureResource {
    pub(crate) _texture: wgpu::Texture,
    pub(crate) _view: wgpu::TextureView,
    pub(crate) _sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) image_path: PathBuf,
    pub(crate) modified_at: Option<SystemTime>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) struct CachedLightMap2dImage {
    pub(crate) image_path: PathBuf,
    pub(crate) modified_at: Option<SystemTime>,
    pub(crate) data: LightMap2dImageData,
}

impl CachedTextureResource {
    pub(crate) fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub(crate) fn view(&self) -> &wgpu::TextureView {
        &self._view
    }
}
