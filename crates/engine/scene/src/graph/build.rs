use std::collections::{BTreeMap, BTreeSet};

use crate::document::SceneComponentDocument as ComponentDocument;
use crate::document::{
    LightMap2dSourceRefDocument, PostFx2dDocument, SceneComponentDocument, SceneDocument,
    SceneEntityDocument,
};
use crate::metadata_traits::MetadataTraitKind;
use crate::{ComponentGraphProviderRegistry, ComponentSchemaRegistry};

use super::component_capabilities::component_2d_traits;
use super::semantics::{SceneGraphSemanticRole, SceneGraphSemantics};
use super::{
    AuthoringSource, SceneGraphDiagnostic, SceneGraphNode, SceneGraphNodeId, SceneGraphNodeKind,
    SceneReferenceEdge, SceneReferenceKind, SceneReferenceTargetKind, SemanticSceneGraph,
};

pub fn build_semantic_scene_graph(
    document: &SceneDocument,
    source_file: impl Into<String>,
) -> SemanticSceneGraph {
    build_semantic_scene_graph_with_services(document, source_file, None, None)
}

pub fn build_semantic_scene_graph_for_runtime(
    runtime: &amigo_runtime::Runtime,
    document: &SceneDocument,
    source_file: impl Into<String>,
) -> SemanticSceneGraph {
    let schemas = runtime.resolve::<ComponentSchemaRegistry>();
    let graph_providers = runtime.resolve::<ComponentGraphProviderRegistry>();
    build_semantic_scene_graph_with_services(
        document,
        source_file,
        schemas.as_deref(),
        graph_providers.as_deref(),
    )
}

