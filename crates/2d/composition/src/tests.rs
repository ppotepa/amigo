use amigo_scene::{
    light_route_2d_plugin_scene_command, render_layer_2d_plugin_scene_command,
    visual2d_spatial_plugin_scene_command, LightRoute2dSceneCommand,
    OpticalLayerRole2dSceneCommand, RenderDepth2dSceneCommand, RenderDepthMode2dSceneCommand,
    RenderLayer2dSceneCommand, SceneCommand,
};
use amigo_scripting_api::ScriptCommand;

use crate::{
    can_handle_composition_scene_command, handle_composition2d_dev_console_command,
    handle_composition2d_script_command, handle_composition_scene_command,
    Composition2dDevConsoleCommandContext, Composition2dDevConsoleCommandOutcome,
    Composition2dScriptCommandContext, Composition2dScriptCommandOutcome,
    CompositionSceneCommandContext, CompositionSceneCommandOutcome, LightRoute2dSceneService,
    RenderDepth2d, RenderDepthMode2d, RenderLayer2dCommand, RenderLayer2dSceneService,
};

#[test]
fn can_handle_composition_scene_command_returns_true_for_composition_commands() {
    assert!(can_handle_composition_scene_command(
        &SceneCommand::Plugin {
            command: render_layer_2d_plugin_scene_command(RenderLayer2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                id: "world".to_owned(),
                label: Some("World".to_owned()),
                order: 0.0,
                visible: true,
                opacity: 1.0,
                depth: RenderDepth2dSceneCommand {
                    mode: RenderDepthMode2dSceneCommand::DepthMap,
                    distance_m: None,
                    z_depth: 0.5,
                    blur_scale: 1.0,
                },
                optical_role: OpticalLayerRole2dSceneCommand::WorldSurface,
            }),
        }
    ));
    assert!(can_handle_composition_scene_command(
        &SceneCommand::Plugin {
            command: light_route_2d_plugin_scene_command(LightRoute2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "world".to_owned(),
                groups: vec!["sun".to_owned()],
            }),
        }
    ));
    assert!(can_handle_composition_scene_command(
        &SceneCommand::Plugin {
            command: visual2d_spatial_plugin_scene_command(amigo_scene::DepthSpace2dSceneCommand {
                near_m: 0.25,
                far_m: 250.0,
                curve: amigo_scene::DepthCurve2dSceneCommand::Logarithmic,
            }),
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
        SceneCommand::Plugin {
            command: render_layer_2d_plugin_scene_command(RenderLayer2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                id: "world".to_owned(),
                label: Some("World".to_owned()),
                order: 0.0,
                visible: true,
                opacity: 1.25,
                depth: RenderDepth2dSceneCommand {
                    mode: RenderDepthMode2dSceneCommand::ZDepth,
                    distance_m: None,
                    z_depth: 0.52,
                    blur_scale: 9.0,
                },
                optical_role: OpticalLayerRole2dSceneCommand::ForegroundMedium,
            }),
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
    assert!(commands[0].depth.is_z_depth());
    assert_eq!(commands[0].depth.z_depth, 0.52);
    assert_eq!(commands[0].depth.blur_scale, 4.0);
    assert_eq!(
        commands[0].optical_role,
        amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium
    );
}

#[test]
fn scene_bridge_preserves_distance_depth_and_computed_z_depth() {
    let command = RenderLayer2dSceneCommand {
        source_mod: "test-mod".to_owned(),
        id: "weather.rain.mid".to_owned(),
        label: None,
        order: 0.0,
        visible: true,
        opacity: 1.0,
        depth: RenderDepth2dSceneCommand {
            mode: RenderDepthMode2dSceneCommand::Distance,
            distance_m: Some(75.0),
            z_depth: 0.41,
            blur_scale: 0.25,
        },
        optical_role: OpticalLayerRole2dSceneCommand::SceneMedium,
    };
    let command: RenderLayer2dCommand = crate::render_layer_2d_command_from_scene(command);
    assert!(command.depth.is_distance());
    assert_eq!(command.depth.distance_m, Some(75.0));
    assert_eq!(command.depth.z_depth, 0.41);
    assert_eq!(command.depth.blur_scale, 0.25);
    assert_eq!(
        command.optical_role,
        amigo_2d_spatial::OpticalLayerRole2d::SceneMedium
    );
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
        SceneCommand::Plugin {
            command: light_route_2d_plugin_scene_command(LightRoute2dSceneCommand {
                source_mod: "test-mod".to_owned(),
                receiver_layer: "world".to_owned(),
                groups: vec!["sun".to_owned()],
            }),
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
fn handle_composition_scene_command_sets_visual2d_depth_space() {
    let render_layer2d_scene_service = RenderLayer2dSceneService::default();
    let light_route2d_scene_service = LightRoute2dSceneService::default();
    let outcome = handle_composition_scene_command(
        CompositionSceneCommandContext {
            render_layer2d_scene_service: &render_layer2d_scene_service,
            light_route2d_scene_service: &light_route2d_scene_service,
        },
        SceneCommand::Plugin {
            command: visual2d_spatial_plugin_scene_command(amigo_scene::DepthSpace2dSceneCommand {
                near_m: 0.25,
                far_m: 250.0,
                curve: amigo_scene::DepthCurve2dSceneCommand::Logarithmic,
            }),
        },
    )
    .expect("visual2d spatial command should be handled");

    assert_eq!(
        outcome,
        CompositionSceneCommandOutcome::RenderLayer {
            id: "visual2d.spatial".to_owned(),
            source_mod: "scene".to_owned(),
        }
    );
    let depth_space = render_layer2d_scene_service.depth_space();
    assert_eq!(depth_space.near_m, 0.25);
    assert_eq!(depth_space.far_m, 250.0);
    assert_eq!(
        depth_space.curve,
        amigo_2d_spatial::DepthCurve2d::Logarithmic
    );
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
        optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
    });

    assert!(service.set_depth_mode("weather.rain.mid", RenderDepthMode2d::ZDepth));
    assert!(service.set_z_depth("weather.rain.mid", 9.0));
    assert!(service.set_depth_blur_scale("weather.rain.mid", 9.0));

    let commands = service.commands();
    assert_eq!(commands.len(), 1);
    assert!(commands[0].depth.is_z_depth());
    assert_eq!(commands[0].depth.z_depth, 1.0);
    assert_eq!(commands[0].depth.blur_scale, 4.0);
    assert_eq!(
        commands[0].optical_role,
        amigo_2d_spatial::OpticalLayerRole2d::WorldSurface
    );
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
        optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
    });

    let result = handle_composition2d_dev_console_command(
        Composition2dDevConsoleCommandContext {
            render_layer2d_scene_service: &render_layers,
            light_route2d_scene_service: &light_routes,
        },
        "layer.depth.mode",
        &["weather.rain.mid".to_owned(), "distance".to_owned()],
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
        "layer.depth.distance_m",
        &["weather.rain.mid".to_owned(), "75".to_owned()],
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

    let result = handle_composition2d_dev_console_command(
        Composition2dDevConsoleCommandContext {
            render_layer2d_scene_service: &render_layers,
            light_route2d_scene_service: &light_routes,
        },
        "layer.optical_role",
        &["weather.rain.mid".to_owned(), "scene_medium".to_owned()],
    );
    assert!(matches!(
        result,
        Composition2dDevConsoleCommandOutcome::Handled(_)
    ));

    let commands = render_layers.commands();
    assert_eq!(commands[0].depth.mode, RenderDepthMode2d::Distance);
    assert_eq!(commands[0].depth.distance_m, Some(75.0));
    assert!((commands[0].depth.z_depth - 0.41).abs() < 0.015);
    assert_eq!(commands[0].depth.blur_scale, 0.25);
    assert_eq!(
        commands[0].optical_role,
        amigo_2d_spatial::OpticalLayerRole2d::SceneMedium
    );
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
        optical_role: amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
    });

    let result = handle_composition2d_script_command(
        Composition2dScriptCommandContext {
            render_layer2d_scene_service: &render_layers,
        },
        ScriptCommand {
            namespace: "2d.render_layer".to_owned(),
            name: "set_depth_mode".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "distance".to_owned()],
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
            name: "set_distance_m".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "75".to_owned()],
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

    let result = handle_composition2d_script_command(
        Composition2dScriptCommandContext {
            render_layer2d_scene_service: &render_layers,
        },
        ScriptCommand {
            namespace: "2d.render_layer".to_owned(),
            name: "set_optical_role".to_owned(),
            arguments: vec!["weather.rain.mid".to_owned(), "scene_medium".to_owned()],
        },
    );
    assert!(matches!(
        result,
        Composition2dScriptCommandOutcome::Updated(_)
    ));

    let commands = render_layers.commands();
    assert_eq!(commands[0].depth.mode, RenderDepthMode2d::Distance);
    assert_eq!(commands[0].depth.distance_m, Some(75.0));
    assert!((commands[0].depth.z_depth - 0.41).abs() < 0.015);
    assert_eq!(commands[0].depth.blur_scale, 0.25);
    assert_eq!(
        commands[0].optical_role,
        amigo_2d_spatial::OpticalLayerRole2d::SceneMedium
    );
}
