use std::path::PathBuf;

use amigo_math::{ColorRgba, Transform3, Vec3};

use super::super::{
    build_scene_hydration_plan, entity_selector_from_document, scene_key_from_document,
};
use crate::{
    CameraController3dModeSceneCommand, CameraController3dSceneCommand, ComponentHydratorRegistry,
    ComponentSchemaRegistry, EntitySelector, InputActionMapSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator, SceneCommand, SceneComponentPayload,
    SceneComponentSchemaProvider, SceneDocumentError, SceneEntitySelectorDocument,
    SceneEntitySelectorKindDocument, compile_scene_document_from_path, load_scene_document_from_path,
    load_scene_document_from_str, load_scene_document_from_str_with_component_schemas,
};
use serde_yaml::{Mapping, Value};

#[derive(Debug)]
struct TestPluginSpritePayload {
    texture: String,
}

impl SceneComponentPayload for TestPluginSpritePayload {
    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Copy)]
struct TestPluginSpriteSchemaProvider;

impl SceneComponentSchemaProvider for TestPluginSpriteSchemaProvider {
    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["Sprite2D"]
    }

    fn parse_yaml(&self, payload: Mapping) -> Result<Value, serde_yaml::Error> {
        Ok(Value::Mapping(payload))
    }

    fn parse_payload_value(
        &self,
        payload: &Value,
    ) -> crate::SceneDocumentResult<Box<dyn SceneComponentPayload>> {
        let mapping = payload
            .as_mapping()
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "expected sprite payload mapping".to_owned(),
            })?;
        let texture = mapping
            .get(Value::String("texture".to_owned()))
            .and_then(Value::as_str)
            .ok_or_else(|| SceneDocumentError::Compile {
                path: None,
                message: "missing sprite texture".to_owned(),
            })?;
        Ok(Box::new(TestPluginSpritePayload {
            texture: texture.to_owned(),
        }))
    }
}

struct TestPluginSpriteHydrator;

impl PluginComponentHydrator for TestPluginSpriteHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.test.sprite"
    }

    fn component_type(&self) -> &'static str {
        "amigo.test.Sprite2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> crate::SceneDocumentResult<()> {
        let Some(payload) = ctx
            .payload
            .as_any()
            .downcast_ref::<TestPluginSpritePayload>()
        else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "wrong sprite payload".to_owned(),
            });
        };
        ctx.commands.push(SceneCommand::SpawnNamedEntity {
            name: format!("plugin-hydrated:{}", payload.texture),
            transform: None,
        });
        Ok(())
    }
}

fn sprite_command(command: &SceneCommand) -> Option<&crate::Sprite2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Sprite2dSceneCommand>(),
        _ => None,
    }
}

fn text_command(command: &SceneCommand) -> Option<&crate::Text2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Text2dSceneCommand>(),
        _ => None,
    }
}

fn vector_command(command: &SceneCommand) -> Option<&crate::VectorShape2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::VectorShape2dSceneCommand>()
        }
        _ => None,
    }
}

fn camera_command(command: &SceneCommand) -> Option<&crate::Camera2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Camera2dSceneCommand>(),
        _ => None,
    }
}

fn beacon_command(command: &SceneCommand) -> Option<&crate::BeaconLight2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::BeaconLight2dSceneCommand>()
        }
        _ => None,
    }
}

fn layered_image_command(command: &SceneCommand) -> Option<&crate::LayeredImage2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::LayeredImage2dSceneCommand>()
        }
        _ => None,
    }
}

fn collision_event_rule_command(
    command: &SceneCommand,
) -> Option<&crate::CollisionEventRule2dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::CollisionEventRule2dSceneCommand>()
        }
        _ => None,
    }
}

fn mesh_command(command: &SceneCommand) -> Option<&crate::Mesh3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Mesh3dSceneCommand>(),
        _ => None,
    }
}

fn npr_preset_command(command: &SceneCommand) -> Option<&crate::NprPreset3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::NprPreset3dSceneCommand>()
        }
        _ => None,
    }
}

fn material_command(command: &SceneCommand) -> Option<&crate::Material3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Material3dSceneCommand>(),
        _ => None,
    }
}

fn text3d_command(command: &SceneCommand) -> Option<&crate::Text3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::Text3dSceneCommand>(),
        _ => None,
    }
}

fn first_mesh_npr_settings(document_yaml: &str) -> amigo_render_api::NprLineSettings3d {
    let document = load_scene_document_from_str(document_yaml).expect("scene document should parse");
    let plan = build_scene_hydration_plan("playground-npr", &document)
        .expect("scene document should hydrate");
    plan.commands
        .iter()
        .find_map(mesh_command)
        .and_then(|command| command.npr.clone())
        .expect("mesh command with NPR settings should exist")
}

