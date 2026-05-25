use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{
    ConsoleControlPath, ControlValue, RuntimeControlCompletionEntry, RuntimeControlCompletionKind,
    RuntimeControlError, RuntimeControlProperty, RuntimeControlProvider, RuntimeControlRegistry,
    RuntimeControlSceneMetadata, RuntimeControlTarget,
};

#[derive(Default)]
pub struct RuntimeControlService {
    providers: RwLock<Vec<Arc<dyn RuntimeControlProvider>>>,
    registry: RwLock<RuntimeControlRegistry>,
    scene_metadata: RwLock<RuntimeControlSceneMetadata>,
    dirty_paths: RwLock<BTreeSet<String>>,
    rebuild_needed: RwLock<bool>,
}

impl RuntimeControlService {
    pub fn register_provider(&self, provider: Arc<dyn RuntimeControlProvider>) {
        self.providers
            .write()
            .expect("runtime control provider lock should not be poisoned")
            .push(provider);
        *self
            .rebuild_needed
            .write()
            .expect("runtime control rebuild flag lock should not be poisoned") = true;
    }

    pub fn replace_scene_metadata(&self, metadata: RuntimeControlSceneMetadata) {
        *self
            .scene_metadata
            .write()
            .expect("runtime control scene metadata lock should not be poisoned") = metadata;
        *self
            .rebuild_needed
            .write()
            .expect("runtime control rebuild flag lock should not be poisoned") = true;
    }

    pub fn clear_scene_metadata(&self) {
        self.replace_scene_metadata(RuntimeControlSceneMetadata::default());
        self.dirty_paths
            .write()
            .expect("runtime control dirty paths lock should not be poisoned")
            .clear();
    }

    pub fn rebuild(&self) -> Result<(), RuntimeControlError> {
        let providers = self
            .providers
            .read()
            .expect("runtime control provider lock should not be poisoned")
            .clone();
        let mut registry = RuntimeControlRegistry::default();
        for provider in &providers {
            provider.rebuild_registry(&mut registry)?;
        }
        *self
            .registry
            .write()
            .expect("runtime control registry lock should not be poisoned") = registry;
        *self
            .rebuild_needed
            .write()
            .expect("runtime control rebuild flag lock should not be poisoned") = false;
        Ok(())
    }

    pub fn registry_snapshot(&self) -> RuntimeControlRegistry {
        let _ = self.ensure_built();
        self.registry
            .read()
            .expect("runtime control registry lock should not be poisoned")
            .clone()
    }

    pub fn completion_entries(&self) -> Vec<RuntimeControlCompletionEntry> {
        self.complete("world.", 6)
    }

    pub fn complete(&self, input: &str, cursor: usize) -> Vec<RuntimeControlCompletionEntry> {
        let _ = self.ensure_built();
        let input = &input[..cursor.min(input.len())];
        if !input.trim_start().starts_with("world") {
            return Vec::new();
        }
        let registry = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned");
        let trimmed = input.trim();

        if let Some(base) = trimmed.strip_suffix(".info()") {
            return method_entries(base, "info");
        }
        if let Some(base) = trimmed.strip_suffix(".reset()") {
            return method_entries(base, "reset");
        }
        if let Some(base) = trimmed.strip_suffix(".commit()") {
            return method_entries(base, "commit");
        }

        let prefix = if trimmed.ends_with('.') {
            trimmed.trim_end_matches('.')
        } else {
            trimmed
                .rsplit_once('.')
                .map(|(head, _)| head)
                .unwrap_or(trimmed)
        };
        let partial = if trimmed.ends_with('.') {
            ""
        } else {
            trimmed
                .rsplit_once('.')
                .map(|(_, tail)| tail)
                .unwrap_or(trimmed)
        };

        registry
            .children_of(prefix)
            .into_iter()
            .filter(|entry| entry.label.starts_with(partial))
            .collect()
    }

