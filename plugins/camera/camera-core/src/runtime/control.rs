use std::sync::Arc;

use amigo_assets::{AssetCatalog, PreparedAsset, PreparedAssetKind};
use amigo_composite_plugin::RainGlass2d;
use amigo_runtime_control::{
    ControlRange, ControlValue, ControlValueType, RuntimeControlError, RuntimeControlProperty,
    RuntimeControlProvider, RuntimeControlRegistry, RuntimeControlTarget,
};

use crate::{
    Camera2dRuntimeState, CameraExposureMode2d, CameraId, CameraQualityProfile2d, CameraService,
    BUILTIN_FILM_STOCKS_2D, BUILTIN_LENS_PROFILES_2D,
};

pub struct Camera2dControlProvider {
    service: Arc<CameraService>,
    assets: Arc<AssetCatalog>,
}

impl Camera2dControlProvider {
    pub fn new(service: Arc<CameraService>, assets: Arc<AssetCatalog>) -> Self {
        Self { service, assets }
    }
}

impl RuntimeControlProvider for Camera2dControlProvider {
    fn provider_id(&self) -> &'static str {
        "camera_2d"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        for camera in self.service.cameras_2d() {
            let camera_id = camera.id.0.clone();
            let target_path = format!("world.camera.{}", sanitize_segment(camera_id.as_str()));
            let mut aliases = vec![format!(
                "world.camera.{}",
                sanitize_segment(camera.entity_name.as_str())
            )];
            if self
                .service
                .main_camera2d_id()
                .is_some_and(|main| main == camera.id)
            {
                aliases.push("world.camera.main".to_owned());
            }
            aliases.sort();
            aliases.dedup();
            registry.register_target(RuntimeControlTarget {
                console_path: target_path.clone(),
                source_id: Some(camera_id),
                label: camera.entity_name.clone(),
                components: vec!["Camera2D".to_owned()],
                aliases,
                source_file: None,
            });

            for (property_path, value_type, range) in [
                ("mode", ControlValueType::String, None),
                (
                    "exposure.iso",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(25.0),
                        max: Some(12800.0),
                    }),
                ),
                ("aperture.enabled", ControlValueType::Bool, None),
                (
                    "aperture.f_stop",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.7),
                        max: Some(32.0),
                    }),
                ),
                (
                    "aperture.focus_distance_m",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.2),
                        max: Some(1000.0),
                    }),
                ),
                (
                    "aperture.focus_depth",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(1.0),
                    }),
                ),
                (
                    "rig.camera_z_m",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(-50.0),
                        max: Some(50.0),
                    }),
                ),
                (
                    "rig.focus_residual_m",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(-5.0),
                        max: Some(5.0),
                    }),
                ),
                (
                    "rig.dolly_signal",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(-1.0),
                        max: Some(1.0),
                    }),
                ),
                ("shutter.enabled", ControlValueType::Bool, None),
                (
                    "shutter.speed_s",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(1.0 / 8000.0),
                        max: Some(2.0),
                    }),
                ),
                (
                    "shutter.opacity",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(1.0),
                    }),
                ),
                (
                    "shutter.history_mix",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(0.98),
                    }),
                ),
                (
                    "shutter.edge_rejection",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(1.0),
                    }),
                ),
                (
                    "aperture.dof.max_blur_px",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(90.0),
                    }),
                ),
                (
                    "aperture.dof.sample_count",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(12.0),
                        max: Some(96.0),
                    }),
                ),
                (
                    "aperture.dof.highlight_gain",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(8.0),
                    }),
                ),
                ("lens.profile", ControlValueType::AssetRef, None),
                ("film.profile", ControlValueType::AssetRef, None),
                ("look.profile", ControlValueType::AssetRef, None),
                (
                    "lens_surface.rain_profile",
                    ControlValueType::AssetRef,
                    None,
                ),
                (
                    "lens_surface.lens_rain.enabled",
                    ControlValueType::Bool,
                    None,
                ),
                (
                    "lens_surface.lens_rain.spawn_rate",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(120.0),
                    }),
                ),
                (
                    "lens_surface.lens_rain.opacity",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: Some(1.0),
                    }),
                ),
                (
                    "lens_surface.lens_rain.distortion_px",
                    ControlValueType::F32,
                    Some(ControlRange {
                        min: Some(0.0),
                        max: None,
                    }),
                ),
                ("quality", ControlValueType::String, None),
                ("debug.view", ControlValueType::String, None),
            ] {
                registry.register_property(RuntimeControlProperty {
                    console_path: format!("{target_path}.Camera2D.{property_path}"),
                    target_path: target_path.clone(),
                    component: Some("Camera2D".to_owned()),
                    property_path: property_path.to_owned(),
                    value_type,
                    range,
                    writable: true,
                    readable: true,
                    animatable: !matches!(
                        property_path,
                        "mode"
                            | "lens.profile"
                            | "film.profile"
                            | "look.profile"
                            | "lens_surface.rain_profile"
                    ),
                    source_file: None,
                    source_pointer: None,
                    provider_id: "camera_2d".to_owned(),
                    description: None,
                });
            }
        }
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let camera = resolve_camera(&self.service, &path.target_path)?;
        match path.property_path.as_str() {
            "mode" => Ok(ControlValue::String(
                match camera.mode {
                    CameraExposureMode2d::Auto => "auto",
                    CameraExposureMode2d::Manual => "manual",
                }
                .to_owned(),
            )),
            "exposure.iso" => Ok(ControlValue::F64(camera.exposure.iso as f64)),
            "aperture.enabled" => Ok(ControlValue::Bool(camera.aperture.enabled)),
            "aperture.f_stop" => Ok(ControlValue::F64(camera.aperture.f_stop as f64)),
            "aperture.focus_depth" => Ok(ControlValue::F64(match camera.aperture.focus {
                amigo_camera_optics_plugin::runtime::CameraFocus2d::Depth { value } => value,
                _ => 0.5,
            } as f64)),
            "aperture.focus_distance_m" => {
                Ok(ControlValue::F64(camera.aperture.focus_distance_m as f64))
            }
            "rig.camera_z_m" => Ok(ControlValue::F64(
                self.service.camera_depth_motion_2d(&camera.id).camera_z_m as f64,
            )),
            "rig.focus_residual_m" => Ok(ControlValue::F64(
                self.service
                    .camera_depth_motion_2d(&camera.id)
                    .focus_residual_m as f64,
            )),
            "rig.dolly_signal" => Ok(ControlValue::F64(
                self.service.camera_depth_motion_2d(&camera.id).dolly_signal as f64,
            )),
            "shutter.enabled" => Ok(ControlValue::Bool(camera.shutter.enabled)),
            "shutter.speed_s" => Ok(ControlValue::F64(
                camera
                    .shutter
                    .speed_s
                    .unwrap_or_else(|| camera.shutter.exposure_seconds()) as f64,
            )),
            "shutter.opacity" => Ok(ControlValue::F64(camera.shutter.opacity as f64)),
            "shutter.history_mix" => Ok(ControlValue::F64(camera.shutter.history_mix as f64)),
            "shutter.edge_rejection" => Ok(ControlValue::F64(camera.shutter.edge_rejection as f64)),
            "aperture.dof.max_blur_px" => Ok(ControlValue::F64(
                camera.aperture.depth_of_field.max_blur_px as f64,
            )),
            "aperture.dof.sample_count" => Ok(ControlValue::F64(
                camera.aperture.depth_of_field.sample_count as f64,
            )),
            "aperture.dof.highlight_gain" => Ok(ControlValue::F64(
                camera.aperture.depth_of_field.highlight_gain as f64,
            )),
            "lens.profile" => Ok(ControlValue::AssetRef(camera.lens.profile.clone())),
            "film.profile" => Ok(ControlValue::AssetRef(camera.film.profile.clone())),
            "look.profile" => Ok(ControlValue::AssetRef(camera.look.profile.clone())),
            "lens_surface.rain_profile" => Ok(camera
                .lens_surface
                .rain_profile
                .clone()
                .map(ControlValue::AssetRef)
                .unwrap_or(ControlValue::Null)),
            "lens_surface.lens_rain.enabled" => Ok(ControlValue::Bool(
                resolved_rain(&self.service, &self.assets, &camera)?.enabled,
            )),
            "lens_surface.lens_rain.spawn_rate" => Ok(ControlValue::F64(
                resolved_rain(&self.service, &self.assets, &camera)?.spawn_rate as f64,
            )),
            "lens_surface.lens_rain.opacity" => Ok(ControlValue::F64(
                resolved_rain(&self.service, &self.assets, &camera)?.opacity as f64,
            )),
            "lens_surface.lens_rain.distortion_px" => Ok(ControlValue::F64(
                resolved_rain(&self.service, &self.assets, &camera)?.distortion_px as f64,
            )),
            "quality" => Ok(ControlValue::String(
                self.service
                    .quality_profile_2d(&camera.id)
                    .as_str()
                    .to_owned(),
            )),
            "debug.view" => Ok(ControlValue::String(
                self.service.debug_view_2d(&camera.id).as_str().to_owned(),
            )),
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
        let camera = resolve_camera(&self.service, &path.target_path)?;
        let updated = match path.property_path.as_str() {
            "mode" => self.service.update_camera_2d(&camera.id, |state| {
                state.mode = match value.as_string() {
                    Some("auto") => CameraExposureMode2d::Auto,
                    Some("manual") => CameraExposureMode2d::Manual,
                    _ => return false,
                };
                true
            }),
            "exposure.iso" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(iso) = value.as_f32() else {
                    return false;
                };
                state.exposure.iso = iso;
                true
            }),
            "aperture.enabled" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(enabled) = value.as_bool() else {
                    return false;
                };
                state.aperture.enabled = enabled;
                true
            }),
            "aperture.f_stop" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(f_stop) = value.as_f32() else {
                    return false;
                };
                state.aperture.f_stop = f_stop;
                true
            }),
            "aperture.focus_depth" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(focus_depth) = value.as_f32() else {
                    return false;
                };
                state.aperture.focus = amigo_camera_optics_plugin::runtime::CameraFocus2d::Depth {
                    value: focus_depth.clamp(0.0, 1.0),
                };
                true
            }),
            "aperture.focus_distance_m" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(focus_distance_m) = value.as_f32() else {
                    return false;
                };
                state.aperture.focus_distance_m = focus_distance_m.clamp(0.2, 1000.0);
                state.aperture.focus =
                    amigo_camera_optics_plugin::runtime::CameraFocus2d::Distance {
                        meters: state.aperture.focus_distance_m,
                    };
                true
            }),
            "rig.camera_z_m" => {
                let Some(camera_z_m) = value.as_f32() else {
                    return Err(RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "rig.camera_z_m expects number".to_owned(),
                    });
                };
                self.service.set_camera_z_m_2d(&camera.id, camera_z_m)
            }
            "rig.focus_residual_m" => {
                let Some(focus_residual_m) = value.as_f32() else {
                    return Err(RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "rig.focus_residual_m expects number".to_owned(),
                    });
                };
                self.service
                    .set_focus_residual_m_2d(&camera.id, focus_residual_m)
            }
            "rig.dolly_signal" => {
                let Some(dolly_signal) = value.as_f32() else {
                    return Err(RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "rig.dolly_signal expects number".to_owned(),
                    });
                };
                self.service.set_dolly_signal_2d(&camera.id, dolly_signal)
            }
            "shutter.enabled" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(enabled) = value.as_bool() else {
                    return false;
                };
                state.shutter.enabled = enabled;
                true
            }),
            "shutter.speed_s" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(speed_s) = value.as_f32() else {
                    return false;
                };
                state.shutter.speed_s = Some(speed_s.clamp(1.0 / 8000.0, 2.0));
                true
            }),
            "shutter.opacity" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(opacity) = value.as_f32() else {
                    return false;
                };
                state.shutter.opacity = opacity.clamp(0.0, 1.0);
                true
            }),
            "shutter.history_mix" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(history_mix) = value.as_f32() else {
                    return false;
                };
                state.shutter.history_mix = history_mix.clamp(0.0, 0.98);
                true
            }),
            "shutter.edge_rejection" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(edge_rejection) = value.as_f32() else {
                    return false;
                };
                state.shutter.edge_rejection = edge_rejection.clamp(0.0, 1.0);
                true
            }),
            "aperture.dof.max_blur_px" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(max_blur_px) = value.as_f32() else {
                    return false;
                };
                state.aperture.depth_of_field.max_blur_px = max_blur_px;
                true
            }),
            "aperture.dof.sample_count" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(sample_count) = value.as_f32() else {
                    return false;
                };
                state.aperture.depth_of_field.sample_count = sample_count.round() as u32;
                true
            }),
            "aperture.dof.highlight_gain" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(highlight_gain) = value.as_f32() else {
                    return false;
                };
                state.aperture.depth_of_field.highlight_gain = highlight_gain;
                true
            }),
            "lens.profile" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(profile) = value.as_string() else {
                    return false;
                };
                state.lens.profile = profile.to_owned();
                true
            }),
            "film.profile" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(profile) = value.as_string() else {
                    return false;
                };
                state.film.profile = profile.to_owned();
                true
            }),
            "look.profile" => self.service.update_camera_2d(&camera.id, |state| {
                let Some(profile) = value.as_string() else {
                    return false;
                };
                state.look.profile = profile.to_owned();
                true
            }),
            "lens_surface.rain_profile" => {
                if let Some(profile) = value.as_string() {
                    self.service.set_lens_rain_profile_2d(&camera.id, profile)
                } else {
                    self.service.update_camera_2d(&camera.id, |state| {
                        state.lens_surface.rain_profile = None;
                        true
                    })
                }
            }
            "lens_surface.lens_rain.enabled" => {
                update_rain(&self.service, &self.assets, &camera.id, |rain| {
                    let Some(enabled) = value.as_bool() else {
                        return false;
                    };
                    rain.enabled = enabled;
                    true
                })
            }
            "lens_surface.lens_rain.spawn_rate" => {
                update_rain(&self.service, &self.assets, &camera.id, |rain| {
                    let Some(spawn_rate) = value.as_f32() else {
                        return false;
                    };
                    rain.spawn_rate = spawn_rate;
                    true
                })
            }
            "lens_surface.lens_rain.opacity" => {
                update_rain(&self.service, &self.assets, &camera.id, |rain| {
                    let Some(opacity) = value.as_f32() else {
                        return false;
                    };
                    rain.opacity = opacity;
                    true
                })
            }
            "lens_surface.lens_rain.distortion_px" => {
                update_rain(&self.service, &self.assets, &camera.id, |rain| {
                    let Some(distortion_px) = value.as_f32() else {
                        return false;
                    };
                    rain.distortion_px = distortion_px;
                    true
                })
            }
            "quality" => {
                let Some(raw) = value.as_string() else {
                    return Err(RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "quality expects string".to_owned(),
                    });
                };
                let profile = CameraQualityProfile2d::parse(raw);
                self.service.set_quality_profile_2d(&camera.id, profile)
            }
            "debug.view" => {
                let Some(raw) = value.as_string() else {
                    return Err(RuntimeControlError::Unsupported {
                        path: path.console_path.clone(),
                        reason: "debug.view expects string".to_owned(),
                    });
                };
                let debug_view = amigo_render_api::CameraDebugView2d::parse(raw);
                self.service.set_debug_view_2d(&camera.id, debug_view)
            }
            _ => false,
        };
        if updated {
            Ok(())
        } else {
            Err(RuntimeControlError::Unsupported {
                path: path.console_path.clone(),
                reason: "camera update failed".to_owned(),
            })
        }
    }

    fn reset(&self, path: &RuntimeControlProperty) -> Result<(), RuntimeControlError> {
        if path.property_path.starts_with("lens_surface.lens_rain.") {
            let camera = resolve_camera(&self.service, &path.target_path)?;
            if self.service.clear_lens_rain_override_2d(&camera.id) {
                return Ok(());
            }
        }
        Err(RuntimeControlError::Unsupported {
            path: path.console_path.clone(),
            reason: "reset not implemented".to_owned(),
        })
    }
}

