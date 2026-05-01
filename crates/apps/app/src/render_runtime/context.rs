use amigo_2d_particles::Particle2dDrawCommand;
use amigo_2d_sprite::SpriteDrawCommand;
use amigo_2d_text::Text2dDrawCommand;
use amigo_2d_tilemap::TileMap2dDrawCommand;
use amigo_2d_vector::VectorShape2dDrawCommand;
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_render_api::RenderFrameExtractorRegistry;
use amigo_render_wgpu::UiOverlayDocument;
use amigo_scene::SceneService;
use amigo_ui::{UiSceneService, UiStateService, UiThemeService};

use amigo_2d_particles::Particle2dSceneService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::TileMap2dSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_3d_material::MaterialSceneService;
use amigo_3d_mesh::MeshSceneService;
use amigo_3d_text::Text3dSceneService;

pub(crate) struct AppRenderExtractContext<'a> {
    pub(crate) scene_service: &'a SceneService,
    pub(crate) tilemap_scene_service: &'a TileMap2dSceneService,
    pub(crate) sprite_scene_service: &'a SpriteSceneService,
    pub(crate) text2d_scene_service: &'a Text2dSceneService,
    pub(crate) vector_scene_service: &'a VectorSceneService,
    pub(crate) particle2d_scene_service: &'a Particle2dSceneService,
    pub(crate) mesh_scene_service: &'a MeshSceneService,
    pub(crate) material_scene_service: &'a MaterialSceneService,
    pub(crate) text3d_scene_service: &'a Text3dSceneService,
    pub(crate) ui_scene_service: &'a UiSceneService,
    pub(crate) ui_state_service: &'a UiStateService,
    pub(crate) ui_theme_service: &'a UiThemeService,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct AppRenderFramePacket {
    world_2d_tilemaps: Vec<TileMap2dDrawCommand>,
    world_2d_sprites: Vec<SpriteDrawCommand>,
    world_2d_text: Vec<Text2dDrawCommand>,
    world_2d_vectors: Vec<VectorShape2dDrawCommand>,
    world_2d_particles: Vec<Particle2dDrawCommand>,
    world_3d_meshes: Vec<MeshDrawCommand>,
    world_3d_materials: Vec<MaterialDrawCommand>,
    world_3d_text: Vec<Text3dDrawCommand>,
    overlay: Vec<UiOverlayDocument>,
}

impl AppRenderFramePacket {
    pub(crate) fn push_world_2d_tilemap(&mut self, command: TileMap2dDrawCommand) {
        self.world_2d_tilemaps.push(command);
    }

    pub(crate) fn push_world_2d_sprite(&mut self, command: SpriteDrawCommand) {
        self.world_2d_sprites.push(command);
    }

    pub(crate) fn push_world_2d_vector(&mut self, command: VectorShape2dDrawCommand) {
        self.world_2d_vectors.push(command);
    }

    pub(crate) fn push_world_2d_text(&mut self, command: Text2dDrawCommand) {
        self.world_2d_text.push(command);
    }

    pub(crate) fn push_world_2d_particle(&mut self, command: Particle2dDrawCommand) {
        self.world_2d_particles.push(command);
    }

    pub(crate) fn push_world_3d_mesh(&mut self, command: MeshDrawCommand) {
        self.world_3d_meshes.push(command);
    }

    pub(crate) fn push_world_3d_material(&mut self, command: MaterialDrawCommand) {
        self.world_3d_materials.push(command);
    }

    pub(crate) fn push_world_3d_text(&mut self, command: Text3dDrawCommand) {
        self.world_3d_text.push(command);
    }

    pub(crate) fn extend_overlay<I>(&mut self, overlay: I)
    where
        I: IntoIterator<Item = UiOverlayDocument>,
    {
        self.overlay.extend(overlay);
    }

    pub(crate) fn world_2d_vectors(&self) -> &[VectorShape2dDrawCommand] {
        &self.world_2d_vectors
    }

    pub(crate) fn world_2d_sprites(&self) -> &[SpriteDrawCommand] {
        &self.world_2d_sprites
    }

    pub(crate) fn world_2d_tilemaps(&self) -> &[TileMap2dDrawCommand] {
        &self.world_2d_tilemaps
    }

    pub(crate) fn world_2d_text(&self) -> &[Text2dDrawCommand] {
        &self.world_2d_text
    }

    pub(crate) fn world_2d_particles(&self) -> &[Particle2dDrawCommand] {
        &self.world_2d_particles
    }

    pub(crate) fn world_3d_meshes(&self) -> &[MeshDrawCommand] {
        &self.world_3d_meshes
    }

    pub(crate) fn world_3d_materials(&self) -> &[MaterialDrawCommand] {
        &self.world_3d_materials
    }

    pub(crate) fn world_3d_text(&self) -> &[Text3dDrawCommand] {
        &self.world_3d_text
    }

    pub(crate) fn overlay(&self) -> &[UiOverlayDocument] {
        &self.overlay
    }
}

pub(crate) type AppRenderExtractorRegistry<'a> =
    RenderFrameExtractorRegistry<AppRenderExtractContext<'a>, AppRenderFramePacket>;
