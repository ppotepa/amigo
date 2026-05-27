use std::sync::Arc;

use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError, RuntimeControlProperty,
    RuntimeControlProvider, RuntimeControlRegistry, RuntimeControlTarget,
};

use crate::LightGroup2dSceneService;

pub struct LightGroup2dControlProvider {
    service: Arc<LightGroup2dSceneService>,
}

impl LightGroup2dControlProvider {
    pub fn new(service: Arc<LightGroup2dSceneService>) -> Self {
        Self { service }
    }
}

impl RuntimeControlProvider for LightGroup2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "light_group_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for group in self.service.commands() {
            let target_path = format!("world.light_group.{}", group.id.replace('-', "_"));
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(group.id.clone()),
                label: group.label.clone().unwrap_or_else(|| group.id.clone()),
                components: vec!["LightGroup2D".to_owned()],
                aliases: vec![format!("world.light_group.{}", group.id)],
                source_file: None,
            });
            registry.register_property(RuntimeControlProperty {
                console_path: format!("{target_path}.LightGroup2D.intensity"),
                target_path,
                component: Some("LightGroup2D".to_owned()),
                property_path: "intensity".to_owned(),
                value_type: ControlValueType::F32,
                range: Some(ControlRange {
                    min: Some(0.0),
                    max: Some(12.0),
                }),
                writable: true,
                readable: true,
                animatable: true,
                source_file: None,
                source_pointer: None,
                provider_id: "light_group_2d".to_owned(),
                description: None,
            });
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let group_id = group_id_from_target(&path.target_path);
        let group = self
            .service
            .commands()
            .into_iter()
            .find(|group| group.id == group_id)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        match path.property_path.as_str() {
            "intensity" => Ok(ControlValue::F64(group.intensity as f64)),
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
        let group_id = group_id_from_target(&path.target_path);
        let updated = match path.property_path.as_str() {
            "intensity" => self.service.set_intensity(
                group_id.as_str(),
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
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

fn group_id_from_target(target_path: &str) -> String {
    target_path
        .trim_start_matches("world.light_group.")
        .replace('_', "-")
}
