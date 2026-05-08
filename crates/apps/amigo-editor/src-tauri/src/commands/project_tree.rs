use std::fs;
use std::path::{Path, PathBuf};

use crate::dto::{
    DiagnosticLevel, EditorDiagnosticDto, EditorModDetailsDto, EditorProjectFileDto,
    EditorProjectStructureNodeDto, EditorProjectStructureTreeDto, EditorProjectTreeDto,
    EditorResolvedAssetRefDto, EditorResolvedPropertyValueDto, EditorSceneComponentInstanceDto,
    EditorSceneEntityDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, EditorUiDocumentDto,
    EditorUiModelBindingDto, EditorUiNodeDto, EditorUiNodeKindDto, EditorUiNodeStyleDto,
};
use crate::mods::discovery::{discover_editor_mods, discovered_mod_ids};
use crate::mods::metadata::mod_details;
use amigo_scene::{
    ComponentRegistry, ComponentTypeDescriptor, EditorPropertyAccess, default_component_registry,
};
use serde_yaml::{Mapping, Value};

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
        root: project_structure_root(&details, &file_root),
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
        kind: classify_project_file(path, is_dir),
        is_dir,
        size_bytes: if is_dir { 0 } else { metadata.len() },
        children,
    })
}

pub fn project_structure_root(
    details: &EditorModDetailsDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let summary = &details.summary.content_summary;
    let diagnostics_count = details.summary.diagnostics.len()
        + details
            .scenes
            .iter()
            .map(|scene| scene.diagnostics.len())
            .sum::<usize>();
    let script_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| {
            file.kind == "script" && !scene_owns_script(&details.scenes, &file.relative_path)
        })
        .cloned()
        .collect::<Vec<_>>();
    let package_files = flatten_project_files(file_root)
        .into_iter()
        .filter(|file| file.relative_path.starts_with("packages/"))
        .cloned()
        .collect::<Vec<_>>();

    node(ProjectStructureNodeInput {
        id: format!("mod:{}", details.summary.id),
        label: details.summary.id.clone(),
        kind: "modRoot",
        icon: "Mod",
        status: Some(project_status_for_editor_status(&format!(
            "{:?}",
            details.summary.status
        ))),
        count: Some(summary.total_files),
        path: Some(details.summary.root_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: None,
        scene: None,
        children: vec![
            node(ProjectStructureNodeInput {
                id: "overview".to_owned(),
                label: "Overview".to_owned(),
                kind: "overview",
                icon: "Info",
                status: Some(project_status_for_editor_status(&format!(
                    "{:?}",
                    details.summary.status
                ))),
                count: None,
                path: None,
                expected_path: None,
                exists: true,
                empty: false,
                ghost: false,
                file: None,
                scene: None,
                children: Vec::new(),
            }),
            manifest_node(file_root, details),
            group_node(
                "scenes",
                "Sc",
                details.scenes.len(),
                root_child_exists(file_root, "scenes"),
                details
                    .scenes
                    .iter()
                    .map(|scene| scene_structure_node(scene, file_root))
                    .collect(),
            ),
            group_node(
                "raw",
                "Raw",
                summary.textures + summary.audio + summary.fonts,
                root_child_exists(file_root, "raw"),
                files_under(file_root, "raw")
                    .into_iter()
                    .take(48)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "spritesheets",
                "Grid",
                summary.spritesheets + summary.tilesets + summary.tilemaps,
                root_child_exists(file_root, "spritesheets"),
                files_under(file_root, "spritesheets")
                    .into_iter()
                    .filter(|file| {
                        matches!(file.kind.as_str(), "spritesheet" | "tileset" | "tilemap")
                    })
                    .take(64)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "audio",
                "Aud",
                summary.audio,
                root_child_exists(file_root, "audio"),
                files_under(file_root, "audio")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "fonts",
                "Type",
                summary.fonts,
                root_child_exists(file_root, "fonts"),
                files_under(file_root, "fonts")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "scripts",
                "Rh",
                script_files.len(),
                root_child_exists(file_root, "scripts"),
                script_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptFile"))
                    .collect(),
            ),
            group_node(
                "packages",
                "Pkg",
                summary.packages,
                root_child_exists(file_root, "packages"),
                package_files
                    .into_iter()
                    .take(24)
                    .map(|file| file_structure_node(file, "scriptPackage"))
                    .collect(),
            ),
            group_node(
                "data",
                "Data",
                files_under(file_root, "data").len(),
                root_child_exists(file_root, "data"),
                files_under(file_root, "data")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "docs",
                "Doc",
                files_under(file_root, "docs").len(),
                root_child_exists(file_root, "docs"),
                files_under(file_root, "docs")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            group_node(
                "custom",
                "Ext",
                files_under(file_root, "custom").len(),
                root_child_exists(file_root, "custom"),
                files_under(file_root, "custom")
                    .into_iter()
                    .take(24)
                    .map(asset_resource_node)
                    .collect(),
            ),
            virtual_node(
                "capabilities",
                "Capabilities",
                "Plug",
                details.summary.capabilities.len(),
                "ok",
            ),
            virtual_node(
                "dependencies",
                "Dependencies",
                "Link",
                details.summary.dependencies.len(),
                if details.summary.missing_dependencies.is_empty() {
                    "ok"
                } else {
                    "warn"
                },
            ),
            virtual_node(
                "diagnostics",
                "Diagnostics",
                "Diag",
                diagnostics_count,
                if diagnostics_count == 0 { "ok" } else { "warn" },
            ),
        ],
    })
}

struct ProjectStructureNodeInput {
    id: String,
    label: String,
    kind: &'static str,
    icon: &'static str,
    status: Option<String>,
    count: Option<usize>,
    path: Option<String>,
    expected_path: Option<String>,
    exists: bool,
    empty: bool,
    ghost: bool,
    file: Option<EditorProjectFileDto>,
    scene: Option<EditorSceneSummaryDto>,
    children: Vec<EditorProjectStructureNodeDto>,
}

fn node(input: ProjectStructureNodeInput) -> EditorProjectStructureNodeDto {
    EditorProjectStructureNodeDto {
        id: input.id,
        label: input.label,
        kind: input.kind.to_owned(),
        icon: input.icon.to_owned(),
        status: input.status,
        count: input.count,
        path: input.path,
        expected_path: input.expected_path,
        exists: input.exists,
        empty: input.empty,
        ghost: input.ghost,
        file: input.file,
        scene: input.scene,
        children: input.children,
    }
}

fn manifest_node(
    file_root: &EditorProjectFileDto,
    details: &EditorModDetailsDto,
) -> EditorProjectStructureNodeDto {
    let manifest = find_project_file(file_root, "mod.toml").cloned();
    node(ProjectStructureNodeInput {
        id: "manifest:mod.toml".to_owned(),
        label: "mod.toml".to_owned(),
        kind: "manifest",
        icon: "Toml",
        status: Some(if manifest.is_some() {
            project_status_for_editor_status(&format!("{:?}", details.summary.status))
        } else {
            "error".to_owned()
        }),
        count: None,
        path: manifest.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some("mod.toml".to_owned()),
        exists: manifest.is_some(),
        empty: false,
        ghost: manifest.is_none(),
        file: manifest,
        scene: None,
        children: Vec::new(),
    })
}

fn group_node(
    label: &str,
    icon: &'static str,
    count: usize,
    exists: bool,
    children: Vec<EditorProjectStructureNodeDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("group:{label}"),
        label: label.to_owned(),
        kind: if exists { "folder" } else { "expectedFolder" },
        icon,
        status: Some(
            if exists {
                if count == 0 { "empty" } else { "ok" }
            } else {
                "missing"
            }
            .to_owned(),
        ),
        count: Some(count),
        path: if exists { Some(label.to_owned()) } else { None },
        expected_path: Some(format!("{label}/")),
        exists,
        empty: exists && count == 0,
        ghost: !exists,
        file: None,
        scene: None,
        children,
    })
}

