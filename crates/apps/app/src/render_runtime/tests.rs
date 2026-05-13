use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2};
use amigo_scene::SceneEntityId;
use amigo_scripting_api::DevConsoleState;
use amigo_runtime_bundles::amigo_ui::{
    UiDocument as RuntimeUiDocument, UiDrawCommand, UiLayer as RuntimeUiLayer,
    UiNode as RuntimeUiNode, UiNodeKind as RuntimeUiNodeKind, UiSceneService, UiStateService,
    UiStyle as RuntimeUiStyle, UiTarget as RuntimeUiTarget, UiTheme, UiThemePalette,
    UiThemeService,
};

use super::*;
use amigo_runtime_bundles::amigo_2d_particles::Particle2dSceneService;
use amigo_runtime_bundles::amigo_2d_sprite::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
use amigo_runtime_bundles::amigo_2d_text::{Text2d, Text2dDrawCommand, Text2dSceneService};
use amigo_runtime_bundles::amigo_2d_tilemap::{TileMap2d, TileMap2dDrawCommand, TileMap2dSceneService};
use amigo_runtime_bundles::amigo_2d_vector::{
    VectorSceneService, VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d,
};
use amigo_runtime_bundles::amigo_3d_material::{Material3d, MaterialDrawCommand, MaterialSceneService};
use amigo_runtime_bundles::amigo_3d_mesh::{Mesh3d, MeshDrawCommand, MeshSceneService};
use amigo_runtime_bundles::amigo_3d_text::{Text3d, Text3dDrawCommand, Text3dSceneService};

fn hud_document(entity_name: &str, text: &str) -> UiDrawCommand {
    UiDrawCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: entity_name.to_owned(),
        document: RuntimeUiDocument {
            target: RuntimeUiTarget::ScreenSpace {
                layer: RuntimeUiLayer::Hud,
                viewport: None,
            },
            root: RuntimeUiNode {
                id: Some("root".to_owned()),
                kind: RuntimeUiNodeKind::Text {
                    content: text.to_owned(),
                    font: None,
                },
                style_class: Some("root".to_owned()),
                style: RuntimeUiStyle::default(),
                binds: Default::default(),
                events: Default::default(),
                children: Vec::new(),
            },
        },
    }
}

fn test_overlay_document(entity_name: &str) -> amigo_render_wgpu::UiOverlayDocument {
    amigo_render_wgpu::UiOverlayDocument {
        entity_name: entity_name.to_owned(),
        layer: amigo_render_wgpu::UiOverlayLayer::Hud,
        viewport: None,
        root: amigo_render_wgpu::UiOverlayNode {
            id: Some("root".to_owned()),
            kind: amigo_render_wgpu::UiOverlayNodeKind::Stack,
            style: amigo_render_wgpu::UiOverlayStyle::default(),
            children: Vec::new(),
        },
    }
}

