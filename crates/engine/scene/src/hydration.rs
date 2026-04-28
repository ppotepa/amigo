use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};

use crate::{
    AabbCollider2dSceneCommand, CameraFollow2dSceneCommand, KinematicBody2dSceneCommand,
    Material3dSceneCommand, Mesh3dSceneCommand, Parallax2dSceneCommand,
    PlatformerController2dSceneCommand, SceneCommand, SceneComponentDocument, SceneDocument,
    SceneDocumentError, SceneDocumentResult, SceneEntityDocument, SceneKey,
    SceneSpriteSheetDocument, SceneTransform2Document, SceneTransform3Document, SceneUiDocument,
    SceneUiEventBinding, SceneUiEventBindingComponentDocument, SceneUiLayer, SceneUiNode,
    SceneUiNodeComponentDocument, SceneUiNodeKind, SceneUiNodeTypeComponentDocument, SceneUiStyle,
    SceneUiStyleComponentDocument, SceneUiTarget, SceneUiTargetComponentDocument,
    SceneUiTargetTypeComponentDocument, Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
    SpriteSheet2dSceneCommand, Text2dSceneCommand, Text3dSceneCommand, TileMap2dSceneCommand,
    TileMapMarker2dSceneCommand, Trigger2dSceneCommand, UiSceneCommand,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneHydrationPlan {
    pub commands: Vec<SceneCommand>,
}

pub fn build_scene_hydration_plan(
    source_mod: &str,
    document: &SceneDocument,
) -> SceneDocumentResult<SceneHydrationPlan> {
    let mut commands = Vec::new();

    for entity in &document.entities {
        let entity_name = entity.display_name();
        commands.push(SceneCommand::SpawnNamedEntity {
            name: entity_name.clone(),
            transform: Some(transform3_for_entity(entity)),
        });

        for component in &entity.components {
            match component {
                SceneComponentDocument::Camera2d
                | SceneComponentDocument::Camera3d
                | SceneComponentDocument::Light3d { .. } => {}
                SceneComponentDocument::Sprite2d {
                    texture,
                    size,
                    sheet,
                    animation,
                    z_index,
                } => {
                    commands.push(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            texture: AssetKey::new(texture.clone()),
                            size: vec2_from_document(*size),
                            sheet: sheet.map(sprite_sheet_from_document),
                            animation: animation.map(sprite_animation_from_document),
                            z_index: *z_index,
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::TileMap2d {
                    tileset,
                    ruleset,
                    tile_size,
                    grid,
                    z_index,
                } => {
                    let mut command = TileMap2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        AssetKey::new(tileset.clone()),
                        vec2_from_document(*tile_size),
                        grid.clone(),
                    );
                    command.ruleset = ruleset.clone().map(AssetKey::new);
                    command.z_index = *z_index;
                    commands.push(SceneCommand::QueueTileMap2d { command });
                }
                SceneComponentDocument::Text2d {
                    content,
                    font,
                    bounds,
                } => {
                    commands.push(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            content: content.clone(),
                            font: AssetKey::new(font.clone()),
                            bounds: vec2_from_document(*bounds),
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::KinematicBody2d {
                    velocity,
                    gravity_scale,
                    terminal_velocity,
                } => {
                    commands.push(SceneCommand::QueueKinematicBody2d {
                        command: KinematicBody2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*velocity),
                            *gravity_scale,
                            *terminal_velocity,
                        ),
                    });
                }
                SceneComponentDocument::AabbCollider2d {
                    size,
                    offset,
                    layer,
                    mask,
                } => {
                    commands.push(SceneCommand::QueueAabbCollider2d {
                        command: AabbCollider2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*size),
                            vec2_from_document(*offset),
                            layer.clone(),
                            mask.clone(),
                        ),
                    });
                }
                SceneComponentDocument::Trigger2d {
                    size,
                    offset,
                    layer,
                    mask,
                    event,
                } => {
                    commands.push(SceneCommand::QueueTrigger2d {
                        command: Trigger2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*size),
                            vec2_from_document(*offset),
                            layer.clone(),
                            mask.clone(),
                            event.clone(),
                        ),
                    });
                }
                SceneComponentDocument::PlatformerController2d {
                    max_speed,
                    acceleration,
                    deceleration,
                    air_acceleration,
                    gravity,
                    jump_velocity,
                    terminal_velocity,
                } => {
                    commands.push(SceneCommand::QueuePlatformerController2d {
                        command: PlatformerController2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            *max_speed,
                            *acceleration,
                            *deceleration,
                            *air_acceleration,
                            *gravity,
                            *jump_velocity,
                            *terminal_velocity,
                        ),
                    });
                }
                SceneComponentDocument::CameraFollow2d {
                    target,
                    offset,
                    lerp,
                } => {
                    commands.push(SceneCommand::QueueCameraFollow2d {
                        command: CameraFollow2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            target.clone(),
                            vec2_from_document(*offset),
                            *lerp,
                        ),
                    });
                }
                SceneComponentDocument::Parallax2d { camera, factor } => {
                    commands.push(SceneCommand::QueueParallax2d {
                        command: Parallax2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            camera.clone(),
                            vec2_from_document(*factor),
                            transform2_for_entity(entity).translation,
                        ),
                    });
                }
                SceneComponentDocument::TileMapMarker2d {
                    symbol,
                    tilemap_entity,
                    index,
                    offset,
                } => {
                    commands.push(SceneCommand::QueueTileMapMarker2d {
                        command: TileMapMarker2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            tilemap_entity.clone(),
                            symbol.clone(),
                            *index,
                            vec2_from_document(*offset),
                        ),
                    });
                }
                SceneComponentDocument::Mesh3d { mesh } => {
                    commands.push(SceneCommand::QueueMesh3d {
                        command: Mesh3dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            mesh_asset: AssetKey::new(mesh.clone()),
                            transform: transform3_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::Material3d {
                    label,
                    source,
                    albedo,
                } => {
                    let mut command = Material3dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        label.clone(),
                        source.as_ref().map(AssetKey::new),
                    );

                    if let Some(albedo) = albedo.as_deref() {
                        command.albedo = parse_color_rgba_hex(
                            albedo,
                            &document.scene.id,
                            &entity.id,
                            component.kind(),
                        )?;
                    }

                    commands.push(SceneCommand::QueueMaterial3d { command });
                }
                SceneComponentDocument::Text3d {
                    content,
                    font,
                    size,
                } => {
                    commands.push(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            content: content.clone(),
                            font: AssetKey::new(font.clone()),
                            size: *size,
                            transform: transform3_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::UiDocument { target, root } => {
                    commands.push(SceneCommand::QueueUi {
                        command: UiSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            document: ui_document_from_component(
                                target,
                                root,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )?,
                        },
                    });
                }
            }
        }
    }

    Ok(SceneHydrationPlan { commands })
}

