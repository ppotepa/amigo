use amigo_scene::{
    GlobalLight2dSceneCommand, LightGroup2dSceneCommand, LightMap2dSourceSceneCommand,
    SceneEntityId, SceneService,
};

use crate::{
    GlobalLight2dCommand, GlobalLight2dSceneService, LightGroup2dCommand, LightGroup2dSceneService,
    LightMap2dChannel, LightMap2dSceneService, LightMap2dSourceCommand, LightMap2dSourceRef,
};

pub fn queue_global_light_2d_scene_command(
    scene_service: &SceneService,
    global_light_scene_service: &GlobalLight2dSceneService,
    command: &GlobalLight2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    global_light_scene_service.queue(GlobalLight2dCommand {
        source_mod: command.source_mod.clone(),
        entity_name: command.entity_name.clone(),
        id: command.id.clone(),
        color: command.color,
        intensity: command.intensity.max(0.0),
    });
    entity
}

pub fn queue_light_group_2d_scene_command(
    light_group_scene_service: &LightGroup2dSceneService,
    command: LightGroup2dSceneCommand,
) {
    light_group_scene_service.queue(LightGroup2dCommand {
        source_mod: command.source_mod,
        id: command.id,
        label: command.label,
        color: command.color,
        intensity: command.intensity.max(0.0),
        render_contributions: amigo_render_api::RenderContributionSet::from_pairs(
            command.render_contributions.roles,
        ),
        camera_response: amigo_render_api::CameraOpticalResponse2d {
            enabled: command.camera_response.enabled,
            intensity: command.camera_response.intensity,
            bloom: command.camera_response.bloom,
            glare: command.camera_response.glare,
            ghosting: command.camera_response.ghosting,
            streaks: command.camera_response.streaks,
            chromatic_smear: command.camera_response.chromatic_smear,
            dirt_response: command.camera_response.dirt_response,
            halation: command.camera_response.halation,
            threshold: command.camera_response.threshold,
        }
        .normalized(),
        sources: command.sources.into_iter().map(Into::into).collect(),
    });
}

pub fn queue_lightmap_2d_source_scene_command(
    scene_service: &SceneService,
    lightmap_scene_service: &LightMap2dSceneService,
    command: &LightMap2dSourceSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    lightmap_scene_service.queue(LightMap2dSourceCommand {
        source_mod: command.source_mod.clone(),
        entity_name: command.entity_name.clone(),
        id: command.id.clone(),
        source: LightMap2dSourceRef::from(&command.source),
        channels: command
            .channels
            .iter()
            .map(LightMap2dChannel::from)
            .collect(),
    });
    entity
}
