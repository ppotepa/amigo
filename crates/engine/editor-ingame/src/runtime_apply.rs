use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_authoring::AuthoringRuntimeBinding;
use amigo_runtime::Runtime;

use crate::state::{EditorPropertyValue, IngameEditorState};

pub fn apply_property_value(
    runtime: &Runtime,
    state: &IngameEditorState,
    property_id: &str,
    target: Option<&AuthoringRuntimeBinding>,
    value: EditorPropertyValue,
) -> AmigoResult<()> {
    state.set_override(property_id.to_owned(), value.clone());

    let Some(target) = target else {
        state.set_status(format!("{property_id}: mock override"));
        return Ok(());
    };

    let result = match (target, value) {
        (
            AuthoringRuntimeBinding::RenderLayerOpacity { layer_id },
            EditorPropertyValue::Number(value),
        ) => {
            let service = runtime.required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
            if service.set_opacity(layer_id, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown render layer `{layer_id}`"
                )))
            }
        }
        (
            AuthoringRuntimeBinding::RenderLayerVisible { layer_id },
            EditorPropertyValue::Bool(value),
        ) => {
            let service = runtime.required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
            if service.set_visible(layer_id, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown render layer `{layer_id}`"
                )))
            }
        }
        (
            AuthoringRuntimeBinding::LayeredImageBaseOpacity { entity_name },
            EditorPropertyValue::Number(value),
        ) => {
            let service = runtime.required::<amigo_2d_layered_image::LayeredImageSceneService>()?;
            if service.set_base_opacity(entity_name, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown layered image `{entity_name}`"
                )))
            }
        }
        (
            AuthoringRuntimeBinding::LayeredImageLayerOpacity {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Number(value),
        ) => {
            let service = runtime.required::<amigo_2d_layered_image::LayeredImageSceneService>()?;
            if service.set_layer_opacity(entity_name, layer_id, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown layered image `{entity_name}` layer `{layer_id}`"
                )))
            }
        }
        (
            AuthoringRuntimeBinding::LayeredImageLayerEnabled {
                entity_name,
                layer_id,
            },
            EditorPropertyValue::Bool(value),
        ) => {
            let service = runtime.required::<amigo_2d_layered_image::LayeredImageSceneService>()?;
            if service.set_layer_enabled(entity_name, layer_id, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown layered image `{entity_name}` layer `{layer_id}`"
                )))
            }
        }
        (
            AuthoringRuntimeBinding::RenderLayerOrder { layer_id },
            EditorPropertyValue::Number(value),
        ) => {
            let service = runtime.required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
            if service.set_order(layer_id, value) {
                Ok(())
            } else {
                Err(AmigoError::Message(format!(
                    "unknown render layer `{layer_id}`"
                )))
            }
        }
        (AuthoringRuntimeBinding::PostFxMock { .. }, _) => {
            state.set_status(format!("{property_id}: mock override"));
            Ok(())
        }
        (AuthoringRuntimeBinding::ParticleEmitterProperty { entity_name, field }, value) => {
            apply_particle_property(runtime, entity_name, field, value)
        }
        (AuthoringRuntimeBinding::Mock { .. }, _) => {
            state.set_status(format!("{property_id}: mock override"));
            Ok(())
        }
        _ => Ok(()),
    };

    if result.is_ok() {
        state.set_status(format!("runtime applied: {property_id}"));
    }
    result
}

fn apply_particle_property(
    runtime: &Runtime,
    entity_name: &str,
    field: &str,
    value: EditorPropertyValue,
) -> AmigoResult<()> {
    let service = runtime.required::<amigo_2d_particles::Particle2dSceneService>()?;
    let attempted = apply_particle_property_to_service(&service, entity_name, field, value);
    match attempted {
        Some(true) => Ok(()),
        Some(false) => Err(AmigoError::Message(format!(
            "unknown particle emitter `{entity_name}` or invalid particle field `{field}`"
        ))),
        None => Ok(()),
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
