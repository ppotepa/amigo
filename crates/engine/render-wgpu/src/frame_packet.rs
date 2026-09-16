use amigo_render_api::MaterialDrawCommand;
use amigo_render_api::{NprBackgroundCommand, NprDrawCommand, NprRenderOutput};
use amigo_render_api::MeshDrawCommand;
use amigo_render_api::Text3dDrawCommand;
use amigo_render_api::{
    CameraCaptureInput2d, CameraDebugView2d, LightSource2dCommon, RenderContribution2d,
    RenderDepthAuxMap2d, RenderDepthMap2d, RenderExtractionOutput2d, RenderLightGroup2d,
    RenderLightMap2dSource, Renderable2dItem, Renderable2dKind, ScopedPostFx2dStack,
};
use amigo_render_api::{LightRoute2dCommand, RenderLayer2dCommand};

use crate::UiOverlayDocument;

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
    world_2d_render_layers: Vec<RenderLayer2dCommand>,
    world_2d_light_routes: Vec<LightRoute2dCommand>,
    light_sources_2d: Vec<LightSource2dCommon>,
    renderables_2d: Vec<Renderable2dItem>,
    render_contributions_2d: Vec<RenderContribution2d>,
    world_3d_meshes: Vec<MeshDrawCommand>,
    world_3d_materials: Vec<MaterialDrawCommand>,
    world_3d_text: Vec<Text3dDrawCommand>,
    npr: Vec<NprDrawCommand>,
    npr_background: Option<NprBackgroundCommand>,
    game_ui_overlay: Vec<UiOverlayDocument>,
    debug_overlay: Vec<UiOverlayDocument>,
    post_fx_stacks: Vec<ScopedPostFx2dStack>,
    active_camera_2d_entity: Option<String>,
    camera_capture_input_2d: Option<CameraCaptureInput2d>,
    camera_debug_view_2d: Option<CameraDebugView2d>,
    visual_source_flags_2d: WgpuVisualSourceFlags2d,
}

impl WgpuRenderFramePacket {
    pub fn push_world_2d_render_layer(&mut self, command: RenderLayer2dCommand) {
        self.world_2d_render_layers.push(command);
    }

    pub fn push_world_2d_light_route(&mut self, command: LightRoute2dCommand) {
        self.world_2d_light_routes.push(command);
    }

    pub fn push_renderable_2d(&mut self, item: Renderable2dItem) {
        self.renderables_2d.push(item);
    }

    pub fn push_render_contribution_2d(&mut self, contribution: RenderContribution2d) {
        self.render_contributions_2d.push(contribution);
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

    pub fn push_npr_draw_command(&mut self, command: NprDrawCommand) { self.npr.push(command); }
    pub fn set_npr_background(&mut self, background: NprBackgroundCommand) { self.npr_background = Some(background); }

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
        self.world_2d_render_layers.clear();
        self.world_2d_light_routes.clear();
        self.light_sources_2d.clear();
        self.renderables_2d.clear();
        self.render_contributions_2d.clear();
        self.world_3d_meshes.clear();
        self.world_3d_materials.clear();
        self.world_3d_text.clear();
        self.npr.clear();
        self.npr_background = None;
        self.post_fx_stacks.clear();
        self.active_camera_2d_entity = None;
        self.camera_capture_input_2d = None;
        self.camera_debug_view_2d = None;
        self.visual_source_flags_2d = WgpuVisualSourceFlags2d::default();
    }

    pub fn world_2d_render_layers(&self) -> &[RenderLayer2dCommand] {
        &self.world_2d_render_layers
    }

    pub fn world_2d_light_routes(&self) -> &[LightRoute2dCommand] {
        &self.world_2d_light_routes
    }

    pub fn world_2d_light_sources(&self) -> &[LightSource2dCommon] {
        &self.light_sources_2d
    }

    /// Unified world/screen/debug renderable items for the 2D visual pipeline.
    /// The world color pass uses this list as its source of truth.
    pub fn renderables_2d(&self) -> &[Renderable2dItem] {
        &self.renderables_2d
    }

    pub fn render_contributions_2d(&self) -> &[RenderContribution2d] {
        &self.render_contributions_2d
    }

    pub fn renderable_2d_count_by_component_kind(&self, component_kind: &str) -> usize {
        self.renderables_2d
            .iter()
            .filter(|item| item.component_kind() == component_kind)
            .count()
    }

