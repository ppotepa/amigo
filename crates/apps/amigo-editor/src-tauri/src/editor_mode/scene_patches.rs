use std::fs;
use std::path::Path;

use amigo_scene::default_component_registry;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use crate::editor_mode::scene_document_path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameSceneRequestDto {
    pub scene_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSceneComponentRequestDto {
    pub scene_id: String,
    pub component_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vec2Dto {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSceneEntityRequestDto {
    pub scene_id: String,
    pub template_id: String,
    pub suggested_name: Option<String>,
    pub asset_key: Option<String>,
    pub position: Option<Vec2Dto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePatchResultDto {
    pub ok: bool,
    pub scene_dirty: bool,
    pub message: Option<String>,
}

/// @codemap P1 editor_mode.scene_patches
pub fn rename_scene(
    root_path: &str,
    request: RenameSceneRequestDto,
) -> Result<ScenePatchResultDto, String> {
    let scene_path = scene_document_path(Path::new(root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    patch_scene_display_name(&mut document, &request.display_name)?;
    write_yaml_document(&scene_path, &document)?;

    let manifest_path = Path::new(root_path).join("mod.toml");
    patch_manifest_scene_label(&manifest_path, &request.scene_id, &request.display_name)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!("Renamed scene to `{}`.", request.display_name)),
    })
}

pub fn add_scene_component(
    root_path: &str,
    request: AddSceneComponentRequestDto,
) -> Result<ScenePatchResultDto, String> {
    let scene_path = scene_document_path(Path::new(root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    let descriptor_default_yaml = default_component_registry()
        .descriptor_by_type_name(&request.component_type)
        .and_then(|descriptor| descriptor.default_yaml);
    let component_node =
        build_default_component_yaml(&request.component_type, descriptor_default_yaml)?;
    patch_add_scene_component(&mut document, component_node)?;
    write_yaml_document(&scene_path, &document)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!(
            "Added scene component `{}`.",
            request.component_type
        )),
    })
}

pub fn add_scene_entity(
    root_path: &str,
    request: AddSceneEntityRequestDto,
) -> Result<ScenePatchResultDto, String> {
    let scene_path = scene_document_path(Path::new(root_path), &request.scene_id);
    let mut document = read_yaml_document(&scene_path)?;
    let entity_node = build_entity_from_template(&document, &request);
    patch_add_scene_entity(&mut document, entity_node)?;
    write_yaml_document(&scene_path, &document)?;

    Ok(ScenePatchResultDto {
        ok: true,
        scene_dirty: true,
        message: Some(format!(
            "Added entity from template `{}`.",
            request.template_id
        )),
    })
}

fn read_yaml_document(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    serde_yaml::from_str::<Value>(&text)
        .map_err(|error| format!("failed to parse `{}`: {error}", path.display()))
}

fn write_yaml_document(path: &Path, document: &Value) -> Result<(), String> {
    let text = serde_yaml::to_string(document)
        .map_err(|error| format!("failed to serialize `{}`: {error}", path.display()))?;
    fs::write(path, text).map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn patch_scene_display_name(document: &mut Value, display_name: &str) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let scene_key = Value::String("scene".to_owned());
    if let Some(scene_value) = root.get_mut(&scene_key) {
        let scene = scene_value
            .as_mapping_mut()
            .ok_or_else(|| "`scene` must be a mapping".to_owned())?;
        scene.insert(
            Value::String("name".to_owned()),
            Value::String(display_name.to_owned()),
        );
        scene.insert(
            Value::String("label".to_owned()),
            Value::String(display_name.to_owned()),
        );
    } else {
        root.insert(
            Value::String("name".to_owned()),
            Value::String(display_name.to_owned()),
        );
    }
    Ok(())
}

fn build_default_component_yaml(
    component_type: &str,
    default_yaml: Option<&str>,
) -> Result<Value, String> {
    let mut node = if let Some(default_yaml) = default_yaml {
        serde_yaml::from_str::<Value>(default_yaml).map_err(|error| {
            format!("failed to parse default YAML for `{component_type}`: {error}")
        })?
    } else {
        Value::Mapping(Mapping::new())
    };

    let mapping = node
        .as_mapping_mut()
        .ok_or_else(|| format!("default YAML for `{component_type}` must be a mapping"))?;
    mapping.insert(
        Value::String("type".to_owned()),
        Value::String(component_type.to_owned()),
    );
    Ok(node)
}

fn patch_add_scene_component(document: &mut Value, component_node: Value) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let components_key = Value::String("components".to_owned());
    if !root.contains_key(&components_key) {
        root.insert(components_key.clone(), Value::Sequence(Vec::new()));
    }

    let components = root
        .get_mut(&components_key)
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "`components` must be a sequence".to_owned())?;

    components.push(component_node);
    Ok(())
}

