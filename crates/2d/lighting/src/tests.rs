use amigo_math::ColorRgba;
use amigo_scene::{
    GlobalLight2dSceneCommand, LightMap2dChannelSceneCommand, LightMap2dSourceKindSceneCommand,
    LightMap2dSourceRefSceneCommand, LightMap2dSourceSceneCommand, SceneService,
};

use crate::{
    GlobalLight2dSceneService, LightMap2dSceneService, LightMap2dSourceKind,
    queue_global_light_2d_scene_command, queue_lightmap_2d_source_scene_command,
};

#[test]
fn global_light_service_updates_by_runtime_id() {
    let service = GlobalLight2dSceneService::default();
    let scene = SceneService::default();

    queue_global_light_2d_scene_command(
        &scene,
        &service,
        &GlobalLight2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "storm-controller".to_owned(),
            id: "lightning".to_owned(),
            color: ColorRgba::WHITE,
            intensity: 0.25,
        },
    );

    assert!(service.set_intensity("lightning", -3.0));
    assert!(service.set_color("lightning", ColorRgba::new(0.2, 0.4, 1.0, 1.0)));
    assert!(!service.set_intensity("missing", 1.0));
    assert!(!service.set_color("missing", ColorRgba::WHITE));

    let commands = service.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].id, "lightning");
    assert_eq!(commands[0].entity_name, "storm-controller");
    assert_eq!(commands[0].intensity, 0.0);
    assert_eq!(commands[0].color, ColorRgba::new(0.2, 0.4, 1.0, 1.0));
}

#[test]
fn lightmap_source_scene_command_maps_channels_and_source_ref() {
    let service = LightMap2dSceneService::default();
    let scene = SceneService::default();

    let entity = queue_lightmap_2d_source_scene_command(
        &scene,
        &service,
        &LightMap2dSourceSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "lightmap-controller".to_owned(),
            id: "city-lightmap".to_owned(),
            source: LightMap2dSourceRefSceneCommand {
                kind: LightMap2dSourceKindSceneCommand::LayeredImage2d,
                entity_name: "background".to_owned(),
            },
            channels: vec![
                LightMap2dChannelSceneCommand {
                    id: "far".to_owned(),
                    layers: vec!["far_light".to_owned()],
                },
                LightMap2dChannelSceneCommand {
                    id: "near".to_owned(),
                    layers: vec!["near_light".to_owned(), "warm_fill".to_owned()],
                },
            ],
        },
    );

    let commands = service.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].id, "city-lightmap");
    assert_eq!(
        commands[0].source.kind,
        LightMap2dSourceKind::LayeredImage2d
    );
    assert_eq!(commands[0].source.entity_name, "background");
    assert_eq!(commands[0].channels.len(), 2);
    assert_eq!(commands[0].channels[1].id, "near");
    assert_eq!(commands[0].channels[1].layers, ["near_light", "warm_fill"]);
    assert_eq!(
        scene
            .entity_by_name("lightmap-controller")
            .map(|item| item.id),
        Some(entity)
    );
}

#[test]
fn lighting_services_clear_queued_runtime_state() {
    let scene = SceneService::default();
    let global_lights = GlobalLight2dSceneService::default();
    let lightmaps = LightMap2dSceneService::default();

    queue_global_light_2d_scene_command(
        &scene,
        &global_lights,
        &GlobalLight2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "storm-controller".to_owned(),
            id: "lightning".to_owned(),
            color: ColorRgba::WHITE,
            intensity: 1.0,
        },
    );
    queue_lightmap_2d_source_scene_command(
        &scene,
        &lightmaps,
        &LightMap2dSourceSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "lightmap-controller".to_owned(),
            id: "city-lightmap".to_owned(),
            source: LightMap2dSourceRefSceneCommand {
                kind: LightMap2dSourceKindSceneCommand::LayeredImage2d,
                entity_name: "background".to_owned(),
            },
            channels: Vec::new(),
        },
    );

    global_lights.clear();
    lightmaps.clear();

    assert!(global_lights.commands().is_empty());
    assert!(lightmaps.commands().is_empty());
}
