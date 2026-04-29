use super::super::*;

pub(super) struct AppSceneCommandContext<'a> {
    pub(super) runtime: &'a Runtime,
    pub(super) scene_command_queue: &'a SceneCommandQueue,
    pub(super) launch_selection: &'a LaunchSelection,
    pub(super) hydrated_scene_state: &'a HydratedSceneState,
    pub(super) scene_transition_service: &'a SceneTransitionService,
    pub(super) scene_service: &'a SceneService,
    pub(super) scene_event_queue: &'a SceneEventQueue,
    pub(super) dev_console_state: &'a DevConsoleState,
    pub(super) asset_catalog: &'a AssetCatalog,
    pub(super) sprite_scene_service: &'a SpriteSceneService,
    pub(super) text_scene_service: &'a Text2dSceneService,
    pub(super) physics_scene_service: &'a Physics2dSceneService,
    pub(super) tilemap_scene_service: &'a TileMap2dSceneService,
    pub(super) platformer_scene_service: &'a PlatformerSceneService,
    pub(super) camera_follow_scene_service: &'a CameraFollow2dSceneService,
    pub(super) parallax_scene_service: &'a Parallax2dSceneService,
    pub(super) mesh_scene_service: &'a MeshSceneService,
    pub(super) text3d_scene_service: &'a Text3dSceneService,
    pub(super) material_scene_service: &'a MaterialSceneService,
    pub(super) ui_scene_service: &'a UiSceneService,
}
