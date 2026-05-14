use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_authoring::AuthoringRuntimeBinding;
use amigo_runtime::Runtime;

use crate::state::{EditorPropertyValue, IngameEditorState};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ApplyResult {
    Applied,
    MockApplied,
    Readonly,
    Unsupported,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ApplyRequest<'a> {
    pub property_id: &'a str,
    pub target: Option<&'a AuthoringRuntimeBinding>,
    pub previous: Option<EditorPropertyValue>,
    pub next: EditorPropertyValue,
}

pub fn apply_property_value(
    runtime: &Runtime,
    state: &IngameEditorState,
    property_id: &str,
    target: Option<&AuthoringRuntimeBinding>,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    apply_property_request(
        runtime,
        state,
        ApplyRequest {
            property_id,
            target,
            previous: None,
            next: value,
        },
    )
}

pub fn apply_property_request(
    runtime: &Runtime,
    state: &IngameEditorState,
    request: ApplyRequest<'_>,
) -> AmigoResult<ApplyResult> {
    let Some(target) = request.target else {
        state.set_status(format!("{}: unsupported", request.property_id));
        return Ok(ApplyResult::Unsupported);
    };

    let previous_label = request
        .previous
        .as_ref()
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "<unknown>".to_owned());
    let next_label = format!("{:?}", request.next);
    let result = match target {
        AuthoringRuntimeBinding::RenderLayerOpacity { layer_id }
        | AuthoringRuntimeBinding::RenderLayerVisible { layer_id }
        | AuthoringRuntimeBinding::RenderLayerOrder { layer_id } => {
            apply_render_layer_binding(runtime, target, layer_id, request.next)
        }
        AuthoringRuntimeBinding::LayeredImageBaseOpacity { .. }
        | AuthoringRuntimeBinding::LayeredImageLayerOpacity { .. }
        | AuthoringRuntimeBinding::LayeredImageLayerEnabled { .. } => {
            apply_layered_image_binding(runtime, target, request.next)
        }
        AuthoringRuntimeBinding::ParticleEmitterProperty { entity_name, field } => {
            apply_particle_property(runtime, entity_name, field, request.next)
        }
        AuthoringRuntimeBinding::PostFxMock { .. } | AuthoringRuntimeBinding::Mock { .. } => {
            apply_mock_binding(state, request.property_id, request.next)
        }
    };

    match &result {
        Ok(ApplyResult::Applied) => state.set_status(format!(
            "{}: {previous_label} -> {next_label} [Applied Live]",
            request.property_id
        )),
        Ok(ApplyResult::Unsupported) => {
            state.set_status(format!("{}: unsupported", request.property_id))
        }
        Ok(ApplyResult::Readonly) => state.set_status(format!("{}: readonly", request.property_id)),
        Ok(ApplyResult::MockApplied) | Err(_) | Ok(ApplyResult::Failed(_)) => {}
    }
    result
}

fn apply_render_layer_binding(
    runtime: &Runtime,
    target: &AuthoringRuntimeBinding,
    layer_id: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
    let applied = match (target, value) {
        (
            AuthoringRuntimeBinding::RenderLayerOpacity { .. },
            EditorPropertyValue::Number(value),
        ) => service.set_opacity(layer_id, value),
        (AuthoringRuntimeBinding::RenderLayerVisible { .. }, EditorPropertyValue::Bool(value)) => {
            service.set_visible(layer_id, value)
        }
        (AuthoringRuntimeBinding::RenderLayerOrder { .. }, EditorPropertyValue::Number(value)) => {
            service.set_order(layer_id, value)
        }
        _ => return Ok(ApplyResult::Unsupported),
    };
    if applied {
        Ok(ApplyResult::Applied)
    } else {
        Err(AmigoError::Message(format!(
            "unknown render layer `{layer_id}`"
        )))
    }
}