pub fn scene_key_from_document(document: &SceneDocument) -> SceneKey {
    SceneKey::new(document.scene.id.clone())
}

fn transform2_for_entity(entity: &SceneEntityDocument) -> Transform2 {
    entity
        .transform2
        .map(transform2_from_document)
        .or_else(|| entity.transform3.map(transform2_from_transform3_document))
        .unwrap_or_default()
}

fn transform3_for_entity(entity: &SceneEntityDocument) -> Transform3 {
    entity
        .transform3
        .map(transform3_from_document)
        .or_else(|| entity.transform2.map(transform3_from_transform2_document))
        .unwrap_or_default()
}

fn transform2_from_document(document: SceneTransform2Document) -> Transform2 {
    Transform2 {
        translation: vec2_from_document(document.translation),
        rotation_radians: document.rotation_radians,
        scale: vec2_from_document(document.scale),
    }
}

fn transform3_from_document(document: SceneTransform3Document) -> Transform3 {
    Transform3 {
        translation: vec3_from_document(document.translation),
        rotation_euler: vec3_from_document(document.rotation_euler),
        scale: vec3_from_document(document.scale),
    }
}

fn transform3_from_transform2_document(document: SceneTransform2Document) -> Transform3 {
    Transform3 {
        translation: Vec3::new(document.translation.x, document.translation.y, 0.0),
        rotation_euler: Vec3::new(0.0, 0.0, document.rotation_radians),
        scale: Vec3::new(document.scale.x, document.scale.y, 1.0),
    }
}