#[test]
fn app_render_extractor_registry_collects_vector_and_ui_data() {
    let tilemaps = TileMap2dSceneService::default();
    tilemaps.queue(TileMap2dDrawCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "arena".to_owned(),
        tilemap: TileMap2d {
            tileset: AssetKey::new(
                "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
            ),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["....".to_owned(), "####".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        },
        render_layer: "default".to_owned(),
        z_index: -1.0,
    });
    let sprites = SpriteSceneService::default();
    sprites.queue(SpriteDrawCommand {
        entity_id: SceneEntityId::new(5),
        entity_name: "player".to_owned(),
        sprite: Sprite {
            texture: AssetKey::new("playground-2d/spritesheets/sprite-lab"),
            size: Vec2::new(32.0, 32.0),
            sheet: Some(SpriteSheet {
                columns: 4,
                rows: 1,
                frame_count: 4,
                frame_size: Vec2::new(32.0, 32.0),
                fps: 8.0,
                looping: true,
            }),
            sheet_is_explicit: true,
            animation_override: None,
            frame_index: 2,
            frame_elapsed: 0.1,
        },
        render_layer: "default".to_owned(),
        z_index: 1.0,
        transform: Transform2::default(),
    });
    let vectors = VectorSceneService::default();
    vectors.queue(VectorShape2dDrawCommand {
        entity_id: SceneEntityId::new(7),
        entity_name: "ship".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polyline {
                points: vec![Vec2::new(0.0, 12.0), Vec2::new(-8.0, -8.0)],
                closed: true,
            },
            style: VectorStyle2d {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 2.0,
                fill_color: None,
            },
        },
        render_layer: "default".to_owned(),
        z_index: 2.0,
        transform: Transform2::default(),
    });
    let particles = Particle2dSceneService::default();
    particles.queue_emitter(amigo_runtime_bundles::amigo_2d_particles::ParticleEmitter2dCommand {
        entity_id: SceneEntityId::new(14),
        entity_name: "spark".to_owned(),
        emitter: amigo_runtime_bundles::amigo_2d_particles::ParticleEmitter2d {
            attached_to: None,
            local_offset: Vec2::ZERO,
            local_direction_radians: 0.0,
            spawn_area: amigo_runtime_bundles::amigo_2d_particles::ParticleSpawnArea2d::Point,
            active: true,
            spawn_rate: 1.0,
            max_particles: 4,
            particle_lifetime: 1.0,
            lifetime_jitter: 0.0,
            initial_speed: 0.0,
            speed_jitter: 0.0,
            spread_radians: 0.0,
            inherit_parent_velocity: 0.0,
            velocity_mode: amigo_runtime_bundles::amigo_2d_particles::ParticleVelocityMode2d::Free,
            simulation_space: amigo_runtime_bundles::amigo_2d_particles::ParticleSimulationSpace2d::World,
            initial_size: 2.0,
            final_size: 2.0,
            color: ColorRgba::WHITE,
            color_ramp: None,
            render_layer: "default".to_owned(),
            z_index: 3.5,
            shape: amigo_runtime_bundles::amigo_2d_particles::ParticleShape2d::Circle { segments: 8 },
            shape_choices: Vec::new(),
            shape_over_lifetime: Vec::new(),
            line_anchor: amigo_runtime_bundles::amigo_2d_particles::ParticleLineAnchor2d::Center,
            align: amigo_runtime_bundles::amigo_2d_particles::ParticleAlignMode2d::Velocity,
            blend_mode: amigo_runtime_bundles::amigo_2d_particles::ParticleBlendMode2d::Alpha,
            motion_stretch: None,
            material: amigo_runtime_bundles::amigo_2d_particles::ParticleMaterial2d {
                lighting_mode: amigo_runtime_bundles::amigo_2d_lighting::Material2dLightingMode::Unlit,
                light_response: 1.0,
                light_receiver: None,
            },
            light: None,
            emission_rate_curve: amigo_math::Curve1d::Constant(1.0),
            size_curve: amigo_math::Curve1d::Constant(1.0),
            alpha_curve: amigo_math::Curve1d::Constant(1.0),
            speed_curve: amigo_math::Curve1d::Constant(1.0),
            forces: Vec::new(),
        },
    });
    particles.tick(
        &[amigo_runtime_bundles::amigo_2d_particles::Particle2dEmitterRuntimeInput {
            emitter_entity_name: "spark".to_owned(),
            source_entity_name: "spark".to_owned(),
            source_transform: Transform2::default(),
            source_velocity: Vec2::ZERO,
            source_visible: true,
            source_simulation_enabled: true,
        }],
        1.0,
    );
    vectors.queue(VectorShape2dDrawCommand {
        entity_id: SceneEntityId::new(13),
        entity_name: "hidden-dot".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Circle {
                radius: 1.0,
                segments: 8,
            },
            style: VectorStyle2d {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 1.0,
                fill_color: Some(ColorRgba::WHITE),
            },
        },
        render_layer: "default".to_owned(),
        z_index: 3.0,
        transform: Transform2::default(),
    });
    let text2d = Text2dSceneService::default();
    text2d.queue(Text2dDrawCommand {
        entity_id: SceneEntityId::new(8),
        entity_name: "label".to_owned(),
        text: Text2d {
            content: "AMIGO".to_owned(),
            font: AssetKey::new("playground-2d/fonts/debug-ui"),
            bounds: Vec2::new(320.0, 64.0),
            transform: Transform2::default(),
        },
        render_layer: "default".to_owned(),
        z_index: 0.0,
    });
    let text3d = Text3dSceneService::default();
    let layered_images = amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService::default();
    let global_lights = amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService::default();
    let lightmaps = amigo_runtime_bundles::amigo_2d_lighting::LightMap2dSceneService::default();
    let render_layers = amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService::default();
    let light_routes = amigo_runtime_bundles::amigo_2d_composition::LightRoute2dSceneService::default();
    let light_groups = amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService::default();
    text3d.queue(Text3dDrawCommand {
        entity_id: SceneEntityId::new(10),
        entity_name: "hello-3d".to_owned(),
        text: Text3d {
            content: "HELLO".to_owned(),
            font: AssetKey::new("playground-3d/fonts/debug-3d"),
            size: 0.5,
            transform: Transform3::default(),
        },
    });
    let meshes = MeshSceneService::default();
    meshes.queue(MeshDrawCommand {
        entity_id: SceneEntityId::new(11),
        entity_name: "probe-mesh".to_owned(),
        mesh: Mesh3d {
            mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
            transform: Transform3::default(),
        },
    });
    let materials = MaterialSceneService::default();
    materials.queue(MaterialDrawCommand {
        entity_id: SceneEntityId::new(12),
        entity_name: "probe-material".to_owned(),
        material: Material3d {
            label: "debug-surface".to_owned(),
            albedo: ColorRgba::WHITE,
            source: Some(AssetKey::new("playground-3d/materials/debug-surface")),
        },
    });

    let ui_scene = UiSceneService::default();
    let ui_state = UiStateService::default();
    let ui_theme = UiThemeService::default();
    ui_theme.register_theme(UiTheme::from_palette(
        "space_dark",
        UiThemePalette {
            background: ColorRgba::new(0.02, 0.03, 0.07, 1.0),
            surface: ColorRgba::new(0.08, 0.1, 0.15, 1.0),
            surface_alt: ColorRgba::new(0.1, 0.12, 0.18, 1.0),
            text: ColorRgba::WHITE,
            text_muted: ColorRgba::new(0.6, 0.7, 0.8, 1.0),
            border: ColorRgba::new(0.2, 0.4, 0.6, 1.0),
            accent: ColorRgba::new(0.0, 0.8, 1.0, 1.0),
            accent_text: ColorRgba::new(0.0, 0.05, 0.08, 1.0),
            danger: ColorRgba::new(1.0, 0.1, 0.2, 1.0),
            warning: ColorRgba::new(1.0, 0.7, 0.0, 1.0),
            success: ColorRgba::new(0.2, 1.0, 0.5, 1.0),
        },
    ));
    ui_theme.set_active_theme("space_dark");
    ui_scene.queue(hud_document("hud", "Hello"));
    let scene = amigo_scene::SceneService::default();
    scene.spawn("hidden-dot");
    scene.set_visible("hidden-dot", false);
    let dev_console_state = DevConsoleState::default();
    let dev_console_completion = crate::dev_console::completion::ConsoleCompletionState::default();
    let debug_overlay_service = crate::debug_overlay::DebugOverlayService::default();
    let post_fx_service = amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService::default();
    let ui_viewport_state = amigo_runtime_bundles::amigo_ui::UiInputViewportState::default();

    let runtime = amigo_runtime::RuntimeBuilder::default()
        .with_service(scene)
        .unwrap()
        .with_service(tilemaps)
        .unwrap()
        .with_service(sprites)
        .unwrap()
        .with_service(layered_images)
        .unwrap()
        .with_service(render_layers)
        .unwrap()
        .with_service(light_routes)
        .unwrap()
        .with_service(global_lights)
        .unwrap()
        .with_service(lightmaps)
        .unwrap()
        .with_service(light_groups)
        .unwrap()
        .with_service(text2d)
        .unwrap()
        .with_service(vectors)
        .unwrap()
        .with_service(particles)
        .unwrap()
        .with_service(meshes)
        .unwrap()
        .with_service(materials)
        .unwrap()
        .with_service(text3d)
        .unwrap()
        .with_service(ui_scene)
        .unwrap()
        .with_service(ui_state)
        .unwrap()
        .with_service(ui_theme)
        .unwrap()
        .with_service(post_fx_service)
        .unwrap()
        .with_service(dev_console_state)
        .unwrap()
        .with_service(dev_console_completion)
        .unwrap()
        .with_service(debug_overlay_service)
        .unwrap()
        .with_service(ui_viewport_state)
        .unwrap()
        .build();

    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(&runtime);

    assert_eq!(packet.world_2d_tilemaps().len(), 1);
    assert_eq!(packet.world_2d_tilemaps()[0].entity_name, "arena");
    assert_eq!(packet.world_2d_sprites().len(), 1);
    assert_eq!(packet.world_2d_sprites()[0].entity_name, "player");
    assert_eq!(packet.world_2d_text().len(), 1);
    assert_eq!(packet.world_2d_text()[0].entity_name, "label");
    assert_eq!(packet.world_2d_vectors().len(), 1);
    assert_eq!(packet.world_2d_vectors()[0].entity_name, "ship");
    assert_eq!(packet.world_2d_particles().len(), 1);
    assert_eq!(packet.world_3d_meshes().len(), 1);
    assert_eq!(packet.world_3d_meshes()[0].entity_name, "probe-mesh");
    assert_eq!(packet.world_3d_materials().len(), 1);
    assert_eq!(packet.world_3d_materials()[0].entity_name, "probe-material");
    assert_eq!(packet.world_3d_text().len(), 1);
    assert_eq!(packet.world_3d_text()[0].entity_name, "hello-3d");
    assert_eq!(packet.game_ui_overlay().len(), 1);
    assert_eq!(packet.game_ui_overlay()[0].entity_name, "hud");
    assert_eq!(
        packet.game_ui_overlay()[0].root.style.background,
        Some(ColorRgba::new(0.02, 0.03, 0.07, 1.0))
    );
}