pub struct AssetCatalogControlProvider {
    catalog: Arc<AssetCatalog>,
}

impl AssetCatalogControlProvider {
    pub fn new(catalog: Arc<AssetCatalog>) -> Self {
        Self { catalog }
    }
}

impl RuntimeControlProvider for AssetCatalogControlProvider {
    fn provider_id(&self) -> &'static str {
        "asset_catalog"
    }

    fn rebuild_registry(
        &self,
        registry: &mut RuntimeControlRegistry,
    ) -> Result<(), RuntimeControlError> {
        register_asset_targets(registry, &self.catalog.prepared_assets());
        Ok(())
    }

    fn get(&self, path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
        let asset_ref = path.description.clone().unwrap_or_default();
        Ok(ControlValue::AssetRef(asset_ref))
    }

    fn set(
        &self,
        path: &RuntimeControlProperty,
        _value: ControlValue,
    ) -> Result<(), RuntimeControlError> {
        Err(RuntimeControlError::NotWritable {
            path: path.console_path.clone(),
        })
    }
}

fn register_asset_targets(
    registry: &mut RuntimeControlRegistry,
    prepared_assets: &[PreparedAsset],
) {
    for namespace in ["films", "lenses", "looks", "lens_rain"] {
        registry.register_target(RuntimeControlTarget {
            console_path: format!("world.assets.{namespace}"),
            source_id: None,
            label: namespace.to_owned(),
            components: Vec::new(),
            aliases: Vec::new(),
            source_file: None,
        });
    }

    for lens in BUILTIN_LENS_PROFILES_2D {
        register_asset_leaf(registry, "lenses", lens.id, lens.id);
    }
    for film in BUILTIN_FILM_STOCKS_2D {
        register_asset_leaf(registry, "films", film.id, film.id);
    }
    register_prepared_assets(registry, prepared_assets);
}

