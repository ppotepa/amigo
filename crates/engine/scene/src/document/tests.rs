use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{
    Camera2dModeDocument, CameraFocus2dDocument, Material2dOpticalModeDocument,
    RenderDepthMode2dDocument, SceneComponentDocument, SceneEntitySelectorDocument,
    SceneEntitySelectorKindDocument, compile_scene_document_from_path, load_scene_document_from_path,
    load_scene_document_from_str,
};
use crate::SceneDocumentError;

static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn scene_document_parses_text2d_material() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: text-material
entities:
  - id: title
    name: title
    components:
      - type: Text2D
        content: ROTTEN CLUB
        font: rotten-club/fonts/game
        bounds: { x: 1180.0, y: 240.0 }
        material:
          optical:
            mode: refractive
            transmission: 0.58
            refraction_px: 4.5
"##,
    )
    .expect("scene document should parse");

    let material = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::Text2d { material, .. } => *material,
            _ => None,
        })
        .expect("text material should parse");

    assert_eq!(material.optical.mode, Material2dOpticalModeDocument::Refractive);
    assert_eq!(material.optical.transmission, 0.58);
    assert_eq!(material.optical.refraction_px, 4.5);
}

#[test]
fn scene_document_parses_sprite2d_material_and_render_contributions() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: sprite-material-test
  label: Sprite Material Test
entities:
  - id: poster
    name: poster
    components:
      - type: Sprite2D
        render_layer: foreground.props
        texture: test/poster
        size: [128, 128]
        render_contributions:
          world.color: true
          material.mask: true
          optics.refract: true
          transmission.source: true
        material:
          optical:
            mode: refractive
            transmission: 0.45
            refraction_px: 7.0
"##,
    )
    .expect("sprite material scene should parse");

    let (material, contributions) = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::Sprite2d {
                material,
                render_contributions,
                ..
            } => Some((material.expect("sprite material should parse"), render_contributions)),
            _ => None,
        })
        .expect("sprite component should exist");

    assert_eq!(material.optical.mode, Material2dOpticalModeDocument::Refractive);
    assert_eq!(material.optical.transmission, 0.45);
    assert_eq!(material.optical.refraction_px, 7.0);
    assert_eq!(contributions.get("world.color"), Some(true));
    assert_eq!(contributions.get("material.mask"), Some(true));
    assert_eq!(contributions.get("optics.refract"), Some(true));
    assert_eq!(contributions.get("transmission.source"), Some(true));
}

#[test]
fn scene_document_parses_vector_shape_material_and_render_contributions() {
    let document = load_scene_document_from_str(
        r##"
version: 1
scene:
  id: vector-material-test
  label: Vector Material Test
entities:
  - id: vector-glass
    name: vector-glass
    components:
      - type: VectorShape2D
        render_layer: foreground.props
        kind: circle
        radius: 48.0
        segments: 24
        fill_color: "#FFFFFFFF"
        render_contributions:
          world.color: true
          material.mask: true
          optics.refract: true
          transmission.source: true
        material:
          optical:
            mode: refractive
            transmission: 0.35
            refraction_px: 5.0
"##,
    )
    .expect("vector material scene should parse");

    let (material, contributions) = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::VectorShape2d {
                material,
                render_contributions,
                ..
            } => Some((material.expect("vector material should parse"), render_contributions)),
            _ => None,
        })
        .expect("vector component should exist");

    assert_eq!(material.optical.mode, Material2dOpticalModeDocument::Refractive);
    assert_eq!(material.optical.transmission, 0.35);
    assert_eq!(material.optical.refraction_px, 5.0);
    assert_eq!(contributions.get("world.color"), Some(true));
    assert_eq!(contributions.get("material.mask"), Some(true));
    assert_eq!(contributions.get("optics.refract"), Some(true));
    assert_eq!(contributions.get("transmission.source"), Some(true));
}