    pub fn describe(&self, path: &str) -> Result<String, RuntimeControlError> {
        self.info(path)
    }

    pub fn get(&self, path: &str) -> Result<ControlValue, RuntimeControlError> {
        self.ensure_built()?;
        if let Some(property) = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned")
            .property(path)
            .cloned()
        {
            return self.dispatch_get(&property);
        }
        Ok(ControlValue::String(self.info(path)?))
    }

    pub fn set(&self, path: &str, value: ControlValue) -> Result<(), RuntimeControlError> {
        self.ensure_built()?;
        let property = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned")
            .property(path)
            .cloned()
            .ok_or_else(|| RuntimeControlError::UnknownProperty {
                path: path.to_owned(),
            })?;
        if !property.writable {
            return Err(RuntimeControlError::NotWritable {
                path: property.console_path.clone(),
            });
        }
        let coerced = value
            .coerce_to(property.value_type)
            .map_err(|error| rewrite_type_mismatch_path(error, property.console_path.clone()))?;
        validate_range(&property, &coerced)?;
        self.dispatch_set(&property, coerced)?;
        self.dirty_paths
            .write()
            .expect("runtime control dirty paths lock should not be poisoned")
            .insert(property.console_path.clone());
        Ok(())
    }

    pub fn info(&self, path: &str) -> Result<String, RuntimeControlError> {
        self.ensure_built()?;
        let registry = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned");
        let dirty = self
            .dirty_paths
            .read()
            .expect("runtime control dirty paths lock should not be poisoned");

        if let Some(property) = registry.property(path) {
            let dirty_flag = dirty.contains(property.console_path.as_str());
            return Ok(format!(
                "path: {}\ntype: {:?}\nprovider: {}\nreadable: {}\nwritable: {}\ndirty: {}\nsource_file: {}\nsource_pointer: {}",
                property.console_path,
                property.value_type,
                property.provider_id,
                property.readable,
                property.writable,
                dirty_flag,
                property
                    .source_file
                    .clone()
                    .unwrap_or_else(|| "-".to_owned()),
                property
                    .source_pointer
                    .clone()
                    .unwrap_or_else(|| "-".to_owned()),
            ));
        }
        if let Some(target) = registry.target(path) {
            return Ok(format!(
                "target: {}\nlabel: {}\ncomponents: {}\nsource_file: {}",
                target.console_path,
                target.label,
                target.components.join(", "),
                target.source_file.clone().unwrap_or_else(|| "-".to_owned()),
            ));
        }

        let metadata = self
            .scene_metadata
            .read()
            .expect("runtime control scene metadata lock should not be poisoned");
        let parsed = ConsoleControlPath::parse(path)?;
        let resolved = parsed.resolve(&metadata)?;
        if let Some(component) = resolved.component {
            return Ok(format!(
                "target: world.{}\ncomponent: {}",
                resolved.target_path, component
            ));
        }
        Err(RuntimeControlError::UnknownProperty {
            path: path.to_owned(),
        })
    }

    pub fn reset(&self, path: &str) -> Result<(), RuntimeControlError> {
        self.ensure_built()?;
        let property = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned")
            .property(path)
            .cloned()
            .ok_or_else(|| RuntimeControlError::UnknownProperty {
                path: path.to_owned(),
            })?;
        self.with_provider(property.provider_id.as_str(), |provider| {
            provider.reset(&property)
        })
    }

