use super::{BeaconLight2dCommand, BeaconLight2dSceneService};
use amigo_scene::{BeaconLight2dSceneCommand, SceneService};

pub fn queue_beacon_light2d_scene_command(
    scene_service: &SceneService,
    beacon_scene_service: &BeaconLight2dSceneService,
    command: &BeaconLight2dSceneCommand,
) -> amigo_scene::SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    beacon_scene_service.queue(BeaconLight2dCommand::from(command));
    entity
}