#[test]
fn parses_scene_document_from_yaml() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
state:
  camera_lens.distortion_px: 11.0
  camera_lens.chromatic_aberration: 0.085
  camera_lens.enabled: true
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/spritesheets/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
    )
    .expect("scene document should parse");

    assert_eq!(document.scene.id, "sprite-lab");
    assert_eq!(document.state.len(), 3);
    assert_eq!(document.entities.len(), 2);
    assert_eq!(document.entity_names()[1], "playground-2d-sprite");
    assert_eq!(
        document.component_kind_counts().get("Sprite2D"),
        Some(&1usize)
    );
    assert!(matches!(
        document.entities[1].components[0],
        SceneComponentDocument::Sprite2d { .. }
    ));
}

#[test]
fn scene_document_defaults_camera2d_render_contributions_to_empty_document() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: camera-defaults
  label: Camera Defaults
entities:
  - id: camera
    name: camera
    components:
      - type: Camera2D
"#,
    )
    .expect("camera scene should parse");

    let contributions = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::Camera2d {
                render_contributions,
                ..
            } => Some(render_contributions),
            _ => None,
        })
        .expect("camera component should exist");

    assert!(contributions.is_empty());
}

#[test]
fn scene_document_parses_camera2d_render_contributions() {
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
          camera.exposure: true
          camera.film: true
          camera.scan_output: false
"#,
    )
    .expect("camera scene should parse");

    let contributions = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::Camera2d {
                render_contributions,
                ..
            } => Some(render_contributions),
            _ => None,
        })
        .expect("camera component should exist");

    assert_eq!(contributions.get("camera.exposure"), Some(true));
    assert_eq!(contributions.get("camera.film"), Some(true));
    assert_eq!(contributions.get("camera.scan_output"), Some(false));
}

#[test]
fn scene_document_parses_beacon2d_render_contributions() {
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
    .expect("beacon scene should parse");

    let contributions = document
        .entities
        .iter()
        .flat_map(|entity| &entity.components)
        .find_map(|component| match component {
            SceneComponentDocument::BeaconLight2d {
                render_contributions,
                ..
            } => Some(render_contributions),
            _ => None,
        })
        .expect("beacon component should exist");

    assert_eq!(contributions.get("overlay.visible"), Some(false));
    assert_eq!(contributions.get("relight.plate"), Some(true));
    assert_eq!(contributions.get("bloom.source"), Some(false));
}

#[test]
fn parses_visual2d_spatial_depth_space_and_distance_layer_depth() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: test
visual2d:
  spatial:
    depth_space:
      near_m: 1.0
      far_m: 1500.0
      curve: logarithmic
  render_layers:
    - id: weather.rain.mid
      depth:
        mode: distance
        distance_m: 75.0
        blur_scale: 0.25
"#,
    )
    .expect("scene document should parse");

    assert_eq!(document.visual2d.spatial.depth_space.near_m, 1.0);
    assert_eq!(
        document.visual2d.render_layers[0].depth.mode,
        RenderDepthMode2dDocument::Distance
    );
    assert_eq!(
        document.visual2d.render_layers[0].depth.distance_m,
        Some(75.0)
    );
}

#[test]
fn parses_entity_lifecycle_groups_and_properties() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy, flying]
    groups: [wave-1]
    visible: false
    simulation_enabled: true
    collision_enabled: false
    properties:
      score_value: 100
      speed: 2.5
      elite: true
      label: scout
"#,
    )
    .expect("scene document should parse");

    let entity = &document.entities[0];
    assert_eq!(entity.tags, vec!["enemy".to_owned(), "flying".to_owned()]);
    assert_eq!(entity.groups, vec!["wave-1".to_owned()]);
    assert!(!entity.visible);
    assert!(entity.simulation_enabled);
    assert!(!entity.collision_enabled);
    assert!(entity.properties.contains_key("score_value"));
    assert!(entity.properties.contains_key("speed"));
    assert!(entity.properties.contains_key("elite"));
    assert!(entity.properties.contains_key("label"));
}