    pub fn commit(&self, path: &str) -> Result<(), RuntimeControlError> {
        self.ensure_built()?;
        let registry = self
            .registry
            .read()
            .expect("runtime control registry lock should not be poisoned");
        if let Some(property) = registry.property(path).cloned() {
            drop(registry);
            return self.commit_property(&property);
        }
        let properties = self.properties_for_commit_scope(path, &registry)?;
        drop(registry);

        let dirty = self
            .dirty_paths
            .read()
            .expect("runtime control dirty paths lock should not be poisoned")
            .clone();
        let dirty_properties = properties
            .into_iter()
            .filter(|property| dirty.contains(&property.console_path))
            .collect::<Vec<_>>();
        if dirty_properties.is_empty() {
            return Err(RuntimeControlError::Unsupported {
                path: path.to_owned(),
                reason: "no dirty committable properties under scope".to_owned(),
            });
        }

        let mut committed = 0usize;
        let mut skipped = Vec::new();
        for property in dirty_properties {
            match self.commit_property(&property) {
                Ok(()) => committed += 1,
                Err(RuntimeControlError::Unsupported { path: _, reason }) => {
                    skipped.push(format!("{} ({reason})", property.console_path))
                }
                Err(error) => return Err(error),
            }
        }

        if committed == 0 {
            return Err(RuntimeControlError::Unsupported {
                path: path.to_owned(),
                reason: if skipped.is_empty() {
                    "no committable properties under scope".to_owned()
                } else {
                    format!(
                        "no committable properties under scope: {}",
                        skipped.join(", ")
                    )
                },
            });
        }
        Ok(())
    }

    fn ensure_built(&self) -> Result<(), RuntimeControlError> {
        let rebuild = *self
            .rebuild_needed
            .read()
            .expect("runtime control rebuild flag lock should not be poisoned");
        if rebuild {
            self.rebuild()?;
            self.decorate_registry_from_scene_metadata();
        } else {
            self.decorate_registry_from_scene_metadata();
        }
        Ok(())
    }

    fn decorate_registry_from_scene_metadata(&self) {
        let metadata = self
            .scene_metadata
            .read()
            .expect("runtime control scene metadata lock should not be poisoned")
            .clone();
        let mut registry = self
            .registry
            .write()
            .expect("runtime control registry lock should not be poisoned");
        for target_metadata in metadata.target_lookup.values() {
            if !registry
                .targets_by_path
                .contains_key(format!("world.{}", target_metadata.canonical_target).as_str())
            {
                registry.register_target(RuntimeControlTarget {
                    console_path: format!("world.{}", target_metadata.canonical_target),
                    source_id: target_metadata.source_id.clone(),
                    label: target_metadata.display_name.clone(),
                    components: target_metadata
                        .components
                        .iter()
                        .map(|component| component.console_component.clone())
                        .collect(),
                    aliases: target_metadata
                        .aliases
                        .iter()
                        .map(|alias| format!("world.{alias}"))
                        .collect(),
                    source_file: target_metadata.source_file.clone(),
                });
            }
            for component in &target_metadata.components {
                for property in &component.properties {
                    let console_path = format!(
                        "world.{}.{}.{}",
                        target_metadata.canonical_target,
                        component.console_component,
                        property.property_path
                    );
                    if registry
                        .properties_by_path
                        .contains_key(console_path.as_str())
                    {
                        continue;
                    }
                    registry.register_property(RuntimeControlProperty {
                        console_path,
                        target_path: format!("world.{}", target_metadata.canonical_target),
                        component: Some(component.console_component.clone()),
                        property_path: property.property_path.clone(),
                        value_type: property.value_type,
                        range: property.range.clone(),
                        writable: false,
                        readable: false,
                        animatable: false,
                        source_file: target_metadata.source_file.clone(),
                        source_pointer: property.source_pointer.clone(),
                        provider_id: "__metadata__".to_owned(),
                        description: Some("metadata-only runtime control stub".to_owned()),
                    });
                }
            }
            for component in &target_metadata.components {
                registry.register_alias(
                    format!(
                        "world.{}.{}",
                        target_metadata.canonical_target, component.source_component
                    ),
                    format!(
                        "world.{}.{}",
                        target_metadata.canonical_target, component.console_component
                    ),
                );
            }
        }
    }

    fn dispatch_get(
        &self,
        property: &RuntimeControlProperty,
    ) -> Result<ControlValue, RuntimeControlError> {
        self.with_provider(property.provider_id.as_str(), |provider| {
            provider.get(property)
        })
    }