fn virtual_node(
    id: &'static str,
    label: &str,
    icon: &'static str,
    count: usize,
    status: &str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("virtual:{id}"),
        label: label.to_owned(),
        kind: id,
        icon,
        status: Some(status.to_owned()),
        count: Some(count),
        path: None,
        expected_path: None,
        exists: true,
        empty: count == 0,
        ghost: false,
        file: None,
        scene: None,
        children: Vec::new(),
    })
}

fn scene_structure_node(
    scene: &EditorSceneSummaryDto,
    file_root: &EditorProjectFileDto,
) -> EditorProjectStructureNodeDto {
    let document_path = relative_project_path(&scene.document_path);
    let script_path = relative_project_path(&scene.script_path);
    let document = find_project_file(file_root, &document_path).cloned();
    let script = find_project_file(file_root, &script_path).cloned();
    let status = project_status_for_editor_status(&format!("{:?}", scene.status));

    node(ProjectStructureNodeInput {
        id: format!("scene:{}", scene.id),
        label: if scene.label.is_empty() {
            scene.id.clone()
        } else {
            scene.label.clone()
        },
        kind: "scene",
        icon: "Play",
        status: Some(if status == "valid" {
            "ready".to_owned()
        } else {
            status
        }),
        count: Some(2),
        path: Some(scene.path.clone()),
        expected_path: None,
        exists: document.is_some(),
        empty: false,
        ghost: false,
        file: None,
        scene: Some(scene.clone()),
        children: vec![
            scene_file_node(
                "sceneDocument",
                format!("scene-doc:{}", scene.id),
                "scene.yml",
                "Yml",
                document_path,
                document,
            ),
            scene_file_node(
                "sceneScript",
                format!("scene-script:{}", scene.id),
                "scene.rhai",
                "Rh",
                script_path,
                script,
            ),
        ],
    })
}