fn build_semantic_scene_graph_with_services(
    document: &SceneDocument,
    source_file: impl Into<String>,
    schemas: Option<&ComponentSchemaRegistry>,
    graph_providers: Option<&ComponentGraphProviderRegistry>,
) -> SemanticSceneGraph {
    let source_file = source_file.into();
    let mut graph = SemanticSceneGraph::new(document.scene.id.as_str());

    let root = graph.root.clone();
    if let Some(root_node) = graph.nodes.get_mut(&root) {
        root_node.kind = SceneGraphNodeKind::Scene2d;
        root_node.semantics = SceneGraphSemantics::new(SceneGraphSemanticRole::Scene2D)
            .with_traits([
                MetadataTraitKind::SceneDocument,
                MetadataTraitKind::HasEntities,
                MetadataTraitKind::HasDiagnostics,
                MetadataTraitKind::PostFxHost2D,
            ])
            .with_post_fx_host(format!("scene:{}:visual2d", document.scene.id), "Frame");
    }

    let settings = graph.add_child(
        &root,
        SceneGraphNode::new("settings", "Settings", SceneGraphNodeKind::Settings).with_semantics(
            SceneGraphSemantics::new(SceneGraphSemanticRole::SceneSettings2D),
        ),
    );

    let visual2d = graph.add_child(
        &settings,
        SceneGraphNode::new(
            "settings/visual2d",
            "Visual2D",
            SceneGraphNodeKind::Visual2d,
        )
        .with_source(AuthoringSource::new(source_file.clone(), "/visual2d")),
    );

    let draw_layers_parent = graph.add_child(
        &visual2d,
        SceneGraphNode::new(
            "settings/visual2d/draw_layers",
            "Draw Layers",
            SceneGraphNodeKind::DrawLayers,
        )
        .with_source(AuthoringSource::new(
            source_file.clone(),
            "/visual2d/render_layers",
        )),
    );

    let frame_post_fx_host = graph.add_child(
        &visual2d,
        SceneGraphNode::new(
            "settings/visual2d/frame_post_fx",
            "Frame Post FX",
            SceneGraphNodeKind::FramePostFxHost,
        )
        .with_source(AuthoringSource::new(
            source_file.clone(),
            "/visual2d/post_fx",
        )),
    );

    let light_groups_parent = graph.add_child(
        &visual2d,
        SceneGraphNode::new(
            "settings/visual2d/light_groups",
            "Light Groups",
            SceneGraphNodeKind::LightGroups,
        )
        .with_source(AuthoringSource::new(
            source_file.clone(),
            "/visual2d/light_groups",
        )),
    );

    let light_routes_parent = graph.add_child(
        &visual2d,
        SceneGraphNode::new(
            "settings/visual2d/light_routes",
            "Light Routes",
            SceneGraphNodeKind::LightRoutes,
        )
        .with_source(AuthoringSource::new(
            source_file.clone(),
            "/visual2d/light_routes",
        )),
    );

    let objects_parent = graph.add_child(
        &root,
        SceneGraphNode::new("objects", "Objects", SceneGraphNodeKind::Objects)
            .with_source(AuthoringSource::new(source_file.clone(), "/entities")),
    );

    let mut draw_layers = BTreeMap::<String, SceneGraphNodeId>::new();
    let mut scene_objects = BTreeMap::<String, SceneGraphNodeId>::new();
    let mut light_groups = BTreeMap::<String, SceneGraphNodeId>::new();

    for (index, layer) in document.visual2d.render_layers.iter().enumerate() {
        let id = node_id("draw_layer", layer.id.as_str());
        let node_id = graph.add_child(
            &draw_layers_parent,
            SceneGraphNode::new(
                id.clone(),
                layer.label.clone().unwrap_or_else(|| layer.id.clone()),
                SceneGraphNodeKind::DrawLayer,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/visual2d/render_layers/{index}"),
            ))
            .with_semantics(
                SceneGraphSemantics::new(SceneGraphSemanticRole::DrawLayer2D)
                    .with_traits([
                        MetadataTraitKind::DrawLayer2D,
                        MetadataTraitKind::PostFxHost2D,
                        MetadataTraitKind::Patchable,
                        MetadataTraitKind::RuntimeControllable,
                    ])
                    .with_post_fx_host(format!("draw_layer:{}", layer.id), "DrawLayer"),
            ),
        );
        draw_layers.insert(layer.id.clone(), node_id);
    }

    for (index, effect) in document.visual2d.post_fx.iter().enumerate() {
        let effect_id = effect.id();
        graph.add_child(
            &frame_post_fx_host,
            SceneGraphNode::new(
                node_id("frame_post_fx", effect_id),
                effect_id,
                SceneGraphNodeKind::PostFxItem,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/visual2d/post_fx/{index}"),
            )),
        );
    }

    for (index, group) in document.visual2d.light_groups.iter().enumerate() {
        let node = graph.add_child(
            &light_groups_parent,
            SceneGraphNode::new(
                node_id("light_group", group.id.as_str()),
                group.label.clone().unwrap_or_else(|| group.id.clone()),
                SceneGraphNodeKind::LightGroup,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/visual2d/light_groups/{index}"),
            )),
        );
        light_groups.insert(group.id.clone(), node);
    }

    for (index, route) in document.visual2d.light_routes.iter().enumerate() {
        let route_node = graph.add_child(
            &light_routes_parent,
            SceneGraphNode::new(
                node_id("light_route", format!("route_{index}").as_str()),
                format!("Route: {}", route.receiver_layer),
                SceneGraphNodeKind::LightRoute,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/visual2d/light_routes/{index}"),
            )),
        );

        add_resolved_or_missing_ref(
            &mut graph,
            route_node.clone(),
            "receiver_layer",
            SceneReferenceKind::LightRouteReceiver,
            SceneReferenceTargetKind::DrawLayer,
            route.receiver_layer.as_str(),
            &draw_layers,
            "missing_light_route_receiver_layer",
        );

        for group in &route.groups {
            add_resolved_or_missing_ref(
                &mut graph,
                route_node.clone(),
                "groups",
                SceneReferenceKind::UsesLightGroup,
                SceneReferenceTargetKind::LightGroup,
                group,
                &light_groups,
                "missing_light_route_group",
            );
        }
    }

    let mut duplicate_objects = BTreeSet::<String>::new();

    for (index, entity) in document.entities.iter().enumerate() {
        if scene_objects.contains_key(entity.id.as_str()) {
            duplicate_objects.insert(entity.id.clone());
        }

        let console_path = entity_console_path(entity, schemas, graph_providers);
        let object_node = graph.add_child(
            &objects_parent,
            SceneGraphNode::new(
                node_id("scene_object", entity.id.as_str()),
                if entity.name.trim().is_empty() {
                    entity.id.clone()
                } else {
                    entity.name.clone()
                },
                SceneGraphNodeKind::SceneObject,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/entities/{index}"),
            ))
            .with_semantics(
                SceneGraphSemantics::new(SceneGraphSemanticRole::SceneObject2D)
                    .with_traits([
                        MetadataTraitKind::SceneObject2D,
                        MetadataTraitKind::PostFxHost2D,
                        MetadataTraitKind::HasIdentity,
                        MetadataTraitKind::HasVisibility,
                        MetadataTraitKind::HasComponents,
                        MetadataTraitKind::Transformable2D,
                        MetadataTraitKind::Selectable,
                        MetadataTraitKind::RuntimeControllable,
                        MetadataTraitKind::Patchable,
                    ])
                    .with_console_path(console_path.clone())
                    .with_post_fx_host(format!("scene_object:{}", entity.id), "SceneObjectPixels"),
            ),
        );

        scene_objects.insert(entity.id.clone(), object_node.clone());

        let components_parent = graph.add_child(
            &object_node,
            SceneGraphNode::new(
                node_id("components", entity.id.as_str()),
                "Components",
                SceneGraphNodeKind::Components,
            )
            .with_source(AuthoringSource::new(
                source_file.clone(),
                format!("/entities/{index}/components"),
            )),
        );

        for (component_index, component) in entity.components.iter().enumerate() {
            let component_traits = component_2d_traits(component);
            let component_node = graph.add_child(
                &components_parent,
                SceneGraphNode::new(
                    node_id(
                        "component",
                        format!("{}:{}:{}", entity.id, component_index, component.kind()).as_str(),
                    ),
                    component.kind(),
                    SceneGraphNodeKind::Component,
                )
                .with_source(AuthoringSource::new(
                    source_file.clone(),
                    format!("/entities/{index}/components/{component_index}"),
                ))
                .with_semantics(
                    SceneGraphSemantics::new(SceneGraphSemanticRole::Component2D)
                        .with_traits(component_traits)
                        .with_console_path(format!(
                            "{}.{}",
                            console_path,
                            console_component_name(component.kind())
                        ))
                        .with_post_fx_host(
                            format!(
                                "component:{}:{}:{}",
                                entity.id,
                                component_index,
                                component.kind()
                            ),
                            "SceneObjectPixels",
                        ),
                ),
            );

            add_component_references(
                &mut graph,
                component_node,
                component,
                &draw_layers,
                &scene_objects,
                schemas,
                graph_providers,
            );
        }
    }

    for duplicate in duplicate_objects {
        graph.add_diagnostic(SceneGraphDiagnostic::error(
            "duplicate_scene_object_id",
            format!("duplicate scene object id `{duplicate}`"),
            None,
        ));
    }

    graph
}

