use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use amigo_runtime::Runtime;
use amigo_scene::{
    ComponentRegistry, EditorPropertyValueKind, SceneComponentDocument, SceneDocument,
    SceneEntityDocument, component_registry_for_runtime, default_component_registry,
};

use crate::{ControlRange, ControlValueType, path::sanitize_console_segment};

#[derive(Debug, Clone, Default)]
pub struct RuntimeControlSceneMetadata {
    pub target_lookup: BTreeMap<String, RuntimeControlTargetMetadata>,
    pub known_components: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeControlTargetMetadata {
    pub canonical_target: String,
    pub source_file: Option<String>,
    pub source_pointer: Option<String>,
    pub source_id: Option<String>,
    pub display_name: String,
    pub entity_name: String,
    pub aliases: Vec<String>,
    pub components: Vec<RuntimeControlComponentMetadata>,
}

#[derive(Debug, Clone)]
pub struct RuntimeControlComponentMetadata {
    pub console_component: String,
    pub source_component: String,
    pub source_pointer: Option<String>,
    pub properties: Vec<RuntimeControlPropertyMetadata>,
}

#[derive(Debug, Clone)]
pub struct RuntimeControlPropertyMetadata {
    pub property_path: String,
    pub value_type: ControlValueType,
    pub range: Option<ControlRange>,
    pub writable: bool,
    pub source_pointer: Option<String>,
}

impl RuntimeControlSceneMetadata {
    pub fn is_known_component(&self, component: &str) -> bool {
        self.known_components.contains(component)
    }
}

pub fn build_scene_metadata(
    document: &SceneDocument,
    relative_document_path: &Path,
) -> RuntimeControlSceneMetadata {
    let registry = default_component_registry();
    build_scene_metadata_with_registry(document, relative_document_path, &registry)
}

pub fn build_scene_metadata_for_runtime(
    runtime: &Runtime,
    document: &SceneDocument,
    relative_document_path: &Path,
) -> RuntimeControlSceneMetadata {
    let registry = component_registry_for_runtime(runtime);
    build_scene_metadata_with_registry(document, relative_document_path, &registry)
}

pub fn build_scene_metadata_with_registry(
    document: &SceneDocument,
    relative_document_path: &Path,
    registry: &ComponentRegistry,
) -> RuntimeControlSceneMetadata {
    let mut metadata = RuntimeControlSceneMetadata::default();

    for entity in &document.entities {
        let preferred_target = preferred_target_alias(entity);
        let display_target = display_target_alias(entity);
        let canonical_target = preferred_target
            .clone()
            .unwrap_or_else(|| display_target.clone());
        let source_file = Some(relative_document_path.display().to_string());
        let source_pointer = Some(entity_source_pointer(document, entity));
        let mut aliases = vec![canonical_target.clone()];
        if canonical_target != display_target {
            aliases.push(display_target);
        }
        aliases.push(sanitize_entity_lookup(entity.id.as_str()));
        aliases.push(sanitize_entity_lookup(entity.display_name().as_str()));
        aliases.sort();
        aliases.dedup();

        let components = entity
            .components
            .iter()
            .enumerate()
            .map(|(index, component)| RuntimeControlComponentMetadata {
                console_component: console_component_name(component.kind()),
                source_component: component.kind().to_owned(),
                source_pointer: Some(format!(
                    "{}/components/{index}",
                    source_pointer.clone().unwrap()
                )),
                properties: component_property_metadata(
                    registry,
                    component,
                    format!("{}/components/{index}", source_pointer.clone().unwrap()),
                ),
            })
            .collect::<Vec<_>>();

        for component in &components {
            metadata
                .known_components
                .insert(component.console_component.clone());
            metadata
                .known_components
                .insert(component.source_component.clone());
        }

        let target = RuntimeControlTargetMetadata {
            canonical_target: canonical_target.clone(),
            source_file,
            source_pointer,
            source_id: Some(entity.id.clone()),
            display_name: entity.display_name(),
            entity_name: entity.display_name(),
            aliases: aliases.clone(),
            components,
        };

        for alias in aliases {
            metadata.target_lookup.insert(alias, target.clone());
        }
    }

    metadata
}

fn component_property_metadata(
    registry: &amigo_scene::ComponentRegistry,
    component: &SceneComponentDocument,
    source_pointer: String,
) -> Vec<RuntimeControlPropertyMetadata> {
    let Some(descriptor) = registry.descriptor_by_type_name(component.kind()) else {
        return Vec::new();
    };
    descriptor
        .properties
        .iter()
        .filter_map(|property| {
            let value_type = control_value_type(property.value_kind)?;
            Some(RuntimeControlPropertyMetadata {
                property_path: property.path.to_owned(),
                value_type,
                range: property.number_constraints.map(|constraints| ControlRange {
                    min: constraints.min.map(f64::from),
                    max: constraints.max.map(f64::from),
                }),
                writable: !matches!(property.access, amigo_scene::EditorPropertyAccess::ReadOnly),
                source_pointer: Some(format!("{source_pointer}/{}", property.path)),
            })
        })
        .collect()
}

fn control_value_type(value_kind: EditorPropertyValueKind) -> Option<ControlValueType> {
    match value_kind {
        EditorPropertyValueKind::Bool => Some(ControlValueType::Bool),
        EditorPropertyValueKind::Number => Some(ControlValueType::F32),
        EditorPropertyValueKind::String => Some(ControlValueType::String),
        EditorPropertyValueKind::AssetRef => Some(ControlValueType::AssetRef),
        EditorPropertyValueKind::Vec2 => Some(ControlValueType::Vec2),
        EditorPropertyValueKind::Vec3 => Some(ControlValueType::Vec3),
        EditorPropertyValueKind::Color => Some(ControlValueType::Color),
        EditorPropertyValueKind::Enum => Some(ControlValueType::String),
    }
}

fn entity_source_pointer(document: &SceneDocument, entity: &SceneEntityDocument) -> String {
    let index = document
        .entities
        .iter()
        .position(|candidate| candidate.id == entity.id)
        .unwrap_or(0);
    format!("/entities/{index}")
}

fn display_target_alias(entity: &SceneEntityDocument) -> String {
    sanitize_entity_lookup(entity.display_name().as_str())
}

fn sanitize_entity_lookup(value: &str) -> String {
    value
        .split('.')
        .map(sanitize_console_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn preferred_target_alias(entity: &SceneEntityDocument) -> Option<String> {
    let id_target = sanitize_entity_lookup(entity.id.as_str());
    if let Some(layer) = primary_render_layer(entity) {
        if layer.contains('.') {
            return Some(layer);
        }
    }
    if let Some(suffix) = entity.id.strip_prefix("beacon-") {
        return Some(format!(
            "lighting.beacon.{}",
            sanitize_console_segment(suffix)
        ));
    }
    Some(id_target)
}

fn primary_render_layer(entity: &SceneEntityDocument) -> Option<String> {
    entity
        .components
        .iter()
        .find_map(|component| component.primary_render_layer().map(str::to_owned))
}

fn console_component_name(kind: &str) -> String {
    match kind {
        "BeaconLight2D" => "Beacon2D".to_owned(),
        other => other.to_owned(),
    }
}