    pub fn renderable_2d_count_by_kind(&self, kind: Renderable2dKind) -> usize {
        self.renderables_2d
            .iter()
            .filter(|item| item.common.kind == kind)
            .count()
    }

    pub fn light_source_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_light_source_2d().is_some())
            .count()
    }

    pub fn lightmap_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_lightmap_2d().is_some())
            .count()
    }

    pub fn light_group_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_light_group_2d().is_some())
            .count()
    }

    pub fn depth_map_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_depth_map_2d().is_some())
            .count()
    }

    pub fn depth_aux_map_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_depth_aux_map_2d().is_some())
            .count()
    }

    pub fn render_lightmaps_2d(&self) -> impl Iterator<Item = &RenderLightMap2dSource> {
        self.render_contributions_2d
            .iter()
            .filter_map(RenderContribution2d::as_lightmap_2d)
    }

    pub fn render_light_groups_2d(&self) -> impl Iterator<Item = &RenderLightGroup2d> {
        self.render_contributions_2d
            .iter()
            .filter_map(RenderContribution2d::as_light_group_2d)
    }

    pub fn render_depth_maps_2d(&self) -> impl Iterator<Item = &RenderDepthMap2d> {
        self.render_contributions_2d
            .iter()
            .filter_map(RenderContribution2d::as_depth_map_2d)
    }

    pub fn render_depth_aux_maps_2d(&self) -> impl Iterator<Item = &RenderDepthAuxMap2d> {
        self.render_contributions_2d
            .iter()
            .filter_map(RenderContribution2d::as_depth_aux_map_2d)
    }

    pub fn camera_optical_candidate_2d_contribution_count(&self) -> usize {
        self.render_contributions_2d
            .iter()
            .filter(|contribution| contribution.as_camera_optical_candidate_2d().is_some())
            .count()
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

    pub fn npr(&self) -> &[NprDrawCommand] { &self.npr }
    pub fn npr_background(&self) -> Option<NprBackgroundCommand> { self.npr_background }

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
        !self.world_2d_render_layers.is_empty()
            || !self.world_2d_light_routes.is_empty()
            || !self.light_sources_2d.is_empty()
            || !self.renderables_2d.is_empty()
            || !self.render_contributions_2d.is_empty()
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
        self.camera_debug_view_2d.clone()
    }

    pub fn visual_source_flags_2d(&self) -> &WgpuVisualSourceFlags2d {
        &self.visual_source_flags_2d
    }
}

impl RenderExtractionOutput2d for WgpuRenderFramePacket {
    fn push_renderable_2d(&mut self, item: Renderable2dItem) {
        WgpuRenderFramePacket::push_renderable_2d(self, item);
    }

    fn push_render_contribution_2d(&mut self, contribution: RenderContribution2d) {
        WgpuRenderFramePacket::push_render_contribution_2d(self, contribution);
    }
}

impl amigo_render_api::Composition2dRenderOutput for WgpuRenderFramePacket {
    fn push_render_layer2d_command(&mut self, command: RenderLayer2dCommand) {
        self.push_world_2d_render_layer(command);
    }

    fn push_light_route2d_command(&mut self, command: LightRoute2dCommand) {
        self.push_world_2d_light_route(command);
    }
}

impl amigo_render_api::PostFx2dRenderOutput for WgpuRenderFramePacket {
    fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>) {
        self.set_post_fx_stacks(stacks);
    }
}

impl amigo_render_api::Mesh3dRenderOutput for WgpuRenderFramePacket {
    fn push_mesh3d_render_command(&mut self, command: MeshDrawCommand) {
        self.push_world_3d_mesh(command);
    }
}

impl amigo_render_api::Material3dRenderOutput for WgpuRenderFramePacket {
    fn push_material3d_render_command(&mut self, command: MaterialDrawCommand) {
        self.push_world_3d_material(command);
    }
}

impl amigo_render_api::Text3dRenderOutput for WgpuRenderFramePacket {
    fn push_text3d_render_command(&mut self, command: Text3dDrawCommand) {
        self.push_world_3d_text(command);
    }
}

impl NprRenderOutput for WgpuRenderFramePacket {
    fn push_npr_draw_command(&mut self, command: NprDrawCommand) { self.push_npr_draw_command(command); }
    fn set_npr_background(&mut self, background: NprBackgroundCommand) { self.set_npr_background(background); }
}
