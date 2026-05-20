use amigo_assets::AssetCatalog;
use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_runtime_control::RuntimeControlService;
use amigo_scene::{RuntimeSceneCommandHandlerRegistry, register_runtime_scene_command_handler};
use std::sync::Arc;

use crate::{
    AssetCatalogControlProvider, Camera2dControlProvider, CameraFocusTarget2dService,
    CameraFollow2dSceneService, CameraSceneCommandHandler, CameraService, Parallax2dSceneService,
    tick_camera_focus_transition_2d_system, tick_camera_follow_2d_system, tick_parallax_2d_system,
};

pub struct CameraPlugin;

impl RuntimePlugin for CameraPlugin {
    fn name(&self) -> &'static str {
        "amigo-camera-core-plugin"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(CameraService::default())?;
        registry.register(CameraFocusTarget2dService::default())?;
        registry.register(CameraFollow2dSceneService::default())?;
        registry.register(Parallax2dSceneService::default())?;
        amigo_scene::register_scene_reset_handler(
            registry,
            crate::runtime::CameraCoreSceneResetHandler,
        )?;

        if let (Some(control), Some(cameras), Some(assets)) = (
            registry.resolve::<RuntimeControlService>(),
            registry.resolve::<CameraService>(),
            registry.resolve::<AssetCatalog>(),
        ) {
            control.register_provider(Arc::new(Camera2dControlProvider::new(
                cameras.clone(),
                assets.clone(),
            )));
            control.register_provider(Arc::new(AssetCatalogControlProvider::new(assets)));
        }

        let scene_handlers = registry.required::<RuntimeSceneCommandHandlerRegistry>()?;
        register_runtime_scene_command_handler(scene_handlers.as_ref(), CameraSceneCommandHandler);

        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "camera_follow_2d",
            tick_camera_follow_2d_system,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "parallax_2d",
            tick_parallax_2d_system,
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PostUpdate,
            "camera_focus_transition_2d",
            tick_camera_focus_transition_2d_system,
        );
        Ok(())
    }
}
