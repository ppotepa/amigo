use std::fs;
use std::path::{Path, PathBuf};

use crate::dto::{
    DiagnosticLevel, EditorDiagnosticDto, EditorProjectFileDto, EditorProjectStructureTreeDto,
    EditorProjectTreeDto, EditorResolvedAssetRefDto, EditorResolvedPropertyValueDto,
    EditorSceneComponentInstanceDto, EditorSceneEntityDto, EditorSceneHierarchyDto,
    EditorUiDocumentDto, EditorUiModelBindingDto, EditorUiNodeDto, EditorUiNodeKindDto,
    EditorUiNodeStyleDto,
};
use crate::mods::discovery::{discover_editor_mods, discovered_mod_ids};
use crate::mods::metadata::mod_details;
use amigo_scene::{
    ComponentRegistry, ComponentTypeDescriptor, EditorPropertyAccess, default_component_registry,
};
use serde_yaml::{Mapping, Value};

pub use super::project_file_classification::classify_project_file;
use super::shared::reveal_path;

pub fn get_scene_hierarchy(
    mod_id: String,
    scene_id: String,
) -> Result<EditorSceneHierarchyDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let document_path = discovered_mod
        .scene_document_path(&scene_id)
        .ok_or_else(|| format!("scene `{scene_id}` was not found in mod `{mod_id}`"))?;
    let text = fs::read_to_string(&document_path).map_err(|error| {
        format!(
            "failed to read scene document `{}`: {error}",
            document_path.display()
        )
    })?;
    let value = serde_yaml::from_str::<Value>(&text).map_err(|error| {
        format!(
            "failed to parse scene document `{}`: {error}",
            document_path.display()
        )
    })?;
    let document = amigo_scene::load_scene_document_from_str(&text).map_err(|error| {
        format!(
            "failed to load scene document `{}`: {error}",
            document_path.display()
        )
    })?;

    scene_hierarchy_from_document_value(mod_id, scene_id, &document, &value)
}

pub fn scene_hierarchy_from_value(
    mod_id: String,
    scene_id: String,
    value: &Value,
) -> Result<EditorSceneHierarchyDto, String> {
    let text = serde_yaml::to_string(value)
        .map_err(|error| format!("failed to serialize in-memory scene document: {error}"))?;
    let document = amigo_scene::load_scene_document_from_str(&text)
        .map_err(|error| format!("failed to load in-memory scene document: {error}"))?;
    scene_hierarchy_from_document_value(mod_id, scene_id, &document, value)
}

fn scene_hierarchy_from_document_value(
    mod_id: String,
    scene_id: String,
    document: &amigo_scene::SceneDocument,
    value: &Value,
) -> Result<EditorSceneHierarchyDto, String> {
    let component_registry = default_component_registry();
    let raw_entities = scene_entities_from_value(value);

    let entities = document
        .entities
        .iter()
        .enumerate()
        .map(|(entity_index, entity)| {
            let raw_entity = raw_entities.and_then(|items| {
                find_raw_entity_value(items, &entity.id).or_else(|| items.get(entity_index))
            });
            let components = raw_entity
                .map(|value| {
                    build_component_instances_for_entity(&entity.id, value, &component_registry)
                })
                .unwrap_or_default();
            let typed_component_types = entity
                .components
                .iter()
                .map(|component| component.kind().to_owned())
                .collect::<Vec<_>>();
            let component_types = if components.is_empty() {
                typed_component_types
            } else {
                components
                    .iter()
                    .map(|component| component.type_name.clone())
                    .collect::<Vec<_>>()
            };
            let component_count = if components.is_empty() {
                entity.components.len()
            } else {
                components.len()
            };

            let has_transform2 = entity.transform2.is_some();
            let has_transform3 = entity.transform3.is_some();
            let own_traits = entity_own_traits();
            let derived_traits = entity_derived_traits(has_transform2, has_transform3, &components);
            let metadata_traits = merge_traits(&own_traits, &derived_traits);

            EditorSceneEntityDto {
                id: entity.id.clone(),
                name: entity.display_name(),
                tags: entity.tags.clone(),
                groups: entity.groups.clone(),
                visible: entity.visible,
                simulation_enabled: entity.simulation_enabled,
                collision_enabled: entity.collision_enabled,
                has_transform2,
                has_transform3,
                property_count: entity.properties.len(),
                component_count,
                component_types,
                components,
                own_traits,
                derived_traits,
                metadata_traits,
            }
        })
        .collect::<Vec<_>>();
    let ui_documents = document
        .entities
        .iter()
        .flat_map(ui_documents_for_entity)
        .collect::<Vec<_>>();
    let component_count = entities.iter().map(|entity| entity.component_count).sum();

    Ok(EditorSceneHierarchyDto {
        mod_id,
        scene_id,
        scene_label: document.scene.label.clone(),
        entity_count: entities.len(),
        component_count,
        entities,
        ui_documents,
        diagnostics: Vec::new(),
    })
}