#[test]
fn parses_entity_selector_documents_from_yaml() {
    let selectors = serde_yaml::from_str::<Vec<SceneEntitySelectorDocument>>(
        r#"
- kind: entity
  value: player
- kind: tag
  value: enemy
- kind: group
  value: wave-1
- kind: pool
  value: bullets
"#,
    )
    .expect("selector documents should parse");

    assert_eq!(
        selectors,
        vec![
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Entity,
                value: "player".to_owned(),
            },
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Tag,
                value: "enemy".to_owned(),
            },
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Group,
                value: "wave-1".to_owned(),
            },
            SceneEntitySelectorDocument {
                kind: SceneEntitySelectorKindDocument::Pool,
                value: "bullets".to_owned(),
            },
        ]
    );
}

#[test]
fn parses_collision_event_rules_from_yaml() {
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
entities: []
"#,
    )
    .expect("scene document should parse");

    assert_eq!(document.collision_events.len(), 1);
    assert_eq!(document.collision_events[0].id, "projectile-hits-target");
    assert_eq!(
        document.collision_events[0].source,
        SceneEntitySelectorDocument {
            kind: SceneEntitySelectorKindDocument::Tag,
            value: "projectile".to_owned(),
        }
    );
    assert!(document.collision_events[0].once_per_overlap);
}

#[test]
fn parses_playground_scene_documents_from_disk() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf();

    let sprite_doc = load_scene_document_from_path(
        workspace_root.join("mods/playground-2d/scenes/sprite-lab/scene.yml"),
    )
    .expect("sprite lab scene should parse");
    let material_doc = load_scene_document_from_path(
        workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
    )
    .expect("material lab scene should parse");

    assert_eq!(sprite_doc.scene.id, "sprite-lab");
    assert_eq!(material_doc.scene.id, "material-lab");
    assert!(sprite_doc.component_kind_counts().contains_key("Sprite2D"));
    assert!(
        material_doc
            .component_kind_counts()
            .contains_key("Material3D")
    );
}

#[test]
fn parses_playground_2d_main_scene_from_disk() {
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

    assert_eq!(document.scene.id, "hello-world-spritesheet");
    assert_eq!(document.transitions.len(), 1);
    assert!(document.component_kind_counts().contains_key("Sprite2D"));
    assert!(document.component_kind_counts().contains_key("Text2D"));
}

#[test]
fn parses_playground_3d_main_scene_from_disk() {
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

    assert_eq!(document.scene.id, "hello-world-cube");
    assert!(document.component_kind_counts().contains_key("Mesh3D"));
    assert!(document.component_kind_counts().contains_key("Material3D"));
    assert!(document.component_kind_counts().contains_key("Text3D"));
}

#[test]
fn parses_playground_2d_screen_space_preview_from_disk() {
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

    assert_eq!(document.scene.id, "screen-space-preview");
    assert!(document.component_kind_counts().contains_key("Sprite2D"));
    assert!(document.component_kind_counts().contains_key("UiDocument"));
}

