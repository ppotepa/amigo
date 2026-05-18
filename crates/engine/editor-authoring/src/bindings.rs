use amigo_editor_api::AuthoringRuntimeBinding;
use amigo_scene::{EditorPropertyDescriptor, EditorRuntimeBindingTemplate};
use serde_yaml::Value;

use crate::{AuthoringNode, AuthoringNodeKind};

pub struct AuthoringBindingResolver;

pub struct BindingResolveContext<'a> {
    pub node: &'a AuthoringNode,
    pub descriptor: &'a EditorPropertyDescriptor,
    pub yaml_value: Option<&'a Value>,
}

pub fn resolve_property_binding(
    node: &AuthoringNode,
    descriptor: &EditorPropertyDescriptor,
    yaml_value: Option<&Value>,
) -> Option<AuthoringRuntimeBinding> {
    AuthoringBindingResolver::resolve(BindingResolveContext {
        node,
        descriptor,
        yaml_value,
    })
}

impl AuthoringBindingResolver {
    pub fn resolve(ctx: BindingResolveContext<'_>) -> Option<AuthoringRuntimeBinding> {
        match ctx.descriptor.binding_template {
            Some(EditorRuntimeBindingTemplate::RenderLayerField) => {
                resolve_draw_layer_field(ctx.node, ctx.descriptor.path)
            }
            Some(EditorRuntimeBindingTemplate::ComponentRuntimeField) => {
                resolve_component_runtime_field(ctx.node, ctx.descriptor.path, ctx.yaml_value)
            }
            Some(EditorRuntimeBindingTemplate::LayeredImageBaseOpacity) => {
                resolve_layered_image_base(ctx.node, ctx.descriptor.path)
            }
            Some(EditorRuntimeBindingTemplate::LayeredImagePartField) => {
                resolve_image_part_field(ctx.node, ctx.descriptor.path)
            }
            Some(EditorRuntimeBindingTemplate::ParticleEmitterField) => {
                resolve_particle_field(ctx.node, ctx.descriptor.path, ctx.yaml_value)
            }
            Some(EditorRuntimeBindingTemplate::MockOnly)
            | Some(EditorRuntimeBindingTemplate::SceneCommandPatch)
            | Some(EditorRuntimeBindingTemplate::None)
            | None => None,
        }
    }
}

pub fn resolve_component_runtime_field(
    node: &AuthoringNode,
    property_path: &str,
    yaml_value: Option<&Value>,
) -> Option<AuthoringRuntimeBinding> {
    resolve_layered_image_base(node, property_path)
        .or_else(|| resolve_particle_field(node, property_path, yaml_value))
}

pub fn resolve_draw_layer_field(
    node: &AuthoringNode,
    property_path: &str,
) -> Option<AuthoringRuntimeBinding> {
    if !matches!(node.kind, AuthoringNodeKind::RenderLayer) {
        return None;
    }
    let layer_id = node.semantic.render_layer_id.clone().or_else(|| {
        node.value
            .as_mapping()?
            .get(Value::String("id".to_owned()))?
            .as_str()
            .map(str::to_owned)
    })?;
    match property_path {
        "opacity" => Some(AuthoringRuntimeBinding::RenderLayerOpacity { layer_id }),
        "visible" => Some(AuthoringRuntimeBinding::RenderLayerVisible { layer_id }),
        "order" => Some(AuthoringRuntimeBinding::RenderLayerOrder { layer_id }),
        "depth.mode" => Some(AuthoringRuntimeBinding::RenderLayerDepthMode { layer_id }),
        "depth.distance_m" => Some(AuthoringRuntimeBinding::RenderLayerDistanceM { layer_id }),
        "depth.z_depth" => Some(AuthoringRuntimeBinding::RenderLayerZDepth { layer_id }),
        "depth.blur_scale" => Some(AuthoringRuntimeBinding::RenderLayerDepthBlurScale { layer_id }),
        _ => None,
    }
}

pub fn resolve_layered_image_base(
    node: &AuthoringNode,
    property_path: &str,
) -> Option<AuthoringRuntimeBinding> {
    if node.semantic.component_type.as_deref() != Some("LayeredImage2D")
        || property_path != "base_opacity"
    {
        return None;
    }
    Some(AuthoringRuntimeBinding::LayeredImageBaseOpacity {
        entity_name: node.semantic.owner_entity_name.clone()?,
    })
}

pub fn resolve_image_part_field(
    node: &AuthoringNode,
    property_path: &str,
) -> Option<AuthoringRuntimeBinding> {
    if node.semantic.component_type.as_deref() != Some("LayeredImage2D") {
        return None;
    }
    let entity_name = node.semantic.owner_entity_name.clone()?;
    if let Some(layer_id) = property_path
        .strip_prefix("layer_overrides.")
        .and_then(|path| path.strip_suffix(".opacity"))
        .map(str::to_owned)
    {
        return Some(AuthoringRuntimeBinding::LayeredImageLayerOpacity {
            entity_name,
            layer_id,
        });
    }
    if let Some(layer_id) = property_path
        .strip_prefix("layer_overrides.")
        .and_then(|path| path.strip_suffix(".enabled"))
        .map(str::to_owned)
    {
        return Some(AuthoringRuntimeBinding::LayeredImageLayerEnabled {
            entity_name,
            layer_id,
        });
    }
    None
}

pub fn resolve_particle_field(
    node: &AuthoringNode,
    property_path: &str,
    yaml_value: Option<&Value>,
) -> Option<AuthoringRuntimeBinding> {
    if node.semantic.component_type.as_deref() != Some("ParticleEmitter2D")
        || !is_live_particle_field(property_path)
    {
        return None;
    }
    match yaml_value {
        Some(Value::Bool(_)) | Some(Value::Number(_)) => {
            Some(AuthoringRuntimeBinding::ParticleEmitterProperty {
                entity_name: node.semantic.owner_entity_name.clone()?,
                field: property_path.to_owned(),
            })
        }
        _ => None,
    }
}

fn is_live_particle_field(property_path: &str) -> bool {
    matches!(
        property_path,
        "active"
            | "spawn_rate"
            | "max_particles"
            | "particle_lifetime"
            | "lifetime_jitter"
            | "initial_speed"
            | "speed_jitter"
            | "spread_degrees"
            | "local_direction_degrees"
            | "inherit_parent_velocity"
            | "initial_size"
            | "final_size"
            | "z_index"
            | "intensity"
            | "quality_scale"
    )
}