pub fn reveal_scene_document(mod_id: String, scene_id: String) -> Result<String, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;
    let document_path = discovered_mod
        .scene_document_path(&scene_id)
        .ok_or_else(|| format!("scene `{scene_id}` was not found in mod `{mod_id}`"))?;
    reveal_path(&document_path)?;
    Ok(document_path.display().to_string())
}

pub fn get_project_tree(mod_id: String) -> Result<EditorProjectTreeDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let mut total_files = 0;
    let root = project_file_node(
        &discovered_mod.root_path,
        &discovered_mod.root_path,
        &mut total_files,
    )?;

    Ok(EditorProjectTreeDto {
        mod_id,
        root_path: discovered_mod.root_path.display().to_string(),
        total_files,
        root,
    })
}

pub fn get_project_structure_tree(mod_id: String) -> Result<EditorProjectStructureTreeDto, String> {
    let discovered = discover_editor_mods().map_err(|diagnostic| diagnostic.message)?;
    let discovered_ids = discovered_mod_ids(&discovered);
    let discovered_mod = discovered
        .iter()
        .find(|candidate| candidate.manifest.id == mod_id)
        .ok_or_else(|| format!("mod `{mod_id}` was not found"))?;

    let mut total_files = 0;
    let file_root = project_file_node(
        &discovered_mod.root_path,
        &discovered_mod.root_path,
        &mut total_files,
    )?;
    let details = mod_details(discovered_mod, &discovered_ids);

    Ok(EditorProjectStructureTreeDto {
        mod_id,
        root_path: discovered_mod.root_path.display().to_string(),
        root: super::project_structure::project_structure_root(&details, &file_root),
    })
}

fn scene_entities_from_value(value: &Value) -> Option<&Vec<Value>> {
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

fn find_raw_entity_value<'a>(entities: &'a [Value], entity_id: &str) -> Option<&'a Value> {
    entities
        .iter()
        .find(|entity| entity_id_from_value(entity).as_deref() == Some(entity_id))
}

fn entity_id_from_value(entity: &Value) -> Option<String> {
    let entity = entity.as_mapping()?;
    string_field(entity, "id")
}

fn entity_own_traits() -> Vec<String> {
    vec![
        "HasIdentity".to_owned(),
        "HasVisibility".to_owned(),
        "HasComponents".to_owned(),
        "DiagnosticSource".to_owned(),
    ]
}

fn entity_derived_traits(
    has_transform2: bool,
    has_transform3: bool,
    components: &[EditorSceneComponentInstanceDto],
) -> Vec<String> {
    let mut traits = Vec::<String>::new();

    if has_transform2 {
        traits.push("Transformable2D".to_owned());
    }
    if has_transform3 {
        traits.push("Transformable3D".to_owned());
    }

    for component in components {
        for trait_kind in &component.metadata_traits {
            match trait_kind.as_str() {
                "Renderable2D"
                | "HasBounds2D"
                | "HasAssetRefs"
                | "Collidable2D"
                | "Trigger2D"
                | "Scriptable"
                | "EventSource"
                | "EventListener"
                | "InputBindable"
                | "UiEditable"
                | "HasUiTree"
                | "Motion2D"
                | "Simulatable"
                | "Poolable"
                | "LifetimeLimited"
                | "Renderable3D"
                | "HasBounds3D"
                | "Camera"
                | "RenderableViewportSource" => {
                    traits.push(trait_kind.clone());
                }
                "UsesTransform2D" if has_transform2 => {
                    traits.push("Transformable2D".to_owned());
                }
                "UsesTransform3D" if has_transform3 => {
                    traits.push("Transformable3D".to_owned());
                }
                _ => {}
            }
        }
    }

    traits.sort();
    traits.dedup();
    traits
}

