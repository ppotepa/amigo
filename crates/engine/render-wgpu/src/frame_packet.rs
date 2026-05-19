use amigo_2d_composition::{LightRoute2dCommand, RenderLayer2dCommand};
use amigo_focus_depth_plugin::{DepthAuxMap2dDrawCommand, DepthMap2dDrawCommand};
use amigo_layered_image_2d_plugin::LayeredImageDrawCommand;
use amigo_light_2d_plugin::{GlobalLight2dCommand, LightGroup2dCommand, LightMap2dSourceCommand};
use amigo_beacon_light_2d_plugin::{Beacon2dRenderOutput, BeaconLight2dDrawCommand};
use amigo_particles_2d_plugin::Particle2dDrawCommand;
use amigo_2d_post_fx::ScopedPostFx2dStack;
use amigo_sprite_2d_plugin::SpriteDrawCommand;
use amigo_text_2d_plugin::Text2dDrawCommand;
use amigo_tilemap_2d_plugin::TileMap2dDrawCommand;
use amigo_vector_2d_plugin::VectorShape2dDrawCommand;
use amigo_3d_material::MaterialDrawCommand;
use amigo_3d_mesh::MeshDrawCommand;
use amigo_3d_text::Text3dDrawCommand;
use amigo_camera_optics_plugin::api::CameraOpticalCandidate2d;
use amigo_render_api::{
    CameraCaptureInput2d, CameraDebugView2d, LightSource2dCommon, RenderSpace2d,
    Renderable2dCommon, Renderable2dKind,
};

use crate::UiOverlayDocument;

#[derive(Debug, Clone)]
pub struct Renderable2dItem {
    pub common: Renderable2dCommon,
    pub payload: Renderable2dPayload,
}

#[derive(Debug, Clone)]
pub enum Renderable2dPayload {
    TileMap(TileMap2dDrawCommand),
    LayeredImage(LayeredImageDrawCommand),
    Vector(VectorShape2dDrawCommand),
    Beacon(BeaconLight2dDrawCommand),
    Sprite(SpriteDrawCommand),
    Text(Text2dDrawCommand),
    Particle(Particle2dDrawCommand),
}

impl Renderable2dItem {
    pub fn render_layer(&self) -> &str {
        &self.common.render_layer
    }

    pub fn z_index(&self) -> f32 {
        self.common.z_index
    }

    pub fn owner_entity(&self) -> &str {
        &self.common.owner_entity
    }

    pub fn component_kind(&self) -> &str {
        &self.common.component_kind
    }

    pub fn render_space(&self) -> RenderSpace2d {
        self.common.render_space
    }

    pub fn payload_kind(&self) -> &'static str {
        self.common.kind.as_str()
    }

    pub fn uses_camera_pipeline(&self) -> bool {
        self.common.uses_camera_pipeline()
    }
}

fn renderable_2d_common(
    owner_entity: String,
    component_kind: &'static str,
    render_layer: String,
    z_index: f32,
    kind: Renderable2dKind,
) -> Renderable2dCommon {
    Renderable2dCommon {
        owner_entity,
        component_kind: component_kind.to_owned(),
        render_space: RenderSpace2d::World,
        render_layer,
        z_index,
        kind,
    }
}

pub fn supported_renderable_2d_component_kinds() -> &'static [&'static str] {
    &[
        "TileMap2D",
        "LayeredImage2D",
        "VectorShape2D",
        "BeaconLight2D",
        "Sprite2D",
        "Text2D",
        "ParticleEmitter2D",
    ]
}

#[derive(Debug, Clone, Default)]
pub struct WgpuVisualSourceFlags2d {
    pub layer_mask_generated: bool,
    pub layer_roles_generated: bool,
    pub scene_normal_generated: bool,
    pub scene_wetness_generated: bool,
    pub scene_highlight_generated: bool,
    pub scene_emissive_generated: bool,
    pub scene_motion_generated: bool,
}

