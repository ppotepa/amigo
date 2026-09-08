use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::component_envelope::SceneComponentEnvelope;
use super::core::{
    SceneActivationSetDocument, SceneAudioCueDocument, SceneCollisionEventRule2dDocument,
    SceneDocument, SceneEntityDocument, SceneMetadataDocument, ScenePrefabInstanceDocument,
    ScenePrefabOverrideDocument, ScenePropertyValueDocument, SceneStateValueDocument,
    SceneTransitionDocument,
};
use super::defaults::{default_entity_lifecycle_flag, default_scene_document_version};
use super::render_values::{SceneTransform2Document, SceneTransform3Document};
use super::visual2d::{PostFx2dDocument, SceneVisual2dDocument};
use super::{
    SceneComponentDocument, is_builtin_component_type, is_rejected_retired_component_type,
    plugin_component_document,
};
use crate::{ComponentSchemaRegistry, SceneDocumentError, SceneDocumentResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct RawSceneDocument {
    #[serde(default = "default_scene_document_version")]
    version: u32,
    scene: SceneMetadataDocument,
    #[serde(default)]
    panels: Vec<super::ScenePanelReferenceDocument>,
    #[serde(default)]
    transitions: Vec<SceneTransitionDocument>,
    #[serde(default)]
    collision_events: Vec<SceneCollisionEventRule2dDocument>,
    #[serde(default)]
    audio_cues: Vec<SceneAudioCueDocument>,
    #[serde(default)]
    activation_sets: Vec<SceneActivationSetDocument>,
    #[serde(default)]
    visual2d: SceneVisual2dDocument,
    #[serde(default)]
    state: BTreeMap<String, SceneStateValueDocument>,
    #[serde(default)]
    entities: Vec<RawSceneEntityDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct RawSceneEntityDocument {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    groups: Vec<String>,
    #[serde(default = "default_entity_lifecycle_flag")]
    visible: bool,
    #[serde(default = "default_entity_lifecycle_flag")]
    simulation_enabled: bool,
    #[serde(default = "default_entity_lifecycle_flag")]
    collision_enabled: bool,
    #[serde(default)]
    properties: BTreeMap<String, ScenePropertyValueDocument>,
    #[serde(default)]
    transform2: Option<SceneTransform2Document>,
    #[serde(default)]
    transform3: Option<SceneTransform3Document>,
    #[serde(default)]
    post_fx: Vec<PostFx2dDocument>,
    #[serde(default)]
    prefab: Option<ScenePrefabInstanceDocument>,
    #[serde(default)]
    prefab_overrides: Vec<ScenePrefabOverrideDocument>,
    #[serde(default)]
    components: Vec<Value>,
}

pub fn load_scene_document_from_str(source: &str) -> SceneDocumentResult<SceneDocument> {
    parse_scene_document(source, None)
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn load_scene_document_from_str_with_component_schemas(
    source: &str,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> SceneDocumentResult<SceneDocument> {
    parse_scene_document(source, component_schemas)
        .map_err(|source| SceneDocumentError::Parse { path: None, source })
}

pub fn load_scene_document_from_path(path: impl AsRef<Path>) -> SceneDocumentResult<SceneDocument> {
    load_scene_document_from_path_with_component_schemas(path, None)
}

pub fn load_scene_document_from_path_with_component_schemas(
    path: impl AsRef<Path>,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> SceneDocumentResult<SceneDocument> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| SceneDocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    parse_scene_document(&raw, component_schemas).map_err(|source| SceneDocumentError::Parse {
        path: Some(path.to_path_buf()),
        source,
    })
}

pub(crate) fn parse_scene_document_value(
    value: Value,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> Result<SceneDocument, serde_yaml::Error> {
    let raw = serde_yaml::from_value::<RawSceneDocument>(value)?;

    Ok(SceneDocument {
        version: raw.version,
        scene: raw.scene,
        panels: raw.panels,
        transitions: raw.transitions,
        collision_events: raw.collision_events,
        audio_cues: raw.audio_cues,
        activation_sets: raw.activation_sets,
        visual2d: raw.visual2d,
        state: raw.state,
        entities: raw
            .entities
            .into_iter()
            .map(|entity| parse_scene_entity(entity, component_schemas))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_scene_document(
    source: &str,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> Result<SceneDocument, serde_yaml::Error> {
    let value = serde_yaml::from_str::<Value>(source)?;
    parse_scene_document_value(value, component_schemas)
}

fn parse_scene_entity(
    raw: RawSceneEntityDocument,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> Result<SceneEntityDocument, serde_yaml::Error> {
    Ok(SceneEntityDocument {
        id: raw.id,
        name: raw.name,
        tags: raw.tags,
        groups: raw.groups,
        visible: raw.visible,
        simulation_enabled: raw.simulation_enabled,
        collision_enabled: raw.collision_enabled,
        properties: raw.properties,
        transform2: raw.transform2,
        transform3: raw.transform3,
        post_fx: raw.post_fx,
        prefab: raw.prefab,
        prefab_overrides: raw.prefab_overrides,
        components: raw
            .components
            .into_iter()
            .map(|component| parse_scene_component(component, component_schemas))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn parse_scene_component(
    value: Value,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> Result<SceneComponentDocument, serde_yaml::Error> {
    let envelope = serde_yaml::from_value::<SceneComponentEnvelope>(value.clone())?;

    if is_rejected_retired_component_type(&envelope.component_type) {
        return serde_yaml::from_value(value);
    }

    if let Some(component_schemas) = component_schemas {
        if let Some((component_type, payload)) = component_schemas
            .parse_plugin_payload_with_canonical_type(
                envelope.component_type.as_str(),
                envelope.payload.clone(),
            )
            .transpose()?
        {
            return Ok(plugin_component_document(component_type, payload));
        }

        if let Some(descriptor) = component_schemas.get(envelope.component_type.as_str()) {
            return Ok(plugin_component_document(
                descriptor.id.as_str().to_owned(),
                Value::Mapping(envelope.payload),
            ));
        }
    }

    if is_builtin_component_type(&envelope.component_type) {
        return serde_yaml::from_value(value);
    }

    Ok(plugin_component_document(
        envelope.component_type,
        Value::Mapping(envelope.payload),
    ))
}

pub fn scene_document_path(
    mod_root: impl AsRef<Path>,
    relative_document_path: impl AsRef<Path>,
) -> PathBuf {
    mod_root.as_ref().join(relative_document_path.as_ref())
}