fn transform2_from_transform3_document(document: SceneTransform3Document) -> Transform2 {
    Transform2 {
        translation: Vec2::new(document.translation.x, document.translation.y),
        rotation_radians: document.rotation_euler.z,
        scale: Vec2::new(document.scale.x, document.scale.y),
    }
}

fn vec2_from_document(value: crate::SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

fn vec3_from_document(value: crate::SceneVec3Document) -> Vec3 {
    Vec3::new(value.x, value.y, value.z)
}

fn sprite_sheet_from_document(value: SceneSpriteSheetDocument) -> SpriteSheet2dSceneCommand {
    SpriteSheet2dSceneCommand {
        columns: value.columns.max(1),
        rows: value.rows.max(1),
        frame_count: value.frame_count.max(1),
        frame_size: vec2_from_document(value.frame_size),
        fps: value.fps.max(0.0),
        looping: value.looping,
    }
}

fn sprite_animation_from_document(
    value: crate::SceneSpriteAnimationDocument,
) -> SpriteAnimation2dSceneOverride {
    SpriteAnimation2dSceneOverride {
        fps: value.fps.map(|fps| fps.max(0.0)),
        looping: value.looping,
        start_frame: value.start_frame,
    }
}

fn ui_document_from_component(
    target: &SceneUiTargetComponentDocument,
    root: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiDocument> {
    Ok(SceneUiDocument {
        target: ui_target_from_component(target),
        root: ui_node_from_component(root, scene_id, entity_id, component_kind)?,
    })
}

fn ui_target_from_component(target: &SceneUiTargetComponentDocument) -> SceneUiTarget {
    match target.kind {
        SceneUiTargetTypeComponentDocument::ScreenSpace => SceneUiTarget::ScreenSpace {
            layer: match target.layer {
                crate::SceneUiLayerComponentDocument::Background => SceneUiLayer::Background,
                crate::SceneUiLayerComponentDocument::Hud => SceneUiLayer::Hud,
                crate::SceneUiLayerComponentDocument::Menu => SceneUiLayer::Menu,
                crate::SceneUiLayerComponentDocument::Debug => SceneUiLayer::Debug,
            },
        },
    }
}

fn ui_node_from_component(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiNode> {
    let kind = match node.kind {
        SceneUiNodeTypeComponentDocument::Panel => SceneUiNodeKind::Panel,
        SceneUiNodeTypeComponentDocument::Row => SceneUiNodeKind::Row,
        SceneUiNodeTypeComponentDocument::Column => SceneUiNodeKind::Column,
        SceneUiNodeTypeComponentDocument::Stack => SceneUiNodeKind::Stack,
        SceneUiNodeTypeComponentDocument::Text => SceneUiNodeKind::Text {
            content: required_ui_text(node, scene_id, entity_id, component_kind, "text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Button => SceneUiNodeKind::Button {
            text: required_ui_text(node, scene_id, entity_id, component_kind, "button text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::ProgressBar => SceneUiNodeKind::ProgressBar {
            value: node.value.unwrap_or(0.0).clamp(0.0, 1.0),
        },
        SceneUiNodeTypeComponentDocument::Spacer => SceneUiNodeKind::Spacer,
    };

    Ok(SceneUiNode {
        id: node.id.clone(),
        kind,
        style: ui_style_from_component(&node.style, scene_id, entity_id, component_kind)?,
        on_click: node.on_click.as_ref().map(ui_event_binding_from_component),
        children: node
            .children
            .iter()
            .map(|child| ui_node_from_component(child, scene_id, entity_id, component_kind))
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}

fn required_ui_text(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    label: &str,
) -> SceneDocumentResult<String> {
    node.text
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!("expected UI node to define non-empty `{label}` content"),
        })
}

fn ui_style_from_component(
    style: &SceneUiStyleComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiStyle> {
    Ok(SceneUiStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: parse_optional_color_rgba_hex(
            style.background.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "background",
        )?,
        color: parse_optional_color_rgba_hex(
            style.color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "color",
        )?,
        border_color: parse_optional_color_rgba_hex(
            style.border_color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "border_color",
        )?,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
    })
}

fn ui_event_binding_from_component(
    binding: &SceneUiEventBindingComponentDocument,
) -> SceneUiEventBinding {
    SceneUiEventBinding {
        event: binding.event.clone(),
        payload: binding.payload.clone(),
    }
}

fn parse_optional_color_rgba_hex(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    _field_name: &str,
) -> SceneDocumentResult<Option<ColorRgba>> {
    value
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()
}

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!(
                    "expected albedo color `{value}` to use #RRGGBB or #RRGGBBAA syntax"
                ),
            });
        }
    };

    Ok(ColorRgba::new(
        channel_to_f32(r),
        channel_to_f32(g),
        channel_to_f32(b),
        channel_to_f32(a),
    ))
}

