use serde_yaml::Value;

use super::scene_hierarchy_from_value;

#[test]
fn scene_hierarchy_dto_exposes_component_and_entity_traits() {
    let value = serde_yaml::from_str::<Value>(
        r#"
version: 1
scene:
  id: trait-test
  label: Trait Test
entities:
  - id: sprite
    name: Sprite
    transform2:
      translation:
        x: 10.0
        y: 20.0
    components:
      - type: Sprite2D
        texture: player
        size:
          x: 32.0
          y: 32.0
        z_index: 10
"#,
    )
    .expect("test YAML should parse");

    let hierarchy =
        scene_hierarchy_from_value("test-mod".to_owned(), "trait-test".to_owned(), &value)
            .expect("scene hierarchy should build");
    let entity = hierarchy
        .entities
        .iter()
        .find(|entity| entity.id == "sprite")
        .expect("sprite entity should exist");
    let component = entity
        .components
        .iter()
        .find(|component| component.type_name == "Sprite2D")
        .expect("Sprite2D component should exist");

    assert!(
        component
            .metadata_traits
            .contains(&"Renderable2D".to_owned())
    );
    assert!(
        component
            .metadata_traits
            .contains(&"HasAssetRefs".to_owned())
    );
    assert!(
        component
            .metadata_traits
            .contains(&"HasBounds2D".to_owned())
    );

    assert!(component.properties.iter().any(|property| {
        property.path == "texture"
            && property.trait_kind.as_deref() == Some("HasAssetRefs")
            && property.group == "assetRefs.primary"
    }));
    assert!(component.properties.iter().any(|property| {
        property.path == "size"
            && property.trait_kind.as_deref() == Some("HasBounds2D")
            && property.group == "bounds2.size"
    }));
    assert!(component.asset_refs.iter().any(|asset_ref| {
        asset_ref.field_path == "texture"
            && asset_ref.trait_kind == "HasAssetRefs"
            && asset_ref.group == "assetRefs.primary"
    }));

    assert!(entity.own_traits.contains(&"HasIdentity".to_owned()));
    assert!(entity.own_traits.contains(&"HasVisibility".to_owned()));
    assert!(entity.own_traits.contains(&"HasComponents".to_owned()));
    assert!(
        entity
            .derived_traits
            .contains(&"Transformable2D".to_owned())
    );
    assert!(entity.derived_traits.contains(&"Renderable2D".to_owned()));
    assert!(entity.derived_traits.contains(&"HasBounds2D".to_owned()));
    assert!(entity.derived_traits.contains(&"HasAssetRefs".to_owned()));
    assert!(entity.metadata_traits.contains(&"HasIdentity".to_owned()));
    assert!(
        entity
            .metadata_traits
            .contains(&"Transformable2D".to_owned())
    );
    assert!(entity.metadata_traits.contains(&"Renderable2D".to_owned()));
}

#[test]
fn scene_hierarchy_dto_exposes_collider_and_ui_traits() {
    let value = serde_yaml::from_str::<Value>(
        r#"
version: 1
scene:
  id: trait-test
  label: Trait Test
entities:
  - id: collider
    name: Collider
    transform2:
      translation:
        x: 0.0
        y: 0.0
    components:
      - type: AabbCollider2D
        size:
          x: 16.0
          y: 8.0
        layer: world
        mask:
          - player
  - id: hud
    name: HUD
    components:
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
          viewport:
            width: 1280.0
            height: 720.0
            scaling: fit
        root:
          type: panel
          id: root
          style: {}
"#,
    )
    .expect("test YAML should parse");

    let hierarchy =
        scene_hierarchy_from_value("test-mod".to_owned(), "trait-test".to_owned(), &value)
            .expect("scene hierarchy should build");
    let collider_entity = hierarchy
        .entities
        .iter()
        .find(|entity| entity.id == "collider")
        .expect("collider entity should exist");
    let collider = collider_entity
        .components
        .iter()
        .find(|component| component.type_name == "AabbCollider2D")
        .expect("AabbCollider2D component should exist");
    let hud_entity = hierarchy
        .entities
        .iter()
        .find(|entity| entity.id == "hud")
        .expect("hud entity should exist");
    let ui_document = hud_entity
        .components
        .iter()
        .find(|component| component.type_name == "UiDocument")
        .expect("UiDocument component should exist");

    assert!(
        collider
            .metadata_traits
            .contains(&"Collidable2D".to_owned())
    );
    assert!(collider.metadata_traits.contains(&"HasBounds2D".to_owned()));
    assert!(collider.properties.iter().any(|property| {
        property.path == "size"
            && property.trait_kind.as_deref() == Some("HasBounds2D")
            && property.group == "bounds2.size"
    }));
    assert!(
        collider_entity
            .derived_traits
            .contains(&"Collidable2D".to_owned())
    );
    assert!(
        ui_document
            .metadata_traits
            .contains(&"UiEditable".to_owned())
    );
    assert!(
        ui_document
            .metadata_traits
            .contains(&"HasUiTree".to_owned())
    );
    assert!(hud_entity.derived_traits.contains(&"UiEditable".to_owned()));
    assert!(hud_entity.derived_traits.contains(&"HasUiTree".to_owned()));
}