#[derive(Debug, Default, Clone)]
pub struct WgpuRenderFramePacket {
    world_2d_tilemaps: Vec<TileMap2dDrawCommand>,
    world_2d_sprites: Vec<SpriteDrawCommand>,
    world_2d_layered_images: Vec<LayeredImageDrawCommand>,
    world_2d_depth_maps: Vec<DepthMap2dDrawCommand>,
    world_2d_depth_aux_maps: Vec<DepthAuxMap2dDrawCommand>,
    world_2d_render_layers: Vec<RenderLayer2dCommand>,
    world_2d_light_routes: Vec<LightRoute2dCommand>,
    world_2d_global_lights: Vec<GlobalLight2dCommand>,
    world_2d_lightmaps: Vec<LightMap2dSourceCommand>,
    world_2d_light_groups: Vec<LightGroup2dCommand>,
    light_sources_2d: Vec<LightSource2dCommon>,
    camera_optical_candidates_2d: Vec<CameraOpticalCandidate2d>,
    world_2d_text: Vec<Text2dDrawCommand>,
    world_2d_vectors: Vec<VectorShape2dDrawCommand>,
    world_2d_beacons: Vec<BeaconLight2dDrawCommand>,
    world_2d_particles: Vec<Particle2dDrawCommand>,
    renderables_2d: Vec<Renderable2dItem>,
    world_3d_meshes: Vec<MeshDrawCommand>,
    world_3d_materials: Vec<MaterialDrawCommand>,
    world_3d_text: Vec<Text3dDrawCommand>,
    game_ui_overlay: Vec<UiOverlayDocument>,
    debug_overlay: Vec<UiOverlayDocument>,
    post_fx_stacks: Vec<ScopedPostFx2dStack>,
    active_camera_2d_entity: Option<String>,
    camera_capture_input_2d: Option<CameraCaptureInput2d>,
    camera_debug_view_2d: Option<CameraDebugView2d>,
    visual_source_flags_2d: WgpuVisualSourceFlags2d,
}