#[test]
fn parses_sidescroller_component_document_from_yaml() {
    let document = load_scene_document_from_str(
        r#####"
version: 1
scene:
  id: vertical-slice
  label: Vertical Slice
entities:
  - id: camera
    name: playground-sidescroller-camera
    components:
      - type: Camera2D
      - type: CameraFollow2D
        target: playground-sidescroller-player
  - id: tilemap
    name: playground-sidescroller-tilemap
    components:
      - type: TileMap2D
        tileset: playground-sidescroller/spritesheets/platformer/tilesets/platform/base
        ruleset: playground-sidescroller/spritesheets/platformer/rulesets/platform/rules
        tile_size: { x: 16.0, y: 16.0 }
        grid:
          - "...."
          - ".P.."
          - "####"
  - id: player
    name: playground-sidescroller-player
    components:
      - type: TileMapMarker2D
        tilemap_entity: playground-sidescroller-tilemap
        symbol: "P"
        offset: { x: 0.0, y: 8.0 }
      - type: KinematicBody2D
        velocity: { x: 0.0, y: 0.0 }
        gravity_scale: 1.0
        terminal_velocity: 720.0
      - type: AabbCollider2D
        size: { x: 20.0, y: 30.0 }
        offset: { x: 0.0, y: 1.0 }
        layer: player
        mask: [world, trigger]
      - type: MotionController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
  - id: coin
    name: playground-sidescroller-coin
    components:
      - type: Sprite2D
        texture: playground-sidescroller/spritesheets/coin
        size: { x: 16.0, y: 16.0 }
        animation:
          fps: 10.0
          looping: true
      - type: Trigger2D
        size: { x: 16.0, y: 16.0 }
        layer: trigger
        mask: [player]
        event: coin.collected
"#####,
    )
    .expect("sidescroller scene document should parse");

    assert_eq!(document.scene.id, "vertical-slice");
    assert!(document.component_kind_counts().contains_key("TileMap2D"));
    let tilemap_component = document
        .entities
        .iter()
        .find(|entity| entity.name == "playground-sidescroller-tilemap")
        .and_then(|entity| {
            entity
                .components
                .iter()
                .find(|component| matches!(component, SceneComponentDocument::TileMap2d { .. }))
        })
        .expect("tilemap component should exist");
    match tilemap_component {
        SceneComponentDocument::TileMap2d { ruleset, .. } => {
            assert_eq!(
                ruleset.as_deref(),
                Some("playground-sidescroller/spritesheets/platformer/rulesets/platform/rules")
            );
        }
        _ => unreachable!("expected tilemap component"),
    }
    assert!(
        document
            .component_kind_counts()
            .contains_key("KinematicBody2D")
    );
    assert!(
        document
            .component_kind_counts()
            .contains_key("AabbCollider2D")
    );
    assert!(document.component_kind_counts().contains_key("Trigger2D"));
    assert!(
        document
            .component_kind_counts()
            .contains_key("MotionController2D")
    );
    assert!(document.component_kind_counts().contains_key("Sprite2D"));
    assert!(
        document
            .component_kind_counts()
            .contains_key("CameraFollow2D")
    );
    assert!(
        document
            .component_kind_counts()
            .contains_key("TileMapMarker2D")
    );
}

#[test]
fn rejects_legacy_platformer_controller_component_alias() {
    let result = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: legacy-motion-alias
  label: Legacy Motion Alias
entities:
  - id: player
    components:
      - type: PlatformerController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
"#,
    );

    assert!(result.is_err());
}

#[test]
fn old_scene_without_prefab_fields_still_loads() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: legacy-no-prefab
entities:
  - id: camera
    components:
      - type: Camera2D
"#,
    )
    .expect("legacy scene should parse without prefab fields");

    assert_eq!(document.scene.id, "legacy-no-prefab");
    assert_eq!(document.entities.len(), 1);
    assert!(document.entities[0].prefab.is_none());
    assert!(document.entities[0].prefab_overrides.is_empty());
}

#[test]
fn scene_with_prefab_instance_loads() {
    let document = load_scene_document_from_str(
        r#"
version: 1
scene:
  id: prefab-scene
entities:
  - id: start-button
    prefab:
      id: ink-wars/ui/menu-button
    prefab_overrides:
      - target: text
        value: START
    transform2:
      translation: { x: -307.0, y: 67.0 }
"#,
    )
    .expect("scene with prefab instance should parse");

    let entity = &document.entities[0];
    assert_eq!(
        entity.prefab.as_ref().map(|prefab| prefab.id.as_str()),
        Some("ink-wars/ui/menu-button")
    );
    assert_eq!(entity.prefab_overrides.len(), 1);
    assert_eq!(entity.prefab_overrides[0].target, "text");
}

#[test]
fn scene_document_parses_lens_droplets_post_fx() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: lens_droplets
      id: rain-lens
      enabled: true
      affects:
        debug_ui: false
      surface:
        blur_px: 3.0
        blur_samples: 4
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
}

#[test]
fn scene_document_parses_downscale_post_fx() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: downscale
      id: chunky-scale
      factor: 2.0
      opacity: 1.0
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
    let crate::PostFx2dDocument::Downscale(effect) = &document.visual2d.post_fx[0] else {
        panic!("expected downscale");
    };
    assert_eq!(effect.factor, 2.0);
    assert_eq!(effect.opacity, 1.0);
}

