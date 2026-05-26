use amigo_material_api::MaterialCandidateDecision2d;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_render_api::LightRoute2dCommand;
use amigo_render_api::{
    LightSource2dCommon, RenderAssetSource, RenderPrimitive2dKind, VisualSourceKind2d,
};
use std::collections::BTreeSet;

use crate::renderer::WgpuMaterialCandidate2d;
use crate::renderer::{
    ColorBatch, LightMap2dSampler, ParticleRenderLight, TextureBatch, Viewport,
};
use crate::{Renderable2dItem, WgpuOffscreenTarget, WgpuSceneRenderer};

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

    fn focus_sample_world_position(&self, _item: &Renderable2dItem) -> Option<Vec2> {
        None
    }

    fn append_visual_map_batches(
        &self,
        _ctx: &mut WgpuVisualMapAdapterContext<'_>,
        _item: &Renderable2dItem,
    ) -> bool {
        false
    }

    fn append_refractive_mask_batches(
        &self,
        _ctx: &mut WgpuRefractiveMaskAdapterContext<'_>,
        _item: &Renderable2dItem,
        _alpha: f32,
    ) -> WgpuRefractiveMaskAppendOutcome {
        WgpuRefractiveMaskAppendOutcome::none()
    }

    fn append_motion_batches(
        &self,
        ctx: &mut WgpuMotionAdapterContext<'_>,
        item: &Renderable2dItem,
    ) -> bool {
        let (Some(transform), Some(size)) = (
            item.primitive.proxy_quad_transform(),
            item.primitive.proxy_quad_size(),
        ) else {
            return false;
        };
        let key = item.source_id().as_str().to_owned();
        let current = transform.translation;
        let previous = ctx.previous_positions.get(&key).copied();
        ctx.current_positions.insert(key, current);
        append_motion_visual_quad(ctx, transform, size, motion_vector_color(previous, current, ctx.target_size));
        true
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

pub(crate) struct WgpuVisualMapAdapterContext<'a> {
    pub renderer: &'a mut WgpuSceneRenderer,
    pub texture_batches: &'a mut Vec<TextureBatch>,
    pub target: &'a WgpuOffscreenTarget,
    pub assets: &'a dyn RenderAssetSource,
    pub viewport: &'a Viewport,
    pub camera: Transform2,
    pub kind: VisualSourceKind2d,
}

pub(crate) struct WgpuRefractiveMaskAdapterContext<'a> {
    pub renderer: &'a mut WgpuSceneRenderer,
    pub texture_batches: &'a mut Vec<TextureBatch>,
    pub color_batches: &'a mut Vec<ColorBatch>,
    pub target: &'a WgpuOffscreenTarget,
    pub assets: &'a dyn RenderAssetSource,
    pub viewport: &'a Viewport,
    pub camera: Transform2,
}

pub(crate) struct WgpuMotionAdapterContext<'a> {
    pub color_batches: &'a mut Vec<ColorBatch>,
    pub viewport: &'a Viewport,
    pub camera: Transform2,
    pub target_size: (u32, u32),
    pub current_positions: &'a mut std::collections::BTreeMap<String, Vec2>,
    pub previous_positions: &'a std::collections::BTreeMap<String, Vec2>,
}

pub(crate) fn motion_vector_color(
    previous: Option<Vec2>,
    current: Vec2,
    target_size: (u32, u32),
) -> ColorRgba {
    let Some(previous) = previous else {
        return ColorRgba::new(0.5, 0.5, 0.0, 1.0);
    };
    let width = (target_size.0.max(1)) as f32;
    let height = (target_size.1.max(1)) as f32;
    let delta = Vec2::new(
        (current.x - previous.x) / width,
        (current.y - previous.y) / height,
    );
    let scale = 8.0;
    let x = (0.5 + delta.x * scale).clamp(0.0, 1.0);
    let y = (0.5 + delta.y * scale).clamp(0.0, 1.0);
    let magnitude = ((delta.x * delta.x + delta.y * delta.y).sqrt() * scale).clamp(0.0, 1.0);
    ColorRgba::new(x, y, magnitude, 1.0)
}

pub(crate) fn append_motion_visual_quad(
    ctx: &mut WgpuMotionAdapterContext<'_>,
    transform: Transform2,
    size: Vec2,
    color: ColorRgba,
) {
    crate::renderer::append_textured_quad_debug_vertices(
        crate::renderer::color_batch_vertices(
            ctx.color_batches,
            crate::renderer::particle_blend_mode(
                amigo_render_api::ParticleBlendMode2dPrimitive::Alpha,
            ),
        ),
        ctx.viewport,
        ctx.camera,
        &amigo_render_api::TexturedQuad2dPrimitive {
            texture: amigo_assets::AssetKey::new("generated://visual-source/quad"),
            size,
            transform,
            sheet: None,
            frame_index: 0,
            visual_maps: None,
            material: amigo_render_api::RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::TextureAlpha,
            ),
        },
        color,
    );
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WgpuRefractiveMaskAppendOutcome {
    pub appended: bool,
    pub generated_geometry: bool,
    pub source: &'static str,
}

impl WgpuRefractiveMaskAppendOutcome {
    pub fn none() -> Self {
        Self {
            appended: false,
            generated_geometry: false,
            source: "none",
        }
    }

    pub fn appended(source: &'static str, generated_geometry: bool) -> Self {
        Self {
            appended: true,
            generated_geometry,
            source,
        }
    }
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

    pub(crate) fn focus_sample_world_position(&self, item: &Renderable2dItem) -> Option<Vec2> {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind() == item.primitive_kind())
            .and_then(|adapter| adapter.focus_sample_world_position(item))
    }

    pub(crate) fn append_visual_map_batches(
        &self,
        ctx: &mut WgpuVisualMapAdapterContext<'_>,
        item: &Renderable2dItem,
    ) -> bool {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind() == item.primitive_kind())
            .is_some_and(|adapter| adapter.append_visual_map_batches(ctx, item))
    }

    pub(crate) fn append_refractive_mask_batches(
        &self,
        ctx: &mut WgpuRefractiveMaskAdapterContext<'_>,
        item: &Renderable2dItem,
        alpha: f32,
    ) -> WgpuRefractiveMaskAppendOutcome {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind() == item.primitive_kind())
            .map(|adapter| adapter.append_refractive_mask_batches(ctx, item, alpha))
            .unwrap_or_else(WgpuRefractiveMaskAppendOutcome::none)
    }

    pub(crate) fn append_motion_batches(
        &self,
        ctx: &mut WgpuMotionAdapterContext<'_>,
        item: &Renderable2dItem,
    ) -> bool {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind() == item.primitive_kind())
            .is_some_and(|adapter| adapter.append_motion_batches(ctx, item))
    }
}