impl WgpuRenderFramePacket {
    pub fn push_world_2d_tilemap(&mut self, command: TileMap2dDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "TileMap2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::TileMap,
            ),
            payload: Renderable2dPayload::TileMap(command.clone()),
        });
        self.world_2d_tilemaps.push(command);
    }

    pub fn push_world_2d_sprite(&mut self, command: SpriteDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "Sprite2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::Sprite,
            ),
            payload: Renderable2dPayload::Sprite(command.clone()),
        });
        self.world_2d_sprites.push(command);
    }

    pub fn push_world_2d_layered_image(&mut self, command: LayeredImageDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "LayeredImage2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::LayeredImage,
            ),
            payload: Renderable2dPayload::LayeredImage(command.clone()),
        });
        self.world_2d_layered_images.push(command);
    }

    pub fn push_world_2d_depth_map(&mut self, command: DepthMap2dDrawCommand) {
        self.world_2d_depth_maps.push(command);
    }

    pub fn push_world_2d_depth_aux_map(&mut self, command: DepthAuxMap2dDrawCommand) {
        self.world_2d_depth_aux_maps.push(command);
    }

    pub fn push_world_2d_render_layer(&mut self, command: RenderLayer2dCommand) {
        self.world_2d_render_layers.push(command);
    }

    pub fn push_world_2d_light_route(&mut self, command: LightRoute2dCommand) {
        self.world_2d_light_routes.push(command);
    }

    pub fn push_world_2d_global_light(&mut self, command: GlobalLight2dCommand) {
        self.world_2d_global_lights.push(command);
    }

    pub fn push_world_2d_lightmap(&mut self, command: LightMap2dSourceCommand) {
        self.world_2d_lightmaps.push(command);
    }

    pub fn push_world_2d_light_group(&mut self, command: LightGroup2dCommand) {
        self.world_2d_light_groups.push(command);
    }

    pub fn push_world_2d_vector(&mut self, command: VectorShape2dDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "VectorShape2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::Vector,
            ),
            payload: Renderable2dPayload::Vector(command.clone()),
        });
        self.world_2d_vectors.push(command);
    }

    pub fn push_world_2d_beacon(&mut self, command: BeaconLight2dDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "BeaconLight2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::Beacon,
            ),
            payload: Renderable2dPayload::Beacon(command.clone()),
        });
        self.world_2d_beacons.push(command);
    }

    pub fn push_world_2d_text(&mut self, command: Text2dDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.entity_name.clone(),
                "Text2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::Text,
            ),
            payload: Renderable2dPayload::Text(command.clone()),
        });
        self.world_2d_text.push(command);
    }

    pub fn push_world_2d_particle(&mut self, command: Particle2dDrawCommand) {
        self.push_renderable_2d(Renderable2dItem {
            common: renderable_2d_common(
                command.emitter_entity_name.clone(),
                "ParticleEmitter2D",
                command.render_layer.clone(),
                command.z_index,
                Renderable2dKind::Particle,
            ),
            payload: Renderable2dPayload::Particle(command.clone()),
        });
        self.world_2d_particles.push(command);
    }

    pub fn push_renderable_2d(&mut self, item: Renderable2dItem) {
        self.renderables_2d.push(item);
    }

    pub fn push_world_3d_mesh(&mut self, command: MeshDrawCommand) {
        self.world_3d_meshes.push(command);
    }

    pub fn push_world_3d_material(&mut self, command: MaterialDrawCommand) {
        self.world_3d_materials.push(command);
    }

    pub fn push_world_3d_text(&mut self, command: Text3dDrawCommand) {
        self.world_3d_text.push(command);
    }

    pub fn push_game_ui_overlay(&mut self, overlay: UiOverlayDocument) {
        self.game_ui_overlay.push(overlay);
    }

    pub fn extend_game_ui_overlay<I>(&mut self, overlay: I)
    where
        I: IntoIterator<Item = UiOverlayDocument>,
    {
        self.game_ui_overlay.extend(overlay);
    }

    pub fn push_debug_overlay(&mut self, overlay: UiOverlayDocument) {
        self.debug_overlay.push(overlay);
    }

    pub fn extend_debug_overlay<I>(&mut self, overlay: I)
    where
        I: IntoIterator<Item = UiOverlayDocument>,
    {
        self.debug_overlay.extend(overlay);
    }

    pub fn set_post_fx_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>) {
        self.post_fx_stacks = stacks;
    }

    pub fn set_active_camera_2d_entity(&mut self, entity_name: Option<String>) {
        self.active_camera_2d_entity = entity_name;
    }

    pub fn set_camera_capture_input_2d(&mut self, input: CameraCaptureInput2d) {
        self.camera_capture_input_2d = Some(input);
    }

    pub fn set_light_sources_2d(&mut self, sources: Vec<LightSource2dCommon>) {
        self.light_sources_2d = sources;
    }

    pub fn set_camera_optical_candidates_2d(&mut self, candidates: Vec<CameraOpticalCandidate2d>) {
        self.camera_optical_candidates_2d = candidates;
    }

    pub fn set_camera_debug_view_2d(&mut self, debug_view: CameraDebugView2d) {
        self.camera_debug_view_2d = Some(debug_view);
    }

    pub fn set_visual_source_flags_2d(&mut self, flags: WgpuVisualSourceFlags2d) {
        self.visual_source_flags_2d = flags;
    }

    pub fn clear_debug_overlay(&mut self) {
        self.debug_overlay.clear();
    }

    pub fn clear_game_ui_overlay(&mut self) {
        self.game_ui_overlay.clear();
    }

    pub fn clear_world_content(&mut self) {
        self.world_2d_tilemaps.clear();
        self.world_2d_sprites.clear();
        self.world_2d_layered_images.clear();
        self.world_2d_depth_maps.clear();
        self.world_2d_depth_aux_maps.clear();
        self.world_2d_render_layers.clear();
        self.world_2d_light_routes.clear();
        self.world_2d_global_lights.clear();
        self.world_2d_lightmaps.clear();
        self.world_2d_light_groups.clear();
        self.light_sources_2d.clear();
        self.camera_optical_candidates_2d.clear();
        self.world_2d_text.clear();
        self.world_2d_vectors.clear();
        self.world_2d_beacons.clear();
        self.world_2d_particles.clear();
        self.renderables_2d.clear();
        self.world_3d_meshes.clear();
        self.world_3d_materials.clear();
        self.world_3d_text.clear();
        self.post_fx_stacks.clear();
        self.active_camera_2d_entity = None;
        self.camera_capture_input_2d = None;
        self.camera_debug_view_2d = None;
        self.visual_source_flags_2d = WgpuVisualSourceFlags2d::default();
    }

    pub fn world_2d_vectors(&self) -> &[VectorShape2dDrawCommand] {
        &self.world_2d_vectors
    }

    pub fn world_2d_beacons(&self) -> &[BeaconLight2dDrawCommand] {
        &self.world_2d_beacons
    }

    pub fn world_2d_sprites(&self) -> &[SpriteDrawCommand] {
        &self.world_2d_sprites
    }

    pub fn world_2d_layered_images(&self) -> &[LayeredImageDrawCommand] {
        &self.world_2d_layered_images
    }

    pub fn world_2d_depth_maps(&self) -> &[DepthMap2dDrawCommand] {
        &self.world_2d_depth_maps
    }

    pub fn world_2d_depth_aux_maps(&self) -> &[DepthAuxMap2dDrawCommand] {
        &self.world_2d_depth_aux_maps
    }

    pub fn world_2d_render_layers(&self) -> &[RenderLayer2dCommand] {
        &self.world_2d_render_layers
    }

    pub fn world_2d_light_routes(&self) -> &[LightRoute2dCommand] {
        &self.world_2d_light_routes
    }

    pub fn world_2d_global_lights(&self) -> &[GlobalLight2dCommand] {
        &self.world_2d_global_lights
    }

    pub fn world_2d_lightmaps(&self) -> &[LightMap2dSourceCommand] {
        &self.world_2d_lightmaps
    }

    pub fn world_2d_light_groups(&self) -> &[LightGroup2dCommand] {
        &self.world_2d_light_groups
    }

    pub fn world_2d_light_sources(&self) -> &[LightSource2dCommon] {
        &self.light_sources_2d
    }

    pub fn camera_optical_candidates_2d(&self) -> &[CameraOpticalCandidate2d] {
        &self.camera_optical_candidates_2d
    }

    pub fn world_2d_tilemaps(&self) -> &[TileMap2dDrawCommand] {
        &self.world_2d_tilemaps
    }

    pub fn world_2d_text(&self) -> &[Text2dDrawCommand] {
        &self.world_2d_text
    }

    pub fn world_2d_particles(&self) -> &[Particle2dDrawCommand] {
        &self.world_2d_particles
    }

    /// Unified world/screen/debug renderable items for the 2D visual pipeline.
    /// The world color pass uses this list as its source of truth. Per-type
    /// command lists remain for diagnostics, material/source extraction, and
    /// domain-specific systems.
    pub fn renderables_2d(&self) -> &[Renderable2dItem] {
        &self.renderables_2d
    }

    pub fn world_3d_meshes(&self) -> &[MeshDrawCommand] {
        &self.world_3d_meshes
    }

    pub fn world_3d_materials(&self) -> &[MaterialDrawCommand] {
        &self.world_3d_materials
    }

    pub fn world_3d_text(&self) -> &[Text3dDrawCommand] {
        &self.world_3d_text
    }

    pub fn game_ui_overlay(&self) -> &[UiOverlayDocument] {
        &self.game_ui_overlay
    }

    pub fn debug_overlay(&self) -> &[UiOverlayDocument] {
        &self.debug_overlay
    }

    pub fn all_overlay_count(&self) -> usize {
        self.game_ui_overlay.len() + self.debug_overlay.len()
    }

    pub fn has_world_2d(&self) -> bool {
        !self.world_2d_tilemaps.is_empty()
            || !self.world_2d_sprites.is_empty()
            || !self.world_2d_layered_images.is_empty()
            || !self.world_2d_depth_maps.is_empty()
            || !self.world_2d_depth_aux_maps.is_empty()
            || !self.world_2d_render_layers.is_empty()
            || !self.world_2d_light_routes.is_empty()
            || !self.world_2d_global_lights.is_empty()
            || !self.world_2d_lightmaps.is_empty()
            || !self.world_2d_light_groups.is_empty()
            || !self.world_2d_vectors.is_empty()
            || !self.world_2d_beacons.is_empty()
            || !self.world_2d_text.is_empty()
            || !self.world_2d_particles.is_empty()
    }

    pub fn has_world_3d(&self) -> bool {
        !self.world_3d_meshes.is_empty()
            || !self.world_3d_materials.is_empty()
            || !self.world_3d_text.is_empty()
    }

    pub fn post_fx_stacks(&self) -> &[ScopedPostFx2dStack] {
        &self.post_fx_stacks
    }

    pub fn active_camera_2d_entity(&self) -> Option<&str> {
        self.active_camera_2d_entity.as_deref()
    }

    pub fn camera_capture_input_2d(&self) -> Option<&CameraCaptureInput2d> {
        self.camera_capture_input_2d.as_ref()
    }

    pub fn camera_debug_view_2d(&self) -> Option<CameraDebugView2d> {
        self.camera_debug_view_2d
    }

    pub fn visual_source_flags_2d(&self) -> &WgpuVisualSourceFlags2d {
        &self.visual_source_flags_2d
    }
}