#[test]
fn app_render_extractor_registry_appends_enabled_debug_overlay() {
    let tilemaps = TileMap2dSceneService::default();
    let sprites = SpriteSceneService::default();
    let layered_images = amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService::default();
    let render_layers = amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService::default();
    let light_routes = amigo_runtime_bundles::amigo_2d_composition::LightRoute2dSceneService::default();
    let global_lights = amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService::default();
    let lightmaps = amigo_runtime_bundles::amigo_2d_lighting::LightMap2dSceneService::default();
    let light_groups = amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService::default();
    let text2d = Text2dSceneService::default();
    let vectors = VectorSceneService::default();
    let particles = Particle2dSceneService::default();
    let meshes = MeshSceneService::default();
    let materials = MaterialSceneService::default();
    let text3d = Text3dSceneService::default();
    let ui_scene = UiSceneService::default();
    let ui_state = UiStateService::default();
    let ui_theme = UiThemeService::default();
    let scene = amigo_scene::SceneService::default();
    let dev_console_state = DevConsoleState::default();
    let dev_console_completion = crate::dev_console::completion::ConsoleCompletionState::default();
    let debug_overlay_service = crate::debug_overlay::DebugOverlayService::default();
    let post_fx_service = amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService::default();
    let ui_viewport_state = amigo_runtime_bundles::amigo_ui::UiInputViewportState::default();
    debug_overlay_service.set_enabled(true);

    let runtime = amigo_runtime::RuntimeBuilder::default()
        .with_service(scene)
        .unwrap()
        .with_service(tilemaps)
        .unwrap()
        .with_service(sprites)
        .unwrap()
        .with_service(layered_images)
        .unwrap()
        .with_service(render_layers)
        .unwrap()
        .with_service(light_routes)
        .unwrap()
        .with_service(global_lights)
        .unwrap()
        .with_service(lightmaps)
        .unwrap()
        .with_service(light_groups)
        .unwrap()
        .with_service(text2d)
        .unwrap()
        .with_service(vectors)
        .unwrap()
        .with_service(particles)
        .unwrap()
        .with_service(meshes)
        .unwrap()
        .with_service(materials)
        .unwrap()
        .with_service(text3d)
        .unwrap()
        .with_service(ui_scene)
        .unwrap()
        .with_service(ui_state)
        .unwrap()
        .with_service(ui_theme)
        .unwrap()
        .with_service(post_fx_service)
        .unwrap()
        .with_service(dev_console_state)
        .unwrap()
        .with_service(dev_console_completion)
        .unwrap()
        .with_service(debug_overlay_service)
        .unwrap()
        .with_service(ui_viewport_state)
        .unwrap()
        .build();

    let packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(&runtime);

    assert_eq!(packet.debug_overlay().len(), 1);
    assert_eq!(packet.debug_overlay()[0].entity_name, "debug-overlay");
}