fn static_box_collider3d_command(
    command: &SceneCommand,
) -> Option<&crate::StaticBoxCollider3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::StaticBoxCollider3dSceneCommand>()
        }
        _ => None,
    }
}

fn physics_spawner3d_command(
    command: &SceneCommand,
) -> Option<&crate::PhysicsSpawner3dSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => {
            command.payload_as::<crate::PhysicsSpawner3dSceneCommand>()
        }
        _ => None,
    }
}

fn ui_command(command: &SceneCommand) -> Option<&crate::UiSceneCommand> {
    match command {
        SceneCommand::Plugin { command } => command.payload_as::<crate::UiSceneCommand>(),
        _ => None,
    }
}

#[test]
fn builds_hydration_plan_for_2d_scene_document() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
      rotation_radians: 0.5
      scale: { x: 2.0, y: 3.0 }
    components:
      - type: amigo.gfx.sprite-2d.Sprite2D
        texture: playground-2d/spritesheets/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

    assert_eq!(scene_key_from_document(&document).as_str(), "sprite-lab");
    assert_eq!(plan.commands.len(), 5);
    assert!(plan.commands.iter().any(|command| matches!(
        command,
        SceneCommand::SpawnNamedEntity {
            name,
            transform: Some(Transform3 { .. })
        } if name == "playground-2d-camera"
    )));
    assert!(
        plan.commands.iter().all(|command| {
            camera_command(command).is_none() && sprite_command(command).is_none()
        })
    );
}

#[test]
fn hydrates_schema_alias_through_canonical_plugin_payload_type() {
    let schemas = ComponentSchemaRegistry::default();
    schemas.register_schema_provider(TestPluginSpriteSchemaProvider);
    let hydrators = ComponentHydratorRegistry::default();
    hydrators.register_plugin(TestPluginSpriteHydrator);

    let document = load_scene_document_from_str_with_component_schemas(
        r#"
version: 1
scene:
  id: plugin-hydration-alias
entities:
  - id: sprite
    name: sprite
    components:
      - type: Sprite2D
        texture: plugin/sprite
        size: { x: 64.0, y: 64.0 }
"#,
        Some(&schemas),
    )
    .expect("scene document should parse");

    let plan = super::super::build_scene_hydration_plan_with_component_hydrators(
        "test",
        &document,
        Some(&hydrators),
        Some(&schemas),
    )
    .expect("plan should build");

    assert!(plan.commands.iter().any(|command| matches!(
        command,
        SceneCommand::SpawnNamedEntity { name, transform: None }
            if name == "plugin-hydrated:plugin/sprite"
    )));
}

#[test]
fn skips_text2d_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: text-material
entities:
  - id: title
    name: title
    components:
      - type: amigo.gfx.text-2d.Text2D
        content: ROTTEN CLUB
        font: rotten-club/fonts/game
        bounds: { x: 1180.0, y: 240.0 }
        material:
          optical:
            mode: refractive
            transmission: 0.58
            refraction_px: 4.5
          lighting:
            receives_light: true
            response: 0.35
"##,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("rotten-club", &document).expect("plan should build");

    assert!(
        plan.commands
            .iter()
            .all(|command| text_command(command).is_none())
    );
}

#[test]
fn skips_sprite2d_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: sprite-material
entities:
  - id: poster
    name: poster
    components:
      - type: amigo.gfx.sprite-2d.Sprite2D
        render_layer: foreground.props
        texture: test/poster
        size: [128, 128]
        render_contributions:
          material.mask: true
          optics.refract: true
        material:
          optical:
            mode: refractive
            transmission: 0.45
            refraction_px: 7.0
"##,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("test-mod", &document).expect("plan should build");

    assert!(
        plan.commands
            .iter()
            .all(|command| sprite_command(command).is_none())
    );
}

#[test]
fn skips_vector_shape_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: vector-material
entities:
  - id: vector-glass
    name: vector-glass
    components:
      - type: amigo.gfx.vector-2d.VectorShape2D
        render_layer: foreground.props
        kind: circle
        radius: 48.0
        fill_color: "#FFFFFFFF"
        render_contributions:
          material.mask: true
          optics.refract: true
        material:
          optical:
            mode: refractive
            transmission: 0.35
            refraction_px: 5.0
"##,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("test-mod", &document).expect("plan should build");

    assert!(
        plan.commands
            .iter()
            .all(|command| vector_command(command).is_none())
    );
}

#[test]
fn skips_camera2d_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: camera-contributions
  label: Camera Contributions
entities:
  - id: camera
    name: camera
    components:
      - type: Camera2D
        render_contributions:
          camera.film: true
          camera.scan_output: false
"#,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("test-mod", &document).expect("plan should build");
    assert!(
        plan.commands
            .iter()
            .all(|command| camera_command(command).is_none())
    );
}

