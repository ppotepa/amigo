use amigo_assets::AssetKey;
use amigo_2d_composition::{LightRoute2dSceneService, RenderLayer2dSceneService};
use amigo_layered_image_2d_plugin::LayeredImageSceneService;
use amigo_light_2d_plugin::{
    GlobalLight2dSceneService, LightGroup2dSceneService, LightMap2dSceneService,
    Material2dLightingMode,
};
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2};
use amigo_render_api::{
    post_fx_blur, post_fx_crt, post_fx_dirty_bloom, post_fx_film_noise, post_fx_lens_droplets,
    post_fx_wet_reflections, Crt2d, DirtyBloom2d, FilmNoise2d, PostFx2dStack, PostFxBlur2d,
    PostFxLensDroplets2d, PostFxWetReflections2d, ScopedPostFx2dStack,
};
use amigo_material_api::MaterialCoverageKind2d;
use amigo_3d_material::{Material3d, MaterialDrawCommand, MaterialSceneService};
use amigo_3d_mesh::{Mesh3d, MeshDrawCommand, MeshSceneService};
use amigo_3d_text::{Text3d, Text3dDrawCommand, Text3dSceneService};
use amigo_composite_plugin::PostFx2dService;
use amigo_focus_depth_plugin::DepthMap2dSceneService;
use amigo_particles_2d_plugin::{
    Particle2dEmitterRuntimeInput, Particle2dSceneService, ParticleAlignMode2d,
    ParticleBlendMode2d, ParticleEmitter2d, ParticleEmitter2dCommand, ParticleLineAnchor2d,
    ParticleMaterial2d, ParticleShape2d, ParticleSimulationSpace2d, ParticleSpawnArea2d,
    ParticleVelocityMode2d,
};
use amigo_scene::SceneEntityId;
use amigo_scripting_api::DevConsoleState;
use amigo_sprite_2d_plugin::{Sprite, SpriteDrawCommand, SpriteSceneService, SpriteSheet};
use amigo_text_2d_plugin::{Text2d, Text2dDrawCommand, Text2dSceneService, Text2dStyle};
use amigo_tilemap_2d_plugin::{TileMap2d, TileMap2dDrawCommand, TileMap2dSceneService};
use amigo_ui::{
    UiDocument as RuntimeUiDocument, UiDrawCommand, UiInputViewportState,
    UiLayer as RuntimeUiLayer, UiNode as RuntimeUiNode, UiNodeKind as RuntimeUiNodeKind,
    UiSceneService, UiStateService, UiStyle as RuntimeUiStyle, UiTarget as RuntimeUiTarget,
    UiTheme, UiThemePalette, UiThemeService,
};
use amigo_vector_2d_plugin::{
    VectorSceneService, VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d,
    VectorStyle2d, VectorViewportFit2d,
};