fn entity_console_path(
    entity: &SceneEntityDocument,
    schemas: Option<&ComponentSchemaRegistry>,
    graph_providers: Option<&ComponentGraphProviderRegistry>,
) -> String {
    if let Some(layer) = primary_render_layer(entity, schemas, graph_providers) {
        if layer.contains('.') {
            return format!("world.{layer}");
        }
    }
    if let Some(suffix) = entity.id.strip_prefix("beacon-") {
        return format!("world.lighting.beacon.{}", sanitize_console_segment(suffix));
    }
    format!(
        "world.{}",
        sanitize_console_path(entity.display_name().as_str())
    )
}

fn primary_render_layer(
    entity: &SceneEntityDocument,
    schemas: Option<&ComponentSchemaRegistry>,
    graph_providers: Option<&ComponentGraphProviderRegistry>,
) -> Option<String> {
    entity
        .components
        .iter()
        .find_map(|component| match component {
            _ if component.primary_render_layer().is_some() => {
                component.primary_render_layer().map(str::to_owned)
            }
            _ if component.plugin_payload().is_some() => {
                let Some((component_type, payload)) = component.plugin_payload() else {
                    return None;
                };
                let (Some(schemas), Some(graph_providers)) = (schemas, graph_providers) else {
                    return None;
                };
                let Some(payload) = schemas.parse_typed_plugin_payload(component_type, payload)
                else {
                    return None;
                };
                let Ok(payload) = payload else {
                    return None;
                };
                graph_providers
                    .with_provider(payload.component_type(), |provider| {
                        provider.primary_render_layer(payload.as_ref())
                    })
                    .flatten()
            }
            _ => None,
        })
}