#[test]
fn skips_beacon2d_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: beacon-contributions
  label: Beacon Contributions
entities:
  - id: beacon
    name: beacon
    components:
      - type: BeaconLight2D
        id: beacon
        render_contributions:
          overlay.visible: false
          relight.plate: true
          bloom.source: false
"##,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("test-mod", &document).expect("plan should build");
    assert!(
        plan.commands
            .iter()
            .all(|command| beacon_command(command).is_none())
    );
}

#[test]
fn skips_layered_image_2d_without_plugin_hydrator() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: main-menu
entities:
  - id: bg
    name: main-menu-background
    components:
      - type: LayeredImage2D
        asset: test-mod/layered-images/test-scene
        size: { x: 1280.0, y: 720.0 }
        base_opacity: 0.25
        z_index: -100.0
        layer_overrides:
          - id: accent_light
            opacity: 0.5
            blend: screen
"#,
    )
    .expect("scene document should parse");

    let plan = build_scene_hydration_plan("test-mod", &document).expect("plan should build");

    assert!(
        plan.commands
            .iter()
            .all(|command| layered_image_command(command).is_none())
    );
}

#[test]
fn builds_hydration_plan_for_entity_metadata() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy]
    groups: [wave-1]
    visible: false
    collision_enabled: false
    properties:
      score_value: 100
      label: scout
"#,
    )
    .expect("scene document should parse");

    let plan =
        build_scene_hydration_plan("metadata-preview", &document).expect("plan should build");

    assert!(plan.commands.iter().any(|command| matches!(
        command,
        SceneCommand::ConfigureEntity {
            entity_name,
            lifecycle,
            tags,
            groups,
            properties,
        } if entity_name == "actor"
            && !lifecycle.visible
            && lifecycle.simulation_enabled
            && !lifecycle.collision_enabled
            && tags == &vec!["enemy".to_owned()]
            && groups == &vec!["wave-1".to_owned()]
            && properties.contains_key("score_value")
            && properties.contains_key("label")
    )));
}

#[test]
fn converts_selector_documents_to_runtime_selectors() {
    let cases = [
        (
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Entity,
                value: "player".to_owned(),
            },
            EntitySelector::Entity("player".to_owned()),
        ),
        (
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Tag,
                value: "enemy".to_owned(),
            },
            EntitySelector::Tag("enemy".to_owned()),
        ),
        (
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Group,
                value: "wave-1".to_owned(),
            },
            EntitySelector::Group("wave-1".to_owned()),
        ),
        (
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Pool,
                value: "bullets".to_owned(),
            },
            EntitySelector::Pool("bullets".to_owned()),
        ),
    ];

    for (document, expected) in cases {
        assert_eq!(entity_selector_from_document(&document), expected);
        assert_eq!(EntitySelector::from(document), expected);
    }
}

#[test]
fn builds_hydration_plan_for_collision_event_rules() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: collision-preview
collision_events:
  - id: projectile-hits-target
    source:
      kind: tag
      value: projectile
    target:
      kind: group
      value: targets
    event: collision.hit
    once_per_overlap: true
entities: []
"#,
    )
    .expect("scene document should parse");

    let plan =
        build_scene_hydration_plan("collision-preview", &document).expect("plan should build");

    assert!(plan.commands.iter().any(|command| {
        collision_event_rule_command(command).is_some_and(|command| {
            command.id == "projectile-hits-target"
                && command.source == EntitySelector::Tag("projectile".to_owned())
                && command.target == EntitySelector::Group("targets".to_owned())
                && command.event == "collision.hit"
                && command.once_per_overlap
        })
    }));
}

#[test]
fn builds_hydration_plan_for_material_scene_document() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
    )
    .expect("material lab scene should parse");

    let plan = build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

    assert!(plan.commands.iter().any(|command| matches!(
        command,
        SceneCommand::SpawnNamedEntity {
            name,
            transform: Some(Transform3 { translation, scale, .. })
        } if name == "playground-3d-material-probe"
            && *translation == Vec3::ZERO
            && *scale == Vec3::ONE
    )));
    assert!(plan.commands.iter().any(|command| {
        material_command(command).is_some_and(|command| {
            command.entity_name == "playground-3d-material-probe"
                && command.label == "debug-surface"
                && command.albedo == ColorRgba::WHITE
        })
    }));
}

#[test]
fn builds_hydration_plan_for_playground_2d_main_scene() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-2d/scenes/hello-world-spritesheet/scene.yml"),
    )
    .expect("playground 2d main scene should parse");

    let plan = build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

    assert!(
        plan.commands.iter().all(|command| {
            sprite_command(command).is_none() && text_command(command).is_none()
        })
    );
}

