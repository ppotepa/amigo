use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_2d_layered_image::LayeredImageSceneService;
use amigo_2d_lighting::{GlobalLight2dSceneService, LightGroup2dCommand, LightMap2dSceneService};
use amigo_2d_lighting_beacon::BeaconLight2dDrawCommand;
use amigo_2d_particles::Particle2dDrawCommand;
use amigo_2d_post_fx::ScopedPostFx2dStack;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::TileMap2dSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_assets::AssetCatalog;
use amigo_render_api::{
    CameraCaptureInput2d, CameraDebugView2d, CameraOpticalCandidate2d, FrameCompositionPlan,
    FrameGraph, LightSource2dCommon,
};
use amigo_scene::SceneService;

use crate::{
    Renderable2dItem, UiOverlayDocument, WgpuOffscreenTarget, WgpuSurfaceState,
    WgpuVisualSourceFlags2d,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuEmergencyOverlayLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgpuEmergencyOverlayLine {
    pub level: WgpuEmergencyOverlayLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WgpuSurfaceRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl WgpuSurfaceRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WgpuGameViewportPlacement {
    pub surface_rect: WgpuSurfaceRect,
    pub logical_width: u32,
    pub logical_height: u32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

pub enum WgpuFrameRenderTarget<'a> {
    Surface(&'a mut WgpuSurfaceState),
    Offscreen(&'a mut WgpuOffscreenTarget),
}

impl WgpuFrameRenderTarget<'_> {
    pub fn width(&self) -> u32 {
        match self {
            Self::Surface(surface) => surface.config.width,
            Self::Offscreen(target) => target.width,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::Surface(surface) => surface.config.height,
            Self::Offscreen(target) => target.height,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Surface(surface) => surface.config.format,
            Self::Offscreen(target) => target.format,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        match self {
            Self::Surface(surface) => &surface.device,
            Self::Offscreen(target) => &target.device,
        }
    }

    pub fn queue(&self) -> &wgpu::Queue {
        match self {
            Self::Surface(surface) => &surface.queue,
            Self::Offscreen(target) => &target.queue,
        }
    }
}

pub struct WgpuFrameRenderRequest<'a> {
    pub target: WgpuFrameRenderTarget<'a>,
    pub scene: &'a SceneService,
    pub assets: &'a AssetCatalog,
    pub world_2d: WgpuWorld2dRenderInput<'a>,
    pub world_3d: WgpuWorld3dRenderInput<'a>,
    pub game_ui: &'a [UiOverlayDocument],
    pub debug_ui: &'a [UiOverlayDocument],
    pub post_fx_stacks: &'a [ScopedPostFx2dStack],
    pub active_camera_2d_entity: Option<&'a str>,
    pub camera_capture_input_2d: Option<&'a CameraCaptureInput2d>,
    pub visual_source_flags_2d: Option<&'a WgpuVisualSourceFlags2d>,
    pub camera_debug_view: CameraDebugView2d,
    pub emergency_overlay: &'a [WgpuEmergencyOverlayLine],
    pub composition_plan: &'a FrameCompositionPlan,
    pub frame_graph: &'a FrameGraph,
    pub game_viewport: Option<WgpuGameViewportPlacement>,
}

pub struct WgpuWorld2dRenderInput<'a> {
    pub renderables: &'a [Renderable2dItem],
    pub tilemaps: &'a TileMap2dSceneService,
    pub sprites: &'a SpriteSceneService,
    pub layered_images: &'a LayeredImageSceneService,
    pub depth_maps: &'a amigo_2d_depth_map::DepthMap2dSceneService,
    pub global_lights: &'a GlobalLight2dSceneService,
    pub lightmaps: &'a LightMap2dSceneService,
    pub text2d: &'a Text2dSceneService,
    pub vectors: &'a VectorSceneService,
    pub beacons: &'a [BeaconLight2dDrawCommand],
    pub light_sources: &'a [LightSource2dCommon],
    pub camera_optical_candidates: &'a [CameraOpticalCandidate2d],
    pub render_layers: &'a [RenderLayer2dCommand],
    pub light_routes: &'a [LightRoute2dCommand],
    pub light_groups: &'a [LightGroup2dCommand],
    pub particles: &'a [Particle2dDrawCommand],
}

pub struct WgpuWorld3dRenderInput<'a> {
    pub meshes: &'a [MeshDrawCommand],
    pub materials: &'a [MaterialDrawCommand],
    pub text3d: Option<&'a [Text3dDrawCommand]>,
}
