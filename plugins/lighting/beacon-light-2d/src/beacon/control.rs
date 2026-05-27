use std::sync::Arc;

use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError, RuntimeControlProperty,
    RuntimeControlProvider, RuntimeControlRegistry, RuntimeControlTarget,
};

use crate::BeaconLight2dSceneService;

pub struct Beacon2dControlProvider {
    service: Arc<BeaconLight2dSceneService>,
}

impl Beacon2dControlProvider {
    pub fn new(service: Arc<BeaconLight2dSceneService>) -> Self {
        Self { service }
    }
}

impl RuntimeControlProvider for Beacon2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "beacon_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for command in self.service.commands() {
            let suffix = command
                .id
                .strip_prefix("beacon-")
                .unwrap_or(command.id.as_str());
            let target_path = format!("world.lighting.beacon.{}", suffix.replace('-', "_"));
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(command.id.clone()),
                label: command.entity_name.clone(),
                components: vec!["Beacon2D".to_owned()],
                aliases: vec![format!("world.{}", command.entity_name.replace('-', "_"))],
                source_file: None,
            });
            for (property_path, value_type) in [
                ("base_intensity", ControlValueType::F32),
                ("glow_strength", ControlValueType::F32),
                ("frequency_hz", ControlValueType::F32),
                ("beam_enabled", ControlValueType::Bool),
            ] {
                registry.register_property(RuntimeControlProperty {
                    console_path: format!("{target_path}.Beacon2D.{property_path}"),
                    target_path: target_path.clone(),
                    component: Some("Beacon2D".to_owned()),
                    property_path: property_path.to_owned(),
                    value_type,
                    range: Some(ControlRange {
                        min: Some(0.0),
                        max: None,
                    }),
                    writable: true,
                    readable: true,
                    animatable: property_path != "beam_enabled",
                    source_file: None,
                    source_pointer: None,
                    provider_id: "beacon_2d".to_owned(),
                    description: None,
                });
            }
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let id = path
            .target_path
            .trim_start_matches("world.lighting.beacon.")
            .replace('_', "-");
        let beacon_id = format!("beacon-{id}");
        let command = self
            .service
            .commands()
            .into_iter()
            .find(|command| command.id == beacon_id)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        match path.property_path.as_str() {
            "base_intensity" => Ok(ControlValue::F64(command.base_intensity as f64)),
            "glow_strength" => Ok(ControlValue::F64(command.glow_strength as f64)),
            "frequency_hz" => Ok(ControlValue::F64(command.frequency_hz as f64)),
            "beam_enabled" => Ok(ControlValue::Bool(command.beam_enabled)),
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
        let id = path
            .target_path
            .trim_start_matches("world.lighting.beacon.")
            .replace('_', "-");
        let beacon_id = format!("beacon-{id}");
        let updated = match path.property_path.as_str() {
            "base_intensity" => self.service.set_base_intensity(
                &beacon_id,
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
                    })?,
            ),
            "glow_strength" => self.service.set_glow_strength(
                &beacon_id,
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
                    })?,
            ),
            "frequency_hz" => self.service.set_frequency_hz(
                &beacon_id,
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
                    })?,
            ),
            "beam_enabled" => self.service.set_beam_enabled(
                &beacon_id,
                value
                    .as_bool()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
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