#[test]
fn composition_plan_puts_debug_after_game_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["world", "game_ui", "debug_overlay", "present"]);
}

#[test]
fn composition_orders_game_ui_before_debug_overlay() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["world", "game_ui", "debug_overlay", "present"]);
}

#[test]
fn composition_places_wet_reflections_between_world_and_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stack(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack::single(
        amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::WetReflections(
            amigo_runtime_bundles::amigo_2d_post_fx::PostFxWetReflections2d {
                reflection_mask: "they-are-rotten/layered-images/neon-alley/reflection_mask.png"
                    .to_owned(),
                edge_map: Some(
                    "they-are-rotten/layered-images/neon-alley/edge_map_2.png".to_owned(),
                ),
                ..Default::default()
            },
        ),
    ));
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:wet_reflections#0",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn composition_places_post_fx_before_game_and_debug_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stack(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack::single(
        amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::Blur(amigo_runtime_bundles::amigo_2d_post_fx::PostFxBlur2d::default()),
    ));
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:blur#0",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn composition_plan_inserts_post_fx_between_world_and_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_sprite(SpriteDrawCommand {
        entity_id: SceneEntityId::new(77),
        entity_name: "marker".to_owned(),
        sprite: Sprite {
            texture: AssetKey::new("debug/marker"),
            size: Vec2::new(16.0, 16.0),
            sheet: None,
            sheet_is_explicit: false,
            animation_override: None,
            frame_index: 0,
            frame_elapsed: 0.0,
        },
        transform: Transform2::default(),
        render_layer: "default".to_owned(),
        z_index: 0.0,
    });
    packet.set_post_fx_stack(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack::single(
        amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::LensDroplets(amigo_runtime_bundles::amigo_2d_post_fx::PostFxLensDroplets2d::default()),
    ));
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:lens_droplets#0",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn build_frame_graph_from_plan_tracks_composition_nodes() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let graph = build_frame_graph_from_plan(
        &plan,
        AppFrameGraphBuildInfo {
            width: 1280,
            height: 720,
        },
    );

    assert_eq!(
        graph.node_labels(),
        vec!["world", "game_ui", "debug_overlay", "present"]
    );
}

