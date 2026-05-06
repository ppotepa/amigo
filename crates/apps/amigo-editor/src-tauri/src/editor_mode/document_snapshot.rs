use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::{Mapping, Value};

use amigo_scene::{
    ComponentCapability, ComponentKind, ComponentRegistry, default_component_registry,
};

use crate::dto::DiagnosticLevel;
use crate::editor_mode::dto::{
    EditorBounds2Dto, EditorCameraDto, EditorObjectEditCommandKindDto,
    EditorObjectPlacementKindDto, EditorPrefabInstanceDto, EditorSceneCanvasKindDto,
    EditorSceneObjectDto, EditorSceneSnapshotDto, EditorSceneSnapshotLayoutSourceDto,
    EditorTransform2Dto, EditorTransform3Dto, EditorUiNodeKindDto, EditorUiNodeObjectDto,
};
use crate::editor_mode::gizmos::{default_selection, default_tool_state};

mod quality;
#[cfg(test)]
mod tests;
mod yaml;

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;
const ICON_BOUNDS_SIZE: f32 = 32.0;
use quality::{append_object_quality_diagnostics, push_diagnostic, snapshot_quality};
use yaml::{bool_field, mapping, number_field, string_field};

pub(super) const DIAG_ENTITY_SKIPPED: &str = "EDITOR_MODE_ENTITY_SKIPPED";
pub(super) const DIAG_NO_DOCUMENT_BOUNDS: &str = "EDITOR_MODE_NO_DOCUMENT_BOUNDS";
pub(super) const DIAG_ENTITY_NO_TRANSFORM2: &str = "ENTITY_NO_TRANSFORM2";
pub(super) const DIAG_ENTITY_NO_BOUNDS2: &str = "ENTITY_NO_BOUNDS2";
pub(super) const DIAG_COMPONENT_BOUNDS_UNSUPPORTED: &str = "COMPONENT_BOUNDS_UNSUPPORTED";
const DIAG_DOCUMENT_PARSE_FAILED: &str = "DOCUMENT_PARSE_FAILED";
pub(super) const DIAG_ENTITY_LOCKED_BY_PLACEMENT: &str = "ENTITY_LOCKED_BY_PLACEMENT";