#[test]
fn scene_document_parses_color_quantize_highlight_bias() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: color_quantize
      id: sixteen-colors
      palette_size: 16
      dither_strength: 0.28
      opacity: 1.0
      luma_preserve: 0.72
      highlight_bias: 0.45
      gamma: 1.55
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
    let crate::PostFx2dDocument::ColorQuantize(effect) = &document.visual2d.post_fx[0] else {
        panic!("expected color quantize");
    };
    assert_eq!(effect.palette_size, 16);
    assert_eq!(effect.highlight_bias, 0.45);
}

#[test]
fn scene_document_parses_color_ramp_grade() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: color_ramp
      id: rotten-noir
      palette_size: 32
      dither_strength: 0.42
      layered_dither: 0.35
      shadow_bias: 0.65
      contrast: 1.18
      saturation: 0.85
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
    let crate::PostFx2dDocument::ColorRamp(effect) = &document.visual2d.post_fx[0] else {
        panic!("expected color ramp");
    };
    assert_eq!(effect.palette_size, 32);
    assert_eq!(effect.shadow_bias, 0.65);
    assert_eq!(effect.contrast, 1.18);
}

#[test]
fn scene_document_parses_camera2d_optics() {
    let yaml = r#"
version: 1
scene:
  id: test-scene
entities:
  - id: camera
    name: camera
    components:
      - type: Camera2D
        id: main
        mode: manual
        exposure:
          iso: 800
          compensation: -0.2
        shutter:
          fps: 12
          angle: 180
          history_mix: 0.72
        lens:
          profile: vintage_soviet_35mm_dirty
          intensity: 0.9
        film:
          profile: polish_1994_push_800
          intensity: 0.85
        look:
          profile: rotten-club/camera/look/rotten-noir-print
          intensity: 0.7
        aperture:
          enabled: false
"#;

    let document = load_scene_document_from_str(yaml).unwrap();

    let component = &document.entities[0].components[0];
    match component {
        SceneComponentDocument::Camera2d {
            id,
            mode,
            exposure,
            shutter,
            lens,
            film,
            look,
            aperture,
            ..
        } => {
            assert_eq!(id, "main");
            assert_eq!(mode, &Camera2dModeDocument::Manual);
            assert_eq!(exposure.iso, 800.0);
            assert_eq!(shutter.fps, 12.0);
            assert_eq!(lens.profile, "vintage_soviet_35mm_dirty");
            assert_eq!(film.profile, "polish_1994_push_800");
            assert_eq!(look.profile, "rotten-club/camera/look/rotten-noir-print");
            assert_eq!(look.intensity, 0.7);
            assert!(!aperture.enabled);
        }
        other => panic!("expected Camera2D component, got {other:?}"),
    }
}

#[test]
fn scene_document_parses_camera2d_distance_and_depth_focus() {
    let distance_yaml = r#"
version: 1
scene:
  id: test-scene
entities:
  - id: camera
    name: camera
    components:
      - type: Camera2D
        id: main
        aperture:
          focus:
            kind: distance
            distance_m: 6.0
"#;

    let document = load_scene_document_from_str(distance_yaml).unwrap();
    let SceneComponentDocument::Camera2d { aperture, .. } = &document.entities[0].components[0]
    else {
        panic!("expected Camera2D component");
    };
    assert!(matches!(
        aperture.focus,
        CameraFocus2dDocument::Distance { distance_m } if (distance_m - 6.0).abs() < f32::EPSILON
    ));

    let depth_yaml = r#"
version: 1
scene:
  id: test-scene
entities:
  - id: camera
    name: camera
    components:
      - type: Camera2D
        id: main
        aperture:
          focus:
            kind: depth
            value: 0.52
"#;

    let document = load_scene_document_from_str(depth_yaml).unwrap();
    let SceneComponentDocument::Camera2d { aperture, .. } = &document.entities[0].components[0]
    else {
        panic!("expected Camera2D component");
    };
    assert!(matches!(
        aperture.focus,
        CameraFocus2dDocument::Depth { value } if (value - 0.52).abs() < f32::EPSILON
    ));
}

