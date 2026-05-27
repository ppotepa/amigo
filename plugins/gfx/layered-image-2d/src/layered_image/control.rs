use std::sync::Arc;

use amigo_assets::AssetCatalog;
use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError, RuntimeControlProperty,
    RuntimeControlProvider, RuntimeControlRegistry, RuntimeControlTarget,
};

use super::{
    apply_layer_overrides, LayeredImageAssetSource, LayeredImageBlendMode2d,
    LayeredImageSceneService,
};

pub struct LayeredImage2dControlProvider {
    service: Arc<LayeredImageSceneService>,
    assets: Arc<AssetCatalog>,
}

impl LayeredImage2dControlProvider {
    pub fn new(service: Arc<LayeredImageSceneService>, assets: Arc<AssetCatalog>) -> Self {
        Self { service, assets }
    }
}

impl RuntimeControlProvider for LayeredImage2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "layered_image_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for command in self.service.commands() {
            let target_path = format!("world.{}", command.render_layer);
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(command.entity_name.clone()),
                label: command.entity_name.clone(),
                components: vec!["LayeredImage2D".to_owned()],
                aliases: vec![format!("world.{}", command.entity_name.replace('-', "_"))],
                source_file: None,
            });
            registry.register_property(RuntimeControlProperty {
                console_path: format!("{target_path}.LayeredImage2D.base_opacity"),
                target_path: target_path.clone(),
                component: Some("LayeredImage2D".to_owned()),
                property_path: "base_opacity".to_owned(),
                value_type: ControlValueType::F32,
                range: Some(ControlRange {
                    min: Some(0.0),
                    max: Some(1.0),
                }),
                writable: true,
                readable: true,
                animatable: true,
                source_file: None,
                source_pointer: None,
                provider_id: "layered_image_2d".to_owned(),
                description: None,
            });
            if let Some(mut asset) = self.assets.layered_image_asset(&command.image.asset) {
                apply_layer_overrides(&mut asset, &command.image.layer_overrides);
                for layer in asset.layers {
                    for (property_path, value_type) in [
                        (
                            format!("layers.{}.opacity", layer.id),
                            ControlValueType::F32,
                        ),
                        (
                            format!("layers.{}.enabled", layer.id),
                            ControlValueType::Bool,
                        ),
                        (
                            format!("layers.{}.blend", layer.id),
                            ControlValueType::String,
                        ),
                    ] {
                        registry.register_property(RuntimeControlProperty {
                            console_path: format!("{target_path}.LayeredImage2D.{property_path}"),
                            target_path: target_path.clone(),
                            component: Some("LayeredImage2D".to_owned()),
                            property_path,
                            value_type,
                            range: None,
                            writable: true,
                            readable: true,
                            animatable: true,
                            source_file: None,
                            source_pointer: None,
                            provider_id: "layered_image_2d".to_owned(),
                            description: None,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let command = self
            .service
            .commands()
            .into_iter()
            .find(|command| format!("world.{}", command.render_layer) == path.target_path)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        if path.property_path == "base_opacity" {
            return Ok(ControlValue::F64(command.image.base_opacity as f64));
        }
        let mut asset = self
            .assets
            .layered_image_asset(&command.image.asset)
            .ok_or_else(|| RuntimeControlError::ProviderUnavailable {
                path: path.console_path.clone(),
            })?;
        apply_layer_overrides(&mut asset, &command.image.layer_overrides);
        let parts = path.property_path.split('.').collect::<Vec<_>>();
        if parts.len() != 3 || parts[0] != "layers" {
            return Err(RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            });
        }
        let layer = asset
            .layers
            .into_iter()
            .find(|layer| layer.id == parts[1])
            .ok_or_else(|| RuntimeControlError::UnknownProperty {
                path: path.console_path.clone(),
            })?;
        match parts[2] {
            "opacity" => Ok(ControlValue::F64(layer.opacity as f64)),
            "enabled" => Ok(ControlValue::Bool(layer.enabled)),
            "blend" => Ok(ControlValue::String(layer.blend_mode.as_str().to_owned())),
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
        let command = self
            .service
            .commands()
            .into_iter()
            .find(|command| format!("world.{}", command.render_layer) == path.target_path)
            .ok_or_else(|| RuntimeControlError::UnknownTarget(path.target_path.clone()))?;
        let updated = if path.property_path == "base_opacity" {
            self.service.set_base_opacity(
                &command.entity_name,
                value
                    .as_f32()
                    .ok_or_else(|| RuntimeControlError::TypeMismatch {
                        path: path.console_path.clone(),
                        expected: "f32".to_owned(),
                        actual: "non-number".to_owned(),
                    })?,
            )
        } else {
            let parts = path.property_path.split('.').collect::<Vec<_>>();
            if parts.len() != 3 || parts[0] != "layers" {
                false
            } else {
                match parts[2] {
                    "opacity" => self.service.set_layer_opacity(
                        &command.entity_name,
                        parts[1],
                        value
                            .as_f32()
                            .ok_or_else(|| RuntimeControlError::TypeMismatch {
                                path: path.console_path.clone(),
                                expected: "f32".to_owned(),
                                actual: "non-number".to_owned(),
                            })?,
                    ),
                    "enabled" => self.service.set_layer_enabled(
                        &command.entity_name,
                        parts[1],
                        value
                            .as_bool()
                            .ok_or_else(|| RuntimeControlError::TypeMismatch {
                                path: path.console_path.clone(),
                                expected: "bool".to_owned(),
                                actual: "non-bool".to_owned(),
                            })?,
                    ),
                    "blend" => self.service.set_layer_blend_mode(
                        &command.entity_name,
                        parts[1],
                        LayeredImageBlendMode2d::parse_strict(value.as_string().unwrap_or(""))
                            .ok_or_else(|| RuntimeControlError::TypeMismatch {
                                path: path.console_path.clone(),
                                expected: "blend string".to_owned(),
                                actual: "invalid".to_owned(),
                            })?,
                    ),
                    _ => false,
                }
            }
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