#[test]
fn composition_always_creates_world_base_before_ui_only_frame() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["world", "game_ui", "present"]);
}

#[test]
fn composition_default_packet_uses_world_base_before_present() {
    let packet = WgpuRenderFramePacket::default();

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["world", "present"]);

    let present = plan
        .views
        .first()
        .and_then(|view| view.passes.last())
        .expect("present pass");

    match present {
        amigo_render_api::RenderPassPlan::Present(pass) => {
            assert_eq!(pass.input, amigo_render_api::RenderPassInput::WorldColor);
        }
        other => panic!("expected present pass, got {:?}", other),
    }

    let graph = build_frame_graph_from_plan(
        &plan,
        AppFrameGraphBuildInfo {
            width: 1280,
            height: 720,
        },
    );

    assert_eq!(graph.node_labels(), vec!["world", "present"]);

    let surface = graph
        .resources
        .iter()
        .find(|resource| resource.label == "surface")
        .expect("surface resource")
        .id;

    for node in graph.nodes.iter().filter(|node| node.label != "present") {
        assert!(
            !node.writes.contains(&surface),
            "non-present node '{}' writes surface",
            node.label
        );
    }
}

#[test]
fn composition_preserves_original_postfx_effect_index() {
    let mut stack = amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack::default();

    let mut inactive = amigo_runtime_bundles::amigo_2d_post_fx::PostFxBlur2d::default();
    inactive.intensity = 0.0;

    stack
        .effects
        .push(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::Blur(inactive));
    stack.effects.push(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::LensDroplets(
        amigo_runtime_bundles::amigo_2d_post_fx::PostFxLensDroplets2d::default(),
    ));

    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stack(stack);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert!(
        labels.contains(&"post_fx:lens_droplets#1".to_owned()),
        "expected original stack index in labels, got {:?}",
        labels
    );
}