fn register_asset_leaf(
    registry: &mut RuntimeControlRegistry,
    namespace: &str,
    leaf: &str,
    asset_ref: &str,
) {
    registry.register_property(RuntimeControlProperty {
        console_path: format!("world.assets.{namespace}.{leaf}"),
        target_path: format!("world.assets.{namespace}"),
        component: None,
        property_path: leaf.to_owned(),
        value_type: ControlValueType::AssetRef,
        range: None,
        writable: false,
        readable: true,
        animatable: false,
        source_file: None,
        source_pointer: None,
        provider_id: "asset_catalog".to_owned(),
        description: Some(asset_ref.to_owned()),
    });
    registry.register_alias(
        format!("world.assets.{namespace}.{}", sanitize_segment(asset_ref)),
        format!("world.assets.{namespace}.{leaf}"),
    );
}

fn register_prepared_assets(
    registry: &mut RuntimeControlRegistry,
    prepared_assets: &[PreparedAsset],
) {
    for prepared in prepared_assets {
        let Some((namespace, id)) = asset_namespace_and_id(prepared) else {
            continue;
        };
        register_asset_leaf(
            registry,
            namespace,
            sanitize_segment(id.as_str()).as_str(),
            prepared.key.as_str(),
        );
    }
}

fn asset_namespace_and_id(prepared: &PreparedAsset) -> Option<(&'static str, String)> {
    match &prepared.kind {
        PreparedAssetKind::Unknown(kind) if kind == "camera-film-stock-2d" => {
            Some(("films", metadata_id(prepared)?))
        }
        PreparedAssetKind::Unknown(kind) if kind == "camera-lens-profile-2d" => {
            Some(("lenses", metadata_id(prepared)?))
        }
        PreparedAssetKind::Unknown(kind) if kind == "camera-look-profile-2d" => {
            Some(("looks", metadata_id(prepared)?))
        }
        PreparedAssetKind::Unknown(kind) if kind == "camera-rain-glass-profile-2d" => {
            Some(("lens_rain", metadata_id(prepared)?))
        }
        _ => None,
    }
}

