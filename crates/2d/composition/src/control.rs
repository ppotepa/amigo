use std::sync::Arc;

use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError, RuntimeControlProperty,
    RuntimeControlProvider, RuntimeControlRegistry, RuntimeControlTarget,
};

use crate::RenderLayer2dSceneService;

pub struct RenderLayer2dControlProvider {
    service: Arc<RenderLayer2dSceneService>,
}

impl RenderLayer2dControlProvider {
    pub fn new(service: Arc<RenderLayer2dSceneService>) -> Self {
        Self { service }
    }
}

impl RuntimeControlProvider for RenderLayer2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "render_layer_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for layer in self.service.commands() {
            let target_path = format!("world.{}", layer.id);
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(layer.id.clone()),
                label: layer.label.clone().unwrap_or_else(|| layer.id.clone()),
                components: vec!["RenderLayer2D".to_owned()],
                aliases: vec![format!("world.{}", layer.id.replace('.', "_"))],
                source_file: None,
            });
            for (property_path, value_type, range) in [
                (
                    "opacity",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(1.0),
                    }),
                ),
                ("visible", ControlValueType::Bool, None),
                ("order", ControlValueType::F32, None),
            ] {
                registry.register_property(RuntimeControlProperty {
                    console_path: format!("{target_path}.RenderLayer2D.{property_path}"),
                    target_path: target_path.clone(),
                    component: Some("RenderLayer2D".to_owned()),
                    property_path: property_path.to_owned(),
                    value_type,
                    range,
                    writable: true,
                    readable: true,
                    animatable: property_path != "visible",
                    source_file: None,
                    source_pointer: None,
                    provider_id: "render_layer_2d".to_owned(),
                    description: None,
                });
            }
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let layer_id = path.target_path.trim_start_matches("world.");
        let layer = self
            .service
            .commands()
            .into_iter()
            .find(|layer| layer.id == layer_id)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        match path.property_path.as_str() {
            "opacity" => Ok(ControlValue::F64(layer.opacity as f64)),
            "visible" => Ok(ControlValue::Bool(layer.visible)),
            "order" => Ok(ControlValue::F64(layer.order as f64)),
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
        let layer_id = path.target_path.trim_start_matches("world.");
        let updated = match path.property_path.as_str() {
            "opacity" => self.service.set_opacity(
                layer_id,
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
                    })?,
            ),
            "visible" => self.service.set_visible(
                layer_id,
                value
                    .as_bool()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "bool".to_owned(),
                        actual: "non-bool".to_owned(),
                    })?,
            ),
            "order" => self.service.set_order(
                layer_id,
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