fn merge_traits(left: &[String], right: &[String]) -> Vec<String> {
    let mut values = left.iter().chain(right.iter()).cloned().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

// @codemap:P1 editor-component-instance-builder
// Builds per-entity component instance DTOs from raw YAML and engine descriptors; keep this as the bridge between scene YAML and metadata-driven inspector UI.
fn build_component_instances_for_entity(
    entity_id: &str,
    entity_yaml: &Value,
    component_registry: &ComponentRegistry,
) -> Vec<EditorSceneComponentInstanceDto> {
    let Some(Value::Sequence(items)) = yaml_get_path(entity_yaml, "components") else {
        return Vec::new();
    };

    items
        .iter()
        .enumerate()
        .map(|(component_index, component_yaml)| {
            build_component_instance_dto(
                entity_id,
                component_index,
                component_yaml,
                component_registry,
            )
        })
        .collect()
}

fn build_component_instance_dto(
    entity_id: &str,
    component_index: usize,
    component_yaml: &Value,
    component_registry: &ComponentRegistry,
) -> EditorSceneComponentInstanceDto {
    let type_name =
        component_type_from_yaml(component_yaml).unwrap_or_else(|| "Unknown".to_owned());
    let descriptor = component_registry.descriptor_by_type_name(&type_name);
    let yaml_path = format!("entities[{entity_id}].components[{component_index}]");
    let label = descriptor
        .map(|descriptor| descriptor.label.to_owned())
        .unwrap_or_else(|| type_name.clone());
    let descriptor_kind = descriptor.map(|descriptor| format_debug(&descriptor.kind));
    let metadata_traits = descriptor
        .map(|descriptor| {
            descriptor
                .metadata_traits
                .iter()
                .map(|trait_kind| trait_kind.id().to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let properties = descriptor
        .map(|descriptor| resolve_component_properties(component_yaml, descriptor))
        .unwrap_or_default();
    let asset_refs = descriptor
        .map(|descriptor| resolve_component_asset_refs(component_yaml, descriptor))
        .unwrap_or_default();

    let mut diagnostics = Vec::new();
    if type_name == "Unknown" {
        diagnostics.push(component_diagnostic(
            DiagnosticLevel::Warning,
            "component.missingType",
            "Component is missing a string `type` or `kind` field.",
            Some(yaml_path.clone()),
        ));
    }
    if descriptor.is_none() {
        diagnostics.push(component_diagnostic(
            DiagnosticLevel::Warning,
            "component.missingDescriptor",
            format!("No component descriptor registered for `{type_name}`."),
            Some(yaml_path.clone()),
        ));
    }
    diagnostics.extend(required_asset_ref_diagnostics(&asset_refs, &yaml_path));

    EditorSceneComponentInstanceDto {
        component_index,
        type_name,
        descriptor_kind,
        label,
        yaml_path,
        values: yaml_to_json_value(component_yaml),
        metadata_traits,
        properties,
        asset_refs,
        diagnostics,
    }
}

fn resolve_component_properties(
    component_yaml: &Value,
    descriptor: &ComponentTypeDescriptor,
) -> Vec<EditorResolvedPropertyValueDto> {
    descriptor
        .properties
        .iter()
        .map(|property| {
            let value = yaml_get_path(component_yaml, property.path);
            let exists = value.is_some();

            EditorResolvedPropertyValueDto {
                path: property.path.to_owned(),
                label: property.label.to_owned(),
                value_kind: format_debug(&property.value_kind),
                editor: format_debug(&property.editor),
                access: format_debug(&property.access),
                value: value
                    .map(yaml_to_json_value)
                    .unwrap_or(serde_json::Value::Null),
                exists,
                editable: matches!(property.access, EditorPropertyAccess::Editable),
                trait_kind: property
                    .trait_kind
                    .map(|trait_kind| trait_kind.id().to_owned()),
                group: property.group.to_owned(),
            }
        })
        .collect()
}

fn resolve_component_asset_refs(
    component_yaml: &Value,
    descriptor: &ComponentTypeDescriptor,
) -> Vec<EditorResolvedAssetRefDto> {
    descriptor
        .asset_refs
        .iter()
        .map(|asset_ref| {
            let value = yaml_get_path(component_yaml, asset_ref.field_path)
                .and_then(Value::as_str)
                .map(str::to_owned);

            EditorResolvedAssetRefDto {
                field_path: asset_ref.field_path.to_owned(),
                domain: format_debug(&asset_ref.domain),
                required: asset_ref.required,
                value,
                trait_kind: asset_ref.trait_kind.id().to_owned(),
                group: asset_ref.group.to_owned(),
            }
        })
        .collect()
}

fn required_asset_ref_diagnostics(
    asset_refs: &[EditorResolvedAssetRefDto],
    yaml_path: &str,
) -> Vec<EditorDiagnosticDto> {
    asset_refs
        .iter()
        .filter(|asset_ref| {
            asset_ref.required
                && asset_ref
                    .value
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty()
        })
        .map(|asset_ref| {
            component_diagnostic(
                DiagnosticLevel::Warning,
                "component.missingRequiredAssetRef",
                format!(
                    "Required {} asset reference `{}` is missing.",
                    asset_ref.domain, asset_ref.field_path
                ),
                Some(format!("{yaml_path}.{}", asset_ref.field_path)),
            )
        })
        .collect()
}

fn component_type_from_yaml(component_yaml: &Value) -> Option<String> {
    let component = component_yaml.as_mapping()?;
    string_field(component, "type").or_else(|| string_field(component, "kind"))
}

fn yaml_get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;

    for segment in path.split('.') {
        match current {
            Value::Mapping(map) => {
                current = map.get(Value::String(segment.to_owned()))?;
            }
            _ => return None,
        }
    }

    Some(current)
}

fn yaml_to_json_value(value: &Value) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

fn component_diagnostic(
    level: DiagnosticLevel,
    code: impl Into<String>,
    message: impl Into<String>,
    path: Option<String>,
) -> EditorDiagnosticDto {
    EditorDiagnosticDto {
        level,
        code: code.into(),
        message: message.into(),
        path,
    }
}

fn format_debug<T: std::fmt::Debug>(value: &T) -> String {
    format!("{value:?}")
}

pub fn project_file_node(
    path: &Path,
    root: &Path,
    total_files: &mut usize,
) -> Result<EditorProjectFileDto, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("failed to read metadata `{}`: {error}", path.display()))?;
    let is_dir = metadata.is_dir();
    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let name = if relative_path.is_empty() {
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("mod root")
            .to_owned()
    } else {
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned()
    };

    let mut children = Vec::new();
    if is_dir {
        let mut entries = std::fs::read_dir(path)
            .map_err(|error| format!("failed to read directory `{}`: {error}", path.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(should_include_project_path)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            let left_is_dir = left.is_dir();
            let right_is_dir = right.is_dir();
            right_is_dir
                .cmp(&left_is_dir)
                .then_with(|| left.file_name().cmp(&right.file_name()))
        });

        for entry_path in entries {
            children.push(project_file_node(&entry_path, root, total_files)?);
        }
    } else {
        *total_files += 1;
    }

    Ok(EditorProjectFileDto {
        name,
        path: path.display().to_string(),
        relative_path,
        kind: super::project_file_classification::classify_project_file(path, is_dir),
        is_dir,
        size_bytes: if is_dir { 0 } else { metadata.len() },
        children,
    })
}

fn should_include_project_path(path: &PathBuf) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    !matches!(name, ".git" | ".amigo-editor" | "target")
}