pub fn document_editor_snapshot(
    mod_id: String,
    root_path: impl AsRef<Path>,
    scene_id: String,
) -> Result<EditorSceneSnapshotDto, String> {
    let scene_path = scene_document_path(root_path.as_ref(), &scene_id);
    let text = fs::read_to_string(&scene_path).map_err(|error| {
        format!(
            "failed to read scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    let value = serde_yaml::from_str::<Value>(&text).map_err(|error| {
        format!(
            "{DIAG_DOCUMENT_PARSE_FAILED}: failed to parse scene document `{}`: {error}",
            scene_path.display()
        )
    })?;

    snapshot_from_scene_value(mod_id, scene_id, &value)
}

pub fn snapshot_from_scene_value(
    mod_id: String,
    scene_id: String,
    value: &Value,
) -> Result<EditorSceneSnapshotDto, String> {
    let component_registry = default_component_registry();
    let entities =
        scene_entities(value).ok_or_else(|| "scene document has no entities array".to_owned())?;
    let mut diagnostics = Vec::new();
    let mut objects = Vec::new();

    for (index, entity_value) in entities.iter().enumerate() {
        match object_from_entity_value(entity_value, &component_registry) {
            Some(object) => objects.push(object),
            None => push_diagnostic(
                &mut diagnostics,
                DiagnosticLevel::Info,
                DIAG_ENTITY_SKIPPED,
                format!(
                    "Scene entity at index {index} could not be converted into an editor object."
                ),
            ),
        }
    }
    append_object_quality_diagnostics(&objects, &mut diagnostics);
    let ui_nodes = ui_node_objects_from_scene_value(value);

    if objects.is_empty() {
        push_diagnostic(
            &mut diagnostics,
            DiagnosticLevel::Info,
            DIAG_NO_DOCUMENT_BOUNDS,
            "Scene document was parsed, but no entity exposes transform/bounds data yet.",
        );
    }

    let canvas_kind = infer_canvas_kind_from_objects(&objects);
    let camera = camera_from_objects(&objects);
    Ok(EditorSceneSnapshotDto {
        mod_id,
        scene_id,
        canvas_kind,
        layout_source: EditorSceneSnapshotLayoutSourceDto::Document,
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        camera,
        quality: snapshot_quality(entities.len(), &objects, &diagnostics),
        objects,
        ui_nodes,
        diagnostics,
        gizmos: Vec::new(),
        selection: default_selection(),
        tool_state: default_tool_state(),
    })
}

pub fn scene_document_path(root_path: &Path, scene_id: &str) -> PathBuf {
    root_path.join("scenes").join(scene_id).join("scene.yml")
}

fn scene_entities(value: &Value) -> Option<&Vec<Value>> {
    let root = value.as_mapping()?;
    root.get(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence)
        .or_else(|| {
            root.get(Value::String("scene".to_owned()))
                .and_then(Value::as_mapping)
                .and_then(|scene| {
                    scene
                        .get(Value::String("entities".to_owned()))
                        .and_then(Value::as_sequence)
                })
        })
}

fn object_from_entity_value(
    entity_value: &Value,
    component_registry: &ComponentRegistry,
) -> Option<EditorSceneObjectDto> {
    let entity = mapping(entity_value)?;
    let entity_id = string_field(entity, "id")?;
    let name = string_field(entity, "name").unwrap_or_else(|| entity_id.clone());
    let visible = bool_field(entity, "visible").unwrap_or(true);
    let components = entity
        .get(Value::String("components".to_owned()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let component_types = components
        .iter()
        .filter_map(component_type)
        .collect::<Vec<_>>();
    let component_kinds = component_types
        .iter()
        .filter_map(|kind| component_kind_from_type_name(kind.as_str()))
        .collect::<Vec<_>>();
    let transform_2 = entity
        .get(Value::String("transform2".to_owned()))
        .and_then(transform2_from_value);
    let transform_3 = entity
        .get(Value::String("transform3".to_owned()))
        .and_then(transform3_from_value);
    let prefab_instance = prefab_instance_from_entity(entity, &entity_id);
    let category = object_category(&component_kinds, transform_3.is_some(), &component_types);
    let placement_kind = placement_kind_for_entity(
        entity,
        &components,
        transform_2.as_ref(),
        transform_3.as_ref(),
    );
    let edit_command_kind = edit_command_kind_for_placement(placement_kind);
    let effective_transform_2 = match placement_kind {
        EditorObjectPlacementKindDto::Transform2 => transform_2.clone(),
        EditorObjectPlacementKindDto::TilemapMarker => tilemap_marker_transform2(&components),
        EditorObjectPlacementKindDto::Attached => attached_object_transform2(&components),
        EditorObjectPlacementKindDto::UiLayout => ui_layout_transform2(&components),
        EditorObjectPlacementKindDto::ComputedRuntime
        | EditorObjectPlacementKindDto::NotEditable => transform_2.clone(),
    };
    let render_bounds_2 =
        bounds2_for_entity(&components, effective_transform_2.as_ref(), &category);
    let selection_bounds_2 = selection_bounds_for_entity(
        render_bounds_2.as_ref(),
        effective_transform_2.as_ref(),
        placement_kind,
    );
    let bounds_2 = selection_bounds_2.clone();
    let selectable_capability = component_kinds
        .iter()
        .any(|kind| component_registry.has_capability(*kind, ComponentCapability::Selectable));
    let has_editor_control = component_kinds.iter().any(|kind| {
        component_registry.has_capability(*kind, ComponentCapability::HasEditorControl)
    });
    let transformable_2d = transform_2.is_some()
        && component_kinds.iter().any(|kind| {
            component_registry.has_capability(*kind, ComponentCapability::Transformable2D)
        });
    if !selectable_capability && !has_editor_control {
        return None;
    }

    let selectable = visible && selectable_capability && selection_bounds_2.is_some();
    let movable = visible
        && selectable
        && transformable_2d
        && matches!(
            edit_command_kind,
            EditorObjectEditCommandKindDto::SetTransform2
                | EditorObjectEditCommandKindDto::SetTilemapMarkerOffset
                | EditorObjectEditCommandKindDto::SetAttachedLocalOffset
                | EditorObjectEditCommandKindDto::SetUiRect
                | EditorObjectEditCommandKindDto::SetTilemapOrigin
        );
    let locked = !movable;
    let locked_reason =
        locked_reason_for_object(placement_kind, edit_command_kind, selectable, movable);

    Some(EditorSceneObjectDto {
        entity_id,
        name,
        visible,
        selectable,
        locked,
        movable,
        locked_reason,
        category,
        component_types,
        placement_kind,
        edit_command_kind,
        transform_2: effective_transform_2,
        transform_3,
        bounds_2,
        render_bounds_2,
        selection_bounds_2,
        prefab_instance,
    })
}

fn prefab_instance_from_entity(
    entity: &Mapping,
    entity_id: &str,
) -> Option<EditorPrefabInstanceDto> {
    let prefab = entity
        .get(Value::String("prefab".to_owned()))
        .and_then(mapping)?;
    let prefab_id = string_field(prefab, "id")?;
    Some(EditorPrefabInstanceDto {
        prefab_id,
        root_entity_id: entity_id.to_owned(),
        is_prefab_root: true,
        source_entity_id: None,
    })
}

fn component_kind_from_type_name(type_name: &str) -> Option<ComponentKind> {
    match type_name {
        "Camera2D" | "camera2d" => Some(ComponentKind::Camera2D),
        "Camera3D" | "camera3d" => Some(ComponentKind::Camera3D),
        "Light3D" | "light3d" => Some(ComponentKind::Light3D),
        "Sprite2D" | "sprite2d" | "sprite" => Some(ComponentKind::Sprite2D),
        "TileMap2D" | "tilemap2d" | "tilemap" => Some(ComponentKind::TileMap2D),
        "Text2D" | "text2d" | "text" => Some(ComponentKind::Text2D),
        "VectorShape2D" | "vector2d" | "vector" => Some(ComponentKind::VectorShape2D),
        "EntityPool" => Some(ComponentKind::EntityPool),
        "Lifetime" => Some(ComponentKind::Lifetime),
        "ProjectileEmitter2D" => Some(ComponentKind::ProjectileEmitter2D),
        "InputActionMap" => Some(ComponentKind::InputActionMap),
        "Behavior" => Some(ComponentKind::Behavior),
        "EventPipeline" => Some(ComponentKind::EventPipeline),
        "UiModelBindings" => Some(ComponentKind::UiModelBindings),
        "ScriptComponent" | "scriptcomponent" => Some(ComponentKind::ScriptComponent),
        "ParticleEmitter2D" => Some(ComponentKind::ParticleEmitter2D),
        "Velocity2D" => Some(ComponentKind::Velocity2D),
        "Bounds2D" => Some(ComponentKind::Bounds2D),
        "FreeflightMotion2D" => Some(ComponentKind::FreeflightMotion2D),
        "KinematicBody2D" => Some(ComponentKind::KinematicBody2D),
        "AabbCollider2D" => Some(ComponentKind::AabbCollider2D),
        "StaticCollider2D" => Some(ComponentKind::StaticCollider2D),
        "CircleCollider2D" => Some(ComponentKind::CircleCollider2D),
        "Trigger2D" => Some(ComponentKind::Trigger2D),
        "MotionController2D" => Some(ComponentKind::MotionController2D),
        "CameraFollow2D" => Some(ComponentKind::CameraFollow2D),
        "Parallax2D" => Some(ComponentKind::Parallax2D),
        "TileMapMarker2D" => Some(ComponentKind::TileMapMarker2D),
        "Mesh3D" | "mesh3d" => Some(ComponentKind::Mesh3D),
        "Material3D" | "material3d" => Some(ComponentKind::Material3D),
        "Text3D" | "text3d" => Some(ComponentKind::Text3D),
        "UiDocument" | "uidocument" => Some(ComponentKind::UiDocument),
        "UiThemeSet" | "uithemeset" => Some(ComponentKind::UiThemeSet),
        _ => None,
    }
}

fn placement_kind_for_entity(
    entity: &Mapping,
    components: &[Value],
    transform_2: Option<&EditorTransform2Dto>,
    transform_3: Option<&EditorTransform3Dto>,
) -> EditorObjectPlacementKindDto {
    if transform_3.is_some() {
        return EditorObjectPlacementKindDto::ComputedRuntime;
    }
    if components.iter().any(component_is_tilemap_marker) {
        return EditorObjectPlacementKindDto::TilemapMarker;
    }
    if components.iter().any(component_is_attached) {
        return EditorObjectPlacementKindDto::Attached;
    }
    if components.iter().any(component_is_ui_layout) {
        return EditorObjectPlacementKindDto::UiLayout;
    }
    if transform_2.is_some() {
        return EditorObjectPlacementKindDto::Transform2;
    }
    if entity.get(Value::String("components".to_owned())).is_some() {
        return EditorObjectPlacementKindDto::NotEditable;
    }
    EditorObjectPlacementKindDto::NotEditable
}

fn edit_command_kind_for_placement(
    placement: EditorObjectPlacementKindDto,
) -> EditorObjectEditCommandKindDto {
    match placement {
        EditorObjectPlacementKindDto::Transform2 => EditorObjectEditCommandKindDto::SetTransform2,
        EditorObjectPlacementKindDto::TilemapMarker => {
            EditorObjectEditCommandKindDto::SetTilemapMarkerOffset
        }
        EditorObjectPlacementKindDto::Attached => {
            EditorObjectEditCommandKindDto::SetAttachedLocalOffset
        }
        EditorObjectPlacementKindDto::UiLayout => EditorObjectEditCommandKindDto::SetUiRect,
        EditorObjectPlacementKindDto::ComputedRuntime
        | EditorObjectPlacementKindDto::NotEditable => EditorObjectEditCommandKindDto::Locked,
    }
}

fn component_is_tilemap_marker(component: &Value) -> bool {
    component_type(component)
        .map(|kind| kind.eq_ignore_ascii_case("TileMapMarker2D"))
        .unwrap_or(false)
}

fn component_is_attached(component: &Value) -> bool {
    mapping(component)
        .and_then(|component| {
            component
                .get(Value::String("attached_to".to_owned()))
                .and_then(Value::as_str)
        })
        .is_some()
}

fn component_is_ui_layout(component: &Value) -> bool {
    component_type(component)
        .map(|kind| {
            let kind = kind.to_lowercase();
            kind == "uidocument" || kind == "uinode" || kind == "ui" || kind.contains("layout")
        })
        .unwrap_or(false)
}

fn tilemap_marker_transform2(components: &[Value]) -> Option<EditorTransform2Dto> {
    let marker = components
        .iter()
        .find(|component| component_is_tilemap_marker(component))?;
    let marker = mapping(marker)?;
    let offset = marker
        .get(Value::String("offset".to_owned()))
        .and_then(mapping);

    Some(EditorTransform2Dto {
        x: offset
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: offset
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        z_index: None,
    })
}

fn attached_object_transform2(components: &[Value]) -> Option<EditorTransform2Dto> {
    let attached = components
        .iter()
        .find(|component| component_is_attached(component))?;
    let attached = mapping(attached)?;
    let offset = attached
        .get(Value::String("local_offset".to_owned()))
        .and_then(mapping);

    Some(EditorTransform2Dto {
        x: offset
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: offset
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation: number_field(attached, "local_direction_degrees")
            .map(|degrees| degrees.to_radians())
            .unwrap_or(0.0),
        scale_x: 1.0,
        scale_y: 1.0,
        z_index: number_field(attached, "z_index").map(|value| value as i32),
    })
}

fn ui_layout_transform2(components: &[Value]) -> Option<EditorTransform2Dto> {
    let ui = components
        .iter()
        .find(|component| component_is_ui_layout(component))?;
    let ui = mapping(ui)?;
    let rect = ui.get(Value::String("rect".to_owned())).and_then(mapping);

    Some(EditorTransform2Dto {
        x: rect
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: rect
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        z_index: number_field(ui, "z_index").map(|value| value as i32),
    })
}

fn selection_bounds_for_entity(
    render_bounds: Option<&EditorBounds2Dto>,
    transform: Option<&EditorTransform2Dto>,
    placement_kind: EditorObjectPlacementKindDto,
) -> Option<EditorBounds2Dto> {
    if let Some(bounds) = render_bounds {
        let padding = match placement_kind {
            EditorObjectPlacementKindDto::TilemapMarker => 10.0,
            EditorObjectPlacementKindDto::Attached => 8.0,
            EditorObjectPlacementKindDto::UiLayout => 4.0,
            _ => 0.0,
        };

        return Some(EditorBounds2Dto {
            x: bounds.x - padding,
            y: bounds.y - padding,
            width: bounds.width + padding * 2.0,
            height: bounds.height + padding * 2.0,
        });
    }

    let transform = transform?;
    let size = match placement_kind {
        EditorObjectPlacementKindDto::TilemapMarker => 48.0,
        EditorObjectPlacementKindDto::Attached => 32.0,
        _ => return None,
    };

    Some(EditorBounds2Dto {
        x: transform.x - size / 2.0,
        y: transform.y - size / 2.0,
        width: size,
        height: size,
    })
}

fn locked_reason_for_object(
    placement: EditorObjectPlacementKindDto,
    edit_command: EditorObjectEditCommandKindDto,
    selectable: bool,
    movable: bool,
) -> Option<String> {
    if movable {
        return None;
    }
    if !selectable {
        return Some("No selectable editor bounds are available for this entity.".to_owned());
    }
    if edit_command == EditorObjectEditCommandKindDto::Locked {
        return Some(format!(
            "No editor command is available for placement `{placement:?}`."
        ));
    }
    Some("Entity is selectable but not movable in the current 2D editor mode.".to_owned())
}

fn component_type(component: &Value) -> Option<String> {
    let component = mapping(component)?;
    string_field(component, "type").or_else(|| string_field(component, "kind"))
}

fn transform2_from_value(value: &Value) -> Option<EditorTransform2Dto> {
    let transform = mapping(value)?;
    let translation = transform
        .get(Value::String("translation".to_owned()))
        .and_then(mapping);
    let scale = transform
        .get(Value::String("scale".to_owned()))
        .and_then(mapping);

    Some(EditorTransform2Dto {
        x: translation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: translation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation: number_field(transform, "rotation_radians")
            .or_else(|| number_field(transform, "rotation"))
            .unwrap_or(0.0),
        scale_x: scale
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(1.0),
        scale_y: scale
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(1.0),
        z_index: number_field(transform, "z_index").map(|value| value as i32),
    })
}

fn transform3_from_value(value: &Value) -> Option<EditorTransform3Dto> {
    let transform = mapping(value)?;
    let translation = transform
        .get(Value::String("translation".to_owned()))
        .and_then(mapping);
    let rotation = transform
        .get(Value::String("rotation".to_owned()))
        .and_then(mapping);
    let scale = transform
        .get(Value::String("scale".to_owned()))
        .and_then(mapping);

    Some(EditorTransform3Dto {
        x: translation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        y: translation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        z: translation
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(0.0),
        rotation_x: rotation
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(0.0),
        rotation_y: rotation
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(0.0),
        rotation_z: rotation
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(0.0),
        scale_x: scale
            .and_then(|value| number_field(value, "x"))
            .unwrap_or(1.0),
        scale_y: scale
            .and_then(|value| number_field(value, "y"))
            .unwrap_or(1.0),
        scale_z: scale
            .and_then(|value| number_field(value, "z"))
            .unwrap_or(1.0),
    })
}

fn bounds2_for_entity(
    components: &[Value],
    transform: Option<&EditorTransform2Dto>,
    category: &str,
) -> Option<EditorBounds2Dto> {
    let transform = transform?;
    let mut best_size = None;

    for component in components {
        let Some(kind) = component_type(component) else {
            continue;
        };
        let Some(component) = mapping(component) else {
            continue;
        };

        let candidate = match kind.as_str() {
            "Sprite2D" | "sprite2d" | "sprite" => size_from_component(component)
                .or_else(|| sheet_frame_size(component))
                .or(Some((96.0, 96.0))),
            "Text2D" | "text2d" | "text" => rendered_text_size_from_component(component)
                .or_else(|| bounds_size_from_component(component))
                .or_else(|| estimated_text_size(component))
                .or(Some((180.0, 42.0))),
            "Vector2D" | "VectorShape2D" | "vector2d" | "vector" => size_from_component(component)
                .or_else(|| polygon_bounds_size(component))
                .or(Some((96.0, 96.0))),
            "TileMap2D" | "Tilemap2D" | "tilemap2d" | "tilemap" => {
                tilemap_size_from_component(component)
            }
            "Bounds2D" | "bounds2d" => size_from_component(component),
            "AabbCollider2D" | "aabbcollider2d" => size_from_component(component),
            "Camera2D" | "camera2d" | "Light2D" | "light2d" => {
                Some((ICON_BOUNDS_SIZE, ICON_BOUNDS_SIZE))
            }
            "UiNode" | "UI" | "ui" | "Button" | "button" => {
                ui_size_from_component(component).or(Some((180.0, 56.0)))
            }
            _ => None,
        };

        if candidate.is_some() {
            best_size = candidate;
            break;
        }
    }

    if best_size.is_none() && matches!(category, "camera" | "light" | "script") {
        best_size = Some((ICON_BOUNDS_SIZE, ICON_BOUNDS_SIZE));
    }

    let (width, height) = best_size?;
    let width = width * transform.scale_x.abs().max(0.0001);
    let height = height * transform.scale_y.abs().max(0.0001);
    Some(EditorBounds2Dto {
        x: transform.x - width / 2.0,
        y: transform.y - height / 2.0,
        width,
        height,
    })
}

fn camera_from_objects(objects: &[EditorSceneObjectDto]) -> EditorCameraDto {
    let camera_transform = objects
        .iter()
        .find(|object| object.category == "camera" && object.transform_2.is_some())
        .and_then(|object| object.transform_2.as_ref());

    EditorCameraDto {
        x: camera_transform.map(|transform| transform.x).unwrap_or(0.0),
        y: camera_transform.map(|transform| transform.y).unwrap_or(0.0),
        // The current 2D renderer subtracts camera translation only; scale is not camera zoom yet.
        zoom: 1.0,
        viewport_width: DEFAULT_WIDTH as f32,
        viewport_height: DEFAULT_HEIGHT as f32,
    }
}

fn ui_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    size_from_component(component)
        .or_else(|| bounds_size_from_component(component))
        .or_else(|| rect_size_from_component(component))
}

fn size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("size".to_owned()))
        .and_then(mapping)
        .and_then(|size| Some((number_field(size, "x")?, number_field(size, "y")?)))
}

