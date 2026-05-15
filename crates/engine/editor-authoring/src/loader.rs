use std::path::{Path, PathBuf};

use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::ModCatalog;
use amigo_runtime::Runtime;
use amigo_session::SceneSessionService;
use serde_yaml::{Mapping, Value};

use crate::graph::{
    AuthoringNode, AuthoringNodeKind, AuthoringNodeOrigin, AuthoringNodeSemantic,
    AuthoringSceneGraph, value_preview,
};
use crate::ids::{child_pointer, node_id};
use crate::refs::{collect_use_refs, mapping_get};

#[derive(Debug, Clone, Default)]
struct BuildContext {
    parent_id: Option<String>,
    owner_entity_name: Option<String>,
}

pub fn load_authoring_scene_graph(runtime: &Runtime) -> AmigoResult<AuthoringSceneGraph> {
    let scene_session = runtime.required::<SceneSessionService>()?;
    let scene_snapshot = scene_session.snapshot();
    let loaded = scene_snapshot
        .loaded_scene_document()
        .cloned()
        .ok_or_else(|| {
            AmigoError::Message("editor authoring: no loaded scene document".to_owned())
        })?;

    let mod_catalog = runtime.required::<ModCatalog>()?;
    let discovered = mod_catalog.mod_by_id(&loaded.source_mod).ok_or_else(|| {
        AmigoError::Message(format!(
            "editor authoring: loaded mod `{}` is not in ModCatalog",
            loaded.source_mod
        ))
    })?;

    let scene_file = discovered
        .scene_document_path(&loaded.scene_id)
        .unwrap_or_else(|| discovered.root_path.join(&loaded.relative_path));

    load_authoring_scene_graph_from_file(
        loaded.source_mod,
        loaded.scene_id,
        discovered.root_path.as_path(),
        scene_file,
    )
}

pub fn load_authoring_scene_graph_from_file(
    source_mod: String,
    scene_id: String,
    mod_root: &Path,
    scene_file: PathBuf,
) -> AmigoResult<AuthoringSceneGraph> {
    let root_value = read_yaml(&scene_file)?;
    let mut source_files = vec![scene_file.clone()];
    let mut root_node = file_node(
        &scene_file,
        "/",
        format!("{source_mod} / {scene_id}"),
        root_value.clone(),
        AuthoringNodeOrigin::Root,
    );

    root_node.children = build_yaml_children(
        &scene_file,
        mod_root,
        "/",
        &root_value,
        AuthoringNodeOrigin::Root,
        BuildContext {
            parent_id: Some(root_node.id.clone()),
            owner_entity_name: None,
        },
    )?;

    let use_nodes = build_use_ref_nodes(&scene_file, mod_root, &root_value, &mut source_files)?;
    root_node.children.extend(use_nodes);
    source_files.sort();
    source_files.dedup();

    Ok(AuthoringSceneGraph {
        source_mod,
        scene_id,
        root_file: scene_file,
        source_files,
        nodes: vec![root_node],
    })
}

fn build_use_ref_nodes(
    scene_file: &Path,
    mod_root: &Path,
    root: &Value,
    source_files: &mut Vec<PathBuf>,
) -> AmigoResult<Vec<AuthoringNode>> {
    let mut nodes = Vec::new();

    for use_ref in collect_use_refs(scene_file, mod_root, root)? {
        let value = read_yaml(&use_ref.path)?;
        source_files.push(use_ref.path.clone());
        let mut node = file_node(
            &use_ref.path,
            "/",
            format!("use.{} -> {}", use_ref.group, use_ref.raw),
            value.clone(),
            AuthoringNodeOrigin::UseRef,
        );
        node.kind = AuthoringNodeKind::Use;
        node.children = build_yaml_children(
            &use_ref.path,
            mod_root,
            "/",
            &value,
            AuthoringNodeOrigin::UseRef,
            BuildContext {
                parent_id: Some(node.id.clone()),
                owner_entity_name: None,
            },
        )?;
        nodes.push(node);
    }

    Ok(nodes)
}

fn build_yaml_children(
    source_file: &Path,
    mod_root: &Path,
    parent_pointer: &str,
    value: &Value,
    origin: AuthoringNodeOrigin,
    ctx: BuildContext,
) -> AmigoResult<Vec<AuthoringNode>> {
    match value {
        Value::Mapping(mapping) => {
            build_mapping_children(source_file, mod_root, parent_pointer, mapping, origin, ctx)
        }
        Value::Sequence(items) => {
            build_sequence_children(source_file, mod_root, parent_pointer, items, origin, ctx)
        }
        Value::Tagged(tagged) => build_yaml_children(
            source_file,
            mod_root,
            parent_pointer,
            &tagged.value,
            origin,
            ctx,
        ),
        _ => Ok(Vec::new()),
    }
}

