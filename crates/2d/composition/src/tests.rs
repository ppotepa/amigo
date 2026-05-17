use amigo_scene::{
    LightRoute2dSceneCommand, RenderDepth2dSceneCommand, RenderDepthMode2dSceneCommand,
    RenderLayer2dSceneCommand, SceneCommand,
};
use amigo_scripting_api::ScriptCommand;

use crate::{
    Composition2dDevConsoleCommandContext, Composition2dDevConsoleCommandOutcome,
    Composition2dScriptCommandContext, Composition2dScriptCommandOutcome,
    CompositionSceneCommandContext, CompositionSceneCommandOutcome, LightRoute2dSceneService,
    RenderDepth2d, RenderDepthMode2d, RenderLayer2dCommand, RenderLayer2dSceneService,
    can_handle_composition_scene_command, handle_composition_scene_command,
    handle_composition2d_dev_console_command, handle_composition2d_script_command,
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
                depth: RenderDepth2dSceneCommand {
                    mode: RenderDepthMode2dSceneCommand::DepthMap,
                    value: 0.5,
                    blur_scale: 1.0,
                },
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
                depth: RenderDepth2dSceneCommand {
                    mode: RenderDepthMode2dSceneCommand::Plane,
                    value: 0.52,
                    blur_scale: 9.0,
                },
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
    assert!(commands[0].depth.is_plane());
    assert_eq!(commands[0].depth.value, 0.52);
    assert_eq!(commands[0].depth.blur_scale, 4.0);
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

#[test]
fn render_layer_service_updates_depth_fields() {
    let service = RenderLayer2dSceneService::default();
    service.queue(RenderLayer2dCommand {
        source_mod: "test-mod".to_owned(),
        id: "weather.rain.mid".to_owned(),
        label: Some("Mid Rain".to_owned()),
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: RenderDepth2d::default(),
    });

    assert!(service.set_depth_mode("weather.rain.mid", RenderDepthMode2d::Plane));
    assert!(service.set_depth_plane_value("weather.rain.mid", 9.0));
    assert!(service.set_depth_blur_scale("weather.rain.mid", 9.0));

    let commands = service.commands();
    assert_eq!(commands.len(), 1);
    assert!(commands[0].depth.is_plane());
    assert_eq!(commands[0].depth.value, 1.0);
    assert_eq!(commands[0].depth.blur_scale, 4.0);
}

#[test]
fn dev_console_updates_render_layer_depth_fields() {
    let render_layers = RenderLayer2dSceneService::default();
    let light_routes = LightRoute2dSceneService::default();
    render_layers.queue(RenderLayer2dCommand {
        source_mod: "test-mod".to_owned(),
        id: "weather.rain.mid".to_owned(),
        label: Some("Mid Rain".to_owned()),
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: RenderDepth2d::default(),
    });

    let result = handle_composition2d_dev_console_command(
        Composition2dDevConsoleCommandContext {
            render_layer2d_scene_service: &render_layers,
            light_route2d_scene_service: &light_routes,
        },
        "layer.depth.mode",
        &["weather.rain.mid".to_owned(), "plane".to_owned()],
    );
    assert!(matches!(
        result,
        Composition2dDevConsoleCommandOutcome::Handled(_)
    ));

    let result = handle_composition2d_dev_console_command(
        Composition2dDevConsoleCommandContext {
            render_layer2d_scene_service: &render_layers,
            light_route2d_scene_service: &light_routes,
        },
        "layer.depth.value",
        &["weather.rain.mid".to_owned(), "0.52".to_owned()],
    );
    assert!(matches!(
        result,
        Composition2dDevConsoleCommandOutcome::Handled(_)
    ));

    let result = handle_composition2d_dev_console_command(
        Composition2dDevConsoleCommandContext {
            render_layer2d_scene_service: &render_layers,
            light_route2d_scene_service: &light_routes,
        },
        "layer.depth.blur_scale",
        &["weather.rain.mid".to_owned(), "0.25".to_owned()],
    );
    assert!(matches!(
        result,
        Composition2dDevConsoleCommandOutcome::Handled(_)
    ));

    let commands = render_layers.commands();
    assert_eq!(commands[0].depth.mode, RenderDepthMode2d::Plane);
    assert_eq!(commands[0].depth.value, 0.52);
    assert_eq!(commands[0].depth.blur_scale, 0.25);
}

#[test]
fn script_command_updates_render_layer_depth_fields() {
    let render_layers = RenderLayer2dSceneService::default();
    render_layers.queue(RenderLayer2dCommand {
        source_mod: "test-mod".to_owned(),
        id: "weather.rain.mid".to_owned(),
        label: Some("Mid Rain".to_owned()),
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: RenderDepth2d::default(),
    });

    let result = handle_composition2d_script_command(
        Composition2dScriptCommandContext {
            render_layer2d_scene_service: &render_layers,
        },
        ScriptCommand {
            namespace: "2d.render_layer".to_owned(),
            name: "set_depth_mode".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "plane".to_owned()],
        },
    );
    assert!(matches!(
        result,
        Composition2dScriptCommandOutcome::Updated(_)
    ));

    let result = handle_composition2d_script_command(
        Composition2dScriptCommandContext {
            render_layer2d_scene_service: &render_layers,
        },
        ScriptCommand {
            namespace: "2d.render_layer".to_owned(),
            name: "set_depth_value".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "0.52".to_owned()],
        },
    );
    assert!(matches!(
        result,
        Composition2dScriptCommandOutcome::Updated(_)
    ));

    let result = handle_composition2d_script_command(
        Composition2dScriptCommandContext {
            render_layer2d_scene_service: &render_layers,
        },
        ScriptCommand {
            namespace: "2d.render_layer".to_owned(),
            name: "set_depth_blur_scale".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "0.25".to_owned()],
        },
    );
    assert!(matches!(
        result,
        Composition2dScriptCommandOutcome::Updated(_)
    ));

    let commands = render_layers.commands();
    assert_eq!(commands[0].depth.mode, RenderDepthMode2d::Plane);
    assert_eq!(commands[0].depth.value, 0.52);
    assert_eq!(commands[0].depth.blur_scale, 0.25);
}