impl amigo_sprite_2d_plugin::Sprite2dRenderOutput for WgpuRenderFramePacket {
    fn push_sprite2d_render_command(&mut self, command: SpriteDrawCommand) {
        self.push_world_2d_sprite(command);
    }
}

impl amigo_tilemap_2d_plugin::TileMap2dRenderOutput for WgpuRenderFramePacket {
    fn push_tilemap2d_render_command(&mut self, command: TileMap2dDrawCommand) {
        self.push_world_2d_tilemap(command);
    }
}

impl amigo_layered_image_2d_plugin::LayeredImage2dRenderOutput for WgpuRenderFramePacket {
    fn push_layered_image2d_render_command(&mut self, command: LayeredImageDrawCommand) {
        self.push_world_2d_layered_image(command);
    }
}

impl amigo_focus_depth_plugin::DepthMap2dRenderOutput for WgpuRenderFramePacket {
    fn push_depth_map2d_render_command(&mut self, command: DepthMap2dDrawCommand) {
        self.push_world_2d_depth_map(command);
    }

    fn push_depth_aux_map2d_render_command(&mut self, command: DepthAuxMap2dDrawCommand) {
        self.push_world_2d_depth_aux_map(command);
    }
}

impl amigo_vector_2d_plugin::Vector2dRenderOutput for WgpuRenderFramePacket {
    fn push_vector2d_render_command(&mut self, command: VectorShape2dDrawCommand) {
        self.push_world_2d_vector(command);
    }
}