fn bounds_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("bounds".to_owned()))
        .and_then(mapping)
        .and_then(|bounds| Some((number_field(bounds, "x")?, number_field(bounds, "y")?)))
}

fn rendered_text_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    let content = string_field(component, "content")?;
    let (_, bounds_height) = bounds_size_from_component(component)?;
    let pixel_size = (bounds_height / 7.0).clamp(4.0, 18.0);
    Some((
        content.chars().count() as f32 * 6.0 * pixel_size,
        7.0 * pixel_size,
    ))
}

fn rect_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("rect".to_owned()))
        .and_then(mapping)
        .and_then(|rect| Some((number_field(rect, "width")?, number_field(rect, "height")?)))
}

fn polygon_bounds_size(component: &Mapping) -> Option<(f32, f32)> {
    let points = component
        .get(Value::String("points".to_owned()))
        .and_then(Value::as_sequence)?;
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in points {
        let point = mapping(point)?;
        let x = number_field(point, "x")?;
        let y = number_field(point, "y")?;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        return Some(((max_x - min_x).max(1.0), (max_y - min_y).max(1.0)));
    }

    None
}

fn sheet_frame_size(component: &Mapping) -> Option<(f32, f32)> {
    component
        .get(Value::String("sheet".to_owned()))
        .and_then(mapping)
        .and_then(|sheet| {
            sheet
                .get(Value::String("frame_size".to_owned()))
                .and_then(mapping)
        })
        .and_then(|frame| Some((number_field(frame, "x")?, number_field(frame, "y")?)))
}