#[test]
fn scene_document_parses_shutter_blur_post_fx() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: shutter_blur
      id: shutter_24
      fps: 24.0
      shutter_angle: 180.0
      opacity: 0.72
      edge_rejection: 0.35
      luma_threshold: 0.04
      frame_hold: true
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
    let crate::PostFx2dDocument::ShutterBlur(effect) = &document.visual2d.post_fx[0] else {
        panic!("expected shutter_blur post-fx");
    };
    assert_eq!(effect.fps, 24.0);
    assert_eq!(effect.shutter_angle, 180.0);
    assert!(effect.frame_hold);
}

#[test]
fn scene_document_parses_extended_rain_glass_post_fx() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: rain_glass
      id: lens
      enabled: true
      spawn_rate: 14.0
      spawn_limit: 1200
      spawn_size: [36.0, 120.0]
      simulation:
        gravity_px_per_sec2: 2400.0
      trails:
        enabled: true
        taper: 0.42
        streak_boost: 0.72
        streak_length: 1.15
      micro_droplets:
        per_second: 650.0
      mist:
        opacity: 0.18
        time: 16.0
        color_strength: 0.012
        blur_step: 4
      render:
        background_blur_px: 7.5
        background_blur_steps: 2
        smooth_edge: [0.90, 0.985]
        chromatic_aberration: 0.18
        distortion_px: 30.0
        normal_strength: 6.0
        focus_blur_strength: 0.8
        body_opacity: 0.86
        trail_refract_scale: 0.52
        trail_opacity: 0.74
        raindrop_compose: smoother
        raindrop_eraser_size: [0.93, 1.0]
      lighting:
        scene_light_response: 1.45
        rim_strength: 1.12
      debug:
        view: final
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
    let crate::PostFx2dDocument::RainGlass(rain) = &document.visual2d.post_fx[0] else {
        panic!("expected rain glass");
    };
    assert_eq!(rain.render.distortion_px, 30.0);
    assert_eq!(rain.render.normal_strength, 6.0);
    assert_eq!(rain.render.focus_blur_strength, 0.8);
    assert_eq!(rain.render.body_opacity, 0.86);
    assert_eq!(rain.render.trail_refract_scale, 0.52);
    assert_eq!(rain.render.trail_opacity, 0.74);
    assert_eq!(rain.render.background_blur_steps, 2);
    assert_eq!(rain.render.raindrop_compose, "smoother");
    assert_eq!(rain.render.raindrop_eraser_size, [0.93, 1.0]);
    assert_eq!(rain.trails.streak_boost, 0.72);
    assert_eq!(rain.trails.streak_length, 1.15);
    assert_eq!(rain.mist.time, 16.0);
    assert_eq!(rain.mist.color_strength, 0.012);
    assert_eq!(rain.mist.blur_step, 4);
    assert_eq!(rain.lighting.scene_light_response, 1.45);
    assert_eq!(rain.lighting.rim_strength, 1.12);
}

#[test]
fn scene_document_parses_wet_reflections_post_fx() {
    let yaml = r#"
version: 1
scene:
  id: test
  label: Test
visual2d:
  post_fx:
    - type: wet_reflections
      id: neon-alley-wet-ground
      enabled: true
      masks:
        reflection: rotten-club/layered-images/neon-alley/reflection_mask.png
        reflection_invert: true
        edges: rotten-club/layered-images/neon-alley/edge_map_2.png
      surface:
        blur_px: 1.5
        distortion_px: 0.8
        shimmer_strength: 0.04
        wet_darken: 0.06
        specular_boost: 0.25
"#;

    let document = crate::load_scene_document_from_str(yaml).unwrap();
    assert_eq!(document.visual2d.post_fx.len(), 1);
}