#[test]
fn builds_hydration_plan_for_playground_3d_main_scene() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-3d/scenes/hello-world-cube/scene.yml"),
    )
    .expect("playground 3d main scene should parse");

    let plan = build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

    assert!(plan.commands.iter().any(|command| {
        mesh_command(command).is_some_and(|command| command.entity_name == "playground-3d-cube")
    }));
    assert!(plan.commands.iter().any(|command| {
        material_command(command).is_some_and(|command| command.entity_name == "playground-3d-cube")
    }));
    assert!(plan.commands.iter().any(|command| {
        text3d_command(command).is_some_and(|command| command.entity_name == "playground-3d-hello")
    }));
}

#[test]
fn builds_hydration_plan_for_playground_npr_mesh_switches() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-npr/scenes/comic-lines/scene.yml"),
    )
    .expect("playground npr scene should parse");

    let plan = build_scene_hydration_plan("playground-npr", &document).expect("plan should build");

    let input_map = plan
        .commands
        .iter()
        .find_map(super::plugin_payload::<InputActionMapSceneCommand>)
        .expect("npr input action map should be present");
    assert_eq!(input_map.id, "playground-npr");
    for index in 1..=2 {
        assert!(
            input_map
                .actions
                .contains_key(&format!("npr.select_{index}")),
            "missing npr select action for slot {index}"
        );
    }
    for index in 3..=6 {
        assert!(
            !input_map
                .actions
                .contains_key(&format!("npr.select_{index}")),
            "old npr select action for slot {index} should not be present"
        );
    }
    for action in [
        "npr.animation_previous",
        "npr.animation_next",
        "npr.camera_toggle",
        "npr.model_autorotate_toggle",
        "npr.preset_previous",
        "npr.preset_next",
        "npr.fly_forward",
        "npr.fly_strafe",
        "npr.fly_lift",
    ] {
        assert!(
            input_map.actions.contains_key(action),
            "missing npr camera action {action}"
        );
    }

    let controller = plan
        .commands
        .iter()
        .find_map(super::plugin_payload::<CameraController3dSceneCommand>)
        .expect("npr camera controller should be present");
    assert_eq!(controller.camera, "playground-npr-camera");
    assert_eq!(controller.mode, CameraController3dModeSceneCommand::Orbit);
    assert_eq!(
        controller.switch_action.as_deref(),
        Some("npr.camera_toggle")
    );
    assert_eq!(controller.move_forward_action, "npr.fly_forward");

    let mesh_commands = plan
        .commands
        .iter()
        .filter_map(mesh_command)
        .collect::<Vec<_>>();
    assert_eq!(mesh_commands.len(), 2);

    let soldier_mesh = mesh_commands
        .iter()
        .copied()
        .find(|command| command.entity_name == "playground-npr-model-1-soldier")
        .expect("soldier mesh command should be present");
    let soldier_npr = soldier_mesh
        .npr
        .as_ref()
        .expect("npr settings block should enable NPR line settings");
    assert!(soldier_npr.boundary);
    assert!(soldier_npr.silhouette);
    assert!(soldier_npr.feature);
    assert_eq!(
        soldier_npr.style_preset,
        amigo_render_api::NprStylePreset3d::GpuStableComic
    );
    assert_eq!(
        soldier_npr.render_strategy,
        amigo_render_api::NprRenderStrategy3d::GpuRealtime
    );
    assert_eq!(soldier_npr.feature_angle_degrees, 42.0);
    assert_eq!(soldier_npr.humanization, 0.16);
    assert_eq!(soldier_npr.endpoint_snap_px, 1.1);
    assert_eq!(soldier_npr.endpoint_lock_start_px, 6.0);
    assert_eq!(soldier_npr.endpoint_lock_end_px, 7.0);
    assert_eq!(soldier_npr.tool_width_multiplier, 1.0);
    assert_eq!(soldier_npr.tool_alpha_multiplier, 1.0);
    assert_eq!(soldier_npr.tool_wobble_multiplier, 1.0);
    assert_eq!(soldier_npr.passes, 1);
    assert_eq!(soldier_npr.search_line_count, 0);
    assert_eq!(
        soldier_npr.gpu_realtime_tuning.debug_mode,
        amigo_render_api::NprGpuDebugMode3d::Final
    );
    assert!(!soldier_npr.gpu_realtime_tuning.search_enabled);
    assert_eq!(soldier_npr.gpu_realtime_tuning.max_chained_walk_edges, 0);
    assert!(soldier_npr.black_mass_material_ids.is_empty());
    assert!(soldier_npr.ink_detail_material_ids.is_empty());

    let khronos_male_mesh = mesh_commands
        .iter()
        .copied()
        .find(|command| command.entity_name == "playground-npr-model-2-khronos-male")
        .expect("khronos male mesh command should be present");
    let khronos_male_npr = khronos_male_mesh
        .npr
        .as_ref()
        .expect("npr settings block should enable NPR line settings");
    assert_eq!(
        khronos_male_mesh.mesh_asset.as_str(),
        "playground-npr/meshes/khronos-male"
    );
    assert_eq!(khronos_male_npr.feature_angle_degrees, 42.0);
    assert_eq!(khronos_male_npr.seed, 1202);
    assert_eq!(khronos_male_npr.passes, 1);
    assert_eq!(khronos_male_npr.search_line_count, 0);
    assert!(!khronos_male_npr.gpu_realtime_tuning.search_enabled);
    assert_eq!(
        khronos_male_npr.black_mass_material_ids,
        vec![4, 5, 6, 7, 11, 12, 13]
    );
    assert_eq!(
        khronos_male_npr.ink_detail_material_ids,
        vec![6, 7, 11, 12, 13]
    );
}