fn tilemap_size_from_component(component: &Mapping) -> Option<(f32, f32)> {
    let columns =
        number_field(component, "columns").or_else(|| number_field(component, "width"))?;
    let rows = number_field(component, "rows").or_else(|| number_field(component, "height"))?;
    let cell_size = component
        .get(Value::String("cell_size".to_owned()))
        .or_else(|| component.get(Value::String("tile_size".to_owned())))
        .and_then(mapping);
    let cell_width = cell_size
        .and_then(|value| number_field(value, "x"))
        .unwrap_or(32.0);
    let cell_height = cell_size
        .and_then(|value| number_field(value, "y"))
        .unwrap_or(32.0);

    Some((columns * cell_width, rows * cell_height))
}

fn estimated_text_size(component: &Mapping) -> Option<(f32, f32)> {
    let content = string_field(component, "content")?;
    Some(((content.chars().count() as f32).max(1.0) * 18.0, 36.0))
}

fn object_category(
    component_kinds: &[ComponentKind],
    has_transform3: bool,
    _component_types: &[String],
) -> String {
    if has_transform3
        || component_kinds.iter().any(|kind| {
            matches!(
                kind,
                ComponentKind::Mesh3D
                    | ComponentKind::Material3D
                    | ComponentKind::Text3D
                    | ComponentKind::Camera3D
                    | ComponentKind::Light3D
            )
        })
    {
        return "3d".to_owned();
    }
    if component_kinds.iter().any(|kind| {
        matches!(
            kind,
            ComponentKind::Camera2D | ComponentKind::CameraFollow2D
        )
    }) {
        return "camera".to_owned();
    }
    if component_kinds
        .iter()
        .any(|kind| matches!(kind, ComponentKind::UiDocument | ComponentKind::UiThemeSet))
    {
        return "ui".to_owned();
    }
    if component_kinds.iter().any(|kind| {
        matches!(
            kind,
            ComponentKind::Sprite2D | ComponentKind::Text2D | ComponentKind::VectorShape2D
        )
    }) {
        return "render".to_owned();
    }
    if component_kinds.iter().any(|kind| {
        matches!(
            kind,
            ComponentKind::TileMap2D | ComponentKind::TileMapMarker2D
        )
    }) {
        return "tilemap".to_owned();
    }
    if component_kinds.iter().any(|kind| {
        matches!(
            kind,
            ComponentKind::AabbCollider2D
                | ComponentKind::CircleCollider2D
                | ComponentKind::StaticCollider2D
                | ComponentKind::KinematicBody2D
                | ComponentKind::Trigger2D
        )
    }) {
        return "physics".to_owned();
    }
    if component_kinds.iter().any(|kind| {
        matches!(
            kind,
            ComponentKind::ScriptComponent | ComponentKind::Behavior | ComponentKind::EventPipeline
        )
    }) {
        return "script".to_owned();
    }

    "other".to_owned()
}