impl Beacon2dRenderOutput for WgpuRenderFramePacket {
    fn push_beacon2d_render_command(&mut self, command: BeaconLight2dDrawCommand) {
        self.push_world_2d_beacon(command);
    }
}

impl amigo_text_2d_plugin::Text2dRenderOutput for WgpuRenderFramePacket {
    fn push_text2d_render_command(&mut self, command: Text2dDrawCommand) {
        self.push_world_2d_text(command);
    }
}

impl amigo_2d_composition::Composition2dRenderOutput for WgpuRenderFramePacket {
    fn push_render_layer2d_command(&mut self, command: RenderLayer2dCommand) {
        self.push_world_2d_render_layer(command);
    }

    fn push_light_route2d_command(&mut self, command: LightRoute2dCommand) {
        self.push_world_2d_light_route(command);
    }
}

impl amigo_light_2d_plugin::Lighting2dRenderOutput for WgpuRenderFramePacket {
    fn push_global_light2d_command(&mut self, command: GlobalLight2dCommand) {
        self.push_world_2d_global_light(command);
    }

    fn push_lightmap2d_command(&mut self, command: LightMap2dSourceCommand) {
        self.push_world_2d_lightmap(command);
    }

    fn push_light_group2d_command(&mut self, command: LightGroup2dCommand) {
        self.push_world_2d_light_group(command);
    }
}

impl amigo_particles_2d_plugin::Particle2dRenderOutput for WgpuRenderFramePacket {
    fn push_particle2d_render_command(&mut self, command: Particle2dDrawCommand) {
        self.push_world_2d_particle(command);
    }
}

impl amigo_2d_post_fx::PostFx2dRenderOutput for WgpuRenderFramePacket {
    fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>) {
        self.set_post_fx_stacks(stacks);
    }
}

impl amigo_3d_mesh::Mesh3dRenderOutput for WgpuRenderFramePacket {
    fn push_mesh3d_render_command(&mut self, command: MeshDrawCommand) {
        self.push_world_3d_mesh(command);
    }
}

impl amigo_3d_material::Material3dRenderOutput for WgpuRenderFramePacket {
    fn push_material3d_render_command(&mut self, command: MaterialDrawCommand) {
        self.push_world_3d_material(command);
    }
}

impl amigo_3d_text::Text3dRenderOutput for WgpuRenderFramePacket {
    fn push_text3d_render_command(&mut self, command: Text3dDrawCommand) {
        self.push_world_3d_text(command);
    }
}
