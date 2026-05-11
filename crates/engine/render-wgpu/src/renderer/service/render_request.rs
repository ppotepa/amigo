use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_2d_layered_image::LayeredImageSceneService;
use amigo_2d_lighting::{GlobalLight2dSceneService, LightGroup2dCommand, LightMap2dSceneService};
use amigo_2d_particles::Particle2dDrawCommand;
use amigo_2d_post_fx::PostFx2dStack;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::TileMap2dSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_assets::AssetCatalog;
use amigo_render_api::{FrameCompositionPlan, FrameGraph};
use amigo_scene::SceneService;

use crate::{UiOverlayDocument, WgpuSurfaceState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuFrameGraphExecutionMode {
    LegacyComposite,
    SplitPassExperimental,
}

pub struct WgpuFrameRenderRequest<'a> {
    pub surface: &'a mut WgpuSurfaceState,
    pub scene: &'a SceneService,
    pub assets: &'a AssetCatalog,
    pub world_2d: WgpuWorld2dRenderInput<'a>,
    pub world_3d: WgpuWorld3dRenderInput<'a>,
    pub game_ui: &'a [UiOverlayDocument],
    pub debug_ui: &'a [UiOverlayDocument],
    pub post_fx_stack: Option<&'a PostFx2dStack>,
    pub composition_plan: &'a FrameCompositionPlan,
    pub frame_graph: &'a FrameGraph,
    pub execution_mode: WgpuFrameGraphExecutionMode,
}

pub struct WgpuWorld2dRenderInput<'a> {
    pub tilemaps: &'a TileMap2dSceneService,
    pub sprites: &'a SpriteSceneService,
    pub layered_images: &'a LayeredImageSceneService,
    pub global_lights: &'a GlobalLight2dSceneService,
    pub lightmaps: &'a LightMap2dSceneService,
    pub text2d: &'a Text2dSceneService,
    pub vectors: &'a VectorSceneService,
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