    fn dispatch_set(
        &self,
        property: &RuntimeControlProperty,
        value: ControlValue,
    ) -> Result<(), RuntimeControlError> {
        self.with_provider(property.provider_id.as_str(), |provider| {
            provider.set(property, value)
        })
    }

    fn commit_property(
        &self,
        property: &RuntimeControlProperty,
    ) -> Result<(), RuntimeControlError> {
        let Some(source_file) = property.source_file.clone() else {
            return self.with_provider(property.provider_id.as_str(), |provider| {
                provider.commit(property)
            });
        };
        let Some(source_pointer) = property.source_pointer.clone() else {
            return self.with_provider(property.provider_id.as_str(), |provider| {
                provider.commit(property)
            });
        };

        let value = self.dispatch_get(property)?;
        let path = PathBuf::from(&source_file);
        let raw =
            std::fs::read_to_string(&path).map_err(|error| RuntimeControlError::Unsupported {
                path: property.console_path.clone(),
                reason: format!("failed to read `{}`: {error}", path.display()),
            })?;
        let mut document = serde_yaml::from_str::<serde_yaml::Value>(&raw).map_err(|error| {
            RuntimeControlError::Unsupported {
                path: property.console_path.clone(),
                reason: format!("failed to parse `{}`: {error}", path.display()),
            }
        })?;
        write_yaml_pointer(
            &mut document,
            source_pointer.as_str(),
            control_value_to_yaml(value),
            property.console_path.as_str(),
        )?;
        let encoded =
            serde_yaml::to_string(&document).map_err(|error| RuntimeControlError::Unsupported {
                path: property.console_path.clone(),
                reason: format!("failed to encode `{}`: {error}", path.display()),
            })?;
        std::fs::write(&path, encoded).map_err(|error| RuntimeControlError::Unsupported {
            path: property.console_path.clone(),
            reason: format!("failed to write `{}`: {error}", path.display()),
        })?;
        self.dirty_paths
            .write()
            .expect("runtime control dirty paths lock should not be poisoned")
            .remove(&property.console_path);
        Ok(())
    }

    fn properties_for_commit_scope(
        &self,
        path: &str,
        registry: &RuntimeControlRegistry,
    ) -> Result<Vec<RuntimeControlProperty>, RuntimeControlError> {
        let resolved_path = registry.resolve_alias(path);
        if let Some(target) = registry.target(path) {
            let prefix = format!("{}.", target.console_path);
            return Ok(registry
                .properties_by_path
                .values()
                .filter(|property| property.console_path.starts_with(&prefix))
                .cloned()
                .collect());
        }
        if let Some(component_properties) =
            registry.properties_by_path.values().find_map(|property| {
                let component_path = format!(
                    "{}.{}",
                    property.target_path,
                    property.component.as_deref().unwrap_or_default()
                );
                (component_path == resolved_path).then(|| {
                    registry.properties_for_target_component(
                        &property.target_path,
                        property.component.as_deref().unwrap_or_default(),
                    )
                })
            })
        {
            return Ok(component_properties);
        }

        let metadata = self
            .scene_metadata
            .read()
            .expect("runtime control scene metadata lock should not be poisoned");
        let parsed = ConsoleControlPath::parse(path)?;
        let resolved = parsed.resolve(&metadata)?;
        let target_path = format!("world.{}", resolved.target_path);
        if let Some(component) = resolved.component {
            return Ok(registry.properties_for_target_component(&target_path, &component));
        }
        Ok(registry
            .properties_by_path
            .values()
            .filter(|property| property.target_path == target_path)
            .cloned()
            .collect())
    }

    fn with_provider<T>(
        &self,
        provider_id: &str,
        run: impl FnOnce(&Arc<dyn RuntimeControlProvider>) -> Result<T, RuntimeControlError>,
    ) -> Result<T, RuntimeControlError> {
        let providers = self
            .providers
            .read()
            .expect("runtime control provider lock should not be poisoned");
        let provider = providers
            .iter()
            .find(|provider| provider.provider_id() == provider_id)
            .ok_or_else(|| RuntimeControlError::ProviderUnavailable {
                path: provider_id.to_owned(),
            })?;
        run(provider)
    }
}

