use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use amigo_scene::{RuntimeSceneCommandHandlerRegistry, register_runtime_scene_command_handler};

use crate::{
    CameraFollow2dSceneService, CameraSceneCommandHandler, CameraService, Parallax2dSceneService,
    tick_camera_follow_2d_system, tick_parallax_2d_system,
};

pub struct CameraPlugin;

impl RuntimePlugin for CameraPlugin {
    fn name(&self) -> &'static str {
        "amigo-camera"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(CameraService::default())?;
        registry.register(CameraFollow2dSceneService::default())?;
        registry.register(Parallax2dSceneService::default())?;

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
        Ok(())
    }
}