fn patch_add_scene_entity(document: &mut Value, entity: Value) -> Result<(), String> {
    let root = document
        .as_mapping_mut()
        .ok_or_else(|| "scene document root must be a mapping".to_owned())?;

    let entities_key = Value::String("entities".to_owned());
    if !root.contains_key(&entities_key) {
        root.insert(entities_key.clone(), Value::Sequence(Vec::new()));
    }

    let entities = root
        .get_mut(&entities_key)
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| "`entities` must be a sequence".to_owned())?;
    entities.push(entity);
    Ok(())
}

fn build_entity_from_template(document: &Value, request: &AddSceneEntityRequestDto) -> Value {
    let entity_id = next_entity_id(document, &request.template_id);
    let entity_name = request
        .suggested_name
        .clone()
        .unwrap_or_else(|| entity_id.clone());

    let mut entity = Mapping::new();
    entity.insert(Value::String("id".to_owned()), Value::String(entity_id));
    entity.insert(Value::String("name".to_owned()), Value::String(entity_name));

    let mut components = Vec::new();
    match request.template_id.as_str() {
        "empty" => {}
        "sprite" => {
            components.push(component_with_transform(request.position.as_ref()));
            let mut sprite = Mapping::new();
            sprite.insert(
                Value::String("type".to_owned()),
                Value::String("Sprite2D".to_owned()),
            );
            if let Some(asset_key) = request
                .asset_key
                .as_ref()
                .filter(|value| !value.trim().is_empty())
            {
                sprite.insert(
                    Value::String("texture".to_owned()),
                    Value::String(asset_key.clone()),
                );
            }
            components.push(Value::Mapping(sprite));
        }
        "tilemap" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("TileMap2D"));
        }
        "trigger" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("Trigger2D"));
        }
        "camera" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("Camera2D"));
        }
        "spawnPoint" => {
            components.push(component_with_transform(request.position.as_ref()));
            components.push(component_only_type("SpawnPoint"));
        }
        _ => {
            components.push(component_with_transform(request.position.as_ref()));
        }
    }

    entity.insert(
        Value::String("components".to_owned()),
        Value::Sequence(components),
    );
    Value::Mapping(entity)
}

fn component_with_transform(position: Option<&Vec2Dto>) -> Value {
    let mut transform = Mapping::new();
    transform.insert(
        Value::String("type".to_owned()),
        Value::String("Transform2D".to_owned()),
    );
    if let Some(position) = position {
        let mut translation = Mapping::new();
        translation.insert(
            Value::String("x".to_owned()),
            serde_yaml::to_value(position.x).unwrap_or(Value::from(0.0_f32)),
        );
        translation.insert(
            Value::String("y".to_owned()),
            serde_yaml::to_value(position.y).unwrap_or(Value::from(0.0_f32)),
        );
        transform.insert(
            Value::String("translation".to_owned()),
            Value::Mapping(translation),
        );
    }
    Value::Mapping(transform)
}

fn component_only_type(type_name: &str) -> Value {
    let mut component = Mapping::new();
    component.insert(
        Value::String("type".to_owned()),
        Value::String(type_name.to_owned()),
    );
    Value::Mapping(component)
}

fn next_entity_id(document: &Value, prefix: &str) -> String {
    let root = match document.as_mapping() {
        Some(root) => root,
        None => return format!("{prefix}_1"),
    };
    let entities = root
        .get(Value::String("entities".to_owned()))
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();

    let mut next_index = 1_u32;
    loop {
        let candidate = format!("{prefix}_{next_index}");
        let exists = entities.iter().any(|entity| {
            entity
                .as_mapping()
                .and_then(|mapping| mapping.get(Value::String("id".to_owned())))
                .and_then(Value::as_str)
                .map(|id| id == candidate)
                .unwrap_or(false)
        });
        if !exists {
            return candidate;
        }
        next_index += 1;
    }
}

fn patch_manifest_scene_label(
    manifest_path: &Path,
    scene_id: &str,
    display_name: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;

    let mut in_scene = false;
    let mut current_scene_matches = false;
    let mut updated_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[scenes]]" {
            in_scene = true;
            current_scene_matches = false;
            updated_lines.push(line.to_owned());
            continue;
        }
        if in_scene && trimmed.starts_with("id = ") {
            current_scene_matches = trimmed.contains(&format!("\"{scene_id}\""));
            updated_lines.push(line.to_owned());
            continue;
        }
        if in_scene && current_scene_matches && trimmed.starts_with("label = ") {
            let leading = line
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>();
            updated_lines.push(format!(
                "{leading}label = \"{}\"",
                display_name.replace('"', "\\\"")
            ));
            continue;
        }
        if in_scene && trimmed.starts_with("[[") && trimmed != "[[scenes]]" {
            in_scene = false;
            current_scene_matches = false;
        }
        updated_lines.push(line.to_owned());
    }

    fs::write(manifest_path, format!("{}\n", updated_lines.join("\n")))
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))
}
