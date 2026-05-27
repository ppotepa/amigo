use amigo_math::ColorRgba;
use amigo_scene::{
    GlobalLight2dSceneCommand, LightMap2dChannelSceneCommand, LightMap2dSourceKindSceneCommand,
    LightMap2dSourceRefSceneCommand, LightMap2dSourceSceneCommand, SceneCommand, SceneEvent,
    SceneEventQueue, SceneService,
};

use super::{
    can_handle_lighting_scene_command, handle_lighting_scene_command,
    queue_global_light_2d_scene_command, queue_lightmap_2d_source_scene_command,
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
    LightMap2dSourceKind, LightingSceneCommandContext, LightingSceneCommandOutcome,
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

#[test]
fn lighting_scene_command_handler_queues_global_light_and_event() {
    let scene = SceneService::default();
    let global_lights = GlobalLight2dSceneService::default();
    let lightmaps = LightMap2dSceneService::default();
    let light_groups = LightGroup2dSceneService::default();
    let events = SceneEventQueue::default();
    let command = GlobalLight2dSceneCommand {
        source_mod: "test-mod".to_owned(),
        entity_name: "storm-controller".to_owned(),
        id: "lightning".to_owned(),
        color: ColorRgba::WHITE,
        intensity: 2.0,
    };

    assert!(can_handle_lighting_scene_command(&SceneCommand::plugin(
        amigo_scene::global_light_2d_plugin_scene_command(command.clone())
    )));

    let outcome = handle_lighting_scene_command(
        LightingSceneCommandContext {
            scene_service: &scene,
            global_light2d_scene_service: &global_lights,
            lightmap2d_scene_service: &lightmaps,
            light_group2d_scene_service: &light_groups,
            scene_event_queue: &events,
            resolve_lightmap_source_layers: &|_| None,
        },
        SceneCommand::plugin(amigo_scene::global_light_2d_plugin_scene_command(command)),
    )
    .expect("global light scene command should be handled");

    let LightingSceneCommandOutcome::GlobalLight {
        id, entity_name, ..
    } = outcome
    else {
        panic!("expected global light outcome");
    };
    assert_eq!(id, "lightning");
    assert_eq!(entity_name, "storm-controller");
    assert_eq!(global_lights.commands().len(), 1);

    let entity = scene
        .entity_by_name("storm-controller")
        .expect("entity should be spawned");
    assert_eq!(
        events.pending(),
        [SceneEvent::EntitySpawned {
            entity_id: entity.id.raw(),
            name: "storm-controller".to_owned()
        }]
    );
}