#[test]
fn compiled_playground_npr_scene_registers_file_backed_npr_presets() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let compiled = compile_scene_document_from_path(
        workspace_root.join("mods/playground-npr/scenes/comic-lines/scene.yml"),
        workspace_root.join("mods/playground-npr"),
        "playground-npr",
    )
    .expect("playground npr scene should compile");

    assert_eq!(compiled.document.npr_presets.len(), 32);

    let plan = build_scene_hydration_plan("playground-npr", &compiled.document)
        .expect("compiled playground npr plan should build");
    let presets = plan
        .commands
        .iter()
        .filter_map(npr_preset_command)
        .collect::<Vec<_>>();

    assert_eq!(presets.len(), 32);
    let default_gpu = presets
        .iter()
        .find(|preset| preset.id == "default_gpu_comic")
        .expect("default gpu comic preset should be registered");
    assert_eq!(
        default_gpu.settings.style_preset,
        amigo_render_api::NprStylePreset3d::GpuStableComic
    );
    assert_eq!(
        default_gpu.settings.render_strategy,
        amigo_render_api::NprRenderStrategy3d::GpuRealtime
    );
    assert_eq!(default_gpu.settings.passes, 1);
    assert_eq!(default_gpu.settings.search_line_count, 0);
    assert_eq!(
        default_gpu.settings.gpu_realtime_tuning.debug_mode,
        amigo_render_api::NprGpuDebugMode3d::Final
    );
    assert!(!default_gpu.settings.gpu_realtime_tuning.search_enabled);
    assert_eq!(default_gpu.settings.gpu_realtime_tuning.max_chained_walk_edges, 0);
    assert!(presets.iter().any(|preset| {
        preset.id == "default_gpu_comic"
            && (preset.settings.humanization - 0.16).abs() < f32::EPSILON
            && preset.settings.render_strategy == amigo_render_api::NprRenderStrategy3d::GpuRealtime
            && preset.settings.stroke_tool == amigo_render_api::NprStrokeTool3d::TechnicalPen
            && (preset.settings.alpha_pressure_curve[3] - 0.9).abs() < f32::EPSILON
            && (preset.settings.depth_alpha - 0.04).abs() < f32::EPSILON
            && (preset.settings.tool_width_multiplier - 1.0).abs() < f32::EPSILON
            && (preset.settings.tool_alpha_multiplier - 1.0).abs() < f32::EPSILON
            && (preset.settings.tool_dropout_multiplier - 1.0).abs() < f32::EPSILON
            && (preset.settings.endpoint_lock_start_px - 6.0).abs() < f32::EPSILON
            && (preset.settings.endpoint_lock_end_px - 7.0).abs() < f32::EPSILON
            && !preset.settings.suggestive
            && !preset.settings.contact
            && (preset.settings.contact_ground_y - 0.0).abs() < f32::EPSILON
            && (preset.settings.contact_threshold - 0.08).abs() < f32::EPSILON
            && !preset.settings.gpu_realtime_tuning.search_enabled
    }));
    assert_playground_npr_cpu_reference_presets_match_gpu_presets(&presets);
    assert!(presets.iter().any(|preset| {
        preset.id == "heavy_noir_ink"
            && preset.settings.width_px > 3.5
            && preset.settings.stroke_tool == amigo_render_api::NprStrokeTool3d::Brush
    }));
    assert!(presets.iter().any(|preset| {
        preset.id == "default_gpu_comic_cpu_reference"
            && (preset.settings.humanization - 0.16).abs() < f32::EPSILON
            && preset.settings.render_strategy == amigo_render_api::NprRenderStrategy3d::CpuReference
            && preset.settings.stroke_tool == amigo_render_api::NprStrokeTool3d::TechnicalPen
            && !preset.settings.suggestive
            && !preset.settings.contact
    }));
    assert!(presets.iter().any(|preset| {
        preset.id == "technical_comic_line"
            && preset.settings.humanization < 0.1
            && preset.settings.render_strategy == amigo_render_api::NprRenderStrategy3d::GpuRealtime
            && preset.settings.straightness >= 1.0
            && preset.settings.stroke_tool == amigo_render_api::NprStrokeTool3d::TechnicalPen
    }));
    let akira = presets
        .iter()
        .find(|preset| preset.id == "akira")
        .expect("akira preset should be registered");
    assert_eq!(
        akira.settings.render_strategy,
        amigo_render_api::NprRenderStrategy3d::GpuRealtime
    );
    assert_eq!(akira.settings.fill_mode, amigo_render_api::NprFillMode3d::None);
    assert_eq!(
        akira.settings.stroke_tool,
        amigo_render_api::NprStrokeTool3d::InkPen
    );
    assert!(akira.settings.silhouette);
    assert!(akira.settings.boundary);
    assert!(akira.settings.feature);
    assert!(!akira.settings.suggestive);
    assert!(!akira.settings.contact);
    assert!(akira.settings.silhouette_width_multiplier > akira.settings.boundary_width_multiplier);
    assert!(akira.settings.boundary_width_multiplier > akira.settings.feature_width_multiplier);
    assert_eq!(akira.settings.search_line_count, 0);
    assert!(!akira.settings.gpu_realtime_tuning.search_enabled);
    assert_eq!(akira.settings.gpu_realtime_tuning.max_chained_walk_edges, 0);
    assert!(akira.settings.temporal_path_smoothing);
    assert_eq!(
        akira.settings.pipeline.candidate_strategy,
        amigo_render_api::NprCandidateStrategy3d::CharacterSemantic
    );
    assert_eq!(
        akira.settings.pipeline.path_strategy,
        amigo_render_api::NprPathStrategy3d::StableStrokedPaths
    );
    assert_eq!(
        akira.settings.pipeline.stroke_strategy,
        amigo_render_api::NprStrokeStrategy3d::AkiraInk
    );
    assert_eq!(
        akira.settings.pipeline.fill_strategy,
        amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass
    );
    assert_eq!(
        akira.settings.pipeline.hatching_strategy,
        amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching
    );
    assert_eq!(
        akira.settings.pipeline.budget_strategy,
        amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority
    );
    assert_eq!(
        akira.settings.pipeline.temporal_strategy,
        amigo_render_api::NprTemporalStrategy3d::StableArcLength
    );
    assert!(akira.settings.black_mass_material_ids.is_empty());
    assert!(akira.settings.ink_detail_material_ids.is_empty());

    let akira_cpu = presets
        .iter()
        .find(|preset| preset.id == "akira_cpu_reference")
        .expect("akira cpu reference preset should be registered");
    assert_eq!(
        akira_cpu.settings.render_strategy,
        amigo_render_api::NprRenderStrategy3d::CpuReference
    );
    assert_eq!(
        akira_cpu.settings.fill_mode,
        amigo_render_api::NprFillMode3d::None
    );
    assert_eq!(akira_cpu.settings.pipeline, akira.settings.pipeline);
    assert_eq!(
        akira_cpu
            .settings
            .gpu_realtime_tuning
            .max_chained_walk_edges,
        0
    );
    assert!(akira_cpu.settings.black_mass_material_ids.is_empty());
    assert!(akira_cpu.settings.ink_detail_material_ids.is_empty());
}