fn build_mapping_children(
    source_file: &Path,
    mod_root: &Path,
    parent_pointer: &str,
    mapping: &Mapping,
    origin: AuthoringNodeOrigin,
    ctx: BuildContext,
) -> AmigoResult<Vec<AuthoringNode>> {
    let mut nodes = Vec::new();

    for (key, value) in mapping {
        let key = key.as_str().unwrap_or("<non-string-key>");
        if key == "use" || key == "uses" {
            continue;
        }

        let pointer = child_pointer(parent_pointer, key);
        let mut node = yaml_node(
            source_file,
            &pointer,
            key.to_owned(),
            classify_mapping_key(key),
            origin.clone(),
            value.clone(),
            AuthoringNodeSemantic {
                parent_id: ctx.parent_id.clone(),
                owner_entity_name: ctx.owner_entity_name.clone(),
                ..AuthoringNodeSemantic::default()
            },
        );
        node.children = build_yaml_children(
            source_file,
            mod_root,
            &pointer,
            value,
            origin.clone(),
            BuildContext {
                parent_id: Some(node.id.clone()),
                owner_entity_name: ctx.owner_entity_name.clone(),
            },
        )?;
        nodes.push(node);
    }

    Ok(nodes)
}

fn build_sequence_children(
    source_file: &Path,
    mod_root: &Path,
    parent_pointer: &str,
    items: &[Value],
    origin: AuthoringNodeOrigin,
    ctx: BuildContext,
) -> AmigoResult<Vec<AuthoringNode>> {
    let mut nodes = Vec::new();

    for (index, value) in items.iter().enumerate() {
        let pointer = child_pointer(parent_pointer, &index.to_string());
        let label = sequence_item_label(parent_pointer, index, value);
        let kind = classify_sequence_item(parent_pointer, value);
        let mut semantic = AuthoringNodeSemantic {
            parent_id: ctx.parent_id.clone(),
            owner_entity_name: ctx.owner_entity_name.clone(),
            ..AuthoringNodeSemantic::default()
        };

        if parent_pointer.ends_with("/entities") {
            semantic.owner_entity_name = mapping_get(value, "name")
                .or_else(|| mapping_get(value, "id"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            semantic.scene_object_id = mapping_get(value, "id")
                .and_then(Value::as_str)
                .map(str::to_owned);
        } else if parent_pointer.ends_with("/components") {
            semantic.component_type = mapping_get(value, "type")
                .and_then(Value::as_str)
                .map(str::to_owned);
        } else if parent_pointer.ends_with("/render_layers") {
            semantic.render_layer_id = mapping_get(value, "id")
                .and_then(Value::as_str)
                .map(str::to_owned);
        } else if parent_pointer.ends_with("/light_groups") {
            semantic.light_group_id = mapping_get(value, "id")
                .or_else(|| mapping_get(value, "name"))
                .and_then(Value::as_str)
                .map(str::to_owned);
        } else if parent_pointer.ends_with("/light_routes") {
            semantic.light_route_receiver_layer = mapping_get(value, "receiver_layer")
                .or_else(|| mapping_get(value, "layer"))
                .and_then(Value::as_str)
                .map(str::to_owned);
        } else if parent_pointer.ends_with("/post_fx") {
            semantic.post_fx_id = mapping_get(value, "id")
                .and_then(Value::as_str)
                .map(str::to_owned);
            semantic.post_fx_type = mapping_get(value, "type")
                .and_then(Value::as_str)
                .map(str::to_owned);
            semantic.post_fx_scope = Some("Frame".to_owned());
        }

        let mut node = yaml_node(
            source_file,
            &pointer,
            label,
            kind,
            origin.clone(),
            value.clone(),
            semantic.clone(),
        );
        let child_ctx = BuildContext {
            parent_id: Some(node.id.clone()),
            owner_entity_name: semantic.owner_entity_name.clone(),
        };
        node.children = build_yaml_children(
            source_file,
            mod_root,
            &pointer,
            value,
            origin.clone(),
            child_ctx,
        )?;
        if matches!(node.kind, AuthoringNodeKind::Entity) {
            if let Some(prefab_id) = crate::prefabs::prefab_id(value) {
                node.children.insert(
                    0,
                    yaml_node(
                        source_file,
                        &child_pointer(&pointer, "prefab"),
                        format!("prefab: {prefab_id}"),
                        AuthoringNodeKind::PrefabRef,
                        origin.clone(),
                        mapping_get(value, "prefab").cloned().unwrap_or(Value::Null),
                        semantic.clone(),
                    ),
                );
            }
        }
        nodes.push(node);
    }

    Ok(nodes)
}

fn sequence_item_label(parent_pointer: &str, index: usize, value: &Value) -> String {
    if parent_pointer.ends_with("/render_layers") {
        return mapping_get(value, "id")
            .and_then(Value::as_str)
            .map(|id| format!("draw layer: {id}"))
            .unwrap_or_else(|| format!("draw layer #{index}"));
    }
    if parent_pointer.ends_with("/light_groups") {
        return mapping_get(value, "id")
            .or_else(|| mapping_get(value, "name"))
            .and_then(Value::as_str)
            .map(|id| format!("light: {id}"))
            .unwrap_or_else(|| format!("light #{index}"));
    }
    if parent_pointer.ends_with("/light_routes") {
        return mapping_get(value, "receiver_layer")
            .or_else(|| mapping_get(value, "layer"))
            .and_then(Value::as_str)
            .map(|layer| format!("route: {layer}"))
            .unwrap_or_else(|| format!("route #{index}"));
    }

    if parent_pointer.ends_with("/post_fx") {
        let effect_type = mapping_get(value, "type")
            .and_then(Value::as_str)
            .unwrap_or("effect");
        let id = mapping_get(value, "id")
            .and_then(Value::as_str)
            .unwrap_or("");
        return if id.is_empty() {
            format!("{effect_type} #{index}")
        } else {
            format!("{effect_type}: {id}")
        };
    }

    if parent_pointer.ends_with("/entities") {
        return mapping_get(value, "id")
            .and_then(Value::as_str)
            .map(|id| format!("object: {id}"))
            .unwrap_or_else(|| format!("object #{index}"));
    }

    if parent_pointer.ends_with("/components") {
        return mapping_get(value, "type")
            .and_then(Value::as_str)
            .map(|kind| format!("component: {kind}"))
            .unwrap_or_else(|| format!("component #{index}"));
    }

    format!("[{index}]")
}

fn classify_mapping_key(key: &str) -> AuthoringNodeKind {
    match key {
        "scene" => AuthoringNodeKind::Scene,
        "visual2d" => AuthoringNodeKind::Visual2d,
        "render_layers" => AuthoringNodeKind::RenderLayers,
        "light_groups" => AuthoringNodeKind::LightGroups,
        "light_routes" => AuthoringNodeKind::LightRoutes,
        "post_fx" => AuthoringNodeKind::PostFx,
        "entities" => AuthoringNodeKind::Entities,
        "components" => AuthoringNodeKind::Components,
        "prefab" => AuthoringNodeKind::PrefabRef,
        "prefab_overrides" => AuthoringNodeKind::PrefabOverrides,
        _ => AuthoringNodeKind::Mapping,
    }
}

fn classify_sequence_item(parent_pointer: &str, value: &Value) -> AuthoringNodeKind {
    if parent_pointer.ends_with("/render_layers") {
        AuthoringNodeKind::RenderLayer
    } else if parent_pointer.ends_with("/light_groups") {
        AuthoringNodeKind::LightGroup
    } else if parent_pointer.ends_with("/light_routes") {
        AuthoringNodeKind::LightRoute
    } else if parent_pointer.ends_with("/post_fx") {
        AuthoringNodeKind::PostFxItem
    } else if parent_pointer.ends_with("/entities") {
        AuthoringNodeKind::Entity
    } else if parent_pointer.ends_with("/components") {
        AuthoringNodeKind::Component
    } else if matches!(value, Value::Sequence(_)) {
        AuthoringNodeKind::Sequence
    } else if matches!(value, Value::Mapping(_)) {
        AuthoringNodeKind::Mapping
    } else {
        AuthoringNodeKind::Scalar
    }
}

fn file_node(
    source_file: &Path,
    pointer: &str,
    label: String,
    value: Value,
    origin: AuthoringNodeOrigin,
) -> AuthoringNode {
    AuthoringNode {
        id: node_id(source_file, pointer),
        label,
        kind: AuthoringNodeKind::File,
        origin,
        source_file: source_file.to_path_buf(),
        yaml_pointer: pointer.to_owned(),
        editable: false,
        value_preview: value_preview(&value),
        value,
        semantic: AuthoringNodeSemantic::default(),
        children: Vec::new(),
    }
}

fn yaml_node(
    source_file: &Path,
    pointer: &str,
    label: String,
    kind: AuthoringNodeKind,
    origin: AuthoringNodeOrigin,
    value: Value,
    semantic: AuthoringNodeSemantic,
) -> AuthoringNode {
    AuthoringNode {
        id: node_id(source_file, pointer),
        label,
        kind,
        origin,
        source_file: source_file.to_path_buf(),
        yaml_pointer: pointer.to_owned(),
        editable: true,
        value_preview: value_preview(&value),
        value,
        semantic,
        children: Vec::new(),
    }
}

fn read_yaml(path: &Path) -> AmigoResult<Value> {
    let raw = std::fs::read_to_string(path).map_err(|error| {
        AmigoError::Message(format!("failed to read `{}`: {error}", path.display()))
    })?;

    serde_yaml::from_str::<Value>(&raw).map_err(|error| {
        AmigoError::Message(format!("failed to parse `{}`: {error}", path.display()))
    })
}