fn ui_documents_for_entity(entity: &amigo_scene::SceneEntityDocument) -> Vec<EditorUiDocumentDto> {
    let bindings = ui_model_bindings_for_entity(entity);

    entity
        .components
        .iter()
        .enumerate()
        .filter_map(|(component_index, component)| {
            if component.kind() != "UiDocument" {
                return None;
            }

            let value = serde_yaml::to_value(component).ok()?;
            let component = value.as_mapping()?;
            let root = component
                .get(Value::String("root".to_owned()))?
                .as_mapping()?;
            let target_layer = component
                .get(Value::String("target".to_owned()))
                .and_then(Value::as_mapping)
                .and_then(|target| target.get(Value::String("layer".to_owned())))
                .and_then(Value::as_str)
                .map(str::to_owned);

            Some(EditorUiDocumentDto {
                entity_id: entity.id.clone(),
                entity_name: entity.display_name(),
                component_index,
                target_layer,
                root: ui_node_from_mapping(root, "root".to_owned()),
                bindings: bindings.clone(),
            })
        })
        .collect()
}

fn ui_model_bindings_for_entity(
    entity: &amigo_scene::SceneEntityDocument,
) -> Vec<EditorUiModelBindingDto> {
    entity
        .components
        .iter()
        .enumerate()
        .flat_map(|(component_index, component)| match component {
            amigo_scene::SceneComponentDocument::UiModelBindings { bindings } => bindings
                .iter()
                .enumerate()
                .map(move |(binding_index, binding)| EditorUiModelBindingDto {
                    id: format!(
                        "{}:{}:{}:{}",
                        entity.id, component_index, binding.path, binding.state
                    ),
                    path: binding.path.clone(),
                    state: binding.state.clone(),
                    kind: ui_model_binding_kind(binding.kind.clone()),
                    format: binding.format.clone(),
                    component_index,
                    binding_index,
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

fn ui_model_binding_kind(kind: amigo_scene::SceneUiModelBindingKindDocument) -> String {
    match kind {
        amigo_scene::SceneUiModelBindingKindDocument::Text => "text",
        amigo_scene::SceneUiModelBindingKindDocument::Value => "value",
        amigo_scene::SceneUiModelBindingKindDocument::Visible => "visible",
        amigo_scene::SceneUiModelBindingKindDocument::Enabled => "enabled",
        amigo_scene::SceneUiModelBindingKindDocument::Selected => "selected",
        amigo_scene::SceneUiModelBindingKindDocument::Options => "options",
        amigo_scene::SceneUiModelBindingKindDocument::Color => "color",
        amigo_scene::SceneUiModelBindingKindDocument::Background => "background",
        amigo_scene::SceneUiModelBindingKindDocument::Theme => "theme",
    }
    .to_owned()
}

fn ui_node_from_mapping(node: &Mapping, fallback_path: String) -> EditorUiNodeDto {
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

    let kind = node
        .get(Value::String("type".to_owned()))
        .and_then(Value::as_str)
        .map(ui_node_kind)
        .unwrap_or(EditorUiNodeKindDto::Unknown);

    let text = node
        .get(Value::String("text".to_owned()))
        .and_then(Value::as_str)
        .map(str::to_owned);

    let style_class = node
        .get(Value::String("style_class".to_owned()))
        .or_else(|| node.get(Value::String("styleClass".to_owned())))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let style = node
        .get(Value::String("style".to_owned()))
        .and_then(Value::as_mapping)
        .map(ui_node_style)
        .unwrap_or_default();

    let action_event = node
        .get(Value::String("on_click".to_owned()))
        .and_then(Value::as_mapping)
        .and_then(|binding| binding.get(Value::String("event".to_owned())))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let action_target = node
        .get(Value::String("on_click".to_owned()))
        .and_then(Value::as_mapping)
        .and_then(|binding| binding.get(Value::String("target".to_owned())))
        .and_then(Value::as_str)
        .map(str::to_owned);

    let children = node
        .get(Value::String("children".to_owned()))
        .and_then(Value::as_sequence)
        .map(|children| {
            children
                .iter()
                .filter_map(Value::as_mapping)
                .map(|child| ui_node_from_mapping(child, path.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let label = text
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| id.clone());

    EditorUiNodeDto {
        path,
        id,
        kind,
        label,
        text,
        style_class,
        style,
        enabled: node
            .get(Value::String("enabled".to_owned()))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        visible: node
            .get(Value::String("visible".to_owned()))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        action_event,
        action_target,
        child_count: children.len(),
        children,
    }
}

fn ui_node_style(style: &Mapping) -> EditorUiNodeStyleDto {
    EditorUiNodeStyleDto {
        left: number_field(style, "left"),
        top: number_field(style, "top"),
        width: number_field(style, "width"),
        height: number_field(style, "height"),
        font_size: number_field(style, "font_size"),
        color: string_field(style, "color"),
        background: string_field(style, "background"),
        border_color: string_field(style, "border_color"),
        border_width: number_field(style, "border_width"),
        border_radius: number_field(style, "border_radius"),
        padding: number_field(style, "padding"),
        gap: number_field(style, "gap"),
    }
}

fn number_field(mapping: &Mapping, key: &str) -> Option<f32> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_f64)
        .map(|value| value as f32)
}

fn string_field(mapping: &Mapping, key: &str) -> Option<String> {
    mapping
        .get(Value::String(key.to_owned()))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn ui_node_kind(kind: &str) -> EditorUiNodeKindDto {
    match kind {
        "panel" => EditorUiNodeKindDto::Panel,
        "group-box" | "group_box" => EditorUiNodeKindDto::GroupBox,
        "row" => EditorUiNodeKindDto::Row,
        "column" => EditorUiNodeKindDto::Column,
        "stack" => EditorUiNodeKindDto::Stack,
        "text" => EditorUiNodeKindDto::Text,
        "button" => EditorUiNodeKindDto::Button,
        "image" => EditorUiNodeKindDto::Image,
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

#[cfg(test)]
#[path = "project_tree_tests.rs"]
mod tests;