#[test]
fn scene_compiler_merges_use_domain_files() {
    let root = scene_compiler_temp_dir("merge");
    write_scene_file(
        &root,
        "scenes/main/scene.yml",
        r#"
version: 1
scene:
  id: main
  label: Main
use:
  visual:
    - ./visual.yml
  entities: ./entities.yml
"#,
    );
    write_scene_file(
        &root,
        "scenes/main/visual.yml",
        r#"
kind: scene-fragment
visual2d:
  render_layers:
    - id: gameplay
      order: 1
"#,
    );
    write_scene_file(
        &root,
        "scenes/main/entities.yml",
        r#"
kind: scene-fragment
entities:
  - id: camera
    name: Camera
    components:
      - type: Camera2D
"#,
    );

    let compiled =
        compile_scene_document_from_path(root.join("scenes/main/scene.yml"), &root, "test-mod")
            .expect("scene should compile");

    assert_eq!(compiled.document.entities.len(), 1);
    assert_eq!(compiled.document.visual2d.render_layers.len(), 1);
    assert!(compiled.value.get("use").is_none());
}

#[test]
fn scene_compiler_collects_scheduling_metadata() {
    let root = scene_compiler_temp_dir("scheduling");
    write_scene_file(
        &root,
        "scenes/main/scene.yml",
        r#"
version: 1
scene:
  id: main
  label: Main
scheduling:
  mode: single_thread
use:
  scheduling:
    - ./scheduling.yml
entities:
  - id: camera
    name: Camera
    components:
      - type: Camera2D
"#,
    );
    write_scene_file(
        &root,
        "scenes/main/scheduling.yml",
        r#"
kind: scene-scheduling
scheduling:
  strict: true
  overrides:
    - target: render:particles2d
      lane: render_prepare
"#,
    );

    let compiled =
        compile_scene_document_from_path(root.join("scenes/main/scene.yml"), &root, "test-mod")
            .expect("scene should compile");

    let scheduling = compiled
        .scheduling
        .as_ref()
        .expect("compiled scheduling metadata should be present");
    assert_eq!(scheduling.mode.as_deref(), Some("single_thread"));
    assert!(scheduling.strict);
    assert_eq!(scheduling.overrides.len(), 1);
    assert!(compiled.dependencies.iter().any(|dependency| matches!(
        dependency.kind,
        crate::SceneDocumentDependencyKind::Scheduling
    )));
    assert!(compiled.value.get("scheduling").is_none());
}

#[test]
fn scene_compiler_rejects_duplicate_entity_ids() {
    let root = scene_compiler_temp_dir("duplicate_entity");
    write_scene_file(
        &root,
        "scenes/main/scene.yml",
        r#"
version: 1
scene:
  id: main
  label: Main
entities:
  - id: camera
    name: Camera A
  - id: camera
    name: Camera B
"#,
    );

    let error =
        compile_scene_document_from_path(root.join("scenes/main/scene.yml"), &root, "test-mod")
            .expect_err("duplicate entity ids should fail");

    assert!(matches!(error, SceneDocumentError::Compile { .. }));
}

#[test]
fn scene_compiler_expands_ui_refs() {
    let root = scene_compiler_temp_dir("ui_refs");
    write_scene_file(
        &root,
        "scenes/main/scene.yml",
        r#"
version: 1
scene:
  id: main
  label: Main
entities:
  - id: ui
    name: UI
    components:
      - type: UiDocumentRef
        asset: mod:ui/menus/main-menu
      - type: UiThemeRef
        asset: mod:ui/themes/rotten-noir
      - type: UiModelBindingsRef
        source: ./bindings.yml
"#,
    );
    write_scene_file(
        &root,
        "ui/menus/main-menu.yml",
        r#"
kind: ui-main-menu
id: main-menu
target:
  type: screen-space
  layer: menu
root:
  type: column
  id: root
"#,
    );
    write_scene_file(
        &root,
        "ui/themes/rotten-noir.yml",
        r#"
kind: ui-theme
id: rotten_noir
palette:
  background: '#000000FF'
  surface: '#111111FF'
  surface_alt: '#222222FF'
  text: '#FFFFFFFF'
  text_muted: '#AAAAAAFF'
  border: '#333333FF'
  accent: '#FF0000FF'
  accent_text: '#000000FF'
  danger: '#FF0000FF'
  warning: '#FFFF00FF'
  success: '#00FF00FF'
"#,
    );
    write_scene_file(
        &root,
        "scenes/main/bindings.yml",
        r#"
bindings:
  - path: start.text
    state: menu.start
    kind: text
"#,
    );

    let compiled =
        compile_scene_document_from_path(root.join("scenes/main/scene.yml"), &root, "test-mod")
            .expect("scene should compile");

    let kinds = compiled.document.entities[0]
        .components
        .iter()
        .map(SceneComponentDocument::kind)
        .collect::<Vec<_>>();
    assert_eq!(kinds, ["UiDocument", "UiThemeSet", "UiModelBindings"]);
}