fn assert_playground_npr_cpu_reference_presets_match_gpu_presets(
    presets: &[&crate::render_commands::NprPreset3dSceneCommand],
) {
    for gpu_id in [
        "default_gpu_comic",
        "rough_comic_ink",
        "clean_comic_ink",
        "loose_pencil",
        "animation_pencil",
        "manga_fine_line",
        "european_clear_line",
        "dry_brush_ink",
        "heavy_noir_ink",
        "storyboard_marker",
        "technical_comic_line",
        "cinematic_12fps",
        "balanced_30fps",
        "target_60fps",
        "low_120fps",
        "akira",
    ] {
        let cpu_id = format!("{gpu_id}_cpu_reference");
        let gpu = presets
            .iter()
            .find(|preset| preset.id == gpu_id)
            .unwrap_or_else(|| panic!("gpu preset `{gpu_id}` should be registered"));
        let cpu_reference = presets
            .iter()
            .find(|preset| preset.id == cpu_id)
            .unwrap_or_else(|| panic!("cpu reference preset `{cpu_id}` should be registered"));
        assert_eq!(
            gpu.settings.render_strategy,
            amigo_render_api::NprRenderStrategy3d::GpuRealtime
        );
        assert_eq!(
            cpu_reference.settings.render_strategy,
            amigo_render_api::NprRenderStrategy3d::CpuReference
        );
        let mut normalized_cpu_reference = cpu_reference.settings.clone();
        normalized_cpu_reference.render_strategy = amigo_render_api::NprRenderStrategy3d::GpuRealtime;
        assert_eq!(
            normalized_cpu_reference, gpu.settings,
            "CPU reference preset `{cpu_id}` should match GPU preset `{gpu_id}` except render strategy"
        );
    }
}