fn infer_canvas_kind_from_objects(objects: &[EditorSceneObjectDto]) -> EditorSceneCanvasKindDto {
    if objects
        .iter()
        .any(|object| object.transform_3.is_some() || object.category == "3d")
    {
        return EditorSceneCanvasKindDto::ThreeD;
    }

    EditorSceneCanvasKindDto::TwoD
}

fn ui_node_objects_from_scene_value(value: &Value) -> Vec<EditorUiNodeObjectDto> {
    let mut output = Vec::new();
    let Some(entities) = scene_entities(value) else {
        return output;
    };

    for entity_value in entities {
        let Some(entity) = mapping(entity_value) else {
            continue;
        };
        let Some(entity_id) = string_field(entity, "id") else {
            continue;
        };
        let components = entity
            .get(Value::String("components".to_owned()))
            .and_then(Value::as_sequence)
            .cloned()
            .unwrap_or_default();

        for (component_index, component_value) in components.iter().enumerate() {
            let Some(kind) = component_type(component_value) else {
                continue;
            };
            if kind != "UiDocument" {
                continue;
            }
            let Some(component) = mapping(component_value) else {
                continue;
            };
            let viewport = ui_document_viewport(component).unwrap_or(EditorBounds2Dto {
                x: 0.0,
                y: 0.0,
                width: DEFAULT_WIDTH as f32,
                height: DEFAULT_HEIGHT as f32,
            });
            let Some(root) = component
                .get(Value::String("root".to_owned()))
                .and_then(mapping)
            else {
                continue;
            };

            layout_ui_node(
                &mut output,
                &entity_id,
                component_index,
                root,
                "root".to_owned(),
                viewport,
            );
        }
    }

    output
}