#[test]
fn scene_compiler_rejects_unsafe_use_paths() {
    let root = scene_compiler_temp_dir("unsafe_path");
    write_scene_file(
        &root,
        "scenes/main/scene.yml",
        r#"
version: 1
scene:
  id: main
  label: Main
use:
  entities: ../outside.yml
"#,
    );

    let error =
        compile_scene_document_from_path(root.join("scenes/main/scene.yml"), &root, "test-mod")
            .expect_err("unsafe path should fail");

    assert!(matches!(error, SceneDocumentError::Compile { .. }));
}

#[test]
fn scene_compiler_compiles_rotten_club_main_menu_from_disk() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("mods/rotten-club");

    let compiled = compile_scene_document_from_path(
        root.join("scenes/main-menu/scene.yml"),
        &root,
        "rotten-club",
    )
    .expect("rotten-club main-menu should compile");

    assert_eq!(compiled.document.scene.id, "main-menu");
    assert!(
        compiled
            .document
            .entities
            .iter()
            .any(|entity| entity.id == "main-menu-camera" && entity.name == "main-menu-camera")
    );
    assert!(
        compiled
            .document
            .entities
            .iter()
            .any(|entity| entity.id == "main-menu-background" && entity.name == "background")
    );
    assert!(
        compiled
            .document
            .entities
            .iter()
            .any(|entity| entity.id == "main-menu-ui")
    );
    assert!(
        compiled
            .document
            .entities
            .iter()
            .flat_map(|entity| entity.components.iter())
            .any(|component| component.kind() == "UiDocument")
    );
    assert!(
        compiled
            .document
            .visual2d
            .render_layers
            .iter()
            .any(|layer| layer.id == "background.city")
    );
    assert!(
        compiled
            .document
            .visual2d
            .render_layers
            .iter()
            .any(|layer| layer.id == "weather.rain.1m"
                && matches!(layer.depth.mode, RenderDepthMode2dDocument::Distance)
                && (layer.depth.distance_m.unwrap_or_default() - 1.0).abs() < f32::EPSILON)
    );
    assert!(
        compiled
            .document
            .visual2d
            .render_layers
            .iter()
            .any(|layer| layer.id == "title.depth2d"
                && matches!(layer.depth.mode, RenderDepthMode2dDocument::Distance)
                && (layer.depth.distance_m.unwrap_or_default() - 1.0).abs() < f32::EPSILON
                && (layer.depth.blur_scale - 1.0).abs() < f32::EPSILON)
    );
    assert!(compiled.document.entities.iter().any(|entity| {
        entity.id == "main-menu-title"
            && entity
                .components
                .iter()
                .any(|component| component.kind() == "Text2D")
    }));
    assert!(
        compiled
            .document
            .visual2d
            .light_groups
            .iter()
            .any(|group| group.id == "lightning")
    );
    assert!(
        compiled
            .document
            .visual2d
            .light_routes
            .iter()
            .any(|route| route.receiver_layer == "weather.rain.1m")
    );
}

fn scene_compiler_temp_dir(name: &str) -> PathBuf {
    let id = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "amigo_scene_compiler_{name}_{}_{}",
        std::process::id(),
        id
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("old temp dir should be removable");
    }
    fs::create_dir_all(&path).expect("temp dir should be created");
    path
}

fn write_scene_file(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("file should have parent"))
        .expect("parent dir should be created");
    fs::write(path, content).expect("test file should be written");
}
