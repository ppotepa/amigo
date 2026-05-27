use amigo_core::{AmigoError, AmigoResult};
use amigo_editor_api::{
    AuthoringRuntimeBinding, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
};
use amigo_runtime::Runtime;

use crate::Particle2dSceneService;

pub struct Particle2dEditorRuntimeApplyProvider;

impl EditorRuntimeApplyProvider for Particle2dEditorRuntimeApplyProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.vfx.particles-2d"
    }

    fn can_apply(&self, request: &EditorRuntimeApplyRequest) -> bool {
        matches!(
            request,
            EditorRuntimeApplyRequest::RuntimeProperty {
                binding: AuthoringRuntimeBinding::ParticleEmitterProperty { .. },
                ..
            }
        )
    }

    fn apply(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> AmigoResult<EditorRuntimeApplyOutcome> {
        let EditorRuntimeApplyRequest::RuntimeProperty {
            binding: AuthoringRuntimeBinding::ParticleEmitterProperty { entity_name, field },
            value,
            ..
        } = request
        else {
            return Ok(EditorRuntimeApplyOutcome::Ignored);
        };

        let service = runtime.required::<Particle2dSceneService>()?;
        match apply_particle_property_to_service(&service, &entity_name, &field, value) {
            Some(true) => Ok(EditorRuntimeApplyOutcome::Applied(format!(
                "particle emitter `{entity_name}` updated"
            ))),
            Some(false) => Err(AmigoError::Message(format!(
                "unknown particle emitter `{entity_name}` or invalid particle field `{field}`"
            ))),
            None => Ok(EditorRuntimeApplyOutcome::Ignored),
        }
    }
}

fn apply_particle_property_to_service(
    service: &Particle2dSceneService,
    entity_name: &str,
    field: &str,
    value: serde_yaml::Value,
) -> Option<bool> {
    match (field, value) {
        ("active", serde_yaml::Value::Bool(value)) => Some(service.set_active(entity_name, value)),
        ("spawn_rate", value) => Some(service.set_spawn_rate(entity_name, number_value(value)?)),
        ("max_particles", value) => Some(
            service.set_max_particles(entity_name, number_value(value)?.round().max(0.0) as usize),
        ),
        ("particle_lifetime", value) => {
            Some(service.set_particle_lifetime(entity_name, number_value(value)?))
        }
        ("lifetime_jitter", value) => {
            Some(service.set_lifetime_jitter(entity_name, number_value(value)?))
        }
        ("initial_speed", value) => {
            Some(service.set_initial_speed(entity_name, number_value(value)?))
        }
        ("speed_jitter", value) => {
            Some(service.set_speed_jitter(entity_name, number_value(value)?))
        }
        ("spread_degrees", value) => {
            Some(service.set_spread_radians(entity_name, number_value(value)?.to_radians()))
        }
        ("local_direction_degrees", value) => Some(
            service.set_local_direction_radians(entity_name, number_value(value)?.to_radians()),
        ),
        ("inherit_parent_velocity", value) => {
            Some(service.set_inherit_parent_velocity(entity_name, number_value(value)?))
        }
        ("initial_size", value) => {
            Some(service.set_initial_size(entity_name, number_value(value)?))
        }
        ("final_size", value) => Some(service.set_final_size(entity_name, number_value(value)?)),
        ("z_index", value) => Some(service.set_z_index(entity_name, number_value(value)?)),
        ("intensity", value) => Some(service.set_intensity(entity_name, number_value(value)?)),
        ("quality_scale", value) => {
            Some(service.set_quality_scale(entity_name, number_value(value)?))
        }
        _ => None,
    }
}

fn number_value(value: serde_yaml::Value) -> Option<f32> {
    match value {
        serde_yaml::Value::Number(number) => number.as_f64().map(|value| value as f32),
        _ => None,
    }
}