#[test]
fn assert_all_playground_npr_cpu_reference_presets_match_gpu_presets() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let compiled = compile_scene_document_from_path(
        workspace_root.join("mods/playground-npr/scenes/comic-lines/scene.yml"),
        workspace_root.join("mods/playground-npr"),
        "playground-npr",
    )
    .expect("playground npr scene should compile");

    let plan = build_scene_hydration_plan("playground-npr", &compiled.document)
        .expect("compiled playground npr plan should build");
    let presets = plan
        .commands
        .iter()
        .filter_map(npr_preset_command)
        .collect::<Vec<_>>();

    assert_playground_npr_cpu_reference_presets_match_gpu_presets(&presets);
}

#[test]
fn mesh3d_npr_grouped_schema_hydrates_to_line_settings() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: grouped-npr
entities:
  - id: model
    name: grouped-npr-model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
          style_preset: rough_comic_ink
          fill_mode: none
          suggestive: true
          contact: true
          contact_ground_y: -0.2
          contact_threshold: 0.16
          tool:
            kind: pencil
            base_width_px: 3.25
            base_alpha: 0.72
            width_multiplier: 1.4
            alpha_multiplier: 0.8
            wobble_multiplier: 1.7
            pressure_jitter_multiplier: 2.0
            dropout_multiplier: 1.6
            search_multiplier: 1.5
          trajectory:
            path_adherence: 0.44
            humanization: 0.81
            gesture_offset_px: 0.9
            gesture_frequency_per_100px: 0.33
            micro_offset_px: 0.22
            micro_frequency_per_100px: 2.4
            angular_drift_degrees: 2.0
            endpoint_snap_px: 2.8
            path_simplify_px: 1.1
          pressure:
            width_curve: [0.4, 0.9, 0.8, 0.3]
            jitter: 0.21
          opacity:
            alpha_curve: [0.3, 0.8, 0.7, 0.2]
          endpoints:
            taper: 0.5
            lock_start_px: 8.0
            lock_end_px: 9.0
            overshoot_end_px: 1.9
            undershoot_end_px: 0.4
          breakup:
            amount: 0.12
            min_gap_px: 6.0
          depth:
            width_influence: 0.25
            alpha_influence: 0.18
          confidence:
            line_confidence: 0.63
          passes:
            primary_count: 3
            search_count: 2
            search_alpha: 0.19
            search_offset_px: 0.7
          class_overrides:
            silhouette:
              width_multiplier: 1.7
              alpha_multiplier: 0.95
"##,
    )
    .expect("grouped NPR scene should parse");

    let plan = build_scene_hydration_plan("playground-npr", &document)
        .expect("grouped NPR scene should build hydration plan");
    let mesh = plan
        .commands
        .iter()
        .find_map(mesh_command)
        .expect("mesh command should hydrate");
    let npr = mesh.npr.as_ref().expect("NPR settings should hydrate");

    assert_eq!(npr.stroke_tool, amigo_render_api::NprStrokeTool3d::Pencil);
    assert_eq!(npr.fill_mode, amigo_render_api::NprFillMode3d::None);
    assert!(npr.suggestive);
    assert!(npr.contact);
    assert_eq!(npr.contact_ground_y, -0.2);
    assert_eq!(npr.contact_threshold, 0.16);
    assert_eq!(npr.width_px, 3.25);
    assert!((npr.ink_color.a - 0.72).abs() < f32::EPSILON);
    assert_eq!(npr.tool_width_multiplier, 1.4);
    assert_eq!(npr.tool_alpha_multiplier, 0.8);
    assert_eq!(npr.tool_wobble_multiplier, 1.7);
    assert_eq!(npr.tool_pressure_jitter_multiplier, 2.0);
    assert_eq!(npr.tool_dropout_multiplier, 1.6);
    assert_eq!(npr.tool_search_multiplier, 1.5);
    assert_eq!(npr.straightness, 0.44);
    assert_eq!(npr.humanization, 0.81);
    assert_eq!(npr.stroke_wobble_px, 0.9);
    assert_eq!(npr.stroke_wobble_frequency, 0.33);
    assert_eq!(npr.micro_wobble_px, 0.22);
    assert_eq!(npr.micro_wobble_frequency, 2.4);
    assert_eq!(npr.local_angular_drift_degrees, 2.0);
    assert_eq!(npr.endpoint_snap_px, 2.8);
    assert_eq!(npr.path_simplify_px, 1.1);
    assert_eq!(npr.width_pressure_curve, [0.4, 0.9, 0.8, 0.3]);
    assert_eq!(npr.alpha_pressure_curve, [0.3, 0.8, 0.7, 0.2]);
    assert_eq!(npr.taper, 0.5);
    assert_eq!(npr.endpoint_lock_start_px, 8.0);
    assert_eq!(npr.endpoint_lock_end_px, 9.0);
    assert_eq!(npr.overshoot_px, 1.9);
    assert_eq!(npr.undershoot_px, 0.4);
    assert_eq!(npr.dropout, 0.12);
    assert_eq!(npr.dropout_segment_min_px, 6.0);
    assert_eq!(npr.depth_pressure, 0.25);
    assert_eq!(npr.depth_alpha, 0.18);
    assert_eq!(npr.line_confidence, 0.63);
    assert_eq!(npr.passes, 3);
    assert_eq!(npr.search_line_count, 2);
    assert_eq!(npr.search_line_alpha, 0.19);
    assert_eq!(npr.pass_offset_px, 0.7);
    assert_eq!(
        npr.silhouette_override
            .expect("silhouette override should hydrate")
            .width_multiplier,
        Some(1.7)
    );
}

