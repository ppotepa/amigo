use std::sync::Arc;

use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError,
    RuntimeControlProperty, RuntimeControlProvider, RuntimeControlRegistry,
    RuntimeControlTarget,
};

pub struct ParticleEmitter2dControlProvider {
    service: Arc<Particle2dSceneService>,
}

impl ParticleEmitter2dControlProvider {
    pub fn new(service: Arc<Particle2dSceneService>) -> Self {
        Self { service }
    }
}

impl RuntimeControlProvider for ParticleEmitter2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "particles_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for emitter in self.service.emitters() {
            let target_path = particle_target_path(&emitter.entity_name, &emitter.emitter.render_layer);
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(emitter.entity_name.clone()),
                label: emitter.entity_name.clone(),
                components: vec!["ParticleEmitter2D".to_owned()],
                aliases: vec![format!("world.{}", sanitize_entity_target(&emitter.entity_name))],
                source_file: None,
            });
            register_particle_property(
                registry,
                &target_path,
                "spawn_rate",
                ControlValueType::F32,
                Some(ControlRange {
                    min: Some(0.0),
                    max: None,
                }),
            );
            register_particle_property(
                registry,
                &target_path,
                "max_particles",
                ControlValueType::U64,
                Some(ControlRange {
                    min: Some(0.0),
                    max: None,
                }),
            );
            register_particle_property(
                registry,
                &target_path,
                "initial_speed",
                ControlValueType::F32,
                Some(ControlRange {
                    min: Some(0.0),
                    max: None,
                }),
            );
            register_particle_property(registry, &target_path, "active", ControlValueType::Bool, None);
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let entity_name = self
            .service
            .emitters()
            .into_iter()
            .find(|emitter| particle_target_path(&emitter.entity_name, &emitter.emitter.render_layer) == path.target_path)
            .map(|emitter| emitter.entity_name)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        let command = self
            .service
            .emitter(entity_name.as_str())
            .ok_or_else(|| RuntimeControlError::UnknownTarget(entity_name.clone()))?;
        match path.property_path.as_str() {
            "spawn_rate" => Ok(ControlValue::F64(command.emitter.spawn_rate as f64)),
            "max_particles" => Ok(ControlValue::U64(command.emitter.max_particles as u64)),
            "initial_speed" => Ok(ControlValue::F64(command.emitter.initial_speed as f64)),
            "active" => Ok(ControlValue::Bool(self.service.is_active(entity_name.as_str()))),
            _ => Err(RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            }),
        }
    }

    fn set(
        &self,
        path: &RuntimeControlProperty,
        value: ControlValue,
    ) -> Result<(), RuntimeControlError> {
        let entity_name = self
            .service
            .emitters()
            .into_iter()
            .find(|emitter| particle_target_path(&emitter.entity_name, &emitter.emitter.render_layer) == path.target_path)
            .map(|emitter| emitter.entity_name)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        let updated = match path.property_path.as_str() {
            "spawn_rate" => self.service.set_spawn_rate(
                entity_name.as_str(),
                value.as_f32().ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: path.console_path.clone(),
                    expected: "f32".to_owned(),
                    actual: "non-number".to_owned(),
                })?,
            ),
            "max_particles" => self.service.set_max_particles(
                entity_name.as_str(),
                value.as_f64().ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: path.console_path.clone(),
                    expected: "u64".to_owned(),
                    actual: "non-number".to_owned(),
                })? as usize,
            ),
            "initial_speed" => self.service.set_initial_speed(
                entity_name.as_str(),
                value.as_f32().ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: path.console_path.clone(),
                    expected: "f32".to_owned(),
                    actual: "non-number".to_owned(),
                })?,
            ),
            "active" => self.service.set_active(
                entity_name.as_str(),
                value.as_bool().ok_or_else(|| RuntimeControlError::TypeMismatch {
                    path: path.console_path.clone(),
                    expected: "bool".to_owned(),
                    actual: "non-bool".to_owned(),
                })?,
            ),
            _ => false,
        };
        if updated {
            Ok(())
        } else {
            Err(RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            })
        }
    }
}

fn register_particle_property(
    registry: &mut RuntimeControlRegistry,
    target_path: &str,
    property_path: &str,
    value_type: ControlValueType,
    range: Option<ControlRange>,
) {
    registry.register_property(RuntimeControlProperty {
        console_path: format!("{target_path}.ParticleEmitter2D.{property_path}"),
        target_path: target_path.to_owned(),
        component: Some("ParticleEmitter2D".to_owned()),
        property_path: property_path.to_owned(),
        value_type,
        range,
        writable: true,
        readable: true,
        animatable: true,
        source_file: None,
        source_pointer: None,
        provider_id: "particles_2d".to_owned(),
        description: None,
    });
}

fn particle_target_path(entity_name: &str, render_layer: &str) -> String {
    let target = if render_layer.contains('.') {
        render_layer.to_owned()
    } else {
        sanitize_entity_target(entity_name)
    };
    format!("world.{target}")
}

fn sanitize_entity_target(entity_name: &str) -> String {
    entity_name.replace('-', "_")
}
