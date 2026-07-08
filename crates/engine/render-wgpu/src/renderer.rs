use std::cmp::Ordering;
pub(crate) use std::collections::{BTreeMap, BTreeSet};
use std::fs;
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::SystemTime;

use amigo_assets::{PreparedAsset, PreparedAssetKind};
use amigo_core::AmigoResult;
pub(crate) use amigo_math::ColorRgba;
use amigo_math::{Transform2, Transform3, Vec2, Vec3};
use amigo_render_api::MaterialDrawCommand;
use amigo_render_api::MeshDrawCommand;
use amigo_render_api::Text3dDrawCommand;
use amigo_render_api::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_render_api::{PostFx2d, PostFx2dCacheKey};
use image::{GenericImageView, RgbaImage};
use wgpu::util::DeviceExt;

use crate::Renderable2dItem;
use crate::ui_overlay::{
    UiDrawPrimitive, UiOverlayDocument, UiViewportSize, build_ui_overlay_primitives,
};
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

    pub fn npr_stroke_stats_3d(&self) -> NprStrokeFrameStats3d {
        self.npr_stroke_stats_3d.clone()
    }

    pub fn offscreen_upload_stats(&self) -> crate::renderer::service::WgpuOffscreenUploadStats {
        self.offscreen_upload_stats
    }

    pub(crate) fn set_render_materials_last_summary(&mut self, summary: impl Into<String>) {
        self.render_materials_last_summary = summary.into();
    }

    pub fn frame_diagnostics(&self) -> &[amigo_render_api::RenderFrameDiagnostic] {
        &self.frame_diagnostics
    }

    pub(crate) fn clear_frame_diagnostics(&mut self) {
        self.frame_diagnostics.clear();
    }

    pub(crate) fn record_frame_diagnostic(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.frame_diagnostics
            .push(amigo_render_api::RenderFrameDiagnostic::new(code, message));
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
mod mesh_draw;
mod mesh_geometry;
mod npr;
mod particles;
mod pipelines;
mod render_types;
mod scene;
mod service;
mod shaders;
mod text;
mod text_3d;
mod vertices;
mod viewport;
mod world_2d;

use assets::*;
use buffers::*;
use core_pipelines::*;
use glyphs::*;
use math::*;
use mesh_draw::*;
pub(crate) use mesh_geometry::*;
use scene::*;
use text::*;
use text_3d::*;
use vertices::*;
use world_2d::*;

pub(crate) use cached_resources::*;
pub(crate) use lightmap2d::lit_particle_color;
pub(crate) use math::sprite_color;
pub use npr::NprStrokeFrameStats3d;
pub(crate) use npr::*;
pub(crate) use particles::append_particle_light_primitive_vertices;
pub(crate) use particles::append_particle_primitive_vertices;
pub(crate) use particles::color_batch_vertices;
pub(crate) use particles::particle_blend_mode;
pub(crate) use particles::particle_render_lights_from_renderables;
pub(crate) use render_types::*;
pub(crate) use service::{WgpuMaterialCandidate2d, collect_material_candidate_2d};
pub(crate) use viewport::*;
pub(crate) use world_2d::append_beacon_vfx_primitive_vertices;
pub(crate) use world_2d::append_text_2d_vertices;
pub(crate) use world_2d::append_textured_quad_debug_vertices;
pub(crate) use world_2d::append_tilemap_primitive_color_vertices;
pub(crate) use world_2d::{
    append_vector_primitive_vertices, vector_primitive_viewport_fit_transform,
};

pub use service::{
    WgpuEmergencyOverlayLevel, WgpuEmergencyOverlayLine, WgpuFrameRenderRequest,
    WgpuFrameRenderTarget, WgpuGameViewportPlacement, WgpuSceneRenderer, WgpuSurfaceRect,
    WgpuWorld2dRenderInput, WgpuWorld3dRenderInput,
};

#[cfg(test)]
mod tests;