fn console_component_name(kind: &str) -> &str {
    match kind {
        "BeaconLight2D" => "Beacon2D",
        other => other,
    }
}

fn sanitize_console_path(raw: &str) -> String {
    raw.split('.')
        .map(sanitize_console_segment)
        .collect::<Vec<_>>()
        .join(".")
}

fn sanitize_console_segment(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn add_component_references(
    graph: &mut SemanticSceneGraph,
    component_node: SceneGraphNodeId,
    component: &SceneComponentDocument,
    draw_layers: &BTreeMap<String, SceneGraphNodeId>,
    scene_objects: &BTreeMap<String, SceneGraphNodeId>,
    schemas: Option<&ComponentSchemaRegistry>,
    graph_providers: Option<&ComponentGraphProviderRegistry>,
) {
    match component {
        ComponentDocument::CameraFollow2d { target, .. } => {
            add_scene_object_ref(
                graph,
                component_node,
                "target",
                SceneReferenceKind::FollowsSceneObject,
                target,
                scene_objects,
                "missing_camera_follow_target",
            );
        }
        ComponentDocument::Parallax2d { camera, .. } => {
            add_scene_object_ref(
                graph,
                component_node,
                "camera",
                SceneReferenceKind::UsesCameraObject,
                camera,
                scene_objects,
                "missing_parallax_camera",
            );
        }
        ComponentDocument::TileMapMarker2d { tilemap_entity, .. } => {
            if let Some(tilemap_entity) = tilemap_entity {
                add_scene_object_ref(
                    graph,
                    component_node,
                    "tilemap_entity",
                    SceneReferenceKind::UsesTileMapObject,
                    tilemap_entity,
                    scene_objects,
                    "missing_tilemap_marker_target",
                );
            }
        }
        ComponentDocument::LightMap2dSource {
            source, channels, ..
        } => {
            match source {
                LightMap2dSourceRefDocument::LayeredImage2d { entity } => {
                    add_scene_object_ref(
                        graph,
                        component_node.clone(),
                        "source.entity",
                        SceneReferenceKind::LightMapSourceObject,
                        entity,
                        scene_objects,
                        "missing_lightmap_source_entity",
                    );
                }
            }

            for channel in channels {
                for layer in &channel.layers {
                    add_external_ref(
                        graph,
                        component_node.clone(),
                        "channels.layers",
                        SceneReferenceKind::UsesImagePart,
                        SceneReferenceTargetKind::ImagePart,
                        layer,
                        false,
                    );
                }
            }
        }
        ComponentDocument::ScriptComponent { script, .. } => {
            add_external_ref(
                graph,
                component_node,
                "script",
                SceneReferenceKind::UsesScript,
                SceneReferenceTargetKind::Script,
                script,
                true,
            );
        }
        ComponentDocument::Mesh3d { mesh } => {
            add_external_ref(
                graph,
                component_node,
                "mesh",
                SceneReferenceKind::UsesMesh,
                SceneReferenceTargetKind::Mesh,
                mesh,
                true,
            );
        }
        ComponentDocument::Material3d { source, albedo, .. } => {
            if let Some(source) = source {
                add_external_ref(
                    graph,
                    component_node.clone(),
                    "source",
                    SceneReferenceKind::UsesMaterial,
                    SceneReferenceTargetKind::Material,
                    source,
                    false,
                );
            }
            if let Some(albedo) = albedo {
                add_external_ref(
                    graph,
                    component_node,
                    "albedo",
                    SceneReferenceKind::UsesAsset,
                    SceneReferenceTargetKind::Asset,
                    albedo,
                    false,
                );
            }
        }
        ComponentDocument::Text3d { font, .. } => {
            add_external_ref(
                graph,
                component_node,
                "font",
                SceneReferenceKind::UsesFont,
                SceneReferenceTargetKind::Font,
                font,
                true,
            );
        }
        _ if component.plugin_payload().is_some() => {
            let Some((component_type, payload)) = component.plugin_payload() else {
                return;
            };
            let (Some(schemas), Some(graph_providers)) = (schemas, graph_providers) else {
                return;
            };
            let Some(payload) = schemas.parse_typed_plugin_payload(component_type, payload) else {
                return;
            };
            let Ok(payload) = payload else {
                return;
            };
            let _ = graph_providers.with_provider(payload.component_type(), |provider| {
                let mut ctx = crate::PluginComponentGraphContext {
                    payload: payload.as_ref(),
                    component_node,
                    graph,
                    draw_layers,
                    scene_objects,
                };
                provider.add_references(&mut ctx);
            });
        }
        _ => {}
    }
}

fn add_scene_object_ref(
    graph: &mut SemanticSceneGraph,
    from: SceneGraphNodeId,
    port: &str,
    kind: SceneReferenceKind,
    raw_target: &str,
    scene_objects: &BTreeMap<String, SceneGraphNodeId>,
    missing_code: &str,
) {
    add_resolved_or_missing_ref(
        graph,
        from,
        port,
        kind,
        SceneReferenceTargetKind::SceneObject,
        raw_target,
        scene_objects,
        missing_code,
    );
}

fn add_resolved_or_missing_ref(
    graph: &mut SemanticSceneGraph,
    from: SceneGraphNodeId,
    port: &str,
    kind: SceneReferenceKind,
    target_kind: SceneReferenceTargetKind,
    raw_target: &str,
    index: &BTreeMap<String, SceneGraphNodeId>,
    missing_code: &str,
) {
    let resolved = index.get(raw_target).cloned();

    graph.add_reference(SceneReferenceEdge::new(
        from.clone(),
        port,
        kind,
        target_kind,
        raw_target,
        true,
        resolved.clone(),
    ));

    if resolved.is_none() {
        graph.add_diagnostic(SceneGraphDiagnostic::error(
            missing_code,
            format!(
                "missing {:?} reference `{}` at `{}`",
                target_kind, raw_target, port
            ),
            Some(from),
        ));
    }
}

fn add_external_ref(
    graph: &mut SemanticSceneGraph,
    from: SceneGraphNodeId,
    port: &str,
    kind: SceneReferenceKind,
    target_kind: SceneReferenceTargetKind,
    raw_target: &str,
    required: bool,
) {
    graph.add_reference(SceneReferenceEdge::new(
        from,
        port,
        kind,
        target_kind,
        raw_target,
        required,
        None,
    ));
}

fn node_id(prefix: &str, raw: &str) -> String {
    let sanitized = raw
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '#' | ' ' => '_',
            other => other,
        })
        .collect::<String>();

    format!("{prefix}:{sanitized}")
}
fn _assert_postfx_document_helpers(effect: &PostFx2dDocument) -> (&str, &'static str) {
    (effect.id(), effect.type_name())
}