fn control_value_to_yaml(value: ControlValue) -> serde_yaml::Value {
    match value {
        ControlValue::Bool(value) => serde_yaml::Value::Bool(value),
        ControlValue::I64(value) => serde_yaml::to_value(value).unwrap_or(serde_yaml::Value::Null),
        ControlValue::U64(value) => serde_yaml::to_value(value).unwrap_or(serde_yaml::Value::Null),
        ControlValue::F64(value) => serde_yaml::to_value(value).unwrap_or(serde_yaml::Value::Null),
        ControlValue::String(value) | ControlValue::AssetRef(value) => {
            serde_yaml::Value::String(value)
        }
        ControlValue::Null => serde_yaml::Value::Null,
    }
}

fn write_yaml_pointer(
    root: &mut serde_yaml::Value,
    pointer: &str,
    value: serde_yaml::Value,
    path: &str,
) -> Result<(), RuntimeControlError> {
    let segments = pointer_segments(pointer);
    if segments.is_empty() {
        *root = value;
        return Ok(());
    }

    let mut current = root;
    for segment in &segments[..segments.len() - 1] {
        current = match current {
            serde_yaml::Value::Sequence(items) => {
                let index =
                    segment
                        .parse::<usize>()
                        .map_err(|_| RuntimeControlError::Unsupported {
                            path: path.to_owned(),
                            reason: format!("invalid yaml sequence index `{segment}`"),
                        })?;
                items
                    .get_mut(index)
                    .ok_or_else(|| RuntimeControlError::Unsupported {
                        path: path.to_owned(),
                        reason: format!("yaml sequence index out of range: `{segment}`"),
                    })?
            }
            serde_yaml::Value::Mapping(map) => {
                let key = serde_yaml::Value::String(segment.clone());
                map.get_mut(&key)
                    .ok_or_else(|| RuntimeControlError::Unsupported {
                        path: path.to_owned(),
                        reason: format!("yaml mapping key missing: `{segment}`"),
                    })?
            }
            _ => {
                return Err(RuntimeControlError::Unsupported {
                    path: path.to_owned(),
                    reason: format!("yaml pointer entered non-container at `{segment}`"),
                });
            }
        };
    }

    let leaf = segments.last().cloned().unwrap_or_default();
    match current {
        serde_yaml::Value::Sequence(items) => {
            let index = leaf
                .parse::<usize>()
                .map_err(|_| RuntimeControlError::Unsupported {
                    path: path.to_owned(),
                    reason: format!("invalid yaml sequence index `{leaf}`"),
                })?;
            let slot = items
                .get_mut(index)
                .ok_or_else(|| RuntimeControlError::Unsupported {
                    path: path.to_owned(),
                    reason: format!("yaml sequence index out of range: `{leaf}`"),
                })?;
            *slot = value;
            Ok(())
        }
        serde_yaml::Value::Mapping(map) => {
            map.insert(serde_yaml::Value::String(leaf), value);
            Ok(())
        }
        _ => Err(RuntimeControlError::Unsupported {
            path: path.to_owned(),
            reason: "yaml pointer leaf is not writable".to_owned(),
        }),
    }
}