fn metadata_id(prepared: &PreparedAsset) -> Option<String> {
    prepared.metadata.get("id").cloned()
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn resolve_camera(
    service: &CameraService,
    target_path: &str,
) -> Result<Camera2dRuntimeState, RuntimeControlError> {
    let resolved = target_path.trim_start_matches("world.camera.");
    let target = if resolved == "main" {
        service.main_camera2d_id()
    } else {
        service
            .cameras_2d()
            .into_iter()
            .find(|camera| {
                sanitize_segment(camera.id.0.as_str()) == resolved
                    || sanitize_segment(camera.entity_name.as_str()) == resolved
            })
            .map(|camera| camera.id)
    }
    .ok_or_else(|| RuntimeControlError::UnknownTarget(target_path.to_owned()))?;
    service
        .get_2d(&target)
        .ok_or_else(|| RuntimeControlError::UnknownTarget(target_path.to_owned()))
}

fn resolved_rain(
    service: &CameraService,
    assets: &AssetCatalog,
    camera: &Camera2dRuntimeState,
) -> Result<RainGlass2d, RuntimeControlError> {
    service
        .resolved_lens_rain_2d(camera, Some(assets))
        .ok_or_else(|| RuntimeControlError::ProviderUnavailable {
            path: camera.entity_name.clone(),
        })
}

fn update_rain(
    service: &CameraService,
    assets: &AssetCatalog,
    camera_id: &CameraId,
    update: impl FnOnce(&mut RainGlass2d) -> bool,
) -> bool {
    let mut changed = false;
    service.update_lens_rain_2d(camera_id, Some(assets), |rain| {
        changed = update(rain);
    }) && changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_camera_optics_plugin::runtime::{
        Camera2dRuntimeState, CameraAperture2d, CameraAutoExposure2d, CameraDepthOfField2d,
        CameraExposure2d, CameraExposureMode2d, CameraFilm2d, CameraFocus2d, CameraLens2d,
        CameraLensSurface2d, CameraLook2d, CameraShutter2d,
    };
    use amigo_runtime_control::RuntimeControlService;

    fn sample_camera() -> Camera2dRuntimeState {
        Camera2dRuntimeState {
            id: CameraId::main(),
            entity_name: "camera.main".to_owned(),
            mode: CameraExposureMode2d::Manual,
            exposure: CameraExposure2d {
                iso: 800.0,
                compensation: 0.0,
                white_balance: 5600.0,
                nd_stops: 0.0,
                auto: CameraAutoExposure2d {
                    target_luma: 0.5,
                    adaptation_speed: 1.0,
                    min_iso: 100.0,
                    max_iso: 3200.0,
                },
            },
            shutter: CameraShutter2d {
                enabled: false,
                speed_s: None,
                fps: 24.0,
                angle: 180.0,
                opacity: 0.0,
                history_mix: 0.0,
                history_mix_2: 0.0,
                edge_rejection: 0.0,
                luma_threshold: 0.0,
                frame_hold: false,
            },
            lens: CameraLens2d {
                profile: "clean_modern_35mm".to_owned(),
                intensity: 1.0,
                aberration_px: None,
                distortion: None,
                vignette: None,
                edge_softness_px: None,
                glare_strength: None,
                dirt: None,
                focal_length_mm: None,
                lens_bloom: None,
                flare_ghosts: None,
                anamorphic_squeeze: None,
                coma: None,
                cat_eye_bokeh: None,
                focus_breathing: None,
            },
            lens_surface: CameraLensSurface2d { rain_profile: None },
            film: CameraFilm2d {
                profile: "neutral_digital_400".to_owned(),
                intensity: 1.0,
                seed: 7,
                color_shift: None,
                contrast: None,
                saturation: None,
                flicker: None,
                vignette: None,
                toe: None,
                shoulder: None,
                black_lift: None,
                print_fade: None,
                dust: None,
                scratches: None,
                push_pull: None,
                gate_weave: None,
                scan_softness: None,
            },
            look: CameraLook2d {
                profile: "none".to_owned(),
                intensity: 0.0,
            },
            aperture: CameraAperture2d {
                enabled: false,
                f_stop: 2.8,
                focus_distance_m: 5.0,
                focus: CameraFocus2d::None,
                depth_of_field: CameraDepthOfField2d {
                    depth_map: None,
                    affected_layers: Vec::new(),
                    max_blur_px: 28.0,
                    depth_contrast: 1.0,
                    focus_width: 0.055,
                    foreground_blur_boost: 1.15,
                    background_blur_boost: 1.0,
                    edge_aware: true,
                    invert_depth: false,
                    debug_view: "final".to_owned(),
                    aperture_blades: 7,
                    aperture_roundness: 0.72,
                    aperture_rotation_degrees: 0.0,
                    sample_count: 64,
                    highlight_threshold: 0.68,
                    highlight_knee: 0.18,
                    highlight_gain: 1.45,
                    highlight_saturation: 1.10,
                },
            },
            render_contributions: amigo_render_api::RenderContributionSet::from_pairs([(
                amigo_render_api::render_contribution_roles::CAMERA_PROJECTION,
                true,
            )]),
        }
    }

    #[test]
    fn runtime_control_updates_camera_iso() {
        let cameras = Arc::new(CameraService::default());
        cameras.upsert_2d(sample_camera());
        let assets = Arc::new(AssetCatalog::default());

        let control = RuntimeControlService::default();
        control.register_provider(Arc::new(Camera2dControlProvider::new(
            cameras.clone(),
            assets,
        )));

        control
            .set(
                "world.camera.main.Camera2D.exposure.iso",
                ControlValue::F64(1200.0),
            )
            .expect("camera iso should update");

        let updated = cameras.main_camera2d().expect("main camera should exist");
        assert_eq!(updated.exposure.iso, 1200.0);
    }

    #[test]
    fn asset_runtime_control_reads_builtin_film_ref() {
        let control = RuntimeControlService::default();
        control.register_provider(Arc::new(AssetCatalogControlProvider::new(Arc::new(
            AssetCatalog::default(),
        ))));

        let value = control
            .get("world.assets.films.neutral_digital_400")
            .expect("builtin film asset should resolve");
        assert_eq!(
            value,
            ControlValue::AssetRef("neutral_digital_400".to_owned())
        );
    }
}
