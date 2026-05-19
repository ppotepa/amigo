use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, SceneEvent, SceneEventQueue, SceneService, format_scene_command};

use crate::BeaconLight2dSceneService;

use super::scene_bridge::queue_beacon_light2d_scene_command;

pub struct Beacon2dSceneCommandHandler;

pub fn can_handle_beacon_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueBeaconLight2d { .. })
}

impl amigo_scene::RuntimeSceneCommandHandler for Beacon2dSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_beacon_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let scene_service = runtime.required::<SceneService>()?;
        let beacon_service = runtime.required::<BeaconLight2dSceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        match command {
            SceneCommand::QueueBeaconLight2d { command } => {
                let entity = queue_beacon_light2d_scene_command(
                    scene_service.as_ref(),
                    beacon_service.as_ref(),
                    &command,
                );
                scene_event_queue.publish(SceneEvent::EntitySpawned {
                    entity_id: entity.raw(),
                    name: command.entity_name,
                });
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "beacon-2d cannot handle command {}",
                format_scene_command(&command)
            ))),
        }
    }
}