#[test]
fn graph_non_present_nodes_do_not_write_surface() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = AppFrameCompositionBuilder::build(&packet);
    let graph = build_frame_graph_from_plan(
        &plan,
        AppFrameGraphBuildInfo {
            width: 1280,
            height: 720,
        },
    );

    let surface = graph
        .resources
        .iter()
        .find(|resource| resource.label == "surface")
        .expect("surface resource")
        .id;

    for node in graph.nodes.iter().filter(|node| node.label != "present") {
        assert!(
            !node.writes.contains(&surface),
            "non-present node '{}' writes surface",
            node.label
        );
    }
}

#[test]
fn rebuilds_vector_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_vector(VectorShape2dDrawCommand {
        entity_id: SceneEntityId::new(9),
        entity_name: "asteroid".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polygon {
                points: vec![
                    Vec2::new(-8.0, 0.0),
                    Vec2::new(0.0, 8.0),
                    Vec2::new(8.0, 0.0),
                ],
            },
            style: VectorStyle2d::default(),
        },
        render_layer: "default".to_owned(),
        z_index: 1.0,
        transform: Transform2::default(),
    });

    let rebuilt = build_vector_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "asteroid");
}

#[test]
fn rebuilds_sprite_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_sprite(SpriteDrawCommand {
        entity_id: SceneEntityId::new(3),
        entity_name: "coin".to_owned(),
        sprite: Sprite {
            texture: AssetKey::new("playground-sidescroller/spritesheets/coin"),
            size: Vec2::new(16.0, 16.0),
            sheet: Some(SpriteSheet {
                columns: 4,
                rows: 1,
                frame_count: 4,
                frame_size: Vec2::new(16.0, 16.0),
                fps: 8.0,
                looping: true,
            }),
            sheet_is_explicit: false,
            animation_override: None,
            frame_index: 1,
            frame_elapsed: 0.0,
        },
        render_layer: "default".to_owned(),
        z_index: 0.0,
        transform: Transform2::default(),
    });

    let rebuilt = build_sprite_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "coin");
    assert_eq!(rebuilt.commands()[0].sprite.frame_index, 1);
}

#[test]
fn rebuilds_text2d_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_text(Text2dDrawCommand {
        entity_id: SceneEntityId::new(4),
        entity_name: "caption".to_owned(),
        text: Text2d {
            content: "Vector Demo".to_owned(),
            font: AssetKey::new("playground-2d/fonts/debug-ui"),
            bounds: Vec2::new(240.0, 48.0),
            transform: Transform2::default(),
        },
        render_layer: "default".to_owned(),
        z_index: 0.0,
    });

    let rebuilt = build_text2d_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "caption");
    assert_eq!(rebuilt.commands()[0].text.content, "Vector Demo");
}