fn parse_hex_channel(
    channel: &str,
    raw_value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|_| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("expected albedo color `{raw_value}` to contain only hex digits"),
    })
}

fn channel_to_f32(value: u8) -> f32 {
    f32::from(value) / 255.0
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};

    use super::{build_scene_hydration_plan, scene_key_from_document};
    use crate::{SceneCommand, load_scene_document_from_path, load_scene_document_from_str};

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
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert_eq!(scene_key_from_document(&document).as_str(), "sprite-lab");
        assert_eq!(plan.commands.len(), 3);
        assert!(matches!(
            &plan.commands[0],
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { .. })
            } if name == "playground-2d-camera"
        ));
        assert!(matches!(
            &plan.commands[2],
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-sprite"
                    && command.size == Vec2::new(128.0, 128.0)
                    && command.transform == Transform2 {
                        translation: Vec2::new(12.0, -4.0),
                        rotation_radians: 0.5,
                        scale: Vec2::new(2.0, 3.0),
                    }
        ));
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

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(matches!(
            &plan.commands[2],
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { translation, scale, .. })
            } if name == "playground-3d-material-probe"
                && *translation == Vec3::ZERO
                && *scale == Vec3::ONE
        ));
        assert!(matches!(
            &plan.commands[4],
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-material-probe"
                    && command.label == "debug-surface"
                    && command.albedo == ColorRgba::WHITE
        ));
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

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-spritesheet"
                    && command.sheet.as_ref().map(|sheet| sheet.frame_count) == Some(8)
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText2d { command }
                if command.entity_name == "playground-2d-hello"
        )));
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

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMesh3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText3d { command }
                if command.entity_name == "playground-3d-hello"
        )));
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

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-ui-preview-square"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.entity_name == "playground-2d-ui-preview"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_sidescroller_components() {
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
        tileset: playground-sidescroller/tilesets/platformer
        ruleset: playground-sidescroller/tilesets/platformer-rules
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
      - type: PlatformerController2D
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
        texture: playground-sidescroller/textures/coin
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
        .expect("sidescroller scene should parse");

        let plan = build_scene_hydration_plan("playground-sidescroller", &document)
            .expect("sidescroller hydration plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMap2d { command }
                if command.entity_name == "playground-sidescroller-tilemap"
                    && command.ruleset.as_ref().map(|ruleset| ruleset.as_str())
                        == Some("playground-sidescroller/tilesets/platformer-rules")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueKinematicBody2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueAabbCollider2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueuePlatformerController2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMapMarker2d { command }
                if command.entity_name == "playground-sidescroller-player"
                    && command.symbol == "P"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueCameraFollow2d { command }
                if command.entity_name == "playground-sidescroller-camera"
                    && command.target == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTrigger2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.event.as_deref() == Some("coin.collected")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.animation.as_ref().and_then(|animation| animation.fps) == Some(10.0)
                    && command.animation.as_ref().and_then(|animation| animation.looping) == Some(true)
        )));
    }
}
