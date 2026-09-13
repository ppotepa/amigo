use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde_yaml::{Mapping, Value};

use super::{
    SceneDocument, SceneFrameClockDocument, SceneFramePresentationDocument,
    SceneSchedulingDocument, parse_scene_document_value,
};
use crate::{ComponentSchemaRegistry, SceneDocumentError, SceneDocumentResult};

#[derive(Debug)]
pub struct CompiledSceneDocument {
    pub document: SceneDocument,
    pub scheduling: Option<SceneSchedulingDocument>,
    pub value: Value,
    pub dependencies: Vec<SceneDocumentDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneDocumentDependency {
    pub path: PathBuf,
    pub kind: SceneDocumentDependencyKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneDocumentDependencyKind {
    Use,
    Scheduling,
    UiDocument,
    UiTheme,
    UiModelBindings,
    Script,
    LocalAsset,
}

pub fn compile_scene_document_from_path(
    scene_path: impl AsRef<Path>,
    mod_root: impl AsRef<Path>,
    mod_id: &str,
) -> SceneDocumentResult<CompiledSceneDocument> {
    compile_scene_document_from_path_with_component_schemas(scene_path, mod_root, mod_id, None)
}

pub fn compile_scene_document_from_path_with_component_schemas(
    scene_path: impl AsRef<Path>,
    mod_root: impl AsRef<Path>,
    mod_id: &str,
    component_schemas: Option<&ComponentSchemaRegistry>,
) -> SceneDocumentResult<CompiledSceneDocument> {
    let scene_path = scene_path.as_ref();
    let mod_root = mod_root.as_ref();
    let mut dependencies = Vec::new();
    let mut value = read_yaml(scene_path)?;
    let mut scheduling_values = Vec::new();

    if let Some(scheduling) = remove_mapping_key(&mut value, "scheduling") {
        scheduling_values.push(scheduling);
    }

    if let Some(path) = remove_mapping_key(&mut value, "script")
        .and_then(|value| string_value(&value).map(str::to_owned))
    {
        dependencies.push(SceneDocumentDependency {
            path: resolve_reference(scene_path, mod_root, &path)?,
            kind: SceneDocumentDependencyKind::Script,
        });
    }

    let use_value =
        remove_mapping_key(&mut value, "use").or_else(|| remove_mapping_key(&mut value, "uses"));
    for use_ref in use_entries(use_value.as_ref())? {
        let path = resolve_reference(scene_path, mod_root, &use_ref.path)?;
        let mut fragment = read_yaml(&path)?;
        dependencies.push(SceneDocumentDependency {
            path: path.clone(),
            kind: if use_ref.kind == UseEntryKind::Scheduling {
                SceneDocumentDependencyKind::Scheduling
            } else {
                SceneDocumentDependencyKind::Use
            },
        });
        if let Some(scheduling) = remove_mapping_key(&mut fragment, "scheduling") {
            scheduling_values.push(scheduling);
        }
        expand_authoring_refs(&mut fragment, &path, mod_root, &mut dependencies)?;
        merge_scene_fragment(&mut value, fragment);
    }

    expand_authoring_refs(&mut value, scene_path, mod_root, &mut dependencies)?;
    validate_compiled_value(&value)?;

    let document =
        parse_scene_document_value(value.clone(), component_schemas).map_err(|source| {
            SceneDocumentError::Parse {
                path: Some(scene_path.to_path_buf()),
                source,
            }
        })?;

    let scheduling = merge_scene_scheduling_documents(scheduling_values)?;

    let _ = mod_id;
    Ok(CompiledSceneDocument {
        document,
        scheduling,
        value,
        dependencies,
    })
}

fn read_yaml(path: &Path) -> SceneDocumentResult<Value> {
    let raw = std::fs::read_to_string(path).map_err(|source| SceneDocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_str::<Value>(&raw).map_err(|source| SceneDocumentError::Parse {
        path: Some(path.to_path_buf()),
        source,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UseEntryKind {
    Generic,
    Scheduling,
}

#[derive(Clone, Debug)]
struct UseEntry {
    path: String,
    kind: UseEntryKind,
}

fn use_entries(value: Option<&Value>) -> SceneDocumentResult<Vec<UseEntry>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    match value {
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                let kind = if key.as_str() == Some("scheduling") {
                    UseEntryKind::Scheduling
                } else {
                    UseEntryKind::Generic
                };
                collect_use_entry(value, kind, &mut entries)?;
            }
        }
        Value::Sequence(items) => {
            for value in items {
                collect_use_entry(value, UseEntryKind::Generic, &mut entries)?;
            }
        }
        value => collect_use_entry(value, UseEntryKind::Generic, &mut entries)?,
    }
    Ok(entries)
}

fn collect_use_entry(
    value: &Value,
    kind: UseEntryKind,
    entries: &mut Vec<UseEntry>,
) -> SceneDocumentResult<()> {
    match value {
        Value::String(path) => entries.push(UseEntry {
            path: path.clone(),
            kind,
        }),
        Value::Sequence(items) => {
            for item in items {
                collect_use_entry(item, kind, entries)?;
            }
        }
        _ => {
            return Err(compile_error(
                "scene use entries must be strings or string lists",
            ));
        }
    }
    Ok(())
}

fn merge_scene_fragment(target: &mut Value, fragment: Value) {
    let Some(fragment) = fragment.as_mapping() else {
        return;
    };
    for key in [
        "visual2d",
        "state",
        "entities",
        "transitions",
        "collision_events",
        "audio_cues",
        "activation_sets",
    ] {
        if let Some(value) = fragment.get(Value::String(key.to_owned())) {
            merge_runtime_key(target, key, value.clone());
        }
    }
}

fn merge_runtime_key(target: &mut Value, key: &str, value: Value) {
    let Some(target_mapping) = target.as_mapping_mut() else {
        return;
    };
    let key_value = Value::String(key.to_owned());
    match (target_mapping.get_mut(&key_value), value) {
        (Some(Value::Sequence(existing)), Value::Sequence(mut incoming)) => {
            existing.append(&mut incoming)
        }
        (Some(Value::Mapping(existing)), Value::Mapping(incoming)) => {
            merge_mapping(existing, incoming)
        }
        (None, value) => {
            target_mapping.insert(key_value, value);
        }
        (Some(existing), value) => {
            *existing = value;
        }
    }
}

fn merge_mapping(existing: &mut Mapping, incoming: Mapping) {
    for (key, value) in incoming {
        match (existing.get_mut(&key), value) {
            (Some(Value::Sequence(existing)), Value::Sequence(mut incoming)) => {
                existing.append(&mut incoming)
            }
            (Some(Value::Mapping(existing)), Value::Mapping(incoming)) => {
                merge_mapping(existing, incoming)
            }
            (None, value) => {
                existing.insert(key, value);
            }
            (Some(existing), value) => {
                *existing = value;
            }
        }
    }
}

fn expand_authoring_refs(
    value: &mut Value,
    scene_path: &Path,
    mod_root: &Path,
    dependencies: &mut Vec<SceneDocumentDependency>,
) -> SceneDocumentResult<()> {
    let Some(entities) = mapping_get_mut(value, "entities").and_then(Value::as_sequence_mut) else {
        return Ok(());
    };
    for entity in entities {
        let Some(components) =
            mapping_get_mut(entity, "components").and_then(Value::as_sequence_mut)
        else {
            continue;
        };
        for component in components {
            expand_component_ref(component, scene_path, mod_root, dependencies)?;
        }
    }
    Ok(())
}

fn expand_component_ref(
    component: &mut Value,
    scene_path: &Path,
    mod_root: &Path,
    dependencies: &mut Vec<SceneDocumentDependency>,
) -> SceneDocumentResult<()> {
    let kind = mapping_get(component, "type")
        .and_then(string_value)
        .map(str::to_owned);
    let Some(kind) = kind.as_deref() else {
        return Ok(());
    };

    match kind {
        "UiDocumentRef" => {
            let asset = mapping_get(component, "asset")
                .and_then(string_value)
                .ok_or_else(|| compile_error("UiDocumentRef requires asset"))?;
            let path = resolve_reference(scene_path, mod_root, asset)?;
            let document = read_yaml(&path)?;
            dependencies.push(SceneDocumentDependency {
                path,
                kind: SceneDocumentDependencyKind::UiDocument,
            });
            let target = mapping_get(&document, "target")
                .cloned()
                .ok_or_else(|| compile_error("UiDocumentRef target document requires target"))?;
            let root = mapping_get(&document, "root")
                .cloned()
                .ok_or_else(|| compile_error("UiDocumentRef target document requires root"))?;
            *component = component_mapping("UiDocument", [("target", target), ("root", root)]);
        }
        "UiThemeRef" => {
            let asset = mapping_get(component, "asset")
                .and_then(string_value)
                .ok_or_else(|| compile_error("UiThemeRef requires asset"))?;
            let path = resolve_reference(scene_path, mod_root, asset)?;
            let theme = read_yaml(&path)?;
            dependencies.push(SceneDocumentDependency {
                path,
                kind: SceneDocumentDependencyKind::UiTheme,
            });
            let id = mapping_get(&theme, "id")
                .and_then(string_value)
                .ok_or_else(|| compile_error("UiThemeRef target document requires id"))?
                .to_owned();
            let palette = mapping_get(&theme, "palette")
                .cloned()
                .ok_or_else(|| compile_error("UiThemeRef target document requires palette"))?;
            let theme_value = component_mapping(
                "",
                [("id", Value::String(id.clone())), ("palette", palette)],
            );
            let mut mapping = Mapping::new();
            mapping.insert(
                Value::String("type".to_owned()),
                Value::String("UiThemeSet".to_owned()),
            );
            mapping.insert(Value::String("active".to_owned()), Value::String(id));
            mapping.insert(
                Value::String("themes".to_owned()),
                Value::Sequence(vec![theme_value]),
            );
            *component = Value::Mapping(mapping);
        }
        "UiModelBindingsRef" => {
            let source = mapping_get(component, "source")
                .and_then(string_value)
                .ok_or_else(|| compile_error("UiModelBindingsRef requires source"))?;
            let path = resolve_reference(scene_path, mod_root, source)?;
            let bindings = read_yaml(&path)?;
            dependencies.push(SceneDocumentDependency {
                path,
                kind: SceneDocumentDependencyKind::UiModelBindings,
            });
            let bindings = mapping_get(&bindings, "bindings")
                .cloned()
                .unwrap_or(bindings);
            *component = component_mapping("UiModelBindings", [("bindings", bindings)]);
        }
        _ => {}
    }
    Ok(())
}

fn validate_compiled_value(value: &Value) -> SceneDocumentResult<()> {
    let mut entity_ids = BTreeSet::new();
    if let Some(entities) = mapping_get(value, "entities").and_then(Value::as_sequence) {
        for entity in entities {
            if let Some(id) = mapping_get(entity, "id").and_then(string_value) {
                if !entity_ids.insert(id.to_owned()) {
                    return Err(compile_error(format!("duplicate scene entity id `{id}`")));
                }
            }
        }
    }

    if let Some(visual2d) = mapping_get(value, "visual2d") {
        reject_duplicate_ids(visual2d, "render_layers", "render layer")?;
        reject_duplicate_ids(visual2d, "light_groups", "light group")?;
        reject_duplicate_ids(visual2d, "post_fx", "post-fx")?;
    }
    Ok(())
}

fn reject_duplicate_ids(value: &Value, key: &str, label: &str) -> SceneDocumentResult<()> {
    let mut ids = BTreeSet::new();
    if let Some(items) = mapping_get(value, key).and_then(Value::as_sequence) {
        for item in items {
            if let Some(id) = mapping_get(item, "id").and_then(string_value) {
                if !ids.insert(id.to_owned()) {
                    return Err(compile_error(format!("duplicate {label} id `{id}`")));
                }
            }
        }
    }
    Ok(())
}

fn resolve_reference(
    base_path: &Path,
    mod_root: &Path,
    value: &str,
) -> SceneDocumentResult<PathBuf> {
    if let Some(rest) = value.strip_prefix("mod:") {
        reject_unsafe_relative(rest)?;
        return resolve_with_yaml_default_extension(&mod_root.join(rest));
    }
    reject_unsafe_relative(value)?;
    let base = base_path.parent().unwrap_or_else(|| Path::new(""));
    resolve_with_yaml_default_extension(&base.join(value))
}

fn reject_unsafe_relative(value: &str) -> SceneDocumentResult<()> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(compile_error(format!(
            "unsafe absolute scene reference `{value}`"
        )));
    }
    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(compile_error(format!("unsafe scene reference `{value}`")));
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_with_yaml_default_extension(path: &Path) -> SceneDocumentResult<PathBuf> {
    if path.extension().is_some() {
        return Ok(path.to_path_buf());
    }
    let yml = path.with_extension("yml");
    if yml.exists() {
        return Ok(yml);
    }
    Ok(path.with_extension("yaml"))
}

fn remove_mapping_key(value: &mut Value, key: &str) -> Option<Value> {
    value
        .as_mapping_mut()
        .and_then(|mapping| mapping.remove(Value::String(key.to_owned())))
}

fn mapping_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn mapping_get_mut<'a>(value: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    value
        .as_mapping_mut()?
        .get_mut(Value::String(key.to_owned()))
}

fn string_value(value: &Value) -> Option<&str> {
    value.as_str()
}

fn component_mapping<const N: usize>(kind: &str, fields: [(&str, Value); N]) -> Value {
    let mut mapping = Mapping::new();
    if !kind.is_empty() {
        mapping.insert(
            Value::String("type".to_owned()),
            Value::String(kind.to_owned()),
        );
    }
    for (key, value) in fields {
        mapping.insert(Value::String(key.to_owned()), value);
    }
    Value::Mapping(mapping)
}

fn compile_error(message: impl Into<String>) -> SceneDocumentError {
    SceneDocumentError::Compile {
        path: None,
        message: message.into(),
    }
}

fn merge_scene_scheduling_documents(
    values: Vec<Value>,
) -> SceneDocumentResult<Option<SceneSchedulingDocument>> {
    if values.is_empty() {
        return Ok(None);
    }

    let mut merged = SceneSchedulingDocument::default();
    for value in values {
        let parsed: SceneSchedulingDocument = serde_yaml::from_value(value)
            .map_err(|source| SceneDocumentError::Parse { path: None, source })?;

        if parsed.mode.is_some() {
            merged.mode = parsed.mode;
        }
        if parsed.max_workers.is_some() {
            merged.max_workers = parsed.max_workers;
        }
        if parsed.allow_frame_latency.is_some() {
            merged.allow_frame_latency = parsed.allow_frame_latency;
        }
        if parsed.frame_clock.is_some() {
            merged.frame_clock =
                merge_frame_clock_documents(merged.frame_clock.take(), parsed.frame_clock);
        }
        merged.strict = merged.strict || parsed.strict;
        merged.overrides.extend(parsed.overrides);
    }

    Ok(Some(merged))
}

fn merge_frame_clock_documents(
    base: Option<SceneFrameClockDocument>,
    incoming: Option<SceneFrameClockDocument>,
) -> Option<SceneFrameClockDocument> {
    let incoming = incoming?;
    let mut merged = base.unwrap_or_default();

    if incoming.strategy.is_some() {
        merged.strategy = incoming.strategy;
    }
    if incoming.simulation_fps.is_some() {
        merged.simulation_fps = incoming.simulation_fps;
    }
    if incoming.render_fps.is_some() {
        merged.render_fps = incoming.render_fps;
    }
    if incoming.max_catch_up_ticks.is_some() {
        merged.max_catch_up_ticks = incoming.max_catch_up_ticks;
    }
    if incoming.clamp_frame_delta_seconds.is_some() {
        merged.clamp_frame_delta_seconds = incoming.clamp_frame_delta_seconds;
    }
    if incoming.presentation.is_some() {
        merged.presentation =
            merge_frame_presentation_documents(merged.presentation.take(), incoming.presentation);
    }

    Some(merged)
}

fn merge_frame_presentation_documents(
    base: Option<SceneFramePresentationDocument>,
    incoming: Option<SceneFramePresentationDocument>,
) -> Option<SceneFramePresentationDocument> {
    let incoming = incoming?;
    let mut merged = base.unwrap_or_default();

    if incoming.cache_game_frame.is_some() {
        merged.cache_game_frame = incoming.cache_game_frame;
    }
    if incoming.hold_last_game_frame.is_some() {
        merged.hold_last_game_frame = incoming.hold_last_game_frame;
    }
    if incoming.game_ui.is_some() {
        merged.game_ui = incoming.game_ui;
    }
    if incoming.devtools.is_some() {
        merged.devtools = incoming.devtools;
    }
    if incoming.editor.is_some() {
        merged.editor = incoming.editor;
    }
    if incoming.debug_overlay.is_some() {
        merged.debug_overlay = incoming.debug_overlay;
    }

    Some(merged)
}