fn pointer_segments(pointer: &str) -> Vec<String> {
    pointer
        .split('/')
        .filter(|segment| !segment.is_empty())
        .flat_map(|segment| segment.split('.'))
        .filter(|segment| !segment.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn validate_range(
    property: &RuntimeControlProperty,
    value: &ControlValue,
) -> Result<(), RuntimeControlError> {
    let Some(range) = &property.range else {
        return Ok(());
    };
    let Some(number) = value.as_f64() else {
        return Ok(());
    };
    if range.min.is_some_and(|min| number < min) || range.max.is_some_and(|max| number > max) {
        return Err(RuntimeControlError::OutOfRange {
            path: property.console_path.clone(),
            value: number.to_string(),
        });
    }
    Ok(())
}

fn rewrite_type_mismatch_path(error: RuntimeControlError, path: String) -> RuntimeControlError {
    match error {
        RuntimeControlError::TypeMismatch {
            expected, actual, ..
        } => RuntimeControlError::TypeMismatch {
            path,
            expected,
            actual,
        },
        other => other,
    }
}

fn method_entries(base: &str, method: &str) -> Vec<RuntimeControlCompletionEntry> {
    vec![RuntimeControlCompletionEntry {
        label: method.to_owned(),
        insert_text: format!("{method}()"),
        path: format!("{base}.{method}()"),
        kind: RuntimeControlCompletionKind::Method,
        detail: None,
    }]
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::{ControlRange, ControlValueType, RuntimeControlTarget};

    const MOCK_COMPONENT: &str = "Emitter";
    const MOCK_COMPONENT_PATH: &str = "world.weather.rain.front.Emitter";

    struct MockProvider {
        last_set: Mutex<Option<(String, ControlValue)>>,
        current_value: Mutex<ControlValue>,
    }

    impl Default for MockProvider {
        fn default() -> Self {
            Self {
                last_set: Mutex::new(None),
                current_value: Mutex::new(ControlValue::F64(120.0)),
            }
        }
    }

    impl RuntimeControlProvider for MockProvider {
        fn provider_id(&self) -> &'static str {
            "mock"
        }

        fn rebuild_registry(
            &self,
            registry: &mut RuntimeControlRegistry,
        ) -> Result<(), RuntimeControlError> {
            registry.register_target(RuntimeControlTarget {
                console_path: "world.weather.rain.front".to_owned(),
                source_id: None,
                label: "rain-front".to_owned(),
                components: vec![MOCK_COMPONENT.to_owned()],
                aliases: Vec::new(),
                source_file: None,
            });
            registry.register_property(RuntimeControlProperty {
                console_path: format!("{MOCK_COMPONENT_PATH}.spawn_rate"),
                target_path: "world.weather.rain.front".to_owned(),
                component: Some(MOCK_COMPONENT.to_owned()),
                property_path: "spawn_rate".to_owned(),
                value_type: ControlValueType::F32,
                range: Some(ControlRange {
                    min: Some(0.0),
                    max: None,
                }),
                writable: true,
                readable: true,
                animatable: true,
                source_file: None,
                source_pointer: None,
                provider_id: "mock".to_owned(),
                description: None,
            });
            Ok(())
        }

        fn get(&self, _path: &RuntimeControlProperty) -> Result<ControlValue, RuntimeControlError> {
            Ok(self
                .current_value
                .lock()
                .expect("mock provider value lock")
                .clone())
        }

        fn set(
            &self,
            path: &RuntimeControlProperty,
            value: ControlValue,
        ) -> Result<(), RuntimeControlError> {
            *self.current_value.lock().expect("mock provider value lock") = value.clone();
            *self.last_set.lock().expect("mock provider lock") =
                Some((path.console_path.clone(), value));
            Ok(())
        }
    }

    #[test]
    fn set_unknown_property_returns_error() {
        let service = RuntimeControlService::default();
        let error = service
            .set("world.weather.rain.front.Emitter.unknown", ControlValue::F64(1.0))
            .expect_err("unknown property should error");
        assert!(matches!(error, RuntimeControlError::UnknownProperty { .. }));
    }

    #[test]
    fn completion_returns_nested_children() {
        let service = RuntimeControlService::default();
        service.register_provider(Arc::new(MockProvider::default()));

        let root = service.complete("world.", "world.".len());
        assert!(root.iter().any(|entry| entry.label == "weather"));

        let nested = service.complete(
            "world.weather.rain.front.Emitter.",
            "world.weather.rain.front.Emitter.".len(),
        );
        assert!(nested.iter().any(|entry| entry.label == "spawn_rate"));
    }

    #[test]
    fn commit_writes_runtime_value_to_source_file() {
        let service = RuntimeControlService::default();
        let provider = Arc::new(MockProvider::default());
        *provider
            .current_value
            .lock()
            .expect("mock provider value lock") = ControlValue::F64(1200.0);
        service.register_provider(provider);
        service.rebuild().expect("registry should build");

        let temp_path = temp_yaml_path("commit");
        fs::write(
            &temp_path,
            "entities:\n  - id: main\n    components:\n      - type: Camera2D\n        exposure:\n          iso: 800\n",
        )
        .expect("temp yaml should be written");

        {
            let mut registry = service.registry.write().expect("registry lock");
            let mut property = registry
                .properties_by_path
                .get("world.weather.rain.front.Emitter.spawn_rate")
                .expect("mock property should exist")
                .clone();
            property.console_path = "world.camera.main.Camera2D.exposure.iso".to_owned();
            property.target_path = "world.camera.main".to_owned();
            property.component = Some("Camera2D".to_owned());
            property.property_path = "exposure.iso".to_owned();
            property.source_file = Some(temp_path.display().to_string());
            property.source_pointer = Some("/entities/0/components/0/exposure/iso".to_owned());
            registry
                .properties_by_path
                .insert(property.console_path.clone(), property);
        }
        service
            .dirty_paths
            .write()
            .expect("dirty lock")
            .insert("world.camera.main.Camera2D.exposure.iso".to_owned());

        service
            .commit("world.camera.main.Camera2D.exposure.iso")
            .expect("commit should succeed");

        let written = fs::read_to_string(&temp_path).expect("temp yaml should be readable");
        assert!(written.contains("1200.0") || written.contains("1200"));
        assert!(
            !service
                .dirty_paths
                .read()
                .expect("dirty lock")
                .contains("world.camera.main.Camera2D.exposure.iso")
        );

        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn component_scope_commit_writes_all_dirty_properties_under_component() {
        let service = RuntimeControlService::default();
        let provider = Arc::new(MockProvider::default());
        *provider
            .current_value
            .lock()
            .expect("mock provider value lock") = ControlValue::F64(1200.0);
        service.register_provider(provider);
        service.rebuild().expect("registry should build");

        let temp_path = temp_yaml_path("component-commit");
        fs::write(
            &temp_path,
            "entities:\n  - id: main\n    components:\n      - type: Camera2D\n        exposure:\n          iso: 800\n",
        )
        .expect("temp yaml should be written");

        {
            let mut registry = service.registry.write().expect("registry lock");
            let mut property = registry
                .properties_by_path
                .get("world.weather.rain.front.Emitter.spawn_rate")
                .expect("mock property should exist")
                .clone();
            property.console_path = "world.camera.main.Camera2D.exposure.iso".to_owned();
            property.target_path = "world.camera.main".to_owned();
            property.component = Some("Camera2D".to_owned());
            property.property_path = "exposure.iso".to_owned();
            property.source_file = Some(temp_path.display().to_string());
            property.source_pointer = Some("/entities/0/components/0/exposure/iso".to_owned());
            registry
                .properties_by_path
                .insert(property.console_path.clone(), property);
        }
        service
            .dirty_paths
            .write()
            .expect("dirty lock")
            .insert("world.camera.main.Camera2D.exposure.iso".to_owned());

        service
            .commit("world.camera.main.Camera2D")
            .expect("component scope commit should succeed");

        let written = fs::read_to_string(&temp_path).expect("temp yaml should be readable");
        assert!(written.contains("1200.0") || written.contains("1200"));

        let _ = fs::remove_file(temp_path);
    }

    fn temp_yaml_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "amigo-runtime-control-{}-{label}.yml",
            std::process::id()
        ))
    }
}