#[test]
fn mesh3d_npr_defaults_strategy_to_gpu_realtime() {
    let npr = first_mesh_npr_settings(
        r#"
version: 1
scene:
  id: npr-gpu-default
entities:
  - id: model
    name: model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
"#,
    );

    assert_eq!(
        npr.render_strategy,
        amigo_render_api::NprRenderStrategy3d::GpuRealtime
    );
}

#[test]
fn mesh3d_npr_hydrates_cpu_reference_strategy() {
    let npr = first_mesh_npr_settings(
        r#"
version: 1
scene:
  id: npr-cpu-reference
entities:
  - id: model
    name: model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
          strategy: cpu_reference
"#,
    );

    assert_eq!(
        npr.render_strategy,
        amigo_render_api::NprRenderStrategy3d::CpuReference
    );
}

#[test]
fn mesh3d_npr_rejects_hybrid_strategy() {
    let error = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: npr-invalid-hybrid
entities:
  - id: model
    name: model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
          strategy: hybrid
"#,
    )
    .and_then(|document| build_scene_hydration_plan("playground-npr", &document))
    .expect_err("hybrid strategy should be rejected");

    let message = error.to_string();
    assert!(message.contains("unsupported Mesh3D.npr.strategy `hybrid`"));
}

#[test]
fn mesh3d_npr_rejects_auto_strategy() {
    let error = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: npr-invalid-auto
entities:
  - id: model
    name: model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
          strategy: auto
"#,
    )
    .and_then(|document| build_scene_hydration_plan("playground-npr", &document))
    .expect_err("auto strategy should be rejected");

    let message = error.to_string();
    assert!(message.contains("unsupported Mesh3D.npr.strategy `auto`"));
}

#[test]
fn mesh3d_npr_rejects_gpu_alias_strategy() {
    let error = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: npr-invalid-gpu-alias
entities:
  - id: model
    name: model
    components:
      - type: Mesh3D
        mesh: playground-npr/meshes/soldier
        npr:
          enabled: true
          strategy: gpu
"#,
    )
    .and_then(|document| build_scene_hydration_plan("playground-npr", &document))
    .expect_err("gpu alias strategy should be rejected");

    let message = error.to_string();
    assert!(message.contains("invalid Mesh3D.npr.strategy `gpu`"));
}

#[test]
fn builds_hydration_plan_for_playground_3d_physics_scene() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-3d/scenes/physics-cubes/scene.yml"),
    )
    .expect("playground 3d physics scene should parse");

    let plan = build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

    assert!(plan.commands.iter().any(|command| {
        mesh_command(command).is_some_and(|command| command.entity_name == "playground-3d-ground")
    }));
    assert!(plan.commands.iter().any(|command| {
        static_box_collider3d_command(command)
            .is_some_and(|command| command.entity_name == "playground-3d-ground")
    }));
    assert!(plan.commands.iter().any(|command| {
        physics_spawner3d_command(command).is_some_and(|command| {
            command.entity_name == "playground-3d-cube-spawner"
                && command.entity_prefix == "playground-3d-cube"
        })
    }));
}

#[test]
fn builds_hydration_plan_for_playground_2d_screen_space_preview() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let document = load_scene_document_from_path(
        workspace_root.join("mods/playground-2d/scenes/screen-space-preview/scene.yml"),
    )
    .expect("screen-space preview scene should parse");

    let plan = build_scene_hydration_plan("playground-2d", &document)
        .expect("screen-space preview plan should build");

    assert!(
        plan.commands
            .iter()
            .all(|command| sprite_command(command).is_none())
    );
    assert!(plan.commands.iter().any(|command| {
        ui_command(command).is_some_and(|command| command.entity_name == "playground-2d-ui-preview")
    }));
}