use super::*;

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
            visual_maps: None,
        },
        render_layer: "default".to_owned(),
        z_index: 1.0,
        transform: Transform2::default(),
        material: None,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
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
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
    });
    let particles = Particle2dSceneService::default();
    particles.queue_emitter(ParticleEmitter2dCommand {
        entity_id: SceneEntityId::new(14),
        entity_name: "spark".to_owned(),
        emitter: ParticleEmitter2d {
            attached_to: None,
            local_offset: Vec2::ZERO,
            local_direction_radians: 0.0,
            spawn_area: ParticleSpawnArea2d::Point,
            active: true,
            spawn_rate: 1.0,
            max_particles: 4,
            particle_lifetime: 1.0,
            lifetime_jitter: 0.0,
            initial_speed: 0.0,
            speed_jitter: 0.0,
            spread_radians: 0.0,
            inherit_parent_velocity: 0.0,
            velocity_mode: ParticleVelocityMode2d::Free,
            simulation_space: ParticleSimulationSpace2d::World,
            initial_size: 2.0,
            final_size: 2.0,
            size_jitter: 0.0,
            color: ColorRgba::WHITE,
            color_ramp: None,
            render_layer: "default".to_owned(),
            z_index: 3.5,
            shape: ParticleShape2d::Circle { segments: 8 },
            shape_choices: Vec::new(),
            shape_over_lifetime: Vec::new(),
            line_anchor: ParticleLineAnchor2d::Center,
            align: ParticleAlignMode2d::Velocity,
            blend_mode: ParticleBlendMode2d::Alpha,
            motion_stretch: None,
            material: ParticleMaterial2d {
                lighting_mode: Material2dLightingMode::Unlit,
                receives_light: false,
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
        &[Particle2dEmitterRuntimeInput {
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
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
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
            style: Text2dStyle::default(),
            post_fx_host_id: None,
        },
        render_layer: "default".to_owned(),
        z_index: 0.0,
        material: None,
        render_contributions: amigo_render_api::RenderContributionSet::default(),
    });
    let text3d = Text3dSceneService::default();
    let layered_images = LayeredImageSceneService::default();
    let depth_maps = DepthMap2dSceneService::default();
    let global_lights = GlobalLight2dSceneService::default();
    let lightmaps = LightMap2dSceneService::default();
    let render_layers = RenderLayer2dSceneService::default();
    let light_routes = LightRoute2dSceneService::default();
    let light_groups = LightGroup2dSceneService::default();
    text3d.queue(Text3dDrawCommand {
        entity_id: 10,
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
        entity_id: 11,
        entity_name: "probe-mesh".to_owned(),
        mesh: Mesh3d {
            mesh_asset: AssetKey::new("playground-3d/meshes/probe"),
            transform: Transform3::default(),
        },
    });
    let materials = MaterialSceneService::default();
    materials.queue(MaterialDrawCommand {
        entity_id: 12,
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
    let dev_console_completion = amigo_devtools::ConsoleCompletionState::default();
    let debug_overlay_service = crate::debug_overlay::DebugOverlayService::default();
    let post_fx_service = PostFx2dService::default();
    let ui_viewport_state = UiInputViewportState::default();

    let runtime = amigo_runtime::RuntimeBuilder::default()
        .with_service(scene)
        .unwrap()
        .with_service(tilemaps)
        .unwrap()
        .with_service(sprites)
        .unwrap()
        .with_service(layered_images)
        .unwrap()
        .with_service(depth_maps)
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

    let packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(&runtime);

    assert_eq!(packet.renderable_2d_count_by_component_kind("TileMap2D"), 1);
    assert_eq!(packet.renderable_2d_count_by_component_kind("Sprite2D"), 1);
    assert_eq!(packet.renderable_2d_count_by_component_kind("Text2D"), 1);
    assert_eq!(
        packet.renderable_2d_count_by_component_kind("VectorShape2D"),
        1
    );
    assert_eq!(
        packet.renderable_2d_count_by_component_kind("ParticleEmitter2D"),
        1
    );
    let renderable_entities = packet
        .renderables_2d()
        .iter()
        .map(|item| item.owner_entity())
        .collect::<Vec<_>>();
    assert!(renderable_entities.contains(&"arena"));
    assert!(renderable_entities.contains(&"player"));
    assert!(renderable_entities.contains(&"label"));
    assert!(renderable_entities.contains(&"ship"));
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
    let layered_images = LayeredImageSceneService::default();
    let render_layers = RenderLayer2dSceneService::default();
    let light_routes = LightRoute2dSceneService::default();
    let global_lights = GlobalLight2dSceneService::default();
    let lightmaps = LightMap2dSceneService::default();
    let light_groups = LightGroup2dSceneService::default();
    let text2d = Text2dSceneService::default();
    let vectors = VectorSceneService::default();
    let particles = Particle2dSceneService::default();
    let depth_maps =
        DepthMap2dSceneService::default();
    let meshes = MeshSceneService::default();
    let materials = MaterialSceneService::default();
    let text3d = Text3dSceneService::default();
    let ui_scene = UiSceneService::default();
    let ui_state = UiStateService::default();
    let ui_theme = UiThemeService::default();
    let scene = amigo_scene::SceneService::default();
    let dev_console_state = DevConsoleState::default();
    let dev_console_completion = amigo_devtools::ConsoleCompletionState::default();
    let debug_overlay_service = crate::debug_overlay::DebugOverlayService::default();
    let post_fx_service = PostFx2dService::default();
    let ui_viewport_state = UiInputViewportState::default();
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
        .with_service(depth_maps)
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

    let packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(&runtime);

    assert_eq!(packet.debug_overlay().len(), 1);
    assert_eq!(packet.debug_overlay()[0].entity_name, "debug-overlay");
}

#[test]
fn composition_plan_puts_debug_after_game_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
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

    let plan = WgpuFrameCompositionBuilder::build(&packet);
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
    packet.set_post_fx_stacks(vec![
        ScopedPostFx2dStack::from_frame_stack(
            PostFx2dStack::single(post_fx_wet_reflections(
                PostFxWetReflections2d {
                    reflection_mask: "rotten-club/layered-images/neon-alley/reflection_mask.png"
                        .to_owned(),
                    edge_map: Some(
                        "rotten-club/layered-images/neon-alley/edge_map_2.png".to_owned(),
                    ),
                    ..Default::default()
                },
            )),
        ),
    ]);
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:frame:frame_fx_000:wet_reflections",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn composition_places_post_fx_before_game_and_debug_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![
        ScopedPostFx2dStack::from_frame_stack(
            PostFx2dStack::single(post_fx_blur(
                PostFxBlur2d::default(),
            )),
        ),
    ]);
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:frame:frame_fx_000:blur",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn composition_plan_inserts_post_fx_between_world_and_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_renderable_2d(amigo_render_api::Renderable2dItem::new(
        amigo_render_api::Renderable2dCommon::world(
            "marker",
            "Sprite2D",
            "default",
            0.0,
            amigo_render_api::Renderable2dKind::Sprite,
        ),
        amigo_render_api::RenderPrimitive2d::TexturedQuad(
            amigo_render_api::TexturedQuad2dPrimitive {
                texture: AssetKey::new("debug/marker"),
                size: Vec2::new(16.0, 16.0),
                transform: Transform2::default(),
                visual_maps: None,
                sheet: None,
                frame_index: 0,
                material: amigo_render_api::RenderMaterialBinding2d::none(
                    MaterialCoverageKind2d::TextureAlpha,
                ),
            },
        ),
    ));
    packet.set_post_fx_stacks(vec![
        ScopedPostFx2dStack::from_frame_stack(
            PostFx2dStack::single(post_fx_lens_droplets(
                PostFxLensDroplets2d::default(),
            )),
        ),
    ]);
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:frame:frame_fx_000:lens_droplets",
            "game_ui",
            "debug_overlay",
            "present"
        ]
    );
}

