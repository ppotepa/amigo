use std::cmp::Ordering;
use std::fs;
pub(crate) use std::collections::{BTreeMap, BTreeSet};
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::SystemTime;

use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
pub(crate) use amigo_math::ColorRgba;
use amigo_math::{Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::{
    LayeredImageAssetSource, PostFx2d, PostFx2dCacheKey,
};
use amigo_scene::SceneService;
use image::{GenericImageView, RgbaImage};
use wgpu::util::DeviceExt;

use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiViewportSize, build_ui_overlay_primitives,
};
use crate::Renderable2dItem;
use crate::{WgpuOffscreenTarget, WgpuSurfaceState};

impl WgpuSceneRenderer {
    pub fn plate_relight_last_summary(&self) -> &str {
        &self.plate_relight_last_summary
    }

    pub(crate) fn set_plate_relight_last_summary(&mut self, summary: impl Into<String>) {
        self.plate_relight_last_summary = summary.into();
    }

    pub fn render_materials_last_summary(&self) -> &str {
        &self.render_materials_last_summary
    }

    pub(crate) fn set_render_materials_last_summary(&mut self, summary: impl Into<String>) {
        self.render_materials_last_summary = summary.into();
    }
}

mod assets;
mod buffers;
mod cached_resources;
mod core_pipelines;
mod glyphs;
mod graph;
mod lightmap2d;
mod math;
mod particles;
mod pipelines;
mod render_types;
mod scene;
mod service;
mod shaders;
mod text;
mod vertices;
mod viewport;
mod world_2d;
mod world_3d;

use assets::*;
use buffers::*;
use core_pipelines::*;
use glyphs::*;
use math::*;
use scene::*;
use text::*;
use vertices::*;
use world_2d::*;
use world_3d::*;

pub(crate) use cached_resources::*;
pub(crate) use particles::color_batch_vertices;
pub(crate) use particles::append_particle_light_primitive_vertices;
pub(crate) use particles::append_particle_primitive_vertices;
pub(crate) use particles::particle_blend_mode;
pub(crate) use particles::particle_render_lights_from_renderables;
pub(crate) use math::sprite_color;
pub(crate) use render_types::*;
pub(crate) use service::{collect_material_candidate_2d, WgpuMaterialCandidate2d};
pub(crate) use world_2d::append_textured_quad_debug_vertices;
pub(crate) use world_2d::append_tilemap_primitive_fallback_vertices;
pub(crate) use world_2d::append_beacon_vfx_primitive_vertices;
pub(crate) use world_2d::append_vector_primitive_vertices;
pub(crate) use viewport::*;

pub use service::{
    WgpuEmergencyOverlayLevel, WgpuEmergencyOverlayLine, WgpuFrameRenderRequest,
    WgpuFrameRenderTarget, WgpuGameViewportPlacement, WgpuSceneRenderer, WgpuSurfaceRect,
    WgpuWorld2dRenderInput, WgpuWorld3dRenderInput,
};

#[cfg(test)]
mod tests;