fn ui_document_viewport(component: &Mapping) -> Option<EditorBounds2Dto> {
    let target = component
        .get(Value::String("target".to_owned()))
        .and_then(mapping)?;
    let viewport = target
        .get(Value::String("viewport".to_owned()))
        .and_then(mapping);
    let width = viewport
        .and_then(|viewport| number_field(viewport, "width"))
        .unwrap_or(DEFAULT_WIDTH as f32);
    let height = viewport
        .and_then(|viewport| number_field(viewport, "height"))
        .unwrap_or(DEFAULT_HEIGHT as f32);

    Some(EditorBounds2Dto {
        x: 0.0,
        y: 0.0,
        width,
        height,
    })
}

fn layout_ui_node(
    output: &mut Vec<EditorUiNodeObjectDto>,
    entity_id: &str,
    component_index: usize,
    node: &Mapping,
    fallback_path: String,
    parent_rect: EditorBounds2Dto,
) {
    let id = node
        .get(Value::String("id".to_owned()))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            fallback_path
                .rsplit('.')
                .next()
                .unwrap_or("node")
                .to_owned()
        });
    let path = if fallback_path == "root" {
        id.clone()
    } else {
        format!("{fallback_path}.{id}")
    };
    let kind_name = node
        .get(Value::String("type".to_owned()))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = editor_ui_node_kind(kind_name);
    let style = node
        .get(Value::String("style".to_owned()))
        .and_then(mapping);
    let rect = ui_rect_from_style(style, parent_rect);
    let text = node
        .get(Value::String("text".to_owned()))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let action_event = node
        .get(Value::String("on_click".to_owned()))
        .and_then(mapping)
        .and_then(|binding| binding.get(Value::String("event".to_owned())))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let label = text
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| id.clone());

    output.push(EditorUiNodeObjectDto {
        entity_id: entity_id.to_owned(),
        component_index,
        node_path: path.clone(),
        node_id: id,
        node_kind: kind,
        label,
        visible: true,
        selectable: true,
        locked: false,
        bounds_2: rect.clone(),
        render_bounds_2: rect.clone(),
        action_event,
    });

    let Some(children) = node
        .get(Value::String("children".to_owned()))
        .and_then(Value::as_sequence)
    else {
        return;
    };
    let padding = style
        .and_then(|style| number_field(style, "padding"))
        .unwrap_or(0.0);
    let gap = style
        .and_then(|style| number_field(style, "gap"))
        .unwrap_or(0.0);
    let content_x = rect.x + padding;
    let content_y = rect.y + padding;
    let content_width = (rect.width - padding * 2.0).max(0.0);
    let mut cursor_y = content_y;

    for child in children.iter().filter_map(mapping) {
        let child_style = child
            .get(Value::String("style".to_owned()))
            .and_then(mapping);
        let child_height = child_style
            .and_then(|style| number_field(style, "height"))
            .unwrap_or(36.0);
        let child_width = child_style
            .and_then(|style| number_field(style, "width"))
            .unwrap_or(content_width);
        let child_left = child_style
            .and_then(|style| number_field(style, "left"))
            .unwrap_or(content_x);
        let child_top = child_style
            .and_then(|style| number_field(style, "top"))
            .unwrap_or(cursor_y);
        let child_rect = EditorBounds2Dto {
            x: child_left,
            y: child_top,
            width: child_width,
            height: child_height,
        };

        layout_ui_node(
            output,
            entity_id,
            component_index,
            child,
            path.clone(),
            child_rect,
        );
        cursor_y += child_height + gap;
    }
}