fn scene_file_node(
    kind: &'static str,
    id: String,
    label: &str,
    icon: &'static str,
    expected_path: String,
    file: Option<EditorProjectFileDto>,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id,
        label: label.to_owned(),
        kind,
        icon,
        status: Some(if file.is_some() { "ok" } else { "missing" }.to_owned()),
        count: None,
        path: file.as_ref().map(|file| file.relative_path.clone()),
        expected_path: Some(expected_path),
        exists: file.is_some(),
        empty: false,
        ghost: file.is_none(),
        file,
        scene: None,
        children: Vec::new(),
    })
}

fn file_structure_node(
    file: EditorProjectFileDto,
    kind: &'static str,
) -> EditorProjectStructureNodeDto {
    node(ProjectStructureNodeInput {
        id: format!("{kind}:{}", file.relative_path),
        label: file.name.clone(),
        kind,
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn asset_resource_node(file: EditorProjectFileDto) -> EditorProjectStructureNodeDto {
    let label = asset_display_label(&file);
    node(ProjectStructureNodeInput {
        id: format!("assetResource:{}", file.relative_path),
        label,
        kind: "assetResource",
        icon: project_file_icon(&file),
        status: Some("ok".to_owned()),
        count: None,
        path: Some(file.relative_path.clone()),
        expected_path: None,
        exists: true,
        empty: false,
        ghost: false,
        file: Some(file),
        scene: None,
        children: Vec::new(),
    })
}

fn flatten_project_files(root: &EditorProjectFileDto) -> Vec<&EditorProjectFileDto> {
    root.children
        .iter()
        .flat_map(|child| {
            let mut files = vec![child];
            files.extend(flatten_project_files(child));
            files
        })
        .filter(|file| !file.is_dir)
        .collect()
}

fn find_project_file<'a>(
    root: &'a EditorProjectFileDto,
    relative_path: &str,
) -> Option<&'a EditorProjectFileDto> {
    if root.relative_path == relative_path {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| find_project_file(child, relative_path))
}

fn root_child_exists(root: &EditorProjectFileDto, relative_path: &str) -> bool {
    find_project_file(root, relative_path).is_some()
}

fn files_under(root: &EditorProjectFileDto, relative_path: &str) -> Vec<EditorProjectFileDto> {
    let prefix = format!("{}/", relative_path.trim_end_matches('/'));
    flatten_project_files(root)
        .into_iter()
        .filter(|file| {
            file.relative_path == relative_path || file.relative_path.starts_with(&prefix)
        })
        .cloned()
        .collect()
}

fn relative_project_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    for prefix in [
        "scenes/",
        "raw/",
        "spritesheets/",
        "audio/",
        "fonts/",
        "scripts/",
        "data/",
        "docs/",
        "custom/",
        "packages/",
    ] {
        if let Some(index) = normalized.find(prefix) {
            return normalized[index..].to_owned();
        }
    }
    normalized
}

fn project_status_for_editor_status(status: &str) -> String {
    match status {
        "Valid" => "valid",
        "Warning" | "MissingDependency" => "warn",
        "Error" | "InvalidManifest" | "MissingSceneFile" | "PreviewFailed" => "error",
        _ => "ok",
    }
    .to_owned()
}

fn project_file_icon(file: &EditorProjectFileDto) -> &'static str {
    match file.kind.as_str() {
        "manifest" => "Toml",
        "sceneDocument" => "Yml",
        "script" => "Rh",
        "imageAsset" | "rawImage" | "texture" => "Img",
        "spritesheet" => "Grid",
        "audio" | "rawAudio" => "Aud",
        "font" | "rawFont" => "Type",
        "tilemap" => "Map",
        "tileset" => "Tile",
        "particle" => "Pt",
        "material" => "Mat",
        "ui" => "Ui",
        _ => "F",
    }
}