fn apply_layered_image_binding(
    runtime: &Runtime,
    target: &AuthoringRuntimeBinding,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_layered_image::LayeredImageSceneService>()?;
    let applied = match (target, value) {
        (
            AuthoringRuntimeBinding::LayeredImageBaseOpacity { entity_name },
            EditorPropertyValue::Number(value),
        ) => service.set_base_opacity(entity_name, value),
        (
            AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Number(value),
        ) => service.set_layer_opacity(entity_name, layer_id, value),
        (
            AuthoringRuntimeBinding::LayeredImageLayerEnabled {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Bool(value),
        ) => service.set_layer_enabled(entity_name, layer_id, value),
        _ => return Ok(ApplyResult::Unsupported),
    };
    if applied {
        Ok(ApplyResult::Applied)
    } else {
        Err(AmigoError::Message(
            "unknown layered image binding target".to_owned(),
        ))
    }
}

fn apply_mock_binding(
    state: &IngameEditorState,
    property_id: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    state.set_override(property_id.to_owned(), value);
    state.set_status(format!("{property_id}: mock override [MockApplied]"));
    Ok(ApplyResult::MockApplied)
}

fn apply_particle_property(
    runtime: &Runtime,
    entity_name: &str,
    field: &str,
    value: EditorPropertyValue,
) -> AmigoResult<ApplyResult> {
    let service = runtime.required::<amigo_2d_particles::Particle2dSceneService>()?;
    let attempted = apply_particle_property_to_service(&service, entity_name, field, value);
    match attempted {
        Some(true) => Ok(ApplyResult::Applied),
        Some(false) => Err(AmigoError::Message(format!(
            "unknown particle emitter `{entity_name}` or invalid particle field `{field}`"
        ))),
        None => Ok(ApplyResult::Unsupported),
    }
}

fn apply_particle_property_to_service(
    service: &amigo_2d_particles::Particle2dSceneService,
    entity_name: &str,
    field: &str,
    value: EditorPropertyValue,
) -> Option<bool> {
    match (field, value) {
        ("active", EditorPropertyValue::Bool(value)) => {
            Some(service.set_active(entity_name, value))
        }
        ("spawn_rate", EditorPropertyValue::Number(value)) => {
            Some(service.set_spawn_rate(entity_name, value))
        }
        ("max_particles", EditorPropertyValue::Number(value)) => {
            Some(service.set_max_particles(entity_name, value.round().max(0.0) as usize))
        }
        ("particle_lifetime", EditorPropertyValue::Number(value)) => {
            Some(service.set_particle_lifetime(entity_name, value))
        }
        ("lifetime_jitter", EditorPropertyValue::Number(value)) => {
            Some(service.set_lifetime_jitter(entity_name, value))
        }
        ("initial_speed", EditorPropertyValue::Number(value)) => {
            Some(service.set_initial_speed(entity_name, value))
        }
        ("speed_jitter", EditorPropertyValue::Number(value)) => {
            Some(service.set_speed_jitter(entity_name, value))
        }
        ("spread_degrees", EditorPropertyValue::Number(value)) => {
            Some(service.set_spread_radians(entity_name, value.to_radians()))
        }
        ("local_direction_degrees", EditorPropertyValue::Number(value)) => {
            Some(service.set_local_direction_radians(entity_name, value.to_radians()))
        }
        ("inherit_parent_velocity", EditorPropertyValue::Number(value)) => {
            Some(service.set_inherit_parent_velocity(entity_name, value))
        }
        ("initial_size", EditorPropertyValue::Number(value)) => {
            Some(service.set_initial_size(entity_name, value))
        }
        ("final_size", EditorPropertyValue::Number(value)) => {
            Some(service.set_final_size(entity_name, value))
        }
        ("z_index", EditorPropertyValue::Number(value)) => {
            Some(service.set_z_index(entity_name, value))
        }
        ("intensity", EditorPropertyValue::Number(value)) => {
            Some(service.set_intensity(entity_name, value))
        }
        ("quality_scale", EditorPropertyValue::Number(value)) => {
            Some(service.set_quality_scale(entity_name, value))
        }
        _ => None,
    }
}