#[test]
fn rebuilds_tilemap_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_2d_tilemap(TileMap2dDrawCommand {
        entity_id: SceneEntityId::new(12),
        entity_name: "tilemap".to_owned(),
        tilemap: TileMap2d {
            tileset: AssetKey::new(
                "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
            ),
            ruleset: None,
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["....".to_owned(), ".##.".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        },
        render_layer: "default".to_owned(),
        z_index: 0.0,
    });

    let rebuilt = build_tilemap_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "tilemap");
    assert_eq!(rebuilt.commands()[0].tilemap.grid.len(), 2);
}

#[test]
fn rebuilds_text3d_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_3d_text(Text3dDrawCommand {
        entity_id: SceneEntityId::new(16),
        entity_name: "caption-3d".to_owned(),
        text: Text3d {
            content: "AMIGO 3D".to_owned(),
            font: AssetKey::new("playground-3d/fonts/debug-3d"),
            size: 0.75,
            transform: Transform3::default(),
        },
    });

    let rebuilt = build_text3d_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "caption-3d");
    assert_eq!(rebuilt.commands()[0].text.content, "AMIGO 3D");
}

#[test]
fn rebuilds_mesh_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_3d_mesh(MeshDrawCommand {
        entity_id: SceneEntityId::new(18),
        entity_name: "probe-mesh".to_owned(),
        mesh: Mesh3d {
            mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
            transform: Transform3::default(),
        },
    });

    let rebuilt = build_mesh_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "probe-mesh");
}

#[test]
fn rebuilds_material_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_3d_material(MaterialDrawCommand {
        entity_id: SceneEntityId::new(19),
        entity_name: "probe-material".to_owned(),
        material: Material3d {
            label: "debug-surface".to_owned(),
            albedo: ColorRgba::WHITE,
            source: Some(AssetKey::new("playground-3d/materials/debug-surface")),
        },
    });

    let rebuilt = build_material_scene_service_from_packet(&packet);

    assert_eq!(rebuilt.commands().len(), 1);
    assert_eq!(rebuilt.commands()[0].entity_name, "probe-material");
    assert_eq!(rebuilt.commands()[0].material.label, "debug-surface");
}

#[test]
fn render_extractor_registry_stays_split_into_three_groups() {
    let source = include_str!("../../../../runtime/bundles/src/wgpu_render_extractors/mod.rs");

    assert!(source.contains("world_2d::register_world_2d_render_extractors"));
    assert!(source.contains("world_3d::register_world_3d_render_extractors"));
    assert!(source.contains("host_overlay::register_host_overlay_render_extractors"));
}

#[test]
fn render_runtime_no_longer_references_deleted_micro_extractor_modules() {
    let source = include_str!("../render_runtime.rs");

    for banned in [
        "extractors_world_2d_basic",
        "extractors_world_2d_fx",
        "extractors_world_2d_text",
        "extractors_world_2d_sprite",
        "extractors_world_2d_tilemap",
        "extractors_world_2d_layered_image",
        "extractors_world_2d_vector",
        "extractors_world_2d_composition",
        "extractors_world_2d_lighting",
        "extractors_world_2d_particles",
        "extractors_world_2d_postfx",
        "extractors_world_3d_mesh",
        "extractors_world_3d_material",
        "extractors_world_3d_text",
    ] {
        assert!(
            !source.contains(banned),
            "render_runtime.rs should not reference deleted micro-module `{banned}`",
        );
    }
}

#[test]
fn render_runtime_uses_only_frame_graph_render_flow() {
    let source = include_str!("../render_runtime.rs");

    for required in [
        "amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all",
        "AppFrameCompositionBuilder::build(&render_packet)",
        "build_frame_graph_from_plan(",
        "renderer.render_frame_request(render_request)?",
    ] {
        assert!(
            source.contains(required),
            "render_runtime.rs should contain `{required}`",
        );
    }

    for banned in [
        "render_frame_request_legacy",
        "LegacyComposite",
        "SplitPassExperimental",
        "render_scene_with_ui_primitives_and_3d_commands",
    ] {
        assert!(
            !source.contains(banned),
            "render_runtime.rs should not contain legacy render symbol `{banned}`",
        );
    }
}