fn asset_display_label(file: &EditorProjectFileDto) -> String {
    let name = file.name.as_str();
    for suffix in [
        ".image.yml",
        ".image.yaml",
        ".sprite.yml",
        ".sprite.yaml",
        ".atlas.yml",
        ".atlas.yaml",
        ".tileset.yml",
        ".tileset.yaml",
        ".tile-ruleset.yml",
        ".tile-ruleset.yaml",
        ".tilemap.yml",
        ".tilemap.yaml",
        ".font.yml",
        ".font.yaml",
        ".audio.yml",
        ".audio.yaml",
        ".particle.yml",
        ".particle.yaml",
        ".material.yml",
        ".material.yaml",
        ".ui.yml",
        ".ui.yaml",
    ] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            return stripped.to_owned();
        }
    }
    name.to_owned()
}

fn scene_owns_script(scenes: &[EditorSceneSummaryDto], relative_path: &str) -> bool {
    scenes
        .iter()
        .any(|scene| relative_project_path(&scene.script_path) == relative_path)
}

fn should_include_project_path(path: &PathBuf) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    !matches!(name, ".git" | ".amigo-editor" | "target")
}

pub fn classify_project_file(path: &Path, is_dir: bool) -> String {
    if is_dir {
        return "directory".to_owned();
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name == "mod.toml" || extension == "toml" {
        "manifest"
    } else if file_name == "package.yml" || file_name == "package.yaml" {
        "scriptPackage"
    } else if file_name == "scene.yml"
        || file_name == "scene.yaml"
        || file_name.ends_with(".scene.yml")
        || file_name.ends_with(".scene.yaml")
    {
        "sceneDocument"
    } else if file_name == "scene.rhai" || file_name.ends_with(".scene.rhai") {
        "sceneScript"
    } else if extension == "rhai" {
        "script"
    } else if file_name.ends_with(".font.yml") || file_name.ends_with(".font.yaml") {
        "font"
    } else if file_name.ends_with(".image.yml") || file_name.ends_with(".image.yaml") {
        "imageAsset"
    } else if file_name == "spritesheet.yml"
        || normalized_path.contains("/spritesheets/") && file_name == "spritesheet.yaml"
    {
        "spritesheet"
    } else if file_name.ends_with(".tileset.yml")
        || file_name.ends_with(".tileset.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/tilesets/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tile-ruleset.yml")
        || file_name.ends_with(".tile-ruleset.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/rulesets/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
        "tilemap"
    } else if file_name.ends_with(".sprite.yml")
        || file_name.ends_with(".sprite.yaml")
        || file_name.ends_with(".atlas.yml")
        || file_name.ends_with(".atlas.yaml")
        || normalized_path.contains("/spritesheets/")
            && normalized_path.contains("/animations/")
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "spritesheet"
    } else if file_name.ends_with(".tileset.yml")
        || file_name.ends_with(".tileset.yaml")
        || file_name.ends_with(".tile-ruleset.yml")
        || file_name.ends_with(".tile-ruleset.yaml")
    {
        "tileset"
    } else if file_name.ends_with(".tilemap.yml") || file_name.ends_with(".tilemap.yaml") {
        "tilemap"
    } else if file_name.ends_with(".sprite.yml")
        || file_name.ends_with(".sprite.yaml")
        || file_name.ends_with(".atlas.yml")
        || file_name.ends_with(".atlas.yaml")
    {
        "spritesheet"
    } else if file_name.ends_with(".particle.yml") || file_name.ends_with(".particle.yaml") {
        "particle"
    } else if file_name.ends_with(".audio.yml") || file_name.ends_with(".audio.yaml") {
        "audio"
    } else if file_name.ends_with(".material.yml") || file_name.ends_with(".material.yaml") {
        "material"
    } else if file_name.ends_with(".ui.yml")
        || file_name.ends_with(".ui.yaml")
        || (normalized_path.starts_with("ui/") || normalized_path.contains("/ui/"))
            && matches!(extension.as_str(), "yml" | "yaml")
    {
        "ui"
    } else if file_name.ends_with(".input.yml") || file_name.ends_with(".input.yaml") {
        "input"
    } else if matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp") {
        "rawImage"
    } else if matches!(extension.as_str(), "wav" | "ogg" | "mp3" | "flac") {
        "rawAudio"
    } else if matches!(extension.as_str(), "ttf" | "otf" | "woff" | "woff2") {
        "rawFont"
    } else if matches!(extension.as_str(), "yml" | "yaml") {
        "yaml"
    } else {
        "unknown"
    }
    .to_owned()
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
mod tests {
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
}
