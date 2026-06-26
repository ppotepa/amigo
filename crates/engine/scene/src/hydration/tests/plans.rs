use std::path::PathBuf;

use amigo_math::{ColorRgba, Transform3, Vec3};

use super::super::{
    build_scene_hydration_plan, entity_selector_from_document, scene_key_from_document,
};
use crate::{
    ComponentHydratorRegistry, ComponentSchemaRegistry, EntitySelector,
    PluginComponentHydrationContext, PluginComponentHydrator, SceneCommand, SceneComponentPayload,
    SceneComponentSchemaProvider, SceneDocumentError, SceneEntitySelectorDocument,
    SceneEntitySelectorKindDocument, load_scene_document_from_path, load_scene_document_from_str,
    load_scene_document_from_str_with_component_schemas,
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

    let box_mesh = plan
        .commands
        .iter()
        .filter_map(mesh_command)
        .find(|command| command.entity_name == "playground-npr-box-source")
        .expect("box mesh command should be present");
    let box_npr = box_mesh
        .npr
        .as_ref()
        .expect("npr: true should enable default NPR line settings");
    assert!(box_npr.boundary);
    assert!(box_npr.silhouette);
    assert!(box_npr.feature);
    assert_eq!(box_npr.feature_angle_degrees, 32.0);

    let fox_mesh = plan
        .commands
        .iter()
        .filter_map(mesh_command)
        .find(|command| command.entity_name == "playground-npr-fox-source")
        .expect("fox mesh command should be present");
    let fox_npr = fox_mesh
        .npr
        .as_ref()
        .expect("npr settings block should enable NPR line settings");
    assert_eq!(fox_npr.feature_angle_degrees, 30.0);
    assert_eq!(fox_npr.seed, 2602);
    assert_eq!(fox_npr.passes, 2);
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
