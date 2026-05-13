use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{
    CameraFollow2dSceneCommand, Parallax2dSceneCommand, RuntimeSceneCommandHandler, SceneCommand,
    SceneEvent, SceneEventQueue, SceneService, format_scene_command,
};

use crate::{CameraFollow2dSceneService, Parallax2dSceneService};

pub struct CameraSceneCommandHandler;

impl RuntimeSceneCommandHandler for CameraSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueCameraFollow2d { .. } | SceneCommand::QueueParallax2d { .. }
        )
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let camera_follow_scene_service = runtime.required::<CameraFollow2dSceneService>()?;
        let parallax_scene_service = runtime.required::<Parallax2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;

        match command {
            SceneCommand::QueueCameraFollow2d { command } => {
                let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                camera_follow_scene_service.queue(CameraFollow2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    target: command.target.clone(),
                    offset: command.offset,
                    lerp: command.lerp,
                    lookahead_velocity_scale: command.lookahead_velocity_scale,
                    lookahead_max_distance: command.lookahead_max_distance,
                    sway_amount: command.sway_amount,
                    sway_frequency: command.sway_frequency,
                });
                scene_event_queue.publish(SceneEvent::CameraFollowQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name,
                    target: command.target,
                });
                Ok(())
            }
            SceneCommand::QueueParallax2d { command } => {
                let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
                parallax_scene_service.queue(Parallax2dSceneCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    camera: command.camera.clone(),
                    factor: command.factor,
                    anchor: command.anchor,
                    camera_origin: None,
                });
                scene_event_queue.publish(SceneEvent::ParallaxQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name,
                    camera: command.camera,
                });
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "camera scene handler cannot handle command {}",
                format_scene_command(&command)
            ))),
        }
    }
}