fn ui_rect_from_style(style: Option<&Mapping>, parent_rect: EditorBounds2Dto) -> EditorBounds2Dto {
    let Some(style) = style else {
        return parent_rect;
    };
    EditorBounds2Dto {
        x: number_field(style, "left").unwrap_or(parent_rect.x),
        y: number_field(style, "top").unwrap_or(parent_rect.y),
        width: number_field(style, "width").unwrap_or(parent_rect.width),
        height: number_field(style, "height").unwrap_or(parent_rect.height),
    }
}

fn editor_ui_node_kind(kind: &str) -> EditorUiNodeKindDto {
    match kind {
        "panel" => EditorUiNodeKindDto::Panel,
        "group-box" | "group_box" => EditorUiNodeKindDto::GroupBox,
        "row" => EditorUiNodeKindDto::Row,
        "column" => EditorUiNodeKindDto::Column,
        "stack" => EditorUiNodeKindDto::Stack,
        "text" => EditorUiNodeKindDto::Text,
        "button" => EditorUiNodeKindDto::Button,
        "progress-bar" | "progress_bar" => EditorUiNodeKindDto::ProgressBar,
        "slider" => EditorUiNodeKindDto::Slider,
        "toggle" => EditorUiNodeKindDto::Toggle,
        "option-set" | "option_set" => EditorUiNodeKindDto::OptionSet,
        "dropdown" => EditorUiNodeKindDto::Dropdown,
        "tab-view" | "tab_view" => EditorUiNodeKindDto::TabView,
        "color-picker-rgb" | "color_picker_rgb" => EditorUiNodeKindDto::ColorPickerRgb,
        "curve-editor" | "curve_editor" => EditorUiNodeKindDto::CurveEditor,
        "spacer" => EditorUiNodeKindDto::Spacer,
        _ => EditorUiNodeKindDto::Unknown,
    }
}
