use serde_yaml::Value;

use crate::{AuthoringNode, AuthoringNodeKind, AuthoringRuntimeBinding};

pub fn slider_hint_for_property(path: &str, value: f32) -> (f32, f32, f32) {
    if path.contains("opacity") || path.contains("strength") || path.contains("vignette") {
        (0.0, 1.0_f32.max(value), 0.01)
    } else if path.contains("order") || path.contains("z_index") {
        (-1000.0, 1000.0, 1.0)
    } else if path.contains("px") || path.contains("radius") {
        (0.0, 100.0_f32.max(value), 0.01)
    } else {
        (0.0, 10.0_f32.max(value), 0.01)
    }
}

pub fn runtime_binding_hint(
    node: &AuthoringNode,
    property_path: &str,
    value: &Value,
) -> Option<AuthoringRuntimeBinding> {
    match node.kind {
        AuthoringNodeKind::RenderLayer => {
            let layer_id = node.semantic.render_layer_id.clone().or_else(|| {
                node.value
                    .as_mapping()?
                    .get(Value::String("id".to_owned()))?
                    .as_str()
                    .map(str::to_owned)
            })?;
            if property_path == "opacity" {
                Some(AuthoringRuntimeBinding::RenderLayerOpacity { layer_id })
            } else if property_path == "visible" {
                Some(AuthoringRuntimeBinding::RenderLayerVisible { layer_id })
            } else if property_path == "order" {
                Some(AuthoringRuntimeBinding::RenderLayerOrder { layer_id })
            } else {
                None
            }
        }
        AuthoringNodeKind::Component => {
            if node.semantic.component_type.as_deref() == Some("LayeredImage2D") {
                let entity_name = node.semantic.owner_entity_name.clone()?;
                if property_path == "base_opacity" {
                    return Some(AuthoringRuntimeBinding::LayeredImageBaseOpacity { entity_name });
                }
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
            }
            if node.semantic.component_type.as_deref() == Some("ParticleEmitter2D") {
                let entity_name = node.semantic.owner_entity_name.clone()?;
                return match value {
                    Value::Bool(_) | Value::Number(_) => {
                        Some(AuthoringRuntimeBinding::ParticleEmitterProperty {
                            entity_name,
                            field: property_path.to_owned(),
                        })
                    }
                    _ => None,
                };
            }
            None
        }
        AuthoringNodeKind::PostFxItem => {
            let effect_id = node
                .semantic
                .post_fx_id
                .clone()
                .unwrap_or_else(|| "effect".to_owned());
            match value {
                Value::Bool(_) | Value::Number(_) => Some(AuthoringRuntimeBinding::PostFxMock {
                    effect_id,
                    field: property_path.to_owned(),
                }),
                _ => None,
            }
        }
        _ => None,
    }
}