#[test]
fn composition_places_film_noise_before_game_ui() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![
        ScopedPostFx2dStack::from_frame_stack(
            PostFx2dStack {
                effects: vec![
                    post_fx_wet_reflections(PostFxWetReflections2d {
                        reflection_mask: "debug/mask.png".to_owned(),
                        ..Default::default()
                    }),
                    post_fx_dirty_bloom(DirtyBloom2d::default()),
                    post_fx_film_noise(FilmNoise2d {
                        iso: 3200.0,
                        ..Default::default()
                    }),
                    post_fx_crt(Crt2d::default()),
                ],
            },
        ),
    ]);
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        vec![
            "world",
            "post_fx:frame:frame_fx_000:wet_reflections",
            "post_fx:frame:frame_fx_001:dirty_bloom",
            "post_fx:frame:frame_fx_002:film_noise",
            "post_fx:frame:frame_fx_003:crt",
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

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let graph = build_frame_graph_from_plan(
        &plan,
        FrameGraphBuildInfo {
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

    let plan = WgpuFrameCompositionBuilder::build(&packet);
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

    let plan = WgpuFrameCompositionBuilder::build(&packet);
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
        FrameGraphBuildInfo {
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
fn composition_preserves_original_postfx_effect_order_in_labels() {
    let mut stack = PostFx2dStack::default();

    let mut inactive = PostFxBlur2d::default();
    inactive.intensity = 0.0;

    stack.effects.push(post_fx_blur(inactive));
    stack.effects.push(post_fx_lens_droplets(
        PostFxLensDroplets2d::default(),
    ));

    let mut packet = WgpuRenderFramePacket::default();
    packet.set_post_fx_stacks(vec![
        ScopedPostFx2dStack::from_frame_stack(stack),
    ]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let labels = plan.views[0]
        .passes
        .iter()
        .map(|pass| pass.label())
        .collect::<Vec<_>>();

    assert!(
        labels.contains(&"post_fx:frame:frame_fx_001:lens_droplets".to_owned()),
        "expected original stack index in labels, got {:?}",
        labels
    );
}

#[test]
fn graph_non_present_nodes_do_not_write_surface() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build(&packet);
    let graph = build_frame_graph_from_plan(
        &plan,
        FrameGraphBuildInfo {
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
fn editor_frame_graph_renders_game_to_logical_target_and_presents_last() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.extend_game_ui_overlay([test_overlay_document("game")]);
    packet.extend_debug_overlay([test_overlay_document("debug")]);

    let plan = WgpuFrameCompositionBuilder::build_with_options(
        &packet,
        WgpuFrameCompositionOptions {
            debug_overlay_after_present: true,
        },
    );
    let graph = build_frame_graph_from_plan(
        &plan,
        FrameGraphBuildInfo {
            width: 1280,
            height: 720,
        },
    );
    let labels = graph.node_labels();

    assert!(labels.contains(&"world"));
    assert!(labels.contains(&"present"));
    assert!(!labels.contains(&"debug_overlay"));
    assert_eq!(labels.last().copied(), Some("present"));

    let texture_sizes = graph
        .resources
        .iter()
        .filter_map(|resource| match resource.kind {
            amigo_render_api::FrameResourceKind::TextureColor { width, height, .. } => {
                Some((width, height))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(texture_sizes.iter().all(|size| *size == (1280, 720)));
}

#[test]
fn rebuilds_vector_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_renderable_2d(amigo_render_api::Renderable2dItem::new(
        amigo_render_api::Renderable2dCommon::world(
            "asteroid",
            "VectorShape2D",
            "default",
            1.0,
            amigo_render_api::Renderable2dKind::Vector,
        ),
        amigo_render_api::RenderPrimitive2d::VectorMesh(amigo_render_api::VectorShape2dPrimitive {
            shape: amigo_render_api::VectorShape2dKindPrimitive::Polygon {
                points: vec![
                    Vec2::new(-8.0, 0.0),
                    Vec2::new(0.0, 8.0),
                    Vec2::new(8.0, 0.0),
                ],
            },
            style: amigo_render_api::VectorShape2dStylePrimitive {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 1.0,
                fill_color: None,
            },
            transform: Transform2::default(),
            viewport_fit: amigo_render_api::VectorShape2dViewportFit::Fixed,
            viewport_canvas_size: None,
            material: amigo_render_api::RenderMaterialBinding2d::none(
                MaterialCoverageKind2d::VectorCoverage,
            ),
        }),
    ));

    assert_eq!(packet.renderables_2d().len(), 1);
    assert_eq!(packet.renderables_2d()[0].owner_entity(), "asteroid");
}

#[test]
fn rebuilds_sprite_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_renderable_2d(amigo_render_api::Renderable2dItem::new(
        amigo_render_api::Renderable2dCommon::world(
            "coin",
            "Sprite2D",
            "default",
            0.0,
            amigo_render_api::Renderable2dKind::Sprite,
        ),
        amigo_render_api::RenderPrimitive2d::TexturedQuad(
            amigo_render_api::TexturedQuad2dPrimitive {
                texture: AssetKey::new("playground-sidescroller/spritesheets/coin"),
                size: Vec2::new(16.0, 16.0),
                transform: Transform2::default(),
                visual_maps: None,
                sheet: Some(amigo_render_api::TexturedQuad2dSheet {
                    columns: 4,
                    rows: 1,
                    frame_count: 4,
                    frame_size: Vec2::new(16.0, 16.0),
                }),
                frame_index: 1,
                material: amigo_render_api::RenderMaterialBinding2d::none(
                    MaterialCoverageKind2d::TextureAlpha,
                ),
            },
        ),
    ));

    assert_eq!(packet.renderables_2d().len(), 1);
    assert_eq!(packet.renderables_2d()[0].owner_entity(), "coin");
    match &packet.renderables_2d()[0].primitive {
        amigo_render_api::RenderPrimitive2d::TexturedQuad(quad) => assert_eq!(quad.frame_index, 1),
        other => panic!("expected textured quad, got {other:?}"),
    }
}

#[test]
fn rebuilds_text2d_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_renderable_2d(amigo_render_api::Renderable2dItem::new(
        amigo_render_api::Renderable2dCommon::world(
            "caption",
            "Text2D",
            "default",
            0.0,
            amigo_render_api::Renderable2dKind::Text,
        ),
        amigo_render_api::RenderPrimitive2d::GlyphRun(amigo_render_api::GlyphRun2dPrimitive {
            font: AssetKey::new("playground-2d/fonts/debug-ui"),
            text: "Vector Demo".to_owned(),
            bounds: Vec2::new(240.0, 48.0),
            transform: Transform2::default(),
            color: ColorRgba::WHITE,
            font_size: None,
            blend: amigo_render_api::GlyphRun2dBlendMode::Alpha,
            shadow: None,
            outline: None,
            glow: None,
            material: amigo_render_api::RenderMaterialBinding2d::none(
                MaterialCoverageKind2d::Glyphs,
            ),
        }),
    ));

    assert_eq!(packet.renderables_2d().len(), 1);
    assert_eq!(packet.renderables_2d()[0].owner_entity(), "caption");
    match &packet.renderables_2d()[0].primitive {
        amigo_render_api::RenderPrimitive2d::GlyphRun(text) => {
            assert_eq!(text.text, "Vector Demo")
        }
        other => panic!("expected glyph run, got {other:?}"),
    }
}

#[test]
fn rebuilds_tilemap_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_renderable_2d(amigo_render_api::Renderable2dItem::new(
        amigo_render_api::Renderable2dCommon::world(
            "tilemap",
            "TileMap2D",
            "default",
            0.0,
            amigo_render_api::Renderable2dKind::TileMap,
        ),
        amigo_render_api::RenderPrimitive2d::TileBatch(amigo_render_api::TileMap2dPrimitive {
            tileset: AssetKey::new(
                "playground-sidescroller/spritesheets/platformer/tilesets/platform/base",
            ),
            tile_size: Vec2::new(16.0, 16.0),
            grid: vec!["....".to_owned(), ".##.".to_owned()],
            origin_offset: Vec2::new(0.0, 0.0),
            resolved: None,
        }),
    ));

    assert_eq!(packet.renderables_2d().len(), 1);
    assert_eq!(packet.renderables_2d()[0].owner_entity(), "tilemap");
    match &packet.renderables_2d()[0].primitive {
        amigo_render_api::RenderPrimitive2d::TileBatch(tilemap) => {
            assert_eq!(tilemap.grid.len(), 2)
        }
        other => panic!("expected tilemap primitive, got {other:?}"),
    }
}

#[test]
fn rebuilds_text3d_scene_service_from_packet() {
    let mut packet = WgpuRenderFramePacket::default();
    packet.push_world_3d_text(Text3dDrawCommand {
        entity_id: 16,
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
        entity_id: 18,
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
        entity_id: 19,
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
    let source = include_str!("../../../../runtime/bundles/src/render_extractor_bridges/mod.rs");

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
        "amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(runtime)",
        "WgpuFrameCompositionBuilder::build(&render_packet)",
        "build_frame_graph_from_plan(",
        "renderer.render_frame_request(render_request)?",
    ] {
        assert!(
            source.contains(required),
            "render_runtime.rs should contain `{required}`",
        );
    }

    for banned in [
        "render_frame_request_retired",
        "LegacyComposite",
        "SplitPassExperimental",
        "render_scene_with_ui_primitives_and_3d_commands",
    ] {
        assert!(
            !source.contains(banned),
            "render_runtime.rs should not contain retired render symbol `{banned}`",
        );
    }
}
