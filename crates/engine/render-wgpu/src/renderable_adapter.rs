use amigo_material_api::MaterialCandidateDecision2d;
use amigo_math::Transform2;
use amigo_render_api::LightRoute2dCommand;
use amigo_render_api::{LightSource2dCommon, RenderAssetSource, RenderPrimitive2dKind};
use std::collections::BTreeSet;

use crate::renderer::WgpuMaterialCandidate2d;
use crate::renderer::{ColorBatch, LightMap2dSampler, ParticleRenderLight, TextureBatch, Viewport};
use crate::{Renderable2dItem, WgpuSceneRenderer};

pub(crate) trait WgpuRenderable2dAdapter: Send + Sync {
    fn kind(&self) -> RenderPrimitive2dKind;

    fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &Renderable2dItem,
    ) -> bool {
        self.append_texture_batches(ctx, item) || self.append_color_batches(ctx, item)
    }

    fn append_texture_batches(
        &self,
        _ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        _item: &Renderable2dItem,
    ) -> bool {
        false
    }

    fn append_color_batches(
        &self,
        _ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        _item: &Renderable2dItem,
    ) -> bool {
        false
    }
}

pub(crate) struct WgpuRenderable2dAdapterContext<'a> {
    pub renderer: &'a mut WgpuSceneRenderer,
    pub texture_batches: &'a mut Vec<TextureBatch>,
    pub color_batches: &'a mut Vec<ColorBatch>,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub assets: &'a dyn RenderAssetSource,
    pub viewport: &'a Viewport,
    pub layer_camera: Transform2,
    pub layer_opacity: f32,
    pub transform: Transform2,
    pub material_candidates: &'a mut Vec<WgpuMaterialCandidate2d>,
    pub material_decisions: &'a mut Vec<MaterialCandidateDecision2d>,
    pub included_layered_image_parts: Option<&'a BTreeSet<String>>,
    pub excluded_layered_image_parts: Option<&'a BTreeSet<String>>,
    pub include_base_layered_image: bool,
    pub particle_lights: &'a [ParticleRenderLight],
    pub lightmap_samplers: &'a [LightMap2dSampler],
    pub light_sources: &'a [LightSource2dCommon],
    pub light_routes: &'a [LightRoute2dCommand],
}

#[derive(Default)]
pub(crate) struct WgpuRenderable2dAdapterRegistry {
    adapters: Vec<Box<dyn WgpuRenderable2dAdapter>>,
}

impl WgpuRenderable2dAdapterRegistry {
    pub(crate) fn register<A>(&mut self, adapter: A)
    where
        A: WgpuRenderable2dAdapter + 'static,
    {
        self.adapters.push(Box::new(adapter));
    }

    pub(crate) fn supports_kind(&self, kind: RenderPrimitive2dKind) -> bool {
        self.adapters.iter().any(|adapter| adapter.kind() == kind)
    }

    pub(crate) fn append_batches(
        &self,
        ctx: &mut WgpuRenderable2dAdapterContext<'_>,
        item: &Renderable2dItem,
    ) -> bool {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind() == item.primitive_kind())
            .is_some_and(|adapter| adapter.append_batches(ctx, item))
    }
}
