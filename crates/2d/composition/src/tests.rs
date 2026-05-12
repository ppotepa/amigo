use amigo_scene::{LightRoute2dSceneCommand, RenderLayer2dSceneCommand, SceneCommand};

use crate::{
    CompositionSceneCommandContext, CompositionSceneCommandOutcome, LightRoute2dSceneService,
    RenderLayer2dSceneService, can_handle_composition_scene_command,
    handle_composition_scene_command,
};

#[test]
fn can_handle_composition_scene_command_returns_true_for_composition_commands() {
    assert!(can_handle_composition_scene_command(
        &SceneCommand::QueueRenderLayer2d {
            command: RenderLayer2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                id: "world".to_owned(),
                label: Some("World".to_owned()),
                order: 0.0,
                visible: true,
                opacity: 1.0,
            },
        }
    ));
    assert!(can_handle_composition_scene_command(
        &SceneCommand::QueueLightRoute2d {
            command: LightRoute2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "world".to_owned(),
                groups: vec!["sun".to_owned()],
            },
        }
    ));
}

#[test]
fn handle_composition_scene_command_queues_render_layer() {
    let render_layer2d_scene_service = RenderLayer2dSceneService::default();
    let light_route2d_scene_service = LightRoute2dSceneService::default();
    let outcome = handle_composition_scene_command(
        CompositionSceneCommandContext {
            render_layer2d_scene_service: &render_layer2d_scene_service,
            light_route2d_scene_service: &light_route2d_scene_service,
        },
        SceneCommand::QueueRenderLayer2d {
            command: RenderLayer2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                id: "world".to_owned(),
                label: Some("World".to_owned()),
                order: 0.0,
                visible: true,
                opacity: 1.25,
            },
        },
    )
    .expect("render layer command should be handled");

    assert_eq!(
        outcome,
        CompositionSceneCommandOutcome::RenderLayer {
            id: "world".to_owned(),
            source_mod: "test-mod".to_owned(),
        }
    );
    let commands = render_layer2d_scene_service.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].id, "world");
    assert_eq!(commands[0].opacity, 1.0);
}

#[test]
fn handle_composition_scene_command_queues_light_route() {
    let render_layer2d_scene_service = RenderLayer2dSceneService::default();
    let light_route2d_scene_service = LightRoute2dSceneService::default();
    let outcome = handle_composition_scene_command(
        CompositionSceneCommandContext {
            render_layer2d_scene_service: &render_layer2d_scene_service,
            light_route2d_scene_service: &light_route2d_scene_service,
        },
        SceneCommand::QueueLightRoute2d {
            command: LightRoute2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "world".to_owned(),
                groups: vec!["sun".to_owned()],
            },
        },
    )
    .expect("light route command should be handled");

    assert_eq!(
        outcome,
        CompositionSceneCommandOutcome::LightRoute {
            receiver_layer: "world".to_owned(),
            source_mod: "test-mod".to_owned(),
        }
    );
    let commands = light_route2d_scene_service.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].receiver_layer, "world");
    assert_eq!(commands[0].groups, vec!["sun".to_owned()]);
}
